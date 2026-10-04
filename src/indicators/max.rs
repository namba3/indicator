use super::padded_window::values as padded_values;
use super::rolling_candidates::{Direction, RollingCandidates};
use crate::{
    Current, Indicator, InvalidRangeError, Next, Parameter, Price, Range, Reset, Result,
    try_deque_with_capacity,
};
use alloc::collections::VecDeque;

/// Maximum
#[derive(Debug, Clone)]
pub struct Max {
    period: usize,
    ring: VecDeque<f64>,
    candidates: RollingCandidates,
    position: usize,
    nan_count: usize,
    fast_path: bool,
    current: Option<f64>,
}
impl Max {
    pub fn new(period: usize) -> Result<Self> {
        if period < 1 {
            Err(InvalidRangeError {
                param: Parameter::new("period", period),
                range: Range::LowerBounded { min: 1 },
            }
            .into())
        } else {
            let ring = try_deque_with_capacity(period)?;
            let candidates = RollingCandidates::new(Direction::Maximum, period)?;
            Ok(Self {
                period,
                ring,
                candidates,
                position: 0,
                nan_count: 0,
                fast_path: true,
                current: None,
            })
        }
    }

    fn _next(&mut self, input: f64) -> <Self as Indicator>::Output {
        if self.current.is_none() {
            self.ring.push_back(input);
            self.position = self.period - 1;
            self.nan_count = if input.is_nan() { self.period } else { 0 };
            self.fast_path = self.nan_count == 0;
            self.candidates.clear();
            if self.fast_path {
                self.candidates.push(self.position, input);
            }
            self.current = Some(input);
            return input;
        }

        self.position = self.position.wrapping_add(1);
        let old_val = if self.ring.len() < self.period {
            *self.ring.front().unwrap()
        } else {
            self.ring.pop_front().unwrap()
        };
        self.ring.push_back(input);
        self.nan_count -= usize::from(old_val.is_nan());
        self.nan_count += usize::from(input.is_nan());

        if self.fast_path && self.nan_count == 0 {
            self.candidates.expire(self.position, self.period);
            self.candidates.push(self.position, input);
            self.current = Some(self.candidates.front().unwrap().1);
        } else {
            let old_max = self.current.unwrap();
            let max = if old_max <= input {
                input
            } else if old_max == old_val {
                padded_values(&self.ring, self.period)
                    .reduce(|acc, x| acc.max(x))
                    .unwrap()
            } else {
                old_max
            };
            self.current = Some(max);

            if self.nan_count == 0 && !max.is_nan() {
                self.rebuild_candidates();
                self.fast_path = true;
            }
        }
        self.current.unwrap()
    }

    fn rebuild_candidates(&mut self) {
        self.candidates.clear();
        let oldest_position = self.position.wrapping_sub(self.period - 1);
        for (offset, value) in padded_values(&self.ring, self.period).enumerate() {
            let position = oldest_position.wrapping_add(offset);
            self.candidates.push(position, value);
        }
        self.current = Some(self.candidates.front().unwrap().1);
    }
}

impl Indicator for Max {
    type Output = f64;
}
impl Current for Max {
    fn current(&self) -> Option<Self::Output> {
        self.current
    }
}
impl Next<f64> for Max {
    fn next(&mut self, input: f64) -> Self::Output {
        self._next(input)
    }
}
impl<Input: Price> Next<&Input> for Max {
    fn next(&mut self, input: &Input) -> Self::Output {
        self._next(input.price())
    }
}
impl Reset for Max {
    fn reset(&mut self) {
        self.ring.clear();
        self.candidates.clear();
        self.position = 0;
        self.nan_count = 0;
        self.fast_path = true;
        self.current = None;
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock as SyncLazy;

    use super::*;
    use crate::test_helper::*;

    #[derive(Clone)]
    struct TestItem(f64);
    impl Price for TestItem {
        fn price(&self) -> f64 {
            self.0
        }
    }

    const PERIOD: usize = 2;
    static INPUTS: SyncLazy<Box<[TestItem]>> = SyncLazy::new(|| {
        [6.0, 7.0, 8.0, 3.0, 2.0, 4.0]
            .into_iter()
            .map(TestItem)
            .collect::<Vec<_>>()
            .into_boxed_slice()
    });
    static OUTPUTS: &[f64] = &[6.0, 7.0, 8.0, 8.0, 3.0, 4.0];

    test_indicator! {
        new: Max::new(PERIOD),
        inputs: INPUTS.iter().map(|x| x.price()),
        outputs: OUTPUTS.iter().copied(),
        additional_tests: {
            new_invalid_parameter: {
                new: Max::new(0),
            },
            current: {
                inputs: RANDOM_DATA.iter().map(|x| x.price()),
            },
            next_ext: {
                inputs: INPUTS.iter(),
                outputs: OUTPUTS.iter().copied(),
            },
            reset: {
                inputs: RANDOM_DATA.iter().map(|x| x.price()),
            },
        }
    }

    #[test]
    fn nan_in_window_preserves_results_and_rebuilds_the_fast_queue_after_expiry() {
        let mut max = Max::new(3).unwrap();
        let inputs = [9.0, 4.0, f64::NAN, 5.0, 6.0, 7.0, 8.0];
        let outputs = [9.0, 9.0, 9.0, 5.0, 6.0, 7.0, 8.0];

        for (input, expected) in inputs.into_iter().zip(outputs) {
            assert_eq!(max.next(input), expected);
        }
    }

    #[test]
    fn first_input_is_stored_once_and_nan_fallback_sees_virtual_padding() -> crate::Result<()> {
        let mut max = Max::new(4)?;
        assert_eq!(max.next(9.0), 9.0);
        assert_eq!(max.ring.iter().copied().collect::<Vec<_>>(), [9.0]);

        assert_eq!(max.next(f64::NAN), 9.0);
        assert_eq!(max.ring[0], 9.0);
        assert!(max.ring[1].is_nan());
        assert_eq!(max.next(5.0), 9.0);
        assert_eq!(max.next(4.0), 9.0);
        assert_eq!(max.next(3.0), 5.0);
        assert!(max.ring[0].is_nan());
        assert_eq!(
            max.ring.iter().copied().skip(1).collect::<Vec<_>>(),
            [5.0, 4.0, 3.0]
        );

        Ok(())
    }

    #[test]
    fn rolling_position_wraps_without_changing_the_maximum() -> crate::Result<()> {
        let mut max = Max::new(3)?;
        assert_eq!(max.next(1.0), 1.0);
        max.position = usize::MAX;
        max.fast_path = false;
        max.candidates.clear();
        max.candidates.push(usize::MAX, 1.0);

        assert_eq!(max.next(0.0), 1.0);
        assert_eq!(max.next(2.0), 2.0);

        Ok(())
    }
}
