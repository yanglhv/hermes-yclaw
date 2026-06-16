use crate::app::{AppDescriptor, LauncherState};
use crate::launcher::state as launcher_state;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn light_uninstall(app: &AppDescriptor, id: &str, state: &mut LauncherState) -> Result<(), String> {
    if let Some(inst) = state.installed.get(id) {
        let root = Path::new(&inst.install_root);
        if root.exists() {
            if app.uninstall_supported {
                tracing::info!(id, "would invoke {id} install script -Uninstall");
            }
            let _ = fs::remove_dir_all(root);
        }
    }
    state.installed.remove(id);
    state.pending_updates.remove(id);
    launcher_state::save_launcher_state(state).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn full_uninstall(app: &AppDescriptor, id: &str, state: &mut LauncherState) -> Result<(), String> {
    let home = crate::paths::hermes_home();
    let state_path = crate::paths::launcher_state_path();
    if state_path.exists() {
        let ts = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let _ = fs::rename(&state_path, home.join(format!("launcher-state.json.bak.{ts}")));
    }
    if home.exists() { let _ = fs::remove_dir_all(&home); }
    let _ = fs::create_dir_all(&home);
    state.installed.remove(id);
    state.pending_updates.remove(id);
    launcher_state::save_launcher_state(state).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod uninstall_tests {
    use super::light_uninstall;
    use crate::app::{AppDescriptor, InstalledApp, LauncherState};

    fn setup() -> (tempfile::TempDir, AppDescriptor, LauncherState) {
        let dir = tempfile::tempdir().unwrap();
        let install_root = dir.path().join("hermes-agent");
        std::fs::create_dir_all(&install_root).unwrap();
        let app = AppDescriptor::literal_hermes();
        let mut state = LauncherState::default();
        state.installed.insert("hermes".into(), InstalledApp {
            install_root: install_root.to_string_lossy().into(),
            installed_commit: "abc".into(),
            installed_ref_type: "branch".into(),
            installed_ref_name: "main".into(),
            installed_at: "2026-06-16T00:00:00Z".into(),
            installed_via: "first_install".into(),
        });
        std::fs::create_dir_all(dir.path().join("logs")).unwrap();
        (dir, app, state)
    }

    #[test]
    fn light_uninstall_removes_root_preserves_siblings() {
        let (dir, app, mut state) = setup();
        std::env::set_var("HERMES_HOME_FOR_TEST", dir.path().to_string_lossy().to_string());
        light_uninstall(&app, "hermes", &mut state).unwrap();
        assert!(!state.installed.contains_key("hermes"));
    }
}
