use crate::{Current, Indicator, InvalidRangeError, Next, Parameter, Price, Range, Reset, Result};
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

    fn _next(&mut self, input: f64) -> <Self as Indicator>::Output {
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
        self.current().unwrap()
    }
}

impl Indicator for Sma {
    type Output = f64;
}
impl Current for Sma {
    fn current(&self) -> Option<Self::Output> {
        self.sum.map(|s| s / self.period as f64)
    }
}
impl Next<f64> for Sma {
    fn next(&mut self, input: f64) -> Self::Output {
        self._next(input)
    }
}
impl<Input: Price> Next<&Input> for Sma {
    fn next(&mut self, input: &Input) -> Self::Output {
        self._next(input.price())
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

    const PERIOD: usize = 5;
    static INPUTS: SyncLazy<Box<[TestItem]>> = SyncLazy::new(|| {
        [100.0, 101.0, 101.0, 102.0, 102.0, 102.0]
            .into_iter()
            .map(TestItem)
            .collect::<Vec<_>>()
            .into_boxed_slice()
    });
    static OUTPUTS: &[f64] = &[100.0, 100.2, 100.4, 100.8, 101.2, 101.6];

    test_indicator! {
        new: Sma::new(PERIOD),
        inputs: INPUTS.iter().map(|x| x.price()),
        outputs: OUTPUTS.iter().copied(),
        additional_tests: {
            new_invalid_parameter: {
                new: Sma::new(0),
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
