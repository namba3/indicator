use indicator::{
    Current, IndicatorExt, Max, MaxIndex, Min, MinIndex, Next, Reset, Sma, StandardDeviation, Vwap,
    Vwma,
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
