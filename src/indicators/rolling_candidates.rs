use alloc::collections::VecDeque;

use crate::{Result, try_deque_with_capacity};

#[derive(Debug, Clone, Copy)]
pub(super) enum Direction {
    Maximum,
    Minimum,
}

#[derive(Debug, Clone)]
pub(super) struct RollingCandidates {
    direction: Direction,
    entries: VecDeque<(usize, f64)>,
}

impl RollingCandidates {
    pub(super) fn new(direction: Direction, capacity: usize) -> Result<Self> {
        Ok(Self {
            direction,
            entries: try_deque_with_capacity(capacity)?,
        })
    }

    pub(super) fn clear(&mut self) {
        self.entries.clear();
    }

    pub(super) fn front(&self) -> Option<&(usize, f64)> {
        self.entries.front()
    }

    pub(super) fn expire(&mut self, position: usize, period: usize) {
        while self.entries.front().is_some_and(|(candidate_position, _)| {
            position.wrapping_sub(*candidate_position) >= period
        }) {
            self.entries.pop_front();
        }
    }

    pub(super) fn push(&mut self, position: usize, value: f64) {
        while self
            .entries
            .back()
            .is_some_and(|(_, candidate)| match self.direction {
                Direction::Maximum => *candidate <= value,
                Direction::Minimum => *candidate >= value,
            })
        {
            self.entries.pop_back();
        }
        self.entries.push_back((position, value));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maximum_candidates_keep_latest_ties_and_expire_with_wrapping_positions() -> Result<()> {
        let mut candidates = RollingCandidates::new(Direction::Maximum, 4)?;
        candidates.push(usize::MAX - 1, 3.0);
        candidates.push(usize::MAX, 5.0);
        candidates.push(0, 5.0);

        assert_eq!(candidates.front(), Some(&(0, 5.0)));

        candidates.expire(1, 3);
        assert_eq!(candidates.front(), Some(&(0, 5.0)));

        candidates.expire(3, 3);
        assert_eq!(candidates.front(), None);

        Ok(())
    }

    #[test]
    fn minimum_candidates_keep_latest_ties_and_remove_dominated_values() -> Result<()> {
        let mut candidates = RollingCandidates::new(Direction::Minimum, 4)?;
        candidates.push(0, 3.0);
        candidates.push(1, 2.0);
        candidates.push(2, 2.0);

        assert_eq!(candidates.front(), Some(&(2, 2.0)));

        candidates.push(3, 4.0);
        assert_eq!(candidates.front(), Some(&(2, 2.0)));

        Ok(())
    }
}
