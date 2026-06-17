# Close Issues: add-launcher-features-to-bootstrap-installer

Verification run during `/openflow close`. Per close-phase rules these are
**recorded, not fixed**. Archive is blocked until each is either resolved in a
new change or explicitly accepted as out-of-scope follow-up.

Reference: acceptance criteria are the V1–V20 list in `proposal.md`.

## Status summary

Verified end-to-end working this session: get_launch_mode, `--settings`
routing, existing-install detection (V15), Home tile grid + CSS, AppDetail
actions, launch_app, light/full uninstall (robust), Settings nav, type
alignment. The gaps below are the **not-safely-completed** items.

## Blockers (core flows not end-to-end functional)

### B1. `apply_pending_update` is a stub — Update / "Install now" does nothing real
- `launcher/commands.rs:151` resolves the cached script then logs
  `"apply_pending_update invoked (bootstrap deferred to M6)"` and returns `Ok`.
- No bootstrap is run; `state.installed` is never updated.
- Violates **V8** (Install update → progress → success) and the
  `launcher-update-flow` "Apply pending update" requirement.
- Frontend already routes to `progress` on click, so the user sees a stuck
  progress screen.

### B2. `repair_app` is a stub — Repair does not re-run install
- `launcher/commands.rs:427` logs intent and returns `Ok`; no install runs.
- Violates **V16** (`--repair` / Repair re-runs install) and the
  `launcher-uninstall-repair` "Repair" requirement.

### B3. `launcher_too_old` is hardcoded `false` — min-launcher-version gating missing
- `launcher/commands.rs:64` sets `launcher_too_old: false` unconditionally;
  the comparison `current_version < app.min_launcher_version` is never computed.
- Violates **V18** and `launcher-app-registry` "Min-launcher-version Gating".

## Partial / risky

### P1. `pre_download_update` does not actually cache a script to `status="ready"`
- `launcher/commands.rs:120` builds a throwaway `tmp_state` and calls a helper,
  but the real download-to-cache + `downloaded_script` path is not wired.
- Violates **V5** and `launcher-update-flow` "Background pre-download".

### P2. `check_for_updates` does not compare installed commits vs catalog
- `launcher/commands.rs` returns `pending_updates.keys()` only — no catalog
  fetch / commit comparison, so updates are never genuinely "detected".
- Violates `launcher-update-flow` "Update check".

### P3. `start_bootstrap` / `start_update` are not parameterized on `app_id`
- Frontend (`store.ts` startInstall) invokes `start_bootstrap` with only `args`,
  no `app_id`; the Hermes-only path works, but multi-app install/update is not
  wired through the commands.
- Violates `launcher-multi-app-install` "Parameterized bootstrap/update".

### P4. Settings UI does not match spec structure
- `routes/settings.tsx` exposes `preferred_channel` / `auto_update` /
  `skip_network_probe` / `repo_override`, not the spec's `repo`/`update`/`ui`
  sections; no Diagnostics open-folder buttons, log path, or schema version.
- Partial coverage of `launcher-settings-ui` "Settings screen".

### P5. AppDetail missing collapsible "Recent log" preview
- `launcher-settings-ui` "AppDetail screen" requires a last-200-lines log
  preview; not implemented in `routes/app-detail.tsx`.

### P6. Stale `launch_hermes_desktop` still registered
- Spec M3.3 says replace it with generic `launch_app`; both currently coexist
  (`launch_app` added, old command not removed from `invoke_handler`).

## Test / quality

### T1. Pre-existing test failures violate V20 ("existing tests still pass")
- `update::tests::lock_probe_paths_include_desktop_app_payload` — deterministic
  failure (baseline).
- `launcher::uninstall::uninstall_tests::light_uninstall_removes_root_preserves_siblings`
  — flaky (writes to real `~/.hermes` without isolating HERMES_HOME).
- Verified both fail on baseline (git stash), so not introduced this change,
  but V20 is not green.

### T2. Plan / tasks tracking not reconciled
- All 51 checkboxes in `tasks.md` and the plan are unchecked (implementation
  was performed directly, outside the formal build loop). Formal tracking gap
  only — not a code defect.

## Recommendation

Do **not** archive. Open a follow-up change (e.g. `complete-launcher-update-and-repair`)
to resolve B1–B3 (the true blockers) and at least P1–P2 so the update flow is
end-to-end. P3–P6 and T1 can be scoped as later hardening. Once B1–B3 are
green, re-run `/openflow close`.
