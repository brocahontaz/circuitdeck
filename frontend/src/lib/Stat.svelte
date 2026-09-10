<script lang="ts">
  import { fmtNumber } from './format';

  /**
   * Stat card: big value with label and optional tone.
   */
  let {
    label,
    value,
    suffix = '',
    tone = 'normal',
    digits = 1,
    hint = '',
  }: {
    label: string;
    value: number | null | undefined;
    suffix?: string;
    tone?: 'normal' | 'warning' | 'danger' | 'good';
    digits?: number;
    hint?: string;
  } = $props();
</script>

<div class="stat tone-{tone}">
  <span class="label">{label}</span>
  <span class="value">{fmtNumber(value, digits)}<span class="suffix">{suffix}</span></span>
  {#if hint}<span class="hint">{hint}</span>{/if}
</div>

<style>
  .stat {
    background: var(--bg-panel-raised);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .label {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    color: var(--text-dim);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .value {
    font-family: var(--font-mono);
    font-size: 22px;
    color: var(--text-bright);
    line-height: 1.2;
  }

  .suffix {
    font-size: 13px;
    color: var(--text-dim);
    margin-left: 3px;
  }

  .hint {
    font-size: 11px;
    color: var(--text-dim);
  }

  .tone-warning .value {
    color: var(--yellow);
  }
  .tone-danger .value {
    color: var(--red);
  }
  .tone-good .value {
    color: var(--green);
  }
</style>