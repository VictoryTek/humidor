# AUDIT_FIX — Spec

## Current state
CI `cargo audit` failed with 10 vulnerabilities. A re-run against the current RustSec DB (1251 advisories)
also reports 4 newer ones. All are locked versions in `Cargo.lock`; `Cargo.toml` needs no change.

## Problem
| Crate | Locked | Advisory | Fixed in |
|---|---|---|---|
| bytes | 1.10.1 | RUSTSEC-2026-0007 | 1.11.1 |
| lettre | 0.11.19 | RUSTSEC-2026-0141 | 0.11.22 |
| postgres-protocol | 0.6.9 | RUSTSEC-2026-0179, -0180 | 0.6.12 |
| tokio-postgres | 0.7.15 | RUSTSEC-2026-0178 | 0.7.18 |
| rustls-webpki | 0.103.8 | RUSTSEC-2026-0104/-0049/-0099/-0098 | 0.103.13 |
| time | 0.3.44 | RUSTSEC-2026-0009 | 0.3.47 |
| crossbeam-epoch | 0.9.18 | RUSTSEC-2026-0204 | 0.9.20 |
| rustls | 0.23.35 | RUSTSEC-2026-0285 | 0.23.45 |
| h2 | 0.4.12 | RUSTSEC-2026-0258 | 0.4.16 |
| h2 | 0.3.27 | RUSTSEC-2026-0258 | **no 0.3.x patch** (via pinned warp 0.3.7 -> hyper 0.14) |

Warnings (non-fatal) also fixable in-semver: anyhow 1.0.100 -> 1.0.103, rand 0.8.5 -> 0.8.6.

## Solution
1. Lock-file only, minimal bumps with `cargo update <crate> --precise <ver>` (semver-compatible; no manifest edits).
   `postgres-protocol` 0.6.12 forces hmac 0.13 / md-5 0.11 / sha2 0.11 / rand 0.10 transitively.
2. h2 0.3.27: user approved ignoring RUSTSEC-2026-0258 (Low, DoS; needs warp/hyper 1.x migration per
   `docs/DEPENDENCY_UPDATES_PLAN.md`). Do NOT bump warp.
3. Discovered: `cargo audit` does not read a root `audit.toml` (only `.cargo/audit.toml` / `~/.cargo/audit.toml`),
   and the file contained an invalid key (`unmaintained`). The existing fxhash ignore was therefore never applied.
   Move it to `.cargo/audit.toml`, drop the invalid key, add the h2 ignore.
4. Phase 6 gap: create `scripts/preflight.sh` mirroring CI lint/test/audit jobs.

## Dependencies / Context7
No new dependencies and no manifest changes; Context7 not required (lock-file refresh only).

## Validation commands (all safe)
`cargo fmt -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --verbose`,
`cargo test --lib --verbose`, `cargo test --tests --verbose` (throwaway `postgres:17` container, no volumes —
NOT `docker compose`, to avoid touching the `postgres_data` volume), `cargo audit`.

## Risks
- Driver stack bump (tokio-postgres/postgres-protocol, new SCRAM crypto crates): mitigated by full integration suite.
- Ignoring RUSTSEC-2026-0258 leaves a Low-severity DoS in the HTTP/2 path of warp's hyper 0.14: documented in
  `.cargo/audit.toml` with removal condition.
- Local env: no `lld`/OpenSSL headers on this NixOS host; used `RUSTFLAGS="-C debuginfo=0"` (overrides
  `.cargo/config.toml` rustflags) inside a nix-shell. Environment-only workaround, no repo change.
