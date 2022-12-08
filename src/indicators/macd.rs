use crate::operators::Diff;
use crate::{
    Current, Ema, Indicator, InvalidBinaryRelationError, Next, Parameter, Price, Reset, Result, Sma,
};

/// Moving Average Convergence Divergence
pub struct Macd {
    macd: Diff<Ema, Ema>,
    signal: Sma,
}

impl Macd {
    pub const DEFAULT_SHORT_PERIOD: usize = 12;
    pub const DEFAULT_LONG_PERIOD: usize = 26;
    pub const DEFAULT_SIGNAL_PERIOD: usize = 9;

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

    fn _next(&mut self, input: f64) -> <Self as Indicator>::Output {
        let _ = self.signal.next(self.macd.next((input, input)));
        self.current().unwrap()
    }
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
    type Output = MacdOutput;
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
impl Next<f64> for Macd {
    fn next(&mut self, input: f64) -> Self::Output {
        self._next(input)
    }
}
impl<Input: Price> Next<&Input> for Macd {
    fn next(&mut self, input: &Input) -> Self::Output {
        self._next(input.price())
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
    pub macd: f64,
    pub signal: f64,
    pub histogram: f64,
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
    use once_cell::sync::Lazy as SyncLazy;

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
    static INPUTS: SyncLazy<Box<[TestItem]>> = SyncLazy::new(|| {
        [100.0, 200.0, 300.0, 200.0, 100.0, 0.0]
            .into_iter()
            .map(TestItem)
            .collect::<Vec<_>>()
            .into_boxed_slice()
    });
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
        inputs: INPUTS.iter().map(|x| x.price()),
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
        let _: Macd = Default::default();
    }
}
