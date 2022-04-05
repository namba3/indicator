//! # Example
//!
//! ```
//! use indicator::*;
//!
//! fn main() {
//!     use std::f64::consts::PI;
//!
//!     let mut sma = Sma::new(5).unwrap();
//!
//!     for input in (0..100).map(|n| f64::sin(PI / 10.0 * n as f64)) {
//!         let value: f64 = sma.next(input);
//!         println!("{value}");
//!     }
//! }
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)]
#![cfg_attr(test, feature(once_cell))]

extern crate alloc;

#[cfg(test)]
#[macro_use]
mod test_helper;

pub mod error;
pub mod indicator_ext;
pub mod indicators;

pub mod indicator_iterator;
pub mod operators;

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
        (self.high() + self.low() + self.open() + self.close()) / 4.0
    }
    /// Shorthand for `(High + Low + Close) / 3`
    fn hlc(&self) -> f64 {
        (self.high() + self.low() + self.close()) / 3.0
    }
    /// Shorthand for `(High + Low + Close + Close) / 4`
    fn hlcc(&self) -> f64 {
        (self.high() + self.low() + self.close() * 2.0) / 4.0
    }

    /// Calculate pivot point
    fn pivot_point(&self) -> PivotPoint {
        let p = self.hlc();
        let d1 = self.high() - p;
        let d2 = p - self.low();
        let d3 = self.high() - self.low();
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
