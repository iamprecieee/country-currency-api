use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Country {
    pub id: i32,
    pub name: String,
    pub capital: Option<String>,
    pub region: Option<String>,
    pub population: i64,
    pub currency_code: Option<String>,
    pub exchange_rate: Option<BigDecimal>,
    pub estimated_gdp: Option<BigDecimal>,
    pub flag_url: Option<String>,
    pub last_refreshed_at: String,
}
