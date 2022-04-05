use crate::{Current, Indicator, InvalidRangeError, Next, Parameter, Price, Range, Reset, Result};
use alloc::collections::VecDeque;

/// Standard Deviation
#[derive(Debug, Clone)]
pub struct StandardDeviation {
    period: usize,
    ring: VecDeque<f64>,
    mean_sse: Option<(f64, f64)>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StandardDeviationOutput {
    pub mean: f64,
    pub sd: f64,
}

impl StandardDeviation {
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
                mean_sse: None,
            })
        }
    }

    fn _next(&mut self, input: f64) -> <Self as Indicator>::Output {
        match &mut self.mean_sse {
            Some((mean, sse)) => {
                let old_input = self.ring.pop_front().unwrap();
                self.ring.push_back(input);

                let delta = input - old_input;
                let old_mean = *mean;
                *mean += delta / self.period as f64;
                let delta2 = input - *mean + old_input - old_mean;
                *sse += delta * delta2;
            }
            None => {
                for _ in 0..self.period {
                    self.ring.push_back(input);
                }
                self.mean_sse = (input, 0.0).into();
            }
        }
        self.current().unwrap()
    }
}

impl Indicator for StandardDeviation {
    type Output = StandardDeviationOutput;
}
impl Current for StandardDeviation {
    fn current(&self) -> Option<Self::Output> {
        if let Some((mean, sse)) = self.mean_sse {
            Self::Output {
                mean,
                sd: (sse / self.period as f64).sqrt(),
            }
            .into()
        } else {
            None
        }
    }
}
impl Next<f64> for StandardDeviation {
    fn next(&mut self, input: f64) -> Self::Output {
        self._next(input)
    }
}
impl<Input: Price> Next<&Input> for StandardDeviation {
    fn next(&mut self, input: &Input) -> Self::Output {
        self._next(input.price())
    }
}
impl Reset for StandardDeviation {
    fn reset(&mut self) {
        self.ring.clear();
        self.mean_sse = None;
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

    impl Round for StandardDeviationOutput {
        fn round(self) -> Self {
            Self {
                mean: Round::round(self.mean),
                sd: Round::round(self.sd),
            }
        }
    }

    const PERIOD: usize = 5;
    static INPUTS: SyncLazy<Box<[TestItem]>> = SyncLazy::new(|| {
        [100.0, 104.0, 102.0, 102.0]
            .into_iter()
            .map(TestItem)
            .collect::<Vec<_>>()
            .into_boxed_slice()
    });
    static OUTPUTS: SyncLazy<Box<[StandardDeviationOutput]>> = SyncLazy::new(|| {
        [
            (100.0, 0.0),
            (100.8, 1.6),
            (101.2, 1.6),
            (101.6, 1.49666295),
        ]
        .into_iter()
        .map(|(mean, sd)| StandardDeviationOutput { mean, sd })
        .collect::<Vec<_>>()
        .into_boxed_slice()
    });

    test_indicator! {
        new: StandardDeviation::new(PERIOD),
        inputs: INPUTS.iter().map(|x| x.price()),
        outputs: OUTPUTS.iter().copied(),
        additional_tests: {
            new_invalid_parameter: {
                new: StandardDeviation::new(0),
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
        },
    }
}
