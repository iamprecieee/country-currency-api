use std::time::Duration;

use reqwest::{Client, Error};

use crate::models::responses::CountryResponse;
use crate::models::responses::ExchangeRateResponse;

pub struct CountriesApiClient {
    client: Client,
    api_url: String,
}

impl CountriesApiClient {
    pub fn new(api_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, api_url }
    }

    pub async fn fetch_all_countries(&self) -> Result<Vec<CountryResponse>, Error> {
        let response = self.client.get(&self.api_url).send().await?;
        let countries = response.json::<Vec<CountryResponse>>().await?;

        Ok(countries)
    }
}

pub struct ExchangeApiClient {
    client: Client,
    api_url: String,
}

impl ExchangeApiClient {
    pub fn new(api_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, api_url }
    }

    pub async fn fetch_rates(&self) -> Result<ExchangeRateResponse, Error> {
        let response = self.client.get(&self.api_url).send().await?;
        let rates = response.json::<ExchangeRateResponse>().await?;

        Ok(rates)
    }
}
