use crate::app::{InstalledApp, LauncherState, PendingStatus, PendingUpdate};
use crate::paths::launcher_state_path;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launcher_state_default_is_empty() {
        let state = LauncherState::default();
        assert!(state.installed.is_empty());
    }

    #[test]
    fn installed_app_round_trips() {
        let app = InstalledApp {
            install_root: "hermes-agent".into(),
            installed_commit: "abc123".into(),
            installed_ref_type: "branch".into(),
            installed_ref_name: "main".into(),
            installed_at: "2026-06-16T00:00:00Z".into(),
            installed_via: "first_install".into(),
        };
        let json = serde_json::to_string(&app).unwrap();
        assert!(json.contains("abc123"));
    }

    #[test]
    fn read_or_default_recovers_from_corrupt_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("launcher-state.json");
        std::fs::write(&path, b"{{ not json").unwrap();
        let s: LauncherState = read_or_default(&path);
        assert_eq!(s, LauncherState { schema_version: 1, ..Default::default() });
        let backup: Vec<_> = std::fs::read_dir(dir.path()).unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains("launcher-state.json.bak."))
            .collect();
        assert_eq!(backup.len(), 1);
    }

    #[test]
    fn validate_pending_cache_degrades_missing_file_to_failed() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = LauncherState::default();
        s.pending_updates.insert("hermes".into(), PendingUpdate {
            latest_commit: "abc".into(),
            latest_ref_name: "main".into(),
            status: PendingStatus::Ready,
            downloaded_script: Some(dir.path().join("missing.ps1").to_string_lossy().into()),
            downloaded_at: Some("2026-06-16T00:00:00Z".into()),
            last_error: None,
            last_error_at: None,
        });
        validate_pending_cache(&mut s);
        assert_eq!(s.pending_updates["hermes"].status, PendingStatus::Failed);
    }

    #[test]
    fn validate_pending_cache_keeps_existing_file_ready() {
        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("install.ps1");
        std::fs::write(&script, "#!/usr/bin/env pwsh\n").unwrap();
        let mut s = LauncherState::default();
        s.pending_updates.insert("hermes".into(), PendingUpdate {
            latest_commit: "abc".into(),
            latest_ref_name: "main".into(),
            status: PendingStatus::Ready,
            downloaded_script: Some(script.to_string_lossy().into()),
            downloaded_at: Some("2026-06-16T00:00:00Z".into()),
            last_error: None,
            last_error_at: None,
        });
        validate_pending_cache(&mut s);
        assert_eq!(s.pending_updates["hermes"].status, PendingStatus::Ready);
    }
}

pub fn read_or_default(path: &Path) -> LauncherState {
    match fs::read_to_string(path) {
        Ok(c) => match serde_json::from_str::<LauncherState>(&c) {
            Ok(mut s) => {
                s.schema_version = 1;
                validate_pending_cache(&mut s);
                s
            }
            Err(err) => {
                tracing::warn!(?err, path = %path.display(), "launcher state corrupt; backing up");
                backup_corrupt(path);
                let fresh = LauncherState { schema_version: 1, ..Default::default() };
                let _ = write_atomic(path, &fresh);
                fresh
            }
        },
        Err(_) => LauncherState { schema_version: 1, ..Default::default() },
    }
}

pub fn load_launcher_state() -> Result<LauncherState> {
    Ok(read_or_default(&launcher_state_path()))
}

pub fn write_atomic(path: &Path, state: &LauncherState) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("json.tmp");
    let mut f = fs::File::create(&tmp)?;
    f.write_all(&serde_json::to_vec_pretty(state).map_err(std::io::Error::other)?)?;
    f.sync_all()?;
    drop(f);
    fs::rename(&tmp, path)
}

pub fn save_launcher_state(state: &LauncherState) -> Result<()> {
    write_atomic(&launcher_state_path(), state).map_err(|e| e.into())
}

pub fn validate_pending_cache(state: &mut LauncherState) {
    for p in state.pending_updates.values_mut() {
        if p.status != PendingStatus::Ready {
            continue;
        }
        if let Some(path) = &p.downloaded_script {
            if !std::path::Path::new(path).exists() {
                p.status = PendingStatus::Failed;
                p.last_error = Some("cached script missing".into());
                p.last_error_at = Some(now_iso());
            }
        }
    }
}

fn backup_corrupt(path: &Path) {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let backup = path.with_file_name(format!(
        "{}.bak.{}",
        path.file_name().and_then(|s| s.to_str()).unwrap_or("state"),
        ts
    ));
    let _ = fs::rename(path, backup);
}

fn now_iso() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{}Z", secs)
}

#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error("io error: {0}")] Io(#[from] std::io::Error),
    #[error("json error: {0}")] Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, StateError>;
