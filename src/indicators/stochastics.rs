use crate::{Current, Indicator, Max, Min, NextExt, Price, Reset, Result, Sma};

/// Stochastics
///
/// Range in \[0.0, 1.0\]
#[derive(Debug, Clone)]
pub struct Stochastics {
    min: Min,
    max: Max,
    d_numerator: Sma,
    d_denominator: Sma,
    current: Option<StochasticsOutput>,
}
impl Stochastics {
    pub fn new(n_period: usize, m_period: usize) -> Result<Self> {
        let min = Min::new(n_period)?;
        let max = Max::new(n_period)?;
        let d_numerator = Sma::new(m_period)?;
        let d_denominator = Sma::new(m_period)?;
        Ok(Self {
            min,
            max,
            d_numerator,
            d_denominator,
            current: None,
        })
    }
}
impl Indicator for Stochastics {
    type Input = f64;
    type Output = StochasticsOutput;
    fn next(&mut self, input: Self::Input) -> Self::Output {
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
            }
            None => {
                let _ = self.d_numerator.next(0.0);
                let _ = self.d_denominator.next(0.0);
                self.current = StochasticsOutput { k: 0.5, d: 0.5 }.into();
            }
        }

        self.current().unwrap()
    }
}
impl Current for Stochastics {
    fn current(&self) -> Option<Self::Output> {
        self.current
    }
}
impl<Input: Price> NextExt<&Input> for Stochastics {
    fn next_ext(&mut self, input: &Input) -> Self::Output {
        self.next(input.price())
    }
}
impl Reset for Stochastics {
    fn reset(&mut self) {
        self.min.reset();
        self.max.reset();
        self.d_numerator.reset();
        self.d_denominator.reset();
        self.current = None;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StochasticsOutput {
    pub k: f64,
    pub d: f64,
}
impl From<(f64, f64)> for StochasticsOutput {
    fn from((k, d): (f64, f64)) -> Self {
        Self { k, d }
    }
}
impl Into<(f64, f64)> for StochasticsOutput {
    fn into(self) -> (f64, f64) {
        (self.k, self.d)
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
    static INPUTS: &[f64] = &[100.0, 101.0, 102.0, 101.0, 100.0, 99.0];
    static OUTPUTS: SyncLazy<Box<[StochasticsOutput]>> = SyncLazy::new(|| {
        [
            (0.5, 0.5),
            (1.0, 1.0),
            (1.0, 1.0),
            (0.5, 0.75),
            (0.0, 0.25),
            (0.0, 0.0),
        ]
        .into_iter()
        .map(StochasticsOutput::from)
        .collect::<Vec<_>>()
        .into_boxed_slice()
    });

    test_indicator! {
        new: Stochastics::new(N_PERIOD, M_PERIOD),
        inputs: INPUTS.iter().copied(),
        outputs: OUTPUTS.iter().copied(),
        additional_tests: {
            new_invalid_parameter: {
                news: [
                    Stochastics::new(0, 1),
                    Stochastics::new(1, 0),
                ],
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
