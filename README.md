# Indicator

[English](README.en.md) | 日本語

テクニカル分析で使うインジケーターを実装した Rust ライブラリです。

## 金融商品におけるテクニカル分析

テクニカル分析は、株式・為替・先物などの金融商品の価格や出来高の時系列を調べ、市場のトレンドや勢い、変動の大きさを把握する手法です。インジケーターは、こうした時系列を特定の計算方法と期間で要約します。

- **トレンド**: SMA、EMA、RMA、Aroon
- **モメンタム**: RSI、MACD、Stochastics
- **価格変動・レンジ**: Bollinger Bands、Standard Deviation、Max、Min
- **出来高**: VWAP、VWMA

同じインジケーターでも、入力する価格・出来高、観測間隔、計算期間によって値は変わります。指標の値や過去のパターンだけで将来の値動きが決まるわけではなく、シグナルが外れることもあります。頻繁な売買を伴う場合は、手数料などの取引コストや、売買タイミングを逃すリスクも考慮してください。

このライブラリは指標の値を計算します。市場データの取得、売買シグナルの評価、投資判断や注文は行いません。投資には損失のリスクがあります。

参考: [CFA Institute Research Foundation: Technical Analysis: Modern Perspectives](https://rpc.cfainstitute.org/research/foundation/2017/technical-analysis)、[FINRA: What Is Market Timing?](https://www.finra.org/investors/insights/market-timing)、[Investor.gov: What Is Risk?](https://www.investor.gov/introduction-investing/investing-basics/what-risk)

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

### 出来高 0 の加重平均を扱う

VWAP と VWMA の `next_option` は、合計出来高が 0 の間 `None` を返します。既存の `Next` は `f64` の戻り値を保ち、計算結果が未定義の間は `NaN` を返します。

NaN・無限大の価格、および負数・NaN・無限大の出来高は無効です。負の有限価格は有効です。`try_next`（タプル入力）または `try_next_ref`（`Price` と `Volume` を実装した型の参照）を使うと、`InvalidPrice` または `InvalidVolume` エラーになり、指標の状態は変化しません。既存の `Next` は戻り値の型を変えず、不正な入力を無視して現在値を返します（有効な値がまだない場合は `NaN`）。外部データを検証する場合は `try_next` 系を使用してください。

```rust
let mut vwap = Vwap::new();

assert_eq!(vwap.next_option((100.0, 0.0)), None);
assert_eq!(vwap.next_option((110.0, 2.0)), Some(110.0));
assert!(vwap.try_next((120.0, -1.0)).is_err());
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
