# LOGIN_REDIRECT_FLASH — Specification

## Current state

- `GET /` is served by `serve_index` (`src/main.rs`), which returns `static/index.html` with an
  injected setup-status `<script>` appended before `</body>`.
- Auth state lives in `localStorage` (`humidor_token`, `humidor_user`); the server cannot see it.
- The only auth check is in `static/app.js` inside a `DOMContentLoaded` handler. `app.js` is the
  single `<script>` at the bottom of `index.html` (~1,600 lines of dashboard markup, ~7,000-line
  script).
- `GET /shared/humidors/:token` (`serve_shared_humidor`) also serves `static/index.html`; `app.js`
  detects that path and skips the auth check.
- `static/login.js` (loaded via `login.js?v=1.5.0`) only checks "already logged in → go to `/`".
  It has no setup-status check.
- Service worker (`static/sw.js`): HTML is network-first; `.css/.js` are cache-first, keyed by the
  full request URL (including query string).

## Problem

The browser paints the dashboard markup (after `styles.css` loads) before `app.js` is downloaded,
parsed and its `DOMContentLoaded` handler runs the auth check. A logged-out visitor therefore sees
the dashboard for a moment before being redirected to `/login.html`.

## Research summary

- Home Registry (`../home-registry/frontend`) ships an empty `<div id="root">` shell and renders a
  neutral loading screen until auth/setup state is resolved — i.e. it never paints protected UI
  before deciding.
- web.dev (critical rendering path): a synchronous `<script>` in `<head>` is parser-blocking, so
  nothing in `<body>` is parsed or painted until it runs.
- MDN `Location.replace()`: the current page is not kept in session history (Back won't return to
  it).
- WHATWG list / bytes.com: setting `location` starts an *asynchronous* navigation; the current
  document may continue loading until the new response commits → also hide the document to
  guarantee no paint.
- FOUC/FOUAC write-ups (Vercel community, theodorusclarence, better-auth-ui): hold rendering until
  auth state is known, or decide on the server (requires cookies).
- OWASP-aligned guidance prefers HttpOnly cookies over `localStorage` for tokens; that would allow
  a server-side redirect but is a security-model change, out of scope.

## Proposed solution

1. **`static/index.html`** — add an inline, synchronous script in `<head>` immediately after
   `<meta charset>` (charset must stay first) and before all stylesheets:
   if the path is not `/shared/humidors/…` and either `humidor_token` or `humidor_user` is missing,
   set `document.documentElement.style.visibility = 'hidden'` and
   `location.replace('/login.html')`. Same condition as `checkAuth()` in `app.js`.
2. **`static/login.js`** — when not already logged in, fetch `/api/v1/setup/status`; if
   `needs_setup`, `location.replace('/setup.html')`. This takes over the first-run setup redirect
   that previously happened from `/`.
3. **`static/login.html`** — bump `login.js?v=1.5.0` → `?v=1.5.4` so service-worker/browser caches
   fetch the new script (new URL misses the cache-first lookup).
4. **`src/main.rs`** — remove the setup-status script injection from `serve_index` (now dead: a
   visitor without a token is redirected from `<head>` before reaching it; a visitor with a token
   skipped it already). The missing-file fallback HTML in `serve_index` is left unchanged.

## Out of scope (noted, not changed)

- Expired-but-present token still shows the dashboard until the first API 401 (would need a
  server round-trip before paint).
- `/login.html` for an already-logged-in user still paints the form before redirecting to `/`.
- Moving tokens to HttpOnly cookies.

## Dependencies

None added. No Context7 lookup required (no external library changes).

## Validation commands (all safe; none in FORBIDDEN COMMANDS)

1. `cargo fmt -- --check`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --verbose`
4. `cargo test --lib --verbose`
5. `scripts/preflight.sh` (Phase 6)

Integration tests are not required: no change to `src/models`, `src/handlers`, `src/services` or
`migrations/`.

## Risks and mitigations

- *Public share links redirected to login* → explicit `/shared/humidors/` exemption, matching
  `app.js`.
- *Fresh install never reaches setup* → setup check moved to `login.js`.
- *Stale cached `login.js`* → version query bump.
- *`localStorage` throws (blocked storage)* → `app.js` already assumes it works; the head script
  does the same, keeping behaviour identical to today.
