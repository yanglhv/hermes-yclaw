# Proposal: Transform bootstrap-installer into multi-app launcher

## Why

The current `apps/bootstrap-installer` is a single-purpose Tauri 2 app that
drives `scripts/install.ps1` (Windows) and `scripts/install.sh` (macOS/Linux)
to install Hermes Agent. Its limitations:

1. **Single-app hardcoding**: Every code path is wired to "Hermes Agent". Adding
   another app means forking the installer.

2. **Single launch path on Windows/Linux**: After install, every subsequent
   double-click re-shows the installer UI on Windows and Linux. Only macOS has
   a fast path that opens the already-installed desktop.

3. **Upstream-controlled update cadence**: `install_script.rs:192` hardcodes
   `https://raw.githubusercontent.com/NousResearch/hermes-agent/{ref}/...`. The
   distributor cannot control update timing without forking the repo.

4. **No pre-launch update check**: Every install runs through the welcome UI
   even on a routine re-launch with no changes.

5. **No repair or uninstall surface**: Users with a broken install have to drop
   to a terminal.

## What Changes

Transform `apps/bootstrap-installer` into a general-purpose multi-app launcher
that:

1. **Is the sole entry point for installed apps** — users open "Hermes Setup"
   to launch any registered app. Apps do not expose their own desktop
   shortcuts; the launcher is the only surface that opens them.

2. **Runs silently by default** — bare double-click runs no UI. It performs a
   2-second network probe, compares installed commits to remote manifests,
   pre-downloads updates when available, then launches the default app. The
   window only appears when `--settings` is passed, when no app is installed,
   or when an install/update is in flight.

3. **Manages multiple apps via a public repo** — apps are directories under
   `apps/<app-id>/` in a configurable public repo. Each app ships its own
   `app.json` (metadata) and `install.{ps1,sh}` (install script). The launcher
   enumerates apps via the GitHub Contents API (no auth, public repos only).

4. **Pre-downloads updates in the background** — when an update is detected
   the new install script is downloaded but not executed. The user sees an
   "update available" banner in `--settings` mode and chooses when to apply.
   Pre-downloads never block the launch.

5. **Provides install / update / repair / uninstall / launch per app** from a
   unified Home screen visible only in `--settings` mode.

6. **Three-tier remote repo configuration** — build-time constants, runtime
   env vars, and user-level `~/.hermes/launcher-config.yaml`. This decouples
   the launcher's release cadence from the install-script release cadence.

7. **Generalized API keys stay in apps** — the launcher does not own secrets.
   Apps keep their own secrets. The launcher only provides an "open app
   settings" affordance via the existing `opener` plugin.

## Impact

### Affected files

**Modified (parameterization, no behavioral change for Hermes):**
- `apps/bootstrap-installer/src-tauri/src/bootstrap.rs`
- `apps/bootstrap-installer/src-tauri/src/update.rs`
- `apps/bootstrap-installer/src-tauri/src/install_script.rs`
- `apps/bootstrap-installer/src-tauri/src/lib.rs`
- `apps/bootstrap-installer/src-tauri/src/paths.rs`
- `apps/bootstrap-installer/src/store.ts`
- `apps/bootstrap-installer/src/app.tsx`
- `apps/bootstrap-installer/src/routes/{welcome,progress,success,failure}.tsx`

**New:**
- `apps/bootstrap-installer/src-tauri/src/app.rs`
- `apps/bootstrap-installer/src-tauri/src/launcher.rs`
- `apps/bootstrap-installer/src/routes/{home,app-detail,settings}.tsx`
- `apps/bootstrap-installer/src/components/{app-tile,pending-update-banner,mini-progress}.tsx`
- `apps/bootstrap-installer/src/lib/launcher-mode.ts`

### New persistent state (under `$HERMES_HOME`)

- `launcher-state.json` — installed apps, pending updates, last check time
- `launcher-config.yaml` — optional user overrides for repo URL + update prefs

### Preserved contracts (must NOT change)

- Single Tauri event channel `"bootstrap"` with the existing 5 event types
  (`manifest` / `stage` / `log` / `complete` / `failed`). Only `appId` is
  added to the payload for routing.
