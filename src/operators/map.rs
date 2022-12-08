use crate::{Current, Indicator, Next, Reset};

/// Create a new indicator that applies a projection to the output of the indicator.
pub struct Map<I, F, R>
where
    I: Indicator,
    F: FnMut(I::Output) -> R,
{
    i: I,
    f: F,
    _phantom_r: core::marker::PhantomData<R>,
}
impl<I, F, R> Map<I, F, R>
where
    I: Indicator,
    F: FnMut(I::Output) -> R,
{
    pub(crate) fn new(i: I, f: F) -> Self {
        Self {
            i,
            f,
            _phantom_r: Default::default(),
        }
    }

    /// Take out the inner indicator that composes this indicator
    pub fn decompose(self) -> I {
        self.i
    }
}

impl<I, F, R> Indicator for Map<I, F, R>
where
    I: Indicator,
    F: FnMut(I::Output) -> R,
{
    type Output = R;
}

impl<I, F, R> Current for Map<I, F, R>
where
    I: Indicator + Current,
    F: Fn(<I as Indicator>::Output) -> R,
{
    fn current(&self) -> Option<R> {
        self.i.current().map(|x| (self.f)(x))
    }
}

impl<I, F, N, R> Next<N> for Map<I, F, R>
where
    I: Indicator + Next<N>,
    F: FnMut(I::Output) -> R,
{
    fn next(&mut self, input: N) -> Self::Output {
        (self.f)(self.i.next(input))
    }
}

impl<I, F, R> Reset for Map<I, F, R>
where
    I: Indicator + Reset,
    F: FnMut(I::Output) -> R,
{
    fn reset(&mut self) {
        self.i.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::*;
    use crate::{Price, Sma};
    use once_cell::sync::Lazy as SyncLazy;

    #[derive(Clone)]
    struct TestItem(f64);
    impl Price for TestItem {
        fn price(&self) -> f64 {
            self.0
        }
    }
    const PERIOD: usize = 5;
    fn f(x: f64) -> f64 {
        x * x
    }
    static INPUTS: SyncLazy<Box<[TestItem]>> = SyncLazy::new(|| {
        [100.0, 101.0, 101.0, 102.0, 102.0, 102.0]
            .into_iter()
            .map(TestItem)
            .collect::<Vec<_>>()
            .into_boxed_slice()
    });
    static OUTPUTS: SyncLazy<Box<[f64]>> = SyncLazy::new(|| {
        [100.0, 100.2, 100.4, 100.8, 101.2, 101.6]
            .into_iter()
            .map(f)
            .collect::<Vec<_>>()
            .into_boxed_slice()
    });

    test_indicator! {
        new: Sma::new(PERIOD).map(|sma| Map::new(sma, f)),
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

    #[test]
    fn map() -> Result<(), Box<dyn std::error::Error>> {
        let period = 10;
        let f = |x: f64| x * x;

        let mut sma = crate::Sma::new(period)?;
        let mut sma_map = Map::new(crate::Sma::new(period)?, f);

        for x in RANDOM_DATA.iter().map(|x| x.price()) {
            let v_sma = sma.next(x);
            let v_sma_map = sma_map.next(x);

            assert_eq!(f(v_sma), v_sma_map)
        }

        Ok(())
    }
}
