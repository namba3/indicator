use alloc::{collections::VecDeque, vec::Vec};

use crate::{Current, Indicator, Next, Reset, Result, try_deque_with_capacity};

struct WindowIter<'a, T, Values> {
    first: Option<&'a T>,
    padding_remaining: usize,
    values: Values,
}

impl<'a, T, Values> Iterator for WindowIter<'a, T, Values>
where
    Values: ExactSizeIterator<Item = &'a T>,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if 0 < self.padding_remaining {
            self.padding_remaining -= 1;
            self.first
        } else {
            self.values.next()
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.padding_remaining + self.values.len();
        (len, Some(len))
    }
}

impl<'a, T, Values> ExactSizeIterator for WindowIter<'a, T, Values> where
    Values: ExactSizeIterator<Item = &'a T>
{
}

impl<'a, T, Values> core::iter::FusedIterator for WindowIter<'a, T, Values> where
    Values: ExactSizeIterator<Item = &'a T> + core::iter::FusedIterator
{
}

/// Create a new indicator that outputs the past N output values ​​of the inner indicator.
pub struct Window<Inner: Indicator> {
    inner: Inner,
    window_size: usize,
    ring: VecDeque<Inner::Output>,
    is_first: bool,
}
impl<Inner: Indicator> Window<Inner> {
    pub(crate) fn new(inner: Inner, window_size: usize) -> Self {
        Self::try_new(inner, window_size).expect("window buffer allocation failed")
    }

    pub(crate) fn try_new(inner: Inner, window_size: usize) -> Result<Self> {
        Ok(Self {
            inner,
            window_size,
            ring: try_deque_with_capacity(window_size)?,
            is_first: true,
        })
    }

    /// Take out the inner indicator that composes this indicator
    pub fn decompose(self) -> Inner {
        self.inner
    }

    /// Iterate over the most recent outputs without allocating a snapshot.
    ///
    /// Before the first input, and after `reset`, the iterator is empty. Once
    /// values are available, missing entries at the start repeat the first
    /// output, matching the snapshots returned by [`Next::next`].
    pub fn iter(
        &self,
    ) -> impl ExactSizeIterator<Item = &Inner::Output> + core::iter::FusedIterator + '_ {
        let first = self.ring.front();
        let padding_remaining = if first.is_some() && !self.is_first {
            self.window_size - self.ring.len()
        } else {
            0
        };

