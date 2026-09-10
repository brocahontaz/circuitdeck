#!/usr/bin/env python3
"""CircuitDeck mock Factorio metrics source.

Emits graftorio3-compatible Prometheus metrics on :9105 so the complete
dashboard stack runs without a real Factorio server. The generator is
deterministic (seeded LCG) but drifts over time so charts show believable
movement. In production this container is replaced by node_exporter's
textfile collector on the Factorio host.

Usage: python3 serve.py [--interval SECONDS] [--port PORT]
"""

from __future__ import annotations

import argparse
import math
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

# ---------------------------------------------------------------------------
# Deterministic pseudo-random generator (LCG) so local data looks the same
# across restarts but still changes over time.
# ---------------------------------------------------------------------------


class Lcg:
    """Small deterministic LCG; good enough for mock drift."""

    def __init__(self, seed: int) -> None:
        self.state = seed & 0xFFFFFFFF

    def next(self) -> float:
        self.state = (self.state * 1103515245 + 12345) & 0x7FFFFFFF
        return self.state / 0x7FFFFFFF

    def jitter(self, amplitude: float) -> float:
        return (self.next() - 0.5) * 2 * amplitude


START_TIME = time.time()
RNG = Lcg(20240909)

# ---------------------------------------------------------------------------
# Mock world state
# ---------------------------------------------------------------------------

ITEMS_PRODUCED = {
    "iron-plate": 0.0,
    "copper-plate": 0.0,
    "steel-plate": 0.0,
    "electronic-circuit": 0.0,
    "copper-cable": 0.0,
    "iron-gear-wheel": 0.0,
    "transport-belt": 0.0,
    "inserter": 0.0,
    "science-pack-automation": 0.0,
    "science-pack-logistic": 0.0,
    "plastic-bar": 0.0,
    "battery": 0.0,
}

ITEMS_CONSUMED = {
    "iron-plate": 0.0,
    "copper-plate": 0.0,
    "iron-gear-wheel": 0.0,
    "copper-cable": 0.0,
    "steel-plate": 0.0,
    "plastic-bar": 0.0,
    "electronic-circuit": 0.0,
    "coal": 0.0,
}

FLUIDS_PRODUCED = {
    "crude-oil": 0.0,
    "petroleum-gas": 0.0,
    "lubricant": 0.0,
}

FLUIDS_CONSUMED = {
    "petroleum-gas": 0.0,
    "sulfuric-acid": 0.0,
    "water": 0.0,
}

# base rates per minute (graftorio3 counters advance in game units)
BASE_RATES = {
    "iron-plate": 2400,
    "copper-plate": 1200,
    "steel-plate": 320,
    "electronic-circuit": 800,
    "copper-cable": 1600,
    "iron-gear-wheel": 640,
    "transport-belt": 120,
    "inserter": 40,
    "science-pack-automation": 180,
    "science-pack-logistic": 120,
    "plastic-bar": 400,
    "battery": 150,
}

CONSUME_RATES = {
    "iron-plate": 2200,
    "copper-plate": 1150,
    "iron-gear-wheel": 600,
    "copper-cable": 1500,
    "steel-plate": 300,
    "plastic-bar": 380,
    "electronic-circuit": 750,
    "coal": 900,
}

FLUID_PROD_RATES = {"crude-oil": 900, "petroleum-gas": 600, "lubricant": 40}
FLUID_CONS_RATES = {"petroleum-gas": 560, "sulfuric-acid": 90, "water": 5000}

MACHINES = {
    "electric-furnace": 48,
    "electric-mining-drill": 60,
    "assembling-machine-2": 72,
    "assembling-machine-3": 24,
    "oil-refinery": 8,
    "chemical-plant": 12,
}

ROBOTS = {
    "logistic-robot": 250,
    "construction-robot": 120,
}

CHEST_ITEMS = {
    "iron-plate": 42000,
    "copper-plate": 24000,
    "steel-plate": 5200,
    "electronic-circuit": 18000,
    "coal": 31000,
    "stone": 8600,
    "plastic-bar": 7200,
    "battery": 3400,
    "low-density-structure": 900,
    "rocket-fuel": 450,
    "processing-unit": 1200,
    "speed-module": 180,
}

