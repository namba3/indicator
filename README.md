# Indicator

An implementation of indicators used for technical analysis.

This is my private project.

## Example

```rust
use indicator::*;

fn main() {
    let mut sma = Sma::new(2).unwrap();
    for n in 0..100 {
        let value: Option<f64> = sma.next(n as f64);
        println!("{value:?}");
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
- VWAP: Volume Weighted Average Price
- VWMA: Volume Weighted Moving Average

## Features

### Indicator against Indicator

You can easily create an indicator against indicator.

```rust
let sma = Sma::new(2).unwrap();
let rsi = Rsi::new(14).unwrap();

let mut sma_against_rsi = rsi.pushforward(sma);

for n in 0..100 {
    let value: Option<f64> = sma_against_rsi.next(n as f64);
    println!("{value:?}");
}
```

### Easily exclude immature values

You can easily exclude values ​​for periods when data is not accumulated enough.

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
