use crate::{Current, Indicator, Next, Price, Reset, Result, Rma};

/// Relative Strength Index
///
/// Range in \[0.0, 1.0\]
#[derive(Debug, Clone)]
pub struct Rsi {
    up: Rma,
    down: Rma,
    prev_input: Option<f64>,
}
impl Rsi {
    pub const DEFAULT_PERIOD: usize = 14;

    pub fn new(period: usize) -> Result<Self> {
        let up = Rma::new(period)?;
        let down = Rma::new(period)?;
        Ok(Self {
            up,
            down,
            prev_input: None,
        })
    }

    fn _next(&mut self, input: f64) -> <Self as Indicator>::Output {
        match &mut self.prev_input {
            Some(prev_input) => {
                let mut change = input - *prev_input;
                if change.is_infinite() && input.is_finite() && prev_input.is_finite() {
                    change = if input > *prev_input {
                        f64::MAX
                    } else {
                        -f64::MAX
                    };
                }
                let _ = self.up.next(change.max(0.0));
                let _ = self.down.next((-change).max(0.0));
                *prev_input = input;
            }
            None => {
                let _ = self.up.next(0.0);
                let _ = self.down.next(0.0);
                self.prev_input = input.into();
            }
        }

        self.current().unwrap()
    }
}
impl Default for Rsi {
    fn default() -> Self {
        Self {
            up: Rma::new(Self::DEFAULT_PERIOD).unwrap(),
            down: Rma::new(Self::DEFAULT_PERIOD).unwrap(),
            prev_input: None,
        }
    }
}

impl Indicator for Rsi {
    type Output = f64;
}
impl Current for Rsi {
    fn current(&self) -> Option<Self::Output> {
        match (self.up.current(), self.down.current()) {
            (Some(up), Some(down)) => match (up, down) {
                (up, down) if up <= 0.0 && down <= 0.0 => 0.5,
                (up, _) if up <= 0.0 => 0.0,
                (_, down) if down <= 0.0 => 1.0,
                (up, down) => 1.0 / (1.0 + down / up),
            }
            .into(),
            _ => None,
        }
    }
}
impl Next<f64> for Rsi {
    fn next(&mut self, input: f64) -> Self::Output {
        self._next(input)
    }
}
impl<Input: Price> Next<&Input> for Rsi {
    fn next(&mut self, input: &Input) -> Self::Output {
        self._next(input.price())
    }
}
impl Reset for Rsi {
    fn reset(&mut self) {
        self.up.reset();
        self.down.reset();
        self.prev_input = None;
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

    const PERIOD: usize = 3;
    static INPUTS: SyncLazy<Box<[TestItem]>> = SyncLazy::new(|| {
        [100.0, 101.0, 100.0, 100.0, 100.0, 102.0]
            .into_iter()
            .map(TestItem)
            .collect::<Vec<_>>()
            .into_boxed_slice()
    });
    static OUTPUTS: &[f64] = &[0.5, 1.0, 0.4, 0.4, 0.4, 0.8811881188118];

    test_indicator! {
        new: Rsi::new(PERIOD),
        inputs: INPUTS.iter().map(|x| x.price()),
        outputs: OUTPUTS.iter().copied(),
        additional_tests: {
            new_invalid_parameter: {
                new: Rsi::new(0),
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
    fn default() {
        let _: Rsi = Default::default();
    }

    #[test]
    fn period_must_meet_rma_minimum() {
        assert!(Rsi::new(1).is_err());
        assert!(Rsi::new(2).is_ok());
    }

    #[test]
    fn extreme_finite_price_changes_do_not_poison_rsi_state() -> crate::Result<()> {
        let mut rsi = Rsi::new(2)?;
        assert_eq!(rsi.next(-f64::MAX), 0.5);
        assert_eq!(rsi.next(f64::MAX), 1.0);

        for input in [-f64::MAX, 0.0, f64::MAX] {
            let value = rsi.next(input);
            assert!(value.is_finite());
            assert!((0.0..=1.0).contains(&value));
        }

        Ok(())
    }
}
