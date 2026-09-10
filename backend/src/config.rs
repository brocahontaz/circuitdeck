use std::time::Duration;

/// Configuration for the CircuitDeck backend, sourced from the environment.
#[derive(Debug, Clone)]
pub struct Config {
    /// Address the HTTP server binds to.
    pub bind_addr: String,
    /// Base URL of the Prometheus server, e.g. `http://prometheus:9090`.
    pub prometheus_url: String,
    /// Per-request timeout for Prometheus queries.
    pub query_timeout: Duration,
    /// Per-request timeout when probing Prometheus reachability.
    pub ping_timeout: Duration,
    /// CORS origins allowed for the frontend. Empty disables the CORS layer.
    pub cors_origins: Vec<String>,
}

impl Config {
    /// Loads configuration from environment variables with development-friendly defaults.
    ///
    /// # Errors
    /// Returns an error when an environment variable cannot be parsed.
    pub fn from_env() -> Result<Self, anyhow::Error> {
        let bind_addr =
            std::env::var("CIRCUITDECK_BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
        let prometheus_url = normalize(
            &std::env::var("PROMETHEUS_URL")
                .unwrap_or_else(|_| "http://localhost:9090".to_string()),
        );
        let query_timeout = parse_secs("PROMETHEUS_QUERY_TIMEOUT_SECS", 10)?;
        let ping_timeout = parse_secs("PROMETHEUS_PING_TIMEOUT_SECS", 3)?;
        let cors_origins = std::env::var("CIRCUITDECK_CORS_ORIGINS")
            .map(|v| {
                v.split(',')
                    .map(|o| o.trim().to_string())
                    .filter(|o| !o.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        Ok(Self {
            bind_addr,
            prometheus_url,
            query_timeout,
            ping_timeout,
            cors_origins,
        })
    }
}

/// Strips a trailing slash so joined paths stay well-formed.
fn normalize(url: &str) -> String {
    let trimmed = url.trim().to_string();
    if trimmed.ends_with('/') {
        trimmed.trim_end_matches('/').to_string()
    } else {
        trimmed
    }
}

fn parse_secs(key: &str, default: u64) -> Result<Duration, anyhow::Error> {
    match std::env::var(key) {
        Ok(v) => {
            let secs: u64 = v
                .parse()
                .map_err(|e| anyhow::anyhow!("{key}: invalid unsigned integer: {e}"))?;
            Ok(Duration::from_secs(secs))
        }
        Err(_) => Ok(Duration::from_secs(default)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_strips_trailing_slash() {
        assert_eq!(normalize("http://p:9090/"), "http://p:9090");
        assert_eq!(normalize("http://p:9090"), "http://p:9090");
        assert_eq!(normalize("  http://p:9090/ "), "http://p:9090");
    }
}
