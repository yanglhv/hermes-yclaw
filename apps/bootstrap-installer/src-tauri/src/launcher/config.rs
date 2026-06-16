pub mod config {
    use crate::app::RepoRef as AppRepoRef;
    use crate::install_script::RepoRef as InstallScriptRepoRef;
    use crate::paths;
    use crate::app::AppDescriptor;
    use url::Url;

    #[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
    pub struct RepoConfig {
        #[serde(default)]
        pub owner: Option<String>,
        #[serde(default)]
        pub name: Option<String>,
        #[serde(rename = "ref", default)]
        pub ref_: Option<String>,
    }

    #[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
    pub struct LauncherConfig {
        #[serde(default)]
        pub preferred_channel: String,
        #[serde(default)]
        pub auto_update: bool,
        #[serde(default)]
        pub skip_network_probe: bool,
        #[serde(default)]
        pub repo: RepoConfig,
    }

    impl LauncherConfig {
        pub fn resolve_install_script_url(&self, app: &AppDescriptor) -> Url {
            let install_repo_ref = InstallScriptRepoRef {
                owner: app.default_repo_ref.repo.split('/').next().unwrap_or("").into(),
                name: app.default_repo_ref.repo.split('/').nth(1).unwrap_or("").into(),
                ref_name: app.default_repo_ref.r#ref.clone(),
            };
            let url_str = crate::install_script::build_raw_url(&install_repo_ref, "install.ps1");
            Url::parse(&url_str).unwrap_or_else(|_| Url::parse("https://raw.githubusercontent.com/").unwrap())
        }

        pub fn resolve_update_script_url(&self, app: &AppDescriptor) -> Url {
            let install_repo_ref = InstallScriptRepoRef {
                owner: app.default_repo_ref.repo.split('/').next().unwrap_or("").into(),
                name: app.default_repo_ref.repo.split('/').nth(1).unwrap_or("").into(),
                ref_name: app.default_repo_ref.r#ref.clone(),
            };
            let url_str = crate::install_script::build_raw_url(&install_repo_ref, "update.ps1");
            Url::parse(&url_str).unwrap_or_else(|_| Url::parse("https://raw.githubusercontent.com/").unwrap())
        }

        pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
            let path = paths::launcher_config_path();
            if !path.exists() {
                return Ok(Self::default());
            }
            let body = std::fs::read_to_string(&path)?;
            let config: LauncherConfig = serde_yaml::from_str(&body)?;
            Ok(config)
        }

        pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
            let path = paths::launcher_config_path();
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let body = serde_yaml::to_string(self)?;
            std::fs::write(&path, body)?;
            Ok(())
        }
    }

    #[derive(Default, Debug)]
    pub struct Overrides {
        pub env_owner: Option<String>,
        pub env_name: Option<String>,
        pub env_ref: Option<String>,
        pub combined_override: Option<String>,
        pub yaml: Option<LauncherConfig>,
    }

    pub fn collect_overrides() -> Overrides {
        Overrides {
            env_owner: std::env::var("LAUNCHER_REPO_OWNER").ok().filter(|s| !s.is_empty()),
            env_name: std::env::var("LAUNCHER_REPO_NAME").ok().filter(|s| !s.is_empty()),
            env_ref: std::env::var("LAUNCHER_REPO_REF").ok().filter(|s| !s.is_empty()),
            combined_override: std::env::var("LAUNCHER_REPO_OVERRIDE").ok().filter(|s| !s.is_empty()),
            yaml: load_yaml(),
        }
    }

    pub fn resolve() -> AppRepoRef {
        resolve_with(&collect_overrides(), build_time_defaults())
    }

    pub fn resolve_with(o: &Overrides, build: (&str, &str, &str)) -> AppRepoRef {
        let (bo, bn, br) = build;
        let mut r = AppRepoRef {
            repo: format!("{}/{}", bo, bn),
            r#ref: br.into(),
            repo_url_base: "https://github.com".into(),
        };
        if let Some(v) = &o.env_owner {
            if let Some((owner, name)) = v.split_once('/') {
                r.repo = format!("{}/{}", owner, name);
            }
        }
        if let Some(v) = &o.env_ref {
            r.r#ref = v.clone();
        }
        if let Some(c) = &o.combined_override {
            if let Some(p) = parse_combined(c) {
                r = p;
            }
        }
        if let Some(y) = &o.yaml {
            if let Some(owner) = &y.repo.owner {
                let name = r.repo.split('/').nth(1).unwrap_or("");
                r.repo = format!("{}/{}", owner, name);
            }
            if let Some(name) = &y.repo.name {
                let owner = r.repo.split('/').next().unwrap_or("");
                r.repo = format!("{}/{}", owner, name);
            }
            if let Some(ref_) = &y.repo.ref_ {
                r.r#ref = ref_.clone();
            }
        }
        r
    }

    pub fn build_time_defaults() -> (&'static str, &'static str, &'static str) {
        (
            option_env!("BUILD_REPO_OWNER").unwrap_or("NousResearch"),
            option_env!("BUILD_REPO_NAME").unwrap_or("hermes-agent"),
            option_env!("BUILD_PIN_BRANCH").unwrap_or("main"),
        )
    }

    pub fn load_yaml() -> Option<LauncherConfig> {
        let path = paths::launcher_config_path();
        let body = std::fs::read_to_string(&path).ok()?;
        match serde_yaml::from_str::<LauncherConfig>(&body) {
            Ok(c) => Some(c),
            Err(err) => {
                tracing::warn!(?err, "ignoring malformed launcher-config.yaml");
                None
            }
        }
    }

    fn parse_combined(s: &str) -> Option<AppRepoRef> {
        let (path_part, ref_part) = match s.split_once('@') {
            Some((p, r)) => (p, Some(r.to_string())),
            None => (s, None),
        };
        let (owner, name) = path_part.split_once('/')?;
        Some(AppRepoRef {
            repo: format!("{}/{}", owner, name),
            r#ref: ref_part.unwrap_or_else(|| "main".into()),
            repo_url_base: "https://github.com".into(),
        })
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn resolve_install_script_url_uses_repo_ref() {
            let config = LauncherConfig::default();
            let url = config.resolve_install_script_url(&AppDescriptor::literal_hermes());
            assert!(url.as_str().contains("raw."));
        }

        #[test]
        fn defaults_to_build_time() {
            let r = resolve_with(&Overrides::default(), ("bo", "bn", "main"));
            assert_eq!(r.repo, "bo/bn");
            assert_eq!(r.r#ref, "main");
        }

        #[test]
        fn env_owner_overrides_build() {
            let o = Overrides { env_owner: Some("eo/en".into()), ..Default::default() };
            let r = resolve_with(&o, ("bo", "bn", "main"));
            assert_eq!(r.repo, "eo/en");
        }

        #[test]
        fn combined_override_parses_owner_slash_name_at_ref() {
            let o = Overrides { combined_override: Some("alice/repo@v2".into()), ..Default::default() };
            let r = resolve_with(&o, ("bo", "bn", "main"));
            assert_eq!(r.repo, "alice/repo");
            assert_eq!(r.r#ref, "v2");
        }
    }
}
