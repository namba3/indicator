use alloc::collections::VecDeque;

pub(super) fn values<T: Copy>(ring: &VecDeque<T>, period: usize) -> impl Iterator<Item = T> + '_ {
    let missing = period - ring.len();
    ring.front()
        .copied()
        .into_iter()
        .flat_map(move |first| core::iter::repeat_n(first, missing))
        .chain(ring.iter().copied())
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
}
