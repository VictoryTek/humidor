# Humidor — Feature Opportunity Analysis

Analysis date: 2026-07-04
Scope: full codebase — `src/` (Rust/warp backend), `migrations/` (V1–V17), `static/` (vanilla JS PWA), `docs/`, `tests/`.

Method: identified (1) features explicitly deferred in the project's own docs, (2) database columns and services that exist but are underused, (3) gaps versus what users of a cigar-inventory app expect, and (4) infrastructure already wired up but idle.

---

## Summary Table

| # | Feature | Category | Priority |
|---|---------|----------|----------|
| 1 | Smoking journal / tasting sessions with ratings | Expected gap | **High** |
| 2 | Per-user data export (CSV/JSON) — and fix backup scoping | Expected gap + security | **High** |
| 3 | Email notifications for share events | Explicitly deferred | **High** |
| 4 | Scraper: capture price + image (fields already exist) | Half-finished | **Medium** |
| 5 | Collection analytics dashboard | Natural complement | **Medium** |
| 6 | Wish list → "Mark as purchased" flow | Natural complement | **Medium** |
| 7 | Humidity/temperature reading log | Natural complement | **Medium** |
| 8 | Scheduled automatic backups | Idle infrastructure | **Medium** |
| 9 | Share expiration for user-to-user shares | Explicitly deferred | **Low-Medium** |
| 10 | In-app toast notification system | Acknowledged debt | **Low-Medium** |
| 11 | Aging tracker ("time in humidor") | Natural complement | **Low** |
| 12 | Low-stock indicators & restock nudges | Natural complement | **Low** |
| 13 | PWA push notifications / background sync | Idle infrastructure | **Low** |

Also noted: the orphaned `src/services/cigar_scraper.py` (dead code, see §4).

---

## 1. Smoking Journal / Tasting Sessions with Ratings — **HIGH**

**What already exists that makes this natural:**
- `cigars.quantity` with a working decrement path — `update_cigar` even flips `is_active` via a `CASE` expression when quantity hits 0 (`src/handlers/cigars.rs:577`), and the UI sorts out-of-stock cigars last (`ORDER BY is_active DESC`).
- A "recommend a cigar" endpoint (`GET /api/v1/cigars/recommend`, `src/handlers/cigars.rs:904`, documented in `docs/CIGAR_RECOMMENDATION_FEATURE.md`) that suggests what to smoke — but nothing records that you smoked it.
- A "report card" UI surface in the frontend (`reportCardImage`, `static/app.js:6524`) showing per-cigar detail.
- Free-text `notes` on cigars, plus favorites — but no structured rating anywhere.

**Concrete feature:** A `smoking_sessions` table (`id, user_id, cigar_id, smoked_at, rating SMALLINT 1–5, duration_minutes, pairing, notes`) as migration V18. A "Smoke one" button on the cigar card that decrements `quantity` by 1 (reusing the existing update path) and opens a quick session form. New endpoints: `POST /api/v1/cigars/:id/sessions`, `GET /api/v1/sessions` (history view), `GET /api/v1/cigars/:id/sessions`. The recommend endpoint can then rank by past ratings instead of pure random. This is *the* core loop of every competing cigar app (Cigar Scanner, Humidor App) and the single biggest expected-feature gap.

**Effort:** ~3–4 days (one migration, one handler/model/route trio following the exact pattern of `favorites`, plus frontend form). **Value:** very high — turns a static inventory into a usage log and makes the existing recommendation feature meaningfully better.

---

## 2. Per-User Data Export (CSV/JSON) + Backup Scoping Fix — **HIGH**

**What already exists:**
- A full backup service (`src/services/backup.rs`) that already serializes every table to JSON and zips it, with download endpoints (`src/routes/backups.rs`).
- **However**: backup routes use `with_current_user`, not `with_admin` (`src/routes/backups.rs:15` ff.), while `export_database()` dumps **all users' data**. Any authenticated user can download the entire database. This is simultaneously a security issue and evidence that what users actually want here is a *scoped* export.

