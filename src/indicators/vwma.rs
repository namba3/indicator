use crate::{
    Current, Indicator, InvalidRangeError, NextExt, Parameter, Price, Range, Reset, Result, Volume,
};
use alloc::collections::VecDeque;

/// Volume Weighted Moving Average
#[derive(Debug, Clone)]
pub struct Vwma {
    period: usize,
    ring: VecDeque<(f64, f64)>,
    sum: Option<(f64, f64)>,
}
impl Vwma {
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
                ring: VecDeque::with_capacity(period),
                sum: None,
            })
        }
    }
}

impl Indicator for Vwma {
    type Input = (f64, f64);
    type Output = f64;
    fn next(&mut self, (price, volume): Self::Input) -> Option<Self::Output> {
        match &mut self.sum {
            Some((sum, total_volume)) => {
                let (old_price, old_volume) = self.ring.pop_front().unwrap();
                self.ring.push_back((price, volume));

                *sum -= old_price * old_volume;
                *total_volume -= old_volume;

                *sum += price * volume;
                *total_volume += volume;
            }
            None => {
                for _ in 0..self.period {
                    self.ring.push_back((price, volume));
                }
                self.sum = (
                    price * volume * self.period as f64,
                    volume * self.period as f64,
                )
                    .into();
            }
        }
        self.current()
    }
}
impl Current for Vwma {
    fn current(&self) -> Option<Self::Output> {
        self.sum.map(|(sum, total_volume)| sum / total_volume)
    }
}
impl<Input: Price + Volume> NextExt<&Input> for Vwma {
    fn next_ext(&mut self, input: &Input) -> Option<Self::Output> {
        self.next((input.price(), input.volume()))
    }
}
impl Reset for Vwma {
    fn reset(&mut self) {
        self.ring.clear();
        self.sum = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{test_helper::*, Volume};
    use std::lazy::SyncLazy;

    #[derive(Clone)]
    struct TestItem(f64, f64);
    impl Price for TestItem {
        fn price(&self) -> f64 {
            self.0
        }
    }
    impl Volume for TestItem {
        fn volume(&self) -> f64 {
            self.1
        }
    }

    const PERIOD: usize = 4;
    static INPUTS: &[(f64, f64)] = &[
        (101.0, 1.0),
        (102.0, 1.0),
        (101.0, 2.0),
        (102.0, 2.0),
        (102.0, 3.0),
        (102.0, 1.0),
    ];
    static OUTPUTS: SyncLazy<Box<[Option<f64>]>> = SyncLazy::new(|| {
        [101.0, 101.25, 101.2, 101.5, 101.75, 101.75]
            .into_iter()
            .map(Some)
            .collect::<Vec<Option<_>>>()
            .into_boxed_slice()
    });

    test_indicator! {
        new: Vwma::new(PERIOD),
        inputs: INPUTS.iter().copied(),
        outputs: OUTPUTS.iter().copied(),
        additional_tests: {
            new_invalid_parameter: {
                new: Vwma::new(0),
            },
            current: {
                inputs: RANDOM_DATA.iter().map(|x| (x.price(), x.volume())),
            },
            next_ext: {
                inputs: INPUTS.iter().map(|(x,y)| TestItem(*x,*y)),
                outputs: OUTPUTS.iter().copied(),
            },
            reset: {
                inputs: RANDOM_DATA.iter().map(|x| (x.price(), x.volume())),
            },
        }
    }
}
