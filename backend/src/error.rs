//! Central error type for the CircuitDeck API.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

/// Errors that can occur while serving dashboard API requests.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// Prometheus could not be reached at all (network / DNS / timeout).
    #[error("prometheus is unreachable: {0}")]
    PrometheusUnreachable(String),

    /// Prometheus responded, but with a non-2xx status.
    #[error("prometheus returned status {status}: {body}")]
    PrometheusStatus { status: u16, body: String },

    /// Prometheus responded 2xx but the body could not be parsed as expected.
    #[error("unexpected prometheus response: {0}")]
    PrometheusBadResponse(String),

    /// The requested time range is not one of the supported values.
    #[error("invalid time range: {0}")]
    InvalidTimeRange(String),

    /// Anything unexpected during request handling.
    #[error("internal error: {0}")]
    Internal(String),
}

impl ApiError {
    /// True when the failure means telemetry data cannot be trusted right now.
    pub fn is_prometheus_failure(&self) -> bool {
        matches!(
            self,
            ApiError::PrometheusUnreachable(_)
                | ApiError::PrometheusStatus { .. }
                | ApiError::PrometheusBadResponse(_)
        )
    }
}

/// JSON body returned for every API error.
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorBody {
    pub error: ErrorDetail,
}

/// Error detail payload.
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, public_message) = match &self {
            ApiError::InvalidTimeRange(msg) => (
                StatusCode::BAD_REQUEST,
                "invalid_time_range",
                format!("invalid time range: {msg}"),
            ),
            ApiError::PrometheusUnreachable(_) => (
                StatusCode::BAD_GATEWAY,
                "prometheus_error",
                "telemetry backend is unreachable".to_string(),
            ),
            ApiError::PrometheusStatus { .. } => (
                StatusCode::BAD_GATEWAY,
                "prometheus_error",
                "telemetry backend returned an error".to_string(),
            ),
            ApiError::PrometheusBadResponse(_) => (
                StatusCode::BAD_GATEWAY,
                "prometheus_error",
                "telemetry backend returned an unexpected response".to_string(),
            ),
            ApiError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "internal error".to_string(),
            ),
        };
        // Full details stay in server logs; the client gets a generic message.
        tracing::warn!(error = %self, status = %status, "api error");
        let body = ErrorBody {
            error: ErrorDetail {
                code: code.to_string(),
                message: public_message,
            },
        };
        (status, axum::Json(body)).into_response()
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            ApiError::PrometheusUnreachable("request timed out".to_string())
        } else if err.is_connect() {
            ApiError::PrometheusUnreachable(err.to_string())
        } else {
            ApiError::PrometheusBadResponse(err.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn invalid_range_maps_to_400() {
        let resp = ApiError::InvalidTimeRange("3w".into()).into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let body: ErrorBody = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body.error.code, "invalid_time_range");
    }

    #[tokio::test]
    async fn prometheus_failures_map_to_502() {
        let resp = ApiError::PrometheusUnreachable("conn refused".into()).into_response();
        assert_eq!(resp.status(), StatusCode::BAD_GATEWAY);
    }
}
