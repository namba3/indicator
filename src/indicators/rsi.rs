use crate::{Current, Indicator, NextExt, Price, Reset, Result, Rma};

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
    pub fn new(period: usize) -> Result<Self> {
        let up = Rma::new(period)?;
        let down = Rma::new(period)?;
        Ok(Self {
            up,
            down,
            prev_input: None,
        })
    }

    pub const DEFAULT_PERIOD: usize = 14;
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
    type Input = f64;
    type Output = f64;
    fn next(&mut self, input: Self::Input) -> Self::Output {
        match &mut self.prev_input {
            Some(prev_input) => {
                let change = input - *prev_input;
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
impl<Input: Price> NextExt<&Input> for Rsi {
    fn next_ext(&mut self, input: &Input) -> Self::Output {
        self.next(input.price())
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
    static INPUTS: &[f64] = &[100.0, 101.0, 100.0, 100.0, 100.0, 102.0];
    static OUTPUTS: &[f64] = &[0.5, 1.0, 0.4, 0.4, 0.4, 0.8811881188118];

    test_indicator! {
        new: Rsi::new(PERIOD),
        inputs: INPUTS.iter().copied(),
        outputs: OUTPUTS.iter().copied(),
        additional_tests: {
            new_invalid_parameter: {
                new: Rsi::new(0),
            },
            current: {
                inputs: RANDOM_DATA.iter().map(|x| x.price()),
            },
            next_ext: {
                inputs: INPUTS.iter().map(|x| TestItem(*x)),
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
}
