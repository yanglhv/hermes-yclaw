# launcher-uninstall-repair Specification

## Purpose
TBD - created by archiving change add-launcher-features-to-bootstrap-installer. Update Purpose after archive.
## Requirements
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

The `repair_app(id)` Tauri command SHALL run the parameterized bootstrap for
the app to completion, choosing the install script as follows: if
`state.pending_updates[id]` has `status = Ready` with an existing
`downloaded_script`, use it; otherwise resolve a fresh copy via
`install_script::resolve(RepoRef, script_path)`. The bootstrap emits the
standard `bootstrap` channel events tagged with `app_id = id`. On `complete`,
`state.installed[id]` is updated (`installed_via = "repair"`,
`installed_at = now`, commit/ref from the run) and any `pending_updates[id]`
is cleared. On `failed`, state is left unchanged and the failure screen offers
retry. Repair never depends on a pre-existing pending update.

Rationale: the prior implementation only logged
`"fresh fetch + bootstrap deferred to M6"` and returned `Ok` without running
any install, so the Repair button was a no-op.

#### Scenario: Repair with no pending update fetches fresh

- **Given** Hermes is installed and there is no pending update
- **When** the frontend invokes `repair_app("hermes")`
- **Then** the bootstrap fetches the latest install script and runs the full
  install flow; on success `state.installed["hermes"].installed_via =
  "repair"`.

#### Scenario: Repair with a ready pending update uses the cache

- **Given** Hermes is installed and has a `ready` pending update with a cached
  script
- **When** the frontend invokes `repair_app("hermes")`
- **Then** the cached script is used (no fresh fetch) and the install flow
  runs; on success `pending_updates["hermes"]` is cleared.

#### Scenario: Repair failure leaves state unchanged

- **Given** a repair run whose install fails
- **When** the bootstrap emits `failed`
- **Then** `state.installed["hermes"]` and any pending entry are unchanged,
  and the failure screen offers retry.

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

