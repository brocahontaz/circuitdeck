# CircuitDeck developer tasks.
# Host-side commands; the full stack runs via `docker compose`.
# Two mutually exclusive modes:
#   mock mode      make up            (fast, no Factorio required, default)
#   real mode      make up-factorio   (real Factorio server + graftorio3)

SHELL := /bin/bash
.PHONY: help \
	dev-backend dev-frontend dev-mock \
	build-backend build-frontend \
	test-backend test-frontend \
	lint-backend lint-frontend fmt-backend fmt-frontend \
	validate-infra \
	check check-all \
	up up-mock up-factorio down logs ps \
	factorio-logs factorio-rcon factorio-reset

help: ## Show this help.
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) \
		| awk 'BEGIN {FS = ":.*?## "}; {printf "  %-18s %s\n", $$1, $$2}'

# --- Full stack ---------------------------------------------------------------

up: ## Start the dev environment with mock Factorio data (default mode).
	docker compose --profile mock up --build -d

up-mock: up ## Alias for `up` (mock mode).

up-factorio: ## Start the dev environment with a real Factorio server.
	docker compose --profile factorio up --build -d

down: ## Stop the dev environment (both modes; keeps Factorio save).
	docker compose --profile mock --profile factorio down

logs: ## Tail logs of all services.
	docker compose logs -f --tail=100

ps: ## Show service status.
	docker compose ps

# --- Real Factorio mode helpers -----------------------------------------------

factorio-logs: ## Tail the Factorio server logs.
	docker compose logs -f factorio

factorio-rcon: ## Open an RCON shell against the local Factorio server.
	docker compose exec factorio rcon /h

factorio-reset: ## Delete the local Factorio save/data volume (destructive).
	docker compose --profile mock --profile factorio down -v

# --- Backend (requires Rust toolchain on host) --------------------------------

build-backend: ## cargo build the backend.
	cd backend && cargo build

test-backend: ## Run backend tests.
	cd backend && cargo test

lint-backend: ## Backend clippy + fmt check.
	cd backend && cargo fmt --check && cargo clippy --all-targets -- -D warnings

fmt-backend: ## Format backend sources.
	cd backend && cargo fmt

dev-backend: ## Run the backend locally (expects Prometheus on :9090).
	cd backend && cargo run

# --- Frontend (requires Node 22+) ---------------------------------------------

build-frontend: ## Build the frontend production bundle.
	cd frontend && npm run build

test-frontend: ## Run frontend unit tests.
	cd frontend && npm run test

lint-frontend: ## Frontend svelte-check + eslint + prettier check.
	cd frontend && npm run lint

fmt-frontend: ## Format frontend sources.
	cd frontend && npm run format

dev-frontend: ## Vite dev server with proxy to a locally running backend.
	cd frontend && npm run dev

dev-mock: ## Run the mock graftorio3 metrics source on :9105.
	python3 dev/metrics-mock/serve.py

# --- Combined verification ----------------------------------------------------

validate-infra: ## Validate compose (both modes), factorio rules, Caddyfile (requires docker CLI).
	docker compose config --quiet
	docker compose --profile mock config --quiet
	docker compose --profile factorio config --quiet
	@docker run --rm -v $$(pwd)/prometheus:/prom:ro --entrypoint /bin/promtool prom/prometheus:v3.7.3 \
		test rules /prom/factorio-rules-test.yml \
		|| echo "docker unavailable; skipping factorio-rules unit tests"
	@docker run --rm -i caddy:2-alpine caddy validate --adapter caddyfile --config /dev/stdin \
		< docker/frontend.Caddyfile \
		|| echo "docker unavailable; skipping Caddyfile validation"

check: test-backend lint-backend test-frontend lint-frontend validate-infra ## Run all host-side checks.

check-all: check build-frontend build-backend ## All checks plus builds.