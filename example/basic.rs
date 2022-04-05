use indicator::*;

fn main() {
    use std::f64::consts::PI;

    let mut sma = Sma::new(5).unwrap();

    for input in (0..100).map(|n| f64::sin(PI / 10.0 * n as f64)) {
        let value: f64 = sma.next(input);
        println!("{value}");
    }
}
