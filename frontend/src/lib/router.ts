/**
 * Tiny hash-based router for the dashboard shell (no dependency).
 */

export type ViewId =
  | "overview"
  | "production"
  | "power"
  | "factory"
  | "logistics"
  | "trains"
  | "research"
  | "platforms";

export interface ViewMeta {
  id: ViewId;
  label: string;
  icon: string;
}

export const VIEWS: ViewMeta[] = [
  { id: "overview", label: "Overview", icon: "⌂" },
  { id: "production", label: "Production", icon: "⚙" },
  { id: "power", label: "Power", icon: "⚡" },
  { id: "factory", label: "Factory", icon: "▤" },
  { id: "logistics", label: "Logistics", icon: "✈" },
  { id: "trains", label: "Trains", icon: "⇄" },
  { id: "research", label: "Research", icon: "⚗" },
  { id: "platforms", label: "Platforms", icon: "◈" },
];

/** Parses `#/view` from location.hash, defaulting to overview. */
export function currentView(hash: string): ViewId {
  const id = hash.replace(/^#\/?/, "") as ViewId;
  return VIEWS.some((v) => v.id === id) ? id : "overview";
}
