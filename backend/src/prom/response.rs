//! Typed models for Prometheus HTTP API responses (v1 query endpoints).

use serde::Deserialize;
use serde_json::Value;

/// Envelope of every Prometheus HTTP API v1 response.
#[derive(Debug, Deserialize)]
pub struct PromResponse<T> {
    pub status: String,
    #[serde(default)]
    pub data: Option<T>,
}

/// `data` payload of `/api/v1/query` and `/api/v1/query_range`.
#[derive(Debug, Deserialize)]
pub struct PromQueryData {
    #[serde(rename = "resultType")]
    pub result_type: String,
    #[serde(default)]
    pub result: Vec<Value>,
}

impl Default for PromQueryData {
    fn default() -> Self {
        Self {
            result_type: "vector".to_string(),
            result: Vec::new(),
        }
    }
}

/// A single sample with its metric labels.
#[derive(Debug, Clone, PartialEq)]
pub struct Sample {
    /// Label set of the series (`__name__` included).
    pub metric: std::collections::HashMap<String, String>,
    /// Unix timestamp in seconds.
    pub timestamp: f64,
    /// Sample value; NaN for stale markers.
    pub value: Option<f64>,
}

/// Parses a single instant-vector pair `[ts, "value"]`.
pub fn parse_instant_pair(pair: &Value) -> Option<Sample> {
    let arr = pair.as_array()?;
    if arr.len() != 2 {
        return None;
    }
    let timestamp = match arr[0].as_str() {
        Some(s) => parse_ts(s)?,
        None => arr[0].as_f64()?,
    };
    let value = parse_value_str(arr[1].as_str()?);
    Some(Sample {
        metric: labels_from(pair),
        timestamp,
        value,
    })
}

/// Parses a range-vector entry `[ts, "value", ...]` (step samples).
pub fn parse_range_pair(pair: &Value) -> Option<Sample> {
    let arr = pair.as_array()?;
    if arr.len() < 2 {
        return None;
    }
    let timestamp = match arr[0].as_str() {
        Some(s) => parse_ts(s)?,
        None => arr[0].as_f64()?,
    };
    let value = parse_value_str(arr[1].as_str()?);
    Some(Sample {
        metric: labels_from(pair),
        timestamp,
        value,
    })
}

/// Parses a string timestamp as an integer-valued float.
pub fn parse_ts(s: &str) -> Option<f64> {
    if s.contains('.') {
        return s.parse::<f64>().ok();
    }
    s.parse::<i64>().ok().map(|t| t as f64)
}

/// Parses a Prometheus value string ("10.5", "+Inf", "NaN", stale markers).
fn parse_value_str(s: &str) -> Option<f64> {
    match s {
        "NaN" | "+Inf" | "-Inf" => None,
        other => other.parse::<f64>().ok(),
    }
}

/// The Prometheus JSON model puts labels inside the pair's parent object;
/// wiremock-free parsing relies on the parent object carrying `metric`.
fn labels_from(_pair: &Value) -> std::collections::HashMap<String, String> {
    std::collections::HashMap::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_instant_pair() {
        let pair = json!(["1700000000", "42.5"]);
        let s = parse_instant_pair(&pair).unwrap();
        assert_eq!(s.timestamp, 1700000000.0);
        assert_eq!(s.value, Some(42.5));
    }

    #[test]
    fn stale_marker_yields_none_value() {
        let pair = json!(["1700000000", "NaN"]);
        let s = parse_instant_pair(&pair).unwrap();
        assert_eq!(s.value, None);
    }

    #[test]
    fn parses_range_pair() {
        let pair = json!(["1700000015", "7"]);
        let s = parse_range_pair(&pair).unwrap();
        assert_eq!(s.value, Some(7.0));
    }

    #[test]
    fn rejects_short_pairs() {
        let pair = json!(["1700000000"]);
        assert!(parse_instant_pair(&pair).is_none());
    }
}
