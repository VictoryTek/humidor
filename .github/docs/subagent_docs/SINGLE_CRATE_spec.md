# SINGLE_CRATE — Spec (MASTER_PLAN #9)

## Current state
`src/lib.rs` declares the whole module tree `pub` (used by `tests/`). `src/main.rs` ALSO declares
`mod db_config; mod errors; mod handlers; mod middleware; mod models; mod routes; mod services; mod validation;`
so the bin recompiles the entire application privately. Consequences: ~2x compile time for the app code, every
unit test compiled and run twice (`cargo test --lib` runs the 18 tests for `src/lib.rs` *and* `src/main.rs`), and
two independent `DbPool` aliases (`lib.rs` and `main.rs:32`) — distinct crates, so bin/lib types could drift.

## Change
- `main.rs`: drop the eight `mod` declarations; `use humidor::{db_config, middleware, routes, services};`
  (the only modules `main.rs` references directly) and `use humidor::DbPool;` in place of its own alias.
- Keep in the bin: `embed_migrations!`, `#![recursion_limit = "256"]` (route composition happens here), and
  everything else in `main.rs` unchanged.
- No public API or behaviour change; no Cargo.toml change (the `[lib]`/`[[bin]]` targets already exist).

## Verification plan
fmt/clippy/build/`scripts/preflight.sh`; unit tests should now run once (main.rs unittests = 0);
run the real binary against a throwaway Postgres and hit `/health`, `/api/v1/setup/status`, `/`.

## Out of scope
The `#[allow(dead_code)]` attributes that existed because the bin didn't use library items (#42) —
left for that item. `main.rs` remains large (#16/#17 touch it).

## Risks
Items previously private to the bin's copy of the tree now resolve to the lib's `pub` items — identical source,
so no semantic change; dead-code lints are stricter for private items only.
