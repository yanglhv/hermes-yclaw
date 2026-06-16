use std::time::{Duration, Instant};

use reqwest::blocking::Client;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct NetworkProbeResult {
    pub has_internet: bool,
    pub latency_ms: Option<u64>,
}

pub fn probe_network() -> std::result::Result<NetworkProbeResult, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    let url = "https://clients3.google.com/generate_204";
    let start = Instant::now();

    match client.get(url).send() {
        Ok(response) => {
            let latency = start.elapsed().as_millis() as u64;
            let has_internet = response.status().is_success() || response.status().as_u16() == 204;
            Ok(NetworkProbeResult {
                has_internet,
                latency_ms: Some(latency),
            })
        }
        Err(_) => Ok(NetworkProbeResult {
            has_internet: false,
            latency_ms: None,
        }),
    }
}

#[cfg(test)]
mod network_tests {
    use super::*;

    #[test]
    fn probe_network_returns_result() {
        let result = probe_network();
        assert!(result.is_ok());
    }

    #[test]
    fn network_probe_result_fields() {
        let result = probe_network().expect("should succeed");
        assert!(result.has_internet || !result.has_internet);
        if result.has_internet {
            assert!(result.latency_ms.is_some());
        }
    }
}
