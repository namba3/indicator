use super::padded_window::values as padded_values;
use crate::{
    Current, Indicator, Next, Price, Reset, Result, try_deque_with_capacity, validate_period,
};
use alloc::collections::VecDeque;

/// Simple Moving Average
#[derive(Debug, Clone)]
pub struct Sma {
    period: usize,
    ring: VecDeque<f64>,
    sum: Option<f64>,
    mean: Option<f64>,
    nan_count: usize,
    positive_infinity_count: usize,
    negative_infinity_count: usize,
}
impl Sma {
    pub fn new(period: usize) -> Result<Self> {
        validate_period(period, 1)?;
        Ok(Self {
            period,
            ring: try_deque_with_capacity(period)?,
            sum: None,
            mean: None,
            nan_count: 0,
            positive_infinity_count: 0,
            negative_infinity_count: 0,
        })
    }

    fn _next(&mut self, input: f64) -> <Self as Indicator>::Output {
        if self.sum.is_none() && self.mean.is_none() {
            self.ring.push_back(input);
            self.nan_count = usize::from(input.is_nan()) * self.period;
            self.positive_infinity_count = usize::from(input == f64::INFINITY) * self.period;
            self.negative_infinity_count = usize::from(input == f64::NEG_INFINITY) * self.period;
            if let Some(mean) = self.non_finite_mean() {
                self.mean = Some(mean);
            } else {
                let sum = input * self.period as f64;
                if sum.is_finite() {
                    self.sum = Some(sum);
                } else {
                    self.mean = Some(input);
                }
            }
            return self.current().unwrap();
        }

        let old_value = self.push_window(input);
        self.nan_count -= usize::from(old_value.is_nan());
        self.nan_count += usize::from(input.is_nan());
        self.positive_infinity_count -= usize::from(old_value == f64::INFINITY);
        self.positive_infinity_count += usize::from(input == f64::INFINITY);
        self.negative_infinity_count -= usize::from(old_value == f64::NEG_INFINITY);
        self.negative_infinity_count += usize::from(input == f64::NEG_INFINITY);

        if let Some(mean) = self.non_finite_mean() {
            self.sum = None;
            self.mean = Some(mean);
        } else if self.mean.is_some_and(|mean| !mean.is_finite()) {
            self.sum = None;
            self.mean = Some(Self::recompute_finite_mean(&self.ring, self.period));
        } else if let Some(sum) = self.sum {
            let sum_without_old = sum - old_value;
            let updated_sum = sum_without_old + input;
            if sum_without_old.is_finite() && updated_sum.is_finite() {
                self.sum = Some(updated_sum);
            } else {
                let mean = sum / self.period as f64;
                self.sum = None;
                self.mean = Some(Self::update_mean(mean, old_value, input, self.period));
            }
        } else if let Some(mean) = self.mean {
            self.mean = Some(Self::update_mean(mean, old_value, input, self.period));
        }
        self.current().unwrap()
    }

    fn push_window(&mut self, input: f64) -> f64 {
        let old_value = if self.ring.len() < self.period {
            *self.ring.front().unwrap()
        } else {
            self.ring.pop_front().unwrap()
        };
        self.ring.push_back(input);
        old_value
    }

    fn update_mean(mean: f64, old_value: f64, input: f64, period: usize) -> f64 {
        if period == 1 {
            input
        } else if old_value.is_sign_positive() != input.is_sign_positive() {
            mean + input / period as f64 - old_value / period as f64
        } else {
            mean + (input - old_value) / period as f64
        }
    }

    fn non_finite_mean(&self) -> Option<f64> {
        if self.nan_count > 0
            || (self.positive_infinity_count > 0 && self.negative_infinity_count > 0)
        {
            Some(f64::NAN)
        } else if self.positive_infinity_count > 0 {
            Some(f64::INFINITY)
        } else if self.negative_infinity_count > 0 {
            Some(f64::NEG_INFINITY)
        } else {
            None
        }
    }

    fn recompute_finite_mean(ring: &VecDeque<f64>, period: usize) -> f64 {
        let scale =
            padded_values(ring, period).fold(0.0_f64, |scale, value| scale.max(value.abs()));
        if scale == 0.0 {
            return 0.0;
        }

        let scaled_sum = padded_values(ring, period).fold(0.0, |sum, value| sum + value / scale);
        (scaled_sum / period as f64) * scale
    }
}

