//! Centralized PromQL definitions.
//!
//! Every query the dashboard performs lives here so metric names and
//! aggregation logic can be audited in one place. Handlers never embed
//! raw PromQL.

use std::fmt;

use crate::promql::range::TimeRange;

/// A named, parameterized PromQL query template.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Query {
    // ---- liveness ----
    /// Is the Factorio metrics job present in Prometheus?
    FactorioTargetUp,
    /// Prometheus self-scrape liveness.
    PrometheusUp,

    // ---- game state ----
    EvolutionFactor,
    PollutionOverall,
    GameTicksPlayed,
    CurrentResearch,
    ResearchCompletedCount,
    ResearchQueueSize,
    ResearchProgressPct,

    // ---- power ----
    PowerProduction,
    PowerConsumption,
    PowerProductionBySource,
    AccumulatorCharge,
    Satisfaction,

    // ---- production ----
    ItemProduced,
    ItemConsumed,
    ItemProducedRate,
    ItemConsumedRate,
    FluidProduced,
    FluidConsumed,
    ResearchPackagesBuilt,

    // ---- factory machines ----
    MachineCounts,

    // ---- logistics ----
    RobotCounts,
    ChestContents,
    NetworkIdCount,

    // ---- trains ----
    TrainCounts,
    TrainStations,
    TrainStationWaiting,

    // ---- space platforms ----
    PlatformCount,
    PlatformSpeed,
    PlatformFuel,
}

