# Capability: launcher-app-registry

## Purpose

Manages the multi-app catalog, persistent launcher state, and repo URL
configuration. This capability is the data backbone: every other launcher
capability reads from or writes to it.

## ADDED Requirements

### Requirement: App Descriptor

The system SHALL parse a JSON object `app.json` at the root of each
`apps/<app-id>/` directory in the distributor repo and produce an
`AppDescriptor` struct with at minimum these fields:

- `id: string` — kebab-case identifier, matches the parent directory name
- `display_name: string` — human-readable name shown on the Home tile
- `category: string` — used for grouping tiles on the Home screen
- `default: bool` — at most one app in the catalog may be `default=true`
- `script_path: string` — relative path to the install script under
  `apps/<app-id>/`
- `install_root: string` — subdirectory name under `HERMES_HOME`
- `binary.{windows,macos,linux}: string` — relative path to the launchable
  artifact, with `binary.macos` ending in `.app`
- `uninstall_supported: bool` — whether the app's `install.ps1` accepts
  `-Uninstall`
- `app_settings_url: string | null` — optional; opened via `opener` plugin
- `min_launcher_version: semver` — launcher rejects installs if its own
  version is below this
- `schema_version: integer` — MUST equal `1`

#### Scenario: Valid app.json is parsed

- **Given** `apps/hermes/app.json` containing a complete schema_version=1
  object with `id="hermes"`, `default=true`, `binary.macos` ending in
  `.app`
- **When** the launcher calls `parse_app_json(contents)`
- **Then** an `AppDescriptor { id: "hermes", default: true, ... }` is
  returned and no error is raised.

#### Scenario: Invalid schema_version is rejected

- **Given** `app.json` containing `"schema_version": 2`
- **When** the launcher calls `parse_app_json(contents)`
- **Then** a `Result::Err` is returned and the app is omitted from the
  catalog.

### Requirement: RepoRef Resolution

The system SHALL resolve the distributor repo URL from three sources in
ascending priority:

1. Build-time env vars `BUILD_REPO_OWNER`, `BUILD_REPO_NAME`,
   `BUILD_PIN_BRANCH`
2. Runtime env vars `LAUNCHER_REPO_OWNER`, `LAUNCHER_REPO_NAME`,
   `LAUNCHER_REPO_REF` (or combined `LAUNCHER_REPO_OVERRIDE=owner/name@ref`)
3. `~/.hermes/launcher-config.yaml` (parsed with `serde_yaml`)

The combined result is a `RepoRef { owner, name, ref_name }`.

#### Scenario: Build-time constants are the lowest priority

- **Given** no runtime env vars and no `launcher-config.yaml`
- **When** the launcher calls `RepoRef::resolve()`
- **Then** the returned `RepoRef` matches the build-time constants.

#### Scenario: Runtime env var overrides build-time

- **Given** build-time `BUILD_REPO_OWNER="default-owner"` and runtime
  `LAUNCHER_REPO_OWNER="user-owner"`
- **When** `RepoRef::resolve()` is called
- **Then** the returned `RepoRef.owner == "user-owner"`.

#### Scenario: launcher-config.yaml overrides env vars

- **Given** runtime `LAUNCHER_REPO_OWNER="env-owner"` and
  `~/.hermes/launcher-config.yaml` containing
  `repo: { owner: yaml-owner }`
- **When** `RepoRef::resolve()` is called
- **Then** the returned `RepoRef.owner == "yaml-owner"`.

### Requirement: Persistent Launcher State

The system SHALL persist launcher state at
`$HERMES_HOME/launcher-state.json` with schema:

- `schema_version: integer` (currently `1`)
- `default_app_id: string | null`
- `installed: Record<app_id, InstalledApp>`
  - `install_root: string` (absolute path)
  - `installed_commit: string`
  - `installed_ref_type: "branch" | "commit"`
  - `installed_ref_name: string`
  - `installed_at: RFC3339 timestamp`
  - `installed_via: string` (diagnostic tag)
