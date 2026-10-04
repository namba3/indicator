# Indicator

English | [日本語](README.md)

A Rust library that implements indicators for technical analysis.

## Example

```rust
use indicator::*;

fn main() {
    use std::f64::consts::PI;

    let mut sma = Sma::new(5).unwrap();

    for input in (0..100).map(|n| f64::sin(PI / 10.0 * n as f64)) {
        let value: f64 = sma.next(input);
        println!("{value}");
    }
}
```

## Implemented Indicators

- Aroon Indicator
- Aroon Oscillator
- Bollinger Bands
- EMA: Exponential Moving Average
- MACD: Moving Average Convergence Divergence
- Max
- Max Index (number of days elapsed since the highest price)
- Min Index (number of days elapsed since the lowest price)
- Min
- RMA: Running Moving Average (also known as Modified Moving Average)
- RSI: Relative Strength Index
- SMA: Simple Moving Average
- Standard Deviation
- Stochastics
- VWAP: Volume Weighted Average Price
- VWMA: Volume Weighted Moving Average

## Features

### Transform indicator output

Apply a function to an indicator's output.

```rust
use std::f64::consts::PI;

let macd = Macd::default();
let mut macd = macd.map(|MacdOutput{ macd, signal: _, histogram: _}| macd);

for input in (0..100).map(|n| f64::sin(PI / 10.0 * n as f64)) {
    let value: f64 = macd.next(input);
    println!("{value}");
}
```

### Compose indicators

Use one indicator's output as the input to another indicator.

```rust
use std::f64::consts::PI;

let sma = Sma::new(2).unwrap();
let rsi = Rsi::new(14).unwrap();

let mut sma_against_rsi = rsi.pushforward(sma);

for input in (0..100).map(|n| f64::sin(PI / 10.0 * n as f64)) {
    let value: f64 = sma_against_rsi.next(input);
    println!("{value}");
}
```

### Exclude immature values

Return `None` until enough data has been accumulated for the calculation.

```rust
let sma = Sma::new(4).unwrap();
let mut sma = sma.mature(3);

assert_eq!(sma.next(1.0), None);
assert_eq!(sma.next(2.0), None);
assert_eq!(sma.next(1.0), None);
assert_eq!(sma.next(2.0), Some(1.5));
assert_eq!(sma.next(1.0), Some(1.5));
assert_eq!(sma.next(2.0), Some(1.5));
```

### Get outputs in a window

Collect the most recent N outputs from an inner indicator.

```rust
let sma = Sma::new(5).unwrap();
let mut sma_window = sma.window(3);

assert_eq!(sma_window.next(100.0), vec![100.0, 100.0, 100.0]);
assert_eq!(sma_window.next(101.0), vec![100.0, 100.0, 100.2]);
assert_eq!(sma_window.next(101.0), vec![100.0, 100.2, 100.4]);
assert_eq!(sma_window.next(102.0), vec![100.2, 100.4, 100.8]);
assert_eq!(sma_window.next(102.0), vec![100.4, 100.8, 101.2]);
assert_eq!(sma_window.next(102.0), vec![100.8, 101.2, 101.6]);
```

### Convert an indicator to an iterator

Create an iterator of indicator outputs from an iterator of input values.

```rust
use std::f64::consts::PI;

let sma = Sma::new(2).unwrap();

let input_iter = (0..100).map(|n| f64::sin(PI / 10.0 * n as f64));
let mut sma_iter = sma.iter_over(input_iter);

while let Some(value) = sma_iter.next() {
    println!("{value}");
}
```

### Convert an indicator to a stream

Create a stream of indicator outputs from an input stream. Enable the `stream` feature to use this functionality.

```rust
use futures_util::{stream, StreamExt};
use std::f64::consts::PI;

let sma = Sma::new(2).unwrap();

let input_iter = (0..100).map(|n| f64::sin(PI / 10.0 * n as f64));
let input_stream = stream::iter(input_iter);
let mut sma_stream = sma.iter_over_stream(input_stream);

while let Some(value) = sma_stream.next().await {
    println!("{value}");
}
```

## Benchmarks

Run the benchmarks using stable Rust's `std::time::Instant` and `std::hint::black_box`.

```sh
cargo bench --bench indicators
cargo bench --bench indicators -- 2000000
```

Results depend on the machine and build environment. Use them for relative comparisons on the same machine rather than comparisons across different environments.
