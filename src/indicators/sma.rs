use crate::{
    Current, Indicator, InvalidRangeError, Next, Parameter, Price, Range, Reset, Result,
    try_deque_with_capacity,
};
use alloc::collections::VecDeque;

/// Simple Moving Average
#[derive(Debug, Clone)]
pub struct Sma {
    period: usize,
    ring: VecDeque<f64>,
    sum: Option<f64>,
    mean: Option<f64>,
}
impl Sma {
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
                ring: try_deque_with_capacity(period)?,
                sum: None,
                mean: None,
            })
        }
    }

    fn _next(&mut self, input: f64) -> <Self as Indicator>::Output {
        if let Some(sum) = self.sum {
            let old_value = self.ring.pop_front().unwrap();
            self.ring.push_back(input);

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
            let old_value = self.ring.pop_front().unwrap();
            self.ring.push_back(input);
            self.mean = Some(Self::update_mean(mean, old_value, input, self.period));
        } else {
            for _ in 0..self.period {
                self.ring.push_back(input);
            }
            let sum = input * self.period as f64;
            if sum.is_finite() {
                self.sum = Some(sum);
            } else {
                self.mean = Some(input);
            }
        }
        self.current().unwrap()
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
}
