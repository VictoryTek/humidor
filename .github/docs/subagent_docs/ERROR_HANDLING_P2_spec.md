# ERROR_HANDLING_P2 — Spec (MASTER_PLAN #8, phase 2)

## Problem
`warp::reject::reject()` is equivalent to `not_found()`, so `handle_rejection` renders it as
`404 "Resource not found"`. 17 database-error paths in `favorites.rs` (6), `wish_list.rs` (7) and `auth.rs` (4,
the password-reset flow) used it: a DB failure looked like "missing resource" to clients and monitoring.

## Change
Each site now returns `warp::reject::custom(AppError::DatabaseError(e.to_string()))` → generic 500
("An error occurred processing your request"); the detail is logged by `to_http_response`. The existing
context-specific `tracing::error!` lines are kept. No `reject::reject()` remains in `src/`.

## Frontend check
`static/*.js` only special-cases 401 and 413; no code path depends on 404 for these endpoints (grep).

## Tests
`tests/db_failure_tests.rs`: a pool on an empty database (`humidor_empty_test`, created on demand) makes every
query fail; handlers `get_favorites`, `is_favorite`, `get_wish_list`, `check_wish_list`, `forgot_password` must reject with
`AppError::DatabaseError`, render as 500, and not leak "relation … does not exist". Mutation-checked: reverting one
site to `reject::reject()` fails the test with `Rejection(NotFound)`.

## Not in this phase
200-on-failure in `cigars.rs` and the organizer handlers; auth.rs mixed regimes.
