use std::time::Duration;

use reqwest::{Client, Error};

use crate::models::responses::CountryResponse;

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
