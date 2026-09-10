<script lang="ts">
  import type { PowerReport } from '../lib/api';
  import Panel from '../lib/Panel.svelte';
  import Stat from '../lib/Stat.svelte';
  import BarList from '../lib/BarList.svelte';
  import Chart from '../lib/Chart.svelte';
  import StateMessage from '../lib/StateMessage.svelte';
  import type { EChartsCoreOption } from '../lib/echarts';

  let { data, loading, error } = $props<{ data: PowerReport | null; loading: boolean; error: string | null }>();

  const powerTone = $derived.by(() => {
    if (!data) return 'normal' as const;
    if (data.consumption_mw === null || data.production_mw === null) return 'normal' as const;
    return data.consumption_mw > data.production_mw ? ('danger' as const) : ('good' as const);
  });

  function powerSeriesOption(
    series: PowerReport['production_series'],
    name: string,
    color: string,
  ): EChartsCoreOption {
    return {
      tooltip: { trigger: 'axis', valueFormatter: (v: unknown) => `${Number(v ?? 0).toFixed(1)} MW` },
      grid: { left: 52, right: 12, top: 16, bottom: 44 },
      dataZoom: [
        { type: 'slider', bottom: 0, height: 16, labelFormatter: () => '' },
        { type: 'inside' },
      ],
      xAxis: {
        type: 'time',
        axisLabel: { color: '#93928c' },
        axisLine: { lineStyle: { color: '#3d444e' } },
      },
      yAxis: {
        type: 'value',
        name: 'MW',
        nameTextStyle: { color: '#93928c' },
        axisLabel: { color: '#93928c' },
        splitLine: { lineStyle: { color: '#2d323a' } },
      },
      series: series.map((s) => ({
        name: s.name.replace(/-/g, ' '),
        type: 'line',
        showSymbol: false,
        color,
        data: s.points.map((p) => [p.t * 1000, p.v]),
        connectNulls: false,
      })),
      ...(series.length > 0 ? { legend: { bottom: 0, textStyle: { color: '#93928c' } } } : {}),
      title: { text: name, left: 'center', textStyle: { color: '#93928c', fontSize: 12 } },
    };
  }

  const empty = $derived(!loading && data === null);
</script>

{#if loading && data === null}
  <StateMessage state="loading" />
{:else if error}
  <StateMessage state="error" message={error} />
{:else if empty}
  <StateMessage state="error" message="No power data available." />
{:else if data}
  <div class="grid cards">
    <Stat label="Production" value={data.production_mw} suffix="MW" />
    <Stat label="Consumption" value={data.consumption_mw} suffix="MW" tone={powerTone} />
    <Stat
      label="Accumulator"
      value={data.accumulator_charge_pct}
      suffix="%"
      digits={0}
      tone={(data.accumulator_charge_pct ?? 100) < 20 ? 'warning' : 'normal'}
    />
    <Stat label="Satisfaction" value={data.satisfaction_pct} suffix="%" digits={0} />
  </div>

  <div class="grid two">
    <Panel title="Production by source" subtitle="MW">
      <BarList readings={data.production_by_source} suffix=" MW" emptyText="No generators reporting." />
    </Panel>
  </div>

  <Panel title="Power history" subtitle="MW">
    {#if data.production_series.length === 0}
      <StateMessage state="empty" message="No power history in this range." />
    {:else}
      <Chart option={powerSeriesOption(data.production_series, 'Production (MW)', '#7dc242')} height="260px" ariaLabel="Power production history" />
      <div class="spacer"></div>
      <Chart option={powerSeriesOption(data.consumption_series, 'Consumption (MW)', '#e04f3f')} height="260px" ariaLabel="Power consumption history" />
    {/if}
  </Panel>
{/if}

<style>
  .grid {
    display: grid;
    gap: 14px;
    margin-bottom: 14px;
  }

  .cards {
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  }

  .two {
    grid-template-columns: 1fr;
  }

  .spacer {
    height: 14px;
  }
</style>