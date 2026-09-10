//! Domain models and Prometheus-to-domain mapping.

pub mod mapping;
pub mod models;

pub use mapping::{
    avg_latest_points, build_factory, build_health, build_logistics, build_overview,
    build_platforms, build_power, build_production, build_research, build_trains,
};
pub use models::*;