TRAIN_STATES = {
    "waiting-at-station": 14,
    "moving-to-station": 9,
    "on-the-path": 4,
    "no-path": 0,
    "manual-control": 1,
}

TRAIN_STATIONS = {
    "iron-outpost": 640,
    "copper-outpost": 480,
    "oil-outpost": 220,
    "unloading-station-main": 900,
}

RESEARCH_STEPS = [
    "automation-2",
    "logistics-2",
    "oil-processing",
    "advanced-material-processing",
    "production-science-pack",
    "utility-science-pack",
    "rocket-silo",
    "space-science-pack",
    "fusion-reactor",
    "railgun",
]

PLATFORMS = {
    "muscovy": {"speed": 214, "fuel": 0.82},
    "pintado": {"speed": 168, "fuel": 0.44},
}

state = {
    "tick": 0,
    "research_index": 2,
    "research_progress": 0.35,
    "completed": 21,
    "evolution": 0.62,
    "pollution": 480,
    "accu": 0.86,
    "satisfaction": 0.98,
    "research_pkgs": 0.0,
}


def advance(tick_delta: int) -> None:
    """Advance the mock world; called once per scrape."""
    minutes = tick_delta / 60.0 / 60.0  # 60 ticks/s -> game minutes
    state["tick"] += tick_delta

    # Slow day/night wave on accumulator charge.
    phase = (time.time() - START_TIME) / 120.0  # 2-minute "day"
    state["accu"] = min(1.0, max(0.15, 0.5 + 0.45 * math.sin(phase * math.pi * 2)))
    state["satisfaction"] = max(0.55, min(1.0, 0.95 + 0.05 * math.sin(phase * 7)))

    # Evolution creeps upward with pollution drifting.
    state["evolution"] = min(0.99, state["evolution"] + 0.00002 * tick_delta)
    state["pollution"] = max(100, 480 + RNG.jitter(4))

    # Research rolls over when complete.
    state["research_progress"] += 0.0008 * tick_delta / 60.0
    if state["research_progress"] >= 1.0:
        state["research_progress"] = 0.0
        state["completed"] += 1
        state["research_index"] = (state["research_index"] + 1) % len(RESEARCH_STEPS)

    # Count produced/consumed increments.
    for item, base in BASE_RATES.items():
        wobble = 1.0 + 0.18 * math.sin(phase * 3 + hash(item) % 7) + RNG.jitter(0.02)
        ITEMS_PRODUCED[item] = ITEMS_PRODUCED.get(item, 0.0) + base * wobble * minutes
    for item, base in CONSUME_RATES.items():
        wobble = 1.0 + 0.15 * math.cos(phase * 2 + hash(item) % 5) + RNG.jitter(0.02)
        ITEMS_CONSUMED[item] = ITEMS_CONSUMED.get(item, 0.0) + base * wobble * minutes
    for fluid, base in FLUID_PROD_RATES.items():
        FLUIDS_PRODUCED[fluid] = FLUIDS_PRODUCED.get(fluid, 0.0) + base * minutes * (1 + RNG.jitter(0.04))
    for fluid, base in FLUID_CONS_RATES.items():
        FLUIDS_CONSUMED[fluid] = FLUIDS_CONSUMED.get(fluid, 0.0) + base * minutes * (1 + RNG.jitter(0.04))
    state["research_pkgs"] += (180 + 60 * math.sin(phase * 4)) * minutes


def power_mw() -> tuple[float, float]:
    """Overall production/consumption in MW."""
    prod = 42 + 16 * math.sin((time.time() - START_TIME) / 90) + RNG.jitter(0.6)
    cons = prod - 2.5 + 6 * math.sin((time.time() - START_TIME) / 47) + RNG.jitter(0.8)
    return max(5.0, prod), max(4.0, cons)


POWER_SOURCES = {
    "steam": 0.55,
    "solar": 0.33,
    "nuclear": 0.12,
}


