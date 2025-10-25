use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct CountryFilters {
    /// Filter by region name (e.g. "Africa")
    pub region: Option<String>,

    /// Filter by currency code (e.g. "NGN")
    pub currency: Option<String>,

    /// Sort by gdp value (e.g. "asc" or "desc")
    pub sort: Option<String>,
}
