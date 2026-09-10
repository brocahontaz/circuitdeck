<script lang="ts">
  import type { Overview } from '../lib/api';
  import Panel from '../lib/Panel.svelte';
  import Stat from '../lib/Stat.svelte';
  import BarList from '../lib/BarList.svelte';
  import StateMessage from '../lib/StateMessage.svelte';
  import { fmtPct } from '../lib/format';

  let { data, loading, error }: { data: Overview | null; loading: boolean; error: string | null } =
    $props();

  const degraded = $derived(data !== null && !data.health.prometheus_reachable);
  const empty = $derived(!loading && data === null && error === null);
  const powerTone = $derived.by(() => {
    if (!data) return 'normal' as const;
    const cons = data.power_consumption_mw;
    const prod = data.power_production_mw;
    if (cons === null || prod === null) return 'normal' as const;
    return cons > prod ? ('danger' as const) : ('good' as const);
  });
</script>

{#if loading && data === null}
  <StateMessage state="loading" />
{:else if error}
  <StateMessage state="error" message={error} />
{:else if empty}
  <StateMessage state="error" message="No overview data available." />
{:else if data}
  {#if degraded}
    <StateMessage state="degraded" message="Prometheus unreachable — showing last known layout." />
  {/if}

  <div class="grid health-row">
    <Panel title="Telemetry" tone={data.health.prometheus_reachable && data.health.factorio_metrics_job_present ? 'good' : 'warning'}>
      <div class="health-lines">
        <span class:ok={data.health.prometheus_reachable} class:bad={!data.health.prometheus_reachable}>
          Prometheus {data.health.prometheus_reachable ? 'reachable' : 'unreachable'}
        </span>
        <span class:ok={data.health.factorio_metrics_job_present} class:bad={!data.health.factorio_metrics_job_present}>
          Factorio metrics {data.health.factorio_metrics_job_present ? 'flowing' : 'missing'}
        </span>
        <span class="dim">Ticks played: {data.health.game_ticks_played ?? '—'}</span>
      </div>
    </Panel>
    <Panel title="Current research">
      {#if data.current_research}
        <p class="research-name">{data.current_research.replace(/-/g, ' ')}</p>
        <div class="progress">
          <span class="progress-fill" style="width: {data.research_progress_pct ?? 0}%"></span>
        </div>
        <span class="dim">{fmtPct(data.research_progress_pct)} complete</span>
      {:else}
        <p class="dim">No research in progress</p>
      {/if}
    </Panel>
  </div>

  <div class="grid cards">
    <Stat label="Power production" value={data.power_production_mw} suffix="MW" />
    <Stat label="Power consumption" value={data.power_consumption_mw} suffix="MW" tone={powerTone} />
    <Stat
      label="Accumulator charge"
      value={data.accumulator_charge_pct}
      suffix="%"
      tone={(data.accumulator_charge_pct ?? 100) < 20 ? 'warning' : 'normal'}
      digits={0}
    />
    <Stat label="Satisfaction" value={data.satisfaction_pct} suffix="%" digits={0} />
    <Stat label="Pollution" value={data.pollution} digits={0} />
    <Stat label="Evolution factor" value={data.evolution_factor} digits={2} />
    <Stat label="Platforms" value={data.platform_count} digits={0} />
  </div>

  <div class="grid two">
    <Panel title="Top produced" subtitle="items/min">
      <BarList readings={data.top_produced} suffix="/min" emptyText="No production recorded." />
    </Panel>
    <Panel title="Top consumed" subtitle="items/min">
      <BarList readings={data.top_consumed} suffix="/min" emptyText="No consumption recorded." />
    </Panel>
  </div>

  <div class="grid three">
    <Panel title="Machines">
      <BarList readings={data.machine_counts} digits={0} emptyText="No machine telemetry." />
    </Panel>
    <Panel title="Robots">
      <BarList readings={data.robot_counts} digits={0} emptyText="No robot telemetry." />
    </Panel>
    <Panel title="Trains">
      <BarList readings={data.train_counts} digits={0} emptyText="No train telemetry." />
    </Panel>
  </div>
{/if}

<style>
  .grid {
    display: grid;
    gap: 14px;
    margin-bottom: 14px;
  }

  .health-row {
    grid-template-columns: 1fr 1fr;
  }

  .cards {
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  }

  .two {
    grid-template-columns: 1fr 1fr;
  }

  .three {
    grid-template-columns: repeat(3, 1fr);
  }

  @media (max-width: 720px) {
    .health-row,
    .two,
    .three {
      grid-template-columns: 1fr;
    }
  }

  .health-lines {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 13px;
  }

  .ok {
    color: var(--green);
  }

  .bad {
    color: var(--red);
  }

  .dim {
    color: var(--text-dim);
    font-size: 12px;
  }

  .research-name {
    margin: 0 0 6px;
    color: var(--text-bright);
    text-transform: capitalize;
    font-weight: 600;
  }

  .progress {
    height: 10px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 3px;
    overflow: hidden;
    margin-bottom: 4px;
  }

  .progress-fill {
    display: block;
    height: 100%;
    background: linear-gradient(90deg, var(--accent-dim), var(--accent));
  }
</style>