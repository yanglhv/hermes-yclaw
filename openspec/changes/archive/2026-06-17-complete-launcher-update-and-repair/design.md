# Design: Complete launcher update, repair, and version-gating flows

## Context

The precursor change shipped the launcher skeleton but left four lifecycle
flows as stubs and one gate uncomputed (see `proposal.md` and the archived
`close-issues.md`). This change completes them with minimal new surface,
reusing the existing parameterized bootstrap.

## Key existing mechanism (load-bearing)

`bootstrap::run_bootstrap` resolves its install script via
`install_script::resolve(repo, script_path, &cached_path)`, whose precedence
is: **dev checkout → cached file at `cached_path` → network**. Therefore:

- If a script already exists at `cached_path(kind, ref_name)`, the bootstrap
  uses it without a fresh fetch.
- This means **pre-download = "populate `cached_path`"** and **apply/repair =
  "run the bootstrap"** — no second script-fetch code path is needed.

This is the central reuse that keeps the change small.

## Approach per item

### B1 + B2 — Apply / Repair share one install driver

Both `apply_pending_update(id)` and `repair_app(id)` ultimately need to "run
the parameterized bootstrap for an app and, on success, update
`LauncherState.installed[id]`." Extract a single async helper, e.g.
`run_app_install(app_handle, descriptor, id, script_source)`, that:

1. Spawns the existing `run_bootstrap` worker (the same task body
   `start_bootstrap` uses), passing the `AppDescriptor`.
2. Tags every emitted `bootstrap` event with `app_id = id`. Today the emit
   sites pass `app_id: None`; this requires threading `app_id` through
   `run_bootstrap` (a focused, mechanical change — add an `app_id: &str`
   param, forward it at each `emit_event` call).
3. On `complete`, locks `LauncherStateHandle`, updates
   `installed[id]` (`installed_commit`/`installed_ref_name` from the run,
   `installed_at = now`, `installed_via = "update" | "repair"`), and removes
   `pending_updates[id]`.
4. On `failed`, leaves launcher state unchanged (the UI failure screen
   already offers retry via `applyPendingUpdate`/`repairApp`).

Difference between the two commands:
- **apply** requires `pending_updates[id].status == Ready` with an existing
  `downloaded_script` (else `Err`); the cached script is already at
  `cached_path`, so the driver uses it.
- **repair** has no precondition: if a `ready` cached script exists it is used,
  otherwise `install_script::resolve` falls through to the network (fresh
  fetch). No extra code — the existing resolve precedence handles it.

The frontend already routes to `progress` on click and listens to the
`bootstrap` channel, so once the events flow with the right `app_id`, the
existing `store.ts` per-app routing renders progress/success/failure
unchanged.

### P1 — Pre-download writes the cache path

`pre_download_update(id)` resolves the app's `RepoRef` + `script_path`, calls
`install_script::resolve(...)`, and ensures the resolved script is at
`cached_path(kind, ref_name)` (if `resolve` returned a temp/network path,
copy/move it there). Then it sets `pending_updates[id] = { status: Ready,
downloaded_script: <cached_path>, downloaded_at: now }` and saves state. The
HTTP work runs inside `tokio::spawn`; the command returns immediately.

### P2 — Update check compares installed commit vs repo HEAD

The baseline spec says "compare against the catalog's latest commit", but the
catalog (GitHub Contents API) returns directory listings, not commits. Since
each registered app **is** the repo (Hermes = `NousResearch/hermes-agent`), the
authoritative "latest commit" is the repo ref's HEAD SHA, obtained via
`GET https://api.github.com/repos/{owner}/{name}/commits/{ref}`. So:

`check_for_updates()` = resolve RepoRef → probe → fetch HEAD SHA → for each
`installed[id]`, if `installed_commit != head_sha`, upsert
`pending_updates[id]` to `Avail` (never downgrading an existing `Ready`).
Return the changed ids.

### B3 — Min-launcher-version gate + version bump

- Add a tiny `semver_lt(a, b)` helper (parse `major.minor.patch` numerically;
  no pre-release handling; reject unparseable as `false`). Avoids pulling the
  `semver` crate (the release profile is size-conscious).
- `list_available_apps` computes
  `launcher_too_old = semver_lt(env!("CARGO_PKG_VERSION"), descriptor.min_launcher_version)`.
- **Version bump:** the launcher `Cargo.toml` is `0.0.1`;
  `AppDescriptor::literal_hermes().min_launcher_version = "0.1.0"`. Without a
  bump the Hermes tile would permanently show the too-old badge. The build
  bumps `Cargo.toml` to `0.1.0` so the gate is meaningful and Hermes passes
  it. (If we deliberately want the badge during dev, skip the bump — but then
  Install is disabled, blocking V8. Recommendation: bump.)

The frontend already reads `launcher_too_old` and the tile/AppDetail already
have the badge + disabled-button branches; B3 is primarily the backend
computation (the `false` hardcode).

## Files touched

```
apps/bootstrap-installer/src-tauri/Cargo.toml                 ★ version 0.0.1 → 0.1.0
apps/bootstrap-installer/src-tauri/src/bootstrap.rs           ★ thread app_id through run_bootstrap emit sites; expose install driver
apps/bootstrap-installer/src-tauri/src/launcher/commands.rs   ★ apply_pending_update, repair_app, pre_download_update, check_for_updates, list_available_apps (launcher_too_old)
apps/bootstrap-installer/src-tauri/src/launcher/update.rs     ★ real pre-download + head-commit fetch helpers
apps/bootstrap-installer/src-tauri/src/app.rs (or util)       ★ semver_lt helper
```

No frontend structural changes required (store/routes already consume
`launcher_too_old`, `pending`, and the `bootstrap` event channel).

## Preserved contracts (must NOT change)

- `bootstrap` event channel + install.ps1/install.sh protocol.
- Launcher state file schema (`LauncherState`, `InstalledApp`, `PendingUpdate`).
- All work shipped by the precursor (launch, uninstall, detection, routing).

## Testing strategy

Per AGENTS.md: Rust `#[cfg(test)]` with `tempfile`/`mockito`; no live network.
New tests:
- `semver_lt`: numeric compare; rejects bad input; the `0.9.0 < 0.10.0` case.
- `check_for_updates`: mocked GitHub HEAD SHA → pending; equal commit → no-op;
  offline → returns existing unchanged.
- `pre_download_update`: mocked resolve → cache file written, `status=Ready`.
- apply/repair driver: on `complete`, `installed[id]` updated + pending
  cleared; on `failed`, state unchanged. (Bootstrap worker itself is exercised
  by the existing 906+1073 LOC of bootstrap/update tests — unchanged.)
