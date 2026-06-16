//! Resolves and downloads `scripts/install.ps1` (and `install.sh`).
//!
//! Resolution order:
//!   1. Dev shortcut: a sibling repo checkout via $HERMES_SETUP_DEV_REPO_ROOT
//!      env var. Lets devs iterate without re-publishing the script.
//!   2. Bundled fallback: if the installer was bundled with a script (e.g.
//!      tauri's `resource` mechanism), serve from there. Not used today.
//!   3. Network: download from GitHub raw at a pinned commit or branch.
//!      Commit pins are immutable; branch pins are HEAD-tracking.
//!
//! Mirrors `apps/desktop/electron/bootstrap-runner.cjs`'s `resolveInstallScript`,
//! but the dev-checkout resolution is driven by an env var rather than the
//! Electron app's APP_ROOT/../.. trick, because Hermes-Setup.exe is meant
//! to live OUTSIDE any repo checkout.

use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

use crate::paths;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RepoRef {
    pub owner: String,
    pub name: String,
    pub ref_name: String,
}

impl RepoRef {
    pub fn hardcoded_default() -> Self {
        Self {
            owner: "NousResearch".into(),
            name: "hermes-agent".into(),
            ref_name: "main".into(),
        }
    }
}

/// Identity of the install.ps1 we'll execute. Used by both the manifest
/// fetch and the per-stage runs.
#[derive(Debug, Clone)]
pub struct ResolvedScript {
    pub path: PathBuf,
    pub source: ScriptSource,
    /// Commit pin (40-char SHA) if known. install.ps1's `-Commit` arg is
    /// what makes the repo stage clone the exact tested SHA.
    pub commit: Option<String>,
    pub branch: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptSource {
    DevCheckout,
    Bundled,
    Cached,
    Downloaded,
}

/// What flavor of script (Windows .ps1 vs Unix .sh).
#[derive(Debug, Clone, Copy)]
pub enum ScriptKind {
    Ps1,
    Sh,
}

impl ScriptKind {
    pub fn for_current_os() -> Self {
        if cfg!(target_os = "windows") {
            Self::Ps1
        } else {
            Self::Sh
        }
    }

    fn filename(&self) -> &'static str {
        match self {
            Self::Ps1 => "install.ps1",
            Self::Sh => "install.sh",
        }
    }
}

