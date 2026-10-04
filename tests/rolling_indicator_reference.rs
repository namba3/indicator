use indicator::{
    AroonIndicator, AroonOscillator, Current, Ema, IndicatorExt, Macd, Max, MaxIndex, Min,
    MinIndex, Next, Reset, Rma, Rsi, Sma, StandardDeviation, Stochastics, Vwap, Vwma,
};

fn deterministic_inputs() -> Vec<f64> {
    let mut state = 0x5eed_u64;
    let mut inputs = vec![
        4.0, 4.0, 9.0, -3.0, 9.0, 2.0, -3.0, 8.0, 8.0, 0.0, -7.0, 9.0,
    ];

    for _ in 0..512 {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        inputs.push(((state >> 32) % 23) as i32 as f64 - 11.0);
    }

    inputs
}

fn push_reference_window(window: &mut Vec<f64>, period: usize, input: f64) {
    if window.is_empty() {
        window.resize(period, input);
    } else {
        window.remove(0);
        window.push(input);
    }
}

#[test]
fn aroon_indicators_match_naive_references_across_long_sequences_and_reset() -> indicator::Result<()>
{
    const PERIOD: usize = 17;
    let mut aroon = AroonIndicator::new(PERIOD)?;
    let mut oscillator = AroonOscillator::new(PERIOD)?;
    let mut window = Vec::with_capacity(PERIOD + 1);

    for (position, input) in deterministic_inputs().into_iter().enumerate() {
        if position == 257 {
            aroon.reset();
            oscillator.reset();
            window.clear();
        }

        push_reference_window(&mut window, PERIOD + 1, input);
        let highest = window.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let lowest = window.iter().copied().fold(f64::INFINITY, f64::min);
        let max_index = window
            .iter()
            .rev()
            .position(|value| *value == highest)
            .unwrap();
        let min_index = window
            .iter()
            .rev()
            .position(|value| *value == lowest)
            .unwrap();
        let expected_up = (PERIOD - max_index) as f64 / PERIOD as f64;
        let expected_down = (PERIOD - min_index) as f64 / PERIOD as f64;
        let expected_oscillator = expected_up - expected_down;

        let actual = aroon.next(input);
        let actual_oscillator = oscillator.next(input);
        assert!(
            (actual.aroon_up - expected_up).abs() <= 1.0e-12,
            "position {position}: Aroon up {}, expected {expected_up}",
            actual.aroon_up
        );
        assert!(
            (actual.aroon_down - expected_down).abs() <= 1.0e-12,
            "position {position}: Aroon down {}, expected {expected_down}",
            actual.aroon_down
        );
        assert!(
            (actual_oscillator - expected_oscillator).abs() <= 1.0e-12,
            "position {position}: Aroon oscillator {actual_oscillator}, expected {expected_oscillator}"
        );
        assert!((0.0..=1.0).contains(&actual.aroon_up));
        assert!((0.0..=1.0).contains(&actual.aroon_down));
        assert!((-1.0..=1.0).contains(&actual_oscillator));
        assert_eq!(aroon.current(), Some(actual));
        assert_eq!(oscillator.current(), Some(actual_oscillator));
    }

    Ok(())
}

#[test]
fn ema_and_rma_match_scalar_references_across_long_sequences_and_reset() -> indicator::Result<()> {
    const EMA_PERIOD: usize = 17;
    const RMA_PERIOD: usize = 23;
    let inputs = deterministic_inputs();
    let mut ema = Ema::new(EMA_PERIOD)?;
    let mut rma = Rma::new(RMA_PERIOD)?;
    let mut expected_ema = None;
    let mut expected_rma = None;

    for (position, input) in inputs.into_iter().enumerate() {
        if position == 257 {
            ema.reset();
            rma.reset();
            expected_ema = None;
            expected_rma = None;
        }

        let alpha_ema = 2.0 / (EMA_PERIOD as f64 + 1.0);
        let alpha_rma = 1.0 / RMA_PERIOD as f64;
        let next_expected_ema = expected_ema.map_or(input, |previous| {
            previous * (1.0 - alpha_ema) + input * alpha_ema
        });
        let next_expected_rma = expected_rma.map_or(input, |previous| {
            previous * (1.0 - alpha_rma) + input * alpha_rma
        });

        let actual_ema = ema.next(input);
        let actual_rma = rma.next(input);
        let ema_tolerance = 1.0e-12 * next_expected_ema.abs().max(1.0);
        let rma_tolerance = 1.0e-12 * next_expected_rma.abs().max(1.0);

        assert!(
            (actual_ema - next_expected_ema).abs() <= ema_tolerance,
            "position {position}: EMA {actual_ema}, expected {next_expected_ema}, tolerance {ema_tolerance}"
        );
        assert!(
            (actual_rma - next_expected_rma).abs() <= rma_tolerance,
            "position {position}: RMA {actual_rma}, expected {next_expected_rma}, tolerance {rma_tolerance}"
        );
        assert_eq!(ema.current(), Some(actual_ema));
        assert_eq!(rma.current(), Some(actual_rma));
        expected_ema = Some(next_expected_ema);
        expected_rma = Some(next_expected_rma);
    }

    Ok(())
}

