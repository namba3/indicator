//! # Example
//!
//! ```
//! use indicator::*;
//!
//! fn main() {
//!     let mut sma = Sma::new(2).unwrap();
//!     for n in 0..100 {
//!         let value: f64 = sma.next(n as f64);
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

pub mod operators;

pub use error::*;
pub use indicator_ext::*;
pub use indicators::*;

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
}
impl<T: High + Low + Open + Close + Volume> Candlestick for T {}

/// Indicator
pub trait Indicator {
    type Input;
    type Output;
    fn next(&mut self, input: Self::Input) -> Self::Output;
}

/// Current
pub trait Current: Indicator {
    fn current(&self) -> Option<Self::Output>;
}
/// Next
pub trait NextExt<Input>: Indicator {
    fn next_ext(&mut self, input: Input) -> Self::Output;
}
/// Reset
pub trait Reset {
    fn reset(&mut self);
}
