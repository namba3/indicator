use crate::{
    Current, Indicator, InvalidRangeError, Next, Parameter, Price, Range, Reset, Result,
    try_deque_with_capacity,
};
use alloc::collections::VecDeque;

/// Minimum
#[derive(Debug, Clone)]
pub struct Min {
    period: usize,
    ring: VecDeque<f64>,
    candidates: VecDeque<(usize, f64)>,
    position: usize,
    nan_count: usize,
    fast_path: bool,
    current: Option<f64>,
}
impl Min {
    pub fn new(period: usize) -> Result<Self> {
        if period < 1 {
            Err(InvalidRangeError {
                param: Parameter::new("period", period),
                range: Range::LowerBounded { min: 1 },
            }
            .into())
        } else {
            let ring = try_deque_with_capacity(period)?;
            let candidates = try_deque_with_capacity(period)?;
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
            for _ in 0..self.period {
                self.ring.push_back(input);
            }
            self.position = self.period - 1;
            self.nan_count = if input.is_nan() { self.period } else { 0 };
            self.fast_path = self.nan_count == 0;
            self.candidates.clear();
            if self.fast_path {
                self.candidates.push_back((self.position, input));
            }
            self.current = Some(input);
            return input;
        }

        self.position += 1;
        let old_val = self.ring.pop_front().unwrap();
        self.ring.push_back(input);
        self.nan_count -= usize::from(old_val.is_nan());
        self.nan_count += usize::from(input.is_nan());

        if self.fast_path && self.nan_count == 0 {
            while self
                .candidates
                .front()
                .is_some_and(|(position, _)| self.position - position >= self.period)
            {
                self.candidates.pop_front();
            }
            while self
                .candidates
                .back()
                .is_some_and(|(_, value)| *value >= input)
            {
                self.candidates.pop_back();
            }
            self.candidates.push_back((self.position, input));
            self.current = Some(self.candidates.front().unwrap().1);
        } else {
            let old_min = self.current.unwrap();
            let min = if input <= old_min {
                input
            } else if old_min == old_val {
                self.ring
                    .iter()
                    .copied()
                    .reduce(|acc, x| acc.min(x))
                    .unwrap()
            } else {
                old_min
            };
            self.current = Some(min);

            if self.nan_count == 0 && !min.is_nan() {
                self.rebuild_candidates();
                self.fast_path = true;
            }
        }
        self.current.unwrap()
    }

    fn rebuild_candidates(&mut self) {
        self.candidates.clear();
        let oldest_position = self.position - (self.period - 1);
        for (offset, value) in self.ring.iter().copied().enumerate() {
            let position = oldest_position + offset;
            while self
                .candidates
                .back()
                .is_some_and(|(_, candidate)| *candidate >= value)
            {
                self.candidates.pop_back();
            }
            self.candidates.push_back((position, value));
        }
        self.current = Some(self.candidates.front().unwrap().1);
    }
}

impl Indicator for Min {
    type Output = f64;
}
impl Current for Min {
    fn current(&self) -> Option<Self::Output> {
        self.current
    }
}
impl Next<f64> for Min {
    fn next(&mut self, input: f64) -> Self::Output {
        self._next(input)
    }
}
impl<Input: Price> Next<&Input> for Min {
    fn next(&mut self, input: &Input) -> Self::Output {
        self._next(input.price())
    }
}
impl Reset for Min {
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
    static OUTPUTS: &[f64] = &[6.0, 6.0, 7.0, 3.0, 2.0, 2.0];

    test_indicator! {
        new: Min::new(PERIOD),
        inputs: INPUTS.iter().map(|x| x.price()),
        outputs: OUTPUTS.iter().copied(),
        additional_tests: {
            new_invalid_parameter: {
                new: Min::new(0),
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
        let mut min = Min::new(3).unwrap();
        let inputs = [1.0, 6.0, f64::NAN, 5.0, 4.0, 3.0, 2.0, 0.0];
        let outputs = [1.0, 1.0, 1.0, 5.0, 4.0, 3.0, 2.0, 0.0];

        for (input, expected) in inputs.into_iter().zip(outputs) {
            assert_eq!(min.next(input), expected);
        }
    }
}
