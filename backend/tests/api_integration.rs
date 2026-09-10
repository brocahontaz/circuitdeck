//! End-to-end API integration tests using a mock Prometheus server.

use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use circuitdeck_backend::api::router::build_router;
use circuitdeck_backend::api::state::AppState;
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

#[tokio::test]
async fn invalid_range_param_returns_400() {
    let server = wiremock::MockServer::start().await;
    let state = AppState::new(
        server.uri().as_str(),
        Duration::from_secs(2),
        Duration::from_secs(2),
    );
    let app = build_router(state, &[]);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/overview?range=3w")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["error"]["code"], "invalid_time_range");
}

#[tokio::test]
async fn health_reports_reachable_prometheus() {
    let server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/api/v1/query"))
        .respond_with(
            wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "success",
                "data": { "resultType": "vector", "result": [
                    { "metric": {"__name__": "up", "job": "factorio-metrics"}, "value": [1700000000, "1"] }
                ]}
            })),
        )
        .mount(&server)
        .await;
    let state = AppState::new(
        server.uri().as_str(),
        Duration::from_secs(2),
        Duration::from_secs(2),
    );
    let app = build_router(state, &[]);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["prometheus_reachable"], true);
    assert_eq!(body["factorio_metrics_job_present"], true);
}

#[tokio::test]
async fn health_reports_unreachable_prometheus() {
    // Port 1 is virtually guaranteed closed.
    let state = AppState::new(
        "http://127.0.0.1:1",
        Duration::from_secs(1),
        Duration::from_millis(300),
    );
    let app = build_router(state, &[]);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["prometheus_reachable"], false);
    assert_eq!(body["factorio_metrics_job_present"], false);
}

#[tokio::test]
async fn overview_maps_prometheus_payload() {
    let server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/api/v1/query"))
        .and(wiremock::matchers::any())
        .respond_with(|req: &wiremock::Request| {
            let query = req
                .url
                .query_pairs()
                .find(|(k, _)| k == "query")
                .map(|(_, v)| v.to_string())
                .unwrap_or_default();
            let result = if query.contains("item_produced_total") {
                serde_json::json!([
                    { "metric": { "item": "iron-plate" }, "value": [1700000000, "42.0"] },
                    { "metric": { "item": "copper-plate" }, "value": [1700000000, "9.5"] }
                ])
            } else if query.contains("evolution_factor") {
                serde_json::json!([
                    { "metric": {}, "value": [1700000000, "0.87"] }
                ])
            } else if query.contains("accumulator_charge") || query.contains("satisfaction") {
                serde_json::json!([
                    { "metric": {}, "value": [1700000000, "0.95"] }
                ])
            } else if query.contains("power") {
                serde_json::json!([
                    { "metric": {}, "value": [1700000000, "50.25"] }
                ])
            } else {
                serde_json::json!([])
            };
            wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "success",
                "data": { "resultType": "vector", "result": result }
            }))
        })
        .mount(&server)
        .await;
    wiremock::Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/api/v1/query_range"))
        .respond_with(
            wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "success",
                "data": { "resultType": "matrix", "result": [] }
            })),
        )
        .mount(&server)
        .await;

    let state = AppState::new(
        server.uri().as_str(),
        Duration::from_secs(2),
        Duration::from_secs(2),
    );
    let app = build_router(state, &[]);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/overview?range=1h")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(body["health"]["prometheus_reachable"], true);
    assert_eq!(body["evolution_factor"], 0.87);
    assert_eq!(body["power_production_mw"], 50.25);
    assert_eq!(body["accumulator_charge_pct"], 95.0);
    assert_eq!(body["satisfaction_pct"], 95.0);
    let produced = body["top_produced"].as_array().unwrap();
    assert_eq!(produced.len(), 2);
    assert_eq!(produced[0]["label"], "copper-plate");
    assert_eq!(produced[0]["value"], 9.5);
    assert_eq!(produced[1]["label"], "iron-plate");
}

