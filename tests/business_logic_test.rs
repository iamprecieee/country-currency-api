use std::collections::HashMap;

use currency_exchange_api::utils::countries::{calculate_gdp, process_currency_and_gdp};

#[test]
fn test_gdp_calculation_random_range() {
    let population = 1000000i64;
    let exchange_rate = 100.0;

    let mut results = Vec::new();
    for _ in 0..10 {
        let gdp = calculate_gdp(population, exchange_rate);
        assert!(gdp.is_some());
        let value = gdp.unwrap();

        let min_gdp = (population as f64 * 1000.0) / exchange_rate;
        let max_gdp = (population as f64 * 2000.0) / exchange_rate;

        assert!(
            value >= min_gdp,
            "GDP {} is below minimum {}",
            value,
            min_gdp
        );
        assert!(
            value <= max_gdp,
            "GDP {} is above maximum {}",
            value,
            max_gdp
        );

        results.push(value);
    }

    let first = results[0];
    let all_same = results.iter().all(|&x| x == first);
    assert!(
        !all_same,
        "GDP calculation should produce different random values"
    );
}

#[test]
fn test_gdp_calculation_zero_exchange_rate() {
    let gdp = calculate_gdp(1000000, 0.0);
    assert!(gdp.is_none());
}

#[test]
fn test_currency_handling_empty_array() {
    let rates = HashMap::new();
    let (code, rate, gdp) = process_currency_and_gdp(None, 1000000, &rates);

    assert!(code.is_none());
    assert!(rate.is_none());
    assert_eq!(gdp, Some(0.0));
}

#[test]
fn test_currency_handling_multiple_currencies() {
    use currency_exchange_api::models::responses::Currency;

    let currencies = vec![
        Currency {
            code: Some("NGN".to_string()),
            name: Some("Nigerian Naira".to_string()),
            symbol: Some("₦".to_string()),
        },
        Currency {
            code: Some("USD".to_string()),
            name: Some("US Dollar".to_string()),
            symbol: Some("$".to_string()),
        },
    ];

    let mut rates = HashMap::new();
    rates.insert("NGN".to_string(), 1600.0);
    rates.insert("USD".to_string(), 1.0);

    let (code, rate, gdp) = process_currency_and_gdp(Some(&currencies), 1000000, &rates);

    assert_eq!(code, Some("NGN".to_string()));
    assert!(rate.is_some());
    assert!(gdp.is_some());
}

#[test]
fn test_currency_not_found_in_rates() {
    use currency_exchange_api::models::responses::Currency;

    let currencies = vec![Currency {
        code: Some("XYZ".to_string()),
        name: Some("Unknown".to_string()),
        symbol: Some("?".to_string()),
    }];

    let rates = HashMap::new();

    let (code, rate, gdp) = process_currency_and_gdp(Some(&currencies), 1000000, &rates);

    assert_eq!(code, Some("XYZ".to_string()));
    assert!(rate.is_none());
    assert!(gdp.is_none());
}
