
use tower::util::ServiceExt;

use axum::{Router, body::Body, http::Request};
use currency_exchange_api::{
    api::build_router,
    db::{pool::create_pool, repositories::CountryRepository},
    models::state::AppState,
    utils::config::load_config,
};
use dotenvy::dotenv;
use reqwest::StatusCode;
use serde_json::{Value, json};
use sqlx::MySqlPool;

async fn setup_test_app() -> (Router, MySqlPool) {
    dotenv().ok();
    let config = load_config().expect("Failed to load config");

    let pool = create_pool(
        &config.database_url,
        config.database_max_connections,
        config.database_connection_timeout,
    )
    .await
    .expect("Failed to create pool");

    sqlx::query("DELETE FROM countries")
        .execute(&pool)
        .await
        .expect("Failed to clean database");

    let repository = CountryRepository::new(pool.clone());
    let state = AppState { repository, config };

    let app = build_router(state);

    (app, pool)
}

async fn make_request(app: &mut Router, method: &str, path: &str) -> (StatusCode, Value) {
    let request = Request::builder()
        .method(method)
        .uri(path)
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap_or(json!({}));

    (status, json)
}

#[tokio::test]
async fn test_status_empty_database() {
    let (mut app, _pool) = setup_test_app().await;

    let (status, body) = make_request(&mut app, "GET", "/status").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total_countries"], 0);
    assert!(body["last_refreshed_at"].is_null());
}

#[tokio::test]
async fn test_get_countries_without_filters() {
    let (mut app, pool) = setup_test_app().await;

    sqlx::query(
        "INSERT INTO countries (name, capital, region, population, currency_code, exchange_rate, estimated_gdp, flag_url)
         VALUES ('Nigeria', 'Abuja', 'Africa', 206139589, 'NGN', 1600.23, 25767448125.2, 'https://flagcdn.com/ng.svg')"
    )
    .execute(&pool)
    .await
    .unwrap();

    let (status, body) = make_request(&mut app, "GET", "/countries").await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array());
    assert_eq!(body.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_get_countries_with_region_filter() {
    let (mut app, pool) = setup_test_app().await;

    sqlx::query(
        "INSERT INTO countries (name, region, population, currency_code)
         VALUES ('Nigeria', 'Africa', 206139589, 'NGN'), ('France', 'Europe', 65273511, 'EUR')"
    )
    .execute(&pool)
    .await
    .unwrap();

    let (status, body) = make_request(&mut app, "GET", "/countries?region=Africa").await;

    assert_eq!(status, StatusCode::OK);
    let countries = body.as_array().unwrap();
    assert_eq!(countries.len(), 1);
    assert_eq!(countries[0]["name"], "Nigeria");
}

#[tokio::test]
async fn test_get_countries_with_currency_filter() {
    let (mut app, pool) = setup_test_app().await;

    sqlx::query(
        "INSERT INTO countries (name, region, population, currency_code)
         VALUES ('Nigeria', 'Africa', 206139589, 'NGN'), ('Ghana', 'Africa', 31072940, 'GHS')"
    )
    .execute(&pool)
    .await
    .unwrap();

    let (status, body) = make_request(&mut app, "GET", "/countries?currency=NGN").await;

    assert_eq!(status, StatusCode::OK);
    let countries = body.as_array().unwrap();
    assert_eq!(countries.len(), 1);
    assert_eq!(countries[0]["name"], "Nigeria");
}

#[tokio::test]
async fn test_get_countries_with_sort_gdp_desc() {
    let (mut app, pool) = setup_test_app().await;

    sqlx::query(
        "INSERT INTO countries (name, population, currency_code, estimated_gdp)
         VALUES ('Country1', 1000000, 'USD', 5000000), ('Country2', 2000000, 'EUR', 10000000)"
    )
    .execute(&pool)
    .await
    .unwrap();

    let (status, body) = make_request(&mut app, "GET", "/countries?sort=gdp_desc").await;

    assert_eq!(status, StatusCode::OK);
    let countries = body.as_array().unwrap();
    assert_eq!(countries[0]["name"], "Country2"); 
    assert_eq!(countries[1]["name"], "Country1");
}

#[tokio::test]
async fn test_get_countries_with_sort_gdp_asc() {
    let (mut app, pool) = setup_test_app().await;

    sqlx::query(
        "INSERT INTO countries (name, population, currency_code, estimated_gdp)
         VALUES ('Country1', 1000000, 'USD', 5000000), ('Country2', 2000000, 'EUR', 10000000)"
    )
    .execute(&pool)
    .await
    .unwrap();

    let (status, body) = make_request(&mut app, "GET", "/countries?sort=gdp_asc").await;

    assert_eq!(status, StatusCode::OK);
    let countries = body.as_array().unwrap();
    assert_eq!(countries[0]["name"], "Country1"); 
    assert_eq!(countries[1]["name"], "Country2");
}

#[tokio::test]
async fn test_get_countries_combined_filters() {
    let (mut app, pool) = setup_test_app().await;

    sqlx::query(
        "INSERT INTO countries (name, region, population, currency_code, estimated_gdp)
         VALUES
         ('Nigeria', 'Africa', 206139589, 'NGN', 25000000),
         ('Ghana', 'Africa', 31072940, 'GHS', 15000000),
         ('France', 'Europe', 65273511, 'EUR', 50000000)"
    )
    .execute(&pool)
    .await
    .unwrap();

    let (status, body) =
        make_request(&mut app, "GET", "/countries?region=Africa&sort=gdp_desc").await;

    assert_eq!(status, StatusCode::OK);
    let countries = body.as_array().unwrap();
    assert_eq!(countries.len(), 2);
    assert_eq!(countries[0]["name"], "Nigeria"); 
    assert_eq!(countries[1]["name"], "Ghana");
}

#[tokio::test]
async fn test_get_country_by_name_case_insensitive() {
    let (mut app, pool) = setup_test_app().await;

    sqlx::query(
        "INSERT INTO countries (name, capital, region, population, currency_code)
         VALUES ('Nigeria', 'Abuja', 'Africa', 206139589, 'NGN')"
    )
    .execute(&pool)
    .await
    .unwrap();

    let (status, body) = make_request(&mut app, "GET", "/countries/nigeria").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "Nigeria");

    let (status, body) = make_request(&mut app, "GET", "/countries/NIGERIA").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "Nigeria");

    let (status, body) = make_request(&mut app, "GET", "/countries/NiGeRiA").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "Nigeria");
}

