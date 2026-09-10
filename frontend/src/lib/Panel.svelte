<script lang="ts">
  /**
   * Panel: riveted industrial container used across all views.
   * Content is passed via the Svelte 5 `children` snippet.
   */
  import type { Snippet } from 'svelte';

  let {
    title,
    subtitle = '',
    tone = 'normal',
    children,
  }: {
    title?: string;
    subtitle?: string;
    tone?: 'normal' | 'warning' | 'danger' | 'good';
    children: Snippet;
  } = $props();
</script>

<section class="panel tone-{tone}">
  <div class="rivets" aria-hidden="true"></div>
  {#if title}
    <header>
      <h2>{title}</h2>
      {#if subtitle}<span class="subtitle">{subtitle}</span>{/if}
    </header>
  {/if}
  <div class="body">
    {@render children()}
  </div>
</section>

<style>
  .panel {
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-top: 3px solid var(--border-riveted);
    border-radius: 6px;
    box-shadow:
      inset 0 0 0 1px rgba(0, 0, 0, 0.25),
      0 2px 6px rgba(0, 0, 0, 0.35);
    padding: 12px 16px 16px;
    min-width: 0;
  }

  .rivets {
    display: flex;
    justify-content: space-between;
    height: 4px;
    margin: -4px 0 8px;
  }

  .rivets::before,
  .rivets::after {
    content: '';
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--border-riveted);
    box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.6);
  }

  .panel.tone-warning {
    border-color: var(--accent-dim);
  }
  .panel.tone-danger {
    border-color: var(--red);
  }
  .panel.tone-good {
    border-color: var(--green);
  }

  header {
    display: flex;
    align-items: baseline;
    gap: 10px;
    margin-bottom: 10px;
    flex-wrap: wrap;
  }

  h2 {
    margin: 0;
    font-size: 14px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--text-dim);
  }

  .subtitle {
    font-size: 12px;
    color: var(--text-dim);
  }

  .body {
    min-width: 0;
  }
</style>