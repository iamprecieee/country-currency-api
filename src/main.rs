use anyhow::{Ok, Result};
use currency_exchange_api::{
    api::build_router,
    db::{pool::create_pool, repositories::CountryRepository},
    models::state::AppState,
    utils::config::load_config,
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config = load_config()?;
    tracing::info!("Configuration loaded successfully");

    let pool = create_pool(
        &config.database_url,
        config.database_max_connections,
        config.database_connection_timeout,
    )
    .await?;
    tracing::info!("Database connection pool created");

    let address = format!("{}:{}", config.server_host, &config.server_port);

    let repository = CountryRepository::new(pool);
    let state = AppState { repository, config };

    let app = build_router(state);

    let listener = TcpListener::bind(&address).await?;
    tracing::info!("Server running on {}", address);

    axum::serve(listener, app).await?;

    Ok(())
}
