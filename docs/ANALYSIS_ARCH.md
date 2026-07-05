# Architecture & Structure Analysis — Humidor

Scope: architecture and structure only (no feature review). Analyzed at v1.5.3, commit `a4dd298`.
Every finding lists priority, file(s) and line numbers, and why it is a problem.

---

## 1. Architectural Anti-Patterns / Design Problems

### 1.1 Dual-crate layout where the binary never uses the library — everything compiles twice
**Priority: HIGH**
**Files:** `src/main.rs:3-9`, `src/lib.rs:4-10`, `src/main.rs:31`, `src/lib.rs:19`, `Cargo.toml:6-12`

`Cargo.toml` defines both a `[[bin]]` and a `[lib]`. The library (`lib.rs`) declares `pub mod errors; pub mod handlers; ...` for integration-test access, but `main.rs` does **not** depend on the library — it re-declares the identical module tree with private `mod errors; mod handlers; ...`. Consequences:

- The entire ~12,000-line module tree is compiled twice on every build (once for the lib target, once for the bin target), doubling compile time for the largest part of the crate.
- There are two independent `DbPool` type aliases (`main.rs:31` and `lib.rs:19`). Types in the bin build and the lib build are distinct to the compiler; nothing forces them to stay in sync.
- Integration tests exercise the *library* build, while production runs the *binary* build. They are compiled from the same source today, but conditional compilation, feature flags, or `#[cfg(test)]` drift would silently open a gap between what is tested and what ships.

The conventional fix is a thin `main.rs` (`use humidor::...;`) with all modules owned by the library only.

### 1.2 JWT secret resolution is duplicated in two places and the copies have diverged
**Priority: HIGH**
**Files:** `src/main.rs:35-92` (`read_secret`), `src/main.rs:95-135` (`get_or_generate_jwt_secret`), `src/handlers/auth.rs:50-74` (`jwt_secret`)

There are two independent implementations of "find the JWT secret":

- Startup (`main.rs read_secret`) checks, in order: `JWT_SECRET_FILE` custom path → `/run/secrets/jwt_secret` → **`/app/data/jwt_secret` (persisted auto-generated secret)** → `JWT_SECRET` env var. If none found, it generates a random secret and persists it to `/app/data/jwt_secret` (`main.rs:110-134`).
- Runtime token signing/verification (`handlers/auth.rs jwt_secret`) checks: `JWT_SECRET_FILE` → `/run/secrets/jwt_secret` → `JWT_SECRET` env var — **it never checks `/app/data/jwt_secret`**, and falls back to the sentinel string `"INVALID_SECRET_NOT_CONFIGURED"` (`auth.rs:66-73`).

Failure scenario: deploy with no `JWT_SECRET` configured (the exact case the auto-generate path exists for). Startup validation passes — it generates and persists a secret. Every subsequent login then signs tokens with `"INVALID_SECRET_NOT_CONFIGURED"` instead of the persisted secret. The persisted secret is never used for anything, and the "graceful" fallback means all deployments without an explicit env var are silently signing tokens with a hardcoded, publicly-known string. This is a direct consequence of duplicating the resolution logic instead of resolving once at startup and sharing the value.

Secondary problem, same file: `jwt_secret()` is called on **every** token generate/verify (`auth.rs:550`, `auth.rs:557`), performing up to two synchronous `fs::read_to_string` calls per request on the async runtime, when the secret is immutable for the process lifetime and should be read once into a `OnceLock`/`Lazy`.

### 1.3 Startup SMTP validation checks environment variables the email service never reads
**Priority: HIGH**
**Files:** `src/main.rs:182-196` (`validate_smtp_config`), `src/services/email.rs:25-27`, `docker-compose.dev.yml:13-17`

`validate_smtp_config()` requires `SMTP_USERNAME` and `SMTP_FROM`. The actual email service reads `SMTP_USER` (`email.rs:25`) and `SMTP_FROM_EMAIL` (`email.rs:27`). The dev compose file sets `SMTP_USER` / `SMTP_FROM_EMAIL`, matching the service, not the validator.

