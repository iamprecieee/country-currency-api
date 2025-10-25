# Country Currency & Exchange API

## Overview

A RESTful API that fetches country and currency data from external APIs, stores them in a database, and provides CRUD operations with filtering and sorting support. The database acts as a local cache of external data that's refreshed on demand.

## Architecture

### Component Stack

- **Web Framework**: Axum
- **Database**: MySQL (with SQLx)
- **HTTP Client**: reqwest
- **Image Generation**: image crate + imageproc
- **Background Tasks**: Tokio spawn (simple async tasks)

### Request Flow

```
POST /countries/refresh
    ↓
Background Task Spawned → Return 202 Accepted immediately
    ↓
Fetch from countries API
    ↓
Fetch from exchange rates API
    ↓
Match currencies to rates
    ↓
Compute estimated_gdp
    ↓
Upsert to MySQL
    ↓
Generate summary image
```

```
GET /countries (with filters)
    ↓
Query MySQL directly
    ↓
Return results
```

## Data Model

### MySQL Schema

**Table: `countries`**

```sql
CREATE TABLE countries (
    id INT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    capital VARCHAR(255),
    region VARCHAR(100),
    population BIGINT NOT NULL,
    currency_code VARCHAR(10),
    exchange_rate DECIMAL(20, 8),
    estimated_gdp DECIMAL(30, 2),
    flag_url TEXT,
    last_refreshed_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    INDEX idx_region (region),
    INDEX idx_currency (currency_code),
    INDEX idx_estimated_gdp (estimated_gdp)
    INDEX idx_name (name),
);
```

### Rust Models

```rust
struct Country {
    id: i32,
    name: String,
    capital: Option,
    region: Option,
    population: i64,
    currency_code: Option,
    exchange_rate: Option,
    estimated_gdp: Option,
    flag_url: Option,
    last_refreshed_at: DateTime,
}
```

## API Endpoints

### POST /countries/refresh

**Purpose**: Trigger background refresh of all country data

**Processing Strategy**:
- Spawn background task immediately
- Return `200 Ok` with message
- Background task does the heavy lifting:
  1. Fetch countries from countries API
  2. Fetch exchange rates from exchange API
  3. Process and match data
  4. Upsert to database
  5. Generate summary image

**Response**:
```json
{
  "message": "Refresh started in background",
}
```

### GET /countries

**Purpose**: List countries with optional filters

**Query Parameters**:
- `region`: Filter by region (e.g., "Africa")
- `currency`: Filter by currency code (e.g., "NGN")
- `sort`: Sort order ("gdp_desc" or "gdp_asc")

**Response**:
```json
[
  {
    "id": 1,
    "name": "Nigeria",
    "capital": "Abuja",
    "region": "Africa",
    "population": 206139589,
    "currency_code": "NGN",
    "exchange_rate": 1600.23,
    "estimated_gdp": 25767448125.2,
    "flag_url": "https://flagcdn.com/ng.svg",
    "last_refreshed_at": "2025-10-22T18:00:00Z"
  },
]
```

### GET /countries/:name

**Purpose**: Get single country by name (case-insensitive)

**Processing**:
- Query by name (case-insensitive match)
- Return `404` if not found

### DELETE /countries/:name

**Purpose**: Remove a country from cache

**Response**:
- `204` No Content if deleted
- `404` if not found

### GET /status

**Purpose**: Show cache status

**Response**:
```json
{
  "total_countries": 250,
  "last_refreshed_at": "2025-10-22T18:00:00Z"
}
```

### GET /countries/image

**Purpose**: Serve generated summary image

**Response**:
- PNG image (binary)
- `404` if image doesn't exist yet

## External API Integration

### restcountries API

**Endpoint**: `https://restcountries.com/v2/all?fields=name,capital,region,population,flag,currencies`

**Response Structure**:
```json
[
  {
    "name": "Nigeria",
    "capital": "Abuja",
    "region": "Africa",
    "population": 206139589,
    "currencies": [
        {
            "code": "NGN",
        "name": "Nigerian naira",
        "symbol": "₦"
      }
    ],
    "flag": "https://flagcdn.com/ng.svg",
    "independent": false
  }
]
```

**Handling**:
- Take first currency code if multiple exist
- If `currencies` is empty: Set `currency_code` and `exchange_rate` to `NULL`, `estimated_gdp` to `0`
- Store country even if currency is missing

### Exchange Rates API

**Endpoint**: `https://open.er-api.com/v6/latest/USD`

**Response Structure**:
```json
{
  "rates": {
    "NGN": 1600.23,
    "GHS": 15.34,
    "USD": 1.0
  }
}
```

**Matching Logic**:
- Look up country's `currency_code` in rates map
- If found: Store rate
- If not found: Set `exchange_rate` to `NULL`

## Business Logic

### Currency Handling

