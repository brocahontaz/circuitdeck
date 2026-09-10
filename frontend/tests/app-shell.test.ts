/**
 * Regression test for the App shell fetch loop:
 * a completed fetch must NOT re-trigger the load effect (Svelte 5 `$effect`
 * tracks writes to stores read inside it, so store reads must be untracked).
 */
import { cleanup, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const fetchMock = vi.fn();

vi.mock("../src/lib/api", async (importOriginal) => {
  const actual = await importOriginal<typeof import("../src/lib/api")>();
  return {
    ...actual,
    fetchHealth: vi.fn(async () => ({
      prometheus_reachable: true,
      factorio_metrics_job_present: true,
      game_ticks_played: 1,
    })),
    fetchOverview: vi.fn(async () => ({
      health: {
        prometheus_reachable: true,
        factorio_metrics_job_present: true,
        game_ticks_played: 1,
      },
      evolution_factor: 0.5,
      pollution: 100,
      power_production_mw: 1,
      power_consumption_mw: 1,
      accumulator_charge_pct: 90,
      satisfaction_pct: 99,
      top_produced: [],
      top_consumed: [],
      current_research: null,
      research_progress_pct: null,
      platform_count: null,
      machine_counts: [],
      robot_counts: [],
      train_counts: [],
    })),
  };
});

import App from "../src/App.svelte";
import { fetchOverview } from "../src/lib/api";

describe("App shell", () => {
  beforeEach(() => {
    fetchMock.mockResolvedValue(
      new Response(JSON.stringify({}), { status: 200 }),
    );
    vi.stubGlobal("fetch", fetchMock);
    window.location.hash = "";
  });

  afterEach(() => {
    cleanup();
    vi.unstubAllGlobals();
    vi.useRealTimers();
  });

  it("loads the overview once per view change, not per store write", async () => {
    render(App);
    // Initial effect run: 1 overview fetch + 1 health fetch.
    await waitFor(() => {
      expect(fetchOverview).toHaveBeenCalledTimes(1);
    });
    // Let microtasks + any (buggy) re-triggering settle.
    await new Promise((r) => setTimeout(r, 150));
    expect(fetchOverview).toHaveBeenCalledTimes(1);
    expect(screen.getByText(/Prometheus reachable/)).toBeInTheDocument();
  });

  it("re-fetches when the range changes", async () => {
    render(App);
    await waitFor(() => {
      expect(fetchOverview).toHaveBeenCalledTimes(1);
    });
    // 6h range button.
    const buttons = screen
      .getAllByRole("button")
      .filter((b) => b.textContent?.trim() === "6h");
    expect(buttons.length).toBeGreaterThan(0);
    buttons[0].click();
    await waitFor(() => {
      expect(fetchOverview).toHaveBeenCalledTimes(2);
    });
    expect(fetchOverview).toHaveBeenLastCalledWith("6h");
  });
});
