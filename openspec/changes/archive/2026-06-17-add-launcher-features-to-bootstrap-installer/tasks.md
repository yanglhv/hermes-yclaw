# Tasks: Add launcher features to bootstrap-installer

Implementation tasks broken into 6 milestones. Tasks are ordered by
execution dependency, not by feature area.

## M1 — Parameterization (no user-visible change)

- [ ] 1.1 Refactor `install_script::resolve()` to accept a `RepoRef`
      struct instead of reading hardcoded `NousResearch/hermes-agent`.
      Default the call sites in `bootstrap.rs` and `update.rs` to the
      current hardcoded values via `RepoRef::hardcoded_default()` so
      existing behavior is preserved.
- [ ] 1.2 Add `AppDescriptor` parsing skeleton in `app.rs` (no public
      API yet, just the struct + `parse_app_json` for a single app).
      Hardcode one literal Hermes descriptor for tests.
- [ ] 1.3 Modify `bootstrap::run_bootstrap` to accept `app: &AppDescriptor`
      instead of constructing Hermes-specific paths internally. Use the
      literal Hermes descriptor as default.
- [ ] 1.4 Modify `update::run_update` to accept `app: &AppDescriptor`.
      Use the literal Hermes descriptor as default.
- [ ] 1.5 Add `appId: Option<String>` to every `BootstrapEvent` variant
      in `events.rs`. Default to `None` at emit sites that have no
      app context.
- [ ] 1.6 Run the full Rust test suite. All 906 lines of bootstrap
      tests, 1073 of update, 357 of powershell, 273 of install_script
      must pass unchanged.

## M2 — Multi-app backend skeleton

- [ ] 2.1 Implement full `app.rs`: `AppDescriptor`, `AppJson`,
      `LaunchableApp`, `LauncherState`, `InstalledApp`,
      `PendingUpdate`, `LauncherConfig`, `RepoRef`. Include
      `serde::{Serialize, Deserialize}` derives and `schema_version`
      constants.
- [ ] 2.2 Implement `paths::launcher_state_path()` and
      `paths::launcher_config_path()`. Add unit tests for per-OS
      resolution.
- [ ] 2.3 Implement `launcher::state` module: load/save atomic writes
      via `tmp + rename`, corruption recovery (backup + empty rebuild),
      and existing-install detection.
- [ ] 2.4 Implement `launcher::catalog::list_available_apps(repo)` —
      GitHub Contents API call + per-app `app.json` fetch + merge with
      state. Use `reqwest` with `rustls`.
- [ ] 2.5 Implement `launcher::config::RepoRef::resolve()` with the
      three-tier precedence (build-time → env → yaml).
- [ ] 2.6 Implement `launcher::network::probe_network(repo, timeout)`
      with mocked reqwest for tests.
- [ ] 2.7 Add Tauri commands: `list_available_apps`, `get_app`,
      `get_launcher_state`, `get_launcher_config`, `set_launcher_config`,
      `set_default_app`, `check_for_updates`. Wire into
      `invoke_handler!`.
- [ ] 2.8 Unit tests for each module: schema parsing, repo resolution
      precedence (covers V12 build-time, V13 env var, V14 yaml
      override), network probe with mock, state round-trip,
      corruption recovery.

## M3 — Silent default launch + CLI flags

- [ ] 3.1 Implement `launcher::run_silent_default(app, repo)`: probe →
      catalog → pre-download pending → launch default app → exit.
- [ ] 3.2 Implement `launcher::launch_app_silent(app, app_id)`: spawn
      binary → sleep 150ms → `app.exit(0)`. On failure log and record
      `last_launch_error` in state.
- [ ] 3.3 Replace `bootstrap::launch_hermes_desktop` with generic
      `launcher::launch_app(app, app_id)`. Resolve binary path from
      `AppDescriptor.binary.<os>` joined with
      `state.installed[app_id].install_root`.
- [ ] 3.4 Add Tauri commands: `launch_app`, `get_launch_mode`. Wire
      `launch_hermes_desktop` removal into invoke_handler.
- [ ] 3.5 Extend `lib.rs` setup hook with the dispatch table from
      `launcher-cli-modes`. Fold the macOS-only fast-path into the
      portable silent flow.
- [ ] 3.6 Add `AtomicBool` concurrency guard for `run_silent_default`.
- [ ] 3.7 E2E test V1 (first install shows welcome), V2 (first install
      completes + Launch Hermes → desktop starts + installer exits),
      V3 (silent no-op when no update), V4 (silent skip on no
      network), V7 (`--launch hermes` launches without UI).

## M4 — Frontend store + new routes + components

- [ ] 4.1 Update `store.ts`: add per-app `$bootstrapByApp` map; replace
      `$bootstrap` with a `computed` view; add all new atoms and
      actions from `launcher-settings-ui/spec.md`. Update event
      listener to route by `payload.appId`.
- [ ] 4.2 Update existing 4 routes (`welcome`, `progress`, `success`,
      `failure`) to accept `appId` and use parameterized actions.
      `progress` reads `bootstrap.stages[currentStage].info.title`.
