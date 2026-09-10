<script lang="ts">
  import { untrack } from 'svelte';
  import { VIEWS, currentView, type ViewId } from './lib/router';
  import {
    fetchHealth,
    fetchOverview,
    fetchProduction,
    fetchPower,
    fetchFactory,
    fetchLogistics,
    fetchTrains,
    fetchResearch,
    fetchPlatforms,
    type HealthReport,
    type Overview as OverviewData,
    type ProductionReport,
    type PowerReport,
    type FactoryReport,
    type LogisticsReport,
    type TrainsReport,
    type ResearchReport,
    type PlatformReport,
    type TimeRange,
  } from './lib/api';
  import Overview from './views/Overview.svelte';
  import Production from './views/Production.svelte';
  import Power from './views/Power.svelte';
  import Factory from './views/Factory.svelte';
  import Logistics from './views/Logistics.svelte';
  import Trains from './views/Trains.svelte';
  import Research from './views/Research.svelte';
  import Platforms from './views/Platforms.svelte';

  let view: ViewId = $state(currentView(window.location.hash));
  let range: TimeRange = $state('1h');

  // Shared view data store: each view caches its latest payload so switching
  // tabs shows data instantly while refreshing in the background.
  let overview = $state<OverviewData | null>(null);
  let production = $state<ProductionReport | null>(null);
  let power = $state<PowerReport | null>(null);
  let factory = $state<FactoryReport | null>(null);
  let logistics = $state<LogisticsReport | null>(null);
  let trains = $state<TrainsReport | null>(null);
  let research = $state<ResearchReport | null>(null);
  let platforms = $state<PlatformReport | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  let health = $state<HealthReport | null>(null);

  async function refreshHealth(): Promise<void> {
    try {
      health = await fetchHealth();
    } catch {
      health = {
        prometheus_reachable: false,
        factorio_metrics_job_present: false,
        game_ticks_played: null,
      };
    }
  }

  async function load(): Promise<void> {
    loading = overview === null && view === 'overview';
    error = null;
    const r = range;
    try {
      switch (view) {
        case 'overview':
          overview = await fetchOverview(r);
          break;
        case 'production':
          production = await fetchProduction(r);
          break;
        case 'power':
          power = await fetchPower(r);
          break;
        case 'factory':
          factory = await fetchFactory(r);
          break;
        case 'logistics':
          logistics = await fetchLogistics(r);
          break;
        case 'trains':
          trains = await fetchTrains(r);
          break;
        case 'research':
          research = await fetchResearch(r);
          break;
        case 'platforms':
          platforms = await fetchPlatforms(r);
          break;
      }
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to load data.';
    }
    loading = false;
  }

  // Reload whenever the view or range changes; poll the active view.
  // IMPORTANT: only `view` and `range` are read inside this effect. The data
  // stores are read via untrack() so that a completed fetch (which writes a
  // new object into a store) does not re-trigger this effect and cause an
  // infinite re-fetch loop.
  $effect(() => {
    // Reads of `view`/`range` register as effect dependencies; everything
    // written by load() is untracked so a finished fetch never re-runs it.
    void view;
    void range;
    loading = !untrack(hasCachedData);
    error = null;
    untrack(() => void load());
    const t = setInterval(() => untrack(() => void load()), 10_000);
    return () => clearInterval(t);
  });

  function hasCachedData(): boolean {
    switch (view) {
      case 'overview':
        return overview !== null;
      case 'production':
        return production !== null;
      case 'power':
        return power !== null;
      case 'factory':
        return factory !== null;
      case 'logistics':
        return logistics !== null;
      case 'trains':
        return trains !== null;
      case 'research':
        return research !== null;
      case 'platforms':
        return platforms !== null;
    }
  }

  // Health beacon refreshes on its own cadence.
  $effect(() => {
    void refreshHealth();
    const t = setInterval(refreshHealth, 15_000);
    return () => clearInterval(t);
  });

  $effect(() => {
    const onHash = (): void => {
      view = currentView(window.location.hash);
    };
    window.addEventListener('hashchange', onHash);
    return () => window.removeEventListener('hashchange', onHash);
  });

  function navigate(id: ViewId): void {
    window.location.hash = `#/${id}`;
    view = id;
  }

  const RANGES: { value: TimeRange; label: string }[] = [
    { value: '1h', label: '1h' },
    { value: '6h', label: '6h' },
    { value: '24h', label: '24h' },
    { value: '7d', label: '7d' },
  ];

  const beaconTone = $derived(
    health === null
      ? 'unknown'
      : health.prometheus_reachable && health.factorio_metrics_job_present
        ? 'good'
        : 'bad',
  );
</script>

