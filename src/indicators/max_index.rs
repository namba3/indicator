use crate::{
    Current, Indicator, InvalidRangeError, Next, Parameter, Price, Range, Reset, Result,
    try_deque_with_capacity,
};
use alloc::collections::VecDeque;

/// Maximum Index (number of samples elapsed since the highest value)
#[derive(Debug, Clone)]
pub struct MaxIndex {
    period: usize,
    ring: VecDeque<f64>,
    candidates: VecDeque<(usize, f64)>,
    position: usize,
    nan_count: usize,
    fast_path: bool,
    current: Option<usize>,
}
impl MaxIndex {
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
                self.ring.push_front(input);
            }
            self.position = self.period - 1;
            self.nan_count = if input.is_nan() { self.period } else { 0 };
            self.fast_path = self.nan_count == 0;
            self.candidates.clear();
            if self.fast_path {
                self.candidates.push_back((self.position, input));
            }
            self.current = Some(0);
            return 0;
        }

        self.position += 1;
        let old_index = self.current.unwrap().min(self.period - 1);
        let old_max = self.ring[old_index];
        let old_value = self.ring.pop_back().unwrap();
        self.ring.push_front(input);
        self.nan_count -= usize::from(old_value.is_nan());
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
                .is_some_and(|(_, value)| *value <= input)
            {
                self.candidates.pop_back();
            }
            self.candidates.push_back((self.position, input));
            self.current = Some(self.position - self.candidates.front().unwrap().0);
        } else {
            let max_index = if old_max <= input {
                0
            } else if old_max == old_value {
                let mut latest_max_index = 0;
                for (index, value) in self.ring.iter().enumerate().skip(1) {
                    if *value > self.ring[latest_max_index] {
                        latest_max_index = index;
                    }
                }
                latest_max_index
            } else {
                old_index.saturating_add(1).min(self.period - 1)
            };
            self.current = Some(max_index);

            if self.nan_count == 0 {
                self.rebuild_candidates();
                self.fast_path = true;
            }
        }
        self.current.unwrap()
    }

    fn rebuild_candidates(&mut self) {
        self.candidates.clear();
        let oldest_position = self.position - (self.period - 1);
        for (offset, value) in self.ring.iter().rev().copied().enumerate() {
            let position = oldest_position + offset;
            while self
                .candidates
                .back()
                .is_some_and(|(_, candidate)| *candidate <= value)
            {
                self.candidates.pop_back();
            }
            self.candidates.push_back((position, value));
        }
        self.current = Some(self.position - self.candidates.front().unwrap().0);
    }
}

impl Indicator for MaxIndex {
    type Output = usize;
}
impl Current for MaxIndex {
    fn current(&self) -> Option<Self::Output> {
        self.current
    }
}
impl Next<f64> for MaxIndex {
    fn next(&mut self, input: f64) -> Self::Output {
        self._next(input)
    }
}
impl<Input: Price> Next<&Input> for MaxIndex {
    fn next(&mut self, input: &Input) -> Self::Output {
        self._next(input.price())
    }
}
impl Reset for MaxIndex {
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
    static OUTPUTS: &[usize] = &[0, 0, 0, 1, 1, 0];

    test_indicator! {
        new: MaxIndex::new(PERIOD),
        inputs: INPUTS.iter().map(|x| x.price()),
        outputs: OUTPUTS.iter().copied(),
        additional_tests: {
            new_invalid_parameter: {
                new: MaxIndex::new(0),
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
    fn rolling_index_tracks_latest_maximum_through_ties_and_expiry() -> crate::Result<()> {
        const PERIOD: usize = 4;
        let inputs = [3.0, 5.0, 5.0, 2.0, 1.0, 5.0, 4.0, 5.0, 1.0, 0.0, 0.0, 2.0];
        let mut indicator = MaxIndex::new(PERIOD)?;
        let mut window = vec![inputs[0]; PERIOD];

        for (position, input) in inputs.into_iter().enumerate() {
            if position > 0 {
                window.remove(0);
                window.push(input);
            }

            let maximum = window.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let expected = window
                .iter()
                .rev()
                .position(|value| *value == maximum)
                .unwrap();
            assert_eq!(indicator.next(input), expected);
        }

        Ok(())
    }

    #[test]
    fn nan_values_do_not_panic_and_extrema_recover_after_expiry() -> crate::Result<()> {
        let mut indicator = MaxIndex::new(3)?;

        for (input, expected) in [(f64::NAN, 0), (5.0, 1), (4.0, 2), (6.0, 0), (3.0, 1)] {
            assert_eq!(indicator.next(input), expected);
        }

        Ok(())
    }
}
