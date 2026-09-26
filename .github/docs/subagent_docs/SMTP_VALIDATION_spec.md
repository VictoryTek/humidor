# SMTP_VALIDATION — Spec (MASTER_PLAN #7)

## Current state
Three places disagreed about what "SMTP configured" means:
- `main.rs::validate_smtp_config` (only when `SMTP_ENABLED=true`, an undocumented flag) required `SMTP_HOST`,
  `SMTP_PORT`, `SMTP_USERNAME`, `SMTP_PASSWORD`, `SMTP_FROM`.
- `EmailService::from_env` (the real mailer, and every doc/compose file) uses `SMTP_USER`, `SMTP_PASSWORD`,
  `SMTP_FROM_EMAIL`; host/port have defaults (smtp.gmail.com / 587).
- `handlers::auth::check_email_config` (drives the forgot-password UI) required non-empty host/user/password
  and ignored `SMTP_FROM_EMAIL`, so it could report "configured" when every send would fail.
Result: the documented config fails startup validation; the validator's own names configure nothing.

## Solution
Single source of truth in `services/email.rs`: `EmailService::validate_env()` (pure `validate_with(lookup)` core):
`SMTP_USER`, `SMTP_PASSWORD`, `SMTP_FROM_EMAIL` required and non-empty; `SMTP_FROM_EMAIL` must parse as a
mailbox; `SMTP_PORT` must parse as u16 if non-empty; reports all problems. Defaults for host/port shared via constants.
- `validate_smtp_config` calls it (still gated by `SMTP_ENABLED`, behaviour otherwise unchanged) and logs effective values.
- `check_email_config` uses `validate_env().is_ok()`.
- `.env.example` documents `SMTP_ENABLED`.

## Behaviour changes (intended)
- Host/port no longer *required* by the validator (mailer has defaults).
- `email_configured` now also requires a valid `SMTP_FROM_EMAIL` and no longer requires `SMTP_HOST`.
- `from_env` itself unchanged.

## Not changing
`SMTP_ENABLED` gating; `SmtpTransport::relay` (implicit TLS) vs. the dev mailpit compose setup (plain SMTP on 1025,
empty credentials) — the dev setup cannot currently send and now reports `email_configured=false` (empty creds), same as before.
