//! Minimal async client for the Prometheus HTTP API v1.
use crate::error::ApiError;
use crate::prom::response::{
    parse_instant_pair, parse_range_pair, PromQueryData, PromResponse, Sample,
};
use serde_json::Value;
use std::time::Duration;
/// A series with its label set and parsed samples.
#[derive(Debug, Clone, PartialEq)]
pub struct SampleSeries {
    pub labels: std::collections::HashMap<String, String>,
    pub samples: Vec<Sample>,
}
/// Result of an instant query: one entry per time series.
pub type InstantResult = Vec<SampleSeries>;
/// Location of a series in a range query result.
#[derive(Debug, Clone, PartialEq)]
pub struct RangePoint {
    pub timestamp: f64,
    pub value: Option<f64>,
}
/// A series with ordered range samples.
#[derive(Debug, Clone, PartialEq)]
pub struct RangeSeries {
    pub labels: std::collections::HashMap<String, String>,
    pub points: Vec<RangePoint>,
}
/// Result of a range query.
pub type RangeResult = Vec<RangeSeries>;
/// HTTP client bound to a Prometheus base URL.
#[derive(Debug, Clone)]
pub struct PromClient {
    http: reqwest::Client,
    base_url: String,
    ping_timeout: Duration,
}
impl PromClient {
    /// Creates a client for the given Prometheus base URL.
    pub fn new(base_url: &str, query_timeout: Duration, ping_timeout: Duration) -> Self {
        let http = reqwest::Client::builder()
            .timeout(query_timeout)
            .build()
            .expect("reqwest client builder cannot fail with these settings");
        Self {
            http,
            base_url: base_url.trim_end_matches('/').to_string(),
            ping_timeout,
        }
    }
    /// Base URL this client queries.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
    /// Performs `GET /api/v1/query` and parses the result into an instant vector.
    ///
    /// # Errors
    /// Returns [`ApiError`] variants for transport, status, and parse failures.
    pub async fn query(&self, promql: &str) -> Result<InstantResult, ApiError> {
        let url = format!("{}/api/v1/query", self.base_url);
        let resp = self
            .http
            .get(&url)
            .query(&[("query", promql)])
            .send()
            .await?;
        Self::check_status(resp.status(), &url).await?;
        let body: PromResponse<PromQueryData> = resp
            .json()
            .await
            .map_err(|e| ApiError::PrometheusBadResponse(format!("query `{promql}`: {e}")))?;
        let pairs: Vec<(std::collections::HashMap<String, String>, Vec<Sample>)> =
            Self::finish(body, |item| {
                let labels = labels_of(item);
                let mut samples = Vec::new();
                if let Some(value) = item.get("value") {
                    if let Some(sample) = parse_instant_pair(value) {
                        samples.push(Sample {
                            metric: std::collections::HashMap::new(),
                            ..sample
                        });
                    }
                }
                (labels, samples)
            })?;
        let result = pairs
            .into_iter()
            .map(|(labels, samples)| SampleSeries { labels, samples })
            .collect();
        Ok(result)
    }
    /// Performs `GET /api/v1/query_range` and parses the result into a matrix.
    ///
    /// # Errors
    /// Returns [`ApiError`] variants for transport, status, and parse failures.
    pub async fn query_range(
        &self,
        promql: &str,
        start: i64,
        end: i64,
        step_secs: u32,
    ) -> Result<RangeResult, ApiError> {
        let url = format!("{}/api/v1/query_range", self.base_url);
        let resp = self
            .http
            .get(&url)
            .query(&[
                ("query", promql),
                ("start", start.to_string().as_str()),
                ("end", end.to_string().as_str()),
                ("step", &format!("{step_secs}s")),
            ])
            .send()
            .await?;
        Self::check_status(resp.status(), &url).await?;
        let body: PromResponse<PromQueryData> = resp
            .json()
            .await
            .map_err(|e| ApiError::PrometheusBadResponse(format!("range query `{promql}`: {e}")))?;
        let pairs: Vec<(std::collections::HashMap<String, String>, Vec<Sample>)> =
            Self::finish(body, |item| {
                let labels = labels_of(item);
                let mut samples = Vec::new();
                if let Some(values) = item.get("values").and_then(Value::as_array) {
                    for value in values {
                        if let Some(sample) = parse_range_pair(value) {
                            samples.push(sample);
                        }
                    }
                }
                (labels, samples)
            })?;
        let result = pairs
            .into_iter()
            .map(|(labels, samples)| RangeSeries {
                labels,
                points: samples
                    .into_iter()
                    .map(|s| RangePoint {
                        timestamp: s.timestamp,
                        value: s.value,
                    })
                    .collect(),
            })
            .collect();
        Ok(result)
    }
    /// Probes Prometheus liveness with a trivial query.
    ///
    /// # Errors
    /// Returns an error when Prometheus cannot be reached.
    pub async fn ping(&self) -> Result<(), ApiError> {
        let url = format!("{}/api/v1/query", self.base_url);
        let resp = self
            .http
            .get(&url)
            .timeout(self.ping_timeout)
            .query(&[("query", "1")])
            .send()
            .await?;
        Self::check_status(resp.status(), &url).await?;
        Ok(())
    }
    async fn check_status(status: reqwest::StatusCode, url: &str) -> Result<(), ApiError> {
        if status.is_success() {
            Ok(())
        } else {
            Err(ApiError::PrometheusStatus {
                status: status.as_u16(),
                body: format!("request to {url} failed"),
            })
        }
    }
    fn finish<T, F>(body: PromResponse<PromQueryData>, mut map: F) -> Result<Vec<T>, ApiError>
    where
        F: FnMut(&Value) -> T,
    {
        if body.status != "success" {
            return Err(ApiError::PrometheusBadResponse(format!(
                "status={}",
                body.status
            )));
        }
        let data = body
            .data
            .ok_or_else(|| ApiError::PrometheusBadResponse("missing data".to_string()))?;
        Ok(data.result.iter().map(&mut map).collect())
    }
}
fn labels_of(item: &Value) -> std::collections::HashMap<String, String> {
    item.get("metric")
        .and_then(Value::as_object)
        .map(|map| {
            map.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn ping_ok_on_success_envelope() {
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/api/v1/query"))
            .respond_with(
                wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "status": "success",
                    "data": { "resultType": "vector", "result": [] }
                })),
            )
            .mount(&server)
            .await;
        let client = PromClient::new(
            server.uri().as_str(),
            Duration::from_secs(2),
            Duration::from_secs(2),
        );
        client.ping().await.expect("ping should succeed");
    }
    #[tokio::test]
    async fn query_parses_instant_vector() {
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/api/v1/query"))
            .respond_with(
                wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "status": "success",
                    "data": {
                        "resultType": "vector",
                        "result": [
                            { "metric": { "item": "iron-plate", "job": "factorio" }, "value": [1700000000, "123.5"] },
                            { "metric": { "item": "copper-plate" }, "value": [1700000000, "NaN"] }
                        ]
                    }
                })),
            )
            .mount(&server)
            .await;
        let client = PromClient::new(
            server.uri().as_str(),
            Duration::from_secs(2),
            Duration::from_secs(2),
        );
        let result = client.query("factorio_item_produced_total").await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0].labels.get("item").map(String::as_str),
            Some("iron-plate")
        );
        assert_eq!(result[0].samples[0].value, Some(123.5));
        assert_eq!(result[1].samples[0].value, None);
    }
    #[tokio::test]
    async fn query_range_parses_matrix() {
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/api/v1/query_range"))
            .respond_with(
                wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "status": "success",
                    "data": {
                        "resultType": "matrix",
                        "result": [
                            { "metric": { "item": "iron-plate" },
                              "values": [["1700000000", "1"], ["1700000015", "2"], ["1700000030", "NaN"]] }
                        ]
                    }
                })),
            )
            .mount(&server)
            .await;
        let client = PromClient::new(
            server.uri().as_str(),
            Duration::from_secs(2),
            Duration::from_secs(2),
        );
        let result = client
            .query_range("rate(x[5m])", 1700000000, 1700000030, 15)
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].points.len(), 3);
        assert_eq!(result[0].points[0].value, Some(1.0));
        assert_eq!(result[0].points[2].value, None);
    }
    #[tokio::test]
    async fn non_success_envelope_is_error() {
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .respond_with(
                wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "status": "error", "errorType": "badData", "error": "parse error"
                })),
            )
            .mount(&server)
            .await;
        let client = PromClient::new(
            server.uri().as_str(),
            Duration::from_secs(2),
            Duration::from_secs(2),
        );
        let err = client.query("bad{").await.unwrap_err();
        assert!(matches!(err, ApiError::PrometheusBadResponse(_)));
    }
    #[tokio::test]
    async fn http_error_status_maps_to_prometheus_status() {
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .respond_with(wiremock::ResponseTemplate::new(500).set_body_string("boom"))
            .mount(&server)
            .await;
        let client = PromClient::new(
            server.uri().as_str(),
            Duration::from_secs(2),
            Duration::from_secs(2),
        );
        let err = client.query("up").await.unwrap_err();
        assert!(matches!(
            err,
            ApiError::PrometheusStatus { status: 500, .. }
        ));
    }
    #[tokio::test]
    async fn unreachable_maps_to_unreachable() {
        let client = PromClient::new(
            "http://127.0.0.1:1",
            Duration::from_secs(1),
            Duration::from_secs(1),
        );
        let err = client.query("up").await.unwrap_err();
        assert!(matches!(err, ApiError::PrometheusUnreachable(_)));
    }
}
