# Capability: launcher-uninstall-repair

## Purpose

Provides uninstall (light and full) and repair operations per app.
Repair is the same install flow re-run; uninstall has two scopes.

## ADDED Requirements

### Requirement: Light uninstall

The system SHALL, on `uninstall_app(id, "light")`:

1. Look up the app's `install_root` from `state.installed[id]`.
2. If `app.uninstall_supported` is `true`, invoke
   `install.<ext> -Uninstall -AppId <id>` against the cached script for
   the app's current ref.
3. Whether or not the script call succeeded, delete the
   `install_root` directory recursively.
4. Remove the entry from `state.installed`.
5. PRESERVE the rest of `HERMES_HOME` (logs, config, profiles,
   state.json with other apps' entries).

#### Scenario: Light uninstall removes install root

- **Given** Hermes is installed at `$HERMES_HOME/hermes-agent`
- **And** `app.uninstall_supported = true`
- **When** the frontend invokes `uninstall_app("hermes", "light")`
- **Then** the install script is invoked with `-Uninstall`, the
  `hermes-agent` directory is removed, `state.installed["hermes"]`
  is removed, and `HERMES_HOME` retains logs, .env, config.yaml, and
  any other apps' state.

#### Scenario: Light uninstall when app does not support uninstall

- **Given** `app.uninstall_supported = false`
- **When** the frontend invokes `uninstall_app(id, "light")`
- **Then** the script invocation is skipped; the install root is
  deleted; the rest of `HERMES_HOME` is preserved.

### Requirement: Full uninstall

The system SHALL, on `uninstall_app(id, "full")`, require a second
explicit confirmation in the UI before execution. After confirmation:

1. Back up `launcher-state.json` to
   `launcher-state.json.bak.<unix_ts>`.
2. Delete the entire `HERMES_HOME` directory tree.
3. Recreate an empty `HERMES_HOME` so the launcher can still write
   `launcher-state.json` on next start.

The frontend MUST present a confirmation dialog before invoking this
command. The Tauri command itself does NOT re-prompt; it trusts the
caller has already confirmed.

#### Scenario: Full uninstall requires confirmation

- **Given** the user clicks "Uninstall (full)" on the Hermes tile
- **When** the confirmation dialog appears
- **And** the user clicks "Cancel"
- **Then** no command is invoked and the launcher remains unchanged.

#### Scenario: Full uninstall removes everything

- **Given** Hermes is installed
- **And** the user has confirmed full uninstall
- **When** the frontend invokes `uninstall_app("hermes", "full")`
- **Then** `HERMES_HOME` is deleted; `launcher-state.json` is backed
  up; a fresh empty `HERMES_HOME` is created.

### Requirement: Repair

The system SHALL, on `repair_app(id)`, run the same flow as
`apply_pending_update(id)` but does NOT require a pre-existing cached
script. If a pending update is cached, it is used; otherwise
`install_script::resolve()` fetches a fresh copy.

#### Scenario: Repair with no pending update

- **Given** Hermes is installed and there is no pending update
- **When** the frontend invokes `repair_app("hermes")`
- **Then** the parameterized bootstrap fetches the latest install
  script and runs the full install flow.

#### Scenario: Repair with a pending update uses the cache

- **Given** Hermes is installed and has a pending update with cached
  script
- **When** the frontend invokes `repair_app("hermes")`
- **Then** the cached script is used and the install flow runs.

### Requirement: Open app settings

The system SHALL, on `open_app_settings(id)`, if `app.app_settings_url`
is non-null, use the `opener` plugin to open the URL with the system's
default handler. If `app_settings_url` is null, the command returns
`Err("App does not expose a settings URL")`.

#### Scenario: App with settings URL opens browser

- **Given** Hermes's `app_settings_url = "http://localhost:8501"`
- **When** the user clicks "Open app settings" on the Hermes tile
- **Then** the URL is opened in the user's default browser.

#### Scenario: App without settings URL is not openable

- **Given** `app.app_settings_url = null`
- **When** the user clicks "Open app settings"
- **Then** the tile shows an inline error: "This app does not expose a
  settings URL."