# CLAUDE.md
**Humidor** — Rust backend web service (REST API) with a static PWA frontend, containerized with Docker/PostgreSQL. Languages: Rust (edition 2024), JavaScript/HTML/CSS (static frontend), SQL (migrations). Frameworks: warp (HTTP), tokio, deadpool-postgres / tokio-postgres, refinery (migrations). Package manager(s): Cargo (crates.io).

## Commands

Build:
- `cargo build`
- `cargo build --release`

Test:
- `cargo fmt -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --lib --verbose` (unit tests, no database needed)
- `cargo test --tests --verbose` (integration tests; needs Postgres: `docker compose up -d humidor_db`, and `TEST_DATABASE_URL` / `DATABASE_URL` / `JWT_SECRET` set as in `.github/workflows/ci.yml`)
- `cargo audit` (governed by `.cargo/audit.toml`; RUSTSEC-2025-0057, RUSTSEC-2026-0258, RUSTSEC-2026-0367/0368/0369 are intentionally ignored; read the comments there before treating one as a new finding)

Preflight (final gate): `scripts/preflight.sh` (fmt, clippy, build, unit tests, `cargo audit`, a tracked-secrets check; runs integration tests only when `TEST_DATABASE_URL` is set)
Fast verify (also run by the Stop hook): `cargo fmt -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --lib`

## Repository

- Key directories:
  - `src/routes/`, `src/handlers/` — warp routes and per-resource handlers (cigars, humidors, users, auth, favorites, wish list, sharing, admin, backups)
  - `src/services/` — business logic (email via `lettre`, backup, brand/cigar scraping)
  - `src/models/`, `src/middleware/` — domain models; auth, rate limiting, metrics
  - `migrations/` — versioned refinery SQL migrations, applied automatically at startup (`src/main.rs`)
  - `static/` — vanilla JS/HTML/CSS frontend, PWA manifest, service worker
  - `tests/` — integration tests (see `tests/README.md`)
  - `docs/` — design, security, and feature documentation; consult before changing sharing, permissions, or the security model
