use axum::{
    Router,
    routing::{delete, get, post},
};

use crate::{
    models::state::AppState,
    routes::countries::{
        delete_country, get_countries, get_country, get_status, get_summary_image,
        refresh_countries,
    },
};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/countries/refresh", post(refresh_countries))
        .route("/countries", get(get_countries))
        .route("/countries/{name}", get(get_country))
        .route("/countries/{name}", delete(delete_country))
        .route("/status", get(get_status))
        .route("/countries/image", get(get_summary_image))
        .with_state(state)
}
