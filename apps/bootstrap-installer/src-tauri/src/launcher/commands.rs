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
    s.installed
        .iter()
        .map(|(id, inst)| LaunchableApp {
            descriptor: descriptor_for_id(id),
            installed: Some(inst.clone()),
            pending: s.pending_updates.get(id).cloned(),
            launcher_too_old: false,
        })
        .collect()
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
pub fn check_for_updates(state: State<'_, LauncherStateHandle>) -> Vec<String> {
    state.0.lock().unwrap().pending_updates.keys().cloned().collect()
}

#[tauri::command]
pub async fn pre_download_update(id: String, state: State<'_, LauncherStateHandle>) -> Result<(), String> {
    let app = crate::app::AppDescriptor::literal_hermes();
    let resolved = super::config::config::resolve();
    let install_script_repo = crate::install_script::RepoRef {
        owner: resolved.repo.split('/').next().unwrap_or("").into(),
        name: resolved.repo.split('/').nth(1).unwrap_or("").into(),
        ref_name: resolved.r#ref.clone(),
    };
    let latest_commit = "pending".to_string();

    let mut tmp_state = LauncherState::default();
    super::update::update::pre_download_update(&app, &install_script_repo, &latest_commit, &mut tmp_state)
        .await
        .map_err(|e| e.to_string())?;

    let mut s = state.0.lock().unwrap();
    if let Some(pu) = tmp_state.pending_updates.get(&app.id) {
        s.pending_updates.insert(app.id.clone(), pu.clone());
    }
    state::save_launcher_state(&s).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn apply_pending_update(id: String, state: State<'_, LauncherStateHandle>) -> Result<(), String> {
    let s = state.0.lock().unwrap();
    let _script = super::update::update::resolve_cached_script(&s, &id).ok_or_else(|| format!("No cached script for {id}"))?;
    tracing::info!(id, "apply_pending_update invoked (bootstrap deferred to M6)");
    Ok(())
}

pub fn load_initial_state() -> LauncherStateHandle {
    let mut s = state::load_launcher_state().unwrap_or_else(|e| {
        tracing::warn!(err = %e, "failed to load launcher state; using default");
        LauncherState::default()
    });
    if s.default_app_id.is_none() {
        s.default_app_id = Some("hermes".into());
    }
    LauncherStateHandle(std::sync::Mutex::new(s))
}
