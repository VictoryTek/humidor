# CORS_CREDENTIALS — Spec (MASTER_PLAN #6)

## Current state
Default `CORS_MODE=permissive` (and the unknown-mode fallback) used `allow_any_origin()` +
`allow_credentials(true)`. In warp 0.3.7, any-origin mode *reflects the request `Origin`* in
`Access-Control-Allow-Origin` and adds `Access-Control-Allow-Credentials: true` (`filters/cors.rs`), i.e. any website
gets credentialed cross-origin access with readable responses. The auth middleware also accepts an `auth_token`
cookie (nothing in the app sets it today), so a cookie-borne session would be readable by any origin.

## Solution
- New `middleware::cors::permissive()` used by both permissive arms in `main.rs`: same origins/headers/methods,
  **no** `allow_credentials`. The frontend uses `Authorization` headers, which don't need credentialed CORS, so
  self-hosted "works from any IP" behaviour is unchanged.
- Strict mode is unchanged (explicit origin list + credentials is spec-valid).
- Docs: CORS_CONFIGURATION.md and SECURITY_MODEL.md updated.
- Unit test asserts neither actual nor preflight responses carry `Access-Control-Allow-Credentials` for a foreign origin.

## Not changing
`auth_token` cookie acceptance in `middleware/auth.rs` (latent; no code sets the cookie) — recommend removing or
hardening (SameSite/CSRF) if cookie auth is ever adopted.

## Risks
Any external client relying on cookies cross-origin in permissive mode will stop working (none exist in-repo).
