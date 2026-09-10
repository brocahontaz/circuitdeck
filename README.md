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
├── prometheus/       Prometheus scrape config + factorio_* recording rules
├── dev/metrics-mock/ graftorio3-compatible mock metrics source (dev only)
├── factorio/         Real-Factorio mode: vendored graftorio3 mod + bootstrap
├── docker/           Frontend Caddyfile
├── Dockerfile/       Multi-stage builds (backend, frontend)
├── docker-compose.yml Dev environment, two modes (mock | factorio profiles)
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

Two mutually exclusive development modes, selected with Compose profiles:

```sh
# Fast/default development using mock Factorio data (no Factorio required)
docker compose --profile mock up --build

# Development against a real Factorio dedicated server
docker compose --profile factorio up --build
```

Then open **http://localhost:8088** in both cases.
(`make up` / `make up-factorio` are shortcuts; copying `.env.example` to
`.env` makes plain `docker compose up` default to mock mode.)

### Mock mode (default)

```sh
docker compose --profile mock up --build
```

This starts, with no real Factorio server required:

| Service                | Port | Purpose                                    |
| ---------------------- | ---- | ------------------------------------------ |
| `circuitdeck-frontend` | 8088 | Dashboard UI + `/api` reverse proxy        |
| `circuitdeck-backend`  | 8080 | Rust API (owns all PromQL)                 |
| `prometheus`           | 9090 | Telemetry storage/query                    |
| `circuitdeck-mock`     | 9105 | graftorio3-compatible mock metrics source  |

The mock generator emits realistic graftorio3 metrics with a deterministic
day/night power cycle, drifting production rates and rolling research, so the
dashboard shows believable moving data immediately. Replace it in production
by pointing Prometheus at your Factorio host (see below).

### Real Factorio mode

```sh
docker compose --profile factorio up --build
```

This starts the dashboard stack plus:

| Service             | Port                | Purpose                                            |
| ------------------- | ------------------- | -------------------------------------------------- |
| `factorio`          | 34197/udp, 27015/tcp | Factorio 2.1 dedicated server (LAN, join in game) |
| `factorio-exporter` | 9100                | Serves the graftorio3 `.prom` textfile to Prometheus |

How it works:

1. `factorio/` contains the vendored
   [graftorio3 2.0.0](https://github.com/Furoon/graftorio3) mod (MIT) and a
   bootstrap script. The server image (`factoriotools/factorio`, version
   configurable via `FACTORIO_VERSION`) installs the mod and LAN server
   settings on first start, then auto-generates a save (`circuitdeck-dev`).
2. graftorio3 writes `script-output/graftorio3/game.prom` (Prometheus
   textfile) every few game seconds.
3. `factorio-exporter` (tiny busybox sidecar) serves that file at
   `:9100/metrics`; Prometheus scrapes it.
4. `prometheus/factorio-rules.yml` records rules translating the mod's
   `factorio_*` series into the canonical `graftorio_factorio_*` series the
   backend queries — the dashboard needs no mock-vs-real knowledge.
5. The save lives in the `factorio-data` Docker volume and survives restarts;
   `docker compose --profile factorio down` keeps it, `make factorio-reset`
   deletes it (next start regenerates a fresh map).

Useful commands:

```sh
make factorio-logs   # tail Factorio server logs
make factorio-reset  # wipe the local save (destructive)
docker compose --profile factorio exec factorio rcon /h   # RCON console
```

To connect with the game client: join `localhost:34197` (LAN), password from
`FACTORIO_GAME_PASSWORD` if set.

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
- the Docker stack in both modes (`docker compose --profile mock up --build`
  and `--profile factorio`) as the end-to-end check

## Container images

On every push to `main` and on `v*` tags, GitHub Actions builds and publishes
`linux/amd64` images to GHCR:

| Image | Purpose |
| ----- | ------- |
| `ghcr.io/<owner>/circuitdeck-backend` | Rust/Axum dashboard API |
| `ghcr.io/<owner>/circuitdeck-frontend` | Caddy serving the Svelte SPA + `/api` proxy |

Tags: `latest` and `main` (default branch), `sha-<short>` (every build),
`<semver>` and `<major>.<minor>` (on `v*` tags). Images are gated by the CI
test suite and carry build provenance attestations. Deployment consumes these
images from a separate infrastructure repository; Prometheus and the dev-only
services run from upstream images / compose profiles as before.

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
   files there. If that host also runs the graftorio3 mod, enable
   `FACTORIO_RECORDING_RULES=true` so its `factorio_*` series are translated
   into the canonical `graftorio_factorio_*` names.

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
| `COMPOSE_PROFILES`             | compose    | `mock` (when using .env) |
| `FACTORIO_METRICS_TARGET`      | prometheus | *(auto: mock or exporter)* |
| `FACTORIO_RECORDING_RULES`     | prometheus | *(auto: on in real mode)* |
| `FACTORIO_VERSION`             | factorio   | `latest` (Factorio 2.1)  |
| `FACTORIO_SAVE_NAME`           | factorio   | `circuitdeck-dev`        |
| `FACTORIO_PRESET`              | factorio   | *(map-gen defaults)*     |
| `FACTORIO_GAME_PORT`           | factorio   | `34197` (UDP)            |
| `FACTORIO_RCON_PORT`           | factorio   | `27015` (TCP, localhost) |
| `FACTORIO_RCON_PASSWORD`       | factorio   | `circuitdeck` (dev only) |
| `FACTORIO_GAME_PASSWORD`       | factorio   | *(none)*                 |

Containers are non-root, multi-stage, and carry health checks. Prometheus owns
all telemetry history (15 days retention by default); no application database
is used.

## Using real graftorio3 metrics

The backend always queries the canonical `graftorio_factorio_*` series.
Three sources feed them:

- **dev mock** — the bundled generator emits `graftorio_factorio_*` directly.
- **dev real / prod with the mod** — real graftorio3 emits `factorio_*`
  textfiles; `prometheus/factorio-rules.yml` records them into
  `graftorio_factorio_*` (auto-enabled in real mode, or explicitly with
  `FACTORIO_RECORDING_RULES=true`).
- **prod** — point Prometheus at your node_exporter textfile collector:
  `FACTORIO_METRICS_TARGET=your-factorio-host:9100`. Any metric your
  graftorio3 build names differently can be adapted in one place:
  `backend/src/promql/query.rs` (+ `prometheus/factorio-rules.yml`).

## License

MIT

The vendored graftorio3 mod (`factorio/mods/graftorio3_2.0.0.zip`) is MIT,
© Keith Thornhill and contributors — see
`factorio/mods/graftorio3_2.0.0-LICENSE`.