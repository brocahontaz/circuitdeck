<script lang="ts">
  /**
   * Chart: responsive ECharts canvas bound to an option object.
   */
  import { onMount } from 'svelte';
  import { echarts, baseOption, type EChartsCoreOption } from './echarts';

  let {
    option,
    height = '280px',
    ariaLabel = 'chart',
  }: {
    option: EChartsCoreOption;
    height?: string;
    ariaLabel?: string;
  } = $props();

  let el: globalThis.HTMLDivElement | undefined = $state();
  let chart: echarts.ECharts | null = null;

  onMount(() => {
    if (!el) return;
    chart = echarts.init(el, undefined, { renderer: 'canvas' });
    const resize = new ResizeObserver(() => chart?.resize());
    resize.observe(el);
    return () => {
      resize.disconnect();
      chart?.dispose();
      chart = null;
    };
  });

  $effect(() => {
    // Re-apply when the option object changes.
    const opt = $state.snapshot(option) as EChartsCoreOption;
    if (chart) {
      chart.setOption({ ...baseOption(), ...opt }, { notMerge: true });
    }
  });
</script>

<div
  bind:this={el}
  class="chart"
  style="height: {height}"
  role="img"
  aria-label={ariaLabel}
></div>

<style>
  .chart {
    width: 100%;
    min-width: 0;
  }
</style>