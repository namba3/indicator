# Indicator

[English](README.en.md) | 日本語

テクニカル分析で使うインジケーターを実装した Rust ライブラリです。

## 使用例

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

## 実装済みのインジケーター

- Aroon Indicator
- Aroon Oscillator
- Bollinger Bands
- EMA: Exponential Moving Average
- MACD: Moving Average Convergence Divergence
- Max
- Max Index（最高値の日からの経過日数）
- Min Index（最安値の日からの経過日数）
- Min
- RMA: Running Moving Average（Modified Moving Average とも呼ばれます）
- RSI: Relative Strength Index
- SMA: Simple Moving Average
- Standard Deviation
- Stochastics
- VWAP: Volume Weighted Average Price
- VWMA: Volume Weighted Moving Average

## 機能

### インジケーターの出力を変換する

出力に関数を適用できます。

```rust
use std::f64::consts::PI;

let macd = Macd::default();
let mut macd = macd.map(|MacdOutput{ macd, signal: _, histogram: _}| macd);

for input in (0..100).map(|n| f64::sin(PI / 10.0 * n as f64)) {
    let value: f64 = macd.next(input);
    println!("{value}");
}
```

### インジケーターを合成する

あるインジケーターの出力を、別のインジケーターへの入力として使えます。

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

### 成熟前の値を除外する

計算に必要なデータが十分に蓄積されるまで、出力を `None` にできます。

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

### 出力をウィンドウで取得する

内部インジケーターの直近 N 個の出力をまとめて取得できます。

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

### Iterator に変換する

入力値の Iterator から、インジケーターの出力を生成する Iterator を作れます。

```rust
use std::f64::consts::PI;

let sma = Sma::new(2).unwrap();

let input_iter = (0..100).map(|n| f64::sin(PI / 10.0 * n as f64));
let mut sma_iter = sma.iter_over(input_iter);

while let Some(value) = sma_iter.next() {
    println!("{value}");
}
```

### Stream に変換する

入力 Stream から、インジケーターの出力を生成する Stream を作れます。この機能を使うには `stream` feature を有効にしてください。

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

## ベンチマーク

stable Rust の `std::time::Instant` と `std::hint::black_box` を使ったベンチマークを実行できます。

```sh
cargo bench --bench indicators
cargo bench --bench indicators -- 2000000
```

数値は実行環境に依存します。異なる環境間の性能比較には使わず、同じ環境での相対比較に利用してください。
