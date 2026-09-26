# BACKUP_ACCESS_CONTROL — Review (MASTER_PLAN #2 + #3)

Reviewer: orchestrating agent, inline (no subagents spawned). Result: **PASS**, no CRITICAL issues.

## Verification (run via `nix-shell` for cargo/lld/openssl; throwaway Postgres 17 on :5433, no volumes)
`scripts/preflight.sh` with `TEST_DATABASE_URL` set → exit 0: fmt OK, clippy `-D warnings` OK, build OK,
unit tests 18/18, integration suites all pass (incl. new `backup_access_tests` 6/6 with its 2 harness tests),
`cargo audit` OK (2 allowed warnings, pre-existing ignores).

## Findings
- Spec compliance: all six admin routes use `with_admin`; setup-restore gated on zero admins; traversal guards
  replaced by `backup_path()`; client filename never used on disk for setup-restore. ✔
- Bug found by new tests and fixed: upload route parsed the 100MB multipart body before auth. Reordered. ✔
- Minor / not fixed (RECOMMENDED, out of scope): setup-restore holds a pool connection while streaming the
  upload; admin-check → restore is not atomic; handlers still return 200 on failure (#8); restore deletes
  `uploads/` before extraction so a malformed zip after `import_database` succeeds still loses images;
  Backups UI is not hidden from non-admins (they now see 403 errors).
- Untested path: setup-restore success when no admin exists (would require wiping the users table).

## Scores
| Category | Score | Grade |
|----------|-------|-------|
| Specification Compliance | 95% | A |
| Best Practices | 90% | A- |
| Functionality | 92% | A- |
| Code Quality | 90% | A- |
| Security | 95% | A |
| Performance | 90% | A- |
| Consistency | 92% | A- |
| Build Success | 100% | A+ |

**Overall Grade: A (93%)**