- install.ps1 / install.sh protocol: `-Manifest`, `-Stage NAME -NonInteractive
  -Json`, last-line-JSON result frame parsing. 100% reuse.
- HERMES_HOME path resolution: Windows `%LOCALAPPDATA%\hermes`, mac/linux
  `~/.hermes`. Matches Python `get_hermes_home()`, `install.sh`, and the
  Electron desktop's `resolveHermesHome()`.
- styles.css inherits from `apps/desktop/src/styles.css` wholesale.
- Existing 906+1073+357+273 LOC of Rust tests must continue to pass.

## Acceptance Criteria

V1.  First launch with `state.json` absent shows the welcome screen.
V2.  First launch completes install → success → "Launch Hermes" → desktop
     starts and installer exits.
V3.  Subsequent bare launch with online + no newer commit shows NO main
     window and Hermes Desktop appears within 1 second.
V4.  Subsequent bare launch with no network shows NO main window and Hermes
     Desktop appears within 1 second, with no visible error.
V5.  Subsequent bare launch with newer commit available shows NO main window;
     Hermes Desktop appears within 1 second; in the background the new
     install script is downloaded; `state.pending_updates[id].status` reaches
     `"ready"` before installer exit.
V6.  `Hermes-Setup.exe --settings` shows the Home screen listing every catalog
     app with current status.
V7.  `Hermes-Setup.exe --launch <app-id>` launches that app without showing
     the installer window.
V8.  In Home screen, clicking "Install update" enters the progress screen,
     runs the staged bootstrap, and shows success on completion.
V9.  `uninstall_app(id, "light")` invokes the app's `install.ps1 -Uninstall`
     if present, deletes the install root, and PRESERVES the rest of
     `HERMES_HOME` (logs, config, profiles).
V10. `uninstall_app(id, "full")` requires a second user confirmation, then
     deletes `HERMES_HOME` and backs up `launcher-state.json` to
     `launcher-state.json.bak.<unix_ts>`.
V11. A manually corrupted `launcher-state.json` is moved to a timestamped
     backup; an empty state is rebuilt; the installer does not crash.
V12. Changing `BUILD_REPO_OWNER` / `BUILD_REPO_NAME` at compile time causes
     the launcher to fetch the catalog from the new repo.
V13. Setting `LAUNCHER_REPO_OVERRIDE=owner/name@ref` overrides build-time
     defaults for that process.
V14. `~/.hermes/launcher-config.yaml` overrides both build-time constants and
     env vars.
V15. An existing install root whose `state.json` is missing is detected on
     next launch and recorded into a fresh `state.json`.
V16. `Hermes-Setup.exe --repair` (or `--reinstall`) shows the welcome screen
     and re-runs `install.ps1` for the default app.
V17. An `app.json` missing `binary.macos` shows a "launch config missing"
     badge on the tile; the launcher does not crash.
V18. An `app.json` whose `min_launcher_version` is higher than the running
     launcher shows a "⚠ update launcher" badge on the tile; the install
     button is disabled.
V19. A pending update whose `downloaded_script` file has been deleted is
     degraded to `status="failed"` on next launch, with the next launch
     re-attempting the download.
V20. Every existing Rust unit test (`bootstrap.rs`, `update.rs`,
     `powershell.rs`, `install_script.rs`) continues to pass without
     modification other than the parameterization.

## Out of Scope

- **Plugin / trait-based app registry.** Apps are directories in a repo, not
  Rust trait implementations. The current install-script protocol is the
  only install mechanism.
- **API key management inside the launcher.** Apps own their own secrets.
  The launcher exposes `app_settings_url` via the existing `opener` plugin
  and does not read or write `.env` files.
- **Replacing the `bootstrap` event channel.** The single channel with 5
  event types is preserved. Only an `appId` discriminator is added.
- **Cross-launcher migration.** Existing Hermes installs are detected
  automatically and a fresh `launcher-state.json` is written on first
  post-change launch. No backfill script ships.
- **Build-time signing or distribution changes.** The `hermes-setup.manifest`
  (Windows UAC bypass + PerMonitorV2 DPI) is unchanged.
- **macOS-only fast path generalization as a separate feature.** The silent
  default flow IS the generalized fast path; the existing macOS-only block
  in `lib.rs:114` is folded into the new flow.