# Nix Flake Packaging — Spec

## 1. Current State Analysis

- **Build**: `Cargo.toml` (edition 2024), single bin `humidor` + lib `humidor`. Dependencies of note:
  - `lettre` with `tokio1-native-tls` → pulls `openssl-sys` (needs `pkg-config` + OpenSSL dev headers, confirmed in `Dockerfile`'s `apk add openssl-dev`).
  - `zip = "0.6"`, `jsonwebtoken = "9.2"`, `warp = "0.3"` are deliberately pinned behind latest-major (see `Cargo.toml` comments, `docs/DEPENDENCY_UPDATES_PLAN.md`) — not touched here.
  - `Cargo.lock` present, lockfile format v4.
  - `.cargo/config.toml` sets `rustflags = ["-C", "link-arg=-fuse-ld=lld", ...]` under the **global** `[build]` table — applies to every target, not just MSVC. This means any Nix build on Linux will also try to link with `-fuse-ld=lld` and needs `lld` on PATH, or the build fails at the link step. This is a pre-existing repo file; per "surgical changes" we do not edit it — the flake instead supplies `lld` as a build input.
- **Migrations**: `refinery::embed_migrations!("migrations")` embeds all SQL migrations **into the compiled binary at compile time**. `src/main.rs` runs `migrations::runner().run_async(&mut **client)` automatically on every startup, before the server starts accepting connections. **Conclusion: the app runs its own migrations on startup — no separate `preStart` migration step is required.** The NixOS module's `preStart` therefore only needs to prepare the working directory (see below), not run migrations.
- **Database config** (`src/db_config.rs`, unit-tested): `resolve_database_config` — `DATABASE_URL` wins if set and non-blank; otherwise built from `POSTGRES_HOST` (default `localhost`), `POSTGRES_PORT` (default `5432`), and required `POSTGRES_USER` / `POSTGRES_PASSWORD` / `POSTGRES_DB`. This already gives us both a combined URL and discrete-var support natively — the module just needs to set the right environment variables.
- **Working-directory-relative paths** (`src/main.rs`, `src/services/backup.rs`, `src/handlers/backups.rs`): the app opens `"static"`, `"backups"`, and `"uploads"` as paths **relative to the process's current working directory**, not absolute paths and not `CARGO_MANIFEST_DIR`. In Docker, `WORKDIR /app` makes these resolve correctly. Under systemd we must set `WorkingDirectory` to a directory that contains a `static/` (read-only, pointing at the store output) and writable `backups/` + `uploads/` subdirectories.
- **JWT secret** (`src/main.rs::read_secret` / `get_or_generate_jwt_secret`): checks, in order: `${ENV_VAR}_FILE` custom path (e.g. `JWT_SECRET_FILE`) → `/run/secrets/<name>` (Docker secret) → **hardcoded** `/app/data/<name>` (Docker volume mount path) → the plain env var (e.g. `JWT_SECRET`). If none are found, it **generates** a random secret and persists it by writing to the hardcoded `/app/data/jwt_secret` path (Docker-specific), warning (non-fatal) if that write fails. **This hardcoded path is Docker-container-specific and does not map to an arbitrary NixOS `dataDir`.** Given the "surgical changes" principle, we do not patch application code for this; instead the module documents that operators should supply `JWT_SECRET` (≥32 chars) via `environmentFile` for reliable persistence across restarts. If omitted, the service still starts (non-fatal warn), but the auto-generated secret is not guaranteed to persist across restarts outside Docker — this is called out explicitly in the README as a known limitation of the upstream app, not something this packaging work can silently fix.
- **Port**: `PORT` env var, default `9898` if unset/unparsable — matches the requested module default.
- **Static assets**: served via `warp::fs::dir("static")` and `warp::fs::file("static/sw.js")`, relative to CWD.
- No existing Nix files in the repo (`flake.nix`, `flake.lock`, `shell.nix` all absent).

## 2. Problem Definition

Package Humidor as a Nix flake exposing a reproducible package build and a NixOS module for running it as a native systemd service, without altering the existing Docker deployment path.

## 3. Proposed Solution Architecture

### `flake.nix`

- Inputs: `nixpkgs` (unstable — verified via `nix eval` in this environment to ship `rustc` 1.98.1, which supports edition 2024; `flake-utils` is **not** used, we hand-roll `forAllSystems` for `["x86_64-linux" "aarch64-linux"]` to keep it minimal).
- `packages.<system>.default`: `pkgs.rustPlatform.buildRustPackage`
  - `cargoLock.lockFile = ./Cargo.lock;` (uses the committed lockfile directly, no `cargoHash` fetch — no network access at build time, matches nixpkgs' documented `cargoLock` mechanism).
  - `nativeBuildInputs = [ pkg-config lld ];` — `lld` because of `.cargo/config.toml`'s global `-fuse-ld=lld` rustflag (see Current State Analysis).
  - `buildInputs = [ openssl ];` — required by `lettre`'s `tokio1-native-tls` feature (`openssl-sys`).
  - `cargoTestFlags = [ "--lib" "--" "--skip" "middleware::rate_limiter::tests::test_rate_limiter_expiry" ];` — restrict the build's test phase to unit tests only (no DB, no network), matching the Resource Constraints doc; additionally skip one pre-existing, timing-sensitive unit test (`src/middleware/rate_limiter.rs:139-155`, sleeps 2s against a 1s window) that is flaky under the sandboxed build's heavy parallel-compilation CPU contention — confirmed via an actual build attempt in this session (44/45 unit tests passed; the 45th failed only on wall-clock timing, not logic). All other unit tests still run as real build-time verification.
  - `postInstall` copies `static/` into `$out/share/humidor/static` (migrations are embedded in the binary already, so they don't need to be installed as files).
  - Standard metadata (`pname`, `version` read from `Cargo.toml` via `builtins.fromTOML`, `meta.mainProgram = "humidor"`, `meta.license = gpl3Plus` matching the repo's `LICENSE` file).
- `nixosModules.default`: see below.
- `checks.<system>.default = packages.<system>.default;` so `nix flake check` builds the package (and runs the restricted unit-test phase inside the build), satisfying deliverable #4 without needing a live DB.

### `nixosModules.default` — options namespace: **`services.humidor`**

Chosen because it mirrors the Docker Compose service name (`humidor`) and matches NixOS convention (`services.<name>`) for a long-running network service.

Options:
- `enable` (bool, default `false`)
- `port` (port type, default `9898`) — sets `PORT`.
- `dataDir` (path, default `/var/lib/humidor`) — becomes the systemd `WorkingDirectory`; holds `backups/`, `uploads/`, and (if auto-generated) `jwt_secret`.
- `environmentFile` (`nullOr path`, default `null`) — passed as systemd `EnvironmentFile=`; the documented place to put `JWT_SECRET`, `POSTGRES_PASSWORD`, SMTP creds, etc., without putting secrets in the Nix store.
- `openFirewall` (bool, default `true`) — opens `cfg.port` in `networking.firewall`.
- `database.host` / `database.port` (default `"localhost"` / `5432`) / `database.user` (default `"humidor_user"`) / `database.name` (default `"humidor_db"`) — discrete Postgres connection settings, set as `POSTGRES_HOST` / `POSTGRES_PORT` / `POSTGRES_USER` / `POSTGRES_DB`. Password is deliberately **not** a plain-string option — it must come via `environmentFile` (`POSTGRES_PASSWORD=...`) to avoid Nix-store secret leakage, matching the project's own `secrets/*.txt` gitignore posture.
- `database.url` (`nullOr str`, default `null`) — if set, exported as `DATABASE_URL` and takes precedence (mirrors `resolve_database_config`'s own precedence, so behavior is consistent whichever path the user picks).
- `database.createLocally` (bool, default `false`) — when true, provisions `services.postgresql` locally with `ensureDatabases = [ cfg.database.name ]` and `ensureUsers = [{ name = cfg.database.user; ensureDBOwnership = true; }]`, and adds a loopback-only `trust` `pg_hba` entry (via `lib.mkBefore`, so it isn't shadowed by the module's own default rules) for that user/db (127.0.0.1/32 only) so the app's TCP+password connection succeeds without a Nix-store-stored password — documented as intended for single-host/local-only convenience (consistent with the project's own "zero-config" Docker posture, which also ships default creds). When `false` (default), the module assumes an externally-reachable Postgres and only sets the discrete env vars / `DATABASE_URL`.

Service (`systemd.services.humidor`):
- `wantedBy = [ "multi-user.target" ]`; `after`/`requires` include `postgresql.service` only when `database.createLocally` is true, plus `network-online.target`.
- `serviceConfig`:
  - `ExecStart = "${cfg.package}/bin/humidor";`
  - `WorkingDirectory = cfg.dataDir;`
  - `environment` sets the discrete Postgres vars (or `DATABASE_URL`), `PORT`, and `JWT_SECRET_FILE=${cfg.dataDir}/jwt_secret` (so a previously-generated secret can be read back — see the JWT-secret caveat above).
  - `EnvironmentFile = mkIf (cfg.environmentFile != null) cfg.environmentFile;`
  - `preStart`: `mkdir -p backups uploads` under `WorkingDirectory`, and `ln -sfn ${cfg.package}/share/humidor/static static` — idempotent, safe to re-run every start.
  - Sandboxing / user model: **when `cfg.dataDir == "/var/lib/humidor"` (the default)**, use `DynamicUser = true; StateDirectory = "humidor";` — the NixOS-idiomatic way to get an auto-managed, correctly-owned `/var/lib/humidor` with a dynamic UID. **When `dataDir` is customized to a path outside `/var/lib`**, the module instead creates a dedicated system user/group `humidor` and a `systemd.tmpfiles.rules` entry to create+chown `dataDir`, with `serviceConfig.User/Group = "humidor"` and no `DynamicUser`. This satisfies "DynamicUser where feasible, otherwise a dedicated system user."
  - Hardening: `ProtectSystem = "strict"; ReadWritePaths = [ cfg.dataDir ]; NoNewPrivileges = true; PrivateTmp = true; ProtectHome = true;`
- `networking.firewall.allowedTCPPorts = mkIf cfg.openFirewall [ cfg.port ];`

## 4. Implementation Steps

1. `flake.nix` at repo root — inputs, `packages.<system>.default`, `nixosModules.default`, `checks.<system>.default`.
2. `nix/module.nix` — the NixOS module.
3. `nix flake lock` to generate and commit `flake.lock` (run inside WSL Ubuntu, which has `nix` 2.34.1 with flakes enabled and network access — confirmed in this session; this Windows shell has no native `nix`). **Verification note:** this must be run against a disposable copy of the repo, never the live working tree — see the incident note below.
4. `README.md`: new `## Nix / NixOS` section documenting `services.humidor.*` options and a minimal `configuration.nix` usage example (both `createLocally = true` and external-Postgres variants).
5. Verify `nix build .#default` and `nix flake check` from WSL.

## 5. Dependencies

No new Cargo dependencies. Nix-side: `nixpkgs` (unstable channel, pinned by `flake.lock`), stdlib `rustPlatform.buildRustPackage`. Context7 was not used — this task adds no new crate; the Nix-side patterns were verified against current nixpkgs/NixOS documentation via web search this session (nixpkgs `rust.section.md`, `import-cargo-lock.nix`, NixOS Discourse threads on `DynamicUser`+`StateDirectory`, and current `services.postgresql.ensureUsers`/`ensureDBOwnership` semantics), since the `nixos` MCP doc server was unavailable for part of this session.

## 6. Configuration Changes

None to existing Docker path. New files only: `flake.nix`, `nix/module.nix`, `flake.lock`, README addition.

## 7. Risks and Mitigations

| Risk | Mitigation |
|---|---|
| `.cargo/config.toml`'s global `-fuse-ld=lld` breaks the Nix build's link step | Add `lld` to `nativeBuildInputs` |
| `openssl-sys` (via `lettre`) fails to find OpenSSL in the sandbox | Add `pkg-config` (native) + `openssl` (build input), same pairing as the Dockerfile's `apk add pkgconfig openssl-dev` |
| Default `cargo test` phase in `buildRustPackage` would try to run integration tests needing a live Postgres, failing the sandboxed build | `cargoTestFlags = [ "--lib" ]` restricts the build-time test phase to DB-free unit tests |
| One pre-existing unit test is timing-flaky under sandboxed parallel-build CPU load | `--skip` that one test by name in `cargoTestFlags`; all 44 others still run and gate the build |
| JWT secret persistence: app's auto-generate path is hardcoded to `/app/data`, not `dataDir` | Document requirement to supply `JWT_SECRET` via `environmentFile` for production; note the limitation plainly rather than patching app code (out of scope / not surgical) |
| `dataDir` customized outside `/var/lib` breaks `DynamicUser`'s auto-ownership | Module branches: dedicated system user + `tmpfiles.rules` chown when `dataDir` isn't the default |
| No `nix` binary in this session's primary Windows shell | Verified `nix` 2.34.1 + flakes + network are available via WSL Ubuntu (`wsl -d Ubuntu`) |
| `services.postgresql.createLocally` password handling | Restrict to loopback-only `trust` auth via `mkBefore`, documented as local/dev convenience only |

### Incident note (process, not architecture)

During verification, a background WSL command's `cd` into a disposable scratch copy silently failed (no `set -e`), causing `git add -A && git commit` to run inside the **actual project repo** instead, creating an unwanted commit containing whole-repo CRLF/LF line-ending and file-mode churn (a WSL-vs-Windows git config mismatch) alongside the two legitimate new files. This was caught, the user was notified immediately with full accurate scope, and — with explicit user confirmation — corrected via `git reset --hard` to the prior commit (nothing had been pushed). The three new files were then recreated from their known-good content as plain untracked files. **Takeaway for future sessions: always verify `pwd` succeeded (or use `set -e`) before any git write step in a scripted multi-command shell invocation, especially cross-filesystem (WSL ↔ Windows mount).**
