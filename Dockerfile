FROM rust:1.90-slim AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml ./

COPY .sqlx ./.sqlx

COPY src/ ./src/

COPY assets ./assets

ENV SQLX_OFFLINE=true

RUN cargo build --release --bin currency_exchange_api

FROM gcr.io/distroless/cc-debian12

COPY --from=builder /app/target/release/currency_exchange_api /app

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

EXPOSE 8000

ENV RUST_LOG=info

ENTRYPOINT ["/app"]