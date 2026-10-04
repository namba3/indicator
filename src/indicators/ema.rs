use crate::{Current, Indicator, InvalidRangeError, Next, Parameter, Price, Range, Reset, Result};

/// Exponential Moving Average
#[derive(Debug, Clone)]
pub struct Ema {
    period: usize,
    current: Option<f64>,
}
impl Ema {
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
                current: None,
            })
        }
    }

    fn _next(&mut self, input: f64) -> <Self as Indicator>::Output {
        match &mut self.current {
            Some(current) => {
                let alpha = 2.0 / (self.period as f64 + 1.0);
                if self.period == 1 {
                    *current = input;
                } else if current.is_sign_positive() != input.is_sign_positive() {
                    *current = *current * (1.0 - alpha) + input * alpha;
                } else {
                    *current += (input - *current) * alpha;
                }
            }
            None => {
                self.current = input.into();
            }
        }
        self.current().unwrap()
    }
}

impl Indicator for Ema {
    type Output = f64;
}
impl Current for Ema {
    fn current(&self) -> Option<Self::Output> {
        self.current
    }
}
impl Next<f64> for Ema {
    fn next(&mut self, input: f64) -> Self::Output {
        self._next(input)
    }
}
impl<Input: Price> Next<&Input> for Ema {
    fn next(&mut self, input: &Input) -> Self::Output {
        self._next(input.price())
    }
}
impl Reset for Ema {
    fn reset(&mut self) {
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

    const PERIOD: usize = 4;
    static INPUTS: SyncLazy<Box<[TestItem]>> = SyncLazy::new(|| {
        [101.0, 101.0, 101.0, 102.0, 102.0, 102.0]
            .into_iter()
            .map(TestItem)
            .collect::<Vec<_>>()
            .into_boxed_slice()
    });
    static OUTPUTS: &[f64] = &[101.0, 101.0, 101.0, 101.4, 101.64, 101.784];

    test_indicator! {
        new: Ema::new(PERIOD),
        inputs: INPUTS.iter().map(|x| x.price()),
        outputs: OUTPUTS.iter().copied(),
        additional_tests: {
            new_invalid_parameter: {
                new: Ema::new(0),
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
    fn period_one_tracks_each_input() -> crate::Result<()> {
        let mut ema = Ema::new(1)?;

        for input in [3.5, -2.0, 0.0, 9.25] {
            assert_eq!(ema.next(input), input);
        }

        Ok(())
    }

    #[test]
    fn maximum_period_does_not_overflow_the_smoothing_factor() -> crate::Result<()> {
        let mut ema = Ema::new(usize::MAX)?;
        assert_eq!(ema.next(0.0), 0.0);

        let output = ema.next(1.0);
        assert!(output.is_finite());
        assert!(output > 0.0);

        Ok(())
    }

    #[test]
    fn opposite_extreme_finite_inputs_do_not_overflow() -> crate::Result<()> {
        let mut ema = Ema::new(3)?;
        assert_eq!(ema.next(f64::MAX), f64::MAX);
        assert_eq!(ema.next(-f64::MAX), 0.0);
        assert_eq!(ema.next(f64::MAX), f64::MAX / 2.0);

        let mut period_one = Ema::new(1)?;
        assert_eq!(period_one.next(f64::INFINITY), f64::INFINITY);
        assert_eq!(period_one.next(-f64::MAX), -f64::MAX);

        Ok(())
    }
}
