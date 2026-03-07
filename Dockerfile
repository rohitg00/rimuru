FROM rust:1.83-slim-bookworm AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    nodejs npm \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY ui ./ui

RUN cd ui && npm ci && npm run build
RUN cargo build --release -p rimuru-core -p rimuru-cli

FROM debian:bookworm-slim

LABEL org.opencontainers.image.title="Rimuru"
LABEL org.opencontainers.image.description="AI agent cost monitor powered by iii-engine"
LABEL org.opencontainers.image.authors="Rohit Ghumare <ghumare64@gmail.com>"
LABEL org.opencontainers.image.source="https://github.com/rohitg00/rimuru"
LABEL org.opencontainers.image.licenses="MIT"

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1000 rimuru

COPY --from=builder /app/target/release/rimuru-worker /usr/local/bin/rimuru-worker
COPY --from=builder /app/target/release/rimuru /usr/local/bin/rimuru

RUN chown rimuru:rimuru /usr/local/bin/rimuru-worker /usr/local/bin/rimuru

USER rimuru
WORKDIR /home/rimuru

ENV RIMURU_ENGINE_URL="ws://127.0.0.1:49134"
ENV RIMURU_API_PORT="3100"
ENV RUST_LOG="rimuru=info"

EXPOSE 3100

ENTRYPOINT ["rimuru-worker"]
