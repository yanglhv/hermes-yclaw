pub mod config {
    use crate::app::RepoRef;

    #[derive(Default, Debug)]
    pub struct Overrides {
        pub env_owner: Option<String>,
        pub env_name: Option<String>,
        pub env_ref: Option<String>,
        pub combined_override: Option<String>,
    }

    pub fn collect_overrides() -> Overrides {
        Overrides {
            env_owner: std::env::var("LAUNCHER_REPO_OWNER").ok().filter(|s| !s.is_empty()),
            env_name: std::env::var("LAUNCHER_REPO_NAME").ok().filter(|s| !s.is_empty()),
            env_ref: std::env::var("LAUNCHER_REPO_REF").ok().filter(|s| !s.is_empty()),
            combined_override: std::env::var("LAUNCHER_REPO_OVERRIDE").ok().filter(|s| !s.is_empty()),
        }
    }

    pub fn resolve() -> RepoRef {
        resolve_with(&collect_overrides(), build_time_defaults())
    }

    pub fn resolve_with(o: &Overrides, build: (&str, &str, &str)) -> RepoRef {
        let (bo, bn, br) = build;
        let mut r = RepoRef {
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
        r
    }

    pub fn build_time_defaults() -> (&'static str, &'static str, &'static str) {
        (
            option_env!("BUILD_REPO_OWNER").unwrap_or("NousResearch"),
            option_env!("BUILD_REPO_NAME").unwrap_or("hermes-agent"),
            option_env!("BUILD_PIN_BRANCH").unwrap_or("main"),
        )
    }
}
