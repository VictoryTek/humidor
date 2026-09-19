# AUDIT_FIX — Review

Changed: `Cargo.lock`, `.cargo/audit.toml` (moved from `audit.toml`), `scripts/preflight.sh` (new).

## Validation (run in nix-shell; `RUSTFLAGS="-C debuginfo=0"` to bypass absent lld)
| Step | Result |
|---|---|
| `cargo fmt -- --check` | exit 0 |
| `cargo clippy --all-targets --all-features -- -D warnings` | exit 0 |
| `cargo build --verbose` | exit 0 |
| `cargo test --lib --verbose` | 6 passed, 0 failed |
| `cargo test --tests --verbose` (throwaway postgres:17) | 12 test binaries, 0 failed (SCRAM auth path exercised) |
| `cargo audit` (cargo-audit 0.22.1, 1251 advisories) | exit 0; 0 vulnerabilities; 1 allowed warning (scc 2.4.0 unsound, dev-dep via serial_test 3.2.0, needs scc 3.x) |
| `scripts/preflight.sh` | exit 0 |

## Findings
- No CRITICAL issues.
- RECOMMENDED (not done, out of scope): update CLAUDE.md references to `audit.toml` -> `.cargo/audit.toml`;
  `serial_test` bump to clear the scc warning; yanked `spin` crate warning seen once (transitive).
- Unrelated pre-existing issue: `audit.toml` at repo root was never read and had an invalid key (fixed by move).

| Category | Score | Grade |
|----------|-------|-------|
| Specification Compliance | 100% | A |
| Best Practices | 95% | A |
| Functionality | 100% | A |
| Code Quality | 95% | A |
| Security | 95% | A |
| Performance | 100% | A |
| Consistency | 95% | A |
| Build Success | 100% | A |

**Overall Grade: A (97%) — PASS**
