# launcher-multi-app-install Specification

## Purpose
TBD - created by archiving change add-launcher-features-to-bootstrap-installer. Update Purpose after archive.
## Requirements
### Requirement: Parameterized bootstrap

The system SHALL modify `start_bootstrap` (Tauri command) to accept an
`app_id: string` parameter alongside the existing `args: StartBootstrapArgs`.
The `app_id` resolves to an `AppDescriptor` via the app registry, which
provides the install script URL, commit pin, and install root path used
by the worker task.

When `app_id` is omitted (legacy callers), the system SHALL default to
`state.default_app_id`, falling back to `"hermes"` if `state` is empty.

#### Scenario: Install via app_id

- **Given** the app registry contains an app with `id="hermes"`
- **When** the frontend invokes `start_bootstrap("hermes", args)`
- **Then** the worker task resolves Hermes's `install.ps1` from the
  registry-provided `RepoRef` and `script_path`, not from a hardcoded
  URL.

#### Scenario: Unknown app_id is rejected

- **Given** no app with `id="does-not-exist"` exists in the registry
- **When** the frontend invokes `start_bootstrap("does-not-exist", args)`
- **Then** the command returns `Err("unknown app id: does-not-exist")`
  and no worker task is spawned.

### Requirement: Parameterized update

The system SHALL modify `start_update` (Tauri command) to accept an
`app_id: string` parameter. The update flow for that app:

1. waits for the app's running process to release file locks (Windows),
2. runs `hermes update --yes --gateway --force --branch <ref>` (or the
   equivalent for non-Hermes apps in a future extension),
3. runs `<app> desktop --build-only` (or the app's build command),
4. macOS: installs the rebuilt `.app` bundle via atomic swap with
   rollback,
5. launches the rebuilt app.

For non-Hermes apps in this change's scope, only steps 1, 4, and 5 are
implemented; the repo-update and rebuild steps remain Hermes-specific
because no other app is registered yet.

#### Scenario: Hermes update via app_id

- **Given** the registry has Hermes with `id="hermes"`, `default=true`,
  and is installed at `$HERMES_HOME/hermes-agent`
- **When** the frontend invokes `start_update("hermes")`
- **Then** the existing update flow runs with `install_root` derived
  from the registry's `install_root` field.

### Requirement: Generalized app launch

The system SHALL replace the existing `launch_hermes_desktop(install_root)`
Tauri command with a generic `launch_app(app_id)` command that:

1. Looks up the `AppDescriptor` from the registry.
2. Resolves the platform-specific binary path from
   `descriptor.binary.{windows|macos|linux}` joined with
   `state.installed[app_id].install_root`.
3. On macOS, calls `/usr/bin/open <binary.macos>` (which MUST end in
   `.app`).
4. On Windows, spawns the exe with `DETACHED_PROCESS=0x00000008`.
5. On Linux, spawns the binary directly.
6. Sleeps 150ms then exits the installer process.

If the resolved binary does not exist on disk, the command returns an
`Err` with a message describing the missing path and how to run the
app's build step from a terminal.

#### Scenario: Hermes launch on macOS

- **Given** Hermes is installed and its binary exists at
  `apps/desktop/release/mac-arm64/Hermes.app`
- **When** the frontend invokes `launch_app("hermes")`
- **Then** `/usr/bin/open` is called with the `.app` path; the installer
  exits 150ms later.

#### Scenario: Hermes launch on Windows

- **Given** Hermes is installed and its binary exists at
  `apps/desktop/release/win-unpacked/Hermes.exe`
- **When** the frontend invokes `launch_app("hermes")`
- **Then** the exe is spawned with `DETACHED_PROCESS` and the
  installer exits 150ms later.

#### Scenario: Binary missing

- **Given** Hermes is registered but no binary exists at the expected
  path (e.g. install was skipped or failed)
- **When** the frontend invokes `launch_app("hermes")`
- **Then** the command returns `Err("Couldn't find a built Hermes
  desktop at ...")`.

### Requirement: Parameterized install script URL

The system SHALL modify `install_script::resolve()` to accept a
`RepoRef { owner, name, ref_name }` and an `app_relative_script_path`
instead of the hardcoded
`https://raw.githubusercontent.com/NousResearch/hermes-agent/{ref}/scripts/{filename}`.

The URL constructed SHALL be:
```
https://raw.githubusercontent.com/{owner}/{name}/{ref_name}/{app_relative_script_path}
```

The cached path SHALL remain
`$HERMES_HOME/bootstrap-cache/install-<sanitized_ref>.<ext>`.

#### Scenario: Custom repo URL produces custom fetch

- **Given** `RepoRef { owner: "alice", name: "launcher-scripts", ref_name: "main" }`
  and `app_relative_script_path = "apps/foo/install.sh"`
- **When** `install_script::resolve(...)` is called
- **Then** it fetches
  `https://raw.githubusercontent.com/alice/launcher-scripts/main/apps/foo/install.sh`.

### Requirement: Event payload appId discriminator

The system SHALL add an `appId?: string` field to every
`BootstrapEvent` payload. When emitted from a per-app worker task, the
field is populated with the worker app's id. When omitted (e.g. legacy
callers or un-scoped log lines), the frontend routes the event to the
currently-selected app's state, defaulting to the registry's default
app if none is selected.

#### Scenario: Per-app event routing

- **Given** two apps `"hermes"` and `"myapp"` are installed and Hermes
  is currently selected (`$currentAppId = "hermes"`)
- **When** a `BootstrapEvent::Stage { name: "venv", state: Running }`
  arrives with `appId = "myapp"`
- **Then** the event updates `$bootstrapByApp["myapp"]`, not the
  Hermes view; the current screen remains on whatever Hermes was
  showing.

### Requirement: install.ps1 protocol preservation

The system SHALL preserve the existing install-script protocol
verbatim:

- `-Manifest` returns JSON on the last stdout line with shape
  `{stages: [...], protocol_version: int|null}`.
- `-Stage <NAME> -NonInteractive -Json` returns JSON on the last stdout
  line with shape `{ok: bool, stage: string, skipped?: bool, reason?: string, data?: any}`.
- Per-stage cancellation via `cancel_bootstrap` kills the child via
  `tokio::process::Child::start_kill`.

#### Scenario: Existing protocol still parses

- **Given** the existing `install.ps1` script with `-Manifest` and
  `-Stage` outputs that conform to the current JSON shape
- **When** the parameterized bootstrap runs the script
- **Then** the same parsing logic in `powershell::parse_manifest` and
  `powershell::parse_stage_result` succeeds without modification.

#### Scenario: Cancellation still works

- **Given** a parameterized bootstrap is running `install.ps1 -Stage venv`
- **When** the frontend invokes `cancel_bootstrap`
- **Then** the child process is killed within 1 second; the
  `BootstrapEvent::Failed { error: "cancelled by user" }` is emitted;
  the worker task exits with `Err`.

