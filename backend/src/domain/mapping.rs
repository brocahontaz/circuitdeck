//! Mapping from Prometheus query results into domain models.

use crate::domain::*;
use crate::error::ApiError;
use crate::prom::client::{InstantResult, RangeResult};
use crate::promql::{Query, TimeRange};

/// Extracts a scalar from a single-series instant result.
fn scalar(result: &InstantResult) -> Option<f64> {
    result
        .first()
        .and_then(|s| s.samples.first())
        .and_then(|s| s.value)
}

/// Extracts a label-carrying scalar list from a multi-series instant result.
fn readings(
    result: &InstantResult,
    label: &str,
    name_fn: impl Fn(&std::collections::HashMap<String, String>) -> String,
) -> Vec<Reading> {
    let mut out = Vec::new();
    for series in result {
        let value = match series.samples.first().and_then(|s| s.value) {
            Some(v) => v,
            None => continue,
        };
        let label_value = series
            .labels
            .get(label)
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        out.push(Reading {
            label: name_fn(&series.labels).replace("{}", &label_value),
            value,
        });
    }
    out.sort_by(|a, b| a.label.cmp(&b.label));
    out
}

/// Simple per-label readings.
fn labeled(result: &InstantResult, label: &str) -> Vec<Reading> {
    readings(result, label, |labels| {
        labels
            .get(label)
            .cloned()
            .unwrap_or_else(|| "unknown".into())
    })
}

/// Converts a range result into chart series.
fn to_series(result: &crate::prom::client::RangeResult, label: &str) -> Vec<Series> {
    let mut out = Vec::new();
    for series in result {
        let name = series
            .labels
            .get(label)
            .cloned()
            .unwrap_or_else(|| "total".to_string());
        let points = series
            .points
            .iter()
            .map(|p| SeriesPoint {
                t: p.timestamp,
                v: p.value,
            })
            .collect();
        out.push(Series { name, points });
    }
    out
}

/// Fetches a single instant query and maps hard failures to `None` when
/// `optional` is set, otherwise propagates the error.
async fn fetch_scalar(
    ctx: &crate::api::state::QueryContext,
    query: Query,
    range: TimeRange,
) -> Result<Option<f64>, ApiError> {
    let promql = query.promql(range);
    let result = ctx.client.query(&promql).await?;
    Ok(scalar(&result))
}

/// Fetches a multi-series instant query, preserving per-series absence as empty.
async fn fetch_series_instant(
    ctx: &crate::api::state::QueryContext,
    query: Query,
    range: TimeRange,
) -> Result<InstantResult, ApiError> {
    let promql = query.promql(range);
    ctx.client.query(&promql).await
}

/// Fetches a range query for charting.
async fn fetch_range(
    ctx: &crate::api::state::QueryContext,
    query: Query,
    range: TimeRange,
) -> Result<RangeResult, ApiError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let start = now - range.secs();
    let promql = query.promql(range);
    ctx.client
        .query_range(&promql, start, now, range.step_secs())
        .await
}

/// Computes the average of the latest points of a series list.
fn avg_latest(series: &[Series]) -> Option<f64> {
    let mut total = 0.0;
    let mut count = 0;
    for s in series {
        if let Some(p) = s.points.last() {
            if let Some(v) = p.v {
                total += v;
                count += 1;
            }
        }
    }
    if count == 0 {
        None
    } else {
        Some(total / count as f64)
    }
}

/// Builds the health report.
///
/// Prometheus reachability is checked with a ping; the factorio job
/// presence is derived from the `up` metric for that job.
pub async fn build_health(ctx: &crate::api::state::QueryContext) -> Result<HealthReport, ApiError> {
    let prometheus_reachable = ctx.client.ping().await.is_ok();
    let factorio = if prometheus_reachable {
        ctx.client
            .query(&Query::FactorioTargetUp.promql(TimeRange::OneHour))
            .await
            .ok()
            .and_then(|r| scalar(&r))
            .unwrap_or(0.0)
            > 0.0
    } else {
        false
    };
    let game_ticks_played = if prometheus_reachable {
        ctx.client
            .query(&Query::GameTicksPlayed.promql(TimeRange::OneHour))
            .await
            .ok()
            .and_then(|r| scalar(&r))
    } else {
        None
    };
    Ok(HealthReport {
        prometheus_reachable,
        factorio_metrics_job_present: factorio,
        game_ticks_played,
    })
}

