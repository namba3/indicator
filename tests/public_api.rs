use indicator::{Current, IndicatorExt, Next, Price, Reset, Volume, Vwap, Vwma};

#[test]
fn bollinger_bands_module_api_is_available() -> indicator::Result<()> {
    use indicator::bollinger_bands::{BollingerBands, BollingerBandsOutput};

    let mut bands = BollingerBands::new(2, 2.0)?;
    let output: BollingerBandsOutput = bands.next(100.0);

    assert_eq!(output.average, 100.0);
    assert_eq!(output.upper_bound, 100.0);
    assert_eq!(output.lower_bound, 100.0);

    Ok(())
}

#[test]
fn bollinger_bands_rejects_non_finite_and_negative_multipliers() {
    use indicator::BollingerBands;

    for multiplier in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0] {
        assert!(matches!(
            BollingerBands::new(5, multiplier),
            Err(indicator::Error::InvalidFloatRange(_))
        ));
    }

    assert!(BollingerBands::new(5, 0.0).is_ok());
    assert!(BollingerBands::new(5, f64::MAX).is_ok());
}

#[test]
fn vwap_handles_finite_volumes_whose_sum_exceeds_f64_max() -> indicator::Result<()> {
    let mut vwap = Vwap::new();

    assert_eq!(vwap.try_next((10.0, f64::MAX))?, Some(10.0));
    assert_eq!(vwap.try_next((20.0, f64::MAX))?, Some(15.0));

    let output = vwap.try_next((40.0, f64::MAX))?.unwrap();
    assert!((output - (70.0 / 3.0)).abs() < 1e-12);

    Ok(())
}

#[test]
fn vwap_handles_finite_prices_with_an_unrepresentable_difference() -> indicator::Result<()> {
    let mut vwap = Vwap::new();

    assert_eq!(vwap.try_next((-f64::MAX, 1.0))?, Some(-f64::MAX));
    assert_eq!(vwap.try_next((f64::MAX, 1.0))?, Some(0.0));

    Ok(())
}

#[test]
fn vwma_stays_stable_with_extreme_prices_and_volumes() -> indicator::Result<()> {
    let mut large_volumes = Vwma::new(2)?;
    assert_eq!(large_volumes.next_option((10.0, f64::MAX)), Some(10.0));
    assert_eq!(large_volumes.next_option((20.0, f64::MAX)), Some(15.0));
    assert_eq!(large_volumes.next_option((40.0, f64::MAX)), Some(30.0));

    let mut extreme_prices = Vwma::new(2)?;
    assert_eq!(
        extreme_prices.next_option((-f64::MAX, 1.0)),
        Some(-f64::MAX)
    );
    assert_eq!(extreme_prices.next_option((f64::MAX, 1.0)), Some(0.0));

    let mut scale_expires = Vwma::new(2)?;
    assert_eq!(scale_expires.next_option((10.0, f64::MAX)), Some(10.0));
    assert_eq!(scale_expires.next_option((20.0, 1.0)), Some(10.0));
    assert_eq!(scale_expires.next_option((40.0, 1.0)), Some(30.0));

    Ok(())
}

#[test]
fn rolling_indicators_report_unallocatable_periods() {
    use indicator::{Max, MaxIndex, Min, MinIndex, Sma, StandardDeviation};

    assert!(matches!(
        Sma::new(usize::MAX),
        Err(indicator::Error::AllocationFailed)
    ));
    assert!(matches!(
        StandardDeviation::new(usize::MAX),
        Err(indicator::Error::AllocationFailed)
    ));
    assert!(matches!(
        Max::new(usize::MAX),
        Err(indicator::Error::AllocationFailed)
    ));
    assert!(matches!(
        Min::new(usize::MAX),
        Err(indicator::Error::AllocationFailed)
    ));
    assert!(matches!(
        MaxIndex::new(usize::MAX),
        Err(indicator::Error::AllocationFailed)
    ));
    assert!(matches!(
        MinIndex::new(usize::MAX),
        Err(indicator::Error::AllocationFailed)
    ));
    assert!(matches!(
        Vwma::new(usize::MAX),
        Err(indicator::Error::AllocationFailed)
    ));
    assert!(matches!(
        indicator::AroonIndicator::new(usize::MAX - 1),
        Err(indicator::Error::AllocationFailed)
    ));
}

