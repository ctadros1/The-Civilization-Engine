import { defineConfig } from "vitest/config";

// civ-host serves the built shell and the observer socket from one origin. During development
// Vite serves the shell and forwards /ws to a host started with `civ-host serve`.
const host = process.env.TCE_HOST ?? "127.0.0.1:7420";

export default defineConfig({
  server: {
    host: "127.0.0.1",
    port: 5173,
    proxy: {
      "/ws": { target: `ws://${host}`, ws: true },
    },
  },
  build: {
    target: "es2022",
    sourcemap: true,
    chunkSizeWarningLimit: 1500,
  },
  test: {
    include: ["tests/**/*.test.ts"],
    environment: "node",
  },
});
