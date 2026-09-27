# USER_EXPORT — Review (MASTER_PLAN #14)

Reviewer: orchestrating agent, inline. Result: **PASS**, with the limits listed below.

## Verification
- `scripts/preflight.sh` (integration tests on throwaway Postgres) -> exit 0: tracked-secrets guard, fmt, clippy `-D warnings`,
  build, unit tests (5 new in `services::export`), 20 suites ok (new `export_tests`: 7), `cargo audit`.
- **Mutation-checked** the two riskiest behaviours: removing the per-user filter on owned cigars makes
  `json_export_contains_only_the_requesters_data` fail ("another user's data leaked"); disabling formula neutralisation makes
  `csv_neutralises_spreadsheet_formula_injection` fail. Source restored byte-for-byte afterwards.
- CSV validated by an independent mini RFC 4180 parser in the test (comma/quote/newline round-trip, 19 fields per row, BOM,
  CRLF), not only by the writer's own unit tests.
- **Real binary, end to end**: created a cigar named `=CMD|' /C calc'!A0, Padrón` with quoted notes through the real API;
  `GET /api/v1/export?format=csv` returned `text/csv; charset=utf-8`, `Content-Disposition: attachment; filename="humidor-collection-<date>.csv"`,
  `Cache-Control: no-store`, the BOM bytes, CRLF records and the cell as `"<TAB>=CMD|' /C calc'!A0, Padrón"`; JSON export returned
  the expected structure; no token -> 401; `format=xml` -> 400.
- Served files verified from the correct process: `/static/sw.js` contains the export bypass; `/` contains both buttons and the
  new `app.js?version=20260926_002`; `/static/app.js` defines `exportCollection`.
- Process note: two of my intermediate local checks were invalid because a stale server from an earlier run still held the port
  (my `kill` had hit a subshell). Found, killed, and redone against the right process; the stale server never affected test results
  (integration tests use in-process filters). No leftover processes now.

## Findings / limits (be aware)
- **Not exercised in a real browser**: the button click -> blob download path (`exportCollection`) was verified by syntax check and by
  serving, not by clicking. Recommend a manual pass in Account Settings.
- **Unverified assumption**: the UTF-8 BOM (for Excel and "Padrón"). No primary source found; documented in the spec.
- OWASP's tab-prefix mitigation leaves a tab in neutralised cells (documented caveat); JSON is the lossless format.
- Scope decisions (stated in the spec): images excluded; shared-with-me humidors excluded; CSV omits favorites whose cigar was
  deleted (snapshot-only) — they are in JSON. Whole export is built in memory (fine for personal collections).
- Pre-existing, not changed: `add_to_wish_list` accepts ANY existing cigar id, so a wish list can reference another user's cigar;
  the export then includes that cigar's descriptive fields, exactly as `GET /api/v1/wish_list` already does.
- Legacy `static/profile.html` (not linked from the app, only precached) does not have the buttons.
- The service worker still caches other `/api/` GETs without logout cleanup (MASTER_PLAN #26); only the export path bypasses it.

## Scores
| Category | Score | Grade |
|----------|-------|-------|
| Specification Compliance | 95% | A |
| Best Practices | 93% | A |
| Functionality | 92% | A- |
| Code Quality | 93% | A |
| Security | 96% | A |
| Performance | 88% | B+ |
| Consistency | 94% | A |
| Build Success | 100% | A+ |

**Overall Grade: A (94%)**
