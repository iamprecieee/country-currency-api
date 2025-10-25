use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Country {
    pub id: i32,
    pub name: String,
    pub capital: Option<String>,
    pub region: Option<String>,
    pub population: i64,
    pub currency_code: Option<String>,
    #[schema(value_type = f64, example = 1234.56)]
    pub exchange_rate: Option<BigDecimal>,
    #[schema(value_type = f64, example = 1234.56)]
    pub estimated_gdp: Option<BigDecimal>,
    pub flag_url: Option<String>,
    pub last_refreshed_at: String,
}
