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

step "no secret files tracked in git (.env, secrets/*.txt)"
# .env and secrets/*.txt hold real credentials and are gitignored; only the *.example templates may be
# committed. If this fails: `git rm --cached <file>` (keeps your local copy), and rotate any secret
# that was already committed, since it remains in history.
tracked=$(git ls-files -- '.env' '.env.*' 'secrets/*.txt' | grep -vE '(^|/)\.env\.example$' || true)
if [[ -n "$tracked" ]]; then
    printf 'ERROR: secret files are tracked by git:\n%s\n' "$tracked" >&2
    exit 1
fi

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
