# Humidor — Bug & Code-Quality Analysis

Scope: static analysis (reading only) of the Rust backend, SQL migrations, and the
static JS frontend. No code was changed and nothing was built or run. Line numbers refer
to the state of the files at the time of review.

Priority legend: **HIGH** = security-critical or user-facing breakage; **MEDIUM** =
real bug or notable weakness that can bite in normal use; **LOW** = correctness/quality
issue with limited blast radius.

---

## HIGH

### H1. JWT can be forged when no secret is configured — auth bypass
**Files:** `src/handlers/auth.rs:50-74`, `src/main.rs:94-135`, `src/main.rs:217-234`

Two different code paths resolve the JWT secret and they disagree:

- Startup (`get_or_generate_jwt_secret`, `main.rs:94`) reads a secret, and if none is
  found **auto-generates** a 64-char secret and persists it to `/app/data/jwt_secret`.
  It also reads that persisted file back via `read_secret` (`main.rs:62-71`).
- Actual token signing/verification (`jwt_secret()`, `auth.rs:50`) only looks at
  `JWT_SECRET_FILE`, `/run/secrets/jwt_secret`, and the `JWT_SECRET` env var. It **does
  not** read `/app/data/jwt_secret`. When none of those is set it returns the hard-coded
  constant `"INVALID_SECRET_NOT_CONFIGURED"` (`auth.rs:72`).

Consequence: on any deployment where `JWT_SECRET`/docker-secret is not set (the
"auto-generate" path the code advertises as supported), every token is signed **and
verified** with the publicly-known constant string. Startup validation passes because it
reads the persisted file, but the running server authenticates with the placeholder.
Anyone can mint a valid admin JWT by signing `{sub, username, exp, iat}` with
`INVALID_SECRET_NOT_CONFIGURED`. This is a full authentication bypass.

Fix direction: use a single secret-resolution function for both startup and runtime, and
make the "no secret" case a hard startup failure rather than a signing key.

### H2. Unauthenticated database restore wipes/replaces the entire DB
**Files:** `src/routes/backups.rs:71-79`, `src/handlers/backups.rs:202-284`, `src/services/backup.rs:233-259`

`POST /api/v1/setup/restore` is wired with **no authentication** (`create_backup_routes`,
`backups.rs:72`). The handler `setup_restore_backup` accepts an uploaded zip and calls
`restore_backup` → `import_database`, which runs `TRUNCATE TABLE ... RESTART IDENTITY
CASCADE` on `users`, `humidors`, `cigars`, etc. (`services/backup.rs:242-259`) and then
imports rows from the attacker-supplied JSON.

There is no check that setup is still pending (unlike `create_setup_user`, which guards on
admin count). So at any time — including on a fully-provisioned instance — an anonymous
attacker can POST a crafted backup zip and (a) destroy all existing data and (b) seed a
`users` row with `is_admin=true` and a password hash they control, giving themselves admin.
This is remote, unauthenticated, destructive, and a privilege-escalation vector.

Fix direction: require admin auth, or gate strictly on "no admin exists yet" the same way
`create_setup_user` does.

### H3. Stored XSS in cigar rendering (unescaped user data in innerHTML)
**Files:** `static/app.js:1067-1114`, `static/app.js:3520`

The cigar card is built by string-interpolating user-controlled fields directly into
`innerHTML` with no escaping:

- `cigar.notes`, `cigar.wrapper`, `cigar.binder`, `cigar.filler`, `cigar.name`,
  `cigar.humidor_location` at `app.js:1071-1105`.
- `retail_link` injected into both an `href` attribute and link text at `app.js:3520`:
  `retailLinkContainer.innerHTML = \`<a href="${cigar.retail_link}" ...>${cigar.retail_link}</a>\``.

An `escapeHtml()` helper exists (`app.js:531-536`) but is not used in these paths. A cigar
named `<img src=x onerror=alert(document.cookie)>` (or a `retail_link` of
`"><script>...` / `javascript:` URL) executes script when rendered. Because cigars are
rendered on **shared** and **public** humidor pages, an attacker can store a payload in a
humidor shared with a victim (or a public share) and run script in the viewer's session —
where the JWT lives in `localStorage` (`app.js:617`) and is therefore stealable. CSP allows
`script-src 'self' 'unsafe-inline'` (`main.rs:651`), so inline handlers are not blocked.

Fix direction: escape all user-provided fields (use the existing `escapeHtml`) or build
nodes with `textContent`; validate/scheme-check `retail_link`.

### H4. Permissive CORS reflects any origin with credentials enabled
**File:** `src/main.rs:545-557`, `src/main.rs:617-628`

Default mode is `permissive`: `allow_any_origin()` combined with
`allow_credentials(true)`. The auth middleware also accepts a token from an `auth_token`
cookie (`src/middleware/auth.rs:55-64`). Any website can therefore issue credentialed
cross-origin requests against a deployment that uses cookie auth, reading responses. Even
with `localStorage` bearer tokens, shipping `Access-Control-Allow-Credentials: true` with a
reflected origin is a classic misconfiguration and defeats the point of CORS for any
cookie-based flow. The default should not be "any origin + credentials".

