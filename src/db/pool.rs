use std::time::Duration;

use sqlx::{MySql, Pool, mysql::MySqlPoolOptions};

pub type DbPool = Pool<MySql>;

pub async fn create_pool(
    database_url: &str,
    max_connections: u32,
    connection_timeout: u64,
) -> Result<DbPool, sqlx::Error> {
    MySqlPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(Duration::from_secs(connection_timeout))
        .connect(database_url)
        .await
}
