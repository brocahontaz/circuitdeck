<script lang="ts">
  import type { PlatformReport } from '../lib/api';
  import Panel from '../lib/Panel.svelte';
  import Stat from '../lib/Stat.svelte';
  import StateMessage from '../lib/StateMessage.svelte';
  import { fmtInt } from '../lib/format';

  let { data, loading, error } = $props<{ data: PlatformReport | null; loading: boolean; error: string | null }>();

  const empty = $derived(!loading && data === null);
  const none = $derived(data !== null && (data.count === null || data.count === 0));
</script>

{#if loading && data === null}
  <StateMessage state="loading" />
{:else if error}
  <StateMessage state="error" message={error} />
{:else if empty}
  <StateMessage state="error" message="No platform data available." />
{:else if data}
  <div class="grid cards">
    <Stat label="Platforms in flight" value={data.count} digits={0} />
  </div>

  {#if none}
    <Panel title="Space platforms">
      <StateMessage state="empty" message="No platforms are currently active (Space Age required)." />
    </Panel>
  {:else}
    <Panel title="Platform status">
      <div class="table-scroll">
      <table>
        <thead>
          <tr>
            <th>Platform</th>
            <th class="num">Speed (km/h)</th>
            <th class="num">Fuel</th>
          </tr>
        </thead>
        <tbody>
          {#each data.platforms as p (p.platform)}
            <tr>
              <td>{p.platform}</td>
              <td class="num mono">{fmtInt(p.speed_kmph)}</td>
              <td class="num mono">{p.fuel_pct === null ? '—' : `${p.fuel_pct.toFixed(0)}%`}</td>
            </tr>
          {/each}
        </tbody>
      </table>
      </div>
    </Panel>
  {/if}
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

  .table-scroll {
    overflow-x: auto;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
  }

  th {
    text-align: left;
    color: var(--text-dim);
    text-transform: uppercase;
    font-size: 11px;
    letter-spacing: 0.06em;
    border-bottom: 1px solid var(--border);
    padding: 6px 8px;
  }

  td {
    padding: 6px 8px;
    border-bottom: 1px solid var(--border);
    color: var(--text);
  }

  tbody tr:last-child td {
    border-bottom: none;
  }

  .num {
    text-align: right;
  }

  .mono {
    font-family: var(--font-mono);
  }
</style>