use crate::{Current, Indicator, InvalidRangeError, Next, Parameter, Price, Range, Reset, Result};
use alloc::collections::VecDeque;

/// Minimum
#[derive(Debug, Clone)]
pub struct Min {
    period: usize,
    ring: VecDeque<f64>,
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
            Ok(Self {
                period,
                ring: VecDeque::with_capacity(period),
                current: None,
            })
        }
    }

    fn _next(&mut self, input: f64) -> <Self as Indicator>::Output {
        match &mut self.current {
            Some(min) => {
                let old_val = self.ring.pop_front().unwrap();
                self.ring.push_back(input);

                match min {
                    min if input <= *min => *min = input,
                    min if *min == old_val => {
                        *min = self
                            .ring
                            .iter()
                            .copied()
                            .reduce(|acc, x| acc.min(x))
                            .unwrap();
                    }
                    _ => (),
                }
            }
            None => {
                for _ in 0..self.period {
                    self.ring.push_back(input);
                }
                self.current = input.into();
            }
        }
        self.current().unwrap()
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
        self.current = None;
    }
}

#[cfg(test)]
mod tests {
    use once_cell::sync::Lazy as SyncLazy;

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
}
