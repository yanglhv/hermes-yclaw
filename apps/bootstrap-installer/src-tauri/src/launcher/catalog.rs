use url::Url;

#[derive(Debug, serde::Deserialize)]
struct CatalogJson {
    apps: Vec<AppJson>,
}

#[derive(Debug, serde::Deserialize)]
struct AppJson {
    app_id: String,
    display_name: String,
    version: String,
}

pub fn parse_catalog_response(body: &str) -> Result<Vec<crate::app::AppDescriptor>, crate::app::AppParseError> {
    let catalog: CatalogJson = serde_json::from_str(body)?;
    Ok(catalog.apps.into_iter().map(|a| {
        let app_id = a.app_id.clone();
        let display_name = a.display_name.clone();
        crate::app::AppDescriptor {
            schema_version: 1,
            id: app_id.clone(),
            app_id: app_id.clone(),
            display_name,
            category: "agent".into(),
            default: false,
            script_path: "install.sh".into(),
            install_root: app_id.clone(),
            binary: crate::app::BinaryPaths {
                windows: format!("{}.exe", app_id),
                macos: format!("{}.app", app_id),
                linux: app_id,
            },
            uninstall_supported: true,
            app_settings_url: None,
            min_launcher_version: "0.1.0".into(),
            icon: None,
            default_repo_ref: crate::app::RepoRef {
                repo: "NousResearch/hermes-agent".into(),
                r#ref: "main".into(),
                repo_url_base: "https://github.com".into(),
            },
            catalog_url: Url::parse("https://api.github.com/repos/NousResearch/hermes-agent").unwrap(),
            icon_url: Url::parse("https://api.github.com/repos/NousResearch/hermes-agent/contents/icon.png").unwrap(),
            description: format!("App: {}", a.display_name),
            version: a.version,
            binaries: vec![],
            check_update_url: Url::parse("https://api.github.com/repos/NousResearch/hermes-agent/releases/latest").unwrap(),
        }
    }).collect())
}

pub async fn list_available_apps(catalog_url: &Url) -> Result<Vec<crate::app::AppDescriptor>, reqwest::Error> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    let resp = client.get(catalog_url.as_str()).send().await?;
    if !resp.status().is_success() {
        return Ok(vec![]);
    }
    let body = resp.text().await?;
    match parse_catalog_response(&body) {
        Ok(apps) => Ok(apps),
        Err(e) => {
            tracing::warn!(err = %e, "failed to parse catalog response");
            Ok(vec![])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_available_apps_parses_catalog_response() {
        let response = r#"{"apps":[{"app_id":"hermes","display_name":"Hermes","version":"2.0.0"}]}"#;
        let apps = parse_catalog_response(response).unwrap();
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].app_id, "hermes");
    }

    #[tokio::test]
    async fn list_available_apps_handles_network_error() {
        let result = list_available_apps(&Url::parse("http://localhost:65535/catalog").unwrap()).await;
        assert!(result.is_err());
    }
}
