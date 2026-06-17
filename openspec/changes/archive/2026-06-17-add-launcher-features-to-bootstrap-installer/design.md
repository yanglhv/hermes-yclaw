# Design: Multi-app launcher

## Architecture overview

The change adds a `launcher` layer above the existing `bootstrap` /
`update` / `install_script` modules. The latter three are parameterized to
accept `AppDescriptor` and `RepoRef` instead of hardcoded Hermes-specific
values; their stage-by-stage protocol, event emission, and PowerShell
streaming are 100% preserved. The `launcher` layer adds:

- catalog discovery (GitHub Contents API)
- persistent state (installed apps + pending updates)
- three-tier repo URL resolution
- silent default flow
- new Tauri commands

The single Tauri event channel `"bootstrap"` is reused unchanged in shape;
only an `appId` discriminator is added to the payload. The frontend store
routes events to per-app state atoms and re-derives the legacy `$bootstrap`
atom for the existing 4 routes.

## Module layout (final)

```
src-tauri/src/
├── main.rs              unchanged
├── lib.rs               ★ modified: setup hook dispatches on CLI flags
├── events.rs            unchanged
├── paths.rs             ★ modified: + launcher_state_path, launcher_config_path
├── install_script.rs    ★ modified: resolve() accepts RepoRef
├── powershell.rs        unchanged
├── bootstrap.rs         ★ modified: run_bootstrap accepts AppDescriptor
├── update.rs            ★ modified: run_update accepts AppDescriptor
├── app.rs               ★ new: data model (AppDescriptor, LauncherState, …)
└── launcher.rs          ★ new: catalog, state IO, network probe, pre-download,
                            uninstall, repair, launch, silent default flow
```

```
src/
├── main.tsx              unchanged
├── app.tsx               ★ modified: 7 routes instead of 4
├── store.ts              ★ modified: per-app atoms; existing $bootstrap derived
├── routes/
│   ├── welcome.tsx       preserved; only shown for first_install / repair
│   ├── progress.tsx      preserved; takes appId prop
│   ├── success.tsx       preserved; launches via launchApp(appId)
│   ├── failure.tsx       preserved; retries via applyPendingUpdate(appId)
│   ├── home.tsx          ★ new: app tile grid + pending-update banner
│   ├── app-detail.tsx    ★ new: per-app actions (install/update/repair/uninstall/launch)
│   └── settings.tsx      ★ new: launcher-level repo + diagnostics config
├── components/
│   ├── button.tsx        unchanged
│   ├── app-tile.tsx      ★ new
│   ├── pending-update-banner.tsx  ★ new
│   └── mini-progress.tsx ★ new
├── lib/
│   ├── utils.ts          unchanged
│   └── launcher-mode.ts  ★ new: initial-route resolution
└── styles.css            unchanged
```

## Data model

### `apps/<app-id>/app.json` (per app, in distributor repo)

```jsonc
{
  "schema_version": 1,
  "id": "hermes",
  "display_name": "Hermes Agent",
  "icon": "icons/hermes.png",          // optional, relative to apps/<id>/
  "category": "agent",                  // groups tiles on Home screen
  "default": true,                      // exactly one app may be default
  "script_path": "install.ps1",         // install.sh on Unix
  "install_root": "hermes-agent",       // subdirectory under HERMES_HOME
  "binary": {
    "windows": "apps/desktop/release/win-unpacked/Hermes.exe",
    "macos":   "apps/desktop/release/mac-arm64/Hermes.app",
    "linux":   "apps/desktop/release/linux-unpacked/hermes"
  },
  "uninstall_supported": true,
  "app_settings_url": null,             // optional; opens via opener plugin
  "min_launcher_version": "0.1.0"       // safety net for protocol mismatches
}
```

### `~/.hermes/launcher-state.json` (per user, launcher-managed)

```jsonc
{
  "schema_version": 1,
  "default_app_id": "hermes",
  "installed": {
    "hermes": {
      "install_root": "/Users/foo/.hermes/hermes-agent",
      "installed_commit": "abc1234...",
      "installed_ref_type": "branch",        // "branch" | "commit"
      "installed_ref_name": "main",
      "installed_at": "2026-06-15T10:30:00Z",
      "installed_via": "first_install"       // diagnostics
    }
  },
  "pending_updates": {
    "hermes": {
      "latest_commit": "def4567...",
      "latest_ref_name": "main",
      "status": "ready",                     // avail | downloading | ready | failed
      "downloaded_script": "/Users/foo/.hermes/bootstrap-cache/install-def4567.ps1",
      "downloaded_at": "2026-06-16T09:00:00Z",
      "last_error": null,
      "last_error_at": null
    }
  },
  "last_update_check_at": "2026-06-16T09:00:00Z"
}
```

### `~/.hermes/launcher-config.yaml` (per user, optional)

```yaml
repo:
  owner: "your-name"          # default: BUILD_REPO_OWNER
  name:  "your-repo"          # default: BUILD_REPO_NAME
  ref:   "main"               # default: BUILD_PIN_BRANCH

update:
  check_on_launch: true       # default true
  auto_pre_download: true     # default true
  check_interval_seconds: 3600  # default 3600

ui:
  start_minimized: false      # default false
  show_pending_update_banner: true  # default true
```

