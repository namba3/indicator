use crate::{Current, Indicator, InvalidRangeError, Next, Parameter, Price, Range, Reset, Result};
use alloc::collections::VecDeque;

/// Maximum Index (Number of days elapsed from the date of the highest price)
#[derive(Debug, Clone)]
pub struct MaxIndex {
    period: usize,
    ring: VecDeque<f64>,
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
            Ok(Self {
                period,
                ring: VecDeque::with_capacity(period),
                current: None,
            })
        }
    }

    fn _next(&mut self, input: f64) -> <Self as Indicator>::Output {
        match &mut self.current {
            Some(max_index) => {
                let max = self.ring[*max_index];

                let old_value = self.ring.pop_back().unwrap();
                self.ring.push_front(input);

                match max {
                    max if max <= input => *max_index = 0,
                    max if max == old_value => {
                        for (index, x) in self.ring.iter().enumerate() {
                            if self.ring[*max_index] < *x {
                                *max_index = index;
                            }
                        }
                    }
                    _ => *max_index += 1,
                }
            }
            None => {
                for _ in 0..self.period {
                    self.ring.push_back(input);
                }
                self.current = 0.into();
            }
        }
        self.current().unwrap()
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
        self.current = None;
    }
}

#[cfg(test)]
mod tests {
    use std::lazy::SyncLazy;

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
}
