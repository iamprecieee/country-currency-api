use axum::{
    Router,
    routing::{delete, get, post},
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    models::{country::Country, requests::CountryFilters, responses::ApiError, state::AppState},
    routes::countries::{
        delete_country, get_countries, get_country, get_status, get_summary_image,
        refresh_countries,
    },
};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::countries::refresh_countries,
        crate::routes::countries::get_countries,
        crate::routes::countries::get_country,
        crate::routes::countries::delete_country,
        crate::routes::countries::get_status,
        crate::routes::countries::get_summary_image,
    ),
    components(
        schemas(
            CountryFilters,
            ApiError,
            Country,
        )
    ),
    tags(
        (name = "Countries", description = "Country Currency & Exchange API endpoints")
    ),
    info(
        title = "Country Currency & Exchange API",
        version = "1.0.0",
        description = "A REST API service that provides country and currency data"
    )
)]
pub struct ApiDoc;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/countries/refresh", post(refresh_countries))
        .route("/countries", get(get_countries))
        .route("/countries/{name}", get(get_country))
        .route("/countries/{name}", delete(delete_country))
        .route("/status", get(get_status))
        .route("/countries/image", get(get_summary_image))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state)
}
