use indicator::{Current, IndicatorExt, Max, MaxIndex, Min, MinIndex, Next, Reset, Sma};

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
