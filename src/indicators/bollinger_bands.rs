use crate::{
    Current, Indicator, InvalidRangeError, Next, Parameter, Price, Range, Reset, Result,
    StandardDeviation,
};

/// Bollinger Bands
#[derive(Debug, Clone)]
pub struct BollingerBands {
    sd: StandardDeviation,
    multiplier: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BollingerBandsOutput {
    pub average: f64,
    pub upper_bound: f64,
    pub lower_bound: f64,
}

impl BollingerBands {
    /// Creates Bollinger Bands with a finite, non-negative standard deviation multiplier.
    pub fn new(period: usize, multiplier: f64) -> Result<Self> {
        if period < 1 {
            return Err(InvalidRangeError {
                param: Parameter::new("period", period),
                range: Range::LowerBounded { min: 1 },
            }
            .into());
        }
        if !multiplier.is_finite() || multiplier < 0.0 {
            return Err(InvalidRangeError {
                param: Parameter::new("multiplier", multiplier),
                range: Range::BothBounded {
                    min: 0.0,
                    max: f64::MAX,
                },
            }
            .into());
        }

        let sd = StandardDeviation::new(period)?;
        Ok(Self { sd, multiplier })
    }

    fn _next(&mut self, input: f64) -> <Self as Indicator>::Output {
        let _ = self.sd.next(input);
        self.current().unwrap()
    }
}

impl Indicator for BollingerBands {
    type Output = BollingerBandsOutput;
}
impl Current for BollingerBands {
    fn current(&self) -> Option<Self::Output> {
        if let Some(x) = self.sd.current() {
            Self::Output {
                average: x.mean,
                upper_bound: Self::bound(x.mean, x.sd, self.multiplier, true),
                lower_bound: Self::bound(x.mean, x.sd, self.multiplier, false),
            }
            .into()
        } else {
            None
        }
    }
}
impl BollingerBands {
    fn bound(mean: f64, sd: f64, multiplier: f64, upper: bool) -> f64 {
        let deviation = sd * multiplier;
        let bound = if upper {
            mean + deviation
        } else {
            mean - deviation
        };
        if bound.is_finite() || !mean.is_finite() || !sd.is_finite() {
            return bound;
        }

        let scale = mean.abs().max(sd.abs());
        if scale == 0.0 {
            return mean;
        }

        let scaled_mean = mean / scale;
        let scaled_deviation = (sd / scale) * multiplier;
        let scaled_bound = if upper {
            scaled_mean + scaled_deviation
        } else {
            scaled_mean - scaled_deviation
        };
        scaled_bound * scale
    }
}
impl Next<f64> for BollingerBands {
    fn next(&mut self, input: f64) -> Self::Output {
        self._next(input)
    }
}
impl<Input: Price> Next<&Input> for BollingerBands {
    fn next(&mut self, input: &Input) -> Self::Output {
        self._next(input.price())
    }
}
impl Reset for BollingerBands {
    fn reset(&mut self) {
        self.sd.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::*;
    use std::sync::LazyLock as SyncLazy;

    #[derive(Clone)]
    struct TestItem(f64);
    impl Price for TestItem {
        fn price(&self) -> f64 {
            self.0
        }
    }

    impl Round for BollingerBandsOutput {
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
    static OUTPUTS: SyncLazy<Box<[BollingerBandsOutput]>> = SyncLazy::new(|| {
        [
            (100.0, 100.0, 100.0),
            (100.8, 104.0, 97.6),
            (101.2, 104.4, 98.0),
            (101.6, 104.59332591, 98.60667409),
        ]
        .into_iter()
        .map(|(average, upper_bound, lower_bound)| BollingerBandsOutput {
            average,
            upper_bound,
            lower_bound,
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
    });

    test_indicator! {
        new: BollingerBands::new(PERIOD, MULTIPLIER),
        inputs: INPUTS.iter().map(|x| x.price()),
        outputs: OUTPUTS.iter().copied(),
        additional_tests: {
            new_invalid_parameter: {
                new: BollingerBands::new(10, -1.0),
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
    fn finite_band_bound_survives_intermediate_product_overflow() -> crate::Result<()> {
        let mut bands = BollingerBands::new(2, 3.0)?;
        let _ = bands.next(-f64::MAX);

        let output = bands.next(0.0);
        assert_eq!(output.average, -f64::MAX / 2.0);
        assert_eq!(output.upper_bound, f64::MAX);
        assert_eq!(output.lower_bound, f64::NEG_INFINITY);

        Ok(())
    }

    #[test]
    fn validates_multiplier_before_allocating_the_rolling_buffer() {
        assert!(matches!(
            BollingerBands::new(usize::MAX, f64::NAN),
            Err(crate::Error::InvalidFloatRange(_))
        ));
        assert!(matches!(
            BollingerBands::new(usize::MAX, -1.0),
            Err(crate::Error::InvalidFloatRange(_))
        ));
        assert!(matches!(
            BollingerBands::new(0, -1.0),
            Err(crate::Error::InvalidUintRange(_))
        ));
    }
}