#[test]
fn window_iterator_exposes_exact_size_and_fused_guarantees() -> indicator::Result<()> {
    fn assert_iterator_traits<I: ExactSizeIterator + core::iter::FusedIterator>(_: &I) {}

    let mut window = indicator::Sma::new(1)?.window(3);
    assert_iterator_traits(&window.iter());
    assert_eq!(window.iter().len(), 0);

    window.next(5.0);
    let mut iter = window.iter();
    assert_eq!(iter.len(), 3);
    assert_eq!(iter.next(), Some(&5.0));
    assert_eq!(iter.len(), 2);
    assert_eq!(iter.collect::<Vec<_>>(), [&5.0, &5.0]);

    window.next(8.0);
    assert_eq!(window.iter().len(), 3);
    assert_eq!(window.iter().copied().collect::<Vec<_>>(), [5.0, 5.0, 8.0]);

    window.reset();
    assert_eq!(window.iter().len(), 0);

    Ok(())
}

#[derive(Clone)]
struct Trade {
    price: f64,
    volume: f64,
}

impl Price for Trade {
    fn price(&self) -> f64 {
        self.price
    }
}

impl Volume for Trade {
    fn volume(&self) -> f64 {
        self.volume
    }
}

#[test]
fn custom_market_data_composes_public_volume_indicators() -> indicator::Result<()> {
    let mut indicators = Vwap::new().together(Vwma::new(2)?);
    let trades = [
        Trade {
            price: 100.0,
            volume: 1.0,
        },
        Trade {
            price: 110.0,
            volume: 1.0,
        },
        Trade {
            price: 120.0,
            volume: 2.0,
        },
    ];

    assert_eq!(indicators.current(), None);
    assert_eq!(indicators.next(&trades[0]), (100.0, 100.0));
    assert_eq!(indicators.current(), Some((100.0, 100.0)));
    assert_eq!(indicators.next(&trades[1]), (105.0, 105.0));

    let (vwap, vwma) = indicators.next(&trades[2]);
    assert!((vwap - 112.5).abs() < 1e-12);
    assert!((vwma - (350.0 / 3.0)).abs() < 1e-12);

    indicators.reset();
    assert_eq!(indicators.current(), None);
    assert_eq!(indicators.next(&trades[0]), (100.0, 100.0));

    Ok(())
}

#[test]
fn option_api_accepts_custom_market_data() {
    let mut vwap = Vwap::new();
    let zero_volume_trade = Trade {
        price: 100.0,
        volume: 0.0,
    };
    let trade = Trade {
        price: 110.0,
        volume: 2.0,
    };

    assert_eq!(vwap.next_option(&zero_volume_trade), None);
    assert_eq!(vwap.next_option(&trade), Some(110.0));
}

#[test]
fn invalid_volume_is_reported_without_mutating_indicators() -> indicator::Result<()> {
    let mut vwap = Vwap::new();
    assert_eq!(vwap.try_next((100.0, 1.0))?, Some(100.0));
    let error = vwap
        .try_next_ref(&Trade {
            price: 999.0,
            volume: f64::INFINITY,
        })
        .unwrap_err();
    assert!(matches!(error, indicator::Error::InvalidVolume(_)));
    assert_eq!(vwap.current(), Some(100.0));
    assert!(matches!(
        vwap.try_next((f64::NAN, 1.0)),
        Err(indicator::Error::InvalidPrice(_))
    ));
    assert_eq!(vwap.current(), Some(100.0));

    let mut vwma = Vwma::new(2)?;
    assert_eq!(vwma.try_next((100.0, 1.0))?, Some(100.0));
    assert!(matches!(
        vwma.try_next((999.0, -1.0)),
        Err(indicator::Error::InvalidVolume(_))
    ));
    assert_eq!(vwma.current(), Some(100.0));
    assert!(matches!(
        vwma.try_next((f64::INFINITY, 1.0)),
        Err(indicator::Error::InvalidPrice(_))
    ));
    assert_eq!(vwma.current(), Some(100.0));

    Ok(())
}
