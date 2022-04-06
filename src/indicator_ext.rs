use crate::{
    indicator_iterator::IndicatorIterator,
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
    fn mature(self, period: usize) -> Mature<Self> {
        Mature::new(self, period)
    }

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
        F: FnMut(Self::Output) -> R,
    {
        Map::new(self, f)
    }

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
        Inner: Indicator<Output = N>,
    {
        Composition::new(inner, self)
    }

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
        Outer: Indicator + Next<Self::Output>,
    {
        Composition::new(self, outer)
    }

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
        Companion: Indicator,
    {
        Together::new(self, companion)
    }

    /// Convert indicator to iterator
    ///
    /// # Example
    ///
    /// ```
    /// # use indicator::*;
    /// # fn main () {
    /// use std::f64::consts::PI;
    ///
    /// let sma = Sma::new(2).unwrap();
    ///
    /// let input_iter = (0..100).map(|n| f64::sin(PI / 10.0 * n as f64));
    /// let mut sma_iter = sma.iter_over(input_iter);
    ///
    /// while let Some(value) = sma_iter.next() {
    ///     println!("{value}");
    /// }
    /// # }
    /// ```
    fn iter_over<InputIterator>(
        self,
        input_iterator: InputIterator,
    ) -> IndicatorIterator<Self, InputIterator>
    where
        InputIterator: Iterator,
        Self: Next<InputIterator::Item>,
    {
        IndicatorIterator::new(self, input_iterator)
    }

    #[cfg(feature = "stream")]
    /// Convert indicator to stream
    ///
    /// # Example
    ///
    /// ```
    /// # use futures_executor::LocalPool;
    /// # use futures_util::task::SpawnExt;
    /// # use indicator::*;
    /// # fn main () {
    /// # let mut pool = LocalPool::new();
    /// # let spawner = pool.spawner();
    /// #
    /// use futures_util::{stream, StreamExt};
    /// use std::f64::consts::PI;
    ///
    /// # spawner.spawn(async {
    /// let sma = Sma::new(2).unwrap();
    ///
    /// let input_iter = (0..100).map(|n| f64::sin(PI / 10.0 * n as f64));
    /// let input_stream = stream::iter(input_iter);
    /// let mut sma_stream = sma.iter_over_stream(input_stream);
    ///
    /// while let Some(value) = sma_stream.next().await {
    ///     println!("{value}");
    /// }
    /// # }).unwrap();
    /// # pool.run();
    /// # }
    /// ```
    fn iter_over_stream<N, InputStream>(
        self,
        input_stream: InputStream,
    ) -> crate::indicator_stream::IndicatorStream<Self, InputStream>
    where
        Self: Next<N>,
        InputStream: futures_core::Stream<Item = N>,
    {
        crate::indicator_stream::IndicatorStream::new(self, input_stream)
    }
}

impl<I> IndicatorExt for I where I: Indicator + Sized {}
