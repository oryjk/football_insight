FROM rust:1.93-slim-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
COPY .env.example ./.env.example

RUN cargo build --release --locked

FROM debian:bookworm-slim

WORKDIR /app

COPY --from=builder /app/target/release/football_insight_service_backend_rs /usr/local/bin/football_insight_service_backend_rs
COPY migrations ./migrations

EXPOSE 8092

CMD ["football_insight_service_backend_rs"]