### H5. SSRF via the cigar-scrape endpoint
**Files:** `src/handlers/cigars.rs:887-902`, `src/services/mod.rs:62-85`, `src/services/mod.rs:481-500`

`POST /api/v1/cigars/scrape` takes an arbitrary `url` from the request body and the server
fetches it (`fetch_html`, `services/mod.rs:82`) with no allow-list, scheme restriction, or
private-IP filtering. Any authenticated user can make the server issue GET requests to
internal services (`http://169.254.169.254/…` cloud metadata, `http://localhost:PORT/…`,
internal admin panels) and the response text is parsed/returned. This is a server-side
request forgery primitive.

Fix direction: restrict to http(s), resolve and reject private/loopback/link-local
addresses, and ideally allow-list the known scraper domains that are already special-cased.

---

## MEDIUM

### M1. `transfer_ownership` deletes the wrong humidor shares (all-humidors branch)
**File:** `src/handlers/admin/users.rs:697-705`, `749-764`

For the "transfer all humidors" case, the delete-shares query is
`DELETE FROM humidor_shares WHERE humidor_id IN (SELECT id FROM humidors WHERE user_id = $1)`
and it is executed bound to `request.to_user_id` (`users.rs:756`). But the humidors were
just re-assigned to `to_user_id` earlier in the same transaction (`users.rs:701`,
`735-740`). So this selects **all** of the target user's humidors — including ones they
already owned before the transfer — and wipes their shares too, rather than only cleaning up
the transferred humidors' shares. The intent ("remove shares on the moved humidors") is not
what the code does.

### M2. `get_humidor_cigars` queries columns that don't exist — always 500s
**Files:** `src/handlers/humidors.rs:419-427`, `migrations/V5__create_cigars_table.sql:2-23`

The query selects `c.brand, c.size, c.strength, c.origin` (`humidors.rs:420-422`). The
`cigars` table has no such columns — it has `brand_id`, `size_id`, `strength_id`,
`origin_id` (V5). Any call to `GET /api/v1/humidors/:id/cigars` fails with "column does not
exist" and returns 500. The endpoint is effectively dead/broken.

### M3. No password strength/length validation anywhere
**Files:** `src/models/user.rs:19-25,70-99`, `src/handlers/auth.rs:108-147,855-916`, `src/handlers/admin/users.rs:156-249,558-613`, `src/validation.rs`

`CreateUserRequest`, `SetupRequest`, `AdminCreateUserRequest`, `ChangePasswordRequest`, and
`ResetPasswordRequest` carry a raw `password: String` with no minimum length or complexity
check. Setup, admin-create, self change-password, and reset all hash whatever is provided —
including an empty string. There is a `validate_length` helper but it is never applied to
passwords. Weak/empty passwords are accepted.

### M4. SMTP env var names disagree between validation and use
**Files:** `src/main.rs:167-211`, `src/services/email.rs:18-29`, `src/handlers/auth.rs:945-960`

Startup validation checks `SMTP_HOST`, `SMTP_PORT`, `SMTP_USERNAME`, `SMTP_PASSWORD`,
`SMTP_FROM` (`main.rs:182-196`). The email service actually reads `SMTP_USER` and
`SMTP_FROM_EMAIL` (`email.rs:25,27`), and `check_email_config` reads `SMTP_USER`
(`auth.rs:948`). So a deployment can "pass" SMTP validation yet fail to build the mailer
(missing `SMTP_USER`/`SMTP_FROM_EMAIL` → `from_env` errors), or vice versa. Email is
silently misconfigured and password-reset delivery fails while validation claimed success.

### M4b. Service worker caches authenticated API responses and never clears them
**File:** `static/sw.js:91-93,129-137`

API GETs are cached network-first into `DYNAMIC_CACHE` (`sw.js:92,134-136`), including
authenticated responses containing a user's full inventory. Nothing clears these caches on
logout (logout only removes `localStorage` keys, `app.js:635-636`). On a shared device the
next user can be served the previous user's cached data offline, and stale inventory can be
shown after server-side changes.

### M5. DB error strings leaked to clients
**File:** `src/handlers/humidors.rs:62-63,132-133,240-242,331-333,384-386,458-459,479-481`

Several humidor handlers return `"details": e.to_string()` from `tokio_postgres` errors
directly in the JSON body. This contradicts the deliberate error-hiding in `errors.rs`
(`AppError::DatabaseError` is logged, never exposed — `errors.rs:83-97`) and can disclose
schema/内部 details. Error bodies should be generic.

### M6. Many handlers return HTTP 200 on failure
**Files:** `src/handlers/auth.rs:99-104,591-597,671-676,706-709,713-716,740-748`, `src/handlers/cigars.rs:396-401,462-465,511-515,615-618`

Numerous error paths do `Ok(warp::reply::json(&json!({"error": ...})))` with no status
override, so the client receives `200 OK` with an error body. Examples: setup-status DB
failure (`auth.rs:99`), `get_current_user` failure (`auth.rs:595`), change-password "current
password incorrect" (`auth.rs:707`, should be 400/401), `get_cigars`/`create_cigar`/
`get_cigar`/`update_cigar` DB failures. Front-end/API consumers cannot reliably distinguish
success from failure, and the "wrong current password" case looks like success.

