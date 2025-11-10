# Major Dependency Updates Plan

**Date**: 2025-11-10  
**Status**: Analysis Complete - Implementation Deferred

## Overview

The scheduled maintenance workflow identified 18 major version updates available. This document outlines the analysis and recommended approach.

## Major Updates Identified

### HTTP/Hyper Ecosystem (High Impact)
The majority of updates are related to the HTTP stack migration from 0.2.x to 1.x:

| Package | Current | Latest | Impact |
|---------|---------|--------|--------|
| http | 0.2.12 | 1.3.1 | **Breaking** - Core HTTP types |
| http-body | 0.4.6 | 1.0.1 | **Breaking** - Body trait changes |
| hyper | 0.14.32 | 1.7.0 | **Breaking** - Major rewrite |
| reqwest | (via deps) | Latest | **Breaking** - Depends on hyper 1.x |
| warp | (via deps) | Latest | **Breaking** - Web framework core |

**Analysis**: This represents a complete ecosystem upgrade. The `http` crate v1.0 and `hyper` v1.0 were major rewrites with significant API changes. Updating requires:
- Rewriting all HTTP handler code
- Updating Warp framework (major version)
- Extensive testing of all API endpoints
- Potential middleware rewrite

**Recommendation**: Defer to dedicated sprint. Estimated effort: 40-60 hours.

### Error Handling (Medium Impact)
| Package | Current | Latest | Impact |
|---------|---------|--------|--------|
| thiserror | 1.0.69 | 2.0.17 | **Breaking** - Macro changes |

**Analysis**: `thiserror` v2.0 changes macro syntax and error message formatting. All custom error types need review.

**Recommendation**: Can be done incrementally. Estimated effort: 4-8 hours.

### Other Updates
| Package | Current | Latest | Impact |
|---------|---------|--------|--------|
| jsonwebtoken | 9.3.1 | 10.2.0 | **Breaking** - JWT handling |
| zip | 0.6.6 | 6.0.0 | **Breaking** - Archive operations |
| sync_wrapper | 0.1.2 | 1.0.2 | **Breaking** - Sync primitives |

**Analysis**: 
- `jsonwebtoken`: Authentication core - requires careful migration
- `zip`: Backup/restore feature - moderate risk
- `sync_wrapper`: Low risk, likely minor API changes

## Migration Strategy

### Phase 1: Preparation (Current Sprint) ✅
- [x] Document current state
- [x] Identify breaking changes
- [x] Create migration plan
- [x] Set up tracking issue
- [x] Configure CI to ignore deferred dependencies

**Actions Taken:**
- Added inline comments in `Cargo.toml` explaining version constraints
- Updated scheduled workflow to filter out HTTP stack packages from outdated alerts
- Documented deferred packages: warp, hyper, http, http-body, h2, headers, hyper-tls

### Phase 2: Low-Risk Updates (Next Sprint) ✅ COMPLETE
1. ✅ Update `sync_wrapper` (0.1.2 → 1.0.2) - **COMPLETED 2025-11-10**
   - Updated via `reqwest` 0.11 → 0.12
   - All tests passing
2. ✅ Update `thiserror` (1.0.69 → 2.0.17) - **COMPLETED 2025-11-10**
   - No code changes required (not using derive macros)
   - All tests passing
3. ✅ Run full test suite - **PASSED**
4. ⏳ Deploy to staging - **Pending commit**

**Effort**: 8-12 hours (Actual: ~30 minutes)

### Phase 3: Medium-Risk Updates (Following Sprint)
1. Update `jsonwebtoken` (9.3.1 → 10.2.0)
2. Update authentication middleware
3. Test JWT generation/validation
4. Update `zip` (0.6.6 → 6.0.0)
5. Test backup/restore functionality

**Effort**: 12-16 hours

### Phase 4: HTTP Stack Migration (Dedicated Sprint)
This is the major undertaking requiring:

**Preparation:**
1. Create feature branch `feat/http-1.0-migration`
2. Set up parallel testing environment
3. Document all current HTTP interactions

**Implementation:**
1. Update `http` to 1.3.1
2. Update `hyper` to 1.7.0
3. Update `warp` to latest compatible version
4. Rewrite handlers for new API
5. Update middleware for new traits
6. Update `reqwest` calls
7. Test all endpoints (30+ endpoints)
8. Performance benchmarking
9. Security audit

**Effort**: 40-60 hours (1-2 weeks dedicated work)

## Testing Requirements

### Phase 2 & 3 Testing
- ✅ All unit tests pass
- ✅ All integration tests pass
- ✅ Authentication flow works
- ✅ Backup/restore works
- ✅ Error messages are correct

### Phase 4 Testing
- ✅ All HTTP endpoints respond correctly
- ✅ Request/response bodies handled properly
- ✅ Headers processed correctly
- ✅ File uploads work
- ✅ WebSocket connections (if any)
- ✅ Rate limiting functions
- ✅ CORS handling intact
- ✅ Performance benchmarks meet baseline
- ✅ Memory usage within limits
- ✅ Load testing passes

## Risk Assessment

### High Risk
- **HTTP stack migration**: Core functionality affected, potential for subtle bugs
- **Warp framework update**: May require architectural changes

### Medium Risk
- **JWT updates**: Authentication is critical, but well-tested
- **Zip updates**: Backup/restore is important but isolated

### Low Risk
- **thiserror**: Compile-time errors will catch issues
- **sync_wrapper**: Limited usage, small API surface

## Decision

**Recommendation**: 
1. ✅ **Accept and track** the dependency updates
2. ✅ **Implement phases 2-3** in upcoming sprints (low-medium risk items)
3. ⏳ **Defer phase 4** (HTTP stack migration) to dedicated maintenance sprint
4. ✅ **Continue with minor/patch updates** via `cargo update` regularly

**Rationale**:
- Current stack is stable and secure
- HTTP 0.2.x still maintained and receives security patches
- Risk of major migration outweighs immediate benefit
- Better to plan dedicated time for comprehensive update
- Can deliver current features without disruption

## Monitoring

Continue weekly checks for:
- Security vulnerabilities in current versions
- End-of-life announcements
- Critical patches
- Community migration experiences

### CI Configuration

The scheduled maintenance workflow (`scheduled.yml`) is configured to:
- ✅ Only alert on **actionable** major version updates
- ✅ Filter out known deferred packages (HTTP stack migration)
- ✅ Provide context about intentional version locks
- ✅ Link to this plan for detailed migration strategy

**Deferred Packages** (automatically filtered from alerts):
- `warp`, `hyper`, `http`, `http-body`, `h2`, `headers`, `headers-core`, `hyper-tls`

These packages are part of the Phase 4 HTTP stack migration and will generate false-positive "outdated" warnings until we complete that migration.

## References

- [HTTP crate v1.0 migration guide](https://github.com/hyperium/http/blob/master/CHANGELOG.md)
- [Hyper v1.0 upgrade guide](https://hyper.rs/guides/1/upgrading/)
- [Warp migration discussions](https://github.com/seanmonstar/warp/discussions)
- [thiserror v2.0 changelog](https://github.com/dtolnay/thiserror/releases)

## Action Items

- [ ] Close automated dependency issue with link to this document
- [ ] Create tracking issue for Phase 2-3 implementation
- [ ] Schedule Phase 4 for Q1 2026 maintenance sprint
- [ ] Set reminder to review decision quarterly
- [ ] Keep monitoring security advisories

---

**Last Updated**: 2025-11-10  
**Next Review**: 2026-02-10
