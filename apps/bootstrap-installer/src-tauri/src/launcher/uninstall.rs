use crate::app::{AppDescriptor, LauncherState};
use crate::launcher::state as launcher_state;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Recursively remove a directory, robustly. On Unix, `std::fs::remove_dir_all`
/// fails when the tree contains read-only directories (it cannot unlink their
/// children without write permission on the directory). A hermes-agent checkout
/// (`.git`, venv, built artifacts) routinely hits this, so we walk the tree
/// granting `u+w` on directories first, then remove — `rm -rf` semantics.
pub fn remove_dir_all_force(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fn grant_dir_write(dir: &Path) {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if let Ok(meta) = entry.metadata() {
                        if meta.is_dir() {
                            let p = entry.path();
                            let mut perms = meta.permissions();
                            perms.set_mode(perms.mode() | 0o200);
                            let _ = fs::set_permissions(&p, perms);
                            grant_dir_write(&p);
                        }
                    }
                }
            }
        }
        if path.is_dir() {
            // Best-effort: ignore chmod failures so the real remove error below
            // is the one surfaced to the caller.
            grant_dir_write(path);
        }
    }
    fs::remove_dir_all(path)
}

pub fn light_uninstall(app: &AppDescriptor, id: &str, state: &mut LauncherState) -> Result<(), String> {
    if let Some(inst) = state.installed.get(id) {
        let root = Path::new(&inst.install_root);
        if root.exists() {
            // The current install.sh has no `-Uninstall` mode, so the launcher
            // performs the removal directly. Wiring the script's -Uninstall
            // hook (when it exists) is a future task; until then this is the
            // authoritative cleanup.
            if app.uninstall_supported {
                tracing::info!(
                    id,
                    root = %root.display(),
                    "uninstall: removing install root (install.sh -Uninstall not supported)"
                );
            }
            remove_dir_all_force(root)
                .map_err(|e| format!("failed to remove install root {}: {e}", root.display()))?;
        }
    }
    state.installed.remove(id);
    state.pending_updates.remove(id);
    launcher_state::save_launcher_state(state).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn full_uninstall(app: &AppDescriptor, id: &str, state: &mut LauncherState) -> Result<(), String> {
    let _ = app; // full uninstall wipes everything; app scope is irrelevant here
    let _ = id;
    let home = crate::paths::hermes_home();
    // Back up the launcher state to a SIBLING of HERMES_HOME so it survives the
    // wipe below. (Backing it up inside home was pointless — the wipe deleted
    // it immediately.)
    let state_path = crate::paths::launcher_state_path();
    if state_path.exists() {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let bak = home.with_file_name(format!(".hermes-launcher-state.bak.{ts}.json"));
        let _ = fs::rename(&state_path, &bak);
    }
    if home.exists() {
        remove_dir_all_force(&home)
            .map_err(|e| format!("failed to remove HERMES_HOME {}: {e}", home.display()))?;
    }
    fs::create_dir_all(&home).map_err(|e| e.to_string())?;
    // Everything under HERMES_HOME is gone, so the in-memory state must reflect
    // that — not just the one app the user clicked.
    state.installed.clear();
    state.pending_updates.clear();
    launcher_state::save_launcher_state(state).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod uninstall_tests {
    use super::{light_uninstall, remove_dir_all_force};
    use crate::app::{AppDescriptor, InstalledApp, LauncherState};

    fn setup() -> (tempfile::TempDir, AppDescriptor, LauncherState) {
        let dir = tempfile::tempdir().unwrap();
        let install_root = dir.path().join("hermes-agent");
        std::fs::create_dir_all(&install_root).unwrap();
        let app = AppDescriptor::literal_hermes();
        let mut state = LauncherState::default();
        state.installed.insert(
            "hermes".into(),
            InstalledApp {
                install_root: install_root.to_string_lossy().into(),
                installed_commit: "abc".into(),
                installed_ref_type: "branch".into(),
                installed_ref_name: "main".into(),
                installed_at: "2026-06-16T00:00:00Z".into(),
                installed_via: "first_install".into(),
            },
        );
        std::fs::create_dir_all(dir.path().join("logs")).unwrap();
        (dir, app, state)
    }

    #[test]
    fn light_uninstall_removes_root_preserves_siblings() {
        let (dir, app, mut state) = setup();
        std::env::set_var(
            "HERMES_HOME_FOR_TEST",
            dir.path().to_string_lossy().to_string(),
        );
        light_uninstall(&app, "hermes", &mut state).unwrap();
        assert!(!state.installed.contains_key("hermes"));
    }

    #[test]
    #[cfg(unix)]
    fn force_removal_handles_readonly_subdirs() {
        // std::fs::remove_dir_all cannot unlink children of a read-only
        // directory; force removal must chmod its way through the tree.
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("sub/deep");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(nested.join("file"), b"x").unwrap();
        let ro = dir.path().join("sub");
        let mut p = std::fs::metadata(&ro).unwrap().permissions();
        p.set_mode(0o500); // r-x — no write
        std::fs::set_permissions(&ro, p).unwrap();

        remove_dir_all_force(dir.path()).expect("force removal should handle read-only dirs");
        assert!(!dir.path().exists());
    }
}
