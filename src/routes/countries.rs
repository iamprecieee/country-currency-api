use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use reqwest::StatusCode;
use serde_json::json;

use crate::models::{
    requests::CountryFilters,
    responses::{ApiError, RefreshResponse, StatusResponse},
    state::AppState,
};

pub async fn refresh_countries(State(_state): State<AppState>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(RefreshResponse {
            message: "Refresh started in background".to_string(),
        }),
    )
        .into_response()
}

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

pub async fn get_status(State(state): State<AppState>) -> impl IntoResponse {
    let count = state.repository.count().await.unwrap_or(0);
    let last_refresh = state.repository.get_last_refresh_time().await.ok();

    match last_refresh {
        Some(timestamp) => (
            StatusCode::OK,
            Json(StatusResponse {
                total_countries: count,
                last_refreshed_at: timestamp,
            })
            .into_response(),
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

pub async fn get_summary_image() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(ApiError::new("Summary image not found")),
    )
        .into_response()
}
