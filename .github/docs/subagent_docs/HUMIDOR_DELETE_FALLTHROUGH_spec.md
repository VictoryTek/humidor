# HUMIDOR_DELETE_FALLTHROUGH — Spec (new finding, not in original MASTER_PLAN)

## Discovery
Found while writing an integration test for #15 (share notifications) that, for the first time, exercised
`DELETE /api/v1/humidors/:id/share/:user_id` through the real HTTP routes for an already-revoked/nonexistent share
(existing tests in `humidor_sharing_tests.rs` only exercised this path by querying the DB directly, never through
the routes with a not-found case). `src/routes/humidors.rs` was NOT touched by any prior item in this session;
confirmed byte-identical to before by diffing against a pre-#15 backup.

## Root cause
1. `delete_humidor`'s route filter had no `warp::path::end()` after `with_uuid()`, so it matched **any** `DELETE`
   request under `/api/v1/humidors/{id}/**`, not only the exact path.
2. warp's `A.or(B)` tries `B` whenever `A`'s `and_then` returns an `Err` — including a business-logic rejection
   from deep inside a handler (e.g. `AppError::NotFound`), not only a routing-level mismatch.
3. `revoke_share`, `delete_public_share` and `revoke_public_share` are all `DELETE` routes positioned *before*
   `delete_humidor` in the `.or()` chain. When any of them correctly rejects with 404 (share already revoked,
   public share already deleted, ...), the request falls through and matches `delete_humidor` instead, which runs
   `DELETE FROM humidors WHERE id = $1 AND user_id = $2`. If the caller owns that humidor — which they must, to
   call `revoke_share` at all — **the entire humidor and every cigar in it is deleted**, reported as
   `200 "Humidor deleted successfully"`.
4. Real-world trigger: `static/app.js`'s `revokeShare()` and `deletePublicShare()` do not disable their button
   while the request is in flight, so a double-click sends the delete twice; the second one 404s and cascades.
   No cross-user exposure: a non-owner's fallthrough into `delete_humidor` 404s (its own `user_id` filter).

## Fix
Add `.and(warp::path::end())` to `delete_humidor` and `update_humidor` (same gap, lower severity: no other route
with a longer path and the `PUT` method exists to cascade into it, but it silently accepted garbage-suffixed PUTs).

## Verified
- Exploit reproduced pre-fix (bisection: removing `delete_humidor` from the `.or()` chain made the same request
  correctly reach `revoke_share` and 404), and closed post-fix.
- Mutation-checked: reverting the fix makes the new regression assertion fail again.
- Real binary: double `DELETE .../share/{fake-user}` → 404, 404; humidor still exists (200 on GET); a legitimate
  `DELETE /api/v1/humidors/{id}` still works (200); `DELETE /api/v1/humidors/{id}/garbage` now 405 instead of
  silently deleting the humidor.
- `scripts/preflight.sh` (integration tests on throwaway Postgres): exit 0, 21 suites.

## Update: organizer routes also fixed (user asked to extend the fix)
The same missing-`path::end()` gap existed in 5 organizer route pairs (`update_brand`/`delete_brand`,
`update_size`/`delete_size`, `update_origin`/`delete_origin`, `update_strength`/`delete_strength`,
`update_ring_gauge`/`delete_ring_gauge` in `src/routes/organizers.rs`) — 10 filters total. These do not cascade
into another handler's destructive action (no longer-path route exists for those resources to fall through from),
so the impact was limited to silently accepting a garbage-suffixed request (e.g. `PUT /api/v1/brands/{id}/garbage`
would rename the brand) rather than rejecting it. Fixed with the same one-line addition to all 10 filters.

Mutation-checked (reverting one filter made a garbage-suffixed `PUT .../garbage` silently rename a brand to
"hacked" again — the exact bug class); `tests/organizer_route_end_tests.rs` covers all 5 organizer types for both
the garbage-suffix rejection and that normal update/delete still work. `scripts/preflight.sh`: exit 0, 22 suites.
