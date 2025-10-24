use anyhow::Error;
use chrono::{DateTime, Utc};

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

    pub async fn insert_or_update(&self, country: &Country) -> Result<(), Error> {
        todo!()
    }

    pub async fn filter(&self, filters: &CountryFilters) -> Result<Vec<Country>, sqlx::Error> {
        todo!()
    }

    pub async fn get_by_name(&self, name: &str) -> Result<Option<Country>, sqlx::Error> {
        todo!()
    }

    pub async fn delete_by_name(&self, name: &str) -> Result<bool, sqlx::Error> {
        todo!()
    }

    pub async fn count(&self) -> Result<i64, sqlx::Error> {
        todo!()
    }

    pub async fn get_last_refresh_time(&self) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
        todo!()
    }
}
