# Country Currency & Exchange API

A RESTful API service that fetches country and currency data from external APIs, stores them, and provides CRUD operations with filtering and sorting capabilities. The API calculates estimated GDP for each country and generates visual summaries.

## Features

- Fetch country data from external countries API
- Fetch real-time exchange rates from open exchange rates API
- Calculate estimated GDP with randomized multipliers
- CRUD operations for country records
- Query filtering (by region, currency) and sorting (by GDP)
- Background refresh task with external API validation
- Automatic image generation with country statistics
- OpenAPI/Swagger UI documentation

## Dependencies

This project uses the following Rust crates:

- **axum** - Web framework
- **tokio** - Async runtime
- **sqlx** - Database (MySQL) support with compile-time query checking
- **serde** and **serde_json** - Serialization/deserialization
- **reqwest** - HTTP client for external APIs
- **chrono** - Date and time handling
- **bigdecimal** - Precise decimal arithmetic for financial data
- **image**, **imageproc**, and **ab_glyph** - Image generation
- **rand** - Random number generation
- **utoipa** and **utoipa-swagger-ui** - OpenAPI/Swagger UI integration
- **dotenvy** and **envy** - Environment variable loading
- **anyhow** - Error handling
- **tracing** and **tracing-subscriber** - Logging implementation

## Prerequisites

- Rust / Cargo
- MySQL
- SQLx CLI

## Installation

### Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Install SQLx CLI
```bash
cargo install sqlx-cli --no-default-features --features mysql,rustls
```

## Setup Instructions

### 1. Clone the repository
```bash
git clone <repository-url>
cd currency_exchange_api
```

### 2. Configure environment variables

Create a `.env` file in the project root:

```env
# Database Configuration
DATABASE_URL=mysql://root:password@localhost:3306/country_api
DATABASE_MAX_CONNECTIONS=10
DATABASE_CONNECTION_TIMEOUT=30

# Server Configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=8000

# External APIs
REST_COUNTRIES_API=https://restcountries.com/v2/all?fields=name,capital,region,population,flag,currencies
EXCHANGE_RATES_API=https://open.er-api.com/v6/latest/USD

# Logging
LOG_LEVEL=info
```

**Environment Variables:**
- `DATABASE_URL`: MySQL connection string
- `DATABASE_MAX_CONNECTIONS`: Connection pool size (default: 10)
- `DATABASE_CONNECTION_TIMEOUT`: Timeout in seconds (default: 30)
- `SERVER_HOST`: HTTP server host (default: 0.0.0.0)
- `SERVER_PORT`: HTTP server port (default: 8000)
- `REST_COUNTRIES_API`: Countries data source URL
- `EXCHANGE_RATES_API`: Exchange rates data source URL
- `LOG_LEVEL`: Logging level (info/debug/warn/error)

### 3. Setup Database

```bash
# Create database (if not using Docker)
sqlx database create

# Run migrations
sqlx migrate run
```

This creates the `countries` table with proper indexes.

### 4. Download Font for Image Generation

```bash
# Create assets directory
mkdir -p assets

# Download DejaVu Sans font
curl -L https://github.com/dejavu-fonts/dejavu-fonts/releases/download/version_2_37/dejavu-fonts-ttf-2.37.tar.bz2 -o dejavu.tar.bz2

# Extract and copy
tar -xjf dejavu.tar.bz2
cp dejavu-fonts-ttf-2.37/ttf/DejaVuSans.ttf assets/

# Cleanup
rm -rf dejavu.tar.bz2 dejavu-fonts-ttf-2.37
```

### 5. Build the project
```bash
cargo build --release
```

## Running Locally

### Development mode
```bash
cargo run
```

### Production mode
```bash
cargo run --release
```

