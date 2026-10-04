use crate::{
    Current, Indicator, InvalidRangeError, InvalidVolumeError, Next, Parameter, Price, Range,
    Reset, Result, Volume,
};
use alloc::collections::VecDeque;

/// Volume Weighted Moving Average
#[derive(Debug, Clone)]
pub struct Vwma {
    period: usize,
    ring: VecDeque<(f64, f64)>,
    sum: Option<(f64, f64)>,
    compensation: (f64, f64),
}

const CANCELLATION_THRESHOLD: f64 = 1.0e8;

fn add_compensated(sum: &mut f64, compensation: &mut f64, value: f64) {
    if !sum.is_finite() || !compensation.is_finite() || !value.is_finite() {
        *sum += value;
        *compensation = 0.0;
        return;
    }

    let corrected_value = value - *compensation;
    let updated_sum = *sum + corrected_value;
    if !updated_sum.is_finite() {
        *sum = updated_sum;
        *compensation = 0.0;
        return;
    }
    *compensation = (updated_sum - *sum) - corrected_value;
    *sum = updated_sum;
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
                compensation: (0.0, 0.0),
            })
        }
    }

    /// Update the indicator and return `None` when the window has zero total volume.
    ///
    /// This accepts the same input forms as [`Next`]. The [`Next`] implementations
    /// keep their `f64` output and return `NaN` while the result is undefined. Use
    /// [`try_next`](Self::try_next) or [`try_next_ref`](Self::try_next_ref) to
    /// detect and reject negative or non-finite volumes.
    pub fn next_option<Input>(&mut self, input: Input) -> Option<f64>
    where
        Self: Next<Input>,
    {
        let _ = <Self as Next<Input>>::next(self, input);
        self.current()
    }

    /// Update the indicator, returning an error without changing state for a
    /// negative or non-finite volume.
    pub fn try_next(&mut self, (price, volume): (f64, f64)) -> Result<Option<f64>> {
        self._try_next(price, volume)
    }

    /// Update from custom market data, returning an error without changing state
    /// for a negative or non-finite volume.
    pub fn try_next_ref<Input: Price + Volume>(&mut self, input: &Input) -> Result<Option<f64>> {
        self._try_next(input.price(), input.volume())
    }

    fn _next(&mut self, price: f64, volume: f64) -> Option<f64> {
        self._try_next(price, volume)
            .unwrap_or_else(|_| self.current())
    }

    fn _try_next(&mut self, price: f64, volume: f64) -> Result<Option<f64>> {
        if !volume.is_finite() || volume < 0.0 {
            return Err(InvalidVolumeError::new(volume).into());
        }

        let mut removed = None;
        match &mut self.sum {
            Some((sum, total_volume)) => {
                let (old_price, old_volume) = self.ring.pop_front().unwrap();
                self.ring.push_back((price, volume));
                removed = Some((old_price * old_volume, old_volume));

                add_compensated(sum, &mut self.compensation.0, -(old_price * old_volume));
                add_compensated(total_volume, &mut self.compensation.1, -old_volume);
                add_compensated(sum, &mut self.compensation.0, price * volume);
                add_compensated(total_volume, &mut self.compensation.1, volume);
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
                self.compensation = (0.0, 0.0);
            }
        }

        if let (Some((removed_sum, removed_volume)), Some((sum, total_volume))) =
            (removed, self.sum)
        {
            let sum_cancellation = requires_rebase(removed_sum, sum);
            let volume_cancellation = requires_rebase(removed_volume, total_volume);
            if sum_cancellation || volume_cancellation {
                let (sum, total_volume) = self.sum.as_mut().unwrap();
                self.compensation = (0.0, 0.0);
                *sum = 0.0;
                *total_volume = 0.0;
                for (price, volume) in &self.ring {
                    add_compensated(sum, &mut self.compensation.0, price * volume);
                    add_compensated(total_volume, &mut self.compensation.1, *volume);
                }
            }
        }
        Ok(self.current())
    }
}