### Three-tier repo URL resolution (lowest to highest precedence)

1. Build-time env vars: `BUILD_REPO_OWNER`, `BUILD_REPO_NAME`, `BUILD_PIN_BRANCH`.
2. Runtime env vars: `LAUNCHER_REPO_OWNER`, `LAUNCHER_REPO_NAME`,
   `LAUNCHER_REPO_REF` (or combined `LAUNCHER_REPO_OVERRIDE=owner/name@ref`).
3. `~/.hermes/launcher-config.yaml` (parsed with `serde_yaml`).

`launcher::RepoRef::resolve()` returns a single `RepoRef` from whichever
layer wins.

## Tauri commands

### New commands

```
list_available_apps()                       -> Vec<LaunchableApp>
get_app(id)                                 -> LaunchableApp
get_launcher_state()                        -> LauncherState
get_launcher_config()                       -> LauncherConfig
set_launcher_config(yaml)                   -> ()
set_default_app(id)                         -> ()
launch_app(id)                              -> ()
uninstall_app(id, scope)                    -> ()  // scope: "light" | "full"
repair_app(id)                              -> ()
open_app_settings(id)                       -> ()  // via opener plugin
check_for_updates()                         -> UpdateSummary
pre_download_update(id)                     -> ()
apply_pending_update(id)                    -> ()
get_launch_mode()                           -> LaunchMode
```

### Modified commands

```
start_bootstrap(app_id, args)               // +app_id, args unchanged
start_update(app_id)                        // +app_id
launch_hermes_desktop(install_root)         // removed; replaced by launch_app
```

### Unchanged commands

```
cancel_bootstrap, get_bootstrap_status, get_mode,
get_log_path, get_hermes_home, open_log_dir
```

## Event channel

The `"bootstrap"` channel keeps its 5 event types (`manifest` / `stage` /
`log` / `complete` / `failed`). Each event payload grows an optional
`appId` field. Frontend `store.ts` routes events to
`$bootstrapByApp[appId]`; the legacy `$bootstrap` atom becomes a `computed`
view of the currently-selected app's state.

This means the 4 existing routes (`welcome`, `progress`, `success`, `failure`)
need only:
- accept an `appId` prop,
- read `$bootstrap` (now derived),
- call the appropriate parameterized action.

No stage protocol changes; no JSON-result-frame parsing changes; no
line-buffered streaming changes.

## Silent default flow

```
lib.rs setup hook:
  args = parse_cli(std::env::args().skip(1))
  match (mode, force_setup, args):
    (Install, false, ["--settings"])            -> full UI; Home screen
    (Install, false, ["--launch", id])          -> launch_app(id); no window
    (Install, false, [])                        -> run_silent_default()
    (Install, true, _)                         -> full UI; welcome screen
    (Update, _, _)                             -> existing update flow
    _                                          -> full UI; welcome screen
```

```
run_silent_default(repo):
  state = LauncherState::load_or_default()
  if state.installed.is_empty():
    emit LaunchMode { kind: "first_install" }; return
  if !probe_network(repo, 2s):                  // GitHub Contents API HEAD
    launch_app_silent(state.default_app_id); return
  catalog = list_available_apps(repo)
  for app in catalog:
    if state.installed[app.id]?.installed_commit != app.latest_commit:
      pre_download_update(app.id)               // background tokio::spawn
  launch_app_silent(state.default_app_id)
```

`launch_app_silent` always exits the installer process 150ms after a
successful spawn. On failure, it logs to `bootstrap-installer.log`, records
`last_launch_error` in `state.json`, and still exits (silent mode never
raises UI).

## Network probe

Single GET to `https://api.github.com/repos/{owner}/{name}/contents/apps?ref={ref}`
with `timeout=2s`. Success = 200. Anything else (timeout, network error,
4xx, 5xx) = offline. No retries. The probe result is purely a gate for the
update check; it never blocks the launch itself.

## Pending update state machine

```
idle --check--> avail --pre_download--> downloading --success--> ready
                                          |                         |
                                          v                         v
                                       failed <--missing cache-- applying
                                                                  |
                                                                  v
                                                                applied
                                                                (or failed)
```

- `idle`: no `pending_updates[id]` entry.
- `avail`: catalog differs from installed; `state.pending_updates[id].status="avail"`.
- `downloading`: in-flight pre-download. NOT persisted (lost on restart; rescheduled).
- `ready`: `downloaded_script` path verified to exist on next launcher start.
- `failed`: download or apply failed; logged in `state.last_error`.
- `applying`: install.ps1 is running; transient.

`status="ready"` is re-validated on every launcher start. If the script file
is missing, the entry degrades to `failed` and the next launch retries.

## Error recovery matrix

