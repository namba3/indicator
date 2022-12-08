use crate::{Current, Indicator, Next, Reset};

pub struct Together<Lhs, Rhs>
where
    Lhs: Indicator,
    Rhs: Indicator,
{
    lhs: Lhs,
    rhs: Rhs,
}
impl<Lhs, Rhs> Together<Lhs, Rhs>
where
    Lhs: Indicator,
    Rhs: Indicator,
{
    pub(crate) fn new(lhs: Lhs, rhs: Rhs) -> Self {
        Self { lhs, rhs }
    }

    /// Take out the indicators that composes this indicator
    pub fn decompose(self) -> (Lhs, Rhs) {
        (self.lhs, self.rhs)
    }
}
impl<Lhs, Rhs> Indicator for Together<Lhs, Rhs>
where
    Lhs: Indicator,
    Rhs: Indicator,
{
    type Output = (Lhs::Output, Rhs::Output);
}

impl<Lhs, Rhs, T> Next<T> for Together<Lhs, Rhs>
where
    T: Clone,
    Lhs: Indicator + Next<T>,
    Rhs: Indicator + Next<T>,
{
    fn next(&mut self, input: T) -> Self::Output {
        (self.lhs.next(input.clone()), self.rhs.next(input))
    }
}

impl<Lhs, Rhs> Current for Together<Lhs, Rhs>
where
    Lhs: Indicator + Current,
    Rhs: Indicator + Current,
{
    fn current(&self) -> Option<Self::Output> {
        match (self.lhs.current(), self.rhs.current()) {
            (Some(lhs), Some(rhs)) => (lhs, rhs).into(),
            _ => None,
        }
    }
}
impl<Lhs, Rhs> Reset for Together<Lhs, Rhs>
where
    Lhs: Indicator + Reset,
    Rhs: Indicator + Reset,
{
    fn reset(&mut self) {
        self.lhs.reset();
        self.rhs.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{test_helper::*, Ema, Price, Sma};
    use once_cell::sync::Lazy as SyncLazy;

    impl Round for (f64, f64) {
        fn round(self) -> Self {
            (Round::round(self.0), Round::round(self.1))
        }
    }

    #[derive(Clone)]
    struct TestItem(f64);
    impl Price for TestItem {
        fn price(&self) -> f64 {
            self.0
        }
    }
    const PERIOD: usize = 4;
    static INPUTS: SyncLazy<Box<[TestItem]>> = SyncLazy::new(|| {
        [101.0, 101.0, 101.0, 102.0, 102.0, 102.0]
            .into_iter()
            .map(TestItem)
            .collect::<Vec<_>>()
            .into_boxed_slice()
    });
    static OUTPUTS: &[(f64, f64)] = &[
        (101.0, 101.0),
        (101.0, 101.0),
        (101.0, 101.0),
        (101.25, 101.4),
        (101.5, 101.64),
        (101.75, 101.784),
    ];

    test_indicator! {
        new: Ok(Together::new(Sma::new(PERIOD).unwrap(), Ema::new(PERIOD).unwrap())),
        inputs: INPUTS.iter().map(|x| x.price()),
        outputs: OUTPUTS.iter().copied(),
        additional_tests: {
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
}
