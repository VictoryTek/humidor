# USER_EXPORT — Spec (MASTER_PLAN #14)

## Goal
`GET /api/v1/export?format=json|csv` returns the requesting user's own collection; "Export my collection" buttons in the
in-app profile section. (The backup-scoping half of the original item was done in #2.)

## Assumptions (stated up front; change any of them and the design adjusts)
1. **"Own collection"** = humidors the user *owns*, the cigars in them (including out-of-stock), their wish list and their
   favorites. Humidors merely *shared with* the user are not included (they are someone else's data).
2. **Images are excluded.** `image_url` can be a multi-MB base64 data URL; embedding them would bloat the export and
   memory use. The admin full backup keeps images. (Documented in the UI text.)
3. **JSON** = everything (`humidors`, `cigars`, `favorites`, `wish_list`, `exported_at`, `format_version`).
   **CSV** = one spreadsheet-oriented file: rows for owned cigars (`list=collection`) and wish-list items (`list=wish_list`),
   with a `favorite` column; favorites whose cigar was deleted (snapshot-only) appear only in JSON.
4. Organizer *names* (brand/size/origin/strength/ring gauge) are exported, not ids.
5. No new dependency: CSV is hand-written (~25 lines) — the `csv` crate has no injection protection and defaults to `\n`
   (docs.rs), so it would not remove the work that matters.

## Design
- `services/export.rs`: `collect_user_export(&Client, user_id)` (4 queries, every one filtered by `user_id`; cigars reach the
  user only via `humidors.user_id` since `cigars` has no `user_id`), Serialize structs, pure `to_csv()`.
- `handlers/export.rs`: query parse (`format` default `json`; anything else -> 400), build response.
- `routes/export.rs`: authenticated (`with_current_user`), registered in `main.rs`.
- Response headers: `Content-Type` (`application/json` / `text/csv; charset=utf-8`), `Content-Disposition: attachment;
  filename="humidor-collection-YYYY-MM-DD.<ext>"` (ASCII only), `Cache-Control: no-store`.
- Service worker: `/api/v1/export` bypasses the SW (it caches every `/api/` GET in `DYNAMIC_CACHE` with no logout cleanup —
  MASTER_PLAN #26 — so the export would otherwise persist in the browser after logout).
- Frontend: two buttons in `#profileSection` (`index.html`) + `exportCollection(format)` in `app.js`, modelled on `downloadBackup`;
  `app.js?version=` cache-buster bumped. The legacy `profile.html` (not linked from the app) is not updated.

## Security / correctness requirements (sources)
- CSV quoting: RFC 4180 (CRLF records; quote fields with `,` `"` CR LF; `"` doubled; `text/csv`).
- **CSV/formula injection** (OWASP CSV Injection): text cells starting with `= + - @` (or tab/CR/LF) are neutralised by
  prefixing a tab inside the quoted field. Caveat from OWASP: the tab stays in the data — acceptable because the CSV is for
  human spreadsheet use; the JSON export is the lossless/programmatic format. Numeric columns are generated from numbers and
  are not touched.
- `Cache-Control: no-store` for personal data (MDN). `Content-Disposition` quoted ASCII filename (MDN).
- **Unverified assumption:** a UTF-8 BOM is prepended to the CSV so Excel shows accents (e.g. "Padrón") correctly. I could
  not confirm Excel's behaviour from a primary source (Microsoft's page and Wikipedia don't cover it); the cost of being
  wrong is a stray BOM for programmatic readers (use JSON, or `utf-8-sig`).
- Privacy: no cross-user leakage — tested with two users and marker strings.

## Tests
Integration (two users, markers): JSON scoped to requester; CSV valid per an independent mini RFC-4180 parser (comma/quote/newline
in a field round-trips), BOM + CRLF + headers; formula-injection neutralised; 401 unauthenticated; `format=xml` -> 400; default
JSON; empty account -> valid empty export. Unit tests for the CSV escaping.

## Out of scope
Import; scheduled exports; including shared-with-me humidors; images; the legacy `profile.html`.
