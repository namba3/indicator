use indicator::{Current, IndicatorExt, Next, Price, Reset, Volume, Vwap, Vwma};

#[test]
fn correctly_spelled_bollinger_bands_api_is_available() -> indicator::Result<()> {
    use indicator::bollinger_bands::{BollingerBands, BollingerBandsOutput};

    let mut bands = BollingerBands::new(2, 2.0)?;
    let output: BollingerBandsOutput = bands.next(100.0);

    assert_eq!(output.average, 100.0);
    assert_eq!(output.upper_bound, 100.0);
    assert_eq!(output.lower_bound, 100.0);

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
