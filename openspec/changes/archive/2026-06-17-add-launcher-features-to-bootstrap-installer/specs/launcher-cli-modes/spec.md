# Capability: launcher-cli-modes

## Purpose

Routes the launcher's behavior based on CLI flags at process startup.
Defines the dispatch in `lib.rs::run()`'s setup hook and the
`get_launch_mode()` Tauri command.

## ADDED Requirements

### Requirement: CLI flag parsing

The system SHALL recognize the following CLI flags:

| Flag                       | Effect                                  |
|----------------------------|-----------------------------------------|
| (none)                     | Silent default flow                     |
| `--settings`               | Show full UI (Home)                     |
| `--launch <app-id>`        | Launch the named app, no UI             |
| `--update`                 | Existing update flow (unchanged)        |
| `--repair` or `--reinstall`| Force full UI + repair flow             |
| `--branch <ref>`           | Override ref for install/update         |
| `--target-app <path>`      | macOS-only: install update into <path>  |

The existing `AppMode::from_args` and `force_setup_from_args` continue
to function; this capability adds a third helper,
`launch_args_from_args`, that returns the optional `--launch <id>` pair.

#### Scenario: Bare launch is silent

- **Given** `std::env::args()` is `["Hermes-Setup.exe"]`
- **When** the launcher starts
- **Then** `launch_args_from_args` returns `None` and the silent
  default flow runs.

#### Scenario: --settings sets mode

- **Given** `std::env::args()` is `["Hermes-Setup.exe", "--settings"]`
- **When** `lib.rs::run()` parses flags
- **Then** the setup hook decides to show the main window and the
  frontend renders the Home screen.

#### Scenario: --launch routes to specific app

- **Given** `std::env::args()` is
  `["Hermes-Setup.exe", "--launch", "myapp"]`
- **When** `launch_args_from_args` is called
- **Then** it returns `Some("myapp")`.

### Requirement: Setup hook dispatch

The system SHALL extend the `lib.rs` setup hook with a dispatch on the
combined `(AppMode, force_setup, launch_args)` triple:

| Mode     | force_setup | launch_args | Behavior                            |
|----------|-------------|-------------|-------------------------------------|
| Install  | false       | None        | `run_silent_default()`              |
| Install  | false       | Some(id)    | `launch_app_silent(id)`             |
| Install  | true        | _           | Show window; frontend renders welcome (first install) or home (existing) |
| Install  | _           | Some(id) + force_setup | Show window; frontend renders home  |
| Update   | _           | _           | Existing update flow                |

The existing macOS fast-path (`lib.rs:114-154`) is folded into this
dispatch. The condition that gates it (`mode == Install && !force_setup
&& hermes_is_installed`) becomes a sub-case of "silent default" —
specifically, when Hermes is installed and is the default app, the
launcher simply calls `launch_app("hermes")` and exits, replacing the
macOS-only fast-path with a portable equivalent.

#### Scenario: Portable silent launch replaces macOS fast path

- **Given** Hermes is installed and is the default app
- **And** the user double-clicks `Hermes-Setup.exe`
- **When** the setup hook runs
- **Then** the launcher calls `launch_app("hermes")` and exits,
  regardless of platform (Windows, macOS, Linux).

#### Scenario: First install shows window

- **Given** `launcher-state.json` does not exist
- **When** the setup hook runs
- **Then** the main window is shown and the frontend's welcome
  screen renders.

#### Scenario: --repair shows window

- **Given** `std::env::args()` contains `--repair`
- **When** the setup hook runs
- **Then** `force_setup == true` and the main window is shown; the
  frontend's welcome screen renders with a "Repair existing install"
  subtitle.

### Requirement: get_launch_mode command

The system SHALL expose a Tauri command `get_launch_mode() -> LaunchMode`
that returns the resolved launch mode:

```rust
struct LaunchMode {
    kind: "first_install" | "settings" | "launch" | "update" | "silent",
    target_app_id: Option<String>,
}
```

The frontend calls this once at startup via `launcher-mode.ts` to
decide the initial route.

The mode is determined by:

1. If `--update` flag → `update`.
2. If `--launch <id>` flag → `launch` with `target_app_id = id`.
3. If `--settings` flag → `settings`.
4. If `--repair` or `--reinstall` flag → `settings` (frontend renders
   welcome for the repair flow).
5. If `launcher-state.json` does not exist or `installed` is empty →
   `first_install`.
6. Otherwise → `silent`.

#### Scenario: First install reports first_install

- **Given** `launcher-state.json` does not exist
- **When** the frontend calls `get_launch_mode()`
- **Then** it returns
  `{ kind: "first_install", target_app_id: null }`.

#### Scenario: Settings mode reports settings

- **Given** `--settings` was passed
- **When** the frontend calls `get_launch_mode()`
- **Then** it returns `{ kind: "settings", target_app_id: null }`.

#### Scenario: Launch mode reports launch with target

- **Given** `--launch myapp` was passed
- **When** the frontend calls `get_launch_mode()`
- **Then** it returns `{ kind: "launch", target_app_id: "myapp" }`.

### Requirement: Silent flow concurrency guard

The system SHALL protect `run_silent_default()` with a process-wide
`AtomicBool` so that React StrictMode's double-invocation, window
reloads, or duplicate CLI invocations cannot spawn two silent flows at
once.

#### Scenario: Double-invocation is a no-op

- **Given** `run_silent_default` is already running
- **When** a second call to `run_silent_default` happens (e.g. via
  React StrictMode)
- **Then** the second call returns `Ok(())` immediately without
  performing any network or launch work; a warning is logged.

#### Scenario: Existing UPDATE_RUNNING guard preserved

- **Given** `start_update` is in flight
- **When** a second `start_update` is invoked
- **Then** the existing `UPDATE_RUNNING` guard returns the synthetic
  manifest re-emit path (preserved behavior).