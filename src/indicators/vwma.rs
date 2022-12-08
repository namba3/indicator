use crate::{
    Current, Indicator, InvalidRangeError, Next, Parameter, Price, Range, Reset, Result, Volume,
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

    fn _next(&mut self, price: f64, volume: f64) -> <Self as Indicator>::Output {
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
        self.current().unwrap()
    }
}

impl Indicator for Vwma {
    type Output = f64;
}
impl Current for Vwma {
    fn current(&self) -> Option<Self::Output> {
        self.sum.map(|(sum, total_volume)| sum / total_volume)
    }
}
impl Next<(f64, f64)> for Vwma {
    fn next(&mut self, (price, volume): (f64, f64)) -> Self::Output {
        self._next(price, volume)
    }
}
impl<Input: Price + Volume> Next<&Input> for Vwma {
    fn next(&mut self, input: &Input) -> Self::Output {
        self._next(input.price(), input.volume())
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
    use once_cell::sync::Lazy as SyncLazy;

    use super::*;
    use crate::{test_helper::*, Volume};

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
    static INPUTS: SyncLazy<Box<[TestItem]>> = SyncLazy::new(|| {
        [
            (101.0, 1.0),
            (102.0, 1.0),
            (101.0, 2.0),
            (102.0, 2.0),
            (102.0, 3.0),
            (102.0, 1.0),
        ]
        .into_iter()
        .map(|(price, volume)| TestItem(price, volume))
        .collect::<Vec<_>>()
        .into_boxed_slice()
    });
    static OUTPUTS: &[f64] = &[101.0, 101.25, 101.2, 101.5, 101.75, 101.75];

    test_indicator! {
        new: Vwma::new(PERIOD),
        inputs: INPUTS.iter().map(|x| (x.price(), x.volume())),
        outputs: OUTPUTS.iter().copied(),
        additional_tests: {
            new_invalid_parameter: {
                new: Vwma::new(0),
            },
            current: {
                inputs: RANDOM_DATA.iter().map(|x| (x.price(), x.volume())),
            },
            next_ext: {
                inputs: INPUTS.iter(),
                outputs: OUTPUTS.iter().copied(),
            },
            reset: {
                inputs: RANDOM_DATA.iter().map(|x| (x.price(), x.volume())),
            },
        }
    }
}