/// Builds the overview payload.
#[allow(clippy::too_many_lines)]
pub async fn build_overview(
    ctx: &crate::api::state::QueryContext,
    range: TimeRange,
) -> Result<Overview, ApiError> {
    let health = build_health(ctx).await?;
    if !health.prometheus_reachable {
        return Ok(Overview {
            health,
            evolution_factor: None,
            pollution: None,
            power_production_mw: None,
            power_consumption_mw: None,
            accumulator_charge_pct: None,
            satisfaction_pct: None,
            top_produced: Vec::new(),
            top_consumed: Vec::new(),
            current_research: None,
            research_progress_pct: None,
            platform_count: None,
            machine_counts: Vec::new(),
            robot_counts: Vec::new(),
            train_counts: Vec::new(),
        });
    }

    let (evolution, pollution, power_prod, power_cons, accu, satisfaction) = tokio::join!(
        fetch_scalar(ctx, Query::EvolutionFactor, range),
        fetch_scalar(ctx, Query::PollutionOverall, range),
        fetch_scalar(ctx, Query::PowerProduction, range),
        fetch_scalar(ctx, Query::PowerConsumption, range),
        fetch_scalar(ctx, Query::AccumulatorCharge, range),
        fetch_scalar(ctx, Query::Satisfaction, range),
    );
    let (top_produced, top_consumed, research, research_progress) = tokio::join!(
        fetch_series_instant(ctx, Query::ItemProduced, range),
        fetch_series_instant(ctx, Query::ItemConsumed, range),
        fetch_series_instant(ctx, Query::CurrentResearch, range),
        fetch_scalar(ctx, Query::ResearchProgressPct, range),
    );
    let (machines, robots, trains, platforms) = tokio::join!(
        fetch_series_instant(ctx, Query::MachineCounts, range),
        fetch_series_instant(ctx, Query::RobotCounts, range),
        fetch_series_instant(ctx, Query::TrainCounts, range),
        fetch_scalar(ctx, Query::PlatformCount, range),
    );

    // The research-in-progress series carries the tech name as its label;
    // the in-progress tech is the one with the lowest progress value.
    let current_research = research?
        .iter()
        .filter(|s| s.samples.first().and_then(|s| s.value).is_some())
        .min_by(|a, b| {
            let va = a.samples.first().and_then(|s| s.value).unwrap_or(f64::MAX);
            let vb = b.samples.first().and_then(|s| s.value).unwrap_or(f64::MAX);
            va.partial_cmp(&vb).unwrap_or(std::cmp::Ordering::Equal)
        })
        .and_then(|s| s.labels.get("tech").cloned());

    Ok(Overview {
        health,
        evolution_factor: evolution?,
        pollution: pollution?,
        power_production_mw: power_prod?,
        power_consumption_mw: power_cons?,
        accumulator_charge_pct: accu?.map(|v| v * 100.0),
        satisfaction_pct: satisfaction?.map(|v| v * 100.0),
        top_produced: labeled(&top_produced?, "item"),
        top_consumed: labeled(&top_consumed?, "item"),
        current_research,
        research_progress_pct: research_progress?,
        platform_count: platforms?,
        machine_counts: labeled(&machines?, "machine"),
        robot_counts: labeled(&robots?, "robot"),
        train_counts: labeled(&trains?, "state"),
    })
}

/// Builds the production report.
pub async fn build_production(
    ctx: &crate::api::state::QueryContext,
    range: TimeRange,
) -> Result<ProductionReport, ApiError> {
    let (produced, consumed, produced_series, consumed_series, packages) = tokio::join!(
        fetch_series_instant(ctx, Query::ItemProduced, range),
        fetch_series_instant(ctx, Query::ItemConsumed, range),
        fetch_range(ctx, Query::ItemProducedRate, range),
        fetch_range(ctx, Query::ItemConsumedRate, range),
        fetch_scalar(ctx, Query::ResearchPackagesBuilt, range),
    );
    Ok(ProductionReport {
        range: range.as_str().to_string(),
        top_produced: labeled(&produced?, "item"),
        top_consumed: labeled(&consumed?, "item"),
        produced_series: to_series(&produced_series?, "item"),
        consumed_series: to_series(&consumed_series?, "item"),
        research_packages_per_minute: packages?,
    })
}

