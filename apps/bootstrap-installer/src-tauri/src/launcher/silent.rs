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
        Ok(r) if r.has_internet => {}
        Ok(_) => {
            tracing::info!("silent default: offline; skipping update check");
            return Ok(InstallOutcome::Installed);
        }
        Err(e) => {
            tracing::warn!(err = %e, "silent default: network probe failed; skipping update check");
            return Ok(InstallOutcome::Installed);
        }
    }

    // We have internet. Fetch the real HEAD SHA from the descriptor's repo
    // and compare against each installed app's commit. Mark `pending_updates`
    // for any app whose installed_commit != head_sha.
    let repo = repo_ref_from_descriptor(app);
    let head_sha = match super::update::update::fetch_head_sha(
        "https://api.github.com",
        &repo,
    )
    .await
    {
        Ok(sha) => sha,
        Err(e) => {
            tracing::warn!(
                err = %e,
                "silent default: fetch_head_sha failed; skipping background update check"
            );
            return Ok(InstallOutcome::Installed);
        }
    };

    let mut state = crate::launcher::state::load_launcher_state()
        .map_err(|e| e.to_string())?;

    for (id, inst) in &state.installed {
        if inst.installed_commit == head_sha {
            continue;
        }
        let entry = state
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
    state.last_update_check_at = Some(crate::launcher::commands::now_unix_iso());

    crate::launcher::state::save_launcher_state(&state)
        .map_err(|e| e.to_string())?;

    Ok(InstallOutcome::Installed)
}

/// Project an `AppDescriptor::default_repo_ref` into the `install_script::RepoRef`
/// shape `update::fetch_head_sha` expects. Centralized so the silent flow and
/// `commands::check_for_updates` can't drift apart.
fn repo_ref_from_descriptor(app: &crate::app::AppDescriptor) -> crate::install_script::RepoRef {
    let repo = &app.default_repo_ref.repo;
    crate::install_script::RepoRef {
        owner: repo.split('/').next().unwrap_or("").into(),
        name: repo.split('/').nth(1).unwrap_or("").into(),
        ref_name: app.default_repo_ref.r#ref.clone(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallOutcome {
    Installed,
    UpdateAvailable { version: String },
    Failed { reason: String },
}