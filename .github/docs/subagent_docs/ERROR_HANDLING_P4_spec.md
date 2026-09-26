# ERROR_HANDLING_P4 — Spec (MASTER_PLAN #8, phase 4 — final)

## Problem
Eight `auth.rs` failure paths returned `200 OK` + `{"error": ...}`: `get_setup_status`, `get_current_user`,
`update_current_user`, and five in `change_password`. Notably a **wrong current password returned 200**, so both
profile UIs hit their success branch and toasted "Password changed successfully!" although nothing changed.

## Change
- `get_setup_status`, `get_current_user`, `change_password` (fetch/update DB failures) → 500 (`DatabaseError`).
- `update_current_user` → `AppError::from_db_error` (username/email are UNIQUE → **409** "Username or email already exists").
- `change_password`: wrong current password → **400** "Current password is incorrect" (deliberately NOT 401:
  `makeAuthenticatedRequest` treats any 401 as an expired session, clears the token and redirects to login);
  bcrypt verify/hash failures → 500 with detail logged only.
- Frontend: the four profile / change-password toasts (`profile.js` x2, `app.js` x2) now show `error.message`
  (fallback `error.error`); `AppError` bodies carry a code in `error`.

## Frontend check
`setup status` consumers (two inline scripts in `main.rs`) only read `data.needs_setup`; unaffected by a 500 body.
Profile/password UIs already branch on `response.ok`.

## Result for MASTER_PLAN #8
Leaked error strings: fixed (P1–P2). 200-OK-on-failure: fixed everywhere (regex sweep of `src/handlers/**` finds none).
"Three regimes": reduced to two *body shapes* with correct statuses in both — 43 sites (`auth.rs` 15, `humidors.rs`
21, dead `images.rs` 7) still return ad-hoc `{"error": "<text>"}` rather than `AppError`'s `{"error": CODE,
"message": text}`. Unifying is cosmetic and risky (login.js etc. read `data.error` as text), so it is left as a
low-priority follow-up.
