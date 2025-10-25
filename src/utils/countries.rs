use std::collections::HashMap;

use rand::random_range;

use crate::models::responses::Currency;

pub fn process_currency_and_gdp(
    currencies: Option<&Vec<Currency>>,
    population: i64,
    rates: &HashMap<String, f64>,
) -> (Option<String>, Option<f64>, Option<f64>) {
    if currencies.is_none() || currencies.unwrap().is_empty() {
        return (None, None, Some(0.0));
    }

    let currencies = currencies.unwrap();
    let first_currency = &currencies[0];

    let currency_code = match &first_currency.code {
        Some(code) => code.clone(),
        None => return (None, None, Some(0.0)),
    };

    match rates.get(&currency_code) {
        Some(rate) => {
            let estimated_gdp = calculate_gdp(population, *rate);

            (Some(currency_code), Some(*rate), estimated_gdp)
        }
        None => (Some(currency_code), None, None),
    }
}

pub fn calculate_gdp(population: i64, exchange_rate: f64) -> Option<f64> {
    if exchange_rate == 0.0 {
        return None;
    }

    let multiplier = random_range(1000.0..=2000.0);

    Some((population as f64 * multiplier) / exchange_rate)
}
