use crate::{Current, Indicator, NextExt, Price, Reset, Volume};

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
}

impl Indicator for Vwap {
    type Input = (f64, f64);
    type Output = f64;
    fn next(&mut self, (price, volume): Self::Input) -> Option<Self::Output> {
        self.total_volume += volume;
        match &mut self.current {
            Some(current) => {
                *current += (price - *current) * volume / self.total_volume;
            }
            None => {
                self.current = price.into();
            }
        }
        self.current()
    }
}
impl Current for Vwap {
    fn current(&self) -> Option<Self::Output> {
        self.current
    }
}
impl<Input: Price + Volume> NextExt<&Input> for Vwap {
    fn next_ext(&mut self, input: &Input) -> Option<Self::Output> {
        self.next((input.price(), input.volume()))
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

    static INPUTS: &[(f64, f64)] = &[
        (101.0, 1.0),
        (102.0, 1.0),
        (101.0, 2.0),
        (102.0, 2.0),
        (102.0, 2.0),
        (102.0, 2.0),
    ];
    static OUTPUTS: SyncLazy<Box<[Option<f64>]>> = SyncLazy::new(|| {
        [101.0, 101.5, 101.25, 101.5, 101.625, 101.7]
            .into_iter()
            .map(Some)
            .collect::<Vec<Option<_>>>()
            .into_boxed_slice()
    });

    test_indicator! {
        new: crate::Result::Ok(Vwap::new()),
        inputs: INPUTS.iter().copied(),
        outputs: OUTPUTS.iter().copied(),
        additional_tests: {
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