impl Indicator for Sma {
    type Output = f64;
}
impl Current for Sma {
    fn current(&self) -> Option<Self::Output> {
        self.mean
            .or_else(|| self.sum.map(|sum| sum / self.period as f64))
    }
}
impl Next<f64> for Sma {
    fn next(&mut self, input: f64) -> Self::Output {
        self._next(input)
    }
}
impl<Input: Price> Next<&Input> for Sma {
    fn next(&mut self, input: &Input) -> Self::Output {
        self._next(input.price())
    }
}
impl Reset for Sma {
    fn reset(&mut self) {
        self.ring.clear();
        self.sum = None;
        self.mean = None;
        self.nan_count = 0;
        self.positive_infinity_count = 0;
        self.negative_infinity_count = 0;
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock as SyncLazy;

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
        [100.0, 101.0, 101.0, 102.0, 102.0, 102.0]
            .into_iter()
            .map(TestItem)
            .collect::<Vec<_>>()
            .into_boxed_slice()
    });
    static OUTPUTS: &[f64] = &[100.0, 100.2, 100.4, 100.8, 101.2, 101.6];

    test_indicator! {
        new: Sma::new(PERIOD),
        inputs: INPUTS.iter().map(|x| x.price()),
        outputs: OUTPUTS.iter().copied(),
        additional_tests: {
            new_invalid_parameter: {
                new: Sma::new(0),
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
    fn period_one_tracks_each_input() -> crate::Result<()> {
        let mut sma = Sma::new(1)?;

        for input in [3.5, -2.0, 0.0, 9.25] {
            assert_eq!(sma.next(input), input);
        }

        Ok(())
    }

    #[test]
    fn first_input_is_stored_once_while_outputs_keep_virtual_padding() -> crate::Result<()> {
        let mut sma = Sma::new(3)?;

        assert_eq!(sma.next(2.0), 2.0);
        assert_eq!(sma.ring.iter().copied().collect::<Vec<_>>(), [2.0]);

        assert_eq!(sma.next(4.0), 2.0 + 2.0 / 3.0);
        assert_eq!(sma.ring.iter().copied().collect::<Vec<_>>(), [2.0, 4.0]);

        assert_eq!(sma.next(6.0), 4.0);
        assert_eq!(
            sma.ring.iter().copied().collect::<Vec<_>>(),
            [2.0, 4.0, 6.0]
        );

        assert_eq!(sma.next(8.0), 6.0);
        assert_eq!(
            sma.ring.iter().copied().collect::<Vec<_>>(),
            [4.0, 6.0, 8.0]
        );

        sma.reset();
        assert_eq!(sma.next(10.0), 10.0);
        assert_eq!(sma.ring.iter().copied().collect::<Vec<_>>(), [10.0]);

        Ok(())
    }

    #[test]
    fn opposite_extreme_finite_inputs_do_not_overflow() -> crate::Result<()> {
        let mut sma = Sma::new(2)?;
        assert_eq!(sma.next(f64::MAX), f64::MAX);
        assert_eq!(sma.current(), Some(f64::MAX));
        assert_eq!(sma.next(-f64::MAX), 0.0);
        assert_eq!(sma.next(-f64::MAX), -f64::MAX);

        let mut rolling_sum = Sma::new(2)?;
        assert_eq!(rolling_sum.next(-f64::MAX / 2.0), -f64::MAX / 2.0);
        assert_eq!(rolling_sum.next(f64::MAX), f64::MAX / 4.0);
        assert_eq!(rolling_sum.next(f64::MAX), f64::MAX);

        let mut period_one = Sma::new(1)?;
        assert_eq!(period_one.next(f64::MAX), f64::MAX);
        assert_eq!(period_one.next(-f64::MAX), -f64::MAX);

        Ok(())
    }

    #[test]
    fn non_finite_values_stop_affecting_the_average_after_leaving_the_window() -> crate::Result<()>
    {
        let mut nan_window = Sma::new(3)?;
        assert!(nan_window.next(f64::NAN).is_nan());
        assert!(nan_window.next(1.0).is_nan());
        assert!(nan_window.next(2.0).is_nan());
        assert_eq!(nan_window.next(3.0), 2.0);
        assert_eq!(nan_window.current(), Some(2.0));
        nan_window.reset();
        assert_eq!(nan_window.next(4.0), 4.0);

        let mut infinite_window = Sma::new(3)?;
        assert_eq!(infinite_window.next(f64::INFINITY), f64::INFINITY);
        assert_eq!(infinite_window.next(1.0), f64::INFINITY);
        assert_eq!(infinite_window.next(2.0), f64::INFINITY);
        assert_eq!(infinite_window.next(3.0), 2.0);

        let mut extreme_finite_recovery = Sma::new(2)?;
        assert_eq!(extreme_finite_recovery.next(f64::INFINITY), f64::INFINITY);
        assert_eq!(extreme_finite_recovery.next(f64::MAX), f64::INFINITY);
        assert_eq!(extreme_finite_recovery.next(-f64::MAX), 0.0);

        let mut mixed_infinities = Sma::new(3)?;
        assert_eq!(mixed_infinities.next(f64::INFINITY), f64::INFINITY);
        assert!(mixed_infinities.next(f64::NEG_INFINITY).is_nan());
        assert!(mixed_infinities.next(5.0).is_nan());
        assert_eq!(mixed_infinities.next(7.0), f64::NEG_INFINITY);
        assert_eq!(mixed_infinities.next(9.0), 7.0);

        Ok(())
    }
}
