use once_cell::sync::Lazy as SyncLazy;
use rand::Rng;

use crate::{Candlestick, Close, High, Low, Open, Price, Volume};

const SIZE: usize = 10000;
pub static RANDOM_DATA: SyncLazy<Box<[TestItem]>> = SyncLazy::new(|| {
    let mut v = Vec::with_capacity(SIZE);
    let mut rng = rand::thread_rng();
    for _ in 0..SIZE {
        let value = rng.gen_range(90.0..=100.0);
        let high = value * rng.gen_range(1.0..=1.1);
        let low = value * rng.gen_range(0.9..=1.0);

        let (open, close) = match rng.gen_range(0..=5) {
            0 => (high, value),
            1 => (high, low),
            2 => (value, high),
            3 => (value, low),
            4 => (low, high),
            5 => (low, value),
            _ => unreachable!(),
        };

        let volume = rng.gen_range(1.0..=100.0);

        v.push(TestItem {
            high,
            low,
            open,
            close,
            volume,
        });
    }
    v.into_boxed_slice()
});

#[derive(Debug, Clone, Copy)]
pub struct TestItem {
    high: f64,
    low: f64,
    open: f64,
    close: f64,
    volume: f64,
}
impl High for TestItem {
    fn high(&self) -> f64 {
        self.high
    }
}
impl Low for TestItem {
    fn low(&self) -> f64 {
        self.low
    }
}
impl Open for TestItem {
    fn open(&self) -> f64 {
        self.open
    }
}
impl Close for TestItem {
    fn close(&self) -> f64 {
        self.close
    }
}
impl Volume for TestItem {
    fn volume(&self) -> f64 {
        self.volume
    }
}
impl Price for TestItem {
    fn price(&self) -> f64 {
        self.hlcc()
    }
}

pub const PRECISION: f64 = 0.00000001;
pub trait Round {
    fn round(self) -> Self;
}
impl Round for f64 {
    fn round(self) -> Self {
        (self / PRECISION).round() * PRECISION
    }
}
impl Round for usize {
    fn round(self) -> Self {
        self
    }
}
impl<T: Round> Round for Option<T> {
    fn round(self) -> Self {
        self.map(Round::round)
    }
}

macro_rules! test_indicator {
    {
        new: $new:expr,
        inputs: $inputs:expr,
        outputs: $outputs:expr$(,)?
    } => {
        test_next! {
            new: $new,
            inputs: $inputs,
            outputs: $outputs,
        }
    };
    {
        new: $new:expr,
        inputs: $inputs:expr,
        outputs: $outputs:expr,
        additional_tests: {
            $(
                $test_name:ident $(: {
                    $(
                        $prop:ident $(: $value:expr)?,
                    )*
                })?,
            )*
        }$(,)?
    } => {
        test_indicator! {
            new: $new,
            inputs: $inputs,
            outputs: $outputs,
        }

        $( additional_test!{ $new, $test_name, { $($($prop$(:$value)*),*)* } } )*
    }
}

macro_rules! test_next {
    {
        new: $new:expr,
        inputs: $inputs:expr,
        outputs: $outputs:expr,
    } => {
        #[test]
        fn next() -> crate::Result<()> {
            let new:crate::Result<_> = $new;
            let mut indicator = new?;

            let inputs: Vec<_> = $inputs.into_iter().collect();
            let outputs: Vec<_> = $outputs.into_iter().collect();

            assert_eq!(inputs.len(), outputs.len());

            for (i, x) in inputs.into_iter().enumerate() {
                let x = Round::round(Next::next(&mut indicator, x));
                let correct = Round::round(outputs[i]);
                assert_eq!(x, correct);
            }

            Ok(())
        }
    };
}

macro_rules! additional_test {
    ($new:expr, current, {
        inputs: $inputs:expr
    }) => {
        test_current! {
            new: $new,
            inputs: $inputs,
        }
    };
    ($new:expr, reset, {
        inputs: $inputs:expr
    }) => {
        test_reset! {
            new: $new,
            inputs: $inputs,
        }
    };
    ($_new:expr, new_invalid_parameter, {
        new: $new:expr
    }) => {
        test_new_invalid_parameter! {
            new: $new
        }
    };
    ($_new:expr, new_invalid_parameter, {
        news: $news:expr
    }) => {
        test_new_invalid_parameter! {
            news: $news
        }
    };
    ($new:expr, next_ext, {
        inputs: $inputs:expr,
        outputs: $outputs:expr
    }) => {
        test_next_ext! {
            new: $new,
            inputs: $inputs,
            outputs: $outputs,
        }
    };
}

macro_rules! test_next_ext {
    {
        new: $new:expr,
        inputs: $inputs:expr,
        outputs: $outputs:expr,
    } => {
        #[test]
        fn next_ext() -> crate::Result<()> {
            let new: crate::Result<_> = $new;
            let mut indicator = new?;

            let inputs: Vec<_> = $inputs.into_iter().collect();
            let outputs: Vec<_> = $outputs.into_iter().collect();

            assert_eq!(inputs.len(), outputs.len());

            for (i, x) in inputs.into_iter().enumerate() {
                let x = Round::round(Next::next(&mut indicator, x));
                let correct = Round::round(outputs[i]);
                assert_eq!(x, correct);
            }

            Ok(())
        }
    };
}

macro_rules! test_reset {
    {
        new: $new:expr,
        inputs: $inputs:expr,
    } => {
        #[test]
        fn reset() -> crate::Result<()> {
            let new:crate::Result<_> = $new;
            let mut indicator = new?;

            let inputs: Vec<_> = $inputs.into_iter().collect();

            let mut v = Vec::with_capacity(inputs.len());
            for x in inputs.iter().copied() {
                v.push(Next::next(&mut indicator, x));
            }

            Reset::reset(&mut indicator);

            for (i, x) in inputs.iter().copied().enumerate() {
                assert_eq!(Next::next(&mut indicator, x), v[i]);
            }

            Ok(())
        }
    };
}

macro_rules! test_new_invalid_parameter {
    { new: $new:expr} => {
        #[test]
        fn new_invalid_parameter() {
            assert!($new.is_err());
        }
    };
    { news: $news:expr} => {
        #[test]
        fn new_invalid_parameter() {
            let news: &[crate::Result<_>] = &$news;
            for new in news.iter() {
                assert!(new.is_err());
            }
        }
    };
}

macro_rules! test_current {
    {
        new: $new:expr,
        inputs: $inputs:expr,
    } => {
        #[test]
        fn current() -> crate::Result<()> {
            let new:crate::Result<_> = $new;
            let mut indicator = new?;

            let inputs: Vec<_> = $inputs.into_iter().collect();

            for x in inputs.iter().copied() {
                let value = Next::next(&mut indicator, x);
                assert_eq!(Current::current(&indicator), Some(value.into()));
            }

            Ok(())
        }
    };
}