/// Builds the power report.
pub async fn build_power(
    ctx: &crate::api::state::QueryContext,
    range: TimeRange,
) -> Result<PowerReport, ApiError> {
    let (prod, cons, by_source, prod_series, cons_series, accu, satisfaction) = tokio::join!(
        fetch_scalar(ctx, Query::PowerProduction, range),
        fetch_scalar(ctx, Query::PowerConsumption, range),
        fetch_series_instant(ctx, Query::PowerProductionBySource, range),
        fetch_range(ctx, Query::PowerProduction, range),
        fetch_range(ctx, Query::PowerConsumption, range),
        fetch_scalar(ctx, Query::AccumulatorCharge, range),
        fetch_scalar(ctx, Query::Satisfaction, range),
    );
    Ok(PowerReport {
        range: range.as_str().to_string(),
        production_mw: prod?,
        consumption_mw: cons?,
        production_by_source: labeled(&by_source?, "source"),
        production_series: to_series(&prod_series?, "source"),
        consumption_series: to_series(&cons_series?, "source"),
        accumulator_charge_pct: accu?.map(|v| v * 100.0),
        satisfaction_pct: satisfaction?.map(|v| v * 100.0),
    })
}

/// Builds the factory report.
pub async fn build_factory(
    ctx: &crate::api::state::QueryContext,
    range: TimeRange,
) -> Result<FactoryReport, ApiError> {
    let machines = fetch_series_instant(ctx, Query::MachineCounts, range).await?;
    let rows = labeled(&machines, "machine");
    Ok(FactoryReport {
        machines: rows
            .into_iter()
            .map(|r| MachineStatus {
                machine: r.label,
                active: Some(r.value),
                total: None,
            })
            .collect(),
    })
}

