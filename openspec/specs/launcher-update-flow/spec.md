# launcher-update-flow Specification

## Purpose
TBD - created by archiving change add-launcher-features-to-bootstrap-installer. Update Purpose after archive.
## Requirements
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

The `check_for_updates` Tauri command SHALL, when invoked, resolve the
`RepoRef`, perform the network probe, and on success fetch the catalog via
`launcher::catalog`. For each app recorded in `state.installed`, it SHALL
compare `installed.installed_commit` against the catalog entry's latest
commit. Apps whose latest commit differs are upserted into
`state.pending_updates[id]` with `status = Avail` (preserving any existing
`ready`/`downloading` entry — an already-ready update is NOT downgraded to
`avail`). The command returns the list of app ids whose commit differs.
When the probe fails or the catalog fetch errors, the command returns the
current pending ids unchanged and logs a warning (no crash, no empty-out).

Rationale: the prior implementation returned `pending_updates.keys()`
without any catalog comparison, so updates were never actually detected.

#### Scenario: Newer commit found

- **Given** `state.installed.hermes.installed_commit = "abc123"` and the
  catalog reports Hermes latest commit `"def456"`
- **When** `check_for_updates()` is called
- **Then** `state.pending_updates["hermes"].status = "avail"` and the
  returned list contains `"hermes"`.

#### Scenario: No newer commit

- **Given** `state.installed.hermes.installed_commit = "abc123"` and the
  catalog reports the same commit `"abc123"`
- **When** `check_for_updates()` is called
- **Then** `"hermes"` is NOT added to `pending_updates` and is absent from
  the returned list.

#### Scenario: Offline leaves pending set intact

- **Given** the network probe fails
- **When** `check_for_updates()` is called
- **Then** no crash occurs, a warning is logged, and the existing
  `pending_updates` entries are returned unchanged.

### Requirement: Background pre-download

The `pre_download_update(id)` Tauri command SHALL resolve the app's install
script via `install_script::resolve(RepoRef, script_path)`, write the
downloaded bytes to `$HERMES_HOME/bootstrap-cache/install-<sanitized_ref>.<ext>`
(creating the directory if absent), and on success set
`state.pending_updates[id]` to `status = Ready` with `downloaded_script`
populated and `downloaded_at` set to now. On failure it SHALL set
`status = Failed` with `last_error`/`last_error_at` populated. The download
runs inside a `tokio::spawn` background task; the command returns immediately
after spawning (the launcher launch path is never blocked by it).

Rationale: the prior implementation built a throwaway in-memory state and
never wrote a cached script, so `status` never reached `ready`.

#### Scenario: Successful pre-download writes cache and sets ready

- **Given** Hermes has a pending (`avail`) update and no cached script
- **When** `pre_download_update("hermes")` is spawned
- **Then** on completion the script exists at the cache path and
  `state.pending_updates["hermes"].status = "ready"` with
  `downloaded_script` set.

#### Scenario: Failed pre-download sets failed, no crash

- **Given** Hermes has a pending update but `install_script::resolve` fails
  (HTTP error / timeout)
- **When** the spawned task completes
- **Then** `state.pending_updates["hermes"].status = "failed"` with
  `last_error` populated; no panic.

#### Scenario: Pre-download does not block the caller

- **Given** `pre_download_update("hermes")` is invoked
- **Then** the Tauri command returns before the download completes; the
  download finishes in the background task.

### Requirement: Apply pending update

The `apply_pending_update(id)` Tauri command SHALL take the cached script at
`state.pending_updates[id].downloaded_script` (which MUST exist and be
`status = Ready`; otherwise return `Err`) and run it through the existing
parameterized bootstrap worker — the same path used by `start_bootstrap` —
emitting `manifest`/`stage`/`log`/`complete`/`failed` events on the
`bootstrap` channel tagged with `app_id = id`. On `complete`, it SHALL
update `state.installed[id]` (commit/ref/`installed_at = now` /
`installed_via = "update"`) and remove `state.pending_updates[id]`. On
`failed`, the cached script and the pending entry are left intact so the
user can retry. This command does NOT depend on a (still absent)
`app_id`-parameterized `start_bootstrap`; it drives `bootstrap::run_bootstrap`
directly with the `AppDescriptor` and the cached script path.

Rationale: the prior implementation resolved the cached script and then
logged `"deferred to M6"` without running anything, leaving the UI on a
stuck progress screen.

#### Scenario: Cached script runs end-to-end and updates state

- **Given** `state.pending_updates["hermes"].status = "ready"` with a valid
  `downloaded_script`
- **When** the user clicks "Install now"
- **Then** the bootstrap runs using the cached script, emits the standard
  events, and on success `state.installed["hermes"]` is updated and
  `state.pending_updates["hermes"]` is removed.

#### Scenario: No ready cache is an error

- **Given** no `ready` pending update for `"hermes"` (missing entry, or
  `status != "ready"`, or script file absent)
- **When** `apply_pending_update("hermes")` is invoked
- **Then** it returns `Err` describing the missing cache; no bootstrap runs.

#### Scenario: Apply failure leaves pending entry intact

- **Given** a `ready` pending update whose install fails at the `venv` stage
- **When** the bootstrap emits `failed`
- **Then** the cached script remains, `pending_updates["hermes"]` is
  unchanged, and the failure screen offers retry.

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

