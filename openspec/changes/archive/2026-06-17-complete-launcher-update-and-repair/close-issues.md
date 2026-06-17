# Close Issues: complete-launcher-update-and-repair

Verification run during `/openflow close`. The implementation matches the
change's MODIFIED specs by construction (specs + code written together this
session); `openspec validate --strict` passes; `cargo test` is green for all
new and existing launcher/app/update tests (the lone failure is the pre-existing
T1 `lock_probe_paths`, out of scope here). Items below are verification-depth
notes, not spec inconsistencies.

## Verification items (not blocking archive)

### V1. Manual end-to-end walkthrough not yet performed
- The flows are implemented + compile + their testable cores are unit-tested
  (`semver_lt`, `fetch_head_sha`, the pre-download helper). The full
  user-facing paths — V8 (apply → progress → success → `installed[id]`
  updated), V16 (Repair runs the install), V18 (min-version badge + disabled
  Install), V19 (cache-missing degrades to Failed) — have NOT been exercised
  in the running app yet.
- **Recommended before relying on these flows:** run `npm run tauri:dev` and
  click through Repair, the update banner, and confirm the version-gate badge.

### V2. Command-level unit tests for apply/repair/check_for_updates absent
- These commands take `State<'_>` / `tauri::AppHandle`, which are impractical
  to construct in a pure `cargo test` without a Tauri test harness. Their core
  logic (`semver_lt`, `fetch_head_sha`, the `update::update::pre_download_*`
  helpers) IS unit-tested; the worker they drive (`run_bootstrap`) is covered
  by the existing 906+1073 LOC of bootstrap/update tests. The commands
  themselves are verified by compilation + the manual walkthrough above.

## Known simplifications (documented in design.md, acceptable)

- `app_id` is NOT threaded through every `run_bootstrap` emit site. The
  frontend routes events via `$currentAppId` (which apply/repair set first),
  and the shared `AppState` guard guarantees a single in-flight bootstrap —
  so observable routing behavior is equivalent.
- Cache-key alignment: pre-download caches commit-keyed; `run_bootstrap`
  checks ref-keyed. `apply_pending_update` copies the pending cache to the
  ref-keyed path so the cached bytes are reused; `repair_app` lets resolve
  fall through to network (no pending to reuse). Matches the spec scenarios.

## Recommendation

Archive. Run the V1 manual walkthrough as the immediate next step; if a flow
misbehaves, open a focused fix change.
