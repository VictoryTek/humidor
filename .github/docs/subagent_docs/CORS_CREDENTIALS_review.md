# CORS_CREDENTIALS — Review (MASTER_PLAN #6)

Reviewer: orchestrating agent, inline. Result: **PASS**, no CRITICAL issues.

## Verification
`scripts/preflight.sh` (integration tests on throwaway Postgres) → exit 0: fmt, clippy `-D warnings`, build,
23 unit tests (1 new: `permissive_never_sends_allow_credentials`), all integration suites, `cargo audit`.

## Findings
- Root cause confirmed in warp 0.3.7 source: any-origin reflects `Origin` and `append_common_headers` adds
  `Allow-Credentials: true` whenever `.allow_credentials(true)` is set. Removed from both permissive arms.
- Behaviour for the app's own frontend is unchanged (Bearer header auth).
- Duplicate builder blocks in `main.rs` collapsed into one helper — the change *required* touching both arms.
- Limitation: the unit test exercises the helper, not `main.rs`'s mode selection (which lives in the bin crate, #9).
  Strict mode is untested here and unchanged.
- RECOMMENDED (out of scope): remove/harden the unused `auth_token` cookie path in `middleware/auth.rs`;
  `docs/SECURITY_MODEL.md` still says "Default: Same-origin only" for CORS, which is inaccurate (default is permissive).

## Scores
| Category | Score | Grade |
|----------|-------|-------|
| Specification Compliance | 96% | A |
| Best Practices | 92% | A- |
| Functionality | 94% | A |
| Code Quality | 94% | A |
| Security | 93% | A |
| Performance | 100% | A+ |
| Consistency | 92% | A- |
| Build Success | 100% | A+ |

**Overall Grade: A (95%)**
