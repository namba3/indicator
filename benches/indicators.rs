use std::hint::black_box;
use std::time::Instant;

use indicator::{Ema, Macd, Max, MaxIndex, Min, MinIndex, Next, Rsi, Sma, Stochastics, Vwap, Vwma};

const DEFAULT_ITERATIONS: usize = 1_000_000;
const SAMPLE_COUNT: usize = 5;
const PRICES: [f64; 16] = [
    100.0, 101.0, 99.5, 102.0, 103.5, 101.5, 104.0, 102.5, 105.0, 104.0, 106.5, 103.0, 107.0,
    105.5, 108.0, 106.0,
];
const PRICE_VOLUMES: [(f64, f64); 16] = [
    (100.0, 1.0),
    (101.0, 2.0),
    (99.5, 1.5),
    (102.0, 3.0),
    (103.5, 2.5),
    (101.5, 1.0),
    (104.0, 4.0),
    (102.5, 2.0),
    (105.0, 3.5),
    (104.0, 1.5),
    (106.5, 2.5),
    (103.0, 3.0),
    (107.0, 4.0),
    (105.5, 2.0),
    (108.0, 3.5),
    (106.0, 1.0),
];

fn run_benchmark(name: &str, iterations: usize, mut next: impl FnMut(usize) -> f64) {
    let warmup_iterations = iterations.min(100_000);
    for index in 0..warmup_iterations {
        black_box(next(black_box(index)));
    }

    let mut samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut checksum = 0.0;
    for _ in 0..SAMPLE_COUNT {
        let start = Instant::now();
        let mut sample_checksum = 0.0;
        for index in 0..iterations {
            sample_checksum += black_box(next(black_box(index)));
        }
        samples.push(start.elapsed());
        checksum += black_box(sample_checksum);
    }
    samples.sort_unstable();
    let elapsed = samples[SAMPLE_COUNT / 2];
    let nanoseconds_per_iteration = elapsed.as_nanos() as f64 / iterations as f64;
    let iterations_per_second = iterations as f64 / elapsed.as_secs_f64();

    println!(
        "{name:<14} {nanoseconds_per_iteration:>10.2} ns/iteration  {iterations_per_second:>12.0} iterations/s  checksum={}",
        black_box(checksum)
    );
}

fn main() {
    let iterations = std::env::args()
        .nth(1)
        .map(|value| {
            value
                .parse()
                .expect("iteration count must be a positive integer")
        })
        .unwrap_or(DEFAULT_ITERATIONS);
    assert!(iterations > 0, "iteration count must be greater than zero");

    println!("Stable Rust indicator benchmarks ({iterations} measured iterations each)");

    let mut sma = Sma::new(14).expect("valid SMA period");
    run_benchmark("SMA", iterations, |index| {
        sma.next(black_box(PRICES[index % PRICES.len()]))
    });

    let mut ema = Ema::new(14).expect("valid EMA period");
    run_benchmark("EMA", iterations, |index| {
        ema.next(black_box(PRICES[index % PRICES.len()]))
    });

    let mut rsi = Rsi::new(14).expect("valid RSI period");
    run_benchmark("RSI", iterations, |index| {
        rsi.next(black_box(PRICES[index % PRICES.len()]))
    });

    let mut macd = Macd::new(12, 26, 9).expect("valid MACD periods");
    run_benchmark("MACD", iterations, |index| {
        let output = macd.next(black_box(PRICES[index % PRICES.len()]));
        output.macd + output.signal + output.histogram
    });

    let mut stochastics = Stochastics::new(14, 3, 3).expect("valid Stochastics periods");
    run_benchmark("Stochastics", iterations, |index| {
        let output = stochastics.next(black_box(PRICES[index % PRICES.len()]));
        output.k + output.d + output.slow_d
    });

    let mut max_index = MaxIndex::new(256).expect("valid MaxIndex period");
    run_benchmark("MaxIndex", iterations, |index| {
        max_index.next(black_box(-(index as f64))) as f64
    });

    let mut min_index = MinIndex::new(256).expect("valid MinIndex period");
    run_benchmark("MinIndex", iterations, |index| {
        min_index.next(black_box(index as f64)) as f64
    });

    let mut max = Max::new(256).expect("valid Max period");
    run_benchmark("Max", iterations, |index| {
        max.next(black_box(-(index as f64)))
    });

    let mut min = Min::new(256).expect("valid Min period");
    run_benchmark("Min", iterations, |index| min.next(black_box(index as f64)));

    let mut vwap = Vwap::new();
    run_benchmark("VWAP", iterations, |index| {
        vwap.next(black_box(PRICE_VOLUMES[index % PRICE_VOLUMES.len()]))
    });

    let mut vwma = Vwma::new(14).expect("valid VWMA period");
    run_benchmark("VWMA", iterations, |index| {
        vwma.next(black_box(PRICE_VOLUMES[index % PRICE_VOLUMES.len()]))
    });
}
