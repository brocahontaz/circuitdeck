//! Axum router assembly.

use axum::routing::get;
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::api::routes;
use crate::api::state::AppState;

/// Builds the complete API router.
pub fn build_router(state: AppState, cors_origins: &[String]) -> Router {
    let api = Router::new()
        .route("/health", get(routes::health))
        .route("/overview", get(routes::overview))
        .route("/production", get(routes::production))
        .route("/power", get(routes::power))
        .route("/factory", get(routes::factory))
        .route("/logistics", get(routes::logistics))
        .route("/trains", get(routes::trains))
        .route("/research", get(routes::research))
        .route("/platforms", get(routes::platforms));

    let mut app = Router::new().nest("/api", api).with_state(state);
    app = app.layer(TraceLayer::new_for_http());
    if !cors_origins.is_empty() {
        let allow = cors_origins.to_vec();
        app = app.layer(
            CorsLayer::new()
                .allow_origin(
                    allow
                        .iter()
                        .map(|o| o.parse().expect("CORS origin must be a valid header value"))
                        .collect::<Vec<_>>(),
                )
                .allow_methods([axum::http::Method::GET, axum::http::Method::OPTIONS]),
        );
    }
    app
}
