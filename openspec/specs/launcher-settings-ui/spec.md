# launcher-settings-ui Specification

## Purpose
TBD - created by archiving change add-launcher-features-to-bootstrap-installer. Update Purpose after archive.
## Requirements
### Requirement: Home screen

The system SHALL render a Home screen in `--settings` mode showing:

- A header section with the launcher wordmark and a footer with
  `HERMES_HOME` path and log path.
- An optional `PendingUpdateBanner` at the top when any app has
  `state.pending_updates` with `status = "ready"`.
- A grid of `AppTile` components, one per registered app, sorted with
  the default app first.

Each tile displays:

- App icon (if `app.icon` is set and resolves)
- `display_name`
- Status badge: `✓ Installed` / `⬇ Update available` / `⬇ Downloading` /
  `○ Not installed` / `⚠ Update launcher`
- Installed ref + commit short SHA (if installed)
- Primary action button: `Install` / `Update` / `Launch`
- A `⋯` menu with: `Repair` / `Uninstall` / `Open app settings` /
  `View details`

#### Scenario: Home shows one tile per registered app

- **Given** the catalog contains Hermes and `myapp`
- **When** the Home screen renders
- **Then** two tiles appear, Hermes first (because it is default).

#### Scenario: Tile shows update badge when pending

- **Given** state has `pending_updates["hermes"].status = "ready"`
- **When** the Hermes tile renders
- **Then** the status badge shows `⬇ Update available` and the primary
  button reads `Update` instead of `Launch`.

#### Scenario: Tile shows launcher-too-old badge

- **Given** Hermes's `min_launcher_version = "1.0.0"` and the running
  launcher is `0.9.5`
- **When** the Hermes tile renders
- **Then** the status badge shows `⚠ Update launcher` and the install
  button is disabled.

### Requirement: AppDetail screen

The system SHALL render an AppDetail screen when a tile is clicked,
showing:

- App icon, display name, category, status badge
- Installed commit + ref (if installed)
- Install root absolute path
- Last install timestamp + `installed_via` diagnostic
- Action buttons: `Install` / `Update` / `Launch` / `Repair` /
  `Uninstall (light)` / `Uninstall (full)` / `Open app settings`
- A collapsible "Recent log" preview showing the last 200 lines from
  the most recent install attempt for this app

`Repair` and `Uninstall (full)` trigger confirmation dialogs before
invoking the Tauri command.

#### Scenario: AppDetail actions are scoped to the app

- **Given** the user is viewing Hermes's AppDetail
- **When** the user clicks `Launch`
- **Then** `launch_app("hermes")` is invoked; `launch_app("myapp")`
  is not.

### Requirement: Settings screen

The system SHALL render a Settings screen reachable from the Home
header's `⋯` menu, showing:

- A `Repo` section with editable fields for `owner`, `name`, `ref`.
  Changes are written to `~/.hermes/launcher-config.yaml` via
  `set_launcher_config`. `Save` and `Reset to defaults` buttons.
- An `Update` section with toggles: `check_on_launch`,
  `auto_pre_download`, and a numeric input for
  `check_interval_seconds`.
- A `Diagnostics` section showing: launcher version, schema version,
  HERMES_HOME path, log path, and `Open log folder` / `Open
  HERMES_HOME` buttons.

#### Scenario: Editing repo and saving

- **Given** Settings screen is open and the user changes `owner` from
  `old-owner` to `new-owner`
- **When** the user clicks `Save`
- **Then** `launcher-config.yaml` is written with the new value;
  the catalog refresh button fetches a new catalog using the new
  repo.

#### Scenario: Reset to defaults

- **Given** `launcher-config.yaml` exists with overrides
- **When** the user clicks `Reset to defaults`
- **Then** `launcher-config.yaml` is deleted; the launcher re-reads
  from build-time constants and env vars on the next `RepoRef::resolve()`.

### Requirement: Pending update banner

The system SHALL render a `PendingUpdateBanner` at the top of the
Home screen when `Object.keys(state.pending_updates).length > 0`. The
banner shows the count of pending updates and offers two buttons:

- `Install now` — applies all pending updates serially, showing the
  progress screen with the current app
- `Later` — dismisses the banner for the current session

