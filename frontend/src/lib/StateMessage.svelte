<script lang="ts">
  /**
   * StateMessage: shared loading / empty / error renderer.
   */
  let {
    state,
    message = '',
  }: {
    state: 'loading' | 'empty' | 'degraded' | 'error';
    message?: string;
  } = $props();

  const ICONS = { loading: '⟳', empty: '∅', degraded: '⚠', error: '✖' };
  const DEFAULT_TEXT = {
    loading: 'Loading telemetry…',
    empty: 'No data in this range yet.',
    degraded: 'Telemetry is degraded — some sources are unreachable.',
    error: 'Failed to load data.',
  };
</script>

<div class="state {state}" role="status">
  <span class="icon" class:spin={state === 'loading'}>{ICONS[state]}</span>
  <span>{message || DEFAULT_TEXT[state]}</span>
</div>

<style>
  .state {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 18px 14px;
    font-size: 14px;
    border: 1px dashed var(--border);
    border-radius: 4px;
    color: var(--text-dim);
    background: rgba(0, 0, 0, 0.15);
  }

  .icon {
    font-size: 18px;
    line-height: 1;
  }

  .spin {
    display: inline-block;
    animation: spin 1.2s linear infinite;
  }

  .error {
    color: var(--red);
    border-color: var(--red);
  }

  .degraded {
    color: var(--yellow);
    border-color: var(--accent-dim);
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>