/// Validates a string looks like a git SHA (7+ hex chars). Mirrors
/// `STAMP_COMMIT_RE` from bootstrap-runner.cjs.
fn is_valid_commit(s: &str) -> bool {
    let len = s.len();
    (7..=40).contains(&len) && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// Resolves the install script to use for this run.
///
/// `repo` provides the GitHub owner/name/ref; `app_relative_script_path`
/// is the path under that repo's scripts/ directory (e.g. `"install.ps1"`).
pub async fn resolve(
    repo: &RepoRef,
    app_relative_script_path: &str,
    cached_path: &Path,
) -> Result<ResolvedScript> {
    // 1. Dev shortcut.
    if let Ok(repo_root) = std::env::var("HERMES_SETUP_DEV_REPO_ROOT") {
        let candidate = PathBuf::from(repo_root).join(app_relative_script_path);
        if candidate.exists() {
            return Ok(ResolvedScript {
                path: candidate,
                source: ScriptSource::DevCheckout,
                commit: Some(repo.ref_name.clone()),
                branch: None,
            });
        }
    }

    // 2. (Not implemented) bundled fallback.

    // 3. Network. ref_name must be a real commit or a branch ref.
    let commit_or_ref = repo.ref_name.clone();
    if !is_valid_commit(&commit_or_ref) && commit_or_ref.trim().is_empty() {
        return Err(anyhow!(
            "install script ref `{}` is not a valid git SHA or branch name",
            commit_or_ref
        ));
    }

    if cached_path.exists() {
        return Ok(ResolvedScript {
            path: cached_path.to_path_buf(),
            source: ScriptSource::Cached,
            commit: Some(commit_or_ref.clone()),
            branch: None,
        });
    }

    download(repo, app_relative_script_path, cached_path).await?;

    Ok(ResolvedScript {
        path: cached_path.to_path_buf(),
        source: ScriptSource::Downloaded,
        commit: Some(commit_or_ref),
        branch: None,
    })
}

#[derive(Debug, Clone, Default)]
pub struct Pin {
    pub commit: Option<String>,
    pub branch: Option<String>,
}

pub fn cached_path(kind: ScriptKind, commit_or_ref: &str) -> PathBuf {
    let safe = sanitize_ref(commit_or_ref);
    let filename = match kind {
        ScriptKind::Ps1 => format!("install-{safe}.ps1"),
        ScriptKind::Sh => format!("install-{safe}.sh"),
    };
    paths::bootstrap_cache_dir().join(filename)
}

/// Replace anything that's not [A-Za-z0-9._-] with `_`. Branch refs can
/// contain `/`, dots, etc.; we want a flat filename.
fn sanitize_ref(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn truncate_ref(s: &str) -> &str {
    if is_valid_commit(s) && s.len() >= 12 {
        &s[..12]
    } else {
        s
    }
}

pub fn build_raw_url(repo: &RepoRef, app_relative_script_path: &str) -> String {
    format!(
        "https://raw.githubusercontent.com/{}/{}/{}/{}",
        repo.owner, repo.name, repo.ref_name, app_relative_script_path
    )
}

/// Downloads to `dest_path` via reqwest with rustls. Atomically renames
/// `dest_path.tmp` → `dest_path` so partial writes don't poison the cache.
async fn download(repo: &RepoRef, app_relative_script_path: &str, dest_path: &Path) -> Result<()> {
    let url = build_raw_url(repo, app_relative_script_path);

    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!("creating bootstrap-cache parent dir {}", parent.display())
        })?;
    }

    let tmp_path = dest_path.with_extension({
        let ext = dest_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("tmp");
        format!("{ext}.tmp")
    });

    let response = reqwest::Client::new()
        .get(&url)
        .header("User-Agent", "hermes-setup/0.0.1")
        .send()
        .await
        .with_context(|| format!("GET {url}"))?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Failed to download {}: HTTP {} from {}",
            app_relative_script_path,
            response.status(),
            url
        ));
    }

    let bytes = response
        .bytes()
        .await
        .with_context(|| format!("reading body of {url}"))?;

    let mut file = tokio::fs::File::create(&tmp_path)
        .await
        .with_context(|| format!("creating temp file {}", tmp_path.display()))?;
    file.write_all(&bytes)
        .await
        .with_context(|| format!("writing temp file {}", tmp_path.display()))?;
    file.flush().await.context("flushing temp file")?;
    drop(file);

    tokio::fs::rename(&tmp_path, dest_path)
        .await
        .with_context(|| {
            format!(
                "renaming {} → {}",
                tmp_path.display(),
                dest_path.display()
            )
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_valid_commit_accepts_short_and_full_shas() {
        assert!(is_valid_commit("02d26981d3d4ad50e142399b8476f59ad5953ff0"));
        assert!(is_valid_commit("02d2698"));
        assert!(!is_valid_commit("02d269"));
        assert!(!is_valid_commit("not-a-sha"));
        assert!(!is_valid_commit(""));
    }

    #[test]
    fn sanitize_ref_replaces_slashes() {
        assert_eq!(sanitize_ref("bb/gui"), "bb_gui");
        assert_eq!(sanitize_ref("main"), "main");
        assert_eq!(sanitize_ref("release/1.2.3"), "release_1.2.3");
    }

    #[test]
    fn repo_ref_hardcoded_default_matches_legacy() {
        let r = super::RepoRef::hardcoded_default();
        assert_eq!(r.owner, "NousResearch");
        assert_eq!(r.name, "hermes-agent");
        assert_eq!(r.ref_name, "main");
    }

    #[test]
    fn build_raw_url_uses_repo_ref() {
        let r = super::RepoRef {
            owner: "alice".into(),
            name: "x".into(),
            ref_name: "main".into(),
        };
        let url = super::build_raw_url(&r, "scripts/install.sh");
        assert_eq!(url, "https://raw.githubusercontent.com/alice/x/main/scripts/install.sh");
    }

    #[test]
    fn build_raw_url_with_legacy_default_matches_existing() {
        let r = super::RepoRef::hardcoded_default();
        let url = super::build_raw_url(&r, "scripts/install.ps1");
        assert_eq!(
            url,
            "https://raw.githubusercontent.com/NousResearch/hermes-agent/main/scripts/install.ps1"
        );
    }
}