def render_metrics() -> str:
    """Produce the graftorio3-compatible exposition."""
    # graftorio3 emits millisecond timestamps (OpenMetrics-style explicit
    # timestamps). They must be absolute wall-clock values, not relative ones,
    # or Prometheus rejects the samples as "too old / too far in the future".
    now_ms = int(time.time() * 1000)
    lines = ["# CircuitDeck mock graftorio3 metrics", ""]

    # --- counters: totals since game start ---
    def counter(name: str, item: str, value: float) -> list[str]:
        return [
            f"# HELP {name} Total count.",
            f"# TYPE {name} counter",
            f'{name}{{item="{item}"}} {value:.2f} {now_ms}',
        ]

    body: list[str] = []
    for item, value in ITEMS_PRODUCED.items():
        body.extend(counter("graftorio_factorio_item_produced_total", item, value))
    for item, value in ITEMS_CONSUMED.items():
        body.extend(counter("graftorio_factorio_item_consumed_total", item, value))
    for fluid, value in FLUIDS_PRODUCED.items():
        body.extend(counter("graftorio_factorio_fluid_produced_total", fluid, value))
    for fluid, value in FLUIDS_CONSUMED.items():
        body.extend(counter("graftorio_factorio_fluid_consumed_total", fluid, value))

    # research packages (label "name" like graftorio)
    body.extend(
        [
            "# HELP graftorio_factorio_research_packages_built_total Total research packages crafted.",
            "# TYPE graftorio_factorio_research_packages_built_total counter",
            f'graftorio_factorio_research_packages_built_total{{name="total"}} {state["research_pkgs"]:.2f} {now_ms}',
        ]
    )

    # --- power: production/consumption in joules per tick sampled per source ---
    prod_mw, cons_mw = power_mw()
    body.extend(
        [
            "# HELP graftorio_factorio_power_production Power produced (joules).",
            "# TYPE graftorio_factorio_power_production gauge",
            f'graftorio_factorio_power_production{{type="overall"}} {prod_mw * 1_000_000 / 60:.2f} {now_ms}',
            "# HELP graftorio_factorio_power_consumption Power consumed (joules).",
            "# TYPE graftorio_factorio_power_consumption gauge",
            f'graftorio_factorio_power_consumption{{type="overall"}} {cons_mw * 1_000_000 / 60:.2f} {now_ms}',
        ]
    )
    for source, share in POWER_SOURCES.items():
        body.append(
            f'graftorio_factorio_power_production{{type="by_source",source="{source}"}} '
            f"{prod_mw * share * 1_000_000 / 60:.2f} {now_ms}"
        )

    body.extend(
        [
            "# HELP graftorio_factorio_accumulator_charge Accumulator charge (0-1).",
            "# TYPE graftorio_factorio_accumulator_charge gauge",
            f'graftorio_factorio_accumulator_charge{{type="charge"}} {state["accu"]:.4f} {now_ms}',
            "# HELP graftorio_factorio_satisfaction Power satisfaction (0-1).",
            "# TYPE graftorio_factorio_satisfaction gauge",
            f'graftorio_factorio_satisfaction{{type="satisfaction"}} {state["satisfaction"]:.4f} {now_ms}',
        ]
    )

    # --- game state gauges ---
    body.extend(
        [
            "# HELP graftorio_factorio_game_tick Current game tick.",
            "# TYPE graftorio_factorio_game_tick gauge",
            f'graftorio_factorio_game_tick{{type="tick"}} {state["tick"]} {now_ms}',
            "# HELP graftorio_factorio_evolution_factor Enemy evolution.",
            "# TYPE graftorio_factorio_evolution_factor gauge",
            f'graftorio_factorio_evolution_factor{{type="time"}} {state["evolution"]:.4f} {now_ms}',
            f'graftorio_factorio_evolution_factor{{type="pollution"}} {state["evolution"] * 0.8:.4f} {now_ms}',
            f'graftorio_factorio_evolution_factor{{type="spawner"}} {state["evolution"] * 0.9:.4f} {now_ms}',
            "# HELP graftorio_factorio_pollution Overall pollution.",
            "# TYPE graftorio_factorio_pollution gauge",
            f'graftorio_factorio_pollution{{type="overall"}} {state["pollution"]:.2f} {now_ms}',
            "# HELP graftorio_factorio_research_completed_total Technologies researched.",
            "# TYPE graftorio_factorio_research_completed_total counter",
            f'graftorio_factorio_research_completed_total{{type="count"}} {state["completed"]} {now_ms}',
            "# HELP graftorio_factorio_research_in_progress Current research.",
            "# TYPE graftorio_factorio_research_in_progress gauge",
            f'graftorio_factorio_research_in_progress{{tech="{RESEARCH_STEPS[state["research_index"]]}"}} '
            f'{state["research_progress"]:.4f} {now_ms}',
            "# HELP graftorio_factorio_research_progress Research progress (0-1).",
            "# TYPE graftorio_factorio_research_progress gauge",
            f'graftorio_factorio_research_progress{{type="progress"}} {state["research_progress"]:.4f} {now_ms}',
            "# HELP graftorio_factorio_research_queue_size Research queue length.",
            "# TYPE graftorio_factorio_research_queue_size gauge",
            f'graftorio_factorio_research_queue_size{{type="size"}} {len(RESEARCH_STEPS) - state["research_index"]} {now_ms}',
        ]
    )

    # --- machine / robot / train gauges ---
    for machine, count in MACHINES.items():
        wobble = count + int(RNG.jitter(2))
        body.append(
            f'graftorio_factorio_machine_count{{machine="{machine}"}} {max(0, wobble)} {now_ms}'
        )
    for robot, count in ROBOTS.items():
        wobble = count + int(RNG.jitter(8))
        body.append(f'graftorio_factorio_robot_count{{robot="{robot}"}} {max(0, wobble)} {now_ms}')
    for tstate, count in TRAIN_STATES.items():
        body.append(f'graftorio_factorio_train_count{{state="{tstate}"}} {count} {now_ms}')
    for station, items in TRAIN_STATIONS.items():
        wobble = max(0, int(items * (1 + RNG.jitter(0.1))))
        body.append(
            f'graftorio_factorio_train_station_waiting{{station="{station}"}} {wobble} {now_ms}'
        )
    body.extend(
        [
            "# HELP graftorio_factorio_train_station_count Train stations.",
            "# TYPE graftorio_factorio_train_station_count gauge",
            f'graftorio_factorio_train_station_count{{type="count"}} {len(TRAIN_STATIONS)} {now_ms}',
        ]
    )

    # --- chest contents ---
    for item, count in CHEST_ITEMS.items():
        wobble = max(0, int(count * (1 + RNG.jitter(0.06))))
        body.append(f'graftorio_factorio_chest_contents{{item="{item}"}} {wobble} {now_ms}')

    # --- space platforms ---
    for platform, info in PLATFORMS.items():
        speed = info["speed"] + RNG.jitter(6)
        fuel = min(1.0, max(0.05, info["fuel"] + RNG.jitter(0.03)))
        body.append(f'graftorio_factorio_platform_speed{{platform="{platform}"}} {speed:.1f} {now_ms}')
        body.append(f'graftorio_factorio_platform_fuel{{platform="{platform}"}} {fuel:.4f} {now_ms}')

    return "\n".join(body) + "\n"


