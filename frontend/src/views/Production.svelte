<script lang="ts">
  import type { ProductionReport } from '../lib/api';
  import Panel from '../lib/Panel.svelte';
  import BarList from '../lib/BarList.svelte';
  import Chart from '../lib/Chart.svelte';
  import StateMessage from '../lib/StateMessage.svelte';
  import type { EChartsCoreOption } from '../lib/echarts';
  import { fmtPerMin } from '../lib/format';

  let { data, loading, error } = $props<{ data: ProductionReport | null; loading: boolean; error: string | null }>();

  function seriesOption(
    series: ProductionReport['produced_series'],
    title: string,
  ): EChartsCoreOption {
    const legendNames = series.map((s) => s.name.replace(/-/g, ' '));
    return {
      title: { text: title, left: 'center', textStyle: { color: '#93928c', fontSize: 12 } },
      tooltip: { trigger: 'axis', valueFormatter: (v: unknown) => `${Number(v ?? 0).toFixed(1)}/min` },
      legend: { data: legendNames, bottom: 0, textStyle: { color: '#93928c' }, type: 'scroll' },
      grid: { left: 48, right: 12, top: 32, bottom: 64 },
      dataZoom: [
        { type: 'slider', bottom: 6, height: 18, labelFormatter: () => '' },
        { type: 'inside' },
      ],
      xAxis: {
        type: 'time',
        axisLabel: { color: '#93928c' },
        axisLine: { lineStyle: { color: '#3d444e' } },
      },
      yAxis: {
        type: 'value',
        name: '/min',
        nameTextStyle: { color: '#93928c' },
        axisLabel: { color: '#93928c' },
        splitLine: { lineStyle: { color: '#2d323a' } },
      },
      series: series.map((s) => ({
        name: s.name.replace(/-/g, ' '),
        type: 'line',
        showSymbol: false,
        data: s.points.map((p) => [p.t * 1000, p.v]),
        connectNulls: false,
      })),
    };
  }

  const empty = $derived(!loading && data === null);
</script>

{#if loading && data === null}
  <StateMessage state="loading" />
{:else if error}
  <StateMessage state="error" message={error} />
{:else if empty}
  <StateMessage state="error" message="No production data available." />
{:else if data}
  <div class="grid two">
    <Panel title="Top produced" subtitle="items/min, top 8">
      <BarList readings={data.top_produced} suffix="/min" emptyText="No production in range." />
    </Panel>
    <Panel title="Top consumed" subtitle="items/min, top 8">
      <BarList readings={data.top_consumed} suffix="/min" emptyText="No consumption in range." />
    </Panel>
  </div>

  <Panel title="Production over time" subtitle="items/min">
    {#if data.produced_series.length === 0}
      <StateMessage state="empty" message="No production history in this range." />
    {:else}
      <Chart option={seriesOption(data.produced_series, 'Produced items/min')} height="320px" ariaLabel="Produced items over time" />
    {/if}
  </Panel>

  <Panel title="Consumption over time" subtitle="items/min">
    {#if data.consumed_series.length === 0}
      <StateMessage state="empty" message="No consumption history in this range." />
    {:else}
      <Chart option={seriesOption(data.consumed_series, 'Consumed items/min')} height="320px" ariaLabel="Consumed items over time" />
    {/if}
  </Panel>

  <Panel title="Science">
    <p class="mono">
      Research packages: {fmtPerMin(data.research_packages_per_minute)}
    </p>
  </Panel>
{/if}

<style>
  .grid {
    display: grid;
    gap: 14px;
    margin-bottom: 14px;
  }

  .two {
    grid-template-columns: 1fr 1fr;
  }

  @media (max-width: 720px) {
    .two {
      grid-template-columns: 1fr;
    }
  }

  .mono {
    font-family: var(--font-mono);
    color: var(--text);
    margin: 0;
  }
</style>