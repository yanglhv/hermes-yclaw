pub mod catalog {
    pub async fn fetch_catalog(_api_base: &str, _repo: &crate::app::RepoRef) -> Result<Vec<crate::app::AppDescriptor>, reqwest::Error> {
        Ok(vec![])
    }
}
