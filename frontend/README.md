# CircuitDeck frontend

Svelte 5 + TypeScript + Vite dashboard for CircuitDeck.

```sh
npm install
npm run dev        # dev server with /api proxy to a local backend
npm run test       # vitest unit/component tests
npm run lint       # eslint + svelte-check + tsc
npm run format     # prettier --write
npm run build      # production bundle (dist/)
```

The browser never talks to Prometheus; all requests go to the Rust API via
`/api` (see `src/lib/api.ts` and `VITE_API_BASE_URL`).