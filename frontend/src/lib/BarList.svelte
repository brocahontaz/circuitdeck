<script lang="ts">
  /**
   * BarList: labeled horizontal bars for rankings.
   */
  import { fmtNumber } from './format';
  import type { Reading } from './api';

  let {
    readings,
    digits = 1,
    suffix = '',
    emptyText = 'No data',
  }: {
    readings: Reading[];
    digits?: number;
    suffix?: string;
    emptyText?: string;
  } = $props();

  const max = $derived(
    readings.length > 0 ? Math.max(...readings.map((r) => r.value), 0.0001) : 1,
  );
</script>

{#if readings.length === 0}
  <p class="empty">{emptyText}</p>
{:else}
  <ul class="bars">
    {#each readings as r (r.label)}
      <li>
        <span class="name" title={r.label}>{r.label.replace(/-/g, ' ')}</span>
        <span class="track">
          <span class="fill" style="width: {Math.max((r.value / max) * 100, 1.5)}%"></span>
        </span>
        <span class="value">{fmtNumber(r.value, digits)}{suffix}</span>
      </li>
    {/each}
  </ul>
{/if}

<style>
  .bars {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  li {
    display: grid;
    grid-template-columns: minmax(90px, 32%) 1fr auto;
    align-items: center;
    gap: 8px;
  }

  .name {
    font-size: 12px;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    text-transform: capitalize;
  }

  .track {
    background: var(--bg);
    border-radius: 3px;
    height: 12px;
    overflow: hidden;
    border: 1px solid var(--border);
  }

  .fill {
    display: block;
    height: 100%;
    background: linear-gradient(90deg, var(--accent-dim), var(--accent));
    border-radius: 2px;
  }

  .value {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-dim);
    white-space: nowrap;
  }

  .empty {
    color: var(--text-dim);
    font-size: 13px;
    font-style: italic;
    margin: 4px 0;
  }
</style>