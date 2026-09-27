# SMOKING_SESSIONS — Spec (MASTER_PLAN #13)

## Decisions (asked of the user)
- Logging a session decrements `cigars.quantity` by 1 (reusing the existing `is_active` flip-to-false-at-0 logic).
- `rating` is optional (1-5 when present); so are `duration_minutes`, `pairing`, `notes`.
- The `recommend` endpoint is NOT changed in this item (follow-up, once ratings exist in volume).

## Schema (migration V18)
```sql
CREATE TABLE smoking_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    cigar_id UUID NOT NULL REFERENCES cigars(id) ON DELETE CASCADE,
    smoked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    rating SMALLINT CHECK (rating IS NULL OR rating BETWEEN 1 AND 5),
    duration_minutes INTEGER CHECK (duration_minutes IS NULL OR duration_minutes > 0),
    pairing TEXT,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_smoking_sessions_user_id ON smoking_sessions(user_id);
CREATE INDEX idx_smoking_sessions_cigar_id ON smoking_sessions(cigar_id);
CREATE INDEX idx_smoking_sessions_smoked_at ON smoking_sessions(smoked_at DESC);
```
`user_id` is the person who logged the session (not necessarily the cigar's owner — a user with edit access to a
shared humidor can log a session too), separate from `cigar_id`'s owner, so a shared collection's sessions are
attributable per-logger. No per-user uniqueness constraint (a cigar can be smoked, and logged, many times).

## Endpoints
- `POST /api/v1/cigars/:id/sessions` — requires edit permission on the cigar's humidor (`verify_cigar_ownership`,
  `require_edit=true`, same rule as other cigar mutations). Body: all fields optional except nothing is required
  (an empty `{}` logs "smoked this, no details"). In one transaction: insert the session row, then run the exact
  `UPDATE cigars ... quantity = quantity - 1, is_active = CASE ...` used by `update_cigar` (decrement by exactly 1,
  clamped at 0, not below). Returns the created session + the cigar's new quantity/is_active.
- `GET /api/v1/cigars/:id/sessions` — requires view permission (`require_edit=false`); list this cigar's sessions,
  newest first.
- `GET /api/v1/sessions` — the requesting user's own logged sessions across all cigars (their `user_id`), newest
  first, joined with cigar name + brand for display; paginated the same way `get_cigars` is (`limit`/`offset`
  query params, default page size matching the existing convention).

## Quantity semantics (matches the existing UI convention exactly)
Decrementing below 0 is rejected (400), not clamped — logging a session on an already-out-of-stock cigar (quantity
0) is a real usage error (the "Smoke one" button should be disabled/hidden client-side once out of stock, matching
how the existing quantity `-` button already only shows while `!isOutOfStock`).

## Frontend
A "Smoke One" button on the cigar card (report-card modal), next to the existing quantity controls, calling the
new endpoint and refreshing the card's quantity/out-of-stock state the same way `updateCigarQuantity` does. No
session-history page/list UI in this pass (out of scope per effort estimate; the read endpoints exist so a future
pass can add a view without another migration).

## Tests
Integration: create session decrements quantity by 1 and flips is_active at 0; logging on a 0-quantity cigar is
400 and does not create a session row (atomicity); optional fields all null is accepted; rating outside 1-5 is
rejected (validation, before touching the DB); permission checks (view-only user gets 403 on POST, can GET);
GET /sessions is scoped to the requesting user; GET /cigars/:id/sessions lists newest-first.