Failure scenario: set `SMTP_ENABLED=true` with a correctly configured mailer (`SMTP_USER`, `SMTP_FROM_EMAIL`) — startup aborts claiming `SMTP_USERNAME, SMTP_FROM` are missing. Conversely, set `SMTP_USERNAME`/`SMTP_FROM` to satisfy the validator and email construction fails at runtime. A fail-fast validator that validates the wrong keys is worse than no validator: it certifies a broken configuration as good.

### 1.4 Layering inversion: middleware depends on handlers; handlers depend on sibling handlers for authorization
**Priority: HIGH**
**Files:** `src/middleware/auth.rs:3`, `src/handlers/auth.rs:525-562`, `src/handlers/humidors.rs:2`, `src/handlers/cigars.rs:659`

- `middleware/auth.rs` imports `verify_token` from `crate::handlers::auth`. JWT encode/decode utilities (`generate_token`, `verify_token`, `jwt_secret`, `Claims`) live inside a 1,224-line HTTP handler file, and the middleware layer — which should sit *below* handlers — reaches *up* into the handler layer for its core primitive. Dependency direction is inverted.
- Authorization predicates (`can_view_humidor`, `can_manage_humidor`) live in `handlers/humidor_shares.rs` and are imported by `handlers/humidors.rs:2` and `handlers/cigars.rs:659`. Cross-cutting authorization logic has no owning module; it lives in whichever handler file happened to need it first. Anyone adding a new resource must know to import permission checks from an unrelated handler.

A `src/auth/` or `src/security/` module (token utilities + permission predicates) would give both a home and restore the middleware → core direction.

### 1.5 PORT is parsed twice with two different defaults; the logged port can differ from the bound port
**Priority: MEDIUM**
**Files:** `src/main.rs:339-342`, `src/main.rs:655-658`

