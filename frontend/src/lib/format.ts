/**
 * Formatting helpers for dashboard values.
 */

export function fmtNumber(
  value: number | null | undefined,
  digits = 1,
): string {
  if (value === null || value === undefined || Number.isNaN(value)) return "—";
  if (Math.abs(value) >= 1_000_000)
    return `${(value / 1_000_000).toFixed(digits)}M`;
  if (Math.abs(value) >= 10_000) return `${(value / 1_000).toFixed(digits)}k`;
  return value.toFixed(digits);
}

export function fmtInt(value: number | null | undefined): string {
  if (value === null || value === undefined || Number.isNaN(value)) return "—";
  return Math.round(value).toLocaleString("en-US");
}

export function fmtPct(value: number | null | undefined, digits = 0): string {
  if (value === null || value === undefined || Number.isNaN(value)) return "—";
  return `${value.toFixed(digits)}%`;
}

export function fmtPerMin(
  value: number | null | undefined,
  digits = 1,
): string {
  if (value === null || value === undefined || Number.isNaN(value)) return "—";
  return `${value.toFixed(digits)}/min`;
}

export function fmtTicks(ticks: number | null | undefined): string {
  if (ticks === null || ticks === undefined || Number.isNaN(ticks)) return "—";
  // Factorio runs at 60 ticks/s.
  const seconds = ticks / 60;
  const days = Math.floor(seconds / 51840); // in-game day = 864s / 60 = 51840 ticks? keep simple: real seconds
  const hours = Math.floor((seconds % 51840) / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  if (days > 0) return `${days}d ${hours}h ${minutes}m`;
  if (hours > 0) return `${hours}h ${minutes}m`;
  return `${minutes}m`;
}
