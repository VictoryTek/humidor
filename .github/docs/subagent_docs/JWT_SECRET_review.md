# JWT_SECRET — Review (MASTER_PLAN #1, reopened)

Reviewer: orchestrating agent, inline. Result: **PASS**.

## Verification
- `scripts/preflight.sh` (integration tests on throwaway Postgres) → exit 0 (fmt, clippy `-D warnings`, build, 32 unit
  tests incl. 4 new JWT tests, all integration suites, `cargo audit`).
- **End-to-end, real binary, vulnerable conditions** (no `JWT_SECRET`, no `.env` in cwd, `/app/data` absent so persistence fails):

  | | before | after |
  |---|---|---|
  | forged sentinel token → `/api/v1/users/self` | 200 | **401** |
  | forged sentinel token → `/api/v1/admin/users` | 200 | **401** |
  | real token from the auto-generated in-memory secret → `/users/self` and `/admin/users` | (worked, but signed with the sentinel) | **200 / 200** |

  The last row matters: deployments relying on an auto-generated secret still work.
- Sweep: the sentinel string now appears only in the regression test.

## Findings / limits
- A run from the repo directory picked up the tracked `.env` (MASTER_PLAN #11), which silently supplied a
  `JWT_SECRET`; the test had to be run from a directory without `.env` to exercise the no-secret path.
- Auto-generated secrets that fail to persist still invalidate tokens on restart (logged as a warning); unchanged.
- **Intermittent test failure observed once** (3 organizer tests returned 500) in one of ~15 full preflight runs; the
  immediate rerun passed and 30 isolated runs of that binary passed. Root cause NOT proven. A real race exists by
  inspection (unserialised `common::tests::*` run `DELETE FROM users` concurrently with `#[serial]` tests), so both are now
  `#[serial]`; 5 subsequent full runs + the final preflight passed. Evidence is consistent with, but does not prove, that
  race as the cause.
- The plan item's file list is stale (`handlers/auth.rs:50-74` moved to `services/jwt.rs` under #10).

## Scores
| Category | Score | Grade |
|----------|-------|-------|
| Specification Compliance | 97% | A+ |
| Best Practices | 94% | A |
| Functionality | 96% | A |
| Code Quality | 94% | A |
| Security | 98% | A+ |
| Performance | 100% | A+ |
| Consistency | 94% | A |
| Build Success | 100% | A+ |

**Overall Grade: A (96%)**