- Architecture: layered REST API — warp routes → handlers → services/models, backed by PostgreSQL through a deadpool pool, with a separately served static PWA frontend
- CI environment: GitHub Actions (`ubuntu-latest`), Postgres 17 service container for tests, Docker Buildx + Trivy for the docker job (`.github/workflows/ci.yml`)
- OS requirements: on Linux, building needs OpenSSL development headers and `pkg-config` (`lettre`'s `tokio1-native-tls` pulls in `openssl-sys`)
- Build layout constraints: `.cargo/config.toml` sets `-fuse-ld=lld` for every target. On a machine without `lld`, build/check can fail at the link step for environment reasons
- Large disk side-effects: `docker compose build` / the multi-stage Dockerfile produces a full image; `target/` grows to multiple GB
- Other constraints: integration tests fail or hang without a live Postgres instance
- Special constraints:
  - Migrations are forward-only and run at startup; do not add hand-rolled destructive reset or rollback logic
  - `secrets/` holds only `*.txt.example` placeholders; real secrets (`secrets/*.txt`, `.env`) are gitignored. Never print, log, or commit their contents

### Project-specific rules

- Several crates are intentionally pinned behind their latest major (see comments in `Cargo.toml` for `warp`, `jsonwebtoken`, `zip`, and `docs/DEPENDENCY_UPDATES_PLAN.md`). Do not bump them outside a task that is explicitly that migration. Documentation lookups for these crates should confirm the pinned major's API, not the latest.
- A build failure mentioning `lld`, or OpenSSL / `pkg-config`, is an environment gap, not a code defect. Report it distinctly instead of treating it as a code failure.
- If a change touches database-facing code (`src/models`, `src/handlers`, `src/services`, `migrations/`), run the integration tests too.
- Run `cargo audit` when dependencies changed.

## Forbidden commands

Never run these. They are also blocked in `.claude/settings.json` where possible.

### Universal (all repositories)

- `git add` (any form)
- `git commit` (any form, including `--amend`)
- `git push` (any form, including `--force`)
- `git stash` (any form, including `pop`, `apply`, `drop`)
- Any other git command that stages, commits, pushes, or stashes changes

What gets staged, committed, pushed, or stashed is the user's decision; the commit message at the end of every change is what the user runs. Read-only git (`status`, `log`, `diff`, `show`) is allowed.

### Project-specific

- `docker compose down -v` / `docker-compose down --volumes` — reason: destroys the named `postgres_data` and `humidor_data` volumes (live Postgres database and uploaded images); permanent data loss. Use instead: `docker compose stop` or `docker compose down` (no volume flags)
- `docker volume rm postgres_data` / `docker volume rm humidor_data` (or compose-prefixed equivalents such as `humidor_postgres_data`) — reason: same irreversible data loss. Use instead: none — ask the user

## How to work

### Task size

State the tier in one line before starting.

- **Tier 1 — ask or explore** (questions, reading code, anything that changes no files): just answer. No commit message.
- **Tier 2 — standard change** (bug fix, small feature, config or docs change, tests; a few files, no new dependency, no change to a public interface, schema, or build/CI config): research → brief plan in chat (goal, assumptions, docs consulted, steps with how each is verified) → implement → verify → preflight → commit message.
- **Tier 3 — major change** (new dependency; many files or modules; public interface, schema, or migration; security-sensitive code; build, CI, or deploy config; hard to reverse): research → full plan in chat (including docs consulted) and **wait for approval** → implement → verify → independent review → preflight → commit message.

If a task fits two tiers, use the larger. If a Tier 2 task grows, stop, say so, and continue as Tier 3. The user can ask for more or less process ("just do it", "full process"); verification, forbidden commands, and the commit message still apply.

### Principles

1. **Think before coding.** State your assumptions. If a request has several valid readings, present them or ask; never pick one silently. If a simpler approach exists, say so.
2. **Simplicity first.** Write the minimum code that solves the problem: no extra features, no abstractions for single-use code, no unrequested configurability, no handling for cases that cannot happen. If 200 lines could be 50, rewrite it.
3. **Surgical changes.** Every changed line must trace to the request. Do not refactor or tidy adjacent code; match the existing style. Remove only the imports, variables, and functions your change orphaned. Mention unrelated dead code instead of deleting it.
4. **Goal-driven.** Turn the task into a check you can run before you start: bug fix → a failing test first, then make it pass; new behaviour → tests for it; refactor → tests pass before and after. For multi-step work, list the steps with "→ verify: …" for each.

### Done means verified

- Do not call work finished until the build and tests have actually run and you have reported the real output. Never write "passes" for something you did not run.
- Final gate: run the preflight command; the exit code must be 0. If none exists, run the build and test commands instead, say so, and offer to create a preflight script (do not create it unasked). A failure means the work is not finished.
- A Stop hook (`.claude/hooks/verify.sh`) re-runs the fast verify command when files have changed and blocks finishing on failure. Treat its report as a failed verification: fix it and re-run.
- Tier 3: before preflight, have the `reviewer` subagent review the diff. Give it the approved plan and the changed files, not your conclusions. If it is unavailable, review `git diff` yourself and say it was a self-review. Fix CRITICAL findings only; list RECOMMENDED ones for the user without implementing them.
- After 2 failed fix cycles (a failed review or preflight, fixed, failed again), STOP and report everything. Do not loop silently.
- Do not create spec, plan, or review files in the repository unless asked.

### Do not guess state

Before saying what is committed, pushed, or up to date, check: `git status`; `git log --oneline origin/$(git branch --show-current)..HEAD` (empty output means fully pushed); `stat <lockfile>`. Never tell the user to push, commit, or update without checking first.

### Research before changing

For Tier 2 and Tier 3, research comes before the plan, and the plan shows what you found.

1. **Read the code first.** Read the files you will change and the nearest existing code that does something similar. Reuse its patterns and helpers.
2. **Check current docs for every external library, framework, CLI, or API the change uses or touches, including ones already in the lock file.** Look up the version in use (the manifest or lock file says which). Use the Context7 MCP if available (resolve the library, then query its docs); otherwise the official documentation. Do not rely on memory for signatures, options, flags, or default behaviour. When a failure involves a library, search its docs and issue tracker for the error before choosing a fix.
3. **Tier 3 only:** for any dependency being added or upgraded, also read its release notes or changelog and check for deprecations and known advisories. When choosing between libraries, compare at least two.
4. **Show it.** The plan has a line `Docs consulted:` naming each source and what it settled. Write `Docs consulted: none needed — <reason>` only when the change uses no external library or API (pure internal logic, comments, docs, formatting). If a lookup fails or the docs contradict the code, say so and ask; do not guess.

### Commit message (final step of every change)

Every change that modifies files ends with a commit message, however small (a one-line typo fix included). It is the last thing in your reply. You never run `git commit`; the user does. Before the message you may add a brief note: RECOMMENDED review items you did not implement, and unrelated issues you noticed.

Format, ready to paste into `git commit`:

```
<type>(<scope>): <description — MAX 72 characters total>

<PARAGRAPH EXPLAINING WHAT CHANGED AND WHY>

Modified Files:
- path/to/file1
- path/to/file2
- path/to/file3

✔ Build successful
✔ Tests passed
✔ Review approved
✔ Preflight passed
```

Valid types: `feat`, `fix`, `chore`, `refactor`, `docs`, `test`, `perf`. Example first line: `fix(network): disable swap on ZFS server roles`

Inside the message: no "Commit Message" heading, no "Edited" summary, no diff statistics, nothing outside the template.

Checkmark lines appear only for checks that actually ran and passed in this session; never include a line for a skipped check. `✔ Review approved` means the Tier 3 review returned PASS (write `✔ Review approved (self-review)` if no independent reviewer was used). Omit the Preflight line if no preflight exists.

After the message, if every check that ran passed, add one line:
> All checks passed. Code is ready to push to GitHub.
