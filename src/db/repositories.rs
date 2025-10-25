use bigdecimal::BigDecimal;
use chrono::{DateTime, SecondsFormat, Utc};
use sqlx::{QueryBuilder, query};

use crate::{
    db::pool::DbPool,
    models::{country::Country, requests::CountryFilters},
};

#[derive(Clone)]
pub struct CountryRepository {
    pool: DbPool,
}

impl CountryRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn insert_or_update(&self, countries: &[Country]) -> Result<usize, sqlx::Error> {
        if countries.is_empty() {
            return Ok(0);
        }

        const BATCH_SIZE: usize = 100;
        let mut total_saved = 0;

        for chunk in countries.chunks(BATCH_SIZE) {
            let mut query_builder = QueryBuilder::new(
                "INSERT INTO countries (id, name, capital, region, population, currency_code, 
                exchange_rate, estimated_gdp, flag_url, last_refreshed_at)",
            );

            query_builder.push_values(chunk, |mut b, country| {
                b.push_bind(country.id)
                    .push_bind(&country.name)
                    .push_bind(&country.capital)
                    .push_bind(&country.region)
                    .push_bind(country.population)
                    .push_bind(&country.currency_code)
                    .push_bind(&country.exchange_rate)
                    .push_bind(&country.estimated_gdp)
                    .push_bind(&country.flag_url)
                    .push_bind(country.last_refreshed_at.parse::<DateTime<Utc>>().unwrap());
            });

            query_builder.push(
                " ON DUPLICATE KEY UPDATE
                        capital = VALUES(capital),
                        region = VALUES(region),
                        population = VALUES(population),
                        currency_code = VALUES(currency_code),
                        exchange_rate = VALUES(exchange_rate),
                        estimated_gdp = VALUES(estimated_gdp),
                        flag_url = VALUES(flag_url),
                        last_refreshed_at = VALUES(last_refreshed_at)",
            );

            let result = query_builder.build().execute(&self.pool).await?;
            total_saved += result.rows_affected() as usize;
        }

        Ok(total_saved)
    }

    pub async fn filter(&self, filters: &CountryFilters) -> Result<Vec<Country>, sqlx::Error> {
        let mut query = QueryBuilder::new(
            "SELECT id, name, capital, region, population, currency_code, 
                exchange_rate, estimated_gdp, flag_url, last_refreshed_at 
         FROM countries WHERE 1=1",
        );

        if let Some(region) = &filters.region {
            query.push(" AND LOWER(region) = LOWER(");
            query.push_bind(region);
            query.push(")");
        }

        if let Some(currency) = &filters.currency {
            query.push(" AND LOWER(currency_code) = LOWER(");
            query.push_bind(currency);
            query.push(")");
        }

        match filters.sort.as_deref() {
            Some("gdp_asc") => query.push(" ORDER BY estimated_gdp ASC"),
            Some("gdp_desc") | _ => query.push(" ORDER BY estimated_gdp DESC"),
        };

        let rows = query
            .build_query_as::<(
                i32,
                String,
                Option<String>,
                Option<String>,
                i64,
                Option<String>,
                Option<BigDecimal>,
                Option<BigDecimal>,
                Option<String>,
                DateTime<Utc>,
            )>()
            .fetch_all(&self.pool)
            .await?;

        let results = rows
            .into_iter()
            .map(|row| Country {
                id: row.0,
                name: row.1,
                capital: row.2,
                region: row.3,
                population: row.4,
                currency_code: row.5,
                exchange_rate: row.6,
                estimated_gdp: row.7,
                flag_url: row.8,
                last_refreshed_at: row.9.to_rfc3339_opts(SecondsFormat::Millis, true),
            })
            .collect();

        Ok(results)
    }

    pub async fn get_by_name(&self, name: &str) -> Result<Option<Country>, sqlx::Error> {
        let country = query!(
            r#"
            SELECT id, name, capital, region, population, currency_code,
                   exchange_rate, estimated_gdp, flag_url, last_refreshed_at
            FROM countries
            WHERE LOWER(name) = LOWER(?)
            "#,
            name
        )
        .fetch_optional(&self.pool)
        .await?;

        match country {
            Some(row) => Ok(Some(Country {
                id: row.id,
                name: row.name,
                capital: row.capital,
                region: row.region,
                population: row.population,
                currency_code: row.currency_code,
                exchange_rate: row.exchange_rate,
                estimated_gdp: row.estimated_gdp,
                flag_url: row.flag_url,
                last_refreshed_at: row
                    .last_refreshed_at
                    .to_rfc3339_opts(SecondsFormat::Millis, true),
            })),
            None => Ok(None),
        }
    }

    pub async fn delete_by_name(&self, name: &str) -> Result<bool, sqlx::Error> {
        let country = sqlx::query!(
            r#"
            DELETE FROM countries
            WHERE LOWER(name) = LOWER(?)
            "#,
            name
        )
        .execute(&self.pool)
        .await?;

        Ok(country.rows_affected() > 0)
    }

    pub async fn count(&self) -> Result<i64, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM countries
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.count)
    }

    pub async fn get_last_refresh_time(&self) -> Result<Option<String>, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            SELECT MAX(last_refreshed_at) as last_refresh
            FROM countries
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result
            .last_refresh
            .map(|ts| ts.to_rfc3339_opts(SecondsFormat::Millis, true)))
    }
}