class MetricsHandler(BaseHTTPRequestHandler):
    ticks_per_scrape = 900  # overridden in main() from --interval

    def do_GET(self) -> None:  # noqa: N802 (http.server API)
        if self.path not in ("/metrics", "/metrics.sh"):
            self.send_response(404)
            self.end_headers()
            return
        advance(MetricsHandler.ticks_per_scrape)
        payload = render_metrics().encode()
        self.send_response(200)
        self.send_header("Content-Type", "text/plain; version=0.0.4; charset=utf-8")
        self.send_header("Content-Length", str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)

    def log_message(self, fmt: str, *args: object) -> None:
        print(f"mock-metrics: {fmt % args}")


def main() -> None:
    parser = argparse.ArgumentParser(description="graftorio3-compatible mock metrics server")
    parser.add_argument("--port", type=int, default=9105)
    parser.add_argument(
        "--interval", type=int, default=15, help="seconds of game time added per scrape"
    )
    args = parser.parse_args()
    MetricsHandler.ticks_per_scrape = args.interval * 60
    server = ThreadingHTTPServer(("0.0.0.0", args.port), MetricsHandler)
    print(f"mock-metrics: serving graftorio3-compatible metrics on :{args.port}/metrics")
    server.serve_forever()


if __name__ == "__main__":
    main()