<script lang="ts">
  import type { LogisticsReport } from '../lib/api';
  import Panel from '../lib/Panel.svelte';
  import BarList from '../lib/BarList.svelte';
  import Stat from '../lib/Stat.svelte';
  import StateMessage from '../lib/StateMessage.svelte';

  let { data, loading, error } = $props<{ data: LogisticsReport | null; loading: boolean; error: string | null }>();

  const empty = $derived(!loading && data === null);
</script>

{#if loading && data === null}
  <StateMessage state="loading" />
{:else if error}
  <StateMessage state="error" message={error} />
{:else if empty}
  <StateMessage state="error" message="No logistics data available." />
{:else if data}
  <div class="grid cards">
    <Stat label="Logistics networks" value={data.networks} digits={0} />
    {#each data.robots as robot (robot.label)}
      <Stat label={robot.label.replace(/-/g, ' ')} value={robot.value} digits={0} />
    {/each}
  </div>

  <Panel title="Top stored items" subtitle="logistics chests, top 10">
    {#if data.top_stored_items.length === 0}
      <StateMessage state="empty" message="No chest telemetry in this range." />
    {:else}
      <BarList readings={data.top_stored_items} digits={0} emptyText="No stored items." />
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