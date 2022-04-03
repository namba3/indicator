use crate::{
    Current, Indicator, InvalidRangeError, NextExt, Parameter, Price, Range, Reset, Result,
};
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
}

impl Indicator for Max {
    type Input = f64;
    type Output = f64;
    fn next(&mut self, input: Self::Input) -> Option<Self::Output> {
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
        self.current()
    }
}
impl Current for Max {
    fn current(&self) -> Option<Self::Output> {
        self.current
    }
}
impl<Input: Price> NextExt<&Input> for Max {
    fn next_ext(&mut self, input: &Input) -> Option<Self::Output> {
        self.next(input.price())
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
    use super::*;
    use crate::test_helper::*;
    use std::lazy::SyncLazy;

    #[derive(Clone)]
    struct TestItem(f64);
    impl Price for TestItem {
        fn price(&self) -> f64 {
            self.0
        }
    }

    const PERIOD: usize = 2;
    static INPUTS: &[f64] = &[6.0, 7.0, 8.0, 3.0, 2.0, 4.0];
    static OUTPUTS: SyncLazy<Box<[Option<f64>]>> = SyncLazy::new(|| {
        [6.0, 7.0, 8.0, 8.0, 3.0, 4.0]
            .into_iter()
            .map(Some)
            .collect::<Vec<Option<_>>>()
            .into_boxed_slice()
    });

    test_indicator! {
        new: Max::new(PERIOD),
        inputs: INPUTS.iter().copied(),
        outputs: OUTPUTS.iter().copied(),
        additional_tests: {
            new_invalid_parameter: {
                new: Max::new(0),
            },
            current: {
                inputs: RANDOM_DATA.iter().map(|x| x.price()),
            },
            next_ext: {
                inputs: INPUTS.iter().map(|x| TestItem(*x)),
                outputs: OUTPUTS.iter().copied(),
            },
            reset: {
                inputs: RANDOM_DATA.iter().map(|x| x.price()),
            },
        }
    }
}
