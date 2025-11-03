# syntax=docker/dockerfile:1
FROM rust:1.82-slim AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create dummy project for dependency caching
RUN cargo init --name humidor .
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

# Build actual application
COPY src ./src
COPY static ./static
COPY migrations ./migrations
RUN touch src/main.rs && cargo build --release && strip target/release/humidor

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd -r humidor && useradd -r -g humidor humidor

WORKDIR /app

COPY --from=builder --chown=humidor:humidor /app/target/release/humidor ./
COPY --from=builder --chown=humidor:humidor /app/static ./static

USER humidor

EXPOSE 9898

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:9898/health || exit 1

CMD ["./humidor"]