`main()` reads `PORT` at line 339 with default **9898** and logs "Configuring server" with it (line 344-348). Then at line 655 it re-reads `PORT` with default **3000**, shadowing the first binding, and binds the server with that value (line 667). With no `PORT` set, the log says 9898 (matching the Dockerfile's `EXPOSE 9898` and healthcheck) while the server actually listens on 3000 — the container healthcheck (`Dockerfile:94-95`, port 9898) would fail. It works today only because compose always sets `PORT`. Dead first read, conflicting defaults, and a misleading log line.

### 1.6 Default CORS mode combines `allow_any_origin()` with `allow_credentials(true)`
**Priority: MEDIUM**
**Files:** `src/main.rs:552-557` (permissive), `src/main.rs:623-627` (unknown-mode fallback)

Per the Fetch/CORS spec, `Access-Control-Allow-Origin: *` may not be combined with `Access-Control-Allow-Credentials: true`; browsers reject credentialed responses in that combination. So either the wildcard is downgraded and credentials silently don't work cross-origin, or the header pair is emitted invalidly — in both cases the configuration expresses an intent the platform cannot honor. Since this is the *default* mode, every default deployment carries a contradictory CORS policy. If credentials are needed, the origin must be echoed back explicitly; if not, `allow_credentials(true)` should be dropped.

### 1.7 Per-request HTML string manipulation to inject a setup-check script
**Priority: MEDIUM**
**Files:** `src/main.rs:672-736` (`serve_index`), `src/main.rs:782-793` (`serve_shared_humidor`)

`serve_index` reads `static/index.html` from disk on every request and injects a `<script>` block via `content.replace("</body>", ...)`. The same 20-line setup-check script is duplicated a second time in the hardcoded fallback HTML (`main.rs:714-728`). Problems: file I/O + string scan per request for what is static content; the script exists in two copies that can drift; and the injected script duplicates logic that belongs in `static/app.js` (which already handles auth/setup state). `serve_shared_humidor` reads the same file again through a different function. This is templating done with `String::replace` in a system that otherwise has no server-side templating — the setup check should simply live in the static JS.

### 1.8 A 500-line scraper implementation lives directly in `services/mod.rs`
**Priority: MEDIUM**
**Files:** `src/services/mod.rs:11-500`

`services/mod.rs` declares `pub mod backup; pub mod email;` — then inlines the entire `CigarScraper` (struct, five site-specific scrapers, extraction helpers, public API) in the module file itself. Its siblings each get a dedicated file (`backup.rs`, `email.rs`). A `mod.rs` should route to submodules, not host the largest service in the codebase. This also makes the scraper invisible to anyone scanning the directory listing for a "scraper" module.

### 1.9 Backup endpoints return HTTP 200 for failures and leak internal error strings
**Priority: MEDIUM**
**Files:** `src/handlers/backups.rs:47-53`, `src/handlers/backups.rs:105-111`, `src/handlers/backups.rs:128-133`, `src/handlers/backups.rs:273-276`

`create_backup_handler`, `delete_backup_handler`, `restore_backup_handler`, and `setup_restore_backup` all respond to failures with `200 OK` and a body like `{"message": "Error creating backup: <raw error>"}`. Two problems: (a) clients (and the frontend) cannot distinguish success from failure by status code — they must string-match "Error" in a message field; (b) the raw `e` from filesystem/zip/database operations is serialized to the client, which `errors.rs:83-97` explicitly forbids for every other endpoint ("Never expose database errors externally"). The error architecture that exists is simply not used here.

### 1.10 79 repetitions of the `api/v1` path prefix
**Priority: LOW**
**Files:** all of `src/routes/*.rs` (e.g. `src/routes/organizers.rs:12-13`, `src/routes/users.rs:10-11`, `src/routes/cigars.rs:10-11`)

Every individual route re-declares `warp::path("api").and(warp::path("v1"))` — 79 occurrences. A single `path!("api" / "v1" / ..)` prefix filter (or applying the prefix once in `main.rs` when combining `api`) would remove ~160 lines and make a future `/api/v2` possible without touching 79 sites.

### 1.11 `/metrics` endpoint is unauthenticated
**Priority: LOW**
**Files:** `src/main.rs:496-499`

The Prometheus endpoint is registered with no auth filter and is reachable by anyone who can reach the app (the app is designed for self-hosting on open LANs, with CORS permissive by default). Request paths, timings, and DB pool stats are exposed. Worth an explicit decision: bind-scoped scraping, a token check, or documented acceptance.

---

## 2. Structural Inconsistencies

### 2.1 `.env` containing `JWT_SECRET` is committed to git despite being gitignored
**Priority: HIGH**
**Files:** `.env` (tracked — confirmed via `git ls-files`), `.gitignore:12`

`.gitignore` lists `.env`, but the file was committed before the ignore rule took effect (gitignore never untracks). The tracked file contains `JWT_SECRET=dev_secret_change_me_in_production_please` and DB credentials. Even as "dev-only" values, a tracked `.env` means: every local secret edit shows up as a pending change inviting accidental commits of real values, and the repo publicly ships a default JWT secret that any unconfigured deployment might actually be running with (see finding 1.2 for why the fallback path matters). Should be `git rm --cached .env` with `.env.example` (which already exists) as the only tracked variant.

### 2.2 `src/handlers/images.rs` is not declared in any module — dead, uncompiled file
**Priority: HIGH**
**Files:** `src/handlers/images.rs` (159 lines), `src/handlers/mod.rs:1-14`

`handlers/mod.rs` declares 14 submodules; `images` is not among them, and no `upload_image` reference exists anywhere else in `src/`. The file — which contains a full multipart image-upload handler with an `UPLOAD_DIR` and `create_dir_all` logic — is not compiled at all. Meanwhile actual image handling went a different way (base64 data URLs stored via JSON, per the 10MB limit comment in `src/routes/helpers.rs:41-42` and 20MB base64 validation in `src/models/humidor.rs:83,131`). This is an abandoned alternative implementation that will mislead anyone searching for how uploads work. Delete it or wire it up.

### 2.3 Route-module boundaries don't match handler-module boundaries
**Priority: MEDIUM**
**Files:** `src/routes/organizers.rs` vs `src/handlers/{brands,sizes,origins,strengths,ring_gauges}.rs`; `src/routes/favorites.rs:52-99` vs `src/handlers/wish_list.rs`; `src/routes/humidors.rs:68-99` vs `src/handlers/humidor_shares.rs`; `src/handlers/admin/users.rs` vs `src/routes/admin.rs`

Four different mapping conventions coexist:
- Five handler files (brands, sizes, origins, strengths, ring_gauges) collapse into one route file (`organizers.rs`) — a name that appears nowhere in the handlers or the API paths.
- `wish_list` has its own handler file but its routes live inside `routes/favorites.rs`.
- `humidor_shares` has its own handler file but its routes live inside `routes/humidors.rs`.
- Admin gets a *directory* (`handlers/admin/users.rs`, with a `mod.rs` that only re-exports) while every other domain is a flat file.

To find the route for a given handler you must know per-domain history. Pick one rule (1:1 handler-file ↔ route-file is the least surprising) and apply it.

### 2.4 Committed development debris: `main.rs.backup`, `build-errors.txt`, `cigar_scraper.py`
**Priority: MEDIUM**
**Files:** `src/main.rs.backup` (1,175 lines, tracked), `build-errors.txt` (tracked; UTF-16LE PowerShell error transcript referencing `C:\Projects\Humidor`), `src/services/cigar_scraper.py` (tracked; Python/BeautifulSoup twin of the Rust scraper, referenced by nothing)

All three are in git. `main.rs.backup` is a stale pre-refactor copy of `main.rs` sitting inside `src/` — it isn't compiled (not declared as a module) but is the first thing a search for any symbol in `main.rs` will double-hit. `cigar_scraper.py` duplicates `services/mod.rs`'s scraper logic in another language with no invocation path from the app (no `Command::new("python")` anywhere). `build-errors.txt` is a machine-local error dump from a Windows machine. All should be deleted; version control is the backup.

### 2.5 `rustfmt.toml` pins edition 2021 while the crate is edition 2024
**Priority: LOW**
**Files:** `rustfmt.toml:4`, `Cargo.toml:4`

Formatting is parsed with edition 2021 rules while the code is edition 2024 (and uses 2024 idioms like let-chains — `src/main.rs:38-40`, `src/services/mod.rs:192-194`). Rustfmt currently tolerates this, but edition-sensitive parsing differences will surface as spurious `cargo fmt -- --check` failures in CI. The values should match.

### 2.6 Handler naming: `_handler` suffix only where names collide with service functions
**Priority: LOW**
**Files:** `src/handlers/backups.rs:34,96,114` (`create_backup_handler`, `delete_backup_handler`, `restore_backup_handler`) vs every other handler (`create_cigar`, `delete_humidor`, ...)

The suffix exists only because `handlers/backups.rs` imports same-named service functions (`create_backup`, `delete_backup`, `restore_backup`) into its namespace. The inconsistency is a symptom of naming collision management rather than a convention; qualified calls (`backup::create_backup`) would let the handlers use the standard names.

### 2.7 Two re-export conventions: curated lists vs glob
**Priority: LOW**
**Files:** `src/handlers/mod.rs:17-53` (explicit curated `pub use` lists), `src/models/mod.rs:13-23` and `src/handlers/admin/mod.rs:3` (`pub use x::*` globs)

`handlers/mod.rs` carefully re-exports named functions "to avoid conflicts" (its own comment), while `models/mod.rs` glob-exports 11 modules into one namespace — which is exactly the pattern that creates such conflicts and hides where a type comes from. One convention should win; explicit lists scale better in a crate this size.

---

## 3. Inconsistent Patterns

### 3.1 Three coexisting error-handling regimes across handlers
**Priority: HIGH**
**Files:** representative examples —
- Rejection + `AppError` (the designed path): `src/handlers/cigars.rs`, `src/handlers/admin/users.rs`, `src/handlers/auth.rs:78-83`, with centralized mapping in `src/errors.rs:123-189`
- `Infallible` + hand-rolled inline `StatusCode` JSON: `src/handlers/humidors.rs:11-69` (and the whole file, 28 `StatusCode` sites)
- `Rejection` signature but errors returned as `200 OK` message bodies: `src/handlers/backups.rs` (see 1.9)

Three consequences:
1. **Error body shape differs by endpoint.** The `handle_rejection` path returns `{"error": CODE, "message": "..."}` (`errors.rs:38-44`); `humidors.rs` returns `{"error": "..."}` or `{"error": "...", "details": "..."}` (`humidors.rs:60-63`); `backups.rs` returns `{"message": "..."}`. A frontend cannot handle errors generically.
2. **The information-hiding policy is enforced in only one regime.** `errors.rs:83-97` guarantees DB errors are never exposed; `humidors.rs:60-63` sends `"details": e.to_string()` — raw Postgres error text — straight to the client, and `backups.rs` interpolates raw errors into messages (1.9).
3. **`Infallible` handlers can't participate in rejection recovery**, so any cross-cutting error behavior added to `handle_rejection` (metrics, request IDs, localization) silently skips `humidors.rs`.

The `AppError` machinery is well-designed; it's just not adopted in 3 of 17 handler files. Migrating `humidors.rs`, `images.rs` (if kept), and `backups.rs` to it removes the whole class.

### 3.2 Row extraction: positional indices in most code, named columns in middleware
**Priority: MEDIUM**
**Files:** positional: `src/handlers/humidors.rs:41-53`, `src/handlers/auth.rs:582-590`, and most handlers; named: `src/middleware/auth.rs:114-121` (`row.get("id")`, `row.get("username")`, ...)

Both styles query the same `users` table columns. Positional `row.get(0)`…`row.get(9)` breaks silently-at-runtime if a SELECT column list is reordered, and the codebase has many near-identical hand-mapped SELECTs (the `Humidor` row-mapping block is copy-pasted in at least `get_humidors` and `get_humidor`). Named access is self-documenting and reorder-proof. Pick one — and factor the repeated row→struct mappings into `From<&Row>` impls on the models, which is what the `models/` layer is for.

### 3.3 Validation is a trait for some request types, ad-hoc handler code for others
**Priority: MEDIUM**
**Files:** trait impls: `src/models/{brand.rs:34,52, cigar.rs:90,113, humidor.rs:46,92, origin.rs:34,58, ring_gauge.rs:33,48, size.rs:37,59, strength.rs:31,52}`; ad-hoc: `src/handlers/auth.rs` (setup/login/password flows call `validate_email`/`validate_length` inline), user/share/public-share request types have no `Validate` impls

The `Validate` trait (`src/validation.rs`) covers cigar/humidor/organizer models, but the security-sensitive request types — user creation, password change, share grants, public share options — validate (or don't) inline in handler bodies. New contributors get no signal about where validation is supposed to live, and there's no single place to audit input rules. Either extend the trait to all `Create*`/`Update*` request types or drop it; half-adoption is the worst state.

### 3.4 Error helper macros defined and never used
**Priority: LOW**
**Files:** `src/errors.rs:192-213` (`db_error!`, `validation_error!`, `not_found!`) — zero call sites in `src/`

Three exported macros intended to standardize error construction have no uses; handlers write `AppError::DatabaseError("Database connection failed".to_string())` longhand everywhere (e.g. `src/handlers/auth.rs:80-82`, repeated dozens of times). Dead abstraction: either adopt the macros (they'd remove a lot of repetition) or delete them.

### 3.5 Scattered `#[allow(dead_code)]` covering real dead code
**Priority: LOW**
**Files:** `src/errors.rs:7` (entire `AppError` enum), `src/errors.rs:55`, `src/middleware/auth.rs:13,38,140`, `src/validation.rs`, `src/models/humidor_share.rs`, `src/models/password_reset.rs`, `src/handlers/auth.rs` (9 sites total)

`#[allow(dead_code)]` on the whole `AppError` enum exists because the *binary* target (which re-declares modules privately — finding 1.1) doesn't use every variant, while the library does. The allows are masking the dual-crate problem rather than documenting intent. `with_optional_auth` (`middleware/auth.rs:140-157`) is genuinely unused from both targets.

---

## 4. Half-Implemented or Abandoned Work

### 4.1 Backup path-traversal checks that cannot ever fail
**Priority: HIGH**
**Files:** `src/handlers/backups.rs:65-67` (download), `src/handlers/backups.rs:163-170` (upload), `src/handlers/backups.rs:222-229` (setup restore)

Each check is `backups_dir.join(&filename)` followed by `backup_path.starts_with(backups_dir)`. `Path::starts_with` compares *lexical components*, and `join("../x")` yields `backups/../x`, whose first component is still `backups` — so the guard passes for every input; it is a no-op. `download_backup`'s filename comes from a path segment (can't contain `/`), but `upload_backup` and `setup_restore_backup` take the filename from the multipart `filename` field, which the client fully controls: `../../app/static/app.js.zip` passes both the `.zip` check and the `starts_with` check and is written outside the backups dir (`backups.rs:185`, `backups.rs:243`). The security *intent* is present in comments ("Security check: ensure the path is within backups directory") but the implementation was never finished — it needs canonicalization or filename sanitization (reject any filename containing path separators / `..`).

### 4.2 Test harness manually re-creates the `wish_list` table with a stale "not embedded yet" comment
**Priority: MEDIUM**
**Files:** `tests/common/mod.rs:39-60`, `migrations/V8__create_wish_list_table.sql`

`setup_test_db` runs refinery migrations, then executes a hand-written `CREATE TABLE IF NOT EXISTS wish_list ...` with the comment "V8 migration not embedded yet". V8 exists and `embed_migrations!("migrations")` at `tests/common/mod.rs:7` embeds the whole directory — the workaround is stale. Worse, the hand-written DDL is a *second copy* of the schema that can drift from V8 (column defaults, index names); tests would then pass against a schema production doesn't have.

### 4.3 `ScrapedCigarData.size` field is never populated
**Priority: LOW**
**Files:** `src/services/mod.rs:15` (field), `src/services/mod.rs:490` (only other mention, a debug log)

All five scrapers fill `length`/`ring_gauge` via `extract_size_info`, but no code path ever assigns `size`. It's serialized to the API response and logged, permanently `null`. Either derive it (`"6 x 52"`) or remove the field from the contract.

### 4.4 Orphaned image-upload handler
**Priority: covered as finding 2.2** (same artifact — dead `src/handlers/images.rs`).

### 4.5 Anti-"false alert" instructions committed for automated tooling
**Priority: LOW**
**Files:** `.github/IGNORE_FALSE_ALERTS.md`, `Dockerfile:2-4`

The repo commits prose instructing automated tools to ignore "Rust 2.0" upgrade alerts ("Ignore any automated alerts claiming 'Rust 2' is available"). Whatever bot produced those alerts, embedding "ignore this class of alert" directives in the Dockerfile header and a dedicated markdown file is fragile process-patching: the Dockerfile comment will outlive the bot, and future automation (or humans) can't tell whether the exception still applies. The durable fix is configuring the alerting tool itself (as the file's own "Action" section suggests) and deleting these artifacts. Note also that any instruction-like text committed to a repo gets read by AI-assisted tooling; keeping such directives around invites tools to over-generalize "ignore version alerts."

### 4.6 Root-level clutter tracked in git
**Priority: LOW**
**Files:** `build-errors.txt` (see 2.4), `docs/` (33 files including one-off session artifacts like `SECURITY_AUDIT_2025-01-11.md`, `MOBILE_UI_OVERHAUL.md`, `DEPENDENCY_UPDATES_PLAN.md` alongside durable docs like `API.md`)

`docs/` mixes permanent reference documentation with dated, task-scoped writeups. Without an `archive/` split or naming convention, it's not discoverable which documents describe the current system (e.g. is `PUBLIC_SHARING_IMPLEMENTATION_OPTIONS.md` a decision record or a live proposal?).

---

## 5. Dependency Findings

### 5.1 `base64` is declared but never used
**Priority: MEDIUM**
**Files:** `Cargo.toml:33`

No `use base64` or `base64::` call exists in `src/` or `tests/` (the only matches are comments). Base64 image handling happens client-side; the server stores the strings opaquely. Dead dependency — remove it (it still costs compile time and audit surface).

### 5.2 `tokio-test` dev-dependency is unused
**Priority: LOW**
**Files:** `Cargo.toml:49`

Zero references in `tests/` or `src/`. Tests use `#[tokio::test]` (from tokio's `full` features) and `serial_test`. Remove.

### 5.3 `metrics-exporter-prometheus` `http-listener` feature is enabled but the listener is never used
**Priority: LOW**
**Files:** `Cargo.toml:46`, `src/main.rs:262-264`, `src/main.rs:496-499`

The app calls `PrometheusBuilder::new().install_recorder()` and serves metrics through its own warp route via `handle.render()`. The `http-listener` feature (which pulls in hyper/http server machinery inside the exporter) exists to run the exporter's *own* HTTP server — a path this code never takes. Dropping the feature trims the dependency tree.

### 5.4 `once_cell` for a single static that std now covers
**Priority: LOW**
**Files:** `Cargo.toml:44`, `src/main.rs:15,26`

The only use is `Lazy<Instant>` for startup time. On the toolchain this project builds with (edition 2024 ⇒ Rust ≥1.85), `std::sync::LazyLock` is stable and drop-in. One fewer dependency.

### 5.5 `rand 0.8` uses APIs renamed/deprecated in the current 0.9 line
**Priority: LOW**
**Files:** `Cargo.toml:39`, `src/main.rs:113-118` (`thread_rng`, `distributions::Alphanumeric`)

Not urgent (0.8 is maintained), but the crate's only substantive use is one 5-line secret generator; when the pinned-dependency migration documented in `docs/DEPENDENCY_UPDATES_PLAN.md` happens, this is a 2-minute rename (`rand::rng()`, `rand::distr`). Flagged so it lands in that plan rather than being forgotten — unlike `warp`/`jsonwebtoken`/`zip`, this pin is *not* annotated in `Cargo.toml`.

### 5.6 Deliberate version pins are documented — good — but `zip 0.6` deserves a timeline
**Priority: LOW**
**Files:** `Cargo.toml:40-41`, `docs/DEPENDENCY_UPDATES_PLAN.md`

`warp 0.3`, `jsonwebtoken 9`, and `zip 0.6` pins all carry rationale comments (a genuinely good practice). `zip 0.6` is the oldest of the three (superseded by a new maintainer lineage at 1.x/2.x+); since it processes *user-uploaded archives* (`src/services/backup.rs`, restore path), it has the highest exposure of the pinned crates and should be first in the migration order, not "Phase 3" alongside cosmetic API churn.

---

## Summary of Highest-Impact Items

| # | Finding | Priority |
|---|---------|----------|
| 1.2 | JWT secret resolution duplicated & diverged — auto-generated-secret deployments sign tokens with a hardcoded sentinel | HIGH |
| 4.1 | Path-traversal guards in backup upload/restore are lexical no-ops | HIGH |
| 1.3 | Startup SMTP validation checks env vars the mailer never reads | HIGH |
| 3.1 | Three error-handling regimes; two of them leak internal error details, violating the codebase's own policy (`humidors.rs:60-63`, `backups.rs`) | HIGH |
| 2.1 | `.env` with `JWT_SECRET` tracked in git | HIGH |
| 1.1 | Dual-crate layout compiles everything twice; bin and lib can drift | HIGH |
| 2.2 | `handlers/images.rs` — 159 lines of uncompiled, unrouted dead code | HIGH |
| 1.4 | Middleware→handler dependency inversion; homeless authorization logic | HIGH |
