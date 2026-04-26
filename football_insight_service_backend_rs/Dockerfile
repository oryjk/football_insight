FROM docker.m.daocloud.io/library/rust:1-bookworm AS builder

WORKDIR /app

RUN sed -i 's/deb.debian.org/mirrors.aliyun.com/g' /etc/apt/sources.list.d/debian.sources \
    && apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN mkdir -p /app/.cargo \
    && printf '%s\n' \
        '[source.crates-io]' \
        'replace-with = "rsproxy-sparse"' \
        '[source.rsproxy-sparse]' \
        'registry = "sparse+https://rsproxy.cn/index/"' \
        '[net]' \
        'git-fetch-with-cli = true' \
        > /app/.cargo/config.toml

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
COPY .env.example ./.env.example

RUN cargo build --release --locked

FROM docker.m.daocloud.io/library/debian:bookworm-slim

WORKDIR /app

RUN sed -i 's/deb.debian.org/mirrors.aliyun.com/g' /etc/apt/sources.list.d/debian.sources \
    && apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 10001 --shell /usr/sbin/nologin appuser \
    && mkdir -p /app/logs \
    && chown -R appuser:appuser /app

COPY --from=builder /app/target/release/football_insight_service_backend_rs /usr/local/bin/football_insight_service_backend_rs
COPY migrations ./migrations

RUN chmod +x /usr/local/bin/football_insight_service_backend_rs \
    && chown -R appuser:appuser /app

USER appuser

ENV PORT=8092
ENV RUST_LOG=info

EXPOSE 8092

HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
  CMD curl -fsS http://127.0.0.1:8092/api/health >/dev/null || exit 1

CMD ["/usr/local/bin/football_insight_service_backend_rs"]
