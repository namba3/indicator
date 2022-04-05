use crate::{AroonIndicator, Current, Indicator, Next, Price, Reset, Result};

/// Aroon Oscillator
/// ///
/// Range in \[-1.0, 1.0\]
#[derive(Debug, Clone)]
pub struct AroonOscillator {
    aroon_indicator: AroonIndicator,
}
impl AroonOscillator {
    pub const DEFAULT_PERIOD: usize = AroonIndicator::DEFAULT_PERIOD;

    pub fn new(period: usize) -> Result<Self> {
        let aroon_indicator = AroonIndicator::new(period)?;
        Ok(Self { aroon_indicator })
    }

    fn _next(&mut self, input: f64) -> <Self as Indicator>::Output {
        let _ = self.aroon_indicator.next(input);
        self.current().unwrap()
    }
}
impl Default for AroonOscillator {
    fn default() -> Self {
        Self {
            aroon_indicator: Default::default(),
        }
    }
}
impl Indicator for AroonOscillator {
    type Output = f64;
}
impl Current for AroonOscillator {
    fn current(&self) -> Option<Self::Output> {
        self.aroon_indicator
            .current()
            .map(|x| x.aroon_up - x.aroon_down)
    }
}
impl Next<f64> for AroonOscillator {
    fn next(&mut self, input: f64) -> Self::Output {
        self._next(input)
    }
}
impl<Input: Price> Next<&Input> for AroonOscillator {
    fn next(&mut self, input: &Input) -> Self::Output {
        self._next(input.price())
    }
}
impl Reset for AroonOscillator {
    fn reset(&mut self) {
        self.aroon_indicator.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::*;
    use std::lazy::SyncLazy;

    #[derive(Clone)]
    struct TestItem(f64);
    impl Price for TestItem {
        fn price(&self) -> f64 {
            self.0
        }
    }

    const PERIOD: usize = 4;
    static INPUTS: SyncLazy<Box<[TestItem]>> = SyncLazy::new(|| {
        vec![6.0, 7.0, 8.0, 3.0, 2.0, 4.0]
            .into_iter()
            .map(TestItem)
            .collect::<Vec<_>>()
            .into_boxed_slice()
    });
    static OUTPUTS: &[f64] = &[0.0, 0.25, 0.5, -0.25, -0.5, -0.5];

    test_indicator! {
        new: AroonOscillator::new(PERIOD),
        inputs: INPUTS.iter().map(|x| x.price()),
        outputs: OUTPUTS.iter().copied(),
        additional_tests: {
            new_invalid_parameter: {
                new: AroonOscillator::new(0),
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
        let _: AroonIndicator = Default::default();
    }
}
