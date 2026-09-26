# SCRAPE_SSRF — Spec (MASTER_PLAN #5)

## Current state
`POST /api/v1/cigars/scrape` (authenticated) passed the user URL straight to `reqwest::Client::get`, which
follows up to 10 redirects, accepts any scheme reqwest supports, and reads an unbounded body. Any user could make
the server fetch internal services / cloud metadata (`169.254.169.254`), loopback, or LAN hosts.

## Solution
New `src/services/url_guard.rs`; `CigarScraper::fetch_html` rewritten to use it. No new dependencies
(`reqwest::Url`, `tokio::net::lookup_host`); reqwest 0.12 API (`ClientBuilder::resolve_to_addrs`,
`redirect::Policy::none`, `Response::chunk`) verified via Context7.
- Allow only http/https, no userinfo, ports 80/443 only.
- Resolve host (5s timeout); **every** resolved address must be public unicast (rejects loopback, RFC1918,
  link-local, CGNAT, multicast, reserved, documentation, IPv4-mapped/6to4/Teredo/NAT64 and ULA; IPv6 must be 2000::/3).
  URL parsing already normalises decimal/hex IPv4 forms (`2130706433`, `0x7f.1`).
- Pin the connection to the validated addresses via `resolve_to_addrs` (defeats DNS rebinding between check and use).
- Redirects disabled in reqwest; followed manually (max 5) and each hop re-validated.
- Response body capped at 5 MiB.

## Not changing
- Handler still returns 200 + generic error JSON on failure (#8). Scraper is still in `services/mod.rs` (#18).
- Host-substring dispatch (`url.contains("famous")`) untouched.

## Risks
- Sites on non-standard ports are no longer scrapable (intended).
- If `HTTP(S)_PROXY` env vars are set, reqwest connects via the proxy, which resolves the target itself; the
  IP check then doesn't bind the actual connection. Not set in the project's Docker setup; document if needed.
- Sites that rely on redirects to non-80/443 ports will fail.