#[tokio::test]
async fn test_get_country_not_found() {
    let (mut app, _pool) = setup_test_app().await;

    let (status, body) = make_request(&mut app, "GET", "/countries/NonExistent").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "Country not found");
}

#[tokio::test]
async fn test_delete_country_success() {
    let (mut app, pool) = setup_test_app().await;

    sqlx::query(
        "INSERT INTO countries (name, population, currency_code)
         VALUES ('TestCountry', 1000000, 'TST')"
    )
    .execute(&pool)
    .await
    .unwrap();

    let (status, _body) = make_request(&mut app, "DELETE", "/countries/TestCountry").await;

    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _body) = make_request(&mut app, "GET", "/countries/TestCountry").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_country_not_found() {
    let (mut app, _pool) = setup_test_app().await;

    let (status, body) = make_request(&mut app, "DELETE", "/countries/NonExistent").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "Country not found");
}

#[tokio::test]
async fn test_delete_country_case_insensitive() {
    let (mut app, pool) = setup_test_app().await;

    sqlx::query(
        "INSERT INTO countries (name, population, currency_code)
         VALUES ('Nigeria', 206139589, 'NGN')"
    )
    .execute(&pool)
    .await
    .unwrap();

    let (status, _body) = make_request(&mut app, "DELETE", "/countries/nigeria").await;

    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_country_fields_structure() {
    let (mut app, pool) = setup_test_app().await;

    sqlx::query(
        "INSERT INTO countries (name, capital, region, population, currency_code, exchange_rate, estimated_gdp, flag_url)
         VALUES ('Nigeria', 'Abuja', 'Africa', 206139589, 'NGN', 1600.23, 25767448125.2, 'https://flagcdn.com/ng.svg')"
    )
    .execute(&pool)
    .await
    .unwrap();

    let (status, body) = make_request(&mut app, "GET", "/countries/Nigeria").await;

    assert_eq!(status, StatusCode::OK);

    assert!(body.get("id").is_some());
    assert_eq!(body["name"], "Nigeria");
    assert_eq!(body["capital"], "Abuja");
    assert_eq!(body["region"], "Africa");
    assert_eq!(body["population"], 206139589);
    assert_eq!(body["currency_code"], "NGN");
    assert!(body.get("exchange_rate").is_some());
    assert!(body.get("estimated_gdp").is_some());
    assert_eq!(body["flag_url"], "https://flagcdn.com/ng.svg");
    assert!(body.get("last_refreshed_at").is_some());
}

#[tokio::test]
async fn test_country_with_null_fields() {
    let (mut app, pool) = setup_test_app().await;

    sqlx::query(
        "INSERT INTO countries (name, population, currency_code, exchange_rate, estimated_gdp)
         VALUES ('TestCountry', 1000000, NULL, NULL, 0)"
    )
    .execute(&pool)
    .await
    .unwrap();

    let (status, body) = make_request(&mut app, "GET", "/countries/TestCountry").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "TestCountry");
    assert!(body["capital"].is_null());
    assert!(body["region"].is_null());
    assert!(body["currency_code"].is_null());
    assert!(body["exchange_rate"].is_null());
    assert!(body["flag_url"].is_null());
}

#[tokio::test]
async fn test_image_endpoint_not_found() {
    let (mut app, _pool) = setup_test_app().await;

    std::fs::remove_file("cache/summary.png").ok();

    let (status, body) = make_request(&mut app, "GET", "/countries/image").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "Summary image not found");
}

#[tokio::test]
async fn test_status_after_data_inserted() {
    let (mut app, pool) = setup_test_app().await;

    sqlx::query(
        "INSERT INTO countries (name, population, currency_code)
         VALUES ('Country1', 1000000, 'USD'), ('Country2', 2000000, 'EUR')"
    )
    .execute(&pool)
    .await
    .unwrap();

    let (status, body) = make_request(&mut app, "GET", "/status").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total_countries"], 2);
    assert!(body.get("last_refreshed_at").is_some());
}