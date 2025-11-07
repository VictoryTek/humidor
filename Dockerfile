# syntax=docker/dockerfile:1
FROM rust:1.82-slim AS builder

WORKDIR /app

# Cache busting argument
ARG CACHEBUST=1

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create dummy project for dependency caching
# We need both binary and library targets to match Cargo.toml
RUN cargo init --name humidor .
COPY Cargo.toml Cargo.lock ./

# Create dummy module directories and files to match the real project structure
# This allows proper dependency caching for both binary and library targets
RUN mkdir -p src/handlers src/middleware src/models src/routes src/services && \
    echo "pub struct AppError;" > src/errors.rs && \
    echo "pub mod auth;" > src/handlers/mod.rs && \
    echo "pub fn setup() {}" > src/handlers/auth.rs && \
    echo "pub mod auth; pub mod rate_limiter; pub use auth::AuthContext; pub use rate_limiter::RateLimiter;" > src/middleware/mod.rs && \
    echo "pub struct AuthContext;" > src/middleware/auth.rs && \
    echo "pub struct RateLimiter;" > src/middleware/rate_limiter.rs && \
    echo "pub mod cigar;" > src/models/mod.rs && \
    echo "pub struct Cigar;" > src/models/cigar.rs && \
    echo "pub mod auth; pub mod helpers;" > src/routes/mod.rs && \
    echo "pub fn create_auth_routes() {}" > src/routes/auth.rs && \
    echo "pub fn with_db() {}" > src/routes/helpers.rs && \
    echo "pub mod mod_stub;" > src/services/mod.rs && \
    echo "pub struct Service;" > src/services/mod_stub.rs && \
    echo "pub fn validate_email(_: &str) -> Result<(), ()> { Ok(()) }" > src/validation.rs

# Create library entry point that matches our actual lib.rs structure
RUN echo "pub mod errors;" > src/lib.rs && \
    echo "pub mod handlers;" >> src/lib.rs && \
    echo "pub mod middleware;" >> src/lib.rs && \
    echo "pub mod models;" >> src/lib.rs && \
    echo "pub mod routes;" >> src/lib.rs && \
    echo "pub mod services;" >> src/lib.rs && \
    echo "pub mod validation;" >> src/lib.rs && \
    echo "pub use errors::AppError;" >> src/lib.rs && \
    echo "pub use middleware::{RateLimiter, AuthContext};" >> src/lib.rs && \
    echo "pub use validation::validate_email;" >> src/lib.rs && \
    echo "use deadpool_postgres::Pool;" >> src/lib.rs && \
    echo "pub type DbPool = Pool;" >> src/lib.rs

# Build dependencies for both binary and library - this caches all external dependencies
RUN cargo build --release

# Build actual application
# Force cache bust when source changes
RUN echo "Cache bust: $CACHEBUST"
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

# Create directories for runtime data with correct permissions
RUN mkdir -p backups uploads && chown -R humidor:humidor backups uploads

USER humidor

EXPOSE 9898

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:9898/health || exit 1

CMD ["./humidor"]