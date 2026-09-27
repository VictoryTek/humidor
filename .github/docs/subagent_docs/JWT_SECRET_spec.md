# JWT_SECRET — Spec (MASTER_PLAN #1, reopened)

## Problem (reproduced)
With no `JWT_SECRET`, startup auto-generates a secret and tries to persist it to `/app/data/jwt_secret`, but
`main.rs` discarded the returned value. Runtime `jwt_secret()` re-resolved from files/env, found nothing when the
write had failed (any run outside the standard Docker layout), and fell back to the hardcoded public sentinel
`"INVALID_SECRET_NOT_CONFIGURED"` for BOTH signing and verifying. Anyone could forge a token for any user id:
reproduced against the real binary — forged token → `GET /api/v1/users/self` 200 and `GET /api/v1/admin/users` 200.
Also: the secret resolution chain existed twice (`main.rs::read_secret` and `services/jwt.rs`) — the "duplicated &
diverged" half of the item.

## Change
- `services/jwt.rs`: `STARTUP_SECRET: OnceLock<String>` + `init_secret()`. Runtime secret = startup secret, else the
  `JWT_SECRET` env var (for tests / library use without `main`). Blank values count as unset (an empty HMAC key is forgeable).
  The runtime copy of the file-based chain is removed (single source of truth: startup's `read_secret`, which also
  enforces the 32-char minimum).
- **No fallback secret.** `generate_token` / `verify_token` return `InvalidKeyFormat` when none is configured
  (logged at error level); the middleware maps that to 401, login to its existing 500 path.
- `main.rs`: `services::jwt::init_secret(get_or_generate_jwt_secret()?)`.
- Core logic split into `generate_with` / `verify_with(Option<&str>, …)` so it is unit-testable without env mutation.

## Tests
4 unit tests: roundtrip; no secret → cannot issue or verify; **token forged with the old sentinel rejected** (with a
real secret and with none); token signed with another secret rejected.

## Test-harness fix found while verifying
`tests/common/mod.rs`: the two `common::tests::*` (compiled into every test binary) were not `#[serial]`, yet
`setup_test_db()` ends with `DELETE FROM users`; they could delete users created by concurrently-running serial tests.
Now `#[serial]`. See review for the evidence level.
