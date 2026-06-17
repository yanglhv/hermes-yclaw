use crate::app::{AppDescriptor, LauncherState};
use crate::launcher::config::config::{LauncherConfig, load_yaml};
use crate::launcher::network;
use crate::launcher::state;
use tauri::State;
use url::Url;

pub struct LauncherStateHandle(pub std::sync::Mutex<LauncherState>);

#[derive(Debug, serde::Serialize)]
pub struct LaunchableApp {
    pub descriptor: AppDescriptor,
    pub installed: Option<crate::app::InstalledApp>,
    pub pending: Option<crate::app::PendingUpdate>,
    pub launcher_too_old: bool,
}

fn descriptor_for_id(id: &str) -> AppDescriptor {
    if id == "hermes" {
        return AppDescriptor::literal_hermes();
    }
    AppDescriptor {
        schema_version: 1,
        id: id.into(),
        app_id: id.into(),
        display_name: id.into(),
        category: "uncategorized".into(),
        default: false,
        script_path: "install.sh".into(),
        install_root: id.into(),
        binary: crate::app::BinaryPaths {
            windows: format!("{id}.exe"),
            macos: format!("{id}.app"),
            linux: id.into(),
        },
        uninstall_supported: false,
        app_settings_url: None,
        min_launcher_version: "0.1.0".into(),
        icon: None,
        default_repo_ref: crate::app::RepoRef {
            repo: "NousResearch/hermes-agent".into(),
            r#ref: "main".into(),
            repo_url_base: "https://github.com".into(),
        },
        catalog_url: Url::parse("https://api.github.com/repos/NousResearch/hermes-agent").unwrap(),
        icon_url: Url::parse("https://api.github.com/repos/NousResearch/hermes-agent/contents/icon.png").unwrap(),
        description: format!("App: {}", id),
        version: "1.0.0".into(),
        binaries: vec![],
        check_update_url: Url::parse("https://api.github.com/repos/NousResearch/hermes-agent/releases/latest").unwrap(),
    }
}

#[tauri::command]
pub fn list_available_apps(state: State<'_, LauncherStateHandle>) -> Vec<LaunchableApp> {
    let s = state.0.lock().unwrap();
    let result: Vec<LaunchableApp> = s
        .installed
        .iter()
        .map(|(id, inst)| {
            let descriptor = descriptor_for_id(id);
            let launcher_too_old = crate::app::semver_lt(
                env!("CARGO_PKG_VERSION"),
                descriptor.min_launcher_version.as_str(),
            );
            LaunchableApp {
                descriptor,
                installed: Some(inst.clone()),
                pending: s.pending_updates.get(id).cloned(),
                launcher_too_old,
            }
        })
        .collect();
    tracing::info!(
        count = result.len(),
        ids = ?s.installed.keys().collect::<Vec<_>>(),
        "list_available_apps"
    );
    result
}

#[tauri::command]
pub fn get_app(id: String, state: State<'_, LauncherStateHandle>) -> Option<LaunchableApp> {
    list_available_apps(state).into_iter().find(|a| a.descriptor.id == id)
}

#[tauri::command]
pub fn get_launcher_state(state: State<'_, LauncherStateHandle>) -> LauncherState {
    state.0.lock().unwrap().clone()
}

#[tauri::command]
pub fn get_launcher_config() -> Result<LauncherConfig, String> {
    load_yaml().ok_or_else(|| "no config".to_string())
}

