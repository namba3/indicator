use crate::{
    Current, Indicator, InvalidRangeError, Next, Parameter, Price, Range, Reset, Result,
    try_deque_with_capacity,
};
use alloc::collections::VecDeque;

const RECOMPUTE_SSE_RATIO: f64 = 1.0e-8;

/// Standard Deviation
#[derive(Debug, Clone)]
pub struct StandardDeviation {
    period: usize,
    ring: VecDeque<f64>,
    mean_sse: Option<(f64, f64)>,
    scaled_mean_sse: Option<(f64, f64, f64, usize)>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StandardDeviationOutput {
    pub mean: f64,
    pub sd: f64,
}

impl StandardDeviation {
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
                mean_sse: None,
                scaled_mean_sse: None,
            })
        }
    }

    fn _next(&mut self, input: f64) -> <Self as Indicator>::Output {
        if let Some((mean, sse)) = self.mean_sse {
            let old_input = self.ring.pop_front().unwrap();
            self.ring.push_back(input);

            let delta = input - old_input;
            let new_mean = mean + delta / self.period as f64;
            let delta2 = input - new_mean + old_input - mean;
            let new_sse = sse + delta * delta2;

            if !input.is_finite() || !old_input.is_finite() {
                self.mean_sse = Some((new_mean, new_sse));
            } else if delta.is_finite()
                && new_mean.is_finite()
                && delta2.is_finite()
                && new_sse.is_finite()
            {
                self.mean_sse = Some((new_mean, new_sse));

                // Rebuild after near-total cancellation to avoid retaining its rounding error.
                if new_sse < 0.0 || (sse > 0.0 && new_sse <= sse * RECOMPUTE_SSE_RATIO) {
                    self.recompute_mean_sse();
                }
            } else {
                self.mean_sse = None;
                self.recompute_scaled_mean_sse();
            }
        } else if let Some((scale, mean, sse, scale_count)) = self.scaled_mean_sse {
            let old_input = self.ring.pop_front().unwrap();
            self.ring.push_back(input);

            let old_is_scale = old_input.abs() == scale;
            let input_is_scale = input.abs() == scale;
            if !input.is_finite()
                || !old_input.is_finite()
                || input.abs() > scale
                || (old_is_scale && !input_is_scale && scale_count == 1)
            {
                self.recompute_scaled_mean_sse();
            } else if scale == 0.0 {
                self.scaled_mean_sse = Some((scale, mean, sse, scale_count));
            } else {
                let old_scaled = old_input / scale;
                let input_scaled = input / scale;
                let delta = input_scaled - old_scaled;
                let new_mean = mean + delta / self.period as f64;
                let delta2 = input_scaled - new_mean + old_scaled - mean;
                let new_sse = sse + delta * delta2;
                let new_scale_count =
                    scale_count - usize::from(old_is_scale) + usize::from(input_is_scale);

                if new_sse.is_finite() && new_sse >= 0.0 {
                    self.scaled_mean_sse = Some((scale, new_mean, new_sse, new_scale_count));
                    if sse > 0.0 && new_sse <= sse * RECOMPUTE_SSE_RATIO {
                        self.recompute_scaled_mean_sse();
                    }
                } else {
                    self.recompute_scaled_mean_sse();
                }
            }
        } else {
            for _ in 0..self.period {
                self.ring.push_back(input);
            }
            self.mean_sse = (input, 0.0).into();
        }
        self.current().unwrap()
    }

    fn recompute_mean_sse(&mut self) {
        let origin = *self.ring.front().unwrap();
        let mean_offset =
            self.ring.iter().map(|value| value - origin).sum::<f64>() / self.period as f64;
        let mean = origin + mean_offset;
        let sse = self
            .ring
            .iter()
            .map(|value| {
                let delta = value - mean;
                delta * delta
            })
            .sum::<f64>();

        if mean.is_finite() && sse.is_finite() && sse >= 0.0 {
            self.mean_sse = Some((mean, sse));
        } else {
            self.mean_sse = None;
            self.recompute_scaled_mean_sse();
        }
    }

    fn recompute_scaled_mean_sse(&mut self) {
        if self.ring.iter().any(|value| !value.is_finite()) {
            self.scaled_mean_sse = Some((f64::NAN, f64::NAN, f64::NAN, 0));
            return;
        }

        let scale = self
            .ring
            .iter()
            .map(|value| value.abs())
            .fold(0.0, f64::max);
        if scale == 0.0 {
            self.scaled_mean_sse = Some((0.0, 0.0, 0.0, self.period));
            return;
        }

        let mean = self.ring.iter().map(|value| value / scale).sum::<f64>() / self.period as f64;
        let sse = self
            .ring
            .iter()
            .map(|value| {
                let delta = value / scale - mean;
                delta * delta
            })
            .sum();
        let scale_count = self
            .ring
            .iter()
            .filter(|value| value.abs() == scale)
            .count();
        self.scaled_mean_sse = Some((scale, mean, sse, scale_count));
    }
}

