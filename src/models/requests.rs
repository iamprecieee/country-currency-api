use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CountryFilters {
    pub region: Option<String>,
    pub currency: Option<String>,
    pub sort: Option<String>,
}
