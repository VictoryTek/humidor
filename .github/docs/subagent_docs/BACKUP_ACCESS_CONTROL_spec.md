# BACKUP_ACCESS_CONTROL — Spec (MASTER_PLAN #2)

## Current state
- `src/routes/backups.rs`: all six authenticated routes (list, create, download, delete, restore, upload) use
  `with_current_user`, so any logged-in non-admin can list/download a full-DB export, create/delete backups,
  and restore (which `TRUNCATE`s `users` and every data table — `services/backup.rs::import_database`).
- `POST /api/v1/setup/restore` uses only `with_db`: anonymous. It runs `restore_backup` regardless of whether
  setup is already complete, so an anonymous caller can replace the whole DB with an attacker-crafted backup
  (including an attacker-controlled admin) at any time.
- `with_admin(pool)` already exists in `src/middleware/auth.rs` (returns `AppError::Forbidden` for non-admins).
- `get_setup_status` defines "setup needed" as zero rows in `users WHERE is_admin = true`.

## Problem
Broken access control on the most destructive/most sensitive endpoints in the app.

## Solution
1. `src/routes/backups.rs`: replace `with_current_user` with `with_admin` on all six authenticated routes
   (spec'd item names only list/download, but create/delete/restore/upload are equally privileged; restore
   is the worst).
2. `src/handlers/backups.rs::setup_restore_backup`: before reading the upload, query
   `SELECT COUNT(*) FROM users WHERE is_admin = true`; if > 0, reject with `AppError::Forbidden`
   ("Setup already completed"). Same definition as `get_setup_status`. Check happens before writing to disk.
3. Out of scope (separate items): path-traversal guards (#3), 200-on-error responses (#8).

## Not changing
- Frontend: `setup.js` legitimately calls setup/restore only while no admin exists — unaffected.
  Backup UI in `app.js` uses `makeAuthenticatedRequest`; non-admins will now see 403 (see Risks).

## Dependencies
None new; no Context7 lookup needed.

## Validation commands (all safe)
`cargo fmt -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build`,
`cargo test --lib --verbose`; integration tests only if a Postgres is available.

## Risks
- Non-admin users who could previously use the Backups UI now get 403; that is intended. Check whether the UI
  is already admin-only in `static/index.html`/`app.js`; hiding it is a follow-up, not part of this change.
- Check-then-restore is not atomic; a concurrent first-admin creation could race. Acceptable: requires
  hitting an uninitialised instance in a narrow window, and is strictly better than the current state.

## Addendum: MASTER_PLAN #3 (path traversal), implemented together
- `services/backup.rs::backup_path(name)`: accepts only a bare `*.zip` filename (no `/`, `\`, `..`, NUL); used by
  download, delete, restore, upload. Replaces the lexical `starts_with` checks, which never rejected `../`.
- `restore_backup` no longer treats names containing `/` as arbitrary filesystem paths; new
  `restore_backup_from_path` is used by setup-restore with a server-generated temp filename
  (`setup_restore_<uuid>.zip` in the OS temp dir) — the client filename never touches the filesystem.
- Upload route: `with_admin` now runs before the multipart body parse.
- Tests: `tests/backup_access_tests.rs` (403/401 on all six routes, admin OK, traversal canary for
  download/delete/upload, setup-restore 403 once an admin exists) + 2 unit tests for `backup_path`.