#[tauri::command]
pub fn save_launcher_state(state: LauncherState) -> Result<(), String> {
    crate::launcher::state::save_launcher_state(&state)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_launcher_config(config: LauncherConfig) -> Result<(), String> {
    config.save().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_catalog_apps(catalog_url: String) -> Result<Vec<AppDescriptor>, String> {
    let url: Url = catalog_url.parse().map_err(|e: url::ParseError| e.to_string())?;
    let rt = tokio::runtime::Handle::current();
    rt.block_on(crate::launcher::catalog::list_available_apps(&url))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn probe_network() -> Result<network::NetworkProbeResult, String> {
    network::probe_network()
}

#[tauri::command]
pub fn set_default_app(id: String, state: State<'_, LauncherStateHandle>) -> Result<(), String> {
    let mut s = state.0.lock().unwrap();
    s.default_app_id = Some(id);
    state::save_launcher_state(&s).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_for_updates(state: State<'_, LauncherStateHandle>) -> Result<Vec<String>, String> {
    let repo = resolved_install_repo();
    // Offline / API failure: log + return current pending unchanged (no crash).
    let head_sha = match super::update::update::fetch_head_sha("https://api.github.com", &repo).await {
        Ok(sha) => sha,
        Err(e) => {
            tracing::warn!(err = %e, "check_for_updates: HEAD fetch failed; returning current pending");
            return Ok(state.0.lock().unwrap().pending_updates.keys().cloned().collect());
        }
    };
    let mut s = state.0.lock().unwrap();
    // Collect first so the immutable borrow of `s.installed` ends before we
    // mutably borrow `s.pending_updates` (Rust's borrow checker otherwise
    // refuses the simultaneous field borrows).
    let changed_ids: Vec<String> = s
        .installed
        .iter()
        .filter(|(_, inst)| inst.installed_commit != head_sha)
        .map(|(id, _)| id.clone())
        .collect();
    let mut changed: Vec<String> = Vec::new();
    for id in &changed_ids {
        changed.push(id.clone());
        let entry = s
            .pending_updates
            .entry(id.clone())
            .or_insert_with(|| crate::app::PendingUpdate {
                latest_commit: head_sha.clone(),
                latest_ref_name: repo.ref_name.clone(),
                status: crate::app::PendingStatus::Avail,
                downloaded_script: None,
                downloaded_at: None,
                last_error: None,
                last_error_at: None,
            });
        // Never downgrade an already-Ready/Downloading pre-download.
        if entry.status != crate::app::PendingStatus::Ready
            && entry.status != crate::app::PendingStatus::Downloading
        {
            entry.status = crate::app::PendingStatus::Avail;
            entry.latest_commit = head_sha.clone();
            entry.latest_ref_name = repo.ref_name.clone();
        }
    }
    let _ = state::save_launcher_state(&s);
    Ok(changed)
}

/// Resolve the launcher config's repo into the `install_script::RepoRef` used
/// by update-check / pre-download. Centralized so the two stay in sync.
fn resolved_install_repo() -> crate::install_script::RepoRef {
    let resolved = super::config::config::resolve();
    crate::install_script::RepoRef {
        owner: resolved.repo.split('/').next().unwrap_or("").into(),
        name: resolved.repo.split('/').nth(1).unwrap_or("").into(),
        ref_name: resolved.r#ref.clone(),
    }
}

#[tauri::command]
pub async fn pre_download_update(id: String, state: State<'_, LauncherStateHandle>) -> Result<(), String> {
    let _ = id; // scoped to hermes (single registered app); descriptor resolves below
    let descriptor = AppDescriptor::literal_hermes();
    let repo = resolved_install_repo();
    // Real HEAD SHA — also used as the cache key by the helper.
    let head_sha = super::update::update::fetch_head_sha("https://api.github.com", &repo)
        .await
        .map_err(|e| e.to_string())?;
    // Operate on a CLONE so the std MutexGuard is not held across the await.
    // (The previous impl used a throwaway tmp_state that the helper then saved,
    // which clobbered `installed` in the on-disk state — a latent data-loss bug.)
    let mut s = state.0.lock().unwrap().clone();
    super::update::update::pre_download_update(&descriptor, &repo, &head_sha, &mut s)
        .await
        .map_err(|e| e.to_string())?;
    // Helper persisted `s` (full state) to disk; sync the managed in-memory copy.
    *state.0.lock().unwrap() = s;
    Ok(())
}

#[tauri::command]
pub async fn apply_pending_update(
    id: String,
    state: State<'_, LauncherStateHandle>,
    app: tauri::AppHandle,
    app_state: tauri::State<'_, std::sync::Arc<crate::AppState>>,
) -> Result<(), String> {
    // Require a Ready pending update with a cached script on disk.
    let (commit, ref_name, cached_script) = {
        let s = state.0.lock().unwrap();
        let p = s
            .pending_updates
            .get(&id)
            .ok_or_else(|| format!("No pending update for {id}"))?;
        if p.status != crate::app::PendingStatus::Ready {
            return Err(format!(
                "Pending update for {id} not ready (status={:?})",
                p.status
            ));
        }
        let cached = super::update::update::resolve_cached_script(&s, &id)
            .ok_or_else(|| format!("Cached script missing for {id}"))?;
        (p.latest_commit.clone(), p.latest_ref_name.clone(), cached)
    };
    // Align the pre-downloaded (commit-keyed) cache with the ref-keyed path
    // run_bootstrap checks, so apply reuses the exact pre-downloaded bytes
    // instead of a fresh fetch.
    let kind = crate::install_script::ScriptKind::for_current_os();
    let target = crate::install_script::cached_path(kind, &ref_name);
    if target != cached_script && cached_script.exists() {
        if let Some(parent) = target.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::copy(&cached_script, &target);
    }
    let descriptor = AppDescriptor::literal_hermes();
    let args = crate::bootstrap::StartBootstrapArgs {
        commit: None,
        branch: Some(ref_name.clone()),
        include_desktop: true,
        hermes_home: None,
    };
    crate::bootstrap::run_app_install(app, app_state, descriptor, args, id, "update".into(), commit, ref_name).await
}

pub fn load_initial_state() -> LauncherStateHandle {
    let mut s = state::load_launcher_state().unwrap_or_else(|e| {
        tracing::warn!(err = %e, "failed to load launcher state; using default");
        LauncherState::default()
    });
    if s.default_app_id.is_none() {
        s.default_app_id = Some("hermes".into());
    }
    // Spec V15 (launcher-app-registry "Existing Install Detection"): if the
    // default app's install root is present on disk but not yet recorded in
    // launcher-state, backfill an InstalledApp entry so the Home tile appears.
    // The gate is "install root exists as a populated directory" — NOT
    // `hermes_is_installed` (which also requires the bootstrap-complete marker
    // + built binary). A partially-removed install (e.g. a prior uninstall
    // that deleted the marker before failing) must still be detected so the
    // user sees the tile and can Repair/Launch/Uninstall it. Persist only when
    // a new entry was inserted.
    let app = AppDescriptor::literal_hermes();
    let install_root = crate::paths::hermes_home().join(&app.install_root);
    let installed = install_root.is_dir()
        && std::fs::read_dir(&install_root)
            .map(|mut it| it.next().is_some())
            .unwrap_or(false);
    if backfill_default_install(&mut s, &install_root, installed) {
        if let Err(e) = state::save_launcher_state(&s) {
            tracing::warn!(err = %e, "failed to persist launcher state after install detection");
        }
    }
    LauncherStateHandle(std::sync::Mutex::new(s))
}

/// Resolved launch mode surfaced to the frontend so it can pick its initial
/// route (see `src/lib/launcher-mode.ts`). Mirrors the determination in the
/// `launcher-cli-modes` spec.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LaunchMode {
    pub kind: LaunchModeKind,
    pub target_app_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LaunchModeKind {
    FirstInstall,
    Settings,
    Launch,
    Update,
    Silent,
}

/// Pure determination of the launch mode from CLI args + whether any app is
/// recorded as installed. Kept free of Tauri state so it is unit-testable and
/// so the setup hook and the `get_launch_mode` command cannot drift apart.
///
/// Precedence matches `specs/launcher-cli-modes/spec.md`:
/// `--update` > `--launch <id>` > `--settings` > `--repair`/`--reinstall` >
/// empty-install-state (first_install) > silent.
pub fn resolve_launch_mode(args: &[String], installed_is_empty: bool) -> LaunchMode {
    if args.iter().any(|a| a == "--update") {
        return LaunchMode { kind: LaunchModeKind::Update, target_app_id: None };
    }
    if let Some(id) = crate::launch_args_from_args(args.iter().map(String::as_str)) {
        return LaunchMode { kind: LaunchModeKind::Launch, target_app_id: Some(id) };
    }
    if args.iter().any(|a| a == "--settings") {
        return LaunchMode { kind: LaunchModeKind::Settings, target_app_id: None };
    }
    // --repair / --reinstall force the full UI; the spec routes them through
    // the same `settings` kind so the frontend renders the welcome/repair flow.
    if args.iter().any(|a| a == "--repair" || a == "--reinstall") {
        return LaunchMode { kind: LaunchModeKind::Settings, target_app_id: None };
    }
    if installed_is_empty {
        return LaunchMode { kind: LaunchModeKind::FirstInstall, target_app_id: None };
    }
    LaunchMode { kind: LaunchModeKind::Silent, target_app_id: None }
}

#[tauri::command]
pub fn get_launch_mode(state: State<'_, LauncherStateHandle>) -> LaunchMode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let installed_is_empty = state.0.lock().unwrap().installed.is_empty();
    resolve_launch_mode(&args, installed_is_empty)
}

#[cfg(test)]
mod launch_mode_tests {
    use super::*;

    #[test]
    fn bare_launch_with_no_installs_is_first_install() {
        let m = resolve_launch_mode(&[], true);
        assert_eq!(m.kind, LaunchModeKind::FirstInstall);
        assert_eq!(m.target_app_id, None);
    }

    #[test]
    fn bare_launch_with_installs_is_silent() {
        let m = resolve_launch_mode(&[], false);
        assert_eq!(m.kind, LaunchModeKind::Silent);
    }

    #[test]
    fn settings_flag_is_settings() {
        let m = resolve_launch_mode(&["--settings".into()], false);
        assert_eq!(m.kind, LaunchModeKind::Settings);
    }

    #[test]
    fn repair_and_reinstall_route_to_settings() {
        assert_eq!(
            resolve_launch_mode(&["--repair".into()], false).kind,
            LaunchModeKind::Settings
        );
        assert_eq!(
            resolve_launch_mode(&["--reinstall".into()], false).kind,
            LaunchModeKind::Settings
        );
    }

    #[test]
    fn launch_flag_carries_target() {
        let m = resolve_launch_mode(&["--launch".into(), "hermes".into()], false);
        assert_eq!(m.kind, LaunchModeKind::Launch);
        assert_eq!(m.target_app_id.as_deref(), Some("hermes"));
    }

    #[test]
    fn update_flag_takes_precedence() {
        let m = resolve_launch_mode(&["--update".into(), "--settings".into()], false);
        assert_eq!(m.kind, LaunchModeKind::Update);
    }

    #[test]
    fn serializes_kind_as_snake_case() {
        let m = resolve_launch_mode(&[], true);
        let json = serde_json::to_string(&m).unwrap();
        assert!(json.contains("\"first_install\""), "json was: {json}");
    }
}

// ---------------------------------------------------------------------------
// Existing-install detection (spec V15)
// ---------------------------------------------------------------------------

/// Reads the HEAD commit of an on-disk git checkout, or `"unknown"` when the
/// directory is not a git checkout (or git is unavailable). Kept pure so it
/// is unit-testable against a tempdir.
fn read_checkout_commit(install_root: &std::path::Path) -> String {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(install_root)
        .arg("rev-parse")
        .arg("HEAD")
        .output();
    match out {
        Ok(o) if o.status.success() => {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() {
                "unknown".into()
            } else {
                s
            }
        }
        _ => "unknown".into(),
    }
}

/// Unix-seconds timestamp, matching the convention used by
/// `launcher::state::now_iso` so all launcher timestamps share one format.
fn now_unix_iso() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}Z")
}

/// Backfill `state.installed[default_app_id]` from an on-disk install that is
/// not yet recorded (spec V15). `installed` is the caller's verdict from
/// `bootstrap::hermes_is_installed`, passed in so this function stays free of
/// real-filesystem/env access and is unit-testable. Returns true iff a new
/// entry was inserted.
pub fn backfill_default_install(
    state: &mut LauncherState,
    install_root: &std::path::Path,
    installed: bool,
) -> bool {
    if !installed {
        return false;
    }
    let app = AppDescriptor::literal_hermes();
    let id = state
        .default_app_id
        .clone()
        .unwrap_or_else(|| app.id.clone());
    if state.installed.contains_key(&id) {
        return false;
    }
    let commit = read_checkout_commit(install_root);
    state.installed.insert(
        id.clone(),
        crate::app::InstalledApp {
            install_root: install_root.to_string_lossy().into(),
            installed_commit: commit,
            installed_ref_type: "branch".into(),
            installed_ref_name: app.default_repo_ref.r#ref.clone(),
            installed_at: now_unix_iso(),
            installed_via: "detected".into(),
        },
    );
    tracing::info!(
        %id,
        root = %install_root.display(),
        "detected existing install; backfilled launcher-state"
    );
    true
}

// ---------------------------------------------------------------------------
// Per-app lifecycle commands (spec M3.3 / M6.4)
// ---------------------------------------------------------------------------

/// Launch the installed app's desktop binary, then exit the installer process
/// (spec launcher-multi-app-install "Generalized app launch").
#[tauri::command]
pub fn launch_app(
    id: String,
    state: State<'_, LauncherStateHandle>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let inst = {
        let s = state.0.lock().unwrap();
        s.installed
            .get(&id)
            .cloned()
            .ok_or_else(|| format!("app '{id}' is not installed"))?
    };
    let descriptor = AppDescriptor::literal_hermes();
    crate::launcher::launch::launch_and_exit(&descriptor, &inst)?;
    app.exit(0);
    Ok(())
}

/// Uninstall an app. `scope` is `"light"` (delete install root, preserve
/// HERMES_HOME) or `"full"` (delete all of HERMES_HOME after backing up
/// state). Spec launcher-uninstall-repair.
#[tauri::command]
pub fn uninstall_app(
    id: String,
    scope: String,
    state: State<'_, LauncherStateHandle>,
) -> Result<(), String> {
    let mut s = state.0.lock().unwrap();
    let app = AppDescriptor::literal_hermes();
    match scope.as_str() {
        "light" => crate::launcher::uninstall::light_uninstall(&app, &id, &mut s),
        "full" => crate::launcher::uninstall::full_uninstall(&app, &id, &mut s),
        other => Err(format!(
            "invalid uninstall scope: {other} (expected 'light' or 'full')"
        )),
    }
}

/// Repair re-runs the install flow for an app. Uses a cached pending-update
/// script when one exists; otherwise a fresh fetch is required. The full
/// bootstrap apply is deferred (M6), so this resolves the script and records
/// intent without erroring. Spec launcher-uninstall-repair "Repair".
#[tauri::command]
pub async fn repair_app(
    id: String,
    app: tauri::AppHandle,
    app_state: tauri::State<'_, std::sync::Arc<crate::AppState>>,
) -> Result<(), String> {
    // Repair runs the install unconditionally. The bootstrap's
    // cached→network script resolution handles "use pending cache if present,
    // else fresh fetch" automatically, so no precondition is needed here.
    let repo = resolved_install_repo();
    let head_sha = super::update::update::fetch_head_sha("https://api.github.com", &repo)
        .await
        .map_err(|e| format!("repair: could not resolve HEAD commit: {e}"))?;
    let descriptor = AppDescriptor::literal_hermes();
    let args = crate::bootstrap::StartBootstrapArgs {
        commit: None,
        branch: Some(repo.ref_name.clone()),
        include_desktop: true,
        hermes_home: None,
    };
    crate::bootstrap::run_app_install(
        app,
        app_state,
        descriptor,
        args,
        id,
        "repair".into(),
        head_sha,
        repo.ref_name,
    )
    .await
}

/// Open the app's external settings URL via the opener plugin. Returns Err if
/// the app exposes no settings URL. Spec launcher-uninstall-repair "Open app
/// settings".
#[tauri::command]
pub fn open_app_settings(id: String, app: tauri::AppHandle) -> Result<(), String> {
    let descriptor = AppDescriptor::literal_hermes();
    let _ = &id;
    let url = descriptor
        .app_settings_url
        .as_ref()
        .ok_or_else(|| format!("App '{id}' does not expose a settings URL"))?;
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod install_detection_tests {
    use super::*;
    use crate::app::InstalledApp;

    #[test]
    fn read_commit_is_unknown_for_non_git_dir() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(read_checkout_commit(dir.path()), "unknown");
    }

    #[test]
    fn backfill_inserts_entry_when_installed() {
        let dir = tempfile::tempdir().unwrap();
        let mut state = LauncherState::default();
        state.default_app_id = Some("hermes".into());
        assert!(backfill_default_install(&mut state, dir.path(), true));
        let inst = state
            .installed
            .get("hermes")
            .expect("hermes should be backfilled");
        assert_eq!(inst.installed_via, "detected");
        assert_eq!(inst.installed_commit, "unknown");
        assert_eq!(inst.installed_ref_name, "main");
    }

    #[test]
    fn backfill_noop_when_not_installed() {
        let mut state = LauncherState::default();
        state.default_app_id = Some("hermes".into());
        assert!(!backfill_default_install(&mut state, std::path::Path::new("/x"), false));
        assert!(state.installed.is_empty());
    }

    #[test]
    fn backfill_noop_when_already_recorded() {
        let mut state = LauncherState::default();
        state.default_app_id = Some("hermes".into());
        state.installed.insert(
            "hermes".into(),
            InstalledApp {
                install_root: "/preexisting".into(),
                installed_commit: "abc".into(),
                installed_ref_type: "branch".into(),
                installed_ref_name: "main".into(),
                installed_at: "0Z".into(),
                installed_via: "first_install".into(),
            },
        );
        assert!(!backfill_default_install(&mut state, std::path::Path::new("/x"), true));
        // Pre-existing entry is untouched.
        assert_eq!(state.installed["hermes"].installed_via, "first_install");
    }
}
