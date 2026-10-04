//! # Example
//!
//! ```
//! use indicator::*;
//! use std::f64::consts::PI;
//!
//! let mut sma = Sma::new(5).unwrap();
//!
//! for input in (0..100).map(|n| f64::sin(PI / 10.0 * n as f64)) {
//!     let value: f64 = sma.next(input);
//!     println!("{value}");
//! }
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)]
extern crate alloc;

pub(crate) fn validate_period(period: usize, minimum: usize) -> Result<()> {
    if period < minimum {
        Err(InvalidRangeError {
            param: Parameter::new("period", period),
            range: Range::LowerBounded { min: minimum },
        }
        .into())
    } else {
        Ok(())
    }
}

pub(crate) fn try_deque_with_capacity<T>(
    capacity: usize,
) -> Result<alloc::collections::VecDeque<T>> {
    let mut deque = alloc::collections::VecDeque::new();
    deque
        .try_reserve(capacity)
        .map_err(|_| Error::AllocationFailed)?;
    Ok(deque)
}

#[cfg(test)]
#[macro_use]
mod test_helper;

pub mod error;
pub mod indicator_ext;
pub mod indicators;

pub mod indicator_iterator;
pub mod operators;

#[cfg(feature = "stream")]
pub mod indicator_stream;

pub use error::*;
pub use indicator_ext::*;
pub use indicators::*;

/// Indicator
pub trait Indicator {
    type Output;
}

/// Next
pub trait Next<Input>: Indicator {
    fn next(&mut self, input: Input) -> Self::Output;
}

/// Current
pub trait Current: Indicator {
    fn current(&self) -> Option<Self::Output>;
}

/// Reset
pub trait Reset {
    fn reset(&mut self);
}

pub trait High {
    fn high(&self) -> f64;
}
pub trait Low {
    fn low(&self) -> f64;
}
pub trait Open {
    fn open(&self) -> f64;
}
pub trait Close {
    fn close(&self) -> f64;
}
pub trait Volume {
    fn volume(&self) -> f64;
}

pub trait Price {
    fn price(&self) -> f64;
}

pub trait Candlestick: High + Low + Open + Close + Volume {
    /// Shorthand for `(High + Low + Open + Close) / 4`
    fn hloc(&self) -> f64 {
        let high = self.high();
        let low = self.low();
        let open = self.open();
        let close = self.close();
        candlestick_average([high, low, open, close], high + low + open + close)
    }
    /// Shorthand for `(High + Low + Close) / 3`
    fn hlc(&self) -> f64 {
        let high = self.high();
        let low = self.low();
        let close = self.close();
        candlestick_average([high, low, close], high + low + close)
    }
    /// Shorthand for `(High + Low + Close + Close) / 4`
    fn hlcc(&self) -> f64 {
        let high = self.high();
        let low = self.low();
        let close = self.close();
        candlestick_average([high, low, close, close], high + low + close * 2.0)
    }

    /// Calculate pivot point
    fn pivot_point(&self) -> PivotPoint {
        let high = self.high();
        let low = self.low();
        let close = self.close();
        let p = candlestick_average([high, low, close], high + low + close);
        let d1 = high - p;
        let d2 = p - low;
        let d3 = high - low;
        PivotPoint {
            r3: p + d2 + d3,
            r2: p + d3,
            r1: p + d2,
            pivot_point: p,
            s1: p - d1,
            s2: p - d3,
            s3: p - d1 - d3,
        }
    }
}
impl<T: High + Low + Open + Close + Volume> Candlestick for T {}

fn candlestick_average<const N: usize>(values: [f64; N], sum: f64) -> f64 {
    if sum.is_finite() || values.iter().any(|value| !value.is_finite()) {
        return sum / N as f64;
    }

    let scale = values
        .iter()
        .fold(0.0_f64, |scale, value| scale.max(value.abs()));
    let scaled_average = values.iter().map(|value| value / scale).sum::<f64>() / N as f64;
    scaled_average.clamp(-1.0, 1.0) * scale
}