**Concrete feature:** (a) Gate the existing full-DB backup endpoints behind `with_admin()` (one-line changes; the middleware already exists from FEATURES_TODO Phase 1). (b) Add `GET /api/v1/export?format=csv|json` that exports only the requesting user's humidors, cigars (with organizer names via the existing `CigarWithNames` join), favorites, and wish list. CSV serialization can use the `csv` crate or hand-rolled rows — the query already exists in `get_cigars()`. Frontend: an "Export my collection" button in profile/settings.

**Effort:** ~1–2 days. **Value:** high — collectors track inventory in spreadsheets; export is a table-stakes feature, and the scoping fix removes a real data-exposure hole.

---

## 3. Email Notifications for Share Events — **HIGH**

**What already exists:**
- `docs/FEATURES_TODO.md` lists this three separate times as a planned future enhancement ("Email notifications for share events", "Notification system for share invitations (future enhancement)").
- `EmailService` (`src/services/email.rs`) is fully wired: SMTP config from env, HTML template, working send path — but it has exactly **one** method (`send_password_reset_email`). The infrastructure cost is already paid.
- Share/revoke/permission-change handlers in `src/handlers/humidor_shares.rs` already look up the target user's row (so the email address is in hand) and already trace every event.

**Concrete feature:** Add `send_share_notification(to_email, owner_name, humidor_name, permission_level)` and `send_share_revoked(…)` methods to `EmailService` (copy the existing HTML template structure), and call them fire-and-forget (`tokio::spawn`, as the auth handler already does for reset emails) from `share_humidor()` and `revoke_share()`. Degrade silently when SMTP env vars are absent, matching current behavior.

**Effort:** ~1 day. **Value:** high — sharing is a headline feature (Phase 4) but recipients currently have no way to know they've been granted access.

---

## 4. Scraper: Capture Price and Image — **MEDIUM** (half-finished feature)

**What already exists:**
- A working scrape endpoint (`POST /api/v1/cigars/scrape`, `src/routes/cigars.rs:30`) with per-site scrapers for Cigar Aficionado, Famous Smoke, Cigars International, JR Cigars plus a generic fallback (`src/services/mod.rs`).
- The `Cigar` model has `price`, `image_url`, and `retail_link` columns (V10 added `retail_link` specifically for this flow) — but `ScrapedCigarData` returns only brand/name/size/length/ring_gauge/strength/origin/wrapper. **Price and image are scraped from retail pages the user is already pasting in, yet never extracted.** The `size` field is populated by `new()` but never actually set by any scraper either.
- **Dead code:** `src/services/cigar_scraper.py` (305 lines) is a Python duplicate of the Rust scraper, referenced nowhere in `src/`, the `Dockerfile`, or CI. It should be deleted or moved to a `tools/` directory — right now it misleadingly sits in `src/services/`.

**Concrete feature:** Add `price: Option<String>` and `image_url: Option<String>` to `ScrapedCigarData`; extract price via a `$\d+(\.\d{2})?` regex scoped to the product-info selectors each site scraper already queries, and image via `meta[property="og:image"]` (present on all four supported retailers) with `img[itemprop=image]` fallback. Frontend autofill already maps scraped fields into the add-cigar form — extend the mapping. Remove the orphaned Python file in the same change.

**Effort:** ~1–2 days. **Value:** medium-high — completes the feature's obvious intent (fields exist, pages contain the data) and removes the most tedious manual entry steps.

---

## 5. Collection Analytics Dashboard — **MEDIUM**

**What already exists:**
- The frontend already computes total quantity, unique brands, and total collection value client-side (`updateStats()`, `static/app.js:1123`) — the appetite is proven, the implementation is minimal.
- The data model supports much more: `price`, `purchase_date`, `quantity`, plus normalized `brand_id`/`origin_id`/`strength_id`/`ring_gauge_id`/`size_id` — ideal group-by dimensions with indexes already in place (V7, V11 composite/FK indexes).
- A Prometheus metrics pipeline exists for ops (`src/main.rs:261`) but there are no *user-facing* analytics.

**Concrete feature:** A `GET /api/v1/stats` endpoint returning one aggregated payload: counts and value grouped by brand, origin, and strength; value over time bucketed by `purchase_date` month; quantity per humidor vs. `capacity` (fill percentage — `capacity` is stored today but never compared against contents). Frontend: a "Stats" nav page with simple bar/donut charts (inline SVG, no new dependency, matching the vanilla-JS approach). Server-side aggregation matters once collections outgrow the current fetch-everything-then-reduce approach.

