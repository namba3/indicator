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

    /// Update the indicator and return `None` until total volume is nonzero.
    ///
    /// This accepts the same input forms as [`Next`]. A zero-volume input does
    /// not affect the accumulated average. The [`Next`] implementations keep
    /// their `f64` output and return `NaN` while the result is undefined.
    pub fn next_option<Input>(&mut self, input: Input) -> Option<f64>
    where
        Self: Next<Input>,
    {
        let _ = <Self as Next<Input>>::next(self, input);
        self.current()
    }

    fn _next(&mut self, price: f64, volume: f64) -> Option<f64> {
        if volume == 0.0 {
            return self.current();
        }

        let previous_total_volume = self.total_volume;
        self.total_volume += volume;
        if self.total_volume == 0.0 {
            self.current = None;
            return None;
        }

        if previous_total_volume == 0.0 {
            self.current = Some(price);
        } else if let Some(current) = &mut self.current {
            *current += (price - *current) * volume / self.total_volume;
        } else {
            self.current = Some(price);
        }

        self.current()
    }
}

impl Indicator for Vwap {
    type Output = f64;
}
impl Current for Vwap {
    fn current(&self) -> Option<Self::Output> {
        (self.total_volume != 0.0).then_some(self.current).flatten()
    }
}
impl Next<(f64, f64)> for Vwap {
    fn next(&mut self, (price, volume): (f64, f64)) -> Self::Output {
        self._next(price, volume).unwrap_or(f64::NAN)
    }
}
impl<Input: Price + Volume> Next<&Input> for Vwap {
    fn next(&mut self, input: &Input) -> Self::Output {
        self._next(input.price(), input.volume())
            .unwrap_or(f64::NAN)
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
    use std::sync::LazyLock as SyncLazy;

    use super::*;
    use crate::{Volume, test_helper::*};

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

    #[test]
    fn zero_volume_does_not_change_weighted_average() {
        let mut indicator = Vwap::new();

        assert_eq!(indicator.next((100.0, 2.0)), 100.0);
        assert_eq!(indicator.next((999.0, 0.0)), 100.0);
        assert_eq!(indicator.next((110.0, 2.0)), 105.0);
    }

    #[test]
    fn zero_total_volume_is_undefined_until_positive_volume_arrives() {
        let mut indicator = Vwap::new();

        assert_eq!(indicator.current(), None);
        assert_eq!(indicator.next_option((100.0, 0.0)), None);
        assert!(indicator.next((100.0, 0.0)).is_nan());
        assert_eq!(indicator.next_option((120.0, 0.0)), None);
        assert_eq!(indicator.next_option((110.0, 2.0)), Some(110.0));
        assert_eq!(indicator.next_option((999.0, 0.0)), Some(110.0));
        assert_eq!(indicator.next_option(&TestItem(120.0, 2.0)), Some(115.0));
    }
}
