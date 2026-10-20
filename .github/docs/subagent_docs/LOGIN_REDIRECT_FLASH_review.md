# LOGIN_REDIRECT_FLASH — Review

Spec: `.github/docs/subagent_docs/LOGIN_REDIRECT_FLASH_spec.md`

## Files reviewed

- `static/index.html` — inline auth-redirect script added in `<head>` after `<meta charset>`,
  before all stylesheets.
- `static/login.js` — `checkExistingAuth()` now returns after the logged-in redirect, otherwise
  checks `/api/v1/setup/status` and `location.replace('/setup.html')` when `needs_setup`.
- `static/login.html` — `login.js?v=1.5.0` → `?v=1.5.4` (cache bust past the service worker's
  cache-first `.js` strategy).
- `src/main.rs` — `serve_index` no longer injects the setup-status script; the missing-file
  fallback is unchanged.

## Findings

- **Spec compliance:** all four spec items implemented; nothing extra.
- **Correctness:** head-script condition matches `checkAuth()` in `app.js` (token AND user
  required) and exempts `/shared/humidors/`, which `serve_shared_humidor` also serves from
  `index.html`. Simulated in Node:

  ```
  logged out  /            {"redirect":"/login.html","hidden":true}
  logged in   /            {"redirect":null,"hidden":false}
  token only  /            {"redirect":"/login.html","hidden":true}
  logged out  /shared/...  {"redirect":null,"hidden":false}
  ```
- **First-run setup:** `/` → `/login.html` → `/setup.html`. Same endpoint and response field
  (`needs_setup`, `src/handlers/auth.rs`) as the removed injected script.
- **Security:** no new data exposure; script reads only presence of existing `localStorage` keys.
  Server still enforces auth on every API route.
- **Performance:** a few bytes of inline JS; the removed server-side string replace saves a
  full-document copy per request.
- **Consistency:** matches existing JS style (`fetch().then().catch(console.error)`), existing
  `?v=` cache-bust convention.
- **Minor/accepted:** on a fresh install the login form may paint briefly before the setup
  redirect (one-time event). Noted out-of-scope items in spec remain unchanged.

No CRITICAL issues. No RECOMMENDED changes.

## Build validation (verbatim tails)

1. `cargo fmt -- --check` → exit 0
2. `cargo clippy --all-targets --all-features -- -D warnings` →
   `Finished \`dev\` profile [unoptimized] target(s) in 32.72s` (exit 0)
3. `cargo build --verbose` → `Finished \`dev\` profile [unoptimized] target(s) in 1m 28s` (exit 0).
   One warning, pre-existing environment config, not from this change:
   `linker stderr: rust-lld: ignoring unknown argument '-fuse-ld=lld'` (`.cargo/config.toml`).
4. `cargo test --lib --verbose` →
   `test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` (exit 0)
5. Integration tests: not required (no changes to models/handlers/services/migrations).
6. `node --check static/login.js` → OK; head script parsed via `new Function` → OK.

| Category | Score | Grade |
|----------|-------|-------|
| Specification Compliance | 100% | A |
| Best Practices | 95% | A |
| Functionality | 95% | A |
| Code Quality | 95% | A |
| Security | 100% | A |
| Performance | 100% | A |
| Consistency | 95% | A |
| Build Success | 100% | A |

**Overall Grade: A (97%)**

**Result: PASS**
