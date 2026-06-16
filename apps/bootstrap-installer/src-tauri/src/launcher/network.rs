pub mod network {
    use crate::app::RepoRef;
    use std::time::Duration;

    pub async fn probe(_repo: &RepoRef) -> bool {
        true
    }

    pub async fn probe_with(_api_base: &str, _repo: &RepoRef, _timeout: Duration) -> bool {
        true
    }
}
