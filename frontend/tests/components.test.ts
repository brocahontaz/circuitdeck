/**
 * Component tests for the shared UI primitives: states, stat, bar list, panel.
 */
import { cleanup, render, screen } from "@testing-library/svelte";
import { afterEach, describe, expect, it } from "vitest";
import StateMessage from "../src/lib/StateMessage.svelte";
import Stat from "../src/lib/Stat.svelte";
import BarList from "../src/lib/BarList.svelte";

describe("StateMessage", () => {
  afterEach(() => {
    cleanup();
  });

  it("shows the default loading text and is announced", () => {
    render(StateMessage, { props: { state: "loading" } });
    const el = screen.getByRole("status");
    expect(el).toHaveTextContent("Loading telemetry…");
  });

  it("shows custom message for error state", () => {
    render(StateMessage, {
      props: { state: "error", message: "Prometheus exploded" },
    });
    expect(screen.getByRole("status")).toHaveTextContent("Prometheus exploded");
  });

  it("shows degraded text by default", () => {
    render(StateMessage, { props: { state: "degraded" } });
    expect(screen.getByRole("status")).toHaveTextContent("degraded");
  });
});

describe("Stat", () => {
  afterEach(() => {
    cleanup();
  });

  it("renders label and formatted value with suffix", () => {
    render(Stat, {
      props: { label: "Power production", value: 42.51, suffix: "MW" },
    });
    expect(screen.getByText("Power production")).toBeInTheDocument();
    expect(screen.getByText("42.5")).toBeInTheDocument();
    expect(screen.getByText("MW")).toBeInTheDocument();
  });

  it("renders a dash for missing values", () => {
    render(Stat, { props: { label: "Pollution", value: null } });
    expect(screen.getByText("—")).toBeInTheDocument();
  });
});

describe("BarList", () => {
  afterEach(() => {
    cleanup();
  });

  it("renders one row per reading with values", () => {
    render(BarList, {
      props: {
        readings: [
          { label: "iron-plate", value: 100 },
          { label: "copper-plate", value: 50 },
        ],
        suffix: "/min",
      },
    });
    expect(screen.getByText("iron plate")).toBeInTheDocument();
    expect(screen.getByText("copper plate")).toBeInTheDocument();
    expect(screen.getByText("100.0/min")).toBeInTheDocument();
  });

  it("renders the empty text when there is no data", () => {
    render(BarList, {
      props: { readings: [], emptyText: "No production recorded." },
    });
    expect(screen.getByText("No production recorded.")).toBeInTheDocument();
  });
});
