# Indicator

An implementation of indicators used for technical analysis.

This is my private project.

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
- Bolinger Bands
- EMA: Exponential Moving Average
- MACD: Moving Average Convergence Divergence
- Max
- Max Index
  (Number of days elapsed from the date of the highest price)
- Min Index
  (Number of days elapsed from the date of the lowest price)
- Min
- RMA: Running Moving Average
  (a.k.a Modified Moving Average)
- RSI: Relative Strength Index
- SMA: Simple Moving Average
- Standard Deviation
- Stochastics
- VWAP: Volume Weighted Average Price
- VWMA: Volume Weighted Moving Average

## Features

### Map transformation for Indicator output

You can apply a functional transformation to the output of the indicator.

```rust
use std::f64::consts::PI;

let macd = Macd::default();
let mut macd = macd.map(|MacdOutput{ macd, signal: _, histogram: _}| macd);

for input in (0..100).map(|n| f64::sin(PI / 10.0 * n as f64)) {
    let value: f64 = macd.next(input);
    println!("{value}");
}
```

### Indicator against Indicator

You can easily create a new indicator against indicator.

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

You can exclude values ​​for periods when data is not accumulated enough.

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

### Windowing Indicator

You can create a new indicator that outputs the past N output values ​​of the inner indicator.

```rust
let sma = Sma::new(5).unwrap();

let mut sma_window = sma.window(3);

assert_eq!(sma_window.next(100.0), &[&100.0, &100.0, &100.0]);
assert_eq!(sma_window.next(101.0), &[&100.0, &100.0, &100.2]);
assert_eq!(sma_window.next(101.0), &[&100.0, &100.2, &100.4]);
assert_eq!(sma_window.next(102.0), &[&100.2, &100.4, &100.8]);
assert_eq!(sma_window.next(102.0), &[&100.4, &100.8, &101.2]);
assert_eq!(sma_window.next(102.0), &[&100.8, &101.2, &101.6]);
```

### Convert Indicator to Iterator

If there is an iterator of the input value for the indicator, you can create an iterator of the output value of the indicator based on it.

```rust
let sma = Sma::new(2).unwrap();

let input_iter = (0..100).map(|n| f64::sin(PI / 10.0 * n as f64));
let mut sma_iter = sma.iter_over(input_iter);

while let Some(value) = sma_iter.next() {
    println!("{value}");
}
```

### Convert Indicator to Stream

If there is an stream of the input value for the indicator, you can create an stream of the output value of the indicator based on it.

This requires `stream` feature to be enabled.

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
