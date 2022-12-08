use std::collections::VecDeque;

use crate::{Current, Indicator, Next, Reset};

/// Create a new indicator that outputs the past N output values ​​of the inner indicator.
pub struct Window<'a, Inner: Indicator>
where
    Self: 'a,
{
    inner: Inner,
    window_size: usize,
    ring: VecDeque<Inner::Output>,
    buf: Box<[&'a Inner::Output]>,
    is_first: bool,
}
impl<'a, Inner: Indicator> Window<'a, Inner>
where
    Self: 'a,
{
    pub(crate) fn new(inner: Inner, window_size: usize) -> Self {
        let buf = unsafe {
            let a = core::mem::MaybeUninit::<&'a Inner::Output>::zeroed();
            vec![a.assume_init(); window_size].into_boxed_slice()
        };
        Self {
            inner,
            window_size,
            ring: VecDeque::with_capacity(window_size),
            buf,
            is_first: true,
        }
    }

    /// Take out the inner indicator that composes this indicator
    pub fn decompose(self) -> Inner {
        self.inner
    }

    fn update_window(&mut self) {
        let buf = self.buf.as_mut();

        let n = self.window_size - self.ring.len();
        let mut iter = self.ring.iter();

        unsafe {
            if n == 0 {
                for (i, r) in iter.enumerate() {
                    buf[i] = &*(r as *const _);
                }
            } else {
                let a = iter.next().unwrap();
                for i in 0..=n {
                    buf[i] = &*(a as *const _);
                }
                for (i, r) in iter.enumerate() {
                    buf[i + n + 1] = &*(r as *const _);
                }
            }
        }
    }

    fn window(&self) -> <Self as Indicator>::Output {
        unsafe { &*(&self.buf[..] as *const _) }
    }
}
impl<'a, Inner: Indicator> Indicator for Window<'a, Inner>
where
    Self: 'a,
{
    type Output = &'a [&'a Inner::Output];
}
impl<'a, Inner: Indicator, N> Next<N> for Window<'a, Inner>
where
    Self: 'a,
    Inner: Next<N>,
{
    fn next(&mut self, input: N) -> Self::Output {
        let value = self.inner.next(input);
        if 0 < self.window_size {
            if self.window_size <= self.ring.len() {
                let _ = self.ring.pop_front();
            }
            self.ring.push_back(value);
            self.update_window();
        }

        self.is_first = false;
        self.window()
    }
}
impl<'a, Inner: Indicator> Current for Window<'a, Inner>
where
    Self: 'a,
    Inner: Current,
{
    fn current(&self) -> Option<Self::Output> {
        if self.is_first {
            None
        } else {
            self.window().into()
        }
    }
}
impl<'a, Inner: Indicator> Reset for Window<'a, Inner>
where
    Self: 'a,
    Inner: Reset,
{
    fn reset(&mut self) {
        self.inner.reset();
        self.ring.clear();
        self.is_first = true;
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
    const WINDOW_SIZE: usize = 3;
    static INPUTS: SyncLazy<Box<[TestItem]>> = SyncLazy::new(|| {
        [100.0, 101.0, 101.0, 102.0, 102.0, 102.0]
            .into_iter()
            .map(TestItem)
            .collect::<Vec<_>>()
            .into_boxed_slice()
    });
    static OUTPUTS: &[f64] = &[100.0, 100.2, 100.4, 100.8, 101.2, 101.6];
    static OUTPUTS_WINDOWS: &[&[&f64]] = &[
        &[&OUTPUTS[0], &OUTPUTS[0], &OUTPUTS[0]],
        &[&OUTPUTS[0], &OUTPUTS[0], &OUTPUTS[1]],
        &[&OUTPUTS[0], &OUTPUTS[1], &OUTPUTS[2]],
        &[&OUTPUTS[1], &OUTPUTS[2], &OUTPUTS[3]],
        &[&OUTPUTS[2], &OUTPUTS[3], &OUTPUTS[4]],
        &[&OUTPUTS[3], &OUTPUTS[4], &OUTPUTS[5]],
    ];

    #[test]
    fn next() -> crate::Result<()> {
        let sma = Sma::new(PERIOD)?;
        let mut window = Window::new(sma, WINDOW_SIZE);

        assert_eq!(window.next(&INPUTS[0]), OUTPUTS_WINDOWS[0]);
        assert_eq!(window.next(&INPUTS[1]), OUTPUTS_WINDOWS[1]);
        assert_eq!(window.next(&INPUTS[2]), OUTPUTS_WINDOWS[2]);
        assert_eq!(window.next(&INPUTS[3]), OUTPUTS_WINDOWS[3]);
        assert_eq!(window.next(&INPUTS[4]), OUTPUTS_WINDOWS[4]);
        assert_eq!(window.next(&INPUTS[5]), OUTPUTS_WINDOWS[5]);

        Ok(())
    }

    #[test]
    fn current() -> crate::Result<()> {
        let sma = Sma::new(PERIOD)?;
        let mut window = Window::new(sma, WINDOW_SIZE);

        for input in RANDOM_DATA.iter() {
            let correct = window.next(input);
            assert_eq!(window.current(), Some(correct));
        }

        Ok(())
    }

    #[test]
    fn reset() -> crate::Result<()> {
        let sma = Sma::new(PERIOD)?;
        let mut window = Window::new(sma, WINDOW_SIZE);

        let mut v: Vec<Vec<f64>> = Vec::with_capacity(RANDOM_DATA.len());

        for input in RANDOM_DATA.iter() {
            let correct = window.next(input);
            v.push(correct.iter().map(|x| **x).collect());
        }

        window.reset();

        for (i, input) in RANDOM_DATA.iter().enumerate() {
            let value = window.next(input);
            let correct = v[i].iter().collect::<Vec<&f64>>();
            assert_eq!(value, &correct[..]);
        }

        Ok(())
    }
}
