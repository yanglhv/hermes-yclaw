use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

pub const APP_SCHEMA_VERSION: u32 = 1;
pub const STATE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoRef {
    pub repo: String,
    pub r#ref: String,
    pub repo_url_base: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppDescriptor {
    pub schema_version: u32,
    pub id: String,
    pub display_name: String,
    pub category: String,
    pub default: bool,
    pub script_path: String,
    pub install_root: String,
    pub binary: BinaryPaths,
    pub uninstall_supported: bool,
    pub app_settings_url: Option<String>,
    pub min_launcher_version: String,
    pub icon: Option<String>,
    pub app_id: String,
    pub default_repo_ref: RepoRef,
    pub catalog_url: Url,
    pub icon_url: Url,
    pub description: String,
    pub version: String,
    pub binaries: Vec<String>,
    pub check_update_url: Url,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BinaryPaths {
    pub windows: String,
    pub macos: String,
    pub linux: String,
}

#[derive(Debug, Error)]
pub enum AppParseError {
    #[error("invalid JSON: {0}")] Json(#[from] serde_json::Error),
    #[error("unsupported schema_version {0}")] SchemaVersion(u32),
    #[error("macos binary must end in .app: {0}")] BadMacosBinary(String),
}

pub fn parse_app_json(contents: &str) -> Result<AppDescriptor, AppParseError> {
    let d: AppDescriptor = serde_json::from_str(contents)?;
    if d.schema_version != APP_SCHEMA_VERSION { return Err(AppParseError::SchemaVersion(d.schema_version)); }
    if !d.binary.macos.ends_with(".app") { return Err(AppParseError::BadMacosBinary(d.binary.macos.clone())); }
    Ok(d)
}

impl AppDescriptor {
    pub fn literal_hermes() -> Self {
        Self {
            schema_version: 1,
            id: "hermes".into(),
            display_name: "Hermes Agent".into(),
            category: "agent".into(),
            default: true,
            script_path: "scripts/install.ps1".into(),
            install_root: "hermes-agent".into(),
            binary: BinaryPaths {
                windows: "apps/desktop/release/win-unpacked/Hermes.exe".into(),
                macos: "apps/desktop/release/mac-arm64/Hermes.app".into(),
                linux: "apps/desktop/release/linux-unpacked/hermes".into(),
            },
            uninstall_supported: true,
            app_settings_url: None,
            min_launcher_version: "0.1.0".into(),
            icon: None,
            app_id: "hermes".into(),
            default_repo_ref: RepoRef {
                repo: "NousResearch/hermes-agent".into(),
                r#ref: "main".into(),
                repo_url_base: "https://github.com".into(),
            },
            catalog_url: Url::parse("https://api.github.com/repos/NousResearch/hermes-agent").unwrap(),
            icon_url: Url::parse("https://api.github.com/repos/NousResearch/hermes-agent/contents/icon.png").unwrap(),
            description: "The Hermes Agent desktop application".into(),
            version: "1.0.0".into(),
            binaries: vec![
                "apps/desktop/release/win-unpacked/Hermes.exe".into(),
                "apps/desktop/release/mac-arm64/Hermes.app".into(),
                "apps/desktop/release/linux-unpacked/hermes".into(),
            ],
            check_update_url: Url::parse("https://api.github.com/repos/NousResearch/hermes-agent/releases/latest").unwrap(),
        }
    }

    pub fn script_extension(&self) -> &'static str {
        if self.script_path.ends_with(".ps1") { "ps1" } else { "sh" }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LauncherState {
    pub schema_version: u32,
    #[serde(default)] pub default_app_id: Option<String>,
    #[serde(default)] pub installed: BTreeMap<String, InstalledApp>,
    #[serde(default)] pub pending_updates: BTreeMap<String, PendingUpdate>,
    #[serde(default)] pub last_update_check_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledApp {
    pub install_root: String,
    pub installed_commit: String,
    pub installed_ref_type: String,
    pub installed_ref_name: String,
    pub installed_at: String,
    pub installed_via: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingUpdate {
    pub latest_commit: String,
    pub latest_ref_name: String,
    pub status: PendingStatus,
    pub downloaded_script: Option<String>,
    pub downloaded_at: Option<String>,
    pub last_error: Option<String>,
    pub last_error_at: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PendingStatus { Avail, Downloading, Ready, Failed }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_hermes_has_expected_fields() {
        let d = AppDescriptor::literal_hermes();
        assert_eq!(d.id, "hermes");
        assert!(d.default);
        assert!(d.binary.macos.ends_with(".app"));
        assert_eq!(d.install_root, "hermes-agent");
    }

    #[test]
    fn app_descriptor_has_launcher_fields() {
        let app = AppDescriptor {
            schema_version: 1,
            id: "myapp".into(),
            display_name: "My App".into(),
            category: "test".into(),
            default: false,
            script_path: "install.ps1".into(),
            install_root: "myapp".into(),
            binary: BinaryPaths {
                windows: "bin/myapp.exe".into(),
                macos: "bin/myapp.app".into(),
                linux: "bin/myapp".into(),
            },
            uninstall_supported: true,
            app_settings_url: None,
            min_launcher_version: "0.1.0".into(),
            icon: None,
            app_id: "myapp".into(),
            default_repo_ref: RepoRef {
                repo: "myorg/myrepo".into(),
                r#ref: "main".into(),
                repo_url_base: "https://github.com".into(),
            },
            catalog_url: "https://api.example.com/catalog".parse().unwrap(),
            icon_url: "https://api.example.com/icon.png".parse().unwrap(),
            description: "A test app".into(),
            version: "1.0.0".into(),
            binaries: vec![],
            check_update_url: "https://api.example.com/check".parse().unwrap(),
        };
        assert_eq!(app.display_name, "My App");
        assert!(app.uninstall_supported);
    }
}
