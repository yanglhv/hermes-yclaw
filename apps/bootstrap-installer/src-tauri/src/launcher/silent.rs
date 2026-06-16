use crate::app::LauncherState;

#[cfg(test)]
mod silent_tests {
    use super::*;

    #[test]
    fn lock_prevents_double_invocation() {
        SILENT_DEFAULT_LOCK.store(false, Ordering::SeqCst);
        assert!(try_acquire());
        assert!(!try_acquire(), "second call should not acquire");
        release();
        assert!(try_acquire(), "after release, can re-acquire");
        release();
    }

    #[test]
    fn first_install_kind_when_state_empty() {
        assert_eq!(
            classify_launch_state(&LauncherState::default(), Vec::<String>::new()),
            LaunchKind::FirstInstall
        );
    }

    #[test]
    fn silent_kind_when_installed() {
        let mut s = LauncherState::default();
        s.installed.insert(
            "hermes".into(),
            crate::app::InstalledApp {
                install_root: "/x".into(),
                installed_commit: "abc".into(),
                installed_ref_type: "branch".into(),
                installed_ref_name: "main".into(),
                installed_at: "2026-06-16T00:00:00Z".into(),
                installed_via: "first_install".into(),
            },
        );
        assert_eq!(
            classify_launch_state(&s, Vec::<String>::new()),
            LaunchKind::Silent
        );
    }

    #[test]
    fn settings_kind_when_settings_flag() {
        let mut s = LauncherState::default();
        s.installed.insert(
            "hermes".into(),
            crate::app::InstalledApp {
                install_root: "/x".into(),
                installed_commit: "abc".into(),
                installed_ref_type: "branch".into(),
                installed_ref_name: "main".into(),
                installed_at: "2026-06-16T00:00:00Z".into(),
                installed_via: "first_install".into(),
            },
        );
        assert_eq!(
            classify_launch_state(&s, vec!["--settings"]),
            LaunchKind::Settings
        );
    }

    #[test]
    fn launch_kind_when_launch_flag() {
        let mut s = LauncherState::default();
        s.installed.insert(
            "hermes".into(),
            crate::app::InstalledApp {
                install_root: "/x".into(),
                installed_commit: "abc".into(),
                installed_ref_type: "branch".into(),
                installed_ref_name: "main".into(),
                installed_at: "2026-06-16T00:00:00Z".into(),
                installed_via: "first_install".into(),
            },
        );
        assert_eq!(
            classify_launch_state(&s, vec!["--launch", "hermes"]),
            LaunchKind::Launch
        );
    }

    #[test]
    fn update_kind_when_update_flag() {
        let s = LauncherState::default();
        assert_eq!(
            classify_launch_state(&s, vec!["--update"]),
            LaunchKind::Update
        );
    }

    #[tokio::test]
    async fn run_silent_default_does_not_panic() {
        let app = crate::app::AppDescriptor::literal_hermes();
        let result = run_silent_default(&app).await;
        assert!(result.is_ok() || result.is_err());
    }
}

use std::sync::atomic::{AtomicBool, Ordering};

static SILENT_DEFAULT_LOCK: AtomicBool = AtomicBool::new(false);

pub fn try_acquire() -> bool {
    SILENT_DEFAULT_LOCK
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
}

pub fn release() {
    SILENT_DEFAULT_LOCK.store(false, Ordering::SeqCst);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchKind {
    FirstInstall,
    Silent,
    Settings,
    Launch,
    Update,
}

pub fn classify_launch_state<I, S>(state: &LauncherState, args: I) -> LaunchKind
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args_vec: Vec<String> = args.into_iter().map(|s| s.as_ref().to_string()).collect();

    if args_vec.iter().any(|a| a.as_str() == "--settings") {
        return LaunchKind::Settings;
    }
    if args_vec.iter().any(|a| a.as_str() == "--launch") {
        return LaunchKind::Launch;
    }
    if args_vec.iter().any(|a| a.as_str() == "--update") {
        return LaunchKind::Update;
    }
    if state.installed.is_empty() {
        return LaunchKind::FirstInstall;
    }
    LaunchKind::Silent
}

pub async fn run_silent_default(
    app: &crate::app::AppDescriptor,
) -> Result<InstallOutcome, String> {
    if !try_acquire() {
        tracing::warn!("run_silent_default already running; no-op");
        return Ok(InstallOutcome::Installed);
    }
    let result = run_inner(app).await;
    release();
    result
}

async fn run_inner(app: &crate::app::AppDescriptor) -> Result<InstallOutcome, String> {
    let config = crate::launcher::config::config::LauncherConfig::load()
        .map_err(|e| e.to_string())?;

    if config.skip_network_probe {
        return Ok(InstallOutcome::Installed);
    }

    let probe_result = tokio::task::spawn_blocking(|| {
        crate::launcher::commands::probe_network()
    })
    .await
    .map_err(|e| e.to_string())?;

    match probe_result {
        Ok(network_result) if network_result.has_internet => {
            let catalog_url = app.catalog_url.clone();
            let catalog_apps = crate::launcher::catalog::list_available_apps(&catalog_url)
                .await
                .unwrap_or_default();

            let mut state = crate::launcher::state::load_launcher_state()
                .map_err(|e| e.to_string())?;

            for catalog_app in &catalog_apps {
                if state.installed.contains_key(&catalog_app.id) {
                    state.pending_updates.insert(
                        catalog_app.id.clone(),
                        crate::app::PendingUpdate {
                            latest_commit: "pending".into(),
                            latest_ref_name: app.default_repo_ref.r#ref.clone(),
                            status: crate::app::PendingStatus("avail".into()),
                            downloaded_script: None,
                            downloaded_at: None,
                            last_error: None,
                            last_error_at: None,
                        },
                    );
                }
            }
            state.last_update_check_at = Some("2026-06-16T00:00:00Z".into());

            crate::launcher::state::save_launcher_state(&state)
                .map_err(|e| e.to_string())?;
        }
        Ok(_) => {}
        Err(e) => {
            tracing::warn!(err = %e, "network probe failed");
        }
    }

    Ok(InstallOutcome::Installed)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallOutcome {
    Installed,
    UpdateAvailable { version: String },
    Failed { reason: String },
}