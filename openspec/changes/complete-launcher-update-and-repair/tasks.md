# Tasks: complete-launcher-update-and-repair

Ordered by execution dependency, not by feature area. TDD per AGENTS.md:
failing test first, then minimum code, then refactor.

## M1 — Version gate (B3) [foundational, no deps]

- [ ] 1.1 Add `semver_lt(a: &str, b: &str) -> bool` helper (numeric
      major.minor.patch compare; unparseable → `false`; reject pre-release
      suffixes). Unit tests incl. `"0.9.0" < "0.10.0"` == true and bad input.
- [ ] 1.2 In `list_available_apps`, compute
      `launcher_too_old = semver_lt(env!("CARGO_PKG_VERSION"), descriptor.min_launcher_version)`
      (replace the `false` hardcode at `commands.rs:64`).
- [ ] 1.3 Bump `Cargo.toml` version `0.0.1` → `0.1.0` (so Hermes min `0.1.0`
      passes the gate). Confirm the Hermes tile no longer trips the badge.
- [ ] 1.4 `cargo test` + `cargo check` green.

## M2 — Update check (P2) [depends on RepoRef resolve]

- [ ] 2.1 Add a helper to fetch the repo ref's HEAD SHA via
      `GET https://api.github.com/repos/{owner}/{name}/commits/{ref}`
      (mockito test: returns SHA; offline → `Err`).
- [ ] 2.2 Rewrite `check_for_updates` to: resolve RepoRef → probe → fetch HEAD
      SHA → for each `installed[id]`, if `installed_commit != head_sha`,
      upsert `pending_updates[id]` to `Avail` (never downgrade `Ready`); return
      changed ids. Offline/err → log + return existing pending unchanged.
      Mockito test covers newer-commit, equal-commit, offline.
- [ ] 2.3 Frontend: confirm `$updateCheckStatus` flow still compiles; no
      structural change expected.

## M3 — Pre-download (P1) [depends on install_script::resolve]

- [ ] 3.1 Add `pre_download_update` real impl: resolve RepoRef + script_path,
      call `install_script::resolve`, ensure the resolved script lands at
      `cached_path(kind, ref_name)` (copy if returned path differs), then set
      `pending_updates[id] = { status: Ready, downloaded_script: <cached>,
      downloaded_at: now }` and save state. On failure set `Failed` +
      `last_error`. Run the HTTP work in `tokio::spawn`; command returns fast.
- [ ] 3.2 Test (mockito): resolve succeeds → cache file exists +
      `status=Ready`; resolve fails → `status=Failed`, no panic.

## M4 — Apply + Repair (B1 + B2) [depends on bootstrap run path]

- [ ] 4.1 Thread `app_id: Option<&str>` through `run_bootstrap` and every
      `emit_event(...)` site inside it (currently `app_id: None`). Existing
      bootstrap tests must still pass (they assert shape, not app_id).
- [ ] 4.2 Extract a shared driver
      `run_app_install(app, descriptor, id, via: &str)` that spawns the
      bootstrap worker for `descriptor`, tags events with `app_id = id`, and
      on `complete` updates `LauncherState.installed[id]`
      (commit/ref/`installed_at`/`installed_via = via`) + clears
      `pending_updates[id]`; on `failed` leaves state unchanged.
- [ ] 4.3 `apply_pending_update(id)`: require `pending_updates[id].status ==
      Ready` with an existing `downloaded_script` (else `Err`); call
      `run_app_install(..., "update")`. Test: no-ready → `Err`; (worker
      success path covered by existing bootstrap tests).
- [ ] 4.4 `repair_app(id)`: no precondition; call `run_app_install(..., "repair")`
      (the bootstrap's cached→network resolve handles script choice). Replace
      the current stub log.
- [ ] 4.5 End-to-end smoke: from `--settings` Home, a `ready` pending update →
      "Install now" reaches progress → success updates `installed[id]`.

## M5 — Verification

- [ ] 5.1 `cargo test` (crate) green; new tests pass; pre-existing
      `lock_probe_paths` / `light_uninstall` failures unchanged (tracked in
      T1, out of scope here).
- [ ] 5.2 `npm run typecheck` green (no frontend structural change expected).
- [ ] 5.3 Acceptance walk-through: V5 (pre-download ready), V8 (apply →
      progress → success), V16 (repair runs install), V18 (min-version badge
      + disabled), V19 (cache-missing degrades to failed).
- [ ] 5.4 `openspec validate complete-launcher-update-and-repair --strict`.

## Dependency graph

```
M1 (gate) ──┐
            ├──> M4 (apply/repair) ──> M5
M2 (check) ─┤
M3 (predl) ──┘
```

M1/M2/M3 are independent of each other; M4 depends on the bootstrap run path
being app_id-aware; M5 verifies all.
