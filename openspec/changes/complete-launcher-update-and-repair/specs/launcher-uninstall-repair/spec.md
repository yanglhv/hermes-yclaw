# launcher-uninstall-repair

## MODIFIED Requirements

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
