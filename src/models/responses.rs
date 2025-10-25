use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Debug, Deserialize)]
pub struct CountryResponse {
    pub name: String,
    pub capital: Option<String>,
    pub region: Option<String>,
    pub population: i64,
    pub currencies: Option<Vec<Currency>>,
    pub flag: Option<String>,
    #[serde(default)]
    pub independent: bool,
}

#[derive(Debug, Deserialize)]
pub struct Currency {
    pub code: Option<String>,
    pub name: Option<String>,
    pub symbol: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExchangeRateResponse {
    pub rates: HashMap<String, f64>,
}

#[derive(Debug, Serialize)]
pub struct RefreshResponse {
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub total_countries: i64,
    pub last_refreshed_at: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiError {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

impl ApiError {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            details: None,
        }
    }

    pub fn with_details(error: impl Into<String>, details: Value) -> Self {
        Self {
            error: error.into(),
            details: Some(details),
        }
    }
}