fn requires_rebase(removed: f64, remaining: f64) -> bool {
    removed != 0.0 && (remaining == 0.0 || removed.abs() > remaining.abs() * CANCELLATION_THRESHOLD)
}

impl Indicator for Vwma {
    type Output = f64;
}
impl Current for Vwma {
    fn current(&self) -> Option<Self::Output> {
        self.sum
            .and_then(|(sum, total_volume)| (total_volume != 0.0).then_some(sum / total_volume))
    }
}
impl Next<(f64, f64)> for Vwma {
    fn next(&mut self, (price, volume): (f64, f64)) -> Self::Output {
        self._next(price, volume).unwrap_or(f64::NAN)
    }
}
impl<Input: Price + Volume> Next<&Input> for Vwma {
    fn next(&mut self, input: &Input) -> Self::Output {
        self._next(input.price(), input.volume())
            .unwrap_or(f64::NAN)
    }
}
impl Reset for Vwma {
    fn reset(&mut self) {
        self.ring.clear();
        self.sum = None;
        self.compensation = (0.0, 0.0);
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

    #[test]
    fn period_one_returns_each_price() -> crate::Result<()> {
        let mut indicator = Vwma::new(1)?;

        for (price, volume) in [(3.5, 1.0), (-2.0, 4.0), (9.25, 2.0)] {
            assert_eq!(indicator.next((price, volume)), price);
        }

        Ok(())
    }

    #[test]
    fn zero_volume_does_not_change_weighted_average() -> crate::Result<()> {
        let mut indicator = Vwma::new(3)?;

        assert_eq!(indicator.next((100.0, 1.0)), 100.0);
        assert_eq!(indicator.next((999.0, 0.0)), 100.0);
        assert_eq!(indicator.next((110.0, 1.0)), 105.0);
        assert_eq!(indicator.next((110.0, 1.0)), 110.0);

        Ok(())
    }

    #[test]
    fn zero_volume_window_is_undefined_until_volume_arrives() -> crate::Result<()> {
        let mut indicator = Vwma::new(2)?;

        assert_eq!(indicator.current(), None);
        assert_eq!(indicator.next_option((100.0, 0.0)), None);
        assert!(indicator.next((200.0, 0.0)).is_nan());
        assert_eq!(indicator.current(), None);
        assert_eq!(indicator.next_option((110.0, 1.0)), Some(110.0));
        assert_eq!(indicator.next_option((120.0, 1.0)), Some(115.0));
        assert_eq!(indicator.next_option((130.0, 0.0)), Some(120.0));
        assert_eq!(indicator.next_option((140.0, 0.0)), None);
        assert!(indicator.next((150.0, 0.0)).is_nan());
        assert_eq!(indicator.next_option((160.0, 1.0)), Some(160.0));
        assert_eq!(indicator.next_option(&TestItem(170.0, 1.0)), Some(165.0));

        Ok(())
    }

    #[test]
    fn invalid_volumes_are_rejected_without_changing_state() -> crate::Result<()> {
        let mut indicator = Vwma::new(2)?;
        assert_eq!(indicator.try_next((100.0, 1.0))?, Some(100.0));
        assert_eq!(indicator.try_next((110.0, 1.0))?, Some(105.0));

        for volume in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(indicator.try_next((999.0, volume)).is_err());
            assert_eq!(indicator.current(), Some(105.0));
        }
        assert_eq!(indicator.next((999.0, -1.0)), 105.0);
        assert_eq!(indicator.try_next_ref(&TestItem(120.0, 1.0))?, Some(115.0));

        let mut empty = Vwma::new(2)?;
        assert!(empty.try_next((100.0, f64::NAN)).is_err());
        assert_eq!(empty.current(), None);
        assert!(empty.next((100.0, -1.0)).is_nan());
        assert_eq!(empty.current(), None);

        Ok(())
    }
}
