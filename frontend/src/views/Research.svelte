<script lang="ts">
  import type { ResearchReport } from '../lib/api';
  import Panel from '../lib/Panel.svelte';
  import Stat from '../lib/Stat.svelte';
  import StateMessage from '../lib/StateMessage.svelte';
  import { fmtPct } from '../lib/format';

  let { data, loading, error } = $props<{ data: ResearchReport | null; loading: boolean; error: string | null }>();

  const empty = $derived(!loading && data === null);
</script>

{#if loading && data === null}
  <StateMessage state="loading" />
{:else if error}
  <StateMessage state="error" message={error} />
{:else if empty}
  <StateMessage state="error" message="No research data available." />
{:else if data}
  <div class="grid cards">
    <Stat label="Completed" value={data.completed_count} digits={0} />
    <Stat label="Queue size" value={data.queue_size} digits={0} />
  </div>

  <Panel title="Current research">
    {#if data.current_research}
      <p class="research-name">{data.current_research.replace(/-/g, ' ')}</p>
      <div class="progress">
        <span class="progress-fill" style="width: {data.progress_pct ?? 0}%"></span>
      </div>
      <span class="mono">{fmtPct(data.progress_pct)} complete</span>
    {:else}
      <StateMessage state="empty" message="No research in progress — all tech complete?" />
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

  .research-name {
    margin: 0 0 6px;
    color: var(--text-bright);
    text-transform: capitalize;
    font-weight: 600;
  }

  .progress {
    height: 12px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 3px;
    overflow: hidden;
    margin-bottom: 6px;
  }

  .progress-fill {
    display: block;
    height: 100%;
    background: linear-gradient(90deg, var(--accent-dim), var(--accent));
  }

  .mono {
    font-family: var(--font-mono);
    color: var(--text-dim);
    font-size: 12px;
  }
</style>