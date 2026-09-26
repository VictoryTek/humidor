# XSS_RENDERING — Spec (MASTER_PLAN #4)

## Current state
`static/app.js` builds cards/tables with template strings assigned to `innerHTML`. User-controlled fields
(`cigar.name/notes/wrapper/binder/filler/humidor_location/image_url/retail_link`, organizer names/descriptions,
humidor name/type, scraped data, error messages, backup names) were interpolated raw. `escapeHtml()` existed
(defined twice, identically) but used `textContent→innerHTML`, which does NOT escape `"` or `'`, so it was also
unsafe in attribute values. The share-modal `onclick` used `escapeHtml(...).replace(/'/g,"\\'")`, which is
injectable via `"`, and (once quotes are entity-escaped) via `&#39;` decoding back to `'` before JS parsing.
Shared/public humidor pages render another user's data, and the JWT lives in `localStorage` → token theft.

## Solution (client-side only, no new dependencies)
1. `escapeHtml`: string-replace `& < > " '`; coerce non-strings; `''` for null/undefined/empty.
2. `jsArg(v)`: `escapeHtml(JSON.stringify(String(v)))` — for values inside inline `onclick="fn(${jsArg(x)})"`.
3. `safeUrl(u)`: allow only http(s) (absolute or same-origin relative); else `''`. Used for `retail_link` href.
4. Wrap every user-controlled interpolation in the templates identified by a full `innerHTML`/template sweep
   (43 edits). UUID/numeric interpolations left as-is (server-generated).
5. Fix stray `)` in the humidor-hub share button `onclick` (was a JS syntax error) as a side-effect of using `jsArg`.
6. Bump `app.js?version=` in `index.html` — JS is cache-first in `sw.js`, so clients need a new cache key.

## Not changing / out of scope
- Server-side validation of `retail_link`/`image_url` schemes and CSP header (defence in depth; separate item).
- JWT in `localStorage` (architectural).
- `showToast` callers pass plain text, so it now escapes its message.

## Validation
`node --check static/app.js`; Node unit checks of the helpers against attack strings; grep sweep for leftover raw
interpolations (only `console.log`/`textContent`/`.value` sinks remain); `scripts/preflight.sh`.
