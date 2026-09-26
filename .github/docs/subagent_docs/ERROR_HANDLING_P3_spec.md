# ERROR_HANDLING_P3 — Spec (MASTER_PLAN #8, phase 3)

## Problem
`cigars.rs` (list/create/get/update/delete/scrape) and all five organizer handlers (brands, sizes, origins,
strengths, ring_gauges) returned `200 OK` with an `{"error": ...}` body on failure. The frontend's `API` /
`OrganizerAPI` wrappers do `if (!response.ok) throw`, so on these paths it silently treated the error object as
data (e.g. `brands = {error: ...}`; a failed scrape showed "Successfully scraped cigar information!").
`get_cigar` also mapped *every* query error to "Cigar not found".

## Schema facts that drive the mapping
- Organizer names are `UNIQUE (user_id, name)` (`ring_gauges`: `(user_id, gauge)`, V13) → the most likely real
  failure is a duplicate name on create/rename → **409**. Cigars have no unique constraints.
- Cigars reference organizers with `ON DELETE SET NULL` → organizer deletes cannot hit FK errors.

## Change
- `AppError::from_db_error(&tokio_postgres::Error, conflict_msg)`: unique violation → 409 Conflict,
  foreign-key violation → 400 ("Referenced record does not exist"), anything else → generic 500 (detail logged).
- Organizers: list/delete DB failure → 500; create/update → `from_db_error` ("<Entity> already exists");
  update/delete of a missing (or not-owned) row → 404 "<Entity> not found" (same message for both → no ownership leak).
- Cigars: list/get/delete DB failure → 500; create/update → `from_db_error`; delete of vanished row → 404;
  scrape failure → 400 "Could not scrape cigar information from that URL" (detail already logged).
- Frontend (one line): the scrape UI now shows `error.message` (falls back to `error.error`); the `AppError` body's
  `error` field is a code such as `BAD_REQUEST`.

## Frontend check
All cigar/organizer calls in `app.js` go through wrappers that throw on `!response.ok`; transfer already reads
`errorData.message`. No code depended on 200 for these failures.

## Not in this phase
`auth.rs`: 8 remaining 200-on-failure sites (`get_setup_status`, `get_current_user`, `update_current_user`,
`change_password` x5). NB: `change_password` "current password incorrect" must NOT be 401 —
`makeAuthenticatedRequest` logs the user out on 401 — needs 400/403 and a `profile.js` check.
