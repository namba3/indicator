use crate::{Current, Indicator, Next, Reset};
use core::ops::Sub;

pub struct Diff<Lhs, Rhs>
where
    Lhs: Indicator,
    Rhs: Indicator<Output = Lhs::Output>,
    Lhs::Output: Sub,
{
    lhs: Lhs,
    rhs: Rhs,
}
impl<Lhs, Rhs> Diff<Lhs, Rhs>
where
    Lhs: Indicator,
    Rhs: Indicator<Output = Lhs::Output>,
    Lhs::Output: Sub,
{
    pub(crate) fn new(lhs: Lhs, rhs: Rhs) -> Self {
        Self { lhs, rhs }
    }
}
impl<Lhs, Rhs> Indicator for Diff<Lhs, Rhs>
where
    Lhs: Indicator,
    Rhs: Indicator<Output = Lhs::Output>,
    Lhs::Output: Sub,
{
    type Output = <Lhs::Output as Sub>::Output;
}

impl<Lhs, Rhs, IL, IR> Next<(IL, IR)> for Diff<Lhs, Rhs>
where
    Lhs: Indicator + Next<IL>,
    Rhs: Indicator<Output = Lhs::Output> + Next<IR>,
    Lhs::Output: Sub,
{
    fn next(&mut self, (input_l, input_r): (IL, IR)) -> Self::Output {
        self.lhs.next(input_l) - self.rhs.next(input_r)
    }
}

impl<Lhs, Rhs> Current for Diff<Lhs, Rhs>
where
    Lhs: Indicator + Current,
    Rhs: Indicator<Output = Lhs::Output> + Current,
    Lhs::Output: Sub,
{
    fn current(&self) -> Option<Self::Output> {
        match (self.lhs.current(), self.rhs.current()) {
            (Some(lhs), Some(rhs)) => (lhs - rhs).into(),
            _ => None,
        }
    }
}
impl<Lhs, Rhs> Reset for Diff<Lhs, Rhs>
where
    Lhs: Indicator + Reset,
    Rhs: Indicator<Output = Lhs::Output> + Reset,
    Lhs::Output: Sub,
{
    fn reset(&mut self) {
        self.lhs.reset();
        self.rhs.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{operators::Identity, test_helper::*, High, Low};
    use std::lazy::SyncLazy;

    static INPUTS: &[(f64, f64)] = &[
        (0.0, 0.0),
        (1.0, 1.0),
        (0.0, 2.0),
        (2.0, 0.0),
        (3.0, 1.0),
        (1.0, 9.0),
    ];
    static OUTPUTS: SyncLazy<Box<[f64]>> =
        SyncLazy::new(|| [0.0, 0.0, -2.0, 2.0, 2.0, -8.0].into());

    test_indicator! {
        new: Ok(Diff::new(Identity::new(), Identity::new())),
        inputs: INPUTS.iter().copied(),
        outputs: OUTPUTS.iter().copied(),
        additional_tests: {
            current: {
                inputs: RANDOM_DATA.iter().map(|x| (x.high(), x.low())),
            },
            reset: {
                inputs: RANDOM_DATA.iter().map(|x| (x.high(), x.low())),
            },
        }
    }
}
