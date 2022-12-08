use crate::{Current, Indicator, Next, Reset};

/// Create a new indicator by combining the two indicators in serial.
pub struct Composition<Inner, Outer>
where
    Inner: Indicator,
    Outer: Indicator + Next<Inner::Output>,
{
    inner: Inner,
    outer: Outer,
}
impl<Inner, Outer> Composition<Inner, Outer>
where
    Inner: Indicator,
    Outer: Indicator + Next<Inner::Output>,
{
    pub(crate) fn new(inner: Inner, outer: Outer) -> Self {
        Self { inner, outer }
    }

    /// Take out the indicators that composes this indicator
    pub fn decompose(self) -> (Inner, Outer) {
        (self.inner, self.outer)
    }
}
impl<Inner, Outer> Indicator for Composition<Inner, Outer>
where
    Inner: Indicator,
    Outer: Indicator + Next<Inner::Output>,
{
    type Output = Outer::Output;
}
impl<Inner, Outer, N> Next<N> for Composition<Inner, Outer>
where
    Inner: Indicator + Next<N>,
    Outer: Indicator + Next<Inner::Output>,
{
    fn next(&mut self, input: N) -> Self::Output {
        self.outer.next(self.inner.next(input))
    }
}
impl<Inner, Outer> Current for Composition<Inner, Outer>
where
    Inner: Indicator,
    Outer: Indicator + Next<Inner::Output> + Current,
{
    fn current(&self) -> Option<Self::Output> {
        self.outer.current()
    }
}
impl<Inner, Outer> Reset for Composition<Inner, Outer>
where
    Inner: Indicator + Reset,
    Outer: Indicator + Next<Inner::Output> + Reset,
{
    fn reset(&mut self) {
        self.inner.reset();
        self.outer.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{test_helper::*, Rsi};
    use crate::{Price, Sma};
    use once_cell::sync::Lazy as SyncLazy;

    #[derive(Clone)]
    struct TestItem(f64);
    impl Price for TestItem {
        fn price(&self) -> f64 {
            self.0
        }
    }
    const RSI_PERIOD: usize = 3;
    const SMA_PERIOD: usize = 2;
    static INPUTS: SyncLazy<Box<[TestItem]>> = SyncLazy::new(|| {
        [100.0, 101.0, 100.0, 100.0, 100.0, 102.0]
            .into_iter()
            .map(TestItem)
            .collect::<Vec<_>>()
            .into_boxed_slice()
    });
    static OUTPUTS: SyncLazy<Box<[f64]>> =
        SyncLazy::new(|| [0.5, 0.75, 0.7, 0.4, 0.4, 0.6405940594].into());

    test_indicator! {
        new: match (Sma::new(SMA_PERIOD), Rsi::new(RSI_PERIOD)) {
            (Ok(sma), Ok(rsi)) => Ok(Composition::new(rsi, sma)),
            (Err(why), _) => Err(why),
            (Ok(_), Err(why)) => Err(why),
        },
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
        },
    }
}