The banner is purely UI: it does not modify state directly. `Install
now` invokes `apply_pending_update(id)` for each id.

#### Scenario: Banner shows when updates are pending

- **Given** `pending_updates["hermes"].status = "ready"`
- **When** the Home screen renders
- **Then** the banner reads "1 update available — Install now or apply
  on next launch" with both buttons.

#### Scenario: Banner hidden after Later

- **Given** the banner is visible and the user clicks `Later`
- **When** the user navigates away and back
- **Then** the banner is hidden for this session only; it returns on
  the next launcher start if the pending update still exists.

### Requirement: Per-app event routing

The system SHALL modify `store.ts` so that the `bootstrap` event
listener routes incoming events by `payload.appId`:

- If `payload.appId` is set, update `$bootstrapByApp[payload.appId]`.
- If unset, update `$bootstrapByApp[$currentAppId.get() ?? default_app_id]`.

The legacy `$bootstrap` atom is replaced with a `computed` that returns
`$bootstrapByApp[$currentAppId ?? '']` (or the INITIAL constant if
empty), so the existing 4 routes (welcome/progress/success/failure)
keep their `bootstrap: BootstrapStateModel` prop interface unchanged.

#### Scenario: Event with appId routes to that app

- **Given** `$currentAppId = "hermes"`
- **When** an event with `appId = "myapp"` arrives
- **Then** `$bootstrapByApp["myapp"]` is updated; `$bootstrapByApp["hermes"]`
  is unchanged.

#### Scenario: Event without appId uses current

- **Given** `$currentAppId = "myapp"`
- **When** an event with no `appId` field arrives
- **Then** `$bootstrapByApp["myapp"]` is updated.

### Requirement: New store atoms and actions

The system SHALL add the following atoms and actions to `store.ts`:

Atoms:
- `$launchMode: atom<LaunchMode | null>(null)`
- `$currentAppId: atom<string | null>(null)`
- `$apps: atom<Record<string, LaunchableApp>>({})`
- `$appsList: atom<string[]>([])`
- `$launcherState: atom<LauncherState | null>(null)`
- `$launcherConfig: atom<LauncherConfig | null>(null)`
- `$pendingUpdates: computed($launcherState, ...)`
- `$updateCheckStatus: atom<'idle' | 'checking' | 'done' | 'error'>('idle')`

Actions:
- `launchApp(id)`
- `uninstallApp(id, scope)`
- `repairApp(id)`
- `checkForUpdates()`
- `preDownloadUpdate(id)`
- `applyPendingUpdate(id)`
- `openAppSettings(id)`
- `setDefaultApp(id)`
- `loadLauncherConfig()`
- `saveLauncherConfig(yaml)`

Each action wraps an `invoke(...)` call with the parameter shape
matching the Tauri command.

#### Scenario: launchApp action invokes correct command

- **Given** the user clicks Launch on Hermes's tile
- **When** `launchApp("hermes")` is called
- **Then** the Tauri command `launch_app` is invoked with
  `{ id: "hermes" }`.

### Requirement: Route resolution

The system SHALL, on startup, query `get_launch_mode()` and select the
initial route:

| `LaunchMode.kind`  | Initial route  |
|--------------------|----------------|
| `first_install`    | `welcome`      |
| `settings`         | `home`         |
| `launch`           | `home` (then immediately invokes `launchApp(targetAppId)`) |
| `update`           | `progress`     |
| `silent`           | `home` (defensive; silent mode normally bypasses UI) |

This logic lives in `lib/launcher-mode.ts::resolveInitialRoute()`.

#### Scenario: First install goes to welcome

- **Given** `launcher-state.json` does not exist
- **When** the launcher starts
- **Then** `get_launch_mode()` returns
  `{ kind: "first_install" }` and the welcome screen renders.

#### Scenario: Settings mode goes to home

- **Given** the user double-clicks `Hermes-Setup.exe --settings`
- **When** the launcher starts
- **Then** `get_launch_mode()` returns `{ kind: "settings" }` and the
  Home screen renders.

#### Scenario: --launch <id> opens home then auto-launches

- **Given** the user runs `Hermes-Setup.exe --launch myapp`
- **When** the launcher starts
- **Then** the Home screen renders briefly and `launchApp("myapp")` is
  invoked; the installer exits 150ms after the spawn.