**Effort:** ~3 days. **Value:** medium — high delight for collectors, zero schema changes.

---

## 6. Wish List → "Mark as Purchased" Flow — **MEDIUM**

**What already exists:**
- Wish list is complete as a *list* (V8 table, 5 handlers, notes editing, `retail_link` on cigars for where to buy). `docs/FEATURES_TODO.md` even documents the design intent: "wish lists are for cigars you want to buy."
- But there is no exit path: when you actually buy the cigar, you must manually delete the wish-list entry and separately create/increment an inventory record. The `transfer_cigar` handler (`src/handlers/cigars.rs:696`) already demonstrates the exact quantity-move pattern needed.

**Concrete feature:** `POST /api/v1/wish-list/:id/purchase` with body `{humidor_id, quantity, price?}`: verifies humidor ownership (existing `verify_humidor_ownership` helper), sets/increments the cigar's `humidor_id` and `quantity`, stamps `purchase_date = now()`, removes the wish-list row — all in one transaction. Frontend: a "Purchased ✓" button on wish-list cards opening a small humidor-picker modal.

**Effort:** ~1–2 days. **Value:** medium — closes the intended want→own loop with existing building blocks.

---

## 7. Humidity/Temperature Reading Log — **MEDIUM**

**What already exists:**
- `humidors.target_humidity` with strict validation (50–85%, `src/models/humidor.rs:66`) — the app already *cares* about humidity but stores only the target, never the actual. A target with no actuals is half a feature.
- Humidor cards already display stats blocks (`humidor-card-stats`, `static/app.js:2830`) with room for a current-reading badge.

**Concrete feature:** Migration V18/V19: `humidor_readings (id, humidor_id FK CASCADE, humidity SMALLINT, temperature_f SMALLINT NULL, recorded_at TIMESTAMPTZ)`. Endpoints: `POST /api/v1/humidors/:id/readings` (manual entry — most collectors check an analog/digital hygrometer weekly), `GET /api/v1/humidors/:id/readings?days=90`. UI: quick-log field on the humidor card, latest reading shown next to target with a red/green deviation indicator (>±3% off target), and a simple sparkline on the humidor detail view. Explicitly *not* IoT sensor integration — manual logging only, no rearchitecture.

**Effort:** ~2–3 days. **Value:** medium — this is the literal purpose of a humidor; every dedicated humidor app has it.

---

## 8. Scheduled Automatic Backups — **MEDIUM**

**What already exists:**
- `create_backup()` (`src/services/backup.rs:24`) is a self-contained async function needing only a DB client — trivially callable from a background task.
- The app already runs long-lived tokio infrastructure at startup (`src/main.rs`: migrations, metrics exporter), so a spawned interval task fits the existing pattern. Backups land in a directory already covered by the `humidor_data` volume.

**Concrete feature:** On startup, if `BACKUP_INTERVAL_HOURS` env var is set, `tokio::spawn` a loop that calls `create_backup()` on that interval and prunes to `BACKUP_RETAIN_COUNT` (default 7) using the existing `list_backups()`/`delete_backup()` functions. Document the two env vars in `docker-compose.yml` and `docs/ADMIN_GUIDE.md`.

**Effort:** ~1 day. **Value:** medium — self-hosted users forget manual backups; every function needed already exists.

---

## 9. Share Expiration for User-to-User Shares — **LOW-MEDIUM**

**What already exists:**
- Public link shares **already have** `expires_at` + `never_expires` (V15/V16/V17, `src/models/public_share.rs:10`) with working expiry checks — the pattern is implemented and tested.
- User-to-user shares (`humidor_shares`, V12) have no expiration, and `docs/FEATURES_TODO.md` lists "Share expiration dates" under Technical Debt / future enhancements.

**Concrete feature:** `ALTER TABLE humidor_shares ADD COLUMN expires_at TIMESTAMPTZ NULL`; add `AND (expires_at IS NULL OR expires_at > NOW())` to the permission-check helpers in `src/handlers/humidor_shares.rs` (`can_view_humidor` etc. — single choke point, which is why this is cheap); optional date field in the existing share modal.

