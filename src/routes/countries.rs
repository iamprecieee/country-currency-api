use axum::{
    Json,
    body::Body,
    extract::{Path, Query, State},
    http::Response,
    response::IntoResponse,
};
use chrono::Utc;
use reqwest::StatusCode;
use serde_json::json;

use crate::{
    models::{
        country::Country,
        requests::CountryFilters,
        responses::{ApiError, RefreshResponse, StatusResponse},
        state::AppState,
    },
    utils::{
        clients::{CountriesApiClient, ExchangeApiClient},
        tasks::{generate_image_task, refresh_countries_task},
    },
};

#[utoipa::path(
    post,
    path = "/countries/refresh",
    responses(
        (status = 200, description = "Refresh started in background"),
        (status = 503, description = "Service Unavailable - External data source unavailable", body = ApiError),
    ),
    tag = "Countries"
)]
pub async fn refresh_countries(State(state): State<AppState>) -> impl IntoResponse {
    let countries_client = CountriesApiClient::new(state.config.rest_countries_api);
    let exchange_client = ExchangeApiClient::new(state.config.exchange_rates_api);

    let countries_data = countries_client.fetch_all_countries().await;
    if let Err(e) = countries_data {
        tracing::error!("Countries API unavailable: {:?}", e);
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ApiError::with_details(
                "External data source unavailable".to_string(),
                "Could not fetch data from restcountries API".into(),
            )),
        )
            .into_response();
    }

    let exchange_rate_data = exchange_client.fetch_rates().await;
    if let Err(e) = exchange_rate_data {
        tracing::error!("Exchange rates API unavailable: {:?}", e);
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ApiError::with_details(
                "External data source unavailable".to_string(),
                "Could not fetch data from exchange rates API".into(),
            )),
        )
            .into_response();
    }

    tokio::spawn(async move {
        let timestamp = Utc::now();

        match refresh_countries_task(
            state.repository.clone(),
            countries_data.unwrap(),
            exchange_rate_data.unwrap(),
            timestamp,
        )
        .await
        {
            Ok(_) => tracing::info!("Refresh completed successfully"),
            Err(e) => tracing::error!("Refresh failed: {:?}", e),
        }

        match generate_image_task(state.repository.clone(), timestamp).await {
            Ok(_) => tracing::info!("Image generated successfully"),
            Err(e) => tracing::error!("Failed to generate summary image: {:?}", e),
        }
    });

    (
        StatusCode::OK,
        Json(RefreshResponse {
            message: "Refresh started in background".to_string(),
        }),
    )
        .into_response()
}

#[utoipa::path(
    get,
    path = "/countries",
    params(CountryFilters),
    responses(
        (status = 200, description = "List of countries matching filters", body = [Country]),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "Countries"
)]
pub async fn get_countries(
    State(state): State<AppState>,
    Query(filters): Query<CountryFilters>,
) -> impl IntoResponse {
    match state.repository.filter(&filters).await {
        Ok(countries) => (StatusCode::OK, Json(countries)).into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch countries: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("Internal server error")),
            )
                .into_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/countries/{name}",
    params(
        ("name" = String, Path, description = "The name of country to retrieve")
    ),
    responses(
        (status = 200, description = "Country found", body = Country),
        (status = 404, description = "Country not found", body = Country),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "Countries"
)]
pub async fn get_country(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    match state.repository.get_by_name(&name).await {
        Ok(Some(country)) => (StatusCode::OK, Json(country)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiError::new("Country not found")),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch country: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("Internal server error")),
            )
                .into_response()
        }
    }
}

#[utoipa::path(
    delete,
    path = "/countries/{name}",
    params(
        ("name" = String, Path, description = "The name of country to delete")
    ),
    responses(
        (status = 204),
        (status = 404, description = "Country not found", body = Country),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "Countries"
)]
pub async fn delete_country(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    match state.repository.delete_by_name(&name).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(ApiError::new("Country not found")),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to delete country: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("Internal server error")),
            )
                .into_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/status",
    responses(
        (status = 200)
    ),
    tag = "Status"
)]
pub async fn get_status(State(state): State<AppState>) -> impl IntoResponse {
    let count = state.repository.count().await.unwrap_or(0);
    let last_refresh = state.repository.get_last_refresh_time().await.ok();

    match last_refresh {
        Some(timestamp) => (
            StatusCode::OK,
            Json(StatusResponse {
                total_countries: count,
                last_refreshed_at: timestamp,
            }),
        )
            .into_response(),
        None => (
            StatusCode::OK,
            Json(json!({
                "total_countries": count,
                "last_refreshed_at": null
            })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/countries/image",
    responses(
        (status = 200, description = "Summary image successfully retrieved", content_type = "image/png"),
        (status = 404, description = "Summary image not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "Summary"
)]
pub async fn get_summary_image() -> impl IntoResponse {
    let image_path = "cache/summary.png";

    if !std::path::Path::new(image_path).exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiError::new("Summary image not found")),
        )
            .into_response();
    }

    match tokio::fs::read(image_path).await {
        Ok(contents) => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "image/png")
            .body(Body::from(contents))
            .unwrap()
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to read image: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError::new("Failed to serve image")),
            )
                .into_response()
        }
    }
}