/// Builds the logistics report.
pub async fn build_logistics(
    ctx: &crate::api::state::QueryContext,
    range: TimeRange,
) -> Result<LogisticsReport, ApiError> {
    let (robots, networks, chest) = tokio::join!(
        fetch_series_instant(ctx, Query::RobotCounts, range),
        fetch_scalar(ctx, Query::NetworkIdCount, range),
        fetch_series_instant(ctx, Query::ChestContents, range),
    );
    let mut top_stored = labeled(&chest?, "item");
    top_stored.sort_by(|a, b| {
        b.value
            .partial_cmp(&a.value)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    top_stored.truncate(10);
    Ok(LogisticsReport {
        robots: labeled(&robots?, "robot"),
        networks: networks?,
        top_stored_items: top_stored,
    })
}

/// Builds the trains report.
pub async fn build_trains(
    ctx: &crate::api::state::QueryContext,
    range: TimeRange,
) -> Result<TrainsReport, ApiError> {
    let (trains, stations, waiting) = tokio::join!(
        fetch_series_instant(ctx, Query::TrainCounts, range),
        fetch_scalar(ctx, Query::TrainStations, range),
        fetch_series_instant(ctx, Query::TrainStationWaiting, range),
    );
    Ok(TrainsReport {
        trains_by_state: labeled(&trains?, "state"),
        station_count: stations?,
        stations_waiting: labeled(&waiting?, "station"),
    })
}

/// Builds the research report.
pub async fn build_research(
    ctx: &crate::api::state::QueryContext,
    range: TimeRange,
) -> Result<ResearchReport, ApiError> {
    let (progress_pct, completed, queue, in_progress) = tokio::join!(
        fetch_scalar(ctx, Query::ResearchProgressPct, range),
        fetch_scalar(ctx, Query::ResearchCompletedCount, range),
        fetch_scalar(ctx, Query::ResearchQueueSize, range),
        fetch_series_instant(ctx, Query::CurrentResearch, range),
    );
    let current_research = in_progress?
        .iter()
        .filter(|s| s.samples.first().and_then(|s| s.value).is_some())
        .min_by(|a, b| {
            let va = a.samples.first().and_then(|s| s.value).unwrap_or(f64::MAX);
            let vb = b.samples.first().and_then(|s| s.value).unwrap_or(f64::MAX);
            va.partial_cmp(&vb).unwrap_or(std::cmp::Ordering::Equal)
        })
        .and_then(|s| s.labels.get("tech").cloned());
    Ok(ResearchReport {
        current_research,
        progress_pct: progress_pct?,
        completed_count: completed?,
        queue_size: queue?,
    })
}

/// Builds the platforms report.
pub async fn build_platforms(
    ctx: &crate::api::state::QueryContext,
    range: TimeRange,
) -> Result<PlatformReport, ApiError> {
    let (count, speed, fuel) = tokio::join!(
        fetch_scalar(ctx, Query::PlatformCount, range),
        fetch_series_instant(ctx, Query::PlatformSpeed, range),
        fetch_series_instant(ctx, Query::PlatformFuel, range),
    );
    let speeds = labeled(&speed?, "platform");
    let fuels = labeled(&fuel?, "platform");
    let mut names: Vec<String> = speeds.iter().map(|r| r.label.clone()).collect();
    for f in &fuels {
        if !names.contains(&f.label) {
            names.push(f.label.clone());
        }
    }
    names.sort();
    let platforms = names
        .into_iter()
        .map(|name| {
            let speed_kmph = speeds.iter().find(|s| s.label == name).map(|s| s.value);
            let fuel_pct = fuels
                .iter()
                .find(|f| f.label == name)
                .map(|f| f.value * 100.0);
            PlatformStatus {
                platform: name,
                speed_kmph,
                fuel_pct,
            }
        })
        .collect();
    Ok(PlatformReport {
        count: count?,
        platforms,
    })
}

/// Average across latest points; exposed for reuse in tests.
pub fn avg_latest_points(series: &[Series]) -> Option<f64> {
    avg_latest(series)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prom::client::{RangePoint, RangeSeries, SampleSeries};
    use crate::prom::response::Sample;
    use std::collections::HashMap;

    fn labels(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn scalar_extracts_first_sample() {
        let result = vec![SampleSeries {
            labels: labels(&[("job", "factorio")]),
            samples: vec![Sample {
                metric: HashMap::new(),
                timestamp: 100.0,
                value: Some(42.0),
            }],
        }];
        assert_eq!(scalar(&result), Some(42.0));
    }

    #[test]
    fn scalar_empty_result_is_none() {
        let empty: InstantResult = Vec::new();
        assert_eq!(scalar(&empty), None);
    }

    #[test]
    fn labeled_uses_label_fallback() {
        let result = vec![SampleSeries {
            labels: HashMap::new(),
            samples: vec![Sample {
                metric: HashMap::new(),
                timestamp: 100.0,
                value: Some(1.0),
            }],
        }];
        let rows = labeled(&result, "item");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].label, "unknown");
    }

    #[test]
    fn to_series_maps_points() {
        let result = vec![RangeSeries {
            labels: labels(&[("item", "iron-plate")]),
            points: vec![
                RangePoint {
                    timestamp: 1.0,
                    value: Some(5.0),
                },
                RangePoint {
                    timestamp: 2.0,
                    value: None,
                },
            ],
        }];
        let series = to_series(&result, "item");
        assert_eq!(series.len(), 1);
        assert_eq!(series[0].name, "iron-plate");
        assert_eq!(series[0].points.len(), 2);
        assert_eq!(series[0].points[1].v, None);
    }

    #[test]
    fn avg_latest_averages_series_ends() {
        let series = vec![
            Series {
                name: "a".into(),
                points: vec![
                    SeriesPoint {
                        t: 1.0,
                        v: Some(2.0),
                    },
                    SeriesPoint {
                        t: 2.0,
                        v: Some(4.0),
                    },
                ],
            },
            Series {
                name: "b".into(),
                points: vec![SeriesPoint {
                    t: 2.0,
                    v: Some(6.0),
                }],
            },
        ];
        assert_eq!(avg_latest(&series), Some(5.0));
    }
}