### M7. Random-recommendation SQL is built by string interpolation
**File:** `src/handlers/cigars.rs:942-1004`

`preference_value` is spliced into the SQL with only `replace("'", "''")`
(`cigars.rs:943-946`). This happens to be safe only because PostgreSQL's
`standard_conforming_strings` is on by default; it is fragile (any future concat, a
`LIKE`, or a config change reopens injection) and diverges from the parameterized-query
style used everywhere else. Use bound parameters instead of formatting values into the
query.

### M8. `transfer_cigar` holds a transaction open while acquiring more pool connections; ownership check is inconsistent
**File:** `src/handlers/cigars.rs:709-737`

A transaction is started on one pooled connection (`cigars.rs:717`), and then
`verify_cigar_ownership`/`verify_humidor_ownership` each call `pool.get().await` for
**additional** connections (`cigars.rs:80`, `34`) while that transaction is still open. Under
load with a small pool this risks connection-starvation deadlock. Separately, the source-row
query requires `h.user_id = $2` (owner only, `cigars.rs:747`), so a user with shared *edit*
permission passes `verify_cigar_ownership(require_edit=true)` but then gets a "Cigar not
found" from the owner-only fetch — the two access checks disagree.

---

## LOW

### L1. `src/handlers/images.rs` is entirely dead code; uploads are never served
**Files:** `src/handlers/images.rs` (whole file), `src/handlers/mod.rs`, `src/main.rs`

`images.rs` is not declared in `handlers/mod.rs`, so the `upload_image` handler is never
compiled into any route. There is also no static route serving `/uploads` in `main.rs`, yet
image flows reference `/uploads/<file>` (e.g. `images.rs:153`, and `image_url` values in
public shares). The image-upload feature is dead/broken as wired.

### L2. Duplicate, contradictory PORT parsing
**File:** `src/main.rs:339-348` vs `src/main.rs:655-667`

`port` is parsed once with default **9898** (`main.rs:339`) and logged at
`main.rs:344-348`, then re-parsed later with default **3000** (`main.rs:655`) which is the
value actually bound (`main.rs:667`). The first block is dead and the two defaults disagree,
so logs can report a different port than the one the server listens on.

### L3. Dead branch in `transfer_cigar`
**File:** `src/handlers/cigars.rs:776-797`

`let existing_cigar: Option<tokio_postgres::Row> = None;` is immediately followed by
`if let Some(existing_row) = existing_cigar { ... }` — the `Some` branch (merge-into-existing
logic, ~18 lines) is unreachable. Either implement the duplicate-merge or delete the dead
branch and the comment scaffolding.

### L4. Username enumeration via login timing
**File:** `src/handlers/auth.rs:410-427`

When the user doesn't exist, the handler returns immediately (`auth.rs:410`) without running
bcrypt, whereas a wrong password runs a full bcrypt verify. The measurable timing difference
lets an attacker enumerate valid usernames/emails. Consider a dummy verify on the not-found
path.

### L5. `forgot_password` never invalidates prior tokens
**File:** `src/handlers/auth.rs:773-852`

Each request inserts a new reset token without deleting the user's previous ones
(`auth.rs:818-822`). Multiple simultaneously-valid reset tokens per user widen the window for
token abuse. The hourly cleanup only removes tokens older than 30 minutes.

### L6. `delete_cigar` acquires two pool connections for one operation
**File:** `src/handlers/cigars.rs:627-640`

`db` and `db_check` are both fetched from the pool (`cigars.rs:627,636`) though a single
connection suffices; wasteful under load.

### L7. `AuthContext` DB errors are all flattened to `Unauthorized`
**File:** `src/middleware/auth.rs:91-135`

A pool/DB failure while loading the current user is mapped to `AppError::Unauthorized`
(`auth.rs:100,133`), so transient database problems surface to the client as 401 (auth
failure) rather than 503/500. This can cause confusing forced-logout behavior during DB
blips.

### L8. `validate_length` counts bytes, not characters
**File:** `src/validation.rs:7-27`

`value.len()` is the UTF-8 **byte** length, so multi-byte names/notes hit the max sooner than
the character-based limits users would expect (e.g. a 100-"char" limit truncates emoji/CJK
input early). Use `.chars().count()` if the limits are meant to be character counts.

---

## Notes / non-issues considered

- `import_row` uses `json_populate_record` with a bound `$1::json` parameter
  (`services/backup.rs:287-312`), which is parameterized and safe from SQL injection — the
  risk there is H2 (who can invoke it), not injection.
- Backup filename handlers do a lexical `starts_with(backups_dir)` check
  (`handlers/backups.rs:65,166,225`; `services/backup.rs:188`). Because warp path params are
  single URL segments, `/` traversal is not reachable via the download/delete routes, so this
  is defense-in-depth rather than an active traversal bug — but note the check does not
  normalize `..`, so keep it segment-only.
- `rate_limiter` is in-memory per process (`middleware/rate_limiter.rs`), which is fine for a
  single instance but will not limit across replicas.
