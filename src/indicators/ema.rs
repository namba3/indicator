use crate::{
    Current, Indicator, InvalidRangeError, NextExt, Parameter, Price, Range, Reset, Result,
};

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
}

impl Indicator for Ema {
    type Input = f64;
    type Output = f64;
    fn next(&mut self, input: Self::Input) -> Option<Self::Output> {
        match &mut self.current {
            Some(current) => {
                *current += (input - *current) * (2.0 / (self.period + 1) as f64);
            }
            None => {
                self.current = input.into();
            }
        }
        self.current()
    }
}
impl Current for Ema {
    fn current(&self) -> Option<Self::Output> {
        self.current
    }
}
impl<Input: Price> NextExt<&Input> for Ema {
    fn next_ext(&mut self, input: &Input) -> Option<Self::Output> {
        self.next(input.price())
    }
}
impl Reset for Ema {
    fn reset(&mut self) {
        self.current = None;
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

    const PERIOD: usize = 4;
    static INPUTS: &[f64] = &[101.0, 101.0, 101.0, 102.0, 102.0, 102.0];
    static OUTPUTS: SyncLazy<Box<[Option<f64>]>> = SyncLazy::new(|| {
        [101.0, 101.0, 101.0, 101.4, 101.64, 101.784]
            .into_iter()
            .map(Some)
            .collect::<Vec<Option<_>>>()
            .into_boxed_slice()
    });

    test_indicator! {
        new: Ema::new(PERIOD),
        inputs: INPUTS.iter().copied(),
        outputs: OUTPUTS.iter().copied(),
        additional_tests: {
            new_invalid_parameter: {
                new: Ema::new(0),
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