#[tokio::test]
async fn prometheus_down_yields_degraded_overview() {
    let state = AppState::new(
        "http://127.0.0.1:1",
        Duration::from_secs(1),
        Duration::from_millis(300),
    );
    let app = build_router(state, &[]);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/overview")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["health"]["prometheus_reachable"], false);
    assert_eq!(body["evolution_factor"], Value::Null);
    assert_eq!(body["top_produced"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn production_maps_series() {
    let server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/api/v1/query"))
        .and(wiremock::matchers::any())
        .respond_with(|req: &wiremock::Request| {
            let query = req
                .url
                .query_pairs()
                .find(|(k, _)| k == "query")
                .map(|(_, v)| v.to_string())
                .unwrap_or_default();
            let result = if query.contains("item_produced_total") {
                serde_json::json!([
                    { "metric": { "item": "iron-plate" }, "value": [1700000000, "12.0"] }
                ])
            } else {
                serde_json::json!([])
            };
            wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "success",
                "data": { "resultType": "vector", "result": result }
            }))
        })
        .mount(&server)
        .await;
    wiremock::Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/api/v1/query_range"))
        .respond_with(
            wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "success",
                "data": { "resultType": "matrix", "result": [
                    { "metric": { "item": "iron-plate" },
                      "values": [["1700000000", "10"], ["1700000030", "11"], ["1700000060", "12"]] }
                ]}
            })),
        )
        .mount(&server)
        .await;

    let state = AppState::new(
        server.uri().as_str(),
        Duration::from_secs(2),
        Duration::from_secs(2),
    );
    let app = build_router(state, &[]);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/production?range=1h")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(body["range"], "1h");
    assert_eq!(body["top_produced"][0]["label"], "iron-plate");
    let series = body["produced_series"].as_array().unwrap();
    assert_eq!(series.len(), 1);
    assert_eq!(series[0]["name"], "iron-plate");
    assert_eq!(series[0]["points"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn platforms_maps_series_to_rows() {
    let server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/api/v1/query"))
        .and(wiremock::matchers::any())
        .respond_with(|req: &wiremock::Request| {
            let query = req
                .url
                .query_pairs()
                .find(|(k, _)| k == "query")
                .map(|(_, v)| v.to_string())
                .unwrap_or_default();
            let result = if query.starts_with("count(") {
                serde_json::json!([{ "metric": {}, "value": [1700000000, "2"] }])
            } else if query.contains("platform_speed") {
                serde_json::json!([
                    { "metric": { "platform": "Muscovy" }, "value": [1700000000, "210.5"] }
                ])
            } else if query.contains("platform_fuel") {
                serde_json::json!([
                    { "metric": { "platform": "Muscovy" }, "value": [1700000000, "0.8"] }
                ])
            } else {
                serde_json::json!([])
            };
            wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "success",
                "data": { "resultType": "vector", "result": result }
            }))
        })
        .mount(&server)
        .await;

    let state = AppState::new(
        server.uri().as_str(),
        Duration::from_secs(2),
        Duration::from_secs(2),
    );
    let app = build_router(state, &[]);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/platforms")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["count"], serde_json::json!(2.0));
    let platforms = body["platforms"].as_array().unwrap();
    assert_eq!(platforms.len(), 1);
    assert_eq!(platforms[0]["platform"], "Muscovy");
    assert_eq!(platforms[0]["speed_kmph"], 210.5);
    assert_eq!(platforms[0]["fuel_pct"], 80.0);
}

#[tokio::test]
async fn prometheus_http_error_maps_to_502() {
    let server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("GET"))
        .respond_with(wiremock::ResponseTemplate::new(503).set_body_string("down"))
        .mount(&server)
        .await;
    let state = AppState::new(
        server.uri().as_str(),
        Duration::from_secs(2),
        Duration::from_secs(2),
    );
    let app = build_router(state, &[]);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/factory")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_GATEWAY);
}

#[tokio::test]
async fn all_endpoints_return_2xx_on_success() {
    let server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("GET"))
        .respond_with(
            wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "success",
                "data": { "resultType": "vector", "result": [] }
            })),
        )
        .mount(&server)
        .await;
    let state = AppState::new(
        server.uri().as_str(),
        Duration::from_secs(2),
        Duration::from_secs(2),
    );
    let app = build_router(state, &[]);

    for path in [
        "/api/health",
        "/api/overview?range=6h",
        "/api/production?range=24h",
        "/api/power?range=7d",
        "/api/factory",
        "/api/logistics",
        "/api/trains",
        "/api/research",
        "/api/platforms",
    ] {
        let resp = app
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = resp.status();
        assert!(status.is_success(), "{path} returned {status}");
    }
}

#[tokio::test]
async fn cors_layer_reflects_configured_origin() {
    let server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("GET"))
        .respond_with(
            wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "success",
                "data": { "resultType": "vector", "result": [] }
            })),
        )
        .mount(&server)
        .await;
    let state = AppState::new(
        server.uri().as_str(),
        Duration::from_secs(2),
        Duration::from_secs(2),
    );
    let app = build_router(state, &["https://factorio.example.com".to_string()]);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .header("Origin", "https://factorio.example.com")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let allow_origin = resp
        .headers()
        .get("access-control-allow-origin")
        .and_then(|v| v.to_str().ok());
    assert_eq!(allow_origin, Some("https://factorio.example.com"));
}

#[tokio::test]
async fn cors_disabled_without_origins() {
    let server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("GET"))
        .respond_with(
            wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "success",
                "data": { "resultType": "vector", "result": [] }
            })),
        )
        .mount(&server)
        .await;
    let state = AppState::new(
        server.uri().as_str(),
        Duration::from_secs(2),
        Duration::from_secs(2),
    );
    let app = build_router(state, &[]);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .header("Origin", "https://evil.example.com")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(resp.headers().get("access-control-allow-origin").is_none());
}
