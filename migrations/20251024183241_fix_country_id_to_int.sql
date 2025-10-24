-- Add migration script here
DROP TABLE IF EXISTS countries;

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
    INDEX idx_estimated_gdp (estimated_gdp),
    INDEX idx_name (name)
);