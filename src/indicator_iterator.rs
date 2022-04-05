use crate::{Indicator, Next};

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
        if let Some(input) = self.input_iterator.next() {
            Some(self.inner.next(input))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::*;
    use crate::Sma;

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
}