<div class="app">
  <header class="topbar">
    <div class="brand">
      <span class="logo" aria-hidden="true">▦</span>
      <div class="brand-text">
        <h1>CircuitDeck</h1>
        <span class="tagline">factorio control deck</span>
      </div>
    </div>

    <nav aria-label="Dashboard views">
      {#each VIEWS as v (v.id)}
        <button
          class="nav-btn"
          class:active={view === v.id}
          onclick={() => navigate(v.id)}
          aria-current={view === v.id ? 'page' : undefined}
        >
          <span class="icon">{v.icon}</span>
          {v.label}
        </button>
      {/each}
    </nav>

    <div class="beacon" title="Telemetry status">
      <span class="dot {beaconTone}"></span>
      <span class="beacon-label">
        {health === null
          ? '…'
          : health.prometheus_reachable
            ? health.factorio_metrics_job_present
              ? 'live'
              : 'no factorio feed'
            : 'offline'}
      </span>
    </div>
  </header>

  <div class="range-bar">
    <span class="range-label">Range:</span>
    {#each RANGES as r (r.value)}
      <button
        class="range-btn"
        class:active={range === r.value}
        onclick={() => (range = r.value)}
      >
        {r.label}
      </button>
    {/each}
  </div>

  <main>
    {#if view === 'overview'}
      <Overview data={overview} {loading} {error} />
    {:else if view === 'production'}
      <Production data={production} {loading} {error} />
    {:else if view === 'power'}
      <Power data={power} {loading} {error} />
    {:else if view === 'factory'}
      <Factory data={factory} {loading} {error} />
    {:else if view === 'logistics'}
      <Logistics data={logistics} {loading} {error} />
    {:else if view === 'trains'}
      <Trains data={trains} {loading} {error} />
    {:else if view === 'research'}
      <Research data={research} {loading} {error} />
    {:else if view === 'platforms'}
      <Platforms data={platforms} {loading} {error} />
    {/if}
  </main>

  <footer>
    <span>CircuitDeck · self-hosted Factorio telemetry · data via Prometheus (graftorio3)</span>
  </footer>
</div>

<style>
  .app {
    max-width: 1280px;
    margin: 0 auto;
    padding: 16px 20px 32px;
    display: flex;
    flex-direction: column;
    min-height: 100vh;
  }

  .topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    flex-wrap: wrap;
    padding-bottom: 14px;
    border-bottom: 1px solid var(--border);
    margin-bottom: 16px;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .logo {
    font-size: 30px;
    color: var(--accent);
    text-shadow: 0 0 12px rgba(255, 154, 31, 0.45);
  }

  h1 {
    margin: 0;
    font-size: 20px;
    letter-spacing: 0.05em;
  }

  .tagline {
    display: block;
    font-size: 11px;
    color: var(--text-dim);
    letter-spacing: 0.12em;
    text-transform: uppercase;
  }

  nav {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
  }

  .nav-btn {
    background: var(--bg-panel);
    color: var(--text);
    border: 1px solid var(--border);
    border-bottom: 3px solid transparent;
    padding: 7px 12px;
    font-size: 13px;
    cursor: pointer;
    border-radius: 3px;
    font-family: inherit;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    transition:
      border-color 120ms,
      background 120ms;
  }

  .nav-btn:hover {
    background: var(--bg-panel-raised);
  }

  .nav-btn.active {
    border-bottom-color: var(--accent);
    color: var(--text-bright);
    background: var(--bg-panel-raised);
  }

  .icon {
    opacity: 0.75;
  }

  .beacon {
    display: flex;
    align-items: center;
    gap: 7px;
    font-size: 12px;
    color: var(--text-dim);
  }

  .dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--text-dim);
  }

  .dot.good {
    background: var(--green);
    box-shadow: 0 0 8px rgba(125, 194, 66, 0.7);
    animation: pulse 2s ease-in-out infinite;
  }

  .dot.bad {
    background: var(--red);
    box-shadow: 0 0 8px rgba(224, 79, 63, 0.7);
  }

  .range-bar {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 14px;
  }

  .range-label {
    font-size: 12px;
    color: var(--text-dim);
    text-transform: uppercase;
    letter-spacing: 0.07em;
  }

  .range-btn {
    background: var(--bg-panel);
    border: 1px solid var(--border);
    color: var(--text-dim);
    font-family: var(--font-mono);
    font-size: 12px;
    padding: 4px 10px;
    border-radius: 3px;
    cursor: pointer;
  }

  .range-btn.active {
    color: var(--text-bright);
    border-color: var(--accent-dim);
    background: rgba(255, 154, 31, 0.12);
  }

  footer {
    margin-top: auto;
    padding-top: 18px;
    font-size: 11px;
    color: var(--text-dim);
    border-top: 1px solid var(--border);
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.55;
    }
  }
</style>