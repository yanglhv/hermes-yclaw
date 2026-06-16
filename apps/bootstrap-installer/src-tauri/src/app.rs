use serde::{Deserialize, Serialize};

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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BinaryPaths {
    pub windows: String,
    pub macos: String,
    pub linux: String,
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
        }
    }
}

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
}
