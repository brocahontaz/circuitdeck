//! Domain models returned by the dashboard JSON API.
//!
//! These are the clean, Factorio-shaped structures the frontend consumes;
//! raw Prometheus samples never cross the API boundary.

use serde::Serialize;

/// Shared shape for a labeled scalar reading.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Reading {
    pub label: String,
    pub value: f64,
}

/// Health snapshot of telemetry dependencies.
#[derive(Debug, Serialize, PartialEq)]
pub struct HealthReport {
    pub prometheus_reachable: bool,
    pub factorio_metrics_job_present: bool,
    pub game_ticks_played: Option<f64>,
}

/// Top-level dashboard summary.
#[derive(Debug, Serialize, PartialEq)]
pub struct Overview {
    pub health: HealthReport,
    pub evolution_factor: Option<f64>,
    pub pollution: Option<f64>,
    pub power_production_mw: Option<f64>,
    pub power_consumption_mw: Option<f64>,
    pub accumulator_charge_pct: Option<f64>,
    pub satisfaction_pct: Option<f64>,
    pub top_produced: Vec<Reading>,
    pub top_consumed: Vec<Reading>,
    pub current_research: Option<String>,
    pub research_progress_pct: Option<f64>,
    pub platform_count: Option<f64>,
    pub machine_counts: Vec<Reading>,
    pub robot_counts: Vec<Reading>,
    pub train_counts: Vec<Reading>,
}

/// Production flows over a time range.
#[derive(Debug, Serialize, PartialEq)]
pub struct ProductionReport {
    pub range: String,
    pub top_produced: Vec<Reading>,
    pub top_consumed: Vec<Reading>,
    pub produced_series: Vec<Series>,
    pub consumed_series: Vec<Series>,
    pub research_packages_per_minute: Option<f64>,
}

/// Power statistics over a time range.
#[derive(Debug, Serialize, PartialEq)]
pub struct PowerReport {
    pub range: String,
    pub production_mw: Option<f64>,
    pub consumption_mw: Option<f64>,
    pub production_by_source: Vec<Reading>,
    pub production_series: Vec<Series>,
    pub consumption_series: Vec<Series>,
    pub accumulator_charge_pct: Option<f64>,
    pub satisfaction_pct: Option<f64>,
}

/// Factory machine census.
#[derive(Debug, Serialize, PartialEq)]
pub struct FactoryReport {
    pub machines: Vec<MachineStatus>,
}

/// One machine type census row.
#[derive(Debug, Serialize, PartialEq)]
pub struct MachineStatus {
    pub machine: String,
    pub active: Option<f64>,
    pub total: Option<f64>,
}

/// Logistics network status.
#[derive(Debug, Serialize, PartialEq)]
pub struct LogisticsReport {
    pub robots: Vec<Reading>,
    pub networks: Option<f64>,
    pub top_stored_items: Vec<Reading>,
}

/// Train status.
#[derive(Debug, Serialize, PartialEq)]
pub struct TrainsReport {
    pub trains_by_state: Vec<Reading>,
    pub station_count: Option<f64>,
    pub stations_waiting: Vec<Reading>,
}

/// Research status.
#[derive(Debug, Serialize, PartialEq)]
pub struct ResearchReport {
    pub current_research: Option<String>,
    pub progress_pct: Option<f64>,
    pub completed_count: Option<f64>,
    pub queue_size: Option<f64>,
}

/// Space platform status.
#[derive(Debug, Serialize, PartialEq)]
pub struct PlatformReport {
    pub count: Option<f64>,
    pub platforms: Vec<PlatformStatus>,
}

/// One space platform.
#[derive(Debug, Serialize, PartialEq)]
pub struct PlatformStatus {
    pub platform: String,
    pub speed_kmph: Option<f64>,
    pub fuel_pct: Option<f64>,
}

/// A named time series for charting, timestamps in epoch seconds.
#[derive(Debug, Serialize, PartialEq)]
pub struct Series {
    pub name: String,
    pub points: Vec<SeriesPoint>,
}

/// One chart point.
#[derive(Debug, Serialize, PartialEq)]
pub struct SeriesPoint {
    /// Epoch seconds.
    pub t: f64,
    pub v: Option<f64>,
}