The server will start on the configured host and port (default: http://0.0.0.0:8000)

**Expected output:**
```
INFO currency_exchange_api: Configuration loaded successfully
INFO currency_exchange_api: Database connection pool created
INFO currency_exchange_api: Server running on 0.0.0.0:8000
```

## API Documentation

Interactive Swagger UI documentation is available at:
```
http://localhost:8000/swagger-ui
```

OpenAPI JSON specification:
```
http://localhost:8000/api-docs/openapi.json
```

## API Endpoints

### 1. Refresh Country Data

Fetches data from external APIs, validates availability, and caches in database.

```
POST /countries/refresh
```

**Response (200 OK):**
```json
{
  "message": "Refresh started in background"
}
```

**Response (503 Service Unavailable):**
```json
{
  "error": "External data source unavailable",
  "details": "Could not fetch data from restcountries API"
}
```

**Process:**
1. Validates both external APIs are accessible
2. Returns 503 if either API is down
3. Spawns background task to fetch and process data
4. Calculates estimated GDP for each country
5. Generates summary image

---

### 2. Get All Countries

Retrieve countries with optional filtering and sorting.

```
GET /countries?region={region}&currency={currency}&sort={sort}
```

**Query Parameters:**
- `region` (optional): Filter by region (e.g., "Africa", "Europe", "Asia")
- `currency` (optional): Filter by currency code (e.g., "NGN", "USD", "GBP")
- `sort` (optional): Sort order - "gdp_asc" or "gdp_desc" (default: "gdp_desc")

**Response (200 OK):**
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
    "estimated_gdp": 25767448125.20,
    "flag_url": "https://flagcdn.com/ng.svg",
    "last_refreshed_at": "2025-10-24T10:30:45.123Z"
  }
]
```

---

### 3. Get Single Country

Retrieve a specific country by name (case-insensitive).

```
GET /countries/{name}
```

**Path Parameters:**
- `name`: Country name (e.g., "Nigeria", "ghana")

**Response (200 OK):**
```json
{
  "id": 1,
  "name": "Nigeria",
  "capital": "Abuja",
  "region": "Africa",
  "population": 206139589,
  "currency_code": "NGN",
  "exchange_rate": 1600.23,
  "estimated_gdp": 25767448125.20,
  "flag_url": "https://flagcdn.com/ng.svg",
  "last_refreshed_at": "2025-10-24T10:30:45.123Z"
}
```

**Response (404 Not Found):**
```json
{
  "error": "Country not found"
}
```

---

### 4. Delete Country

Remove a country from the cache.

```
DELETE /countries/{name}
```

**Path Parameters:**
- `name`: Country name to delete

**Response:**
- `204 No Content` - Country deleted successfully
- `404 Not Found` - Country doesn't exist

---

### 5. Get Status

Shows total countries and last refresh timestamp.

```
GET /status
```

**Response (200 OK):**
```json
{
  "total_countries": 250,
  "last_refreshed_at": "2025-10-24T10:30:45.123Z"
}
```

**Response (empty database):**
```json
{
  "total_countries": 0,
  "last_refreshed_at": null
}
```

---

### 6. Get Summary Image

Serves a generated PNG image with country statistics.

```
GET /countries/image
```

**Response:**
- `200 OK` - PNG image with:
  - Total number of countries
  - Top 5 countries by estimated GDP
  - Last refresh timestamp
- `404 Not Found` - Image hasn't been generated yet

**Content-Type:** `image/png`

---

## Example Usage

```bash
# Refresh country data
curl -X POST http://localhost:8000/countries/refresh

# Get countries by region and currency
curl "http://localhost:8000/countries?region=Africa&currency=NGN"

# Sort by GDP ascending
curl "http://localhost:8000/countries?region=Europe&currency=EUR&sort=asc"

# Get a specific country
curl http://localhost:8000/countries/Nigeria

# Delete a country
curl -X DELETE http://localhost:8000/countries/Nigeria

# Check status
curl http://localhost:8000/status

# View summary image in browser
open http://localhost:8000/countries/image

