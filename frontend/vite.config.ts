import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vite";

// https://vite.dev/config/
export default defineConfig({
  plugins: [svelte()],
  build: {
    // Vite 8 (rolldown): split the large charting library out of the app
    // bundle so app changes don't invalidate the vendor chunk in caches.
    rollupOptions: {
      output: {
        advancedChunks: {
          groups: [
            {
              name: "echarts",
              test: /node_modules\/(echarts|zrender)/,
              priority: 1,
            },
          ],
        },
      },
    },
  },
  server: {
    // Local dev: proxy /api to a locally running backend (cargo run).
    proxy: {
      "/api": {
        target: process.env.VITE_DEV_API_TARGET ?? "http://127.0.0.1:8080",
        changeOrigin: true,
      },
    },
  },
});
