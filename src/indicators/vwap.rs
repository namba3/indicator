use crate::{Current, Indicator, Next, Price, Reset, Volume};

/// Volume Weighted Average Price
#[derive(Debug, Clone)]
pub struct Vwap {
    current: Option<f64>,
    total_volume: f64,
}
impl Vwap {
    pub fn new() -> Self {
        Self {
            current: None,
            total_volume: 0.0,
        }
    }

    fn _next(&mut self, price: f64, volume: f64) -> <Self as Indicator>::Output {
        self.total_volume += volume;
        match &mut self.current {
            Some(current) => {
                *current += (price - *current) * volume / self.total_volume;
            }
            None => {
                self.current = price.into();
            }
        }
        self.current().unwrap()
    }
}

impl Indicator for Vwap {
    type Output = f64;
}
impl Current for Vwap {
    fn current(&self) -> Option<Self::Output> {
        self.current
    }
}
impl Next<(f64, f64)> for Vwap {
    fn next(&mut self, (price, volume): (f64, f64)) -> Self::Output {
        self._next(price, volume)
    }
}
impl<Input: Price + Volume> Next<&Input> for Vwap {
    fn next(&mut self, input: &Input) -> Self::Output {
        self._next(input.price(), input.volume())
    }
}
impl Reset for Vwap {
    fn reset(&mut self) {
        self.current = None;
        self.total_volume = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use std::lazy::SyncLazy;

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

    static INPUTS: SyncLazy<Box<[TestItem]>> = SyncLazy::new(|| {
        [
            (101.0, 1.0),
            (102.0, 1.0),
            (101.0, 2.0),
            (102.0, 2.0),
            (102.0, 2.0),
            (102.0, 2.0),
        ]
        .into_iter()
        .map(|(price, volume)| TestItem(price, volume))
        .collect::<Vec<_>>()
        .into_boxed_slice()
    });
    static OUTPUTS: &[f64] = &[101.0, 101.5, 101.25, 101.5, 101.625, 101.7];

    test_indicator! {
        new: crate::Result::Ok(Vwap::new()),
        inputs: INPUTS.iter().map(|x| (x.price(), x.volume())),
        outputs: OUTPUTS.iter().copied(),
        additional_tests: {
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