# Download summary image
curl http://localhost:8000/countries/image --output summary.png
```

## Business Logic

### Currency Handling Rules

The API follows these rules when processing country currencies:

**Rule 1: Empty currencies array**
```
currency_code = NULL
exchange_rate = NULL
estimated_gdp = 0
```

**Rule 2: Multiple currencies**
- Takes only the first currency code from the array
- Ignores subsequent currencies

**Rule 3: Currency not found in exchange rates**
```
currency_code = {code}
exchange_rate = NULL
estimated_gdp = NULL
```

### GDP Calculation Formula

```
estimated_gdp = (population × random(1000-2000)) ÷ exchange_rate
```

**Key Points:**
- Random multiplier regenerated on **every refresh** for **every country**
- Returns `NULL` if exchange rate is 0 or missing
- Uses BigDecimal for precise financial calculations

### Update Logic

The API uses MySQL's `ON DUPLICATE KEY UPDATE`:
- Matches existing countries by `name` (case-insensitive)
- If country exists: Updates all fields including recalculating GDP
- If country doesn't exist: Inserts new record
- Generates new random multiplier on each refresh

## Error Responses

All errors return JSON with consistent format:

**400 Bad Request:**
```json
{
  "error": "Validation failed",
  "details": {
    "region": "is required"
  }
}
```

**404 Not Found:**
```json
{
  "error": "Country not found"
}
```

**500 Internal Server Error:**
```json
{
  "error": "Internal server error"
}
```

**503 Service Unavailable:**
```json
{
  "error": "External data source unavailable",
  "details": {
    "details": "Could not fetch data from restcountries API"
  }
}
```

## Project Structure

```
currency_exchange_api/
├── assets/
│   └── DejaVuSans.ttf        # Font for image generation
├── cache/
│   └── summary.png           # Generated summary image
├── migrations/
│   └── *.sql                 # Database migrations
├── src/
│   ├── db/
│   │   ├── pool.rs           # Connection pooling
│   │   └── repositories.rs   # CRUD operations
│   ├── models/
│   │   ├── country.rs        # Data models
│   │   ├── requests.rs       # Query filters
│   │   ├── responses.rs      # API responses
│   │   └── state.rs          # App state
│   ├── routes/
│   │   └── countries.rs      # Request handlers
│   ├── utils/
│   │   ├── config.rs         # Environment config
│   │   ├── countries.rs      # Countries API client
│   │   ├── exchange.rs       # Exchange API client
│   │   ├── image.rs          # Image generation
│   │   └── task.rs           # Refresh task logic
│   ├── api.rs                # Router setup
│   ├── lib.rs                # Module exports
│   └── main.rs               # App entry point
├── .env
├── .env.example
├── .gitignore
├── .dockerignore
├── Dockerfile
├── Cargo.toml
├── DESIGN.md
├── LICENSE
└── README.md
```

## Development

### Running in Development Mode
```bash
cargo run
```

### Running Tests
```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture
```

### Code Quality
```bash
# Check for compilation errors
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy
```

### Viewing Logs

Adjust log level in `.env`:
```env
LOG_LEVEL=debug  # Options: trace, debug, info, warn, error
```

Logs are printed to stdout and include:
- Configuration loading
- Database connection status
- API call results
- Refresh progress
- Error details

## Deployment

### Building for Production
```bash
cargo build --release
```

Binary location: `target/release/currency_exchange_api`

### Environment Variables for Production

Ensure these are set in your production environment:
- `DATABASE_URL` - Production MySQL connection string
- `SERVER_HOST` - Usually `0.0.0.0`
- `SERVER_PORT` - Production port
- `REST_COUNTRIES_API` - Countries API URL
- `EXCHANGE_RATES_API` - Exchange rates API URL
- `LOG_LEVEL` - Set to `info` or `warn`

### Deployment Platforms

**Docker:**
```bash
docker build -t currency-exchange-api .
docker run -p 8000:8000 --env-file .env currency-exchange-api
```