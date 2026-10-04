use alloc::collections::VecDeque;

pub(super) fn values<T: Copy>(ring: &VecDeque<T>, period: usize) -> impl Iterator<Item = T> + '_ {
    let missing = period - ring.len();
    ring.front()
        .copied()
        .into_iter()
        .flat_map(move |first| core::iter::repeat_n(first, missing))
        .chain(ring.iter().copied())
}

pub(super) fn newest_first<T: Copy>(
    ring: &VecDeque<T>,
    period: usize,
) -> impl Iterator<Item = T> + '_ {
    let missing = period - ring.len();
    ring.iter().copied().chain(
        ring.back()
            .copied()
            .into_iter()
            .flat_map(move |oldest| core::iter::repeat_n(oldest, missing)),
    )
}

pub(super) fn oldest_first_from_newest<T: Copy>(
    ring: &VecDeque<T>,
    period: usize,
) -> impl Iterator<Item = T> + '_ {
    let missing = period - ring.len();
    ring.back()
        .copied()
        .into_iter()
        .flat_map(move |oldest| core::iter::repeat_n(oldest, missing))
        .chain(ring.iter().rev().copied())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_repeat_the_first_observation_until_the_window_is_full() {
        let mut ring = VecDeque::new();
        assert_eq!(values(&ring, 3).collect::<Vec<_>>(), []);

        ring.push_back(2.0);
        assert_eq!(values(&ring, 3).collect::<Vec<_>>(), [2.0, 2.0, 2.0]);

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
}
