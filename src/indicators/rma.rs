use crate::{Current, Indicator, InvalidRangeError, Next, Parameter, Price, Range, Reset, Result};

/// Running Moving Average (a.k.a Modified Moving Average)
#[derive(Debug, Clone)]
pub struct Rma {
    period: usize,
    current: Option<f64>,
}
impl Rma {
    pub fn new(period: usize) -> Result<Self> {
        if period < 2 {
            Err(InvalidRangeError {
                param: Parameter::new("period", period),
                range: Range::LowerBounded { min: 2 },
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
                *current += (input - *current) / self.period as f64;
            }
            None => {
                self.current = input.into();
            }
        }
        self.current().unwrap()
    }
}

impl Indicator for Rma {
    type Output = f64;
}
impl Current for Rma {
    fn current(&self) -> Option<Self::Output> {
        self.current
    }
}
impl Next<f64> for Rma {
    fn next(&mut self, input: f64) -> Self::Output {
        self._next(input)
    }
}
impl<Input: Price> Next<&Input> for Rma {
    fn next(&mut self, input: &Input) -> Self::Output {
        self._next(input.price())
    }
}
impl Reset for Rma {
    fn reset(&mut self) {
        self.current = None;
    }
}

#[cfg(test)]
mod tests {
    use std::lazy::SyncLazy;

    use super::*;
    use crate::test_helper::*;

    #[derive(Clone)]
    struct TestItem(f64);
    impl Price for TestItem {
        fn price(&self) -> f64 {
            self.0
        }
    }

    const PERIOD: usize = 5;
    static INPUTS: SyncLazy<Box<[TestItem]>> = SyncLazy::new(|| {
        [101.0, 101.0, 101.0, 102.0, 101.0, 101.0]
            .into_iter()
            .map(TestItem)
            .collect::<Vec<_>>()
            .into_boxed_slice()
    });
    static OUTPUTS: &[f64] = &[101.0, 101.0, 101.0, 101.2, 101.16, 101.128];

    test_indicator! {
        new: Rma::new(PERIOD),
        inputs: INPUTS.iter().map(|x| x.price()),
        outputs: OUTPUTS.iter().copied(),
        additional_tests: {
            new_invalid_parameter: {
                new: Rma::new(0),
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
