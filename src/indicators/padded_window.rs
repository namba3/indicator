use alloc::collections::VecDeque;

struct PaddedWindowIter<T, Values> {
    padding_value: Option<T>,
    padding_remaining: usize,
    padding_before_values: bool,
    values: Values,
}

impl<T: Copy, Values> Iterator for PaddedWindowIter<T, Values>
where
    Values: ExactSizeIterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.padding_before_values && 0 < self.padding_remaining {
            self.padding_remaining -= 1;
            return self.padding_value;
        }

        if let Some(value) = self.values.next() {
            return Some(value);
        }

        if 0 < self.padding_remaining {
            self.padding_remaining -= 1;
            return self.padding_value;
        }

        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.padding_remaining + self.values.len();
        (len, Some(len))
    }
}

impl<T: Copy, Values> ExactSizeIterator for PaddedWindowIter<T, Values> where
    Values: ExactSizeIterator<Item = T>
{
}

impl<T: Copy, Values> core::iter::FusedIterator for PaddedWindowIter<T, Values> where
    Values: ExactSizeIterator<Item = T> + core::iter::FusedIterator
{
}

fn padded_window<T: Copy, Values>(
    values: Values,
    padding_value: Option<T>,
    missing: usize,
    padding_before_values: bool,
) -> PaddedWindowIter<T, Values>
where
    Values: ExactSizeIterator<Item = T>,
{
    PaddedWindowIter {
        padding_value,
        padding_remaining: if padding_value.is_some() { missing } else { 0 },
        padding_before_values,
        values,
    }
}

pub(super) fn values<T: Copy>(
    ring: &VecDeque<T>,
    period: usize,
) -> impl ExactSizeIterator<Item = T> + core::iter::FusedIterator + '_ {
    padded_window(
        ring.iter().copied(),
        ring.front().copied(),
        period - ring.len(),
        true,
    )
}

pub(super) fn newest_first<T: Copy>(
    ring: &VecDeque<T>,
    period: usize,
) -> impl ExactSizeIterator<Item = T> + core::iter::FusedIterator + '_ {
    padded_window(
        ring.iter().copied(),
        ring.back().copied(),
        period - ring.len(),
        false,
    )
}

pub(super) fn oldest_first_from_newest<T: Copy>(
    ring: &VecDeque<T>,
    period: usize,
) -> impl ExactSizeIterator<Item = T> + core::iter::FusedIterator + '_ {
    padded_window(
        ring.iter().rev().copied(),
        ring.back().copied(),
        period - ring.len(),
        true,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_repeat_the_first_observation_until_the_window_is_full() {
        let mut ring = VecDeque::new();
        let iter = values(&ring, 3);
        assert_eq!(iter.len(), 0);
        assert_eq!(iter.collect::<Vec<_>>(), []);

        ring.push_back(2.0);
        let mut iter = values(&ring, 3);
        assert_eq!(iter.len(), 3);
        assert_eq!(iter.next(), Some(2.0));
        assert_eq!(iter.len(), 2);
        assert_eq!(iter.collect::<Vec<_>>(), [2.0, 2.0]);

        ring.push_back(4.0);
        assert_eq!(values(&ring, 3).collect::<Vec<_>>(), [2.0, 2.0, 4.0]);

        ring.push_back(6.0);
        assert_eq!(values(&ring, 3).collect::<Vec<_>>(), [2.0, 4.0, 6.0]);
    }

    #[test]
    fn newest_first_repeats_the_oldest_observation_at_the_end() {
        let mut ring = VecDeque::new();
        assert_eq!(newest_first(&ring, 3).collect::<Vec<_>>(), []);

        ring.push_front(2.0);
        assert_eq!(newest_first(&ring, 3).collect::<Vec<_>>(), [2.0, 2.0, 2.0]);

        ring.push_front(4.0);
        assert_eq!(newest_first(&ring, 3).collect::<Vec<_>>(), [4.0, 2.0, 2.0]);

        ring.push_front(6.0);
        assert_eq!(newest_first(&ring, 3).collect::<Vec<_>>(), [6.0, 4.0, 2.0]);
    }

    #[test]
    fn oldest_first_from_newest_repeats_padding_before_actual_values() {
        let mut ring = VecDeque::new();
        assert_eq!(oldest_first_from_newest(&ring, 3).collect::<Vec<_>>(), []);

        ring.push_front(2.0);
        assert_eq!(
            oldest_first_from_newest(&ring, 3).collect::<Vec<_>>(),
            [2.0, 2.0, 2.0]
        );

        ring.push_front(4.0);
        assert_eq!(
            oldest_first_from_newest(&ring, 3).collect::<Vec<_>>(),
            [2.0, 2.0, 4.0]
        );

        ring.push_front(6.0);
        assert_eq!(
            oldest_first_from_newest(&ring, 3).collect::<Vec<_>>(),
            [2.0, 4.0, 6.0]
        );
    }

    #[test]
    fn padded_iterators_report_exact_lengths_and_are_fused() {
        fn assert_behavior<I: ExactSizeIterator<Item = f64> + core::iter::FusedIterator>(
            mut iter: I,
        ) {
            assert_eq!(iter.len(), 4);
            for _ in 0..4 {
                assert!(iter.next().is_some());
                assert_eq!(iter.size_hint().0, iter.len());
            }
            assert_eq!(iter.next(), None);
            assert_eq!(iter.next(), None);
        }

        let mut ring = VecDeque::from([2.0, 4.0]);
        assert_behavior(values(&ring, 4));
        assert_behavior(newest_first(&ring, 4));
        assert_behavior(oldest_first_from_newest(&ring, 4));

        ring.clear();
        assert_eq!(values(&ring, 4).len(), 0);
        assert_eq!(newest_first(&ring, 4).len(), 0);
        assert_eq!(oldest_first_from_newest(&ring, 4).len(), 0);
    }
}