#[test]
fn rsi_matches_gain_loss_reference_across_long_sequence_and_reset() -> indicator::Result<()> {
    const PERIOD: usize = 14;
    let inputs = deterministic_inputs();
    let mut rsi = Rsi::new(PERIOD)?;
    let mut previous_input: Option<f64> = None;
    let mut average_gain: Option<f64> = None;
    let mut average_loss: Option<f64> = None;

    for (position, input) in inputs.into_iter().enumerate() {
        if position == 257 {
            rsi.reset();
            previous_input = None;
            average_gain = None;
            average_loss = None;
        }

        let (gain, loss) = if let Some(previous_input) = previous_input {
            let change = input - previous_input;
            (change.max(0.0), (-change).max(0.0))
        } else {
            (0.0, 0.0)
        };
        let next_average_gain = average_gain.map_or(gain, |average| {
            (average * (PERIOD - 1) as f64 + gain) / PERIOD as f64
        });
        let next_average_loss = average_loss.map_or(loss, |average| {
            (average * (PERIOD - 1) as f64 + loss) / PERIOD as f64
        });
        let expected = match (next_average_gain, next_average_loss) {
            (gain, loss) if gain <= 0.0 && loss <= 0.0 => 0.5,
            (gain, _) if gain <= 0.0 => 0.0,
            (_, loss) if loss <= 0.0 => 1.0,
            (gain, loss) => gain / (gain + loss),
        };

        let actual = rsi.next(input);
        let tolerance = 1.0e-12;
        assert!(
            (actual - expected).abs() <= tolerance,
            "position {position}: RSI {actual}, expected {expected}, tolerance {tolerance}"
        );
        assert!((0.0..=1.0).contains(&actual));
        assert_eq!(rsi.current(), Some(actual));

        previous_input = Some(input);
        average_gain = Some(next_average_gain);
        average_loss = Some(next_average_loss);
    }

    Ok(())
}

#[test]
fn macd_matches_ema_and_signal_references_across_long_sequence_and_reset() -> indicator::Result<()>
{
    const SHORT_PERIOD: usize = 8;
    const LONG_PERIOD: usize = 21;
    const SIGNAL_PERIOD: usize = 5;
    let inputs = deterministic_inputs();
    let mut macd = Macd::new(SHORT_PERIOD, LONG_PERIOD, SIGNAL_PERIOD)?;
    let mut short_ema: Option<f64> = None;
    let mut long_ema: Option<f64> = None;
    let mut signal_window = Vec::with_capacity(SIGNAL_PERIOD);

    for (position, input) in inputs.into_iter().enumerate() {
        if position == 257 {
            macd.reset();
            short_ema = None;
            long_ema = None;
            signal_window.clear();
        }

        let short_alpha = 2.0 / (SHORT_PERIOD as f64 + 1.0);
        let long_alpha = 2.0 / (LONG_PERIOD as f64 + 1.0);
        let next_short_ema = short_ema.map_or(input, |previous| {
            previous * (1.0 - short_alpha) + input * short_alpha
        });
        let next_long_ema = long_ema.map_or(input, |previous| {
            previous * (1.0 - long_alpha) + input * long_alpha
        });
        let expected_macd = next_short_ema - next_long_ema;

        if signal_window.is_empty() {
            signal_window.resize(SIGNAL_PERIOD, expected_macd);
        } else {
            signal_window.remove(0);
            signal_window.push(expected_macd);
        }
        let expected_signal = signal_window.iter().sum::<f64>() / SIGNAL_PERIOD as f64;
        let expected_histogram = expected_macd - expected_signal;

        let actual = macd.next(input);
        let tolerance = 1.0e-12;
        for (name, actual, expected) in [
            ("MACD", actual.macd, expected_macd),
            ("signal", actual.signal, expected_signal),
            ("histogram", actual.histogram, expected_histogram),
        ] {
            let allowed_error = tolerance * expected.abs().max(1.0);
            assert!(
                (actual - expected).abs() <= allowed_error,
                "position {position}: {name} {actual}, expected {expected}, tolerance {allowed_error}"
            );
        }
        assert_eq!(macd.current(), Some(actual));

        short_ema = Some(next_short_ema);
        long_ema = Some(next_long_ema);
    }

    Ok(())
}

