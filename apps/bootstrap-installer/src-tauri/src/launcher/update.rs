pub mod update {
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};
    use crate::app::{AppDescriptor, LauncherState, PendingStatus, PendingUpdate};
    use crate::install_script::{self, RepoRef};

    #[derive(Debug, thiserror::Error)]
    pub enum UpdateError {
        #[error("network error: {0}")] NetworkError(#[from] reqwest::Error),
        #[error("script not found")] ScriptNotFound,
        #[error("IO error: {0}")] IoError(#[from] std::io::Error),
        #[error("parse error: {0}")] ParseError(String),
    }

    pub async fn pre_download_update(app: &AppDescriptor, repo: &RepoRef, latest_commit: &str, state: &mut LauncherState) -> Result<PendingUpdate, UpdateError> {
        let cache_dir = crate::paths::bootstrap_cache_dir();
        std::fs::create_dir_all(&cache_dir).map_err(|e| std::io::Error::other(e))?;
        pre_download_update_to(app, repo, latest_commit, &cache_dir, state).await
            .map_err(UpdateError::ParseError)?;
        Ok(state.pending_updates.get(&app.id).cloned().unwrap_or_else(|| PendingUpdate {
            latest_commit: latest_commit.into(),
            latest_ref_name: repo.ref_name.clone(),
            status: PendingStatus::Ready,
            downloaded_script: None,
            downloaded_at: None,
            last_error: None,
            last_error_at: None,
        }))
    }

    pub async fn pre_download_update_to(app: &AppDescriptor, repo: &RepoRef, latest_commit: &str, cache_dir: &Path, state: &mut LauncherState) -> Result<(), String> {
        let entry = state.pending_updates.entry(app.id.clone()).or_insert_with(|| PendingUpdate {
            latest_commit: latest_commit.into(),
            latest_ref_name: repo.ref_name.clone(),
            status: PendingStatus::Avail,
            downloaded_script: None,
            downloaded_at: None,
            last_error: None,
            last_error_at: None,
        });
        entry.status = PendingStatus::Downloading;
        entry.latest_commit = latest_commit.into();
        entry.latest_ref_name = repo.ref_name.clone();
        let cached = cache_dir.join(format!("install-{}.{}", sanitize(latest_commit), app.script_extension()));
        match install_script::resolve(repo, &app.script_path, &cached).await {
            Ok(path) => {
                entry.status = PendingStatus::Ready;
                entry.downloaded_script = Some(path.path.to_string_lossy().into());
                entry.downloaded_at = Some(now_iso());
                entry.last_error = None;
                entry.last_error_at = None;
            }
            Err(err) => {
                entry.status = PendingStatus::Failed;
                entry.last_error = Some(format!("{err:?}"));
                entry.last_error_at = Some(now_iso());
            }
        }
        let _ = crate::launcher::state::save_launcher_state(state);
        Ok(())
    }

    pub fn resolve_cached_script(state: &LauncherState, id: &str) -> Option<PathBuf> {
        let p = state.pending_updates.get(id)?;
        if p.status != PendingStatus::Ready {
            return None;
        }
        let path = p.downloaded_script.as_ref()?;
        let pb = PathBuf::from(path);
        if pb.exists() {
            Some(pb)
        } else {
            None
        }
    }

    fn sanitize(s: &str) -> String {
        s.chars().take(40).map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect()
    }

    fn now_iso() -> String {
        let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        format!("{}Z", secs)
    }

    #[cfg(test)]
    mod update_tests {
        use crate::app::{AppDescriptor, LauncherState, PendingStatus, PendingUpdate};
        use crate::install_script::RepoRef;

        #[tokio::test]
        async fn pre_download_writes_ready_when_fetch_succeeds() {
            let _server = mockito::Server::new_async().await;
            let dir = tempfile::tempdir().unwrap();
            let cached_file = dir.path().join("install-def456.ps1");
            std::fs::write(&cached_file, "#!/usr/bin/env pwsh\nWrite-Host 'hi'").unwrap();
            let mut state = LauncherState::default();
            let app = AppDescriptor::literal_hermes();
            let repo = RepoRef { owner: "o".into(), name: "n".into(), ref_name: "main".into() };
            let r = super::pre_download_update_to(&app, &repo, "def456", dir.path(), &mut state).await;
            assert!(r.is_ok(), "pre_download: {r:?}");
            let p = state.pending_updates.get("hermes").expect("pending");
            assert_eq!(p.status, PendingStatus::Ready);
            assert!(p.downloaded_script.is_some());
        }

        #[tokio::test]
        async fn pre_download_records_failed_on_http_error() {
            let mut server = mockito::Server::new_async().await;
            let _m = server.mock("GET", mockito::Matcher::Regex(r".*".into()))
                .with_status(500).create_async().await;
            let dir = tempfile::tempdir().unwrap();
            let mut state = LauncherState::default();
            let app = AppDescriptor::literal_hermes();
            let repo = RepoRef { owner: "o".into(), name: "n".into(), ref_name: "main".into() };
            let _ = super::pre_download_update_to(&app, &repo, "def456", dir.path(), &mut state).await;
            let p = state.pending_updates.get("hermes").expect("pending");
            assert_eq!(p.status, PendingStatus::Failed);
        }
    }
}
