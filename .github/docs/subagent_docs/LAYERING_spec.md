# LAYERING — Spec (MASTER_PLAN #10)

## Problem
- `middleware/auth.rs` imported `verify_token` from `handlers::auth` (lower layer depending on a higher one).
- Authorization predicates (`get_user_permission_level`, `can_view_humidor`, `can_edit_humidor`,
  `can_manage_humidor`, `is_humidor_owner`) lived in `handlers/humidor_shares.rs` but were used by
  `handlers/cigars.rs`, `handlers/humidors.rs` and `handlers/public_shares.rs` — no owning module.

## Change (pure move; no logic edits)
- `src/services/jwt.rs`: `Claims`, `jwt_secret`, `generate_token` (now `pub`), `verify_token`.
- `src/services/permissions.rs`: the five predicates, verbatim.
- Importers repointed (`middleware/auth.rs`, `handlers/{auth,cigars,humidors,humidor_shares,public_shares}.rs`);
  imports orphaned in `handlers/auth.rs` removed; one stale section comment updated.
- Resulting direction: routes → handlers → services/models; `middleware` → services/models/errors.

## Verification
fmt/clippy/build; `scripts/preflight.sh`; `grep` for `handlers` imports inside `middleware/`, `services/`, `models/`,
`errors.rs` (none). Behaviour is unchanged by design; integration suites for auth/permissions/sharing exercise the moved code.

## Out of scope
`admin/users.rs` still imports `seed_default_organizers` from `handlers::auth` (same anti-pattern, not listed for #10).
