# SHARE_NOTIFICATIONS — Spec (MASTER_PLAN #15)

## Goal
Email notifications for humidor share/revoke events, reusing `EmailService`. Defaults used (agreed with user):
no opt-out, sent fire-and-forget in the background (never delays/fails the share/revoke request), revoke
notifications on.

## Design
- `services/email.rs`: two new public async methods, `send_humidor_shared_email` and
  `send_humidor_share_revoked_email`, plus a shared `fn send()` (from/to/subject/html -> SmtpTransport) that
  `send_password_reset_email` was left untouched (surgical change — no refactor of working code).
  HTML bodies are built by pure, unit-testable free functions (`shared_email_html`/`revoked_email_html`) so the
  escaping logic can be tested without a network round-trip.
- **XSS-in-email parity with #4**: humidor names and usernames are user-controlled and go into an HTML email body,
  which mail clients render like a browser. A new `EmailService::escape_html` (same 5-character escape as the
  frontend's `escapeHtml`) is applied to both before interpolation.
- `handlers/humidor_shares.rs`: `notify_humidor_shared` / `notify_share_revoked` wrap `tokio::spawn`; on failure or
  missing SMTP config they log (error/debug) and never affect the response. `share_humidor`'s existing "does the
  target user exist" query is extended to also fetch their email (no extra query); `revoke_share` adds one query
  (user email + humidor name) run *before* the delete, since the share row disappearing shouldn't require a second
  lookup — and used only if the delete actually affected a row.
- Sharer's username comes from `auth.username` (already in `AuthContext`, no query needed).

## Frontend
None required: the feature is server-initiated (an email to the other party), no new UI surface for the sharer.

## Tests
- 6 new unit tests in `services::email` (HTML escaping for both templates, permission level/plain names present).
- `tests/share_notification_tests.rs`: share/revoke succeed (201/200) even with no SMTP configured (the test
  environment's actual condition — this is the "not configured" branch, exercised for real, not mocked); sharing
  with a nonexistent user is still 404 (regression: the existence-check query changed shape); revoking a
  nonexistent share is still 404 **and must not delete the humidor** (this assertion is what surfaced the
  HUMIDOR_DELETE_FALLTHROUGH bug — see that doc; unrelated to the notification logic itself, fixed separately).

## Verified but not exercised by automated tests
`SmtpTransport::relay()` (used by `EmailService::send`, unchanged since before #15) does implicit TLS. Probed
directly against a real mailpit container (plaintext SMTP): `send_humidor_shared_email` returns
`Err("... wrong version number ...")` — the same pre-existing limitation flagged in #7's review. Real delivery
therefore cannot be demonstrated against the project's own dev mailpit setup; the notification code path itself
(construction, escaping, fire-and-forget wiring) is fully covered by tests that don't depend on actual delivery
succeeding.

## Out of scope
Opt-out preference; digesting multiple events; retry-on-failure (fire-and-forget, matching `forgot_password`'s
existing convention of logging and moving on).
