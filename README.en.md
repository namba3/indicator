# Indicator

English | [日本語](README.md)

A Rust library that implements indicators for technical analysis.

## Technical Analysis for Financial Instruments

Technical analysis examines time series of prices and trading volume for financial instruments such as equities, foreign exchange, and futures to describe trends, momentum, and volatility. Indicators summarize those series using a particular calculation and lookback period.

- **Trend**: SMA, EMA, RMA, Aroon
- **Momentum**: RSI, MACD, Stochastics
- **Price range and volatility**: Bollinger Bands, Standard Deviation, Max, Min
- **Volume**: VWAP, VWMA

Indicator values depend on the input prices and volumes, sampling interval, and calculation period. An indicator or historical pattern does not determine future price movements, and signals can be wrong. When a strategy involves frequent trading, account for transaction costs such as fees and the risk of missing market moves.

This library calculates indicator values. It does not provide market data, evaluate trading signals, make investment decisions, or place orders. Investing involves the risk of loss.

Further reading: [CFA Institute Research Foundation: Technical Analysis: Modern Perspectives](https://rpc.cfainstitute.org/research/foundation/2017/technical-analysis), [FINRA: What Is Market Timing?](https://www.finra.org/investors/insights/market-timing), [Investor.gov: What Is Risk?](https://www.investor.gov/introduction-investing/investing-basics/what-risk)

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
- Bollinger Bands (`BollingerBands`)
- EMA: Exponential Moving Average
- MACD: Moving Average Convergence Divergence
- Max
- Max Index (number of samples elapsed since the highest value)
- Min Index (number of samples elapsed since the lowest value)
- Min
- RMA: Running Moving Average (also known as Modified Moving Average)
- RSI: Relative Strength Index
- SMA: Simple Moving Average
- Standard Deviation
- Stochastics
- VWAP: Volume Weighted Average Price
- VWMA: Volume Weighted Moving Average

## Initialization and Warm-up

Unless noted below, `next` returns a value starting with the first input; indicators do not wait for a full period of samples. The table describes initialization and output ranges for period-based indicators.

| Indicator | Initialization and first output | Output range or behavior |
| --- | --- | --- |
| SMA, Standard Deviation, Bollinger Bands, Max, Min | The first input fills the entire window, so output starts immediately. | SMA, mean, and extrema use the input scale. Standard deviation is non-negative; Bollinger Bands satisfy `lower_bound <= average <= upper_bound`. |
| Max Index, Min Index | The first input fills the entire window, so output starts immediately. | Number of samples since the extreme, in `0..period` (excluding `period`). |
| EMA, RMA | The first input is the initial value, and output starts immediately. EMA updates with coefficient `2 / (period + 1)`; RMA uses `1 / period`. | Same scale as the input. |
| RSI | The first change is treated as 0, so the first output is `0.5`. | `0..=1`. |
| MACD | The short and long EMAs start from the first input, so MACD, Signal, and Histogram are all `0` on the first input. | No fixed range. |
| Stochastics | `%K`, `%D`, and `Slow %D` are all `0.5` on the first input. A zero denominator also uses `0.5`. | All three outputs are in `0..=1`. |
| Aroon Indicator, Aroon Oscillator | The first input fills the window, so output starts immediately. Aroon Up/Down start at `1`; the Oscillator starts at `0`. | Aroon Up/Down are in `0..=1`; the Oscillator is in `-1..=1`. |
| VWAP | A value is available after cumulative volume first becomes positive. Until then, `current()` and `next_option()` return `None`, while ordinary `next()` returns `NaN`. | Weighted average of prices with positive volume. |
| VWMA | The first price-volume pair fills the window. A value is available once the window's total volume is positive. | Weighted average of prices with positive volume in the window. |

Using `mature(n)` hides the first `n` outputs as `None` and returns values starting with output `n + 1`. It delays when outputs are exposed; it does not change the indicator's own initialization rule.

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

### Handle zero-volume weighted averages

`next_option` on VWAP and VWMA returns `None` while total volume is zero. The existing `Next` implementations keep their `f64` output and return `NaN` while the result is undefined.

NaN and infinite prices, and negative, NaN, and infinite volumes are invalid. Finite negative prices are valid. Use `try_next` with tuple input or `try_next_ref` with a reference to a type implementing `Price` and `Volume`; these return an `InvalidPrice` or `InvalidVolume` error without changing indicator state. The existing `Next` implementations keep their return type, ignore invalid input, and return the current value (or `NaN` before a valid value is available). Use a fallible method when validating external data.

```rust
let mut vwap = Vwap::new();

assert_eq!(vwap.next_option((100.0, 0.0)), None);
assert_eq!(vwap.next_option((110.0, 2.0)), Some(110.0));
assert!(vwap.try_next((120.0, -1.0)).is_err());
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
