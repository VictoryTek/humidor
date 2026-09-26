# ERROR_HANDLING_P1 — Spec (MASTER_PLAN #8, phase 1 of N)

## Scope (agreed with user)
Phase 1 only: (a) stop leaking raw DB/internal error text to clients, (b) fix 200-OK-on-failure in the **backup**
handlers, after checking the frontend's status-code assumptions. Remaining handlers are later phases.

## Frontend check (done first)
`static/app.js` backup flows already do `if (response.ok) … else showToast(error.message)`; `setup.js` checks
`response.ok`. They are written for real HTTP statuses. Before this change a failed restore/delete returned
`200 {"message":"Error …"}`, so the UI showed "Backup restored successfully. Reloading…" / "deleted successfully".
Nothing in `static/` or `tests/` reads the `details` field of humidor error bodies.

## Changes
- `handlers/backups.rs`: every failure is now an `AppError` rejection → real status via `handle_rejection`:
  missing backup 404, bad filename / no file / corrupt zip 400, everything else 500 with a generic body (detail is
  logged only). DB-pool failures are 500 (`reject::reject()` had been rendering them as **404**). Empty list on
  `list_backups` failure (silently hiding errors) is now a 500.
- `services/backup.rs`: typed `BackupInputError {NotFound, InvalidFilename}` so classification doesn't string-match.
  Corrupt archives are detected via `ZipError` / `serde_json::Error` downcast. `restore_backup` keeps the typed
  filename error (bug found by the new tests: it had been flattened to a String → 500 instead of 400).
- `handlers/humidors.rs`: removed the `"details": e.to_string()` field from 7 DB-error 500 responses (already logged).
  Validation messages (`e.to_string()` on 400s) are user-facing by design and unchanged.

## Deliberately not in this phase
`reject::reject()`→404 misuse in favorites/wish_list/auth (17 sites); 200-on-failure in cigars and organizer
handlers; auth.rs mixed regimes; `AuthContext` flattening DB errors to 401 (#55); response envelope unification.
Each needs its own frontend-compat check.
