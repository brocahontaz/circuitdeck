# CircuitDeck developer tasks.
# Host-side commands; the full stack runs via `docker compose up --build`.

SHELL := /bin/bash
.PHONY: help \
	dev-backend dev-frontend dev-mock \
	build-backend build-frontend \
	test-backend test-frontend \
	lint-backend lint-frontend fmt-backend fmt-frontend \
	validate-infra \
	check check-all \
	up down logs ps

help: ## Show this help.
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) \
		| awk 'BEGIN {FS = ":.*?## "}; {printf "  %-18s %s\n", $$1, $$2}'

# --- Full stack --------------------------------------------------------------

up: ## Build and start the complete dev environment (docker compose).
	docker compose up --build -d

down: ## Stop the dev environment.
	docker compose down

logs: ## Tail logs of all services.
	docker compose logs -f --tail=100

ps: ## Show service status.
	docker compose ps

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

validate-infra: ## Validate compose + Caddyfile syntax (requires docker CLI + caddy).
	docker compose config --quiet
	@command -v caddy >/dev/null 2>&1 && caddy validate --config docker/frontend.Caddyfile --adapter caddyfile || \
		echo "caddy CLI not installed; skipping Caddyfile validation"

check: test-backend lint-backend test-frontend lint-frontend validate-infra ## Run all host-side checks.

check-all: check build-frontend build-backend ## All checks plus builds.