#[test]
fn stochastics_matches_naive_reference_across_long_sequence_and_reset() -> indicator::Result<()> {
    const N_PERIOD: usize = 11;
    const M_PERIOD: usize = 4;
    const X_PERIOD: usize = 3;
    let inputs = deterministic_inputs();
    let mut stochastics = Stochastics::new(N_PERIOD, M_PERIOD, X_PERIOD)?;
    let mut price_window = Vec::with_capacity(N_PERIOD);
    let mut d_numerator_window = Vec::with_capacity(M_PERIOD);
    let mut d_denominator_window = Vec::with_capacity(M_PERIOD);
    let mut slow_d_window = Vec::with_capacity(X_PERIOD);

    for (position, input) in inputs.into_iter().enumerate() {
        if position == 257 {
            stochastics.reset();
            price_window.clear();
            d_numerator_window.clear();
            d_denominator_window.clear();
            slow_d_window.clear();
        }

        push_reference_window(&mut price_window, N_PERIOD, input);
        let lowest = price_window.iter().copied().fold(f64::INFINITY, f64::min);
        let highest = price_window
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let expected_k = if lowest == highest {
            0.5
        } else {
            (input - lowest) / (highest - lowest)
        };

        let first_output = d_numerator_window.is_empty();
        let numerator = if first_output { 0.0 } else { input - lowest };
        let denominator = if first_output { 0.0 } else { highest - lowest };
        push_reference_window(&mut d_numerator_window, M_PERIOD, numerator);
        push_reference_window(&mut d_denominator_window, M_PERIOD, denominator);
        let average_numerator = d_numerator_window.iter().sum::<f64>() / M_PERIOD as f64;
        let average_denominator = d_denominator_window.iter().sum::<f64>() / M_PERIOD as f64;
        let expected_d = if first_output || average_denominator == 0.0 {
            0.5
        } else {
            average_numerator / average_denominator
        };

        let slow_d_input = if first_output { 0.0 } else { expected_d };
        push_reference_window(&mut slow_d_window, X_PERIOD, slow_d_input);
        let expected_slow_d = if first_output {
            0.5
        } else {
            slow_d_window.iter().sum::<f64>() / X_PERIOD as f64
        };

        let actual = stochastics.next(input);
        let tolerance = 1.0e-12;
        for (name, actual, expected) in [
            ("%K", actual.k, expected_k),
            ("%D", actual.d, expected_d),
            ("Slow %D", actual.slow_d, expected_slow_d),
        ] {
            assert!(
                (actual - expected).abs() <= tolerance,
                "position {position}: {name} {actual}, expected {expected}, tolerance {tolerance}"
            );
            assert!((0.0..=1.0).contains(&actual));
        }
        assert_eq!(stochastics.current(), Some(actual));
    }

    Ok(())
}

#[test]
fn rolling_extrema_and_indices_match_a_naive_reference() -> indicator::Result<()> {
    const PERIOD: usize = 11;

    let inputs = deterministic_inputs();
    let mut max = Max::new(PERIOD)?;
    let mut min = Min::new(PERIOD)?;
    let mut max_index = MaxIndex::new(PERIOD)?;
    let mut min_index = MinIndex::new(PERIOD)?;
    let mut window = Vec::with_capacity(PERIOD);

    for (position, input) in inputs.into_iter().enumerate() {
        if window.is_empty() {
            window.resize(PERIOD, input);
        } else {
            window.remove(0);
            window.push(input);
        }

        let expected_max = window.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let expected_min = window.iter().copied().fold(f64::INFINITY, f64::min);
        let expected_max_index = window
            .iter()
            .rev()
            .position(|value| *value == expected_max)
            .unwrap();
        let expected_min_index = window
            .iter()
            .rev()
            .position(|value| *value == expected_min)
            .unwrap();

        assert_eq!(max.next(input), expected_max, "position {position}");
        assert_eq!(min.next(input), expected_min, "position {position}");
        assert_eq!(
            max_index.next(input),
            expected_max_index,
            "position {position}"
        );
        assert_eq!(
            min_index.next(input),
            expected_min_index,
            "position {position}"
        );
        assert_eq!(max.current(), Some(expected_max));
        assert_eq!(min.current(), Some(expected_min));
        assert_eq!(max_index.current(), Some(expected_max_index));
        assert_eq!(min_index.current(), Some(expected_min_index));
    }

    Ok(())
}

