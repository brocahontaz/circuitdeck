/**
 * Typed API client for the CircuitDeck backend.
 *
 * The browser never talks to Prometheus directly; every request goes to
 * the Rust API, which owns all PromQL.
 */

export type TimeRange = "1h" | "6h" | "24h" | "7d";

export const TIME_RANGES: { value: TimeRange; label: string }[] = [
  { value: "1h", label: "1 hour" },
  { value: "6h", label: "6 hours" },
  { value: "24h", label: "24 hours" },
  { value: "7d", label: "7 days" },
];

export interface Reading {
  label: string;
  value: number;
}

export interface HealthReport {
  prometheus_reachable: boolean;
  factorio_metrics_job_present: boolean;
  game_ticks_played: number | null;
}

export interface Overview {
  health: HealthReport;
  evolution_factor: number | null;
  pollution: number | null;
  power_production_mw: number | null;
  power_consumption_mw: number | null;
  accumulator_charge_pct: number | null;
  satisfaction_pct: number | null;
  top_produced: Reading[];
  top_consumed: Reading[];
  current_research: string | null;
  research_progress_pct: number | null;
  platform_count: number | null;
  machine_counts: Reading[];
  robot_counts: Reading[];
  train_counts: Reading[];
}

export interface SeriesPoint {
  t: number;
  v: number | null;
}

export interface Series {
  name: string;
  points: SeriesPoint[];
}

export interface ProductionReport {
  range: string;
  top_produced: Reading[];
  top_consumed: Reading[];
  produced_series: Series[];
  consumed_series: Series[];
  research_packages_per_minute: number | null;
}

export interface PowerReport {
  range: string;
  production_mw: number | null;
  consumption_mw: number | null;
  production_by_source: Reading[];
  production_series: Series[];
  consumption_series: Series[];
  accumulator_charge_pct: number | null;
  satisfaction_pct: number | null;
}

export interface MachineStatus {
  machine: string;
  active: number | null;
  total: number | null;
}

export interface FactoryReport {
  machines: MachineStatus[];
}

export interface LogisticsReport {
  robots: Reading[];
  networks: number | null;
  top_stored_items: Reading[];
}

export interface TrainsReport {
  trains_by_state: Reading[];
  station_count: number | null;
  stations_waiting: Reading[];
}

export interface ResearchReport {
  current_research: string | null;
  progress_pct: number | null;
  completed_count: number | null;
  queue_size: number | null;
}

export interface PlatformStatus {
  platform: string;
  speed_kmph: number | null;
  fuel_pct: number | null;
}

export interface PlatformReport {
  count: number | null;
  platforms: PlatformStatus[];
}

export class ApiError extends Error {
  readonly code: string;
  readonly status: number;

  constructor(code: string, message: string, status: number) {
    super(message);
    this.code = code;
    this.status = status;
  }
}

const API_BASE: string =
  (import.meta.env?.VITE_API_BASE_URL as string | undefined) ?? "/api";

async function request<T>(path: string, params?: URLSearchParams): Promise<T> {
  const qs = params?.toString();
  const url = `${API_BASE}${path}${qs ? `?${qs}` : ""}`;
  let resp: Response;
  try {
    resp = await fetch(url, { headers: { Accept: "application/json" } });
  } catch {
    throw new ApiError(
      "network_error",
      `Cannot reach dashboard API at ${url}`,
      0,
    );
  }
  if (!resp.ok) {
    let code = "http_error";
    let message = `API returned ${resp.status}`;
    try {
      const body = (await resp.json()) as {
        error?: { code?: string; message?: string };
      };
      if (body?.error?.code) code = body.error.code;
      if (body?.error?.message) message = body.error.message;
    } catch {
      // non-JSON error body; keep defaults
    }
    throw new ApiError(code, message, resp.status);
  }
  return (await resp.json()) as T;
}

export function fetchHealth(): Promise<HealthReport> {
  return request<HealthReport>("/health");
}

export function fetchOverview(range: TimeRange): Promise<Overview> {
  return request<Overview>("/overview", new URLSearchParams({ range }));
}

export function fetchProduction(range: TimeRange): Promise<ProductionReport> {
  return request<ProductionReport>(
    "/production",
    new URLSearchParams({ range }),
  );
}

export function fetchPower(range: TimeRange): Promise<PowerReport> {
  return request<PowerReport>("/power", new URLSearchParams({ range }));
}

export function fetchFactory(range: TimeRange): Promise<FactoryReport> {
  return request<FactoryReport>("/factory", new URLSearchParams({ range }));
}

export function fetchLogistics(range: TimeRange): Promise<LogisticsReport> {
  return request<LogisticsReport>("/logistics", new URLSearchParams({ range }));
}

export function fetchTrains(range: TimeRange): Promise<TrainsReport> {
  return request<TrainsReport>("/trains", new URLSearchParams({ range }));
}

export function fetchResearch(range: TimeRange): Promise<ResearchReport> {
  return request<ResearchReport>("/research", new URLSearchParams({ range }));
}

export function fetchPlatforms(range: TimeRange): Promise<PlatformReport> {
  return request<PlatformReport>("/platforms", new URLSearchParams({ range }));
}
