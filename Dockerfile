# Rimuru - AI Agent Orchestration and Cost Tracking Platform
# Multi-stage build for minimal image size

# Build stage
FROM rust:1.83-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY rimuru-core ./rimuru-core
COPY rimuru-cli ./rimuru-cli
COPY rimuru-tui ./rimuru-tui
COPY rimuru-plugin-sdk ./rimuru-plugin-sdk

# Build release binaries
RUN cargo build --release -p rimuru-cli -p rimuru-tui

# Runtime stage
FROM debian:bookworm-slim

LABEL org.opencontainers.image.title="Rimuru"
LABEL org.opencontainers.image.description="A unified AI agent orchestration and cost tracking platform"
LABEL org.opencontainers.image.authors="Rohit Ghumare <ghumare64@gmail.com>"
LABEL org.opencontainers.image.source="https://github.com/rohitg00/rimuru"
LABEL org.opencontainers.image.licenses="Apache-2.0"

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 rimuru

# Copy binaries from builder
COPY --from=builder /app/target/release/rimuru /usr/local/bin/rimuru
COPY --from=builder /app/target/release/rimuru-tui /usr/local/bin/rimuru-tui

# Set ownership
RUN chown rimuru:rimuru /usr/local/bin/rimuru /usr/local/bin/rimuru-tui

# Switch to non-root user
USER rimuru
WORKDIR /home/rimuru

# Default environment variables
ENV DATABASE_URL=""
ENV RIMURU_LOG_LEVEL="info"

# Default command
ENTRYPOINT ["rimuru"]
CMD ["--help"]
