# Ignore List for False Security Alerts

## Known False Positives

### Rust Version "2.0" Alert
**Status**: FALSE POSITIVE - Ignore permanently

**Details**:
- Alert claims "Rust 2.0" is available
- **Rust 2.0 does not exist** as of December 2025
- Current stable Rust is in the 1.8x series (e.g., 1.83)
- The Dockerfile uses `rust:1-alpine` which auto-tracks latest 1.x stable

**Root Cause**:
The scheduled maintenance bot incorrectly interprets version numbers from other packages (like Alpine 3.21, or seeing "2" in logs) as a Rust major version.

**Action**:
- Do NOT update to "rust:2-alpine" - this image does not exist
- Current `rust:1-alpine` tag is correct and will auto-update to latest stable 1.x
- Configure bot to ignore Rust version alerts or add exception for "Rust 2"

**Last Addressed**: 
- Addressed 3 times prior to December 15, 2025
- Final resolution: December 15, 2025 - Changed from pinned `rust:1.91-alpine` to floating `rust:1-alpine` tag

---

## Legitimate Alerts to Address

### Package Security Vulnerabilities
These are real and should be investigated:
- CVE alerts for Alpine packages (c-ares, openssl, etc.)
- Cargo dependency vulnerabilities from `cargo audit`
- Container scan findings from Trivy

**Resolution Process**:
1. Rebuild Docker image to pull latest Alpine packages (`apk upgrade` in Dockerfile handles this)
2. For Rust dependencies: Run `cargo update` and test
3. For major updates: Follow `docs/DEPENDENCY_UPDATES_PLAN.md`