#[derive(Debug, Clone, PartialEq)]
pub struct PivotPoint {
    pub r3: f64,
    pub r2: f64,
    pub r1: f64,
    pub pivot_point: f64,
    pub s1: f64,
    pub s2: f64,
    pub s3: f64,
}

#[cfg(test)]
mod candlestick_tests {
    use core::cell::Cell;

    use super::{Candlestick, Close, High, Low, Open, Volume};

    struct TestCandle {
        high: f64,
        low: f64,
        open: f64,
        close: f64,
    }

    impl High for TestCandle {
        fn high(&self) -> f64 {
            self.high
        }
    }

    impl Low for TestCandle {
        fn low(&self) -> f64 {
            self.low
        }
    }

    impl Open for TestCandle {
        fn open(&self) -> f64 {
            self.open
        }
    }

    impl Close for TestCandle {
        fn close(&self) -> f64 {
            self.close
        }
    }

    impl Volume for TestCandle {
        fn volume(&self) -> f64 {
            1.0
        }
    }

    #[test]
    fn typical_price_averages_avoid_intermediate_overflow() {
        let candle = TestCandle {
            high: f64::MAX,
            low: f64::MAX,
            open: -f64::MAX,
            close: -f64::MAX,
        };

        assert_eq!(candle.hloc(), 0.0);
        assert!((candle.hlc() - f64::MAX / 3.0).abs() / f64::MAX < f64::EPSILON);
        assert_eq!(candle.hlcc(), 0.0);
    }

    #[test]
    fn typical_price_averages_keep_regular_results() {
        let candle = TestCandle {
            high: 10.0,
            low: 2.0,
            open: 7.0,
            close: 8.0,
        };

        assert_eq!(candle.hloc(), 6.75);
        assert_eq!(candle.hlc(), 20.0 / 3.0);
        assert_eq!(candle.hlcc(), 7.0);
    }

    #[test]
    fn typical_price_averages_preserve_non_finite_input_behavior() {
        let candle = TestCandle {
            high: f64::INFINITY,
            low: 1.0,
            open: 2.0,
            close: 3.0,
        };

        assert_eq!(candle.hloc(), f64::INFINITY);
        assert_eq!(candle.hlc(), f64::INFINITY);
        assert_eq!(candle.hlcc(), f64::INFINITY);

        let candle = TestCandle {
            high: f64::NAN,
            low: 1.0,
            open: 2.0,
            close: 3.0,
        };
        assert!(candle.hloc().is_nan());
        assert!(candle.hlc().is_nan());
        assert!(candle.hlcc().is_nan());
    }

    struct CountingCandle {
        high_reads: Cell<usize>,
        low_reads: Cell<usize>,
        close_reads: Cell<usize>,
    }

    impl High for CountingCandle {
        fn high(&self) -> f64 {
            self.high_reads.set(self.high_reads.get() + 1);
            10.0
        }
    }

    impl Low for CountingCandle {
        fn low(&self) -> f64 {
            self.low_reads.set(self.low_reads.get() + 1);
            2.0
        }
    }

    impl Open for CountingCandle {
        fn open(&self) -> f64 {
            unreachable!()
        }
    }

    impl Close for CountingCandle {
        fn close(&self) -> f64 {
            self.close_reads.set(self.close_reads.get() + 1);
            8.0
        }
    }

    impl Volume for CountingCandle {
        fn volume(&self) -> f64 {
            unreachable!()
        }
    }

    #[test]
    fn pivot_point_reads_each_required_candlestick_value_once() {
        let candle = CountingCandle {
            high_reads: Cell::new(0),
            low_reads: Cell::new(0),
            close_reads: Cell::new(0),
        };

        let pivot = candle.pivot_point();

        assert_eq!(pivot.pivot_point, 20.0 / 3.0);
        assert_eq!(pivot.r1, pivot.pivot_point + (pivot.pivot_point - 2.0));
        assert_eq!(candle.high_reads.get(), 1);
        assert_eq!(candle.low_reads.get(), 1);
        assert_eq!(candle.close_reads.get(), 1);
    }
}
