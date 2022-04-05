use crate::{Current, Indicator, Max, Min, Next, Price, Reset, Result, Sma};

/// Stochastics
///
/// Range in \[0.0, 1.0\]
#[derive(Debug, Clone)]
pub struct Stochastics {
    min: Min,
    max: Max,
    d_numerator: Sma,
    d_denominator: Sma,
    slow_d: Sma,
    current: Option<StochasticsOutput>,
}
impl Stochastics {
    pub const DEFAULT_N_PERIOD: usize = 14;
    pub const DEFAULT_M_PERIOD: usize = 3;
    pub const DEFAULT_X_PERIOD: usize = 3;

    ///
    pub fn new(n_period: usize, m_period: usize, x_period: usize) -> Result<Self> {
        let min = Min::new(n_period)?;
        let max = Max::new(n_period)?;
        let d_numerator = Sma::new(m_period)?;
        let d_denominator = Sma::new(m_period)?;
        let slow_d = Sma::new(x_period)?;
        Ok(Self {
            min,
            max,
            d_numerator,
            d_denominator,
            slow_d,
            current: None,
        })
    }

    fn _next(&mut self, input: f64) -> <Self as Indicator>::Output {
        let min = self.min.next(input);
        let max = self.max.next(input);

        match &mut self.current {
            Some(current) => {
                let d_numerator = self.d_numerator.next(input - min);
                let d_denominator = self.d_denominator.next(max - min);
                current.k = if min == max {
                    0.5
                } else {
                    (input - min) / (max - min)
                };
                current.d = if d_denominator == 0.0 {
                    0.5
                } else {
                    d_numerator / d_denominator
                };
                current.slow_d = self.slow_d.next(current.d);
            }
            None => {
                let _ = self.d_numerator.next(0.0);
                let _ = self.d_denominator.next(0.0);
                let _ = self.slow_d.next(0.0);
                self.current = StochasticsOutput {
                    k: 0.5,
                    d: 0.5,
                    slow_d: 0.5,
                }
                .into();
            }
        }

        self.current().unwrap()
    }
}
impl Default for Stochastics {
    fn default() -> Self {
        Self {
            min: Min::new(Self::DEFAULT_N_PERIOD).unwrap(),
            max: Max::new(Self::DEFAULT_N_PERIOD).unwrap(),
            d_numerator: Sma::new(Self::DEFAULT_M_PERIOD).unwrap(),
            d_denominator: Sma::new(Self::DEFAULT_M_PERIOD).unwrap(),
            slow_d: Sma::new(Self::DEFAULT_X_PERIOD).unwrap(),
            current: None,
        }
    }
}
impl Indicator for Stochastics {
    type Output = StochasticsOutput;
}
impl Current for Stochastics {
    fn current(&self) -> Option<Self::Output> {
        self.current
    }
}
impl Next<f64> for Stochastics {
    fn next(&mut self, input: f64) -> Self::Output {
        self._next(input)
    }
}
impl<Input: Price> Next<&Input> for Stochastics {
    fn next(&mut self, input: &Input) -> Self::Output {
        self._next(input.price())
    }
}
impl Reset for Stochastics {
    fn reset(&mut self) {
        self.min.reset();
        self.max.reset();
        self.d_numerator.reset();
        self.d_denominator.reset();
        self.slow_d.reset();
        self.current = None;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StochasticsOutput {
    pub k: f64,
    pub d: f64,
    pub slow_d: f64,
}
impl From<(f64, f64, f64)> for StochasticsOutput {
    fn from((k, d, slow_d): (f64, f64, f64)) -> Self {
        Self { k, d, slow_d }
    }
}
impl Into<(f64, f64, f64)> for StochasticsOutput {
    fn into(self) -> (f64, f64, f64) {
        (self.k, self.d, self.slow_d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::*;
    use std::lazy::SyncLazy;

    impl Round for StochasticsOutput {
        fn round(self) -> Self {
            Self {
                k: Round::round(self.k),
                d: Round::round(self.d),
                slow_d: Round::round(self.slow_d),
            }
        }
    }

    #[derive(Clone)]
    struct TestItem(f64);
    impl Price for TestItem {
        fn price(&self) -> f64 {
            self.0
        }
    }

    const N_PERIOD: usize = 4;
    const M_PERIOD: usize = 2;
    const X_PERIOD: usize = 2;
    static INPUTS: SyncLazy<Box<[TestItem]>> = SyncLazy::new(|| {
        [100.0, 101.0, 102.0, 101.0, 100.0, 99.0]
            .into_iter()
            .map(TestItem)
            .collect::<Vec<_>>()
            .into_boxed_slice()
    });
    static OUTPUTS: SyncLazy<Box<[StochasticsOutput]>> = SyncLazy::new(|| {
        [
            (0.5, 0.5, 0.5),
            (1.0, 1.0, 0.5),
            (1.0, 1.0, 1.0),
            (0.5, 0.75, 0.875),
            (0.0, 0.25, 0.5),
            (0.0, 0.0, 0.125),
        ]
        .into_iter()
        .map(StochasticsOutput::from)
        .collect::<Vec<_>>()
        .into_boxed_slice()
    });

    test_indicator! {
        new: Stochastics::new(N_PERIOD, M_PERIOD, X_PERIOD),
        inputs: INPUTS.iter().map(|x| x.price()),
        outputs: OUTPUTS.iter().copied(),
        additional_tests: {
            new_invalid_parameter: {
                news: [
                    Stochastics::new(0, 1, 1),
                    Stochastics::new(1, 0, 1),
                    Stochastics::new(1, 1, 0),
                ],
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
    fn default() {
        let _: Stochastics = Default::default();
    }
}
