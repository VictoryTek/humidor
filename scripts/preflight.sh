#!/usr/bin/env bash
# Local preflight: mirrors the `lint`, `security` and `test` jobs in .github/workflows/ci.yml.
# Exit code 0 means CI should pass. Skips the Docker/Trivy and coverage jobs (not local-safe).
#
# Integration tests need a running Postgres instance and are only run when TEST_DATABASE_URL
# is set (also export DATABASE_URL and JWT_SECRET, as in ci.yml). Never run
# `docker compose down -v` or remove the postgres_data / humidor_data volumes.
set -euo pipefail

cd "$(dirname "$0")/.."

step() { printf '\n==> %s\n' "$*"; }

step "cargo fmt -- --check"
cargo fmt -- --check

step "cargo clippy --all-targets --all-features -- -D warnings"
cargo clippy --all-targets --all-features -- -D warnings

step "cargo build --verbose"
cargo build --verbose

step "cargo test --lib --verbose"
cargo test --lib --verbose

if [[ -n "${TEST_DATABASE_URL:-}" ]]; then
    step "cargo test --tests --verbose"
    cargo test --tests --verbose
else
    step "Skipping integration tests (TEST_DATABASE_URL not set)"
fi

step "cargo audit"
cargo audit

printf '\nPreflight passed.\n'
