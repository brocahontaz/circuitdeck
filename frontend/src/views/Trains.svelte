<script lang="ts">
  import type { TrainsReport } from '../lib/api';
  import Panel from '../lib/Panel.svelte';
  import Stat from '../lib/Stat.svelte';
  import BarList from '../lib/BarList.svelte';
  import StateMessage from '../lib/StateMessage.svelte';

  let { data, loading, error } = $props<{ data: TrainsReport | null; loading: boolean; error: string | null }>();

  const empty = $derived(!loading && data === null);
</script>

{#if loading && data === null}
  <StateMessage state="loading" />
{:else if error}
  <StateMessage state="error" message={error} />
{:else if empty}
  <StateMessage state="error" message="No train data available." />
{:else if data}
  <div class="grid cards">
    <Stat label="Stations" value={data.station_count} digits={0} />
    {#each data.trains_by_state as row (row.label)}
      <Stat label={row.label.replace(/-/g, ' ')} value={row.value} digits={0} />
    {/each}
  </div>

  <Panel title="Stations with waiting cargo" subtitle="items">
    {#if data.stations_waiting.length === 0}
      <StateMessage state="empty" message="No station is waiting on cargo." />
    {:else}
      <BarList readings={data.stations_waiting} digits={0} emptyText="All clear." />
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
</style>