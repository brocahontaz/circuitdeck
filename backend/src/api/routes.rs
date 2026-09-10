//! HTTP route definitions.

use axum::extract::{Query as AxumQuery, State};
use axum::Json;
use serde::Deserialize;

use crate::api::state::AppState;
use crate::domain;
use crate::error::ApiError;
use crate::promql::TimeRange;

/// `GET /api/health` — telemetry dependency health.
pub async fn health(State(state): State<AppState>) -> Result<Json<domain::HealthReport>, ApiError> {
    let ctx = state.context();
    let report = domain::build_health(&ctx).await?;
    Ok(Json(report))
}

/// `GET /api/overview?range=1h`
pub async fn overview(
    State(state): State<AppState>,
    AxumQuery(params): AxumQuery<RangeParams>,
) -> Result<Json<domain::Overview>, ApiError> {
    let range = params.parse_range()?;
    let ctx = state.context();
    Ok(Json(domain::build_overview(&ctx, range).await?))
}

/// `GET /api/production?range=24h`
pub async fn production(
    State(state): State<AppState>,
    AxumQuery(params): AxumQuery<RangeParams>,
) -> Result<Json<domain::ProductionReport>, ApiError> {
    let range = params.parse_range()?;
    let ctx = state.context();
    Ok(Json(domain::build_production(&ctx, range).await?))
}

/// `GET /api/power?range=6h`
pub async fn power(
    State(state): State<AppState>,
    AxumQuery(params): AxumQuery<RangeParams>,
) -> Result<Json<domain::PowerReport>, ApiError> {
    let range = params.parse_range()?;
    let ctx = state.context();
    Ok(Json(domain::build_power(&ctx, range).await?))
}

/// `GET /api/factory?range=1h`
pub async fn factory(
    State(state): State<AppState>,
    AxumQuery(params): AxumQuery<RangeParams>,
) -> Result<Json<domain::FactoryReport>, ApiError> {
    let range = params.parse_range()?;
    let ctx = state.context();
    Ok(Json(domain::build_factory(&ctx, range).await?))
}

/// `GET /api/logistics?range=1h`
pub async fn logistics(
    State(state): State<AppState>,
    AxumQuery(params): AxumQuery<RangeParams>,
) -> Result<Json<domain::LogisticsReport>, ApiError> {
    let range = params.parse_range()?;
    let ctx = state.context();
    Ok(Json(domain::build_logistics(&ctx, range).await?))
}

/// `GET /api/trains?range=1h`
pub async fn trains(
    State(state): State<AppState>,
    AxumQuery(params): AxumQuery<RangeParams>,
) -> Result<Json<domain::TrainsReport>, ApiError> {
    let range = params.parse_range()?;
    let ctx = state.context();
    Ok(Json(domain::build_trains(&ctx, range).await?))
}

/// `GET /api/research?range=1h`
pub async fn research(
    State(state): State<AppState>,
    AxumQuery(params): AxumQuery<RangeParams>,
) -> Result<Json<domain::ResearchReport>, ApiError> {
    let range = params.parse_range()?;
    let ctx = state.context();
    Ok(Json(domain::build_research(&ctx, range).await?))
}

/// `GET /api/platforms?range=1h`
pub async fn platforms(
    State(state): State<AppState>,
    AxumQuery(params): AxumQuery<RangeParams>,
) -> Result<Json<domain::PlatformReport>, ApiError> {
    let range = params.parse_range()?;
    let ctx = state.context();
    Ok(Json(domain::build_platforms(&ctx, range).await?))
}

/// Shared `?range=` parameter for historical endpoints.
#[derive(Debug, Deserialize)]
pub struct RangeParams {
    range: Option<String>,
}

impl RangeParams {
    /// Validates and converts the range parameter.
    ///
    /// # Errors
    /// Returns [`ApiError::InvalidTimeRange`] for unsupported values.
    pub fn parse_range(&self) -> Result<TimeRange, ApiError> {
        match &self.range {
            None => Ok(TimeRange::OneHour),
            Some(s) => TimeRange::parse(s).ok_or_else(|| ApiError::InvalidTimeRange(s.clone())),
        }
    }
}
