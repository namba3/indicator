use crate::{Current, Indicator, Next, Reset};

///
pub struct Mature<I: Indicator> {
    i: I,
    period: usize,
    cnt: usize,
}
impl<I: Indicator> Mature<I> {
    pub(crate) fn new(i: I, period: usize) -> Self {
        Self {
            i,
            period,
            cnt: period + 1,
        }
    }
}
impl<I: Indicator> Indicator for Mature<I> {
    type Output = Option<I::Output>;
}
impl<I: Indicator, N> Next<N> for Mature<I>
where
    I: Next<N>,
{
    fn next(&mut self, input: N) -> Self::Output {
        let output = self.i.next(input);
        if self.cnt <= 1 {
            self.cnt = 0;
            Some(output)
        } else {
            self.cnt -= 1;
            None
        }
    }
}
impl<I: Indicator> Current for Mature<I>
where
    I: Current,
{
    fn current(&self) -> Option<Self::Output> {
        if self.cnt <= 0 {
            self.i.current().into()
        } else {
            None
        }
    }
}
impl<I: Indicator> Reset for Mature<I>
where
    I: Reset,
{
    fn reset(&mut self) {
        self.i.reset();
        self.cnt = self.period + 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::*;
    use crate::{Price, Sma};
    use std::lazy::SyncLazy;

    #[derive(Clone)]
    struct TestItem(f64);
    impl Price for TestItem {
        fn price(&self) -> f64 {
            self.0
        }
    }
    const PERIOD: usize = 5;
    const MATURE_PERIOD: usize = 3;
    static INPUTS: SyncLazy<Box<[TestItem]>> = SyncLazy::new(|| {
        [100.0, 101.0, 101.0, 102.0, 102.0, 102.0]
            .into_iter()
            .map(TestItem)
            .collect::<Vec<_>>()
            .into_boxed_slice()
    });
    static OUTPUTS: SyncLazy<Box<[Option<f64>]>> = SyncLazy::new(|| {
        [100.0, 100.2, 100.4, 100.8, 101.2, 101.6]
            .into_iter()
            .enumerate()
            .map(|(i, x)| if MATURE_PERIOD <= i { x.into() } else { None })
            .collect::<Vec<Option<_>>>()
            .into_boxed_slice()
    });

    test_indicator! {
        new: Sma::new(PERIOD).map(|sma| Mature::new(sma, MATURE_PERIOD)),
        inputs: INPUTS.iter().map(|x| x.price()),
        outputs: OUTPUTS.iter().copied(),
        additional_tests: {
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
    fn mature() -> Result<(), Box<dyn std::error::Error>> {
        let period = PERIOD;
        let mature_period = MATURE_PERIOD;

        let mut sma = crate::Sma::new(period)?;
        let mut sma_mature = Mature::new(crate::Sma::new(period)?, mature_period);

        for x in RANDOM_DATA[..mature_period].iter().map(|x| x.price()) {
            let _v_sma = sma.next(x);
            let v_sma_mature = sma_mature.next(x);

            assert_eq!(v_sma_mature, None)
        }

        for x in RANDOM_DATA[mature_period..].iter().map(|x| x.price()) {
            let v_sma = sma.next(x);
            let v_sma_mature = sma_mature.next(x);

            assert_eq!(Some(v_sma), v_sma_mature)
        }

        Ok(())
    }

    #[test]
    fn current() -> crate::Result<()> {
        let mut indicator = Sma::new(PERIOD).map(|sma| Mature::new(sma, MATURE_PERIOD))?;

        let inputs = RANDOM_DATA.iter().map(|x| x.price());

        for x in inputs {
            let value = Next::next(&mut indicator, x);
            let current = indicator.current();
            match value {
                Some(value) => {
                    assert_eq!(Some(value), current.flatten())
                }
                None => {
                    assert!(current.is_none());
                }
            }
        }

        Ok(())
    }
}
