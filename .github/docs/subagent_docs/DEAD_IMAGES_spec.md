# DEAD_IMAGES — Spec (MASTER_PLAN #12)

## Finding
`src/handlers/images.rs` (159 lines: multipart `upload_image`, 5 MB cap, `/app/uploads` target) is not just unrouted —
it is not part of the crate at all: `src/handlers/mod.rs` has no `mod images`, so it was never compiled. Nothing in
`src/`, `tests/`, `static/` or the docs references it or an image-upload endpoint. Images actually enter the app as
base64 data URLs (`readAsDataURL` in `static/app.js`) inside the normal cigar/humidor JSON.

## Change
Delete `src/handlers/images.rs`. No `mod.rs` edit needed (it was never declared). No dependency became unused
(`bytes`, `futures`, `tokio` are used elsewhere, e.g. backups).

## Verification
Grep for module/endpoint references (none); `scripts/preflight.sh` (identical result expected since the file was never compiled).

## Side effect on other items
MASTER_PLAN #8's residual count included 7 ad-hoc `{"error"}` sites in this file; they disappear (43 -> 36).
