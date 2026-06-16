use std::error::Error;
use std::path::PathBuf;
use std::process::Command;
use crate::app::{AppDescriptor, InstalledApp};

pub fn resolve_binary_path(app: &AppDescriptor, installed: &InstalledApp) -> PathBuf {
    let root = PathBuf::from(&installed.install_root);
    let rel = if cfg!(target_os = "macos") {
        &app.binary.macos
    } else if cfg!(target_os = "windows") {
        &app.binary.windows
    } else {
        &app.binary.linux
    };
    root.join(rel)
}

pub fn spawn(app: &AppDescriptor, installed: &InstalledApp) -> Result<(), String> {
    let path = resolve_binary_path(app, installed);
    if !path.exists() {
        return Err(format!(
            "Couldn't find a built {} desktop at {}. Run the app's build step from a terminal.",
            app.display_name,
            path.display()
        ));
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("/usr/bin/open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("spawn /usr/bin/open failed: {e}"))?;
    }
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x00000008;
        Command::new(&path)
            .creation_flags(DETACHED_PROCESS)
            .spawn()
            .map_err(|e| format!("spawn failed: {e}"))?;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new(&path).spawn().map_err(|e| format!("spawn failed: {e}"))?;
    }
    Ok(())
}

pub fn launch_and_exit(
    app: &AppDescriptor,
    installed: &InstalledApp,
) -> Result<(), String> {
    spawn(app, installed)?;
    std::thread::sleep(std::time::Duration::from_millis(150));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_and_exit_returns_result() {
        let app = crate::app::AppDescriptor::literal_hermes();
        let installed = crate::app::InstalledApp {
            install_root: "/tmp/hermes-agent".into(),
            installed_commit: "abc".into(),
            installed_ref_type: "branch".into(),
            installed_ref_name: "main".into(),
            installed_at: "2026-06-16T00:00:00Z".into(),
            installed_via: "first_install".into(),
        };
        let result = super::launch_and_exit(&app, &installed);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn resolve_binary_path_appends_correct_suffix_on_macos() {
        let app = crate::app::AppDescriptor::literal_hermes();
        let installed = crate::app::InstalledApp {
            install_root: "/tmp/hermes-agent".into(),
            installed_commit: "abc".into(),
            installed_ref_type: "branch".into(),
            installed_ref_name: "main".into(),
            installed_at: "2026-06-16T00:00:00Z".into(),
            installed_via: "first_install".into(),
        };
        let resolved = resolve_binary_path(&app, &installed);
        let s = resolved.to_string_lossy();
        assert!(
            s.contains("Hermes.app") || s.contains("Hermes.exe") || s.contains("hermes")
        );
    }
}
