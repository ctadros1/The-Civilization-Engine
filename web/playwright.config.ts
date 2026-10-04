import { defineConfig } from "@playwright/test";

// The specs start their own civ-host processes (see e2e/host.ts) against the built shell in
// dist/, so run `npm run build` and build civ-host first.
export default defineConfig({
  testDir: "./e2e",
  timeout: 240_000,
  expect: { timeout: 60_000 },
  fullyParallel: false,
  workers: 1,
  reporter: [["list"]],
  outputDir: "test-results",
  use: {
    headless: true,
    viewport: { width: 1400, height: 860 },
    screenshot: "only-on-failure",
    trace: "retain-on-failure",
    video: process.env.TCE_DEMO_VIDEO ? "on" : "off",
  },
});