#[test]
fn window_matches_a_naive_reference_across_updates_and_reset() -> indicator::Result<()> {
    const WINDOW_SIZE: usize = 7;

    let inputs = deterministic_inputs();
    let mut window = Sma::new(1)?.window(WINDOW_SIZE);
    let mut expected = Vec::with_capacity(WINDOW_SIZE);

    for (index, input) in inputs.iter().copied().enumerate() {
        if index == 0 {
            expected.resize(WINDOW_SIZE, input);
        } else {
            expected.remove(0);
            expected.push(input);
        }

        let output = window.next(input);
        assert_eq!(output, expected);
        assert_eq!(window.current(), Some(expected.clone()));
    }

    window.reset();
    assert_eq!(window.current(), None);

    let first_after_reset = inputs[0];
    expected.clear();
    expected.resize(WINDOW_SIZE, first_after_reset);
    assert_eq!(window.next(first_after_reset), expected);

    Ok(())
}

#[test]
fn vwma_matches_naive_recomputation_across_a_long_mixed_scale_sequence() -> indicator::Result<()> {
    const PERIOD: usize = 37;
    const INPUT_COUNT: usize = 20_000;
    const PRICE_SCALES: [f64; 4] = [1.0e-6, 1.0, 1.0e3, 1.0e6];
    const VOLUME_SCALES: [f64; 4] = [1.0e-9, 1.0e-3, 1.0, 1.0e6];

    let mut state = 0x71a5_b9c3_d42e_806f_u64;
    let inputs = (0..INPUT_COUNT)
        .map(|index| {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            let price_scale = PRICE_SCALES[(index / 503) % PRICE_SCALES.len()];
            let price_fraction = 1.0 + f64::from((state >> 32) as u32 % 10_000) / 10_000.0;
            let volume_scale = VOLUME_SCALES[(index / 61) % VOLUME_SCALES.len()];
            let volume_fraction = f64::from((state >> 16) as u16 % 1_000) / 1_000.0;
            let volume = if index % 211 < 7 {
                0.0
            } else {
                volume_scale * volume_fraction
            };
            (price_scale * price_fraction, volume)
        })
        .collect::<Vec<_>>();

    let mut indicator = Vwma::new(PERIOD)?;
    let mut window = Vec::with_capacity(PERIOD);
    for (position, input) in inputs.into_iter().enumerate() {
        if window.is_empty() {
            window.resize(PERIOD, input);
        } else {
            window.remove(0);
            window.push(input);
        }

        let weighted_sum = window
            .iter()
            .map(|(price, volume)| price * volume)
            .sum::<f64>();
        let total_volume = window.iter().map(|(_, volume)| volume).sum::<f64>();
        let expected = (total_volume != 0.0).then_some(weighted_sum / total_volume);
        let actual = indicator.next_option(input);

        match (actual, expected) {
            (Some(actual), Some(expected)) => {
                let tolerance = 1.0e-5 * expected.abs().max(1.0e-6);
                assert!(
                    (actual - expected).abs() <= tolerance,
                    "position {position}: actual {actual}, expected {expected}, tolerance {tolerance}"
                );
            }
            (None, None) => {}
            (actual, expected) => {
                panic!("position {position}: actual {actual:?}, expected {expected:?}")
            }
        }
        assert_eq!(indicator.current(), actual);
    }

    Ok(())
}

