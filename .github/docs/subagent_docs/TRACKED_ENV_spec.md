# TRACKED_ENV — Spec (MASTER_PLAN #11)

## Findings (values deliberately not printed)
- `.env` is tracked (in history since the first push, 5 commits) and present on `origin/main`, although `.gitignore` lists it.
- Contents: `DATABASE_URL`, `RUST_LOG`, `PORT`, `ALLOWED_ORIGINS`, `JWT_SECRET`, `JWT_TOKEN_LIFETIME_HOURS`.
  Classification: `JWT_SECRET` is a 41-char dev-style placeholder (passes the 32-char check, so it is *usable*);
  `DATABASE_URL` uses the documented dev defaults against localhost; `ALLOWED_ORIGINS` is localhost-only.
  So: a publicly known signing key + default DB creds that ship with the repo — not leaked production credentials.
- Real exposure: `dotenvy::dotenv()` loads `.env` from the working directory, so anyone who clones and runs from the
  checkout (`cargo run`) authenticates with a published JWT secret (same class of problem as #1; it also silently masked
  the no-secret code path while verifying #1).
- NOT affected: the Docker image (Dockerfile copies only `src/static/migrations`, never `.env`); compose (its `${PORT}` /
  `${RUST_LOG}` substitutions read `.env`, but both values equal compose's own defaults, no `POSTGRES_*` overrides exist,
  and compose never references the secret variables).
- The secret's value also appears in `docs/ANALYSIS_ARCH.md` (quoted while describing the finding).
- `.env.example` already documents every variable (all commented out) and the app works with no `.env` at all, so
  untracking loses nothing for new clones.

## Changes made (working tree only)
- `.gitignore`: add `.env.*` and `!.env.example` (blocks `.env.local`, `.env.production`, ...).
- `scripts/preflight.sh`: first step fails if `.env`, `.env.*` (except `.env.example`) or `secrets/*.txt` is tracked.
  Filter logic verified on simulated file lists; verified against the real repo (correctly fails while `.env` is tracked).

## Required user action (I may not stage/commit per CLAUDE.md)
```
git rm --cached .env          # untracks; your local .env file is kept
```
then commit (`.gitignore`, `scripts/preflight.sh`, the deletion of `.env`, docs). Optional: redact the value in `docs/ANALYSIS_ARCH.md`.

## Not done / recommendations
- History still contains the file. Because the values are dev placeholders, rewriting history is not recommended; instead
  treat that JWT secret and the default DB password as public and never reuse them for a real deployment.
  (If any real instance ever used this `.env`, rotate its JWT secret — all sessions are invalidated — and DB password.)
- Consider running the same tracked-secrets check in CI (`.github/workflows/ci.yml`); not changed here.
