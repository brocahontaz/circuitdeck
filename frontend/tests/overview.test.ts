/**
 * Tests for the Overview view's state handling.
 */
import { cleanup, render, screen } from "@testing-library/svelte";
import { afterEach, describe, expect, it } from "vitest";
import Overview from "../src/views/Overview.svelte";
import type { Overview as OverviewData } from "../src/lib/api";

function makeOverview(): OverviewData {
  return {
    health: {
      prometheus_reachable: true,
      factorio_metrics_job_present: true,
      game_ticks_played: 1234567,
    },
    evolution_factor: 0.62,
    pollution: 480,
    power_production_mw: 42.3,
    power_consumption_mw: 39.8,
    accumulator_charge_pct: 86,
    satisfaction_pct: 98,
    top_produced: [
      { label: "iron-plate", value: 120.4 },
      { label: "copper-plate", value: 60.2 },
    ],
    top_consumed: [{ label: "iron-plate", value: 110 }],
    current_research: "oil-processing",
    research_progress_pct: 42,
    platform_count: 2,
    machine_counts: [{ label: "electric-furnace", value: 48 }],
    robot_counts: [{ label: "logistic-robot", value: 250 }],
    train_counts: [{ label: "waiting-at-station", value: 14 }],
  };
}

describe("Overview view", () => {
  afterEach(() => {
    cleanup();
  });

  it("shows loading state while the first fetch is in flight", () => {
    render(Overview, { props: { data: null, loading: true, error: null } });
    expect(screen.getByRole("status")).toHaveTextContent("Loading telemetry…");
  });

  it("shows an error state on failure", () => {
    render(Overview, {
      props: { data: null, loading: false, error: "API returned 502" },
    });
    expect(screen.getByText("API returned 502")).toBeInTheDocument();
  });

  it("renders telemetry health and key stats with data", () => {
    render(Overview, {
      props: { data: makeOverview(), loading: false, error: null },
    });
    expect(screen.getByText(/Prometheus reachable/)).toBeInTheDocument();
    expect(screen.getByText(/Factorio metrics flowing/)).toBeInTheDocument();
    expect(screen.getByText("Power production")).toBeInTheDocument();
    expect(screen.getAllByText("iron plate").length).toBeGreaterThan(0);
    expect(screen.getByText("Current research")).toBeInTheDocument();
    expect(screen.getByText("oil processing")).toBeInTheDocument();
  });

  it("renders degraded banner when prometheus is unreachable", () => {
    const data = makeOverview();
    data.health.prometheus_reachable = false;
    render(Overview, { props: { data, loading: false, error: null } });
    expect(
      screen.getAllByText(/Prometheus unreachable/).length,
    ).toBeGreaterThan(0);
  });
});
