# Humidor — Master Plan

Consolidated from `ANALYSIS_ARCH.md`, `ANALYSIS_BUGS.md`, `ANALYSIS_FEATURES.md`. Duplicate
findings (same root cause reported from multiple angles) have been merged into one entry with
all source tags kept for traceability. Ordered: High → Medium → Low; no particular order within
a tier unless noted.

Tags: `[ARCH]` architecture doc, `[BUG]` bug doc, `[FEAT]` feature doc.

---

## High Priority

- [x] 1. JWT secret resolution duplicated & diverged — runtime falls back to the hardcoded
  sentinel `"INVALID_SECRET_NOT_CONFIGURED"`, a full authentication bypass on any deployment
  without an explicit `JWT_SECRET`. **[ARCH 1.2] [BUG H1]**
  Files: `src/main.rs:35-135`, `src/handlers/auth.rs:50-74`
- [ ] 2. Backup endpoints access control gap — `POST /api/v1/setup/restore` has no auth at all
  (anonymous DB wipe + admin takeover); backup list/download use `with_current_user` instead of
  `with_admin`, so any authenticated user can download a full-DB export of every user's data.
  **[BUG H2] [FEAT #2 security part]**
  Files: `src/routes/backups.rs`, `src/handlers/backups.rs`, `src/services/backup.rs`
- [ ] 3. Backup path-traversal guards are lexical no-ops — `starts_with(backups_dir)` on a
  joined `../` path always passes; attacker-controlled multipart filename in upload/setup-restore
  can write outside the backups directory. **[ARCH 4.1]**
  Files: `src/handlers/backups.rs:65-67,163-170,222-229`
- [ ] 4. Stored XSS in cigar rendering — user-controlled fields (`notes`, `wrapper`, `binder`,
  `filler`, `name`, `retail_link`, ...) interpolated unescaped into `innerHTML` on shared/public
  humidor pages; JWT lives in `localStorage` and is stealable. `escapeHtml()` exists but isn't
  used on these paths. **[BUG H3]**
  Files: `static/app.js:1067-1114,3520`
- [ ] 5. SSRF via cigar-scrape endpoint — arbitrary URL fetched server-side with no scheme
  restriction or private/loopback-IP filtering. **[BUG H5]**
  Files: `src/handlers/cigars.rs:887-902`, `src/services/mod.rs:62-85,481-500`
- [ ] 6. Permissive CORS default combines `allow_any_origin()` with `allow_credentials(true)` —
  invalid per the Fetch/CORS spec and a real credential-leak misconfiguration for any cookie-based
  flow. **[ARCH 1.6] [BUG H4]**
  Files: `src/main.rs:545-557,617-628`
- [ ] 7. SMTP startup validation checks env vars the mailer never reads (`SMTP_USERNAME`/
  `SMTP_FROM` vs. actual `SMTP_USER`/`SMTP_FROM_EMAIL`) — a fail-fast validator that certifies
  broken configs as good, or rejects working ones. **[ARCH 1.3] [BUG M4]**
  Files: `src/main.rs:167-211`, `src/services/email.rs:18-29`, `src/handlers/auth.rs:945-960`
- [ ] 8. Three coexisting error-handling regimes across handlers; DB/internal error strings
  leaked to clients in several handlers; many endpoints return `200 OK` on failure so clients
  can't distinguish success from error by status code. **[ARCH 3.1] [ARCH 1.9] [BUG M5] [BUG M6]**
  Files: `src/handlers/humidors.rs`, `src/handlers/backups.rs`, `src/handlers/auth.rs`,
  `src/handlers/cigars.rs`, `src/errors.rs`
- [ ] 9. Dual-crate layout — `main.rs` re-declares the entire module tree privately instead of
  depending on the library crate; doubles compile time and lets bin/lib types (e.g. two `DbPool`
  aliases) drift apart silently. **[ARCH 1.1]**
  Files: `src/main.rs:3-9`, `src/lib.rs:4-10`, `Cargo.toml:6-12`
- [ ] 10. Middleware → handler dependency inversion; authorization predicates
  (`can_view_humidor`, `can_manage_humidor`) live in an unrelated handler file with no owning
  module. **[ARCH 1.4]**
  Files: `src/middleware/auth.rs:3`, `src/handlers/humidor_shares.rs`, `src/handlers/humidors.rs:2`,
  `src/handlers/cigars.rs:659`
- [ ] 11. `.env` containing a real-looking `JWT_SECRET` and DB credentials is tracked in git
  despite being gitignored. **[ARCH 2.1]**
  Files: `.env`, `.gitignore:12`
- [ ] 12. `src/handlers/images.rs` — 159 lines of dead, unrouted multipart image-upload code;
  misleads anyone searching for how uploads actually work (they're base64 JSON, not multipart).
  **[ARCH 2.2] [BUG L1]**
  Files: `src/handlers/images.rs`, `src/handlers/mod.rs`
- [ ] 13. Smoking journal / tasting sessions with ratings — biggest expected-feature gap for a
  cigar-inventory app; new `smoking_sessions` table + endpoints, reusing the existing
  quantity-decrement pattern. **[FEAT #1]**
- [ ] 14. Per-user data export (CSV/JSON) of the requesting user's own collection. **[FEAT #2
  feature part]**
- [ ] 15. Email notifications for share/revoke events — `EmailService` infrastructure already
  paid for, just needs two more methods and fire-and-forget calls from the share handlers.
  **[FEAT #3]**

## Medium Priority

- [ ] 16. `PORT` is parsed twice with two different defaults (9898 vs 3000); the logged port can
  differ from the bound port, breaking the Docker healthcheck. **[ARCH 1.5] [BUG L2]**
  Files: `src/main.rs:339-348,655-667`
- [ ] 17. `serve_index` reads and string-replaces `static/index.html` on every request to inject
  a setup-check script that's duplicated a second time in a hardcoded fallback. **[ARCH 1.7]**
  Files: `src/main.rs:672-736,782-793`
- [ ] 18. A ~500-line scraper implementation lives directly in `services/mod.rs` instead of its
  own module. **[ARCH 1.8]**
  Files: `src/services/mod.rs:11-500`
- [ ] 19. Route-module boundaries don't consistently match handler-module boundaries (four
  different mapping conventions coexist). **[ARCH 2.3]**
- [ ] 20. Committed development debris: `main.rs.backup`, `build-errors.txt`,
  `src/services/cigar_scraper.py` (orphaned Python duplicate of the Rust scraper). **[ARCH 2.4]**
- [ ] 21. Test harness manually re-creates the `wish_list` table with a stale "not embedded yet"
  comment, risking schema drift from the real V8 migration. **[ARCH 4.2]**
  Files: `tests/common/mod.rs:39-60`, `migrations/V8__create_wish_list_table.sql`
- [ ] 22. `base64` crate declared in `Cargo.toml` but never used. **[ARCH 5.1]**
- [ ] 23. `transfer_ownership` ("transfer all humidors" branch) deletes shares for the target
  user's entire humidor collection, not just the transferred ones, because the delete query runs
  after reassignment. **[BUG M1]**
  Files: `src/handlers/admin/users.rs:697-705,749-764`
- [ ] 24. `get_humidor_cigars` selects columns (`c.brand`, `c.size`, etc.) that don't exist on
  the `cigars` table — the endpoint always 500s. **[BUG M2]**
  Files: `src/handlers/humidors.rs:419-427`, `migrations/V5__create_cigars_table.sql`
- [ ] 25. No password strength/length validation anywhere (setup, admin-create, change,
  reset all accept empty/weak passwords). **[BUG M3]**
- [ ] 26. Service worker caches authenticated API GET responses and never clears them on
  logout — a shared device can see the previous user's cached inventory offline. **[BUG M4b]**
  Files: `static/sw.js:91-93,129-137`, `static/app.js:635-636`
- [ ] 27. Random-recommendation SQL built by string interpolation (safe today only because of a
  Postgres default setting) instead of bound parameters. **[BUG M7]**
  Files: `src/handlers/cigars.rs:942-1004`
- [ ] 28. `transfer_cigar` holds a transaction open while acquiring additional pool connections
  (starvation risk under load); its ownership check disagrees with the shared-edit-permission
  check used elsewhere. **[BUG M8]**
  Files: `src/handlers/cigars.rs:709-737`
- [ ] 29. Row extraction inconsistency: positional indices almost everywhere, named columns in
  middleware — positional access breaks silently if a SELECT column list is reordered. **[ARCH 3.2]**
- [ ] 30. Validation is a trait for some request types (cigar/humidor/organizer models) and
  ad-hoc handler code for others (user/auth/share request types — the security-sensitive ones).
  **[ARCH 3.3]**
- [ ] 31. Scraper never captures price or image despite `Cigar` having `price`/`image_url`
  columns and the scraped pages containing both; delete the orphaned `cigar_scraper.py` in the
  same change (see #20). **[FEAT #4]**
- [ ] 32. Collection analytics dashboard — server-side aggregation endpoint (`GET
  /api/v1/stats`) grouped by brand/origin/strength/value-over-time/humidor fill %. **[FEAT #5]**
- [ ] 33. Wish list → "Mark as purchased" flow, closing the want→own loop. **[FEAT #6]**
- [ ] 34. Humidity/temperature manual reading log per humidor (new table + endpoints + simple
  UI trend). **[FEAT #7]**
- [ ] 35. Scheduled automatic backups via an interval env var, reusing existing
  `create_backup`/`list_backups`/`delete_backup`. **[FEAT #8]**

## Low Priority

- [ ] 36. 79 repetitions of the `api/v1` path prefix across route files instead of one shared
  prefix filter. **[ARCH 1.10]**
- [ ] 37. `/metrics` endpoint is unauthenticated. **[ARCH 1.11]**
- [ ] 38. `rustfmt.toml` pins edition 2021 while the crate is edition 2024. **[ARCH 2.5]**
- [ ] 39. Handler naming inconsistency: `_handler` suffix only in `backups.rs` to avoid a
  naming collision. **[ARCH 2.6]**
- [ ] 40. Two re-export conventions coexist: curated `pub use` lists vs. glob exports. **[ARCH 2.7]**
- [ ] 41. Error helper macros (`db_error!`, `validation_error!`, `not_found!`) defined and never
  used. **[ARCH 3.4]**
- [ ] 42. Scattered `#[allow(dead_code)]` masking the dual-crate problem (#9) rather than
  documenting real intent; `with_optional_auth` is genuinely unused. **[ARCH 3.5]**
- [ ] 43. `ScrapedCigarData.size` field is declared but never populated by any scraper. **[ARCH 4.3]**
- [ ] 44. Committed prose instructing automated tools to ignore "Rust 2.0" alerts — fragile
  process-patching that should be configured in the alerting tool instead. **[ARCH 4.5]**
- [ ] 45. Root-level/`docs/` clutter mixes durable reference docs with one-off session
  writeups, with no archive convention. **[ARCH 4.6]**
- [ ] 46. `tokio-test` dev-dependency is unused. **[ARCH 5.2]**
- [ ] 47. `metrics-exporter-prometheus`'s `http-listener` feature is enabled but never used
  (metrics are served through the app's own warp route). **[ARCH 5.3]**
- [ ] 48. `once_cell::Lazy` used for a single static that `std::sync::LazyLock` now covers on
  this toolchain. **[ARCH 5.4]**
- [ ] 49. `rand 0.8` uses APIs renamed/deprecated in the 0.9 line — flag for the pinned-dependency
  migration plan. **[ARCH 5.5]**
- [ ] 50. `zip 0.6` (processes user-uploaded archives) is the oldest and highest-exposure pinned
  dependency — should be first in the documented migration order, not last. **[ARCH 5.6]**
- [ ] 51. Dead unreachable branch in `transfer_cigar` (`existing_cigar` is always `None`).
  **[BUG L3]**
  Files: `src/handlers/cigars.rs:776-797`
- [ ] 52. Username enumeration via login timing (non-existent user skips bcrypt, wrong password
  doesn't). **[BUG L4]**
  Files: `src/handlers/auth.rs:410-427`
- [ ] 53. `forgot_password` never invalidates a user's prior reset tokens, widening the abuse
  window. **[BUG L5]**
  Files: `src/handlers/auth.rs:773-852`
- [ ] 54. `delete_cigar` acquires two pool connections for one operation. **[BUG L6]**
  Files: `src/handlers/cigars.rs:627-640`
- [ ] 55. `AuthContext` flattens all DB errors to `401 Unauthorized`, so DB blips look like
  forced logouts instead of `503`/`500`. **[BUG L7]**
  Files: `src/middleware/auth.rs:91-135`
- [ ] 56. `validate_length` counts UTF-8 bytes, not characters — multi-byte input (emoji/CJK)
  hits limits earlier than users would expect. **[BUG L8]**
  Files: `src/validation.rs:7-27`
- [ ] 57. Share expiration for user-to-user shares (public link shares already have this;
  `humidor_shares` doesn't). **[FEAT #9]**
- [ ] 58. In-app toast notification system to replace blocking `alert()` calls. **[FEAT #10]**
- [ ] 59. Aging tracker — display "time in humidor" computed from `purchase_date`. **[FEAT #11]**
- [ ] 60. Low-stock indicators & restock nudges using existing `quantity`/`retail_link` fields.
  **[FEAT #12]**
- [ ] 61. PWA push notifications / background sync (do only after #15 proves demand; explicitly
  not offline-write sync — that needs a rearchitecture). **[FEAT #13]**

---

## Deliberately not included (per source docs)

- User groups / bulk sharing, activity audit log — real value only at multi-tenant scale this
  self-hosted app doesn't target.
- Offline-first editing — conflict resolution would be a rearchitecture.
- IoT hygrometer integrations — #34's manual logging covers most of the value.
- Organizer per-user scoping — documented as an intentional, settled design decision.
