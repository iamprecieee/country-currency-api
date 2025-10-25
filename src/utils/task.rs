use std::{collections::HashMap};

use anyhow::Result;
use chrono::{DateTime, Utc};
use rand::random_range;

use crate::{
    db::repositories::CountryRepository,
    models::{
        country::Country,
        requests::CountryFilters,
        responses::{CountryResponse, Currency, ExchangeRateResponse},
    },
    utils::image::generate_summary_image,
};

pub async fn refresh_countries_task(
    repository: CountryRepository,
    countries_data: Vec<CountryResponse>,
    exchange_rate_data: ExchangeRateResponse,
) -> Result<()> {
    let timestamp = Utc::now();

    let countries = countries_data
        .into_iter()
        .map(|country_data| {
            let (currency_code, exchange_rate, estimated_gdp) = process_currency_and_gdp(
                country_data.currencies.as_ref(),
                country_data.population,
                &exchange_rate_data.rates,
            );

            Country {
                id: 0,
                name: country_data.name,
                capital: country_data.capital,
                region: country_data.region,
                population: country_data.population,
                currency_code,
                exchange_rate,
                estimated_gdp,
                flag_url: country_data.flag,
                last_refreshed_at: timestamp.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            }
        })
        .collect::<Vec<Country>>();

    tracing::info!(
        "Processed {} countries, starting batch insert",
        countries.len()
    );

    let saved_count = repository.insert_or_update(&countries).await?;

    tracing::info!("Successfully saved {} countries", saved_count);

    if let Err(e) = generate_image_after_refresh(&repository, timestamp).await {
        tracing::error!("Failed to generate summary image: {:?}", e);
    }

    Ok(())
}

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

async fn generate_image_after_refresh(
    repository: &CountryRepository,
    last_refresh_time: DateTime<Utc>,
) -> Result<()> {
    let total = repository.count().await?;

    let filters = CountryFilters {
        region: None,
        currency: None,
        sort: Some("desc".to_string()),
    };
    let all_countries = repository.filter(&filters).await?;
    let top_5: Vec<_> = all_countries.into_iter().take(5).collect();

    generate_summary_image(total, top_5, last_refresh_time).await?;

    Ok(())
}