impl Query {
    /// Canonical query name used in logs.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Query::FactorioTargetUp => "factorio_target_up",
            Query::PrometheusUp => "prometheus_up",
            Query::EvolutionFactor => "evolution_factor",
            Query::PollutionOverall => "pollution_overall",
            Query::GameTicksPlayed => "game_ticks_played",
            Query::CurrentResearch => "current_research",
            Query::ResearchCompletedCount => "research_completed_count",
            Query::ResearchQueueSize => "research_queue_size",
            Query::ResearchProgressPct => "research_progress_pct",
            Query::PowerProduction => "power_production",
            Query::PowerConsumption => "power_consumption",
            Query::PowerProductionBySource => "power_production_by_source",
            Query::AccumulatorCharge => "accumulator_charge",
            Query::Satisfaction => "satisfaction",
            Query::ItemProduced => "item_produced",
            Query::ItemConsumed => "item_consumed",
            Query::ItemProducedRate => "item_produced_rate",
            Query::ItemConsumedRate => "item_consumed_rate",
            Query::FluidProduced => "fluid_produced",
            Query::FluidConsumed => "fluid_consumed",
            Query::ResearchPackagesBuilt => "research_packages_built",
            Query::MachineCounts => "machine_counts",
            Query::RobotCounts => "robot_counts",
            Query::ChestContents => "chest_contents",
            Query::NetworkIdCount => "network_id_count",
            Query::TrainCounts => "train_counts",
            Query::TrainStations => "train_stations",
            Query::TrainStationWaiting => "train_station_waiting",
            Query::PlatformCount => "platform_count",
            Query::PlatformSpeed => "platform_speed",
            Query::PlatformFuel => "platform_fuel",
        }
    }

    /// Builds the PromQL string for this query.
    ///
    /// `range` is used by rate()/delta() style queries to select the
    /// lookback window; state-style gauges ignore it.
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn promql(self, range: TimeRange) -> String {
        let r = range.selector();
        match self {
            Query::FactorioTargetUp => "up{job=\"factorio-metrics\"}".to_string(),
            Query::PrometheusUp => "up".to_string(),
            Query::EvolutionFactor => "graftorio_factorio_evolution_factor{type=\"time\"}".to_string(),
            Query::PollutionOverall => {
                "sum(graftorio_factorio_pollution{type=\"overall\"})".to_string()
            }
            Query::GameTicksPlayed => "max(graftorio_factorio_game_tick)".to_string(),
            Query::CurrentResearch => "graftorio_factorio_research_in_progress".to_string(),
            Query::ResearchCompletedCount => {
                "graftorio_factorio_research_completed_total".to_string()
            }
            Query::ResearchQueueSize => "graftorio_factorio_research_queue_size".to_string(),
            Query::ResearchProgressPct => {
                format!("avg_over_time(graftorio_factorio_research_progress{{type=\"progress\"}}{r}) * 100")
            }
            // graftorio3 power gauges are joules produced/consumed per game
            // tick; multiply by 60 ticks/s to get watts, and by 1e-6 for MW.
            Query::PowerProduction => {
                "sum(graftorio_factorio_power_production{type=\"overall\"}) * 60 / 1000000"
                    .to_string()
            }
            Query::PowerConsumption => {
                "sum(graftorio_factorio_power_consumption{type=\"overall\"}) * 60 / 1000000"
                    .to_string()
            }
            Query::PowerProductionBySource => {
                "sum by (source) (graftorio_factorio_power_production{type=\"by_source\"}) * 60 / 1000000"
                    .to_string()
            }
            Query::AccumulatorCharge => "graftorio_factorio_accumulator_charge".to_string(),
            Query::Satisfaction => "graftorio_factorio_satisfaction".to_string(),
            Query::ItemProduced => format!(
                "topk(8, sum by (item) (rate(graftorio_factorio_item_produced_total{r})))"
            ),
            Query::ItemConsumed => format!(
                "topk(8, sum by (item) (rate(graftorio_factorio_item_consumed_total{r})))"
            ),
            Query::ItemProducedRate => {
                format!("sum by (item) (rate(graftorio_factorio_item_produced_total{r}))")
            }
            Query::ItemConsumedRate => {
                format!("sum by (item) (rate(graftorio_factorio_item_consumed_total{r}))")
            }
            Query::FluidProduced => format!(
                "topk(8, sum by (fluid) (rate(graftorio_factorio_fluid_produced_total{r})))"
            ),
            Query::FluidConsumed => format!(
                "topk(8, sum by (fluid) (rate(graftorio_factorio_fluid_consumed_total{r})))"
            ),
            Query::ResearchPackagesBuilt => {
                format!("sum(rate(graftorio_factorio_research_packages_built_total{r})) * 60")
            }
            Query::MachineCounts => "graftorio_factorio_machine_count".to_string(),
            Query::RobotCounts => "graftorio_factorio_robot_count".to_string(),
            Query::ChestContents => "graftorio_factorio_chest_contents".to_string(),
            Query::NetworkIdCount => "count(graftorio_factorio_chest_contents)".to_string(),
            Query::TrainCounts => "graftorio_factorio_train_count".to_string(),
            Query::TrainStations => "graftorio_factorio_train_station_count".to_string(),
            Query::TrainStationWaiting => "graftorio_factorio_train_station_waiting".to_string(),
            Query::PlatformCount => "count(graftorio_factorio_platform_speed)".to_string(),
            Query::PlatformSpeed => "graftorio_factorio_platform_speed".to_string(),
            Query::PlatformFuel => "graftorio_factorio_platform_fuel".to_string(),
        }
    }
}

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_produced_uses_range_selector() {
        let q = Query::ItemProduced.promql(TimeRange::OneHour);
        assert!(q.contains("[1h]"), "query: {q}");
        assert!(q.contains("topk(8"), "query: {q}");
    }

    #[test]
    fn power_queries_scale_to_megawatts() {
        let q = Query::PowerProduction.promql(TimeRange::SixHours);
        assert!(q.contains("* 60"), "expected ticks/s scaling in: {q}");
        assert!(q.contains("/ 1000000"), "expected MW division in: {q}");
    }

    #[test]
    fn every_query_is_non_empty_for_all_ranges() {
        let queries = [
            Query::FactorioTargetUp,
            Query::PrometheusUp,
            Query::EvolutionFactor,
            Query::PollutionOverall,
            Query::GameTicksPlayed,
            Query::CurrentResearch,
            Query::ResearchCompletedCount,
            Query::ResearchQueueSize,
            Query::ResearchProgressPct,
            Query::PowerProduction,
            Query::PowerConsumption,
            Query::PowerProductionBySource,
            Query::AccumulatorCharge,
            Query::Satisfaction,
            Query::ItemProduced,
            Query::ItemConsumed,
            Query::ItemProducedRate,
            Query::ItemConsumedRate,
            Query::FluidProduced,
            Query::FluidConsumed,
            Query::ResearchPackagesBuilt,
            Query::MachineCounts,
            Query::RobotCounts,
            Query::ChestContents,
            Query::NetworkIdCount,
            Query::TrainCounts,
            Query::TrainStations,
            Query::TrainStationWaiting,
            Query::PlatformCount,
            Query::PlatformSpeed,
            Query::PlatformFuel,
        ];
        for range in TimeRange::all() {
            for query in queries {
                let promql = query.promql(range);
                assert!(!promql.is_empty(), "{} produced empty PromQL", query.name());
                assert!(!promql.contains('\n'), "{} contains newline", query.name());
            }
        }
    }
}