- [ ] 4.3 Implement `lib/launcher-mode.ts::resolveInitialRoute()` that
      queries `get_launch_mode` and returns the right initial route.
      Wire into `app.tsx`.
- [ ] 4.4 Implement `components/app-tile.tsx` — icon, name, status
      badge, primary action, `⋯` menu.
- [ ] 4.5 Implement `components/pending-update-banner.tsx`.
- [ ] 4.6 Implement `components/mini-progress.tsx` for the
      `--launch <id>` mode.
- [ ] 4.7 Implement `routes/home.tsx` with the tile grid + banner +
      footer.
- [ ] 4.8 Implement `routes/app-detail.tsx` with actions and a recent
      log preview.
- [ ] 4.9 Implement `routes/settings.tsx` with repo, update, and
      diagnostics sections.
- [ ] 4.10 Update `app.tsx` to render the new routes from `$route`.
- [ ] 4.11 TS unit tests: launcher-mode route resolution; per-app
      event routing in store; tile state derivation.
- [ ] 4.12 E2E V6 (`--settings` shows Home), V8 (Install update
      progress → success).

## M5 — Pending update flow

- [ ] 5.1 Implement `launcher::pre_download_update(app, repo, id)`:
      resolve script via `install_script::resolve()` and update state
      to `status="ready"`. Emit progress events on the bootstrap
      channel.
- [ ] 5.2 Implement `launcher::apply_pending_update(app, id)`: copy
      cached script if available, run parameterized bootstrap, on
      success update `state.installed[id]` and clear
      `state.pending_updates[id]`.
- [ ] 5.3 Implement `launcher::validate_pending_cache(state)` — on
      startup, downgrade `ready` entries whose cached script file is
      missing to `failed`.
- [ ] 5.4 Add Tauri commands `pre_download_update`, `apply_pending_update`.
      Wire into invoke_handler.
- [ ] 5.5 Wire `PendingUpdateBanner` "Install now" button to iterate
      `apply_pending_update` for each ready entry serially.
- [ ] 5.6 E2E V5 (silent pre-download writes ready to state), V19
      (cache deletion degrades to failed).

## M6 — Uninstall + repair + settings editor

- [ ] 6.1 Implement `launcher::uninstall_app(app, id, scope)` for
      `light` and `full`. Light: call `-Uninstall` if supported, then
      delete install_root, preserve HERMES_HOME. Full: backup
      state.json, delete HERMES_HOME, recreate empty.
- [ ] 6.2 Implement `launcher::repair_app(app, id)` — equivalent to
      `apply_pending_update` but may use a fresh script if no cached
      pending update exists.
- [ ] 6.3 Implement `launcher::open_app_settings(app, id)` via the
      opener plugin. Return `Err` if `app_settings_url` is null.
- [ ] 6.4 Add Tauri commands `uninstall_app`, `repair_app`,
      `open_app_settings`. Wire into invoke_handler.
- [ ] 6.5 Wire Settings screen "Save" to `saveLauncherConfig` (calls
      `set_launcher_config`) and "Reset to defaults" to delete
      launcher-config.yaml.
- [ ] 6.6 E2E V9 (light uninstall preserves rest of HERMES_HOME), V10
      (full uninstall with confirmation), V14 (yaml overrides build-time
      constants), V16 (--repair runs welcome), V17 (corrupt app.json),
      V18 (min_launcher_version gating), V11 (corrupt state.json
      recovery).

## M7 — Verification

- [ ] 7.1 Run `openspec validate add-launcher-features-to-bootstrap-installer --strict`.
      Fix any failures.
- [ ] 7.2 Run full Rust test suite (`cargo test -p hermes-bootstrap`).
      Confirm V20 (existing tests still pass).
- [ ] 7.3 Run full TypeScript test suite (`npm run typecheck` +
      vitest).
- [ ] 7.4 E2E verification of V1 through V20 from `proposal.md`.
- [ ] 7.5 Update `AGENTS.md` if the change introduces new conventions
      worth documenting for future contributors.
- [ ] 7.6 Commit everything to git with conventional commit messages.

## Dependency graph

```
M1 ──┬──> M2 ──┬──> M3 ──┐
     │         │         ├──> M5 ──┐
     │         └──> M4 ──┘         ├──> M6
     │                             │       │
     │                             └───────┴──> M7
```

- M1 must precede all others.
- M2 is independent of M3/M4 (pure backend).
- M3 depends on M2 (uses commands defined in M2).
- M4 depends on M2 (frontend consumes backend commands).
- M5 depends on M3 + M4.
- M6 depends on M4.
- M7 depends on M5 + M6.

## Effort estimate

| Milestone | Person-days |
|-----------|-------------|
| M1        | ~2          |
| M2        | ~3          |
| M3        | ~2          |
| M4        | ~4          |
| M5        | ~2          |
| M6        | ~2          |
| M7        | ~0.5 (verification only) |
| **Total** | **~15.5**   |

Minimum demonstrable (M1+M2+M3+M4) = ~11 person-days, end-to-end
"silent launch + `--settings` Home screen" working.