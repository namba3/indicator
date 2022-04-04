use crate::{
    Current, Indicator, InvalidRangeError, MaxIndex, MinIndex, NextExt, Parameter, Price, Range,
    Reset, Result,
};

/// Aroon Indicator
/// ///
/// Range in \[0.0, 1.0\]
#[derive(Debug, Clone)]
pub struct AroonIndicator {
    period: usize,
    max_index: MaxIndex,
    min_index: MinIndex,
}
impl AroonIndicator {
    pub fn new(period: usize) -> Result<Self> {
        if period < 1 {
            Err(InvalidRangeError {
                param: Parameter::new("period", period),
                range: Range::LowerBounded { min: 1 },
            }
            .into())
        } else {
            let min_index = MinIndex::new(period + 1)?;
            let max_index = MaxIndex::new(period + 1)?;
            Ok(Self {
                period,
                min_index,
                max_index,
            })
        }
    }

    pub const DEFAULT_PERIOD: usize = 14;
}
impl Default for AroonIndicator {
    fn default() -> Self {
        Self {
            period: Self::DEFAULT_PERIOD,
            min_index: MinIndex::new(Self::DEFAULT_PERIOD + 1).unwrap(),
            max_index: MaxIndex::new(Self::DEFAULT_PERIOD + 1).unwrap(),
        }
    }
}
impl Indicator for AroonIndicator {
    type Input = f64;
    type Output = AroonIndicatorOutput;
    fn next(&mut self, input: Self::Input) -> Self::Output {
        let _ = self.min_index.next(input);
        let _ = self.max_index.next(input);
        self.current().unwrap()
    }
}
impl Current for AroonIndicator {
    fn current(&self) -> Option<Self::Output> {
        match (self.min_index.current(), self.max_index.current()) {
            (Some(min_index), Some(max_index)) => {
                let aroon_up = (self.period - max_index) as f64 / self.period as f64;
                let aroon_down = (self.period - min_index) as f64 / self.period as f64;
                AroonIndicatorOutput {
                    aroon_up,
                    aroon_down,
                }
                .into()
            }
            _ => None,
        }
    }
}
impl<Input: Price> NextExt<&Input> for AroonIndicator {
    fn next_ext(&mut self, input: &Input) -> Self::Output {
        self.next(input.price())
    }
}
impl Reset for AroonIndicator {
    fn reset(&mut self) {
        self.min_index.reset();
        self.max_index.reset();
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AroonIndicatorOutput {
    pub aroon_up: f64,
    pub aroon_down: f64,
}
impl From<(f64, f64)> for AroonIndicatorOutput {
    fn from((aroon_up, aroon_down): (f64, f64)) -> Self {
        Self {
            aroon_up,
            aroon_down,
        }
    }
}
impl Into<(f64, f64)> for AroonIndicatorOutput {
    fn into(self) -> (f64, f64) {
        (self.aroon_up, self.aroon_down)
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

    impl Round for AroonIndicatorOutput {
        fn round(self) -> Self {
            Self {
                aroon_up: Round::round(self.aroon_up),
                aroon_down: Round::round(self.aroon_down),
            }
        }
    }

    const PERIOD: usize = 4;
    static INPUTS: &[f64] = &[6.0, 7.0, 8.0, 3.0, 2.0, 4.0];
    static OUTPUTS: SyncLazy<Box<[AroonIndicatorOutput]>> = SyncLazy::new(|| {
        [
            (1.0, 1.0),
            (1.0, 0.75),
            (1.0, 0.5),
            (0.75, 1.0),
            (0.5, 1.0),
            (0.25, 0.75),
        ]
        .into_iter()
        .map(AroonIndicatorOutput::from)
        .collect::<Vec<_>>()
        .into_boxed_slice()
    });

    test_indicator! {
        new: AroonIndicator::new(PERIOD),
        inputs: INPUTS.iter().copied(),
        outputs: OUTPUTS.iter().copied(),
        additional_tests: {
            new_invalid_parameter: {
                new: AroonIndicator::new(0),
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
        let _: AroonIndicator = Default::default();
    }
}