**Effort:** ~1 day. **Value:** modest but explicitly on the project's own wishlist, and the consistency with public shares is free.

---

## 10. In-App Toast Notification System — **LOW-MEDIUM**

**What already exists:**
- `docs/FEATURES_TODO.md` Phase 2 notes verbatim: "Alert-based notifications (showNotification function not implemented yet)" — grep confirms `showNotification` still doesn't exist in `static/app.js`; blocking `alert()` calls are used throughout admin/user management.
- The PWA update banner (`showUpdateNotification()`, `static/app.js:170`) already implements a styled non-blocking banner — the CSS/DOM pattern to generalize.

**Concrete feature:** A single `showNotification(message, type)` toast helper (success/error/info, auto-dismiss, stacking container in `index.html`), then replace `alert()` call sites. Pure frontend, no backend work.

**Effort:** <1 day. **Value:** polish, but it unblocks acknowledged debt and every other feature above benefits from it.

---

## 11. Aging Tracker — **LOW**

**What already exists:** `purchase_date` is stored, editable, and rendered as a raw date (`static/app.js:3506`) — but never interpreted. Cigar aging is a core hobbyist concern.

**Concrete feature:** Frontend-only first pass: compute and display "Aging: 1 yr 3 mo" on cigar cards/detail from `purchase_date`, plus an "Age" sort option in the existing sort dropdown (`purchase_date` sorting is already half-present at `static/app.js:1268`). Optional later: a `target_age_months` column and a "ready to smoke" badge.

**Effort:** ~0.5 day for the display pass. **Value:** small but cheap; visible payoff for data users already enter.

---

## 12. Low-Stock Indicators & Restock Nudges — **LOW**

**What already exists:** `quantity` tracking, automatic `is_active=false` at zero, out-of-stock cards sorted last and visually distinct, and `retail_link` for one-click reordering.

**Concrete feature:** A "Low stock" badge when `quantity <= 2` (client-side threshold, or a per-cigar `low_stock_threshold` column later), a "Low stock" filter chip alongside the existing brand/origin/strength filters, and a "Reorder" button on out-of-stock cards that opens `retail_link`. Pairs naturally with #1 (smoking sessions drive quantity down) and #6 (restock → wish list).

**Effort:** ~0.5–1 day. **Value:** small, but nearly free given existing fields.

---

## 13. PWA Push Notifications / Background Sync — **LOW**

**What already exists:** A full service worker (`static/sw.js`) with cache strategies, offline fallback page, and update flow; a complete manifest with icons/screenshots. No `push`, `sync`, or `periodicsync` handlers yet.

**Concrete feature:** Web Push (VAPID) for share-event notifications (complementing #3) — a `push_subscriptions` table, a subscribe endpoint, and the `web-push` crate on send. Honest caveat: this is the only item here that adds a real new dependency surface and browser-permission UX; do it only after #3 proves notification demand. Background *write* sync (queueing offline edits) is explicitly **not** recommended — conflict handling would be a significant architectural change.

**Effort:** ~4–5 days. **Value:** low-medium; nice-to-have for the PWA story.

---

## Deliberately Not Recommended

- **User groups / bulk sharing, activity audit log** (listed in FEATURES_TODO tech debt) — real value only at multi-tenant scale this self-hosted app doesn't target; audit log spans every handler.
- **Offline-first editing** — see #13; conflict resolution is a rearchitecture.
- **IoT hygrometer integrations** — #7's manual logging covers 90% of the value with none of the device-protocol surface.
- **Organizer per-user scoping** — FEATURES_TODO documents global organizers as an intentional, settled design decision.

## Suggested Sequencing

1. Quick wins that unblock everything else: **#10 toasts** (0.5d) → **#2 export + backup scoping fix** (the scoping part is also a security fix — do first) → **#3 share emails**.
2. Core loop: **#1 smoking journal**, then **#4 scraper completion** (+ delete the orphaned Python scraper) and **#6 wish-list purchase flow**.
3. Depth: **#5 analytics**, **#7 humidity log**, **#8 scheduled backups**, then the low-priority items opportunistically.