| Failure point | User-visible behavior | Recovery |
|---|---|---|
| Network probe times out | None | Next launch retries |
| Catalog fetch fails | None | Logged; next launch retries |
| Pre-download fails | None | Logged; state.last_error set; next launch retries |
| launch_app fails | None | Logged; state.last_launch_error set; visible in `--settings` |
| state.json corrupt | Home shows "launcher state corrupted" banner | Backed up to `.bak.<ts>`; rebuilt empty |
| app.json corrupt | Tile shows "invalid metadata" | Other apps unaffected |
| min_launcher_version too high | Tile shows "⚠ update launcher" badge | Install button disabled |
| install.ps1 stage fails | failure.tsx (existing UI) | Retry button → applyPendingUpdate |

## CLI flags

```
Hermes-Setup.exe                       # silent default → launch default app
Hermes-Setup.exe --settings            # show full UI (Home)
Hermes-Setup.exe --launch <app-id>     # launch specific app, no UI
Hermes-Setup.exe --update              # existing update flow (unchanged)
Hermes-Setup.exe --repair              # force full UI; repair flow
Hermes-Setup.exe --reinstall           # alias for --repair
```

`force_setup_from_args` keeps its existing responsibility (forces full UI
even when default would be silent) so `--repair` works on a working install.

## Frontend routing

```
$route:
  'welcome'      first_install / --repair / --reinstall
  'home'         --settings (default)
  'app-detail'   drill-down from Home tile
  'settings'     launcher-level config
  'progress'     install/update in flight (appId-bound)
  'success'      install/update complete (appId-bound)
  'failure'      install/update failed (appId-bound)

$currentAppId: string | null
$bootstrap: computed from $bootstrapByApp[$currentAppId]
```

`lib/launcher-mode.ts::resolveInitialRoute()` queries `get_launch_mode()`
once at startup and returns the appropriate route.

## Testing strategy

Per AGENTS.md:
- Rust: `#[cfg(test)]` modules; `tempfile` for fs; no live network.
- TS: vitest, mocked `invoke`.
- No change-detector tests.

New test matrix (full table in `tasks.md`):
- `app.rs`: parse_app_json accepts/rejects schemas; LauncherState round-trip
- `paths.rs`: launcher_state_path / launcher_config_path per-OS
- `install_script.rs`: parameterized URL construction
- `launcher.rs`: RepoRef resolution precedence; probe_network with
  mocked reqwest; pending_update state transitions
- `bootstrap.rs` / `update.rs`: existing tests still pass; AppDescriptor
  threaded through correctly
- `lib/launcher-mode.ts`: each LaunchMode.kind → correct initial route
- `store.ts`: per-app event routing; new actions invoke correct params

## Implementation milestones

```
M1 — Parameterization (no user-visible change)              ~2 person-days
M2 — Multi-app backend skeleton                             ~3 person-days
M3 — Silent default launch + CLI flags                      ~2 person-days
M4 — Frontend: store + new routes + components              ~4 person-days
M5 — Pending update flow                                   ~2 person-days
M6 — Uninstall + repair + settings editor                   ~2 person-days
───────────────────────────────────────────────────────────────────────────
TOTAL                                                       ~15 person-days
```

Minimum demonstrable (M1+M2+M3+M4) = ~11 person-days.
Full feature set = ~15 person-days.

Dependency graph:
- M1 must precede all others (parameterization is the foundation).
- M2 is independent of M3/M4/M5 (pure backend).
- M3 depends on M2.
- M4 depends on M2 (consumes new commands).
- M5 depends on M3 + M4.
- M6 depends on M4.

## Files touched

### Modified

```
apps/bootstrap-installer/src-tauri/src/lib.rs              ~80 LOC
apps/bootstrap-installer/src-tauri/src/paths.rs           ~10 LOC
apps/bootstrap-installer/src-tauri/src/install_script.rs   ~30 LOC
apps/bootstrap-installer/src-tauri/src/bootstrap.rs        ~50 LOC
apps/bootstrap-installer/src-tauri/src/update.rs           ~80 LOC
apps/bootstrap-installer/src/store.ts                      ~150 LOC
apps/bootstrap-installer/src/app.tsx                       ~20 LOC
apps/bootstrap-installer/src/routes/welcome.tsx            ~5 LOC
apps/bootstrap-installer/src/routes/progress.tsx           ~10 LOC
apps/bootstrap-installer/src/routes/success.tsx            ~10 LOC
apps/bootstrap-installer/src/routes/failure.tsx            ~10 LOC
```

### New

```
apps/bootstrap-installer/src-tauri/src/app.rs             ~250 LOC
apps/bootstrap-installer/src-tauri/src/launcher.rs         ~600 LOC
apps/bootstrap-installer/src/routes/home.tsx               ~120 LOC
apps/bootstrap-installer/src/routes/app-detail.tsx         ~180 LOC
apps/bootstrap-installer/src/routes/settings.tsx           ~150 LOC
apps/bootstrap-installer/src/components/app-tile.tsx       ~100 LOC
apps/bootstrap-installer/src/components/pending-update-banner.tsx ~40 LOC
apps/bootstrap-installer/src/components/mini-progress.tsx  ~60 LOC
apps/bootstrap-installer/src/lib/launcher-mode.ts         ~30 LOC
```

Net addition: ~1700 LOC plus ~365 LOC of modifications to existing files.