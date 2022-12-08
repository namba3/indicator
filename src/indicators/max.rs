use crate::{Current, Indicator, InvalidRangeError, Next, Parameter, Price, Range, Reset, Result};
use alloc::collections::VecDeque;

/// Maximum
#[derive(Debug, Clone)]
pub struct Max {
    period: usize,
    ring: VecDeque<f64>,
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
            Ok(Self {
                period,
                ring: VecDeque::with_capacity(period),
                current: None,
            })
        }
    }

    fn _next(&mut self, input: f64) -> <Self as Indicator>::Output {
        match &mut self.current {
            Some(max) => {
                let old_val = self.ring.pop_front().unwrap();
                self.ring.push_back(input);

                match max {
                    max if *max <= input => *max = input,
                    max if *max == old_val => {
                        *max = self
                            .ring
                            .iter()
                            .copied()
                            .reduce(|acc, x| acc.max(x))
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
}