- `pending_updates: Record<app_id, PendingUpdate>`
  - `latest_commit`, `latest_ref_name`
  - `status: "avail" | "downloading" | "ready" | "failed"`
  - `downloaded_script: string | null`
  - `downloaded_at: RFC3339 timestamp | null`
  - `last_error: string | null`, `last_error_at: RFC3339 timestamp | null`
- `last_update_check_at: RFC3339 timestamp | null`

Writes SHALL be atomic via `tmp + rename` to avoid partial writes on crash.

#### Scenario: Atomic write does not corrupt on crash

- **Given** a `launcher-state.json` being written
- **When** the writing process is killed mid-write
- **Then** the existing `launcher-state.json` remains intact (rename is
  atomic on POSIX; CreateFileW + FlushFileBuffers on Windows).

#### Scenario: Corrupt state.json is backed up and rebuilt

- **Given** `launcher-state.json` containing invalid JSON
- **When** the launcher starts
- **Then** the file is renamed to `launcher-state.json.bak.<unix_ts>` and
  an empty state is written; the Home screen shows a "launcher state
  recovered" notice.

### Requirement: Catalog Enumeration

The system SHALL enumerate available apps by:

1. Calling `GET https://api.github.com/repos/{owner}/{name}/contents/apps?ref={ref}`
2. For each directory entry, calling
   `GET .../apps/{app-id}/app.json`
3. Parsing each response into an `AppDescriptor`
4. Cross-referencing `launcher-state.json` to compute the merged
   `LaunchableApp` (descriptor + installed state + pending update flag)

#### Scenario: Catalog fetch is unauthenticated

- **Given** a public GitHub repo
- **When** the launcher calls `list_available_apps(repo)`
- **Then** no auth token is sent and the call succeeds for repos that
  allow anonymous access.

#### Scenario: Network failure yields empty catalog without crash

- **Given** the GitHub API is unreachable
- **When** `list_available_apps(repo)` is called
- **Then** it returns `Ok(empty Vec)` and logs a warning; the launcher
  proceeds with the current installed set only.

#### Scenario: Invalid app.json is skipped

- **Given** one of the apps has a malformed `app.json`
- **When** `list_available_apps(repo)` is called
- **Then** the invalid app is omitted and other apps are returned
  normally; a warning is logged with the app id.

### Requirement: Min-launcher-version Gating

The system SHALL compute, for each `LaunchableApp`, a boolean
`launcher_too_old = current_launcher_version < app.min_launcher_version`
and surface it on the tile. When `launcher_too_old == true`, the install
and update actions are disabled in the UI.

#### Scenario: Newer app pinned to newer launcher

- **Given** an `app.json` with `min_launcher_version = "1.0.0"` and the
  running launcher version is `0.9.5`
- **When** the Home screen renders the tile
- **Then** the tile shows a "⚠ update launcher" badge and the install
  button is disabled.

#### Scenario: Older app on newer launcher is fine

- **Given** an `app.json` with `min_launcher_version = "0.1.0"` and the
  running launcher version is `0.9.5`
- **When** the Home screen renders the tile
- **Then** the tile shows no badge and the install button is enabled.

### Requirement: Existing Install Detection

The system SHALL detect an existing install on first launch post-change
when `launcher-state.json` is missing but `install_root` for the default
app exists on disk. In that case the launcher writes a fresh
`launcher-state.json` with `installed[default_app_id]` populated from
the on-disk state (commit read from the existing repo's HEAD if
available, else `"unknown"`).

#### Scenario: Hermes already installed, no state.json

- **Given** `$HERMES_HOME/hermes-agent/` exists with a populated git
  checkout, but `~/.hermes/launcher-state.json` does not
- **When** the launcher starts
- **Then** a fresh `launcher-state.json` is written with
  `installed.hermes.install_root` set to the absolute path and
  `installed_commit` set to `git rev-parse HEAD` (or `"unknown"` if the
  directory is not a git checkout).