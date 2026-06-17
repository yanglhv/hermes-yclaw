# Capability: launcher-update-flow

## Purpose

Detects available updates, pre-downloads updated install scripts in the
background, and surfaces pending updates to the user without blocking
the default-app launch. Pre-downloads are not installs; the user
explicitly triggers an apply.

## ADDED Requirements

### Requirement: Network probe

The system SHALL perform a network probe before any update-related
network call. The probe is a single GET to
`https://api.github.com/repos/{owner}/{name}/contents/apps?ref={ref}`
with `timeout=2s`. The probe result is a boolean: `true` on HTTP 2xx,
`false` on timeout, network error, or any other status.

The probe does NOT block the launch. If the probe returns `false`, the
launcher proceeds to launch the default app without any update check.

#### Scenario: Online probe succeeds

- **Given** the GitHub API is reachable
- **When** the launcher calls `probe_network(repo, 2s)`
- **Then** the call returns `true` within 2.1 seconds and the launcher
  proceeds with catalog fetch.

#### Scenario: Offline probe fails fast

- **Given** the network is unreachable
- **When** the launcher calls `probe_network(repo, 2s)`
- **Then** the call returns `false` within 2.1 seconds and the
  launcher skips update check entirely.

### Requirement: Update check

The system SHALL, when the network probe succeeds, fetch the catalog
and compare each installed app's `installed_commit` against the
catalog's latest commit for that app. Apps whose latest commit
differs from the installed commit are added to a `pending` list.

#### Scenario: Newer commit found

- **Given** state has `installed.hermes.installed_commit = "abc123"`
  and the catalog reports Hermes latest commit `"def456"`
- **When** `check_for_updates()` is called
- **Then** the returned `UpdateSummary.updates_available` contains
  `"hermes"`.

#### Scenario: No newer commit

- **Given** state has `installed.hermes.installed_commit = "abc123"`
  and the catalog reports Hermes latest commit `"abc123"`
- **When** `check_for_updates()` is called
- **Then** the returned `UpdateSummary.updates_available` does NOT
  contain `"hermes"`.

### Requirement: Background pre-download

The system SHALL, for each app in `pending`, run
`install_script::resolve()` and write the resulting script to the
bootstrap cache without invoking the install. The download runs in a
background `tokio::spawn` task so it does not block the launch. The
launch proceeds to launch the default app immediately after the spawn
returns.

#### Scenario: Pre-download does not block launch

- **Given** Hermes has a pending update and the script is not cached
- **When** the launcher runs `run_silent_default()`
- **Then** the call to `launch_app_silent` happens BEFORE the
  download completes; the download runs to completion in the
  background after the installer process exits.

#### Scenario: Successful pre-download updates state

- **Given** Hermes has a pending update and the script downloads
  successfully to `$HERMES_HOME/bootstrap-cache/install-def456.ps1`
- **When** the download task completes
- **Then** `state.pending_updates["hermes"].status` is set to
  `"ready"`, `downloaded_script` is set to the cache path, and
  `state.last_update_check_at` is updated to the current time.

#### Scenario: Failed pre-download does not block launch

- **Given** Hermes has a pending update but the download fails (HTTP
  error, timeout, etc.)
- **When** the download task completes
- **Then** the failure is logged and `state.pending_updates["hermes"]`
  is set to `status="failed"` with `last_error` populated; the
  default app launch has already happened; the next launcher launch
  re-attempts.

### Requirement: Apply pending update

The system SHALL, on user request, take the cached install script for
an app and run the full parameterized bootstrap flow against it. The
apply command `apply_pending_update(app_id)` is equivalent to
`start_bootstrap(app_id, args)` but uses the cached script as its
source (no fresh fetch) and runs the install end-to-end.

#### Scenario: Cached script is used

- **Given** Hermes has `pending_updates["hermes"].status = "ready"`
  with `downloaded_script = ".../install-def456.ps1"`
- **When** the user clicks "Install now" in the Home screen banner
- **Then** the launcher runs the full bootstrap flow using the cached
  script, emits the standard manifest/stage/log events, and updates
  `state.installed["hermes"]` on success.

#### Scenario: Apply failure leaves pending entry intact

- **Given** Hermes has `pending_updates["hermes"].status = "ready"`
- **When** the user clicks "Install now" but the install fails at the
  `venv` stage
- **Then** the cached script remains; `pending_updates["hermes"]` is
  unchanged; the failure screen shows the retry button.

### Requirement: Pending-update cache validation

The system SHALL, on every launcher startup, re-validate that
`state.pending_updates[id].downloaded_script` points to an existing
file. If the file is missing, the entry degrades to
`status = "failed"` and the next `check_for_updates()` reschedules a
fresh download.

#### Scenario: Cache file deleted out-of-band

- **Given** `state.pending_updates["hermes"].downloaded_script =
  ".../install-def456.ps1"` and `status = "ready"`
- **When** the user (or another process) deletes that file
- **And** the launcher restarts
- **Then** on startup the launcher detects the missing file,
  sets `status = "failed"`, and the next update check re-downloads.