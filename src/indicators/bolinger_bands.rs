use crate::{
    Current, Indicator, InvalidRangeError, Next, Parameter, Price, Range, Reset, Result,
    StandardDeviation,
};

/// Bolinger Bands
#[derive(Debug, Clone)]
pub struct BolingerBands {
    sd: StandardDeviation,
    multiplier: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BolingerBandsOutput {
    pub average: f64,
    pub upper_bound: f64,
    pub lower_bound: f64,
}

impl BolingerBands {
    pub fn new(period: usize, multiplier: f64) -> Result<Self> {
        let sd = StandardDeviation::new(period)?;
        if multiplier < 0.0 {
            Err(InvalidRangeError {
                param: Parameter::new("multiplier", multiplier),
                range: Range::LowerBounded { min: 0.0 },
            }
            .into())
        } else {
            Ok(Self {
                sd,
                multiplier: multiplier as f64,
            })
        }
    }

    fn _next(&mut self, input: f64) -> <Self as Indicator>::Output {
        let _ = self.sd.next(input);
        self.current().unwrap()
    }
}

impl Indicator for BolingerBands {
    type Output = BolingerBandsOutput;
}
impl Current for BolingerBands {
    fn current(&self) -> Option<Self::Output> {
        if let Some(x) = self.sd.current() {
            Self::Output {
                average: x.mean,
                upper_bound: x.mean + x.sd * self.multiplier,
                lower_bound: x.mean - x.sd * self.multiplier,
            }
            .into()
        } else {
            None
        }
    }
}
impl Next<f64> for BolingerBands {
    fn next(&mut self, input: f64) -> Self::Output {
        self._next(input)
    }
}
impl<Input: Price> Next<&Input> for BolingerBands {
    fn next(&mut self, input: &Input) -> Self::Output {
        self._next(input.price())
    }
}
impl Reset for BolingerBands {
    fn reset(&mut self) {
        self.sd.reset();
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

    impl Round for BolingerBandsOutput {
        fn round(self) -> Self {
            Self {
                average: Round::round(self.average),
                upper_bound: Round::round(self.upper_bound),
                lower_bound: Round::round(self.lower_bound),
            }
        }
    }

    const PERIOD: usize = 5;
    const MULTIPLIER: f64 = 2.0;
    static INPUTS: SyncLazy<Box<[TestItem]>> = SyncLazy::new(|| {
        [100.0, 104.0, 102.0, 102.0]
            .into_iter()
            .map(TestItem)
            .collect::<Vec<_>>()
            .into_boxed_slice()
    });
    static OUTPUTS: SyncLazy<Box<[BolingerBandsOutput]>> = SyncLazy::new(|| {
        [
            (100.0, 100.0, 100.0),
            (100.8, 104.0, 97.6),
            (101.2, 104.4, 98.0),
            (101.6, 104.59332591, 98.60667409),
        ]
        .into_iter()
        .map(|(average, upper_bound, lower_bound)| BolingerBandsOutput {
            average,
            upper_bound,
            lower_bound,
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
    });

    test_indicator! {
        new: BolingerBands::new(PERIOD, MULTIPLIER),
        inputs: INPUTS.iter().map(|x| x.price()),
        outputs: OUTPUTS.iter().copied(),
        additional_tests: {
            new_invalid_parameter: {
                new: BolingerBands::new(10, -1.0),
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