#[test]
fn vwap_matches_naive_recomputation_across_a_long_mixed_scale_sequence() -> indicator::Result<()> {
    const INPUT_COUNT: usize = 12_000;
    const PRICE_SCALES: [f64; 4] = [1.0e-6, 1.0, 1.0e3, 1.0e6];
    const VOLUME_SCALES: [f64; 4] = [1.0e-9, 1.0e-3, 1.0, 1.0e6];

    let mut state = 0x6ac1_38d5_917e_42bf_u64;
    let inputs = (0..INPUT_COUNT)
        .map(|index| {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            let price_scale = PRICE_SCALES[(index / 317) % PRICE_SCALES.len()];
            let price_fraction = 1.0 + f64::from((state >> 32) as u32 % 10_000) / 10_000.0;
            let volume_scale = VOLUME_SCALES[(index / 47) % VOLUME_SCALES.len()];
            let volume_fraction = f64::from((state >> 16) as u16 % 1_000) / 1_000.0;
            let volume = if index % 101 < 3 {
                0.0
            } else {
                volume_scale * volume_fraction
            };
            (price_scale * price_fraction, volume)
        })
        .collect::<Vec<_>>();

    let mut indicator = Vwap::new();
    let mut history = Vec::with_capacity(INPUT_COUNT);
    for (position, input) in inputs.into_iter().enumerate() {
        history.push(input);
        let actual = indicator.try_next(input)?;

        if position % 31 != 0 && position + 1 != INPUT_COUNT {
            continue;
        }

        let weighted_sum = history
            .iter()
            .map(|(price, volume)| price * volume)
            .sum::<f64>();
        let total_volume = history.iter().map(|(_, volume)| volume).sum::<f64>();
        let expected = (total_volume != 0.0).then_some(weighted_sum / total_volume);

        match (actual, expected) {
            (Some(actual), Some(expected)) => {
                let tolerance = 1.0e-9 * expected.abs().max(1.0e-6);
                assert!(
                    (actual - expected).abs() <= tolerance,
                    "position {position}: actual {actual}, expected {expected}, tolerance {tolerance}"
                );
            }
            (None, None) => {}
            (actual, expected) => {
                panic!("position {position}: actual {actual:?}, expected {expected:?}")
            }
        }
        assert_eq!(indicator.current(), actual);
    }

    Ok(())
}

#[test]
fn standard_deviation_matches_two_pass_reference_across_long_mixed_scales() -> indicator::Result<()>
{
    const PERIOD: usize = 49;
    const INPUT_COUNT: usize = 20_000;
    const CENTERS: [f64; 4] = [1.0e-6, 1.0e3, -1.0e6, 1.0e9];
    const AMPLITUDES: [f64; 4] = [1.0e-9, 1.0e-2, 1.0, 100.0];

    let mut state = 0x8ce4_195b_76a2_d30f_u64;
    let inputs = (0..INPUT_COUNT)
        .map(|index| {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            let regime = (index / 401) % CENTERS.len();
            let offset = f64::from((state >> 32) as u32 % 2_001) / 1_000.0 - 1.0;
            CENTERS[regime] + AMPLITUDES[regime] * offset
        })
        .collect::<Vec<_>>();

    let mut indicator = StandardDeviation::new(PERIOD)?;
    let mut window = Vec::with_capacity(PERIOD);
    for (position, input) in inputs.into_iter().enumerate() {
        if window.is_empty() {
            window.resize(PERIOD, input);
        } else {
            window.remove(0);
            window.push(input);
        }

        let origin = window[0];
        let mean_offset = window.iter().map(|value| value - origin).sum::<f64>() / PERIOD as f64;
        let expected_mean = origin + mean_offset;
        let expected_sd = (window
            .iter()
            .map(|value| {
                let delta = value - expected_mean;
                delta * delta
            })
            .sum::<f64>()
            / PERIOD as f64)
            .sqrt();

        let actual = indicator.next(input);
        let window_scale = window.iter().map(|value| value.abs()).fold(0.0, f64::max);
        let variation_scale = window
            .iter()
            .map(|value| (value - origin).abs())
            .fold(0.0, f64::max);
        let roundoff_floor = window_scale * f64::EPSILON * 4.0;
        let mean_tolerance = roundoff_floor.max(variation_scale * 1.0e-8).max(1.0e-15);
        let sd_tolerance = roundoff_floor.max(expected_sd * 1.0e-6).max(1.0e-15);

        assert!(
            (actual.mean - expected_mean).abs() <= mean_tolerance,
            "position {position}: mean {}, expected {expected_mean}, tolerance {mean_tolerance}",
            actual.mean
        );
        assert!(
            actual.sd.is_finite() && (actual.sd - expected_sd).abs() <= sd_tolerance,
            "position {position}: sd {}, expected {expected_sd}, tolerance {sd_tolerance}",
            actual.sd
        );
        assert_eq!(indicator.current(), Some(actual));
    }

    Ok(())
}
