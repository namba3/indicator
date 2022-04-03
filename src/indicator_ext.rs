use crate::{
    operators::{Composition, Map, Mature},
    Indicator,
};

/// Provides extended methods for Indicator.
pub trait IndicatorExt: Indicator + Sized {
    /// Create a new indicator that applies a projection to the output of the indicator.
    fn map<F, R>(self, f: F) -> Map<Self, F, R>
    where
        F: FnMut(Self::Output) -> R;

    /// Mature
    fn mature(self, period: usize) -> Mature<Self>;

    /// Create a new indicator by combining the two indicators in serial.
    fn pushforward<Inner: Indicator<Output = Self::Input>>(
        self,
        inner: Inner,
    ) -> Composition<Inner, Self>;

    /// Create a new indicator by combining the two indicators in serial.
    fn pullback<Outer: Indicator<Input = Self::Output>>(
        self,
        outer: Outer,
    ) -> Composition<Self, Outer>;
}

impl<I: Indicator> IndicatorExt for I {
    fn map<F, R>(self, f: F) -> Map<I, F, R>
    where
        F: FnMut(I::Output) -> R,
    {
        Map::new(self, f)
    }

    fn mature(self, period: usize) -> Mature<I> {
        Mature::new(self, period)
    }

    fn pushforward<Inner: Indicator<Output = Self::Input>>(
        self,
        inner: Inner,
    ) -> Composition<Inner, Self> {
        Composition::new(inner, self)
    }

    fn pullback<Outer: Indicator<Input = Self::Output>>(
        self,
        outer: Outer,
    ) -> Composition<Self, Outer> {
        Composition::new(self, outer)
    }
}
