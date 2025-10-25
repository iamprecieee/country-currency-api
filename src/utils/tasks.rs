use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::{
    db::repositories::CountryRepository,
    models::{
        country::Country,
        requests::CountryFilters,
        responses::{CountryResponse, ExchangeRateResponse},
    },
    utils::{countries::process_currency_and_gdp, image::generate_summary_image},
};

pub async fn refresh_countries_task(
    repository: CountryRepository,
    countries_data: Vec<CountryResponse>,
    exchange_rate_data: ExchangeRateResponse,
    timestamp: DateTime<Utc>,
) -> Result<()> {
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

    Ok(())
}

pub async fn generate_image_task(
    repository: CountryRepository,
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
