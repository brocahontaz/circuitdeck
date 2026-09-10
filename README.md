# CircuitDeck

A self-hosted Factorio 2.1 dashboard: production stats, power, research, trains,
Space Age platforms and telemetry health in one place.

CircuitDeck consumes the graftorio3 Prometheus metrics exported by a Factorio
server, stores history in Prometheus, and serves a custom Svelte dashboard over
a Rust/Axum API. **The browser never queries Prometheus directly** — the Rust
backend owns all PromQL and exposes a stable, Factorio-specific JSON API.
No Grafana is required.

## Architecture

```
Factorio 2.1
  -> graftorio3 (writes .prom files / metrics endpoint)
  -> node_exporter textfile collector (production) | mock generator (dev)
  -> Prometheus
  -> Rust/Axum dashboard API  (owns every PromQL query)
  -> Svelte 5 dashboard       (never touches Prometheus)
```

```
├── backend/          Rust + Axum API
│   └── src/
│       ├── api/      Axum routes, state, router assembly
│       ├── domain/   Domain models + Prometheus→domain mapping
│       ├── prom/     Prometheus HTTP API client (typed)
│       ├── promql/   Centralized PromQL definitions + time ranges
│       ├── config.rs Environment configuration
│       └── error.rs  Structured error type (JSON error bodies)
├── frontend/         Svelte 5 + TypeScript + Vite + ECharts
│   └── src/
│       ├── lib/      API client, chart wrapper, UI primitives, router
│       └── views/    One component per dashboard view
├── prometheus/       Prometheus scrape configuration
├── dev/metrics-mock/ graftorio3-compatible mock metrics source (dev only)
├── docker/           Frontend Caddyfile
├── Dockerfile/       Multi-stage builds (backend, frontend)
├── docker-compose.yml Complete dev environment (one command)
└── Makefile           Host-side development tasks
```

### API endpoints

| Endpoint          | Description                                       |
| ----------------- | ------------------------------------------------- |
| `GET /api/health` | Prometheus reachability + Factorio feed presence  |
| `GET /api/overview`  | Summary cards: power, evolution, research, top items |
| `GET /api/production` | Item/fluid flows with time series          |
| `GET /api/power`  | Production/consumption, sources, accumulators     |
| `GET /api/factory`| Machine census                                    |
| `GET /api/logistics` | Robots, networks, top stored items            |
| `GET /api/trains` | Trains by state, station waiting cargo            |
| `GET /api/research` | Current research, progress, queue               |
| `GET /api/platforms` | Space Age platform speed/fuel                 |

All historical endpoints accept `?range=1h|6h|24h|7d` (default `1h`).
Errors return a structured JSON body: `{"error": {"code", "message"}}`.

## Quick start (development)

```sh
docker compose up --build
```

Then open **http://localhost:8088**.

This starts, with no real Factorio server required:

| Service              | Port | Purpose                                            |
| -------------------- | ---- | -------------------------------------------------- |
| `circuitdeck-frontend` | 8088 | Dashboard UI + `/api` reverse proxy               |
| `circuitdeck-backend`  | 8080 | Rust API (owns all PromQL)                        |
| `prometheus`           | 9090 | Telemetry storage/query                           |
| `circuitdeck-mock`     | 9105 | graftorio3-compatible mock metrics source         |

The mock generator emits realistic graftorio3 metrics with a deterministic
day/night power cycle, drifting production rates and rolling research, so the
dashboard shows believable moving data immediately. Replace it in production
by pointing Prometheus at your Factorio host (see below).

### Host-side development (no Docker)

```sh
# 1. mock metrics source
python3 dev/metrics-mock/serve.py                    # :9105

# 2. Prometheus (any local install), e.g.
prometheus --config.file=prometheus/prometheus.yml   # :9090

# 3. backend
cd backend && cargo run                              # :8080

# 4. frontend with /api proxy
cd frontend && npm install && npm run dev            # :5173
```

The Vite dev server proxies `/api` to `http://127.0.0.1:8080`
(override with `VITE_DEV_API_TARGET`).

## Testing and linting

```sh
make check        # backend + frontend tests and lints
make check-all    # + production builds
```

Individually:

```sh
cd backend  && cargo test && cargo fmt --check && cargo clippy --all-targets -- -D warnings
cd frontend && npm run test && npm run lint && npm run format:check && npm run build
```

Test coverage includes:

- backend unit tests (Prometheus response parsing, PromQL definitions, config)
- backend API integration tests against a mock Prometheus (wiremock)
- frontend unit tests (formatting, routing, API error mapping)
- frontend component tests (loading/empty/degraded/error states)
- the Docker stack (`docker compose up --build`) as the end-to-end check

## Production setup

The stack is designed to sit behind an external Traefik reverse proxy — Traefik
is **not** bundled. Example labels for the compose services:

```yaml
# frontend service
labels:
  - "traefik.enable=true"
  - "traefik.http.routers.circuitdeck.rule=Host(`factorio.public.fjallbark.cloud`)"
  - "traefik.http.routers.circuitdeck.entrypoints=websecure"
  - "traefik.http.services.circuitdeck.loadbalancer.server.port=8080"
```

1. Copy `.env.example` to `.env` and adjust.
2. Point Prometheus at the real metrics source:

   ```env
   FACTORIO_METRICS_TARGET=your-factorio-host:9100
   ```

   On the Factorio host, run node_exporter with the textfile collector
   (`--collector.textfile.directory=...`) and let graftorio3 write its `.prom`
   files there.

   Note: the official `prom/prometheus` image does **not** expand environment
   variables inside config files, so the compose entrypoint renders the
   `prometheus/prometheus.yml` template with `sed` at startup into the
   `/prometheus` volume (writable by the image's `nobody` user) and feeds the
   rendered copy to Prometheus.
3. Do **not** publish the backend or Prometheus ports publicly; only the
   frontend container (which proxies `/api`) needs exposure.

### Configuration (environment variables)

| Variable                       | Used by    | Default                  |
| ------------------------------ | ---------- | ------------------------ |
| `PROMETHEUS_URL`               | backend    | `http://localhost:9090`  |
| `CIRCUITDECK_BIND_ADDR`        | backend    | `0.0.0.0:8080`           |
| `PROMETHEUS_QUERY_TIMEOUT_SECS`| backend    | `10`                     |
| `PROMETHEUS_PING_TIMEOUT_SECS` | backend    | `3`                      |
| `CIRCUITDECK_CORS_ORIGINS`     | backend    | *(same-origin only)*     |
| `VITE_API_BASE_URL`            | frontend   | `/api`                   |
| `FACTORIO_METRICS_TARGET`      | prometheus | `circuitdeck-mock:9105`  |

Containers are non-root, multi-stage, and carry health checks. Prometheus owns
all telemetry history (15 days retention by default); no application database
is used.

## Using real graftorio3 metrics

The mock source implements the same metric names and labels the backend
queries. Once graftorio3 metrics flow into Prometheus (via the textfile
collector), set `FACTORIO_METRICS_TARGET` and the dashboard switches to real
data — no code changes. Any metric your graftorio3 build names differently can
be adapted in one place: `backend/src/promql/query.rs`.

## License

MIT