```rust
if currencies.is_empty() {
    currency_code = None;
    exchange_rate = None;
    estimated_gdp = Some(0.0);
} else {
    let code = currencies[0].code;
    currency_code = Some(code.clone());
    
    match rates.get(&code) {
        Some(rate) => {
            exchange_rate = Some(*rate);
            estimated_gdp = calculate_gdp(population, *rate);
        }
        None => {
            exchange_rate = None;
            estimated_gdp = None;
        }
    }
}
```

### GDP Calculation

```rust
fn calculate_gdp(population: i64, exchange_rate: f64) -> Option {
    if exchange_rate == 0.0 {
        return None;
    }
    
    // Generate new random multiplier on EVERY refresh for EVERY country.
    let mut rng = rand::thread_rng();
    let multiplier = rng.gen_range(1000.0..=2000.0);
    
    Some((population as f64 * multiplier) / exchange_rate)
}
```

### Upsert Logic

```sql
INSERT INTO countries (name, capital, region, population, currency_code, exchange_rate, estimated_gdp, flag_url, last_refreshed_at)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
ON DUPLICATE KEY UPDATE
    capital = VALUES(capital),
    region = VALUES(region),
    population = VALUES(population),
    currency_code = VALUES(currency_code),
    exchange_rate = VALUES(exchange_rate),
    estimated_gdp = VALUES(estimated_gdp),
    flag_url = VALUES(flag_url),
    last_refreshed_at = VALUES(last_refreshed_at)
```

## Image Generation

### Requirements

Generate `cache/summary.png` with:
- Total number of countries
- Top 5 countries by estimated GDP
- Timestamp of last refresh

### Simple Implementation Approach

```rust
use image::{ImageBuffer, Rgb};
use imageproc::drawing::draw_text_mut;

// Create 800x600 white canvas
// Draw title: "Country Data Summary"
// Draw total countries
// Draw top 5 in a list
// Draw timestamp
// Save to cache/summary.png
```

### External API Errors

**If restcountries API fails:**
```json
{
  "error": "External data source unavailable",
  "details": "Could not fetch data from [API name]"
}
```
Status: `503 Service Unavailable`

### Validation Errors

**Missing required fields:**
```json
{
  "error": "Validation failed",
  "details": {
    "name_of_field": "is required"
  }
}
```
Status: `400 Bad Request`

### Standard Error Responses

```rust
struct ApiError {
    error: String,
    details: Option,
}
```

## Query Filters

### Region Filter

```sql
WHERE region = ?
```
Case-insensitive match.

### Currency Filter

```sql
WHERE currency_code = ?
```
Exact match (case-insensitive).

### Sorting

**GDP Descending (default):**
```sql
ORDER BY estimated_gdp DESC NULLS LAST
```

**GDP Ascending:**
```sql
ORDER BY estimated_gdp ASC NULLS LAST
```

## Performance Considerations

### Database

- Index on `name` (unique constraint also creates index)
- Index on `region` for filter queries
- Index on `currency_code` for filter queries
- Connection pool: 10-20 connections

### External API Calls

- Use timeouts (30 seconds per request)
- Single refresh endpoint - no concurrent refreshes needed
- If refresh fails, keep old data intact

### Background Tasks

```rust
tokio::spawn(async move {
    match refresh_countries_task(state).await {
        Ok(_) => tracing::info!("Refresh completed successfully"),
        Err(e) => tracing::error!("Refresh failed: {:?}", e),
    }
});
```

No complex queue system needed yet. If server crashes during refresh, just trigger it again manually.

## Project Structure

```
country_api/
├── src/
│   ├── db/
│   │   ├── mod.rs
│   │   ├── pool.rs
│   │   └── repositories.rs
│   ├── routes/
│   │   ├── mod.rs
│   │   └── countries.rs
│   ├── models/
│   │   ├── mod.rs
│   │   ├── country.rs
│   │   ├── requests.rs
│   │   ├── responses.rs
│   │   └── state.rs
│   ├── utils/
│   │   ├── mod.rs
│   │   ├── config.rs
│   │   ├── clients.rs  
│   │   ├── tasks.rs  
│   │   ├── countries.rs  
│   │   └── image.rs     
│   ├── api.rs            
│   ├── lib.rs
│   └── main.rs
├── cache/                
│   └── summary.png     
├── migrations/        
├── Cargo.toml
├── DESIGN.md
├── README.md
└── .env
```

## Environment Variables

```env
DATABASE_URL=mysql://user:password@localhost:3306/country_api
DATABASE_MAX_CONNECTIONS=10
DATABASE_CONNECTION_TIMEOUT=30
SERVER_HOST=0.0.0.0
SERVER_PORT=8000
LOG_LEVEL=info
REST_COUNTRIES_API=https://restcountries.com/v2/all?fields=name,capital,region,population,flag,currencies
EXCHANGE_RATES_API=https://open.er-api.com/v6/latest/USD
```