        WindowIter {
            first,
            padding_remaining,
            values: self.ring.iter(),
        }
    }

    /// Advance the inner indicator without creating an owned window snapshot.
    ///
    /// Read the updated window through [`iter`](Self::iter). This can avoid the
    /// allocation and output cloning performed by [`Next::next`].
    pub fn advance<N>(&mut self, input: N)
    where
        Inner: Next<N>,
    {
        let value = self.inner.next(input);
        if 0 < self.window_size {
            if self.window_size <= self.ring.len() {
                let _ = self.ring.pop_front();
            }
            self.ring.push_back(value);
        }

        self.is_first = false;
    }

    fn window(&self) -> Vec<Inner::Output>
    where
        Inner::Output: Clone,
    {
        let missing = self.window_size - self.ring.len();
        let mut window = Vec::with_capacity(self.window_size);

        if let Some(first) = self.ring.front() {
            for _ in 0..missing {
                window.push(first.clone());
            }
        }

        window.extend(self.ring.iter().cloned());
        window
    }
}
impl<Inner: Indicator> Indicator for Window<Inner> {
    type Output = Vec<Inner::Output>;
}
impl<Inner: Indicator, N> Next<N> for Window<Inner>
where
    Inner: Next<N>,
    Inner::Output: Clone,
{
    fn next(&mut self, input: N) -> Self::Output {
        self.advance(input);
        self.window()
    }
}
impl<Inner: Indicator> Current for Window<Inner>
where
    Inner: Current,
    Inner::Output: Clone,
{
    fn current(&self) -> Option<Self::Output> {
        if self.is_first {
            None
        } else {
            self.window().into()
        }
    }
}
impl<Inner: Indicator> Reset for Window<Inner>
where
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
    use crate::{IndicatorExt, Price, Sma};
    use std::sync::LazyLock as SyncLazy;

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
    static OUTPUTS_WINDOWS: &[[f64; WINDOW_SIZE]] = &[
        [100.0, 100.0, 100.0],
        [100.0, 100.0, 100.2],
        [100.0, 100.2, 100.4],
        [100.2, 100.4, 100.8],
        [100.4, 100.8, 101.2],
        [100.8, 101.2, 101.6],
    ];

    #[test]
    fn next() -> crate::Result<()> {
        let sma = Sma::new(PERIOD)?;
        let mut window = Window::new(sma, WINDOW_SIZE);

        assert_eq!(window.next(&INPUTS[0]).as_slice(), &OUTPUTS_WINDOWS[0]);
        assert_eq!(window.next(&INPUTS[1]).as_slice(), &OUTPUTS_WINDOWS[1]);
        assert_eq!(window.next(&INPUTS[2]).as_slice(), &OUTPUTS_WINDOWS[2]);
        assert_eq!(window.next(&INPUTS[3]).as_slice(), &OUTPUTS_WINDOWS[3]);
        assert_eq!(window.next(&INPUTS[4]).as_slice(), &OUTPUTS_WINDOWS[4]);
        assert_eq!(window.next(&INPUTS[5]).as_slice(), &OUTPUTS_WINDOWS[5]);

        Ok(())
    }

    #[test]
    fn next_results_are_independent_snapshots() -> crate::Result<()> {
        let sma = Sma::new(1)?;
        let mut window = Window::new(sma, 2);

        let first = window.next(1.0);
        let second = window.next(2.0);

        assert_eq!(first, [1.0, 1.0]);
        assert_eq!(second, [1.0, 2.0]);

        Ok(())
    }

    #[test]
    fn current_returns_an_independent_snapshot() -> crate::Result<()> {
        let sma = Sma::new(1)?;
        let mut window = Window::new(sma, 2);

        assert_eq!(window.current(), None);
        window.next(1.0);
        let first_current = window.current().unwrap();
        window.next(2.0);

        assert_eq!(first_current, [1.0, 1.0]);
        assert_eq!(window.current(), Some(vec![1.0, 2.0]));

        window.reset();
        assert_eq!(window.current(), None);

        Ok(())
    }

    #[test]
    fn zero_sized_window_returns_empty_snapshots() -> crate::Result<()> {
        let sma = Sma::new(1)?;
        let mut window = Window::new(sma, 0);

        assert_eq!(window.next(1.0), Vec::<f64>::new());
        assert_eq!(window.current(), Some(Vec::<f64>::new()));
        assert_eq!(window.next(2.0), Vec::<f64>::new());

        Ok(())
    }

    #[test]
    fn current() -> crate::Result<()> {
        let sma = Sma::new(PERIOD)?;
        let mut window = Window::new(sma, WINDOW_SIZE);

        for input in RANDOM_DATA.iter() {
            let correct = window.next(input);
            assert_eq!(window.current(), Some(correct.clone()));
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
            v.push(correct);
        }

        window.reset();

        for (i, input) in RANDOM_DATA.iter().enumerate() {
            let value = window.next(input);
            assert_eq!(value, v[i]);
        }

        Ok(())
    }

    #[test]
    fn iter_borrows_padded_and_rolling_outputs_without_changing_snapshot_behavior()
    -> crate::Result<()> {
        let mut window = Sma::new(1)?.window(3);
        assert_eq!(window.iter().count(), 0);
        assert_eq!(window.iter().len(), 0);
        assert_eq!(window.current(), None);

        assert_eq!(window.next(1.0), [1.0, 1.0, 1.0]);
        assert_eq!(window.iter().len(), 3);
        assert_eq!(window.iter().copied().collect::<Vec<_>>(), [1.0, 1.0, 1.0]);

        assert_eq!(window.next(2.0), [1.0, 1.0, 2.0]);
        assert_eq!(window.iter().len(), 3);
        assert_eq!(window.iter().copied().collect::<Vec<_>>(), [1.0, 1.0, 2.0]);

        assert_eq!(window.next(3.0), [1.0, 2.0, 3.0]);
        assert_eq!(window.iter().copied().collect::<Vec<_>>(), [1.0, 2.0, 3.0]);

        window.reset();
        assert_eq!(window.iter().count(), 0);
        assert_eq!(window.iter().len(), 0);
        assert_eq!(window.current(), None);

        Ok(())
    }

    #[test]
    fn iter_is_empty_for_a_zero_sized_window() -> crate::Result<()> {
        let mut window = Sma::new(1)?.window(0);
        assert_eq!(window.iter().count(), 0);
        assert_eq!(window.iter().len(), 0);

        window.next(1.0);
        assert_eq!(window.iter().count(), 0);
        assert_eq!(window.iter().len(), 0);

        Ok(())
    }

    #[test]
    fn iter_reports_remaining_length_and_is_fused() -> crate::Result<()> {
        let mut window = Sma::new(1)?.window(3);
        window.advance(7.0);

        let mut iter = window.iter();
        assert_eq!(iter.len(), 3);
        assert_eq!(iter.next(), Some(&7.0));
        assert_eq!(iter.len(), 2);
        assert_eq!(iter.next(), Some(&7.0));
        assert_eq!(iter.len(), 1);
        assert_eq!(iter.next(), Some(&7.0));
        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);

        Ok(())
    }

    #[test]
    fn advance_matches_snapshots_and_reset_behavior() -> crate::Result<()> {
        let mut snapshots = Sma::new(1)?.window(3);
        let mut borrowed = Sma::new(1)?.window(3);

        for input in [1.0, 2.0, 3.0, 4.0] {
            let expected = snapshots.next(input);
            borrowed.advance(input);

            assert_eq!(borrowed.iter().copied().collect::<Vec<_>>(), expected);
            assert_eq!(borrowed.current(), Some(expected));
        }

        borrowed.reset();
        assert_eq!(borrowed.iter().count(), 0);
        assert_eq!(borrowed.current(), None);
        Ok(())
    }

    #[test]
    fn advance_updates_inner_state_for_a_zero_sized_window() -> crate::Result<()> {
        let mut window = Sma::new(1)?.window(0);

        window.advance(42.0);

        assert_eq!(window.iter().count(), 0);
        assert_eq!(window.current(), Some(Vec::new()));
        assert_eq!(window.decompose().current(), Some(42.0));
        Ok(())
    }

    #[test]
    fn try_window_reports_capacity_overflow() -> crate::Result<()> {
        let result = Sma::new(1)?.try_window(usize::MAX);

        assert!(matches!(result, Err(crate::Error::AllocationFailed)));
        Ok(())
    }

    #[test]
    fn try_window_preserves_window_behavior() -> crate::Result<()> {
        let mut window = Sma::new(1)?.try_window(2)?;

        assert_eq!(window.next(1.0), [1.0, 1.0]);
        assert_eq!(window.next(2.0), [1.0, 2.0]);
        assert_eq!(window.iter().copied().collect::<Vec<_>>(), [1.0, 2.0]);
        Ok(())
    }
}