impl Indicator for StandardDeviation {
    type Output = StandardDeviationOutput;
}
impl Current for StandardDeviation {
    fn current(&self) -> Option<Self::Output> {
        if let Some((mean, sse)) = self.mean_sse {
            Self::Output {
                mean,
                sd: libm::sqrt(sse / self.period as f64),
            }
            .into()
        } else if let Some((scale, mean, sse, _)) = self.scaled_mean_sse {
            Self::Output {
                mean: mean.clamp(-1.0, 1.0) * scale,
                sd: libm::sqrt(sse / self.period as f64).clamp(0.0, 1.0) * scale,
            }
            .into()
        } else {
            None
        }
    }
}
impl Next<f64> for StandardDeviation {
    fn next(&mut self, input: f64) -> Self::Output {
        self._next(input)
    }
}
impl<Input: Price> Next<&Input> for StandardDeviation {
    fn next(&mut self, input: &Input) -> Self::Output {
        self._next(input.price())
    }
}
impl Reset for StandardDeviation {
    fn reset(&mut self) {
        self.ring.clear();
        self.mean_sse = None;
        self.scaled_mean_sse = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::*;
    use std::sync::LazyLock as SyncLazy;

    #[derive(Clone)]
    struct TestItem(f64);
    impl Price for TestItem {
        fn price(&self) -> f64 {
            self.0
        }
    }

    impl Round for StandardDeviationOutput {
        fn round(self) -> Self {
            Self {
                mean: Round::round(self.mean),
                sd: Round::round(self.sd),
            }
        }
    }

    const PERIOD: usize = 5;
    static INPUTS: SyncLazy<Box<[TestItem]>> = SyncLazy::new(|| {
        [100.0, 104.0, 102.0, 102.0]
            .into_iter()
            .map(TestItem)
            .collect::<Vec<_>>()
            .into_boxed_slice()
    });
    static OUTPUTS: SyncLazy<Box<[StandardDeviationOutput]>> = SyncLazy::new(|| {
        [
            (100.0, 0.0),
            (100.8, 1.6),
            (101.2, 1.6),
            (101.6, 1.49666295),
        ]
        .into_iter()
        .map(|(mean, sd)| StandardDeviationOutput { mean, sd })
        .collect::<Vec<_>>()
        .into_boxed_slice()
    });

    test_indicator! {
        new: StandardDeviation::new(PERIOD),
        inputs: INPUTS.iter().map(|x| x.price()),
        outputs: OUTPUTS.iter().copied(),
        additional_tests: {
            new_invalid_parameter: {
                new: StandardDeviation::new(0),
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
        },
    }

    #[test]
    fn period_one_has_zero_standard_deviation() -> crate::Result<()> {
        let mut indicator = StandardDeviation::new(1)?;

        for input in [3.5, -2.0, 0.0, 9.25] {
            assert_eq!(
                indicator.next(input),
                StandardDeviationOutput {
                    mean: input,
                    sd: 0.0,
                }
            );
        }

        Ok(())
    }

    #[test]
    fn extreme_finite_values_keep_representable_statistics() -> crate::Result<()> {
        let mut indicator = StandardDeviation::new(2)?;
        assert_eq!(
            indicator.next(f64::MAX),
            StandardDeviationOutput {
                mean: f64::MAX,
                sd: 0.0,
            }
        );
        assert_eq!(
            indicator.next(-f64::MAX),
            StandardDeviationOutput {
                mean: 0.0,
                sd: f64::MAX,
            }
        );
        assert_eq!(
            indicator.next(-f64::MAX),
            StandardDeviationOutput {
                mean: -f64::MAX,
                sd: 0.0,
            }
        );
        assert_eq!(
            indicator.next(0.0),
            StandardDeviationOutput {
                mean: -f64::MAX / 2.0,
                sd: f64::MAX / 2.0,
            }
        );
        assert_eq!(
            indicator.next(0.0),
            StandardDeviationOutput { mean: 0.0, sd: 0.0 }
        );

        let mut squared_deviation = StandardDeviation::new(2)?;
        let _ = squared_deviation.next(f64::MAX);
        assert_eq!(
            squared_deviation.next(f64::MAX / 2.0),
            StandardDeviationOutput {
                mean: f64::MAX * 0.75,
                sd: f64::MAX * 0.25,
            }
        );

        Ok(())
    }
}
