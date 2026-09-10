<script lang="ts">
  import type { FactoryReport } from '../lib/api';
  import Panel from '../lib/Panel.svelte';
  import StateMessage from '../lib/StateMessage.svelte';
  import { fmtInt } from '../lib/format';

  let { data, loading, error } = $props<{ data: FactoryReport | null; loading: boolean; error: string | null }>();

  const rows = $derived(data?.machines ?? []);
  const empty = $derived(!loading && data === null);
</script>

{#if loading && data === null}
  <StateMessage state="loading" />
{:else if error}
  <StateMessage state="error" message={error} />
{:else if empty}
  <StateMessage state="error" message="No factory data available." />
{:else if data}
  <Panel title="Machine census" subtitle="active machines by type">
    {#if rows.length === 0}
      <StateMessage state="empty" message="No machine telemetry in this range." />
    {:else}
      <div class="table-scroll">
      <table>
        <thead>
          <tr>
            <th>Machine</th>
            <th class="num">Active</th>
          </tr>
        </thead>
        <tbody>
          {#each rows as row (row.machine)}
            <tr>
              <td>{row.machine.replace(/-/g, ' ')}</td>
              <td class="num mono">{fmtInt(row.active)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
      </div>
    {/if}
  </Panel>
{/if}

<style>
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
    text-transform: capitalize;
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