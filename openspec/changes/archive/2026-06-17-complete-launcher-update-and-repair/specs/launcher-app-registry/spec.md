# launcher-app-registry

## MODIFIED Requirements

### Requirement: Min-launcher-version Gating

The `list_available_apps` command SHALL compute, for each `LaunchableApp`,
`launcher_too_old = semver_lt(current_launcher_version,
descriptor.min_launcher_version)`, where `current_launcher_version` is the
launcher's own version from `env!("CARGO_PKG_VERSION")` at compile time. The
comparison is a proper semantic-version less-than (major.minor.patch numeric
compare), not a lexicographic string compare. When `launcher_too_old == true`,
the frontend tile and AppDetail SHALL render the "⚠ update launcher" badge
and disable the Install/Update primary action.

Rationale: the prior implementation hardcoded `launcher_too_old: false`, so
the gate never evaluated and the badge/disabled state never appeared.

**Version-source note (implementation decision):** the launcher
`Cargo.toml` version is currently `0.0.1` while `AppDescriptor::literal_hermes().min_launcher_version = "0.1.0"`. Until the launcher is
versioned `>= 0.1.0`, the Hermes tile will correctly show the too-old badge.
The build step SHOULD bump `Cargo.toml` to `0.1.0` (or the comparison will
gate Hermes during dev). This decision is surfaced in `design.md`.

#### Scenario: Newer app pinned to newer launcher

- **Given** `descriptor.min_launcher_version = "1.0.0"` and the running
  launcher version is `0.9.5`
- **When** `list_available_apps()` builds the tile
- **Then** `launcher_too_old == true`; the tile shows "⚠ update launcher" and
  Install/Update are disabled.

#### Scenario: Older app on newer launcher is fine

- **Given** `descriptor.min_launcher_version = "0.1.0"` and the running
  launcher version is `0.9.5`
- **When** `list_available_apps()` builds the tile
- **Then** `launcher_too_old == false`; no badge; Install is enabled.

#### Scenario: Lexicographic compare is NOT used

- **Given** `descriptor.min_launcher_version = "0.10.0"` and the running
  launcher version is `0.9.0`
- **When** the comparison runs
- **Then** `launcher_too_old == true` (0.9.0 < 0.10.0 numerically), even
  though `"0.10.0" < "0.9.0"` lexicographically.
