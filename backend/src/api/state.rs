//! Shared application state and per-request query context.

use std::sync::Arc;
use std::time::Duration;

use crate::prom::PromClient;

/// Concrete Prometheus client type used by handlers.
pub type SharedPromClient = Arc<PromClient>;

/// Bundle handed to handlers for executing PromQL queries.
#[derive(Clone)]
pub struct QueryContext {
    pub client: SharedPromClient,
}

/// Top-level application state injected into the Axum router.
#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    client: PromClient,
}

impl AppState {
    /// Builds state from explicit components (used in production wiring).
    pub fn new(prometheus_url: &str, query_timeout: Duration, ping_timeout: Duration) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                client: PromClient::new(prometheus_url, query_timeout, ping_timeout),
            }),
        }
    }

    /// Builds state from an existing client (used in tests).
    pub fn from_client(client: PromClient) -> Self {
        Self {
            inner: Arc::new(AppStateInner { client }),
        }
    }

    /// Creates a cheap per-request query context.
    pub fn context(&self) -> QueryContext {
        QueryContext {
            client: Arc::new(self.inner.client.clone()),
        }
    }
}
