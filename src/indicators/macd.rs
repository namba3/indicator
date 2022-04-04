use crate::operators::Diff;
use crate::{
    Current, Ema, Indicator, InvalidBinaryRelationError, NextExt, Parameter, Price, Reset, Result,
    Sma,
};

/// Moving Average Convergence Divergence
pub struct Macd {
    macd: Diff<Ema, Ema>,
    signal: Sma,
}

impl Macd {
    pub fn new(short_period: usize, long_period: usize, signal_period: usize) -> Result<Self> {
        if !(short_period < long_period) {
            return Err(InvalidBinaryRelationError {
                operator: "<",
                lhs: Parameter::new("short_period", short_period),
                rhs: Parameter::new("long_period", long_period),
            }
            .into());
        }

        Ok(Self {
            macd: Diff::new(Ema::new(short_period)?, Ema::new(long_period)?),
            signal: Sma::new(signal_period)?,
        })
    }

    pub const DEFAULT_SHORT_PERIOD: usize = 12;
    pub const DEFAULT_LONG_PERIOD: usize = 26;
    pub const DEFAULT_SIGNAL_PERIOD: usize = 9;
}
impl Default for Macd {
    fn default() -> Self {
        Self {
            macd: Diff::new(
                Ema::new(Self::DEFAULT_SHORT_PERIOD).unwrap(),
                Ema::new(Self::DEFAULT_LONG_PERIOD).unwrap(),
            ),
            signal: Sma::new(Self::DEFAULT_SIGNAL_PERIOD).unwrap(),
        }
    }
}

impl Indicator for Macd {
    type Input = f64;
    type Output = MacdOutput;
    fn next(&mut self, input: Self::Input) -> Self::Output {
        let _ = self.signal.next(self.macd.next((input, input)));
        self.current().unwrap()
    }
}
impl Current for Macd {
    fn current(&self) -> Option<Self::Output> {
        match (self.macd.current(), self.signal.current()) {
            (Some(macd), Some(signal)) => MacdOutput {
                macd,
                signal,
                histogram: macd - signal,
            }
            .into(),
            _ => None,
        }
    }
}
impl<Input: Price> NextExt<&Input> for Macd {
    fn next_ext(&mut self, input: &Input) -> Self::Output {
        self.next(input.price())
    }
}
impl Reset for Macd {
    fn reset(&mut self) {
        self.macd.reset();
        self.signal.reset();
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MacdOutput {
    macd: f64,
    signal: f64,
    histogram: f64,
}
impl From<(f64, f64, f64)> for MacdOutput {
    fn from((macd, signal, histogram): (f64, f64, f64)) -> Self {
        Self {
            macd,
            signal,
            histogram,
        }
    }
}
impl Into<(f64, f64, f64)> for MacdOutput {
    fn into(self) -> (f64, f64, f64) {
        (self.macd, self.signal, self.histogram)
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

    impl Round for MacdOutput {
        fn round(self) -> Self {
            Self {
                macd: Round::round(self.macd),
                signal: Round::round(self.signal),
                histogram: Round::round(self.histogram),
            }
        }
    }

    const SHORT_PERIOD: usize = 2;
    const LONG_PERIOD: usize = 4;
    const SIGNAL_PERIOD: usize = 2;
    static INPUTS: &[f64] = &[100.0, 200.0, 300.0, 200.0, 100.0, 0.0];
    static OUTPUTS: SyncLazy<Box<[MacdOutput]>> = SyncLazy::new(|| {
        [
            (0.0, 0.0, 0.0),
            (26.66666667, 13.33333333, 13.33333333),
            (51.55555556, 39.11111111, 12.44444444),
            (16.11851852, 33.83703704, -17.71851852),
            (-21.93382716, -2.90765432, -19.02617284),
            (-50.36194239, -36.14788477, -14.214057610),
        ]
        .into_iter()
        .map(MacdOutput::from)
        .collect::<Vec<_>>()
        .into_boxed_slice()
    });

    test_indicator! {
        new: Macd::new(SHORT_PERIOD, LONG_PERIOD, SIGNAL_PERIOD),
        inputs: INPUTS.iter().copied(),
        outputs: OUTPUTS.iter().copied(),
        additional_tests: {
            new_invalid_parameter: {
                news: [
                    Macd::new(0, 2, 2),
                    Macd::new(2, 0, 2),
                    Macd::new(2, 3, 0),
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

    #[test]
    fn default() {
        let _: Macd = Default::default();
    }
}
