/**
 * Small wrapper around Apache ECharts for Svelte 5.
 *
 * ECharts is imported lazily so the bundle stays tree-shaken per chart type.
 */
import * as echarts from "echarts/core";
import { LineChart, BarChart, PieChart } from "echarts/charts";
import {
  GridComponent,
  TooltipComponent,
  LegendComponent,
  TitleComponent,
  DataZoomComponent,
} from "echarts/components";
import { CanvasRenderer } from "echarts/renderers";
import type { EChartsCoreOption } from "echarts/core";

echarts.use([
  LineChart,
  BarChart,
  PieChart,
  GridComponent,
  TooltipComponent,
  LegendComponent,
  TitleComponent,
  DataZoomComponent,
  CanvasRenderer,
]);

export { echarts };
export type { EChartsCoreOption };

/** Shared palette drawn from the dashboard theme. */
export const PALETTE = [
  "#ff9a1f",
  "#4fa8e0",
  "#7dc242",
  "#e04f3f",
  "#ffd23f",
  "#a06fe0",
  "#2ec4b6",
  "#e07a5f",
];

/** Base ECharts option shared by all charts (dark industrial look). */
export function baseOption(): EChartsCoreOption {
  return {
    backgroundColor: "transparent",
    color: PALETTE,
    textStyle: {
      color: "#93928c",
      fontFamily: "Inter, system-ui, sans-serif",
    },
    animationDuration: 300,
  };
}
