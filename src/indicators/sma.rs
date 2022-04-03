use crate::{
    Current, Indicator, InvalidRangeError, NextExt, Parameter, Price, Range, Reset, Result,
};
use alloc::collections::VecDeque;

/// Simple Moving Average
#[derive(Debug, Clone)]
pub struct Sma {
    period: usize,
    ring: VecDeque<f64>,
    sum: Option<f64>,
}
impl Sma {
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
                sum: None,
            })
        }
    }
}

impl Indicator for Sma {
    type Input = f64;
    type Output = f64;
    fn next(&mut self, input: Self::Input) -> Option<Self::Output> {
        match &mut self.sum {
            Some(sum) => {
                *sum -= self.ring.pop_front().unwrap();
                self.ring.push_back(input);
                *sum += input;
            }
            None => {
                for _ in 0..self.period {
                    self.ring.push_back(input);
                }
                self.sum = (input * self.period as f64).into();
            }
        }
        self.current()
    }
}
impl Current for Sma {
    fn current(&self) -> Option<Self::Output> {
        self.sum.map(|s| s / self.period as f64)
    }
}
impl<Input: Price> NextExt<&Input> for Sma {
    fn next_ext(&mut self, input: &Input) -> Option<Self::Output> {
        self.next(input.price())
    }
}
impl Reset for Sma {
    fn reset(&mut self) {
        self.ring.clear();
        self.sum = None;
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

    const PERIOD: usize = 5;
    static INPUTS: &[f64] = &[100.0, 101.0, 101.0, 102.0, 102.0, 102.0];
    static OUTPUTS: SyncLazy<Box<[Option<f64>]>> = SyncLazy::new(|| {
        [100.0, 100.2, 100.4, 100.8, 101.2, 101.6]
            .into_iter()
            .map(Some)
            .collect::<Vec<Option<_>>>()
            .into_boxed_slice()
    });

    test_indicator! {
        new: Sma::new(PERIOD),
        inputs: INPUTS.iter().copied(),
        outputs: OUTPUTS.iter().copied(),
        additional_tests: {
            new_invalid_parameter: {
                new: Sma::new(0),
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
