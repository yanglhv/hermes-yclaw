# Proposal: Complete launcher update, repair, and version-gating flows

## Why

The precursor change `add-launcher-features-to-bootstrap-installer` shipped the
launcher skeleton end-to-end (Home tiles, launch, uninstall, detection,
`--settings` routing, types, CSS), but four core lifecycle flows were left as
stubs "deferred to M5/M6", and one safety gate was never computed. Concretely
(see `archive/2026-06-17-add-launcher-features-to-bootstrap-installer/close-issues.md`):

- `apply_pending_update` resolves the cached script then logs and returns —
  **no bootstrap runs**, so "Install now" leaves the user on a stuck progress
  screen. (V8 not met.)
- `repair_app` logs and returns `Ok` — **Repair never re-installs**. (V16.)
- `pre_download_update` builds a throwaway state and never writes a cached
  script; `status` never reaches `ready`. (V5.)
- `check_for_updates` returns `pending_updates.keys()` only — **no catalog
  fetch / commit comparison**, so updates are never genuinely detected.
- `launcher_too_old` is hardcoded `false` — the `min_launcher_version` gate is
  never evaluated. (V18.)

These are the remaining blockers before the launcher can be considered
feature-complete and safely closeable.

## What Changes

1. **Apply pending update (B1).** `apply_pending_update(id)` runs the cached
   install script through the parameterized bootstrap, emits the standard
   manifest/stage/log events on the `bootstrap` channel, and on success updates
   `state.installed[id]` (commit/ref/timestamp) and clears the pending entry.
   On failure the cached script is retained and the failure screen offers retry.

2. **Repair (B2).** `repair_app(id)` runs the same install flow, using the
   cached pending script when present, otherwise fetching a fresh copy via
   `install_script::resolve()`.

3. **Min-launcher-version gating (B3).** Compute
   `launcher_too_old = current_launcher_version < app.min_launcher_version`
   per `LaunchableApp`. When true, the tile/AppDetail disable Install/Update
   and show the "⚠ update launcher" badge.

4. **Background pre-download (P1).** `pre_download_update(id)` resolves the
   install script, writes it to `$HERMES_HOME/bootstrap-cache/`, and sets
   `pending_updates[id]` to `status="ready"` with `downloaded_script` populated.
   Runs in a background task; never blocks launch.

5. **Update check (P2).** `check_for_updates()` fetches the catalog (when the
   network probe succeeds), compares each installed app's `installed_commit`
   against the catalog's latest commit, and adds changed apps to
   `pending_updates` with `status="avail"`.

## Impact

### Affected files

- `apps/bootstrap-installer/src-tauri/src/launcher/commands.rs` —
  `apply_pending_update`, `repair_app`, `pre_download_update`,
  `check_for_updates`, `list_available_apps` (launcher_too_old).
- `apps/bootstrap-installer/src-tauri/src/launcher/update.rs` — real
  pre-download + resolve-cached-script helpers.
- `apps/bootstrap-installer/src-tauri/src/bootstrap.rs` — apply path reused
  for apply/repair.
- `apps/bootstrap-installer/src/store.ts` — gating flags consumed by UI.
- `apps/bootstrap-installer/src/components/app-tile.tsx`,
  `routes/app-detail.tsx`, `routes/home.tsx` — disable Install/Update when
  `launcher_too_old`.

### Preserved contracts (must NOT change)

- The `bootstrap` event channel + install.ps1/install.sh protocol — reused
  verbatim by apply/repair.
- All launcher state file schemas.
- launch/uninstall/detection work completed in the precursor — unchanged.

## Acceptance Criteria

- **V5** — Bare launch with a newer commit available shows no window; the new
  install script is downloaded in the background and
  `state.pending_updates[id].status` reaches `"ready"` before exit.
- **V8** — From Home, clicking "Install update" enters progress, runs the
  staged bootstrap using the cached script, shows success, and updates
  `state.installed[id]`.
- **V16** — `--repair` (or the Repair button) re-runs the install flow to
  completion.
- **V18** — An app whose `min_launcher_version` exceeds the running launcher
  shows the "⚠ update launcher" badge and a disabled Install button; a
  lower `min_launcher_version` shows no badge and an enabled button.
- **V19** — A pending update whose `downloaded_script` file was deleted
  degrades to `status="failed"` on next launch and is re-attempted.
- Update check: a catalog reporting a newer commit than installed adds the app
  to `pending_updates`; an unchanged commit does not.

## Out of Scope

Deferred to a later hardening change (see close-issues.md P3–P6, T1):

- **P3** Multi-app `app_id` parameterization of `start_bootstrap`/`start_update`
  (Hermes-only path remains).
- **P4** Settings UI restructure to spec `repo`/`update`/`ui` sections +
  diagnostics open-folder buttons.
- **P5** AppDetail collapsible "Recent log" preview.
- **P6** Removal of the stale `launch_hermes_desktop` command.
- **T1** Pre-existing test failures (`lock_probe_paths…` deterministic,
  `light_uninstall…` flaky) — not introduced by launcher work.
