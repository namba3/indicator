use crate::{Indicator, Next};

/// Applies an indicator to each item from an input iterator.
///
/// The adapter preserves size hints, exact lengths, and fused behavior when
/// those guarantees are provided by the input iterator.
pub struct IndicatorIterator<Inner, InputIterator>
where
    Inner: Indicator + Next<InputIterator::Item>,
    InputIterator: Iterator,
{
    inner: Inner,
    input_iterator: InputIterator,
}

impl<Inner, InputIterator> IndicatorIterator<Inner, InputIterator>
where
    Inner: Indicator + Next<InputIterator::Item>,
    InputIterator: Iterator,
{
    pub(crate) fn new(inner: Inner, input_iterator: InputIterator) -> Self {
        Self {
            inner,
            input_iterator,
        }
    }

    pub fn decompose(self) -> Inner {
        self.inner
    }
}

impl<Inner, InputIterator> Iterator for IndicatorIterator<Inner, InputIterator>
where
    Inner: Indicator + Next<InputIterator::Item>,
    InputIterator: Iterator,
{
    type Item = Inner::Output;
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        match self.input_iterator.next() {
            Some(input) => Some(self.inner.next(input)),
            _ => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.input_iterator.size_hint()
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let Self {
            mut inner,
            input_iterator,
        } = self;
        input_iterator.fold(init, |acc, input| f(acc, inner.next(input)))
    }
}

impl<Inner, InputIterator> ExactSizeIterator for IndicatorIterator<Inner, InputIterator>
where
    Inner: Indicator + Next<InputIterator::Item>,
    InputIterator: ExactSizeIterator,
{
}

impl<Inner, InputIterator> core::iter::FusedIterator for IndicatorIterator<Inner, InputIterator>
where
    Inner: Indicator + Next<InputIterator::Item>,
    InputIterator: core::iter::FusedIterator,
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::*;
    use crate::{IndicatorExt, Sma};

    #[test]
    fn test() -> crate::Result<()> {
        let mut sma = Sma::new(4)?;
        let mut iter = IndicatorIterator::new(sma.clone(), RANDOM_DATA.iter());

        for input in RANDOM_DATA.iter() {
            let correct = sma.next(input);
            assert_eq!(iter.next().unwrap(), correct)
        }

        assert_eq!(iter.next(), None);

        Ok(())
    }

    #[test]
    fn preserves_exact_size_and_fused_iterator_guarantees() -> crate::Result<()> {
        fn assert_fused<I: core::iter::FusedIterator>(_: &I) {}

        let sma = Sma::new(1)?;
        let mut iter = IndicatorIterator::new(sma, [1.0, 2.0, 3.0].into_iter());

        assert_fused(&iter);
        assert_eq!(iter.len(), 3);
        assert_eq!(iter.size_hint(), (3, Some(3)));
        assert_eq!(iter.next(), Some(1.0));
        assert_eq!(iter.len(), 2);
        assert_eq!(iter.collect::<Vec<_>>(), [2.0, 3.0]);

        let sma = Sma::new(1)?;
        let mut iter = IndicatorIterator::new(sma, [4.0].into_iter());
        assert_eq!(iter.next(), Some(4.0));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);

        Ok(())
    }

    #[test]
    fn fold_applies_the_indicator_to_every_input_in_order() -> crate::Result<()> {
        let sma = Sma::new(3)?;
        let sum = sma.iter_over([1.0, 2.0, 3.0, 4.0].into_iter()).sum::<f64>();

        let mut expected_sma = Sma::new(3)?;
        let expected_sum = [1.0, 2.0, 3.0, 4.0]
            .into_iter()
            .map(|input| expected_sma.next(input))
            .sum::<f64>();

        assert_eq!(sum, expected_sum);
        Ok(())
    }
}
