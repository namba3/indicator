use crate::{
    operators::{Composition, Map, Mature, Together},
    Indicator, Next,
};

/// Provides extended methods for Indicator.
pub trait IndicatorExt: Indicator + Sized {
    /// Mature
    ///
    /// # Example
    ///
    /// ```
    /// # use indicator::*;
    /// # fn main () {
    /// let sma = Sma::new(4).unwrap();
    /// let mut sma = sma.mature(3);
    ///
    /// assert_eq!(sma.next(1.0), None);
    /// assert_eq!(sma.next(2.0), None);
    /// assert_eq!(sma.next(1.0), None);
    /// assert_eq!(sma.next(2.0), Some(1.5));
    /// assert_eq!(sma.next(1.0), Some(1.5));
    /// assert_eq!(sma.next(2.0), Some(1.5));
    /// # }
    /// ```
    fn mature(self, period: usize) -> Mature<Self>;

    /// Create a new indicator that applies a functional transformation to the output of the indicator.
    ///
    /// # Example
    ///
    /// ```
    /// # use indicator::*;
    /// # fn main() {
    /// use std::f64::consts::PI;
    ///
    /// let macd = Macd::default();
    /// let mut macd = macd.map(|MacdOutput{ macd, signal: _, histogram: _}| macd);
    ///
    /// for input in (0..100).map(|n| f64::sin(PI / 10.0 * n as f64)) {
    ///     let value: f64 = macd.next(input);
    ///     println!("{value}");
    /// }
    /// # }
    /// ```
    fn map<F, R>(self, f: F) -> Map<Self, F, R>
    where
        F: FnMut(Self::Output) -> R;

    /// Create a new indicator by combining the two indicators in serial.
    ///
    /// # Example
    ///
    /// ```
    /// # use indicator::*;
    /// # fn main () {
    /// use std::f64::consts::PI;
    ///
    /// let sma = Sma::new(2).unwrap();
    /// let rsi = Rsi::new(14).unwrap();
    ///
    /// let mut sma_against_rsi = rsi.pushforward(sma);
    ///
    /// for input in (0..100).map(|n| f64::sin(PI / 10.0 * n as f64)) {
    ///     let value: f64 = sma_against_rsi.next(input);
    ///     println!("{value}");
    /// }
    /// # }
    /// ```
    fn pushforward<N, Inner>(self, inner: Inner) -> Composition<Inner, Self>
    where
        Self: Next<N>,
        Inner: Indicator<Output = N>;

    /// Create a new indicator by combining the two indicators in serial.
    ///
    /// # Example
    ///
    /// ```
    /// # use indicator::*;
    /// # fn main () {
    /// use std::f64::consts::PI;
    ///
    /// let sma = Sma::new(2).unwrap();
    /// let rsi = Rsi::new(14).unwrap();
    ///
    /// let mut sma_against_rsi = sma.pullback(rsi);
    ///
    /// for input in (0..100).map(|n| f64::sin(PI / 10.0 * n as f64)) {
    ///     let value: f64 = sma_against_rsi.next(input);
    ///     println!("{value}");
    /// }
    /// # }
    /// ```
    fn pullback<Outer>(self, outer: Outer) -> Composition<Self, Outer>
    where
        Outer: Indicator + Next<Self::Output>;

    /// Create a new indicator by combining the two indicators in parallel.
    ///
    /// # Example
    ///
    /// ```
    /// # use indicator::*;
    /// # fn main () {
    /// use std::f64::consts::PI;
    ///
    /// let sma = Sma::new(2).unwrap();
    /// let rsi = Rsi::new(14).unwrap();
    ///
    /// let mut sma_and_rsi = sma.together(rsi);
    ///
    /// for input in (0..100).map(|n| f64::sin(PI / 10.0 * n as f64)) {
    ///     let (sma_value, rsi_value) = sma_and_rsi.next(input);
    ///     println!("{sma_value}, {rsi_value}");
    /// }
    /// # }
    /// ```
    fn together<Companion>(self, companion: Companion) -> Together<Self, Companion>
    where
        Companion: Indicator;
}

impl<I> IndicatorExt for I
where
    I: Indicator + Sized,
{
    fn mature(self, period: usize) -> Mature<I> {
        Mature::new(self, period)
    }

    fn map<F, R>(self, f: F) -> Map<I, F, R>
    where
        F: FnMut(I::Output) -> R,
    {
        Map::new(self, f)
    }

    fn pushforward<N, Inner>(self, inner: Inner) -> Composition<Inner, Self>
    where
        Self: Next<N>,
        Inner: Indicator<Output = N>,
    {
        Composition::new(inner, self)
    }

    fn pullback<Outer>(self, outer: Outer) -> Composition<Self, Outer>
    where
        Outer: Indicator + Next<Self::Output>,
    {
        Composition::new(self, outer)
    }

    fn together<Companion>(self, companion: Companion) -> Together<Self, Companion>
    where
        Companion: Indicator,
    {
        Together::new(self, companion)
    }
}
