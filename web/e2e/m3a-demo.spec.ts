// The M3a demo's pictures (plan §7): one seed made twice from the command line, under household
// fields and under village fields, a village of hundreds (a founding band of 125 joined by 20
// families) lived five years; then each world's wealth and market panels and its village. It takes
// some time, so it runs only with TCE_DEMO=1, and TCE_SHOTS_DIR saves the screenshots.
// TCE_DEMO_SEED, TCE_DEMO_BAND, TCE_DEMO_FAMILIES and TCE_DEMO_DAYS change the seed, the band,
// the families sent and the days lived. Runs differ (each world draws its own identity), so the
// numbers it logs are one run's; the decision log compares several runs of each regime.

import { expect, test, type Page } from "@playwright/test";

import { makeWorld, startHost, tempSaves } from "./host.js";

test.skip(!process.env.TCE_DEMO, "the M3a demo runs only with TCE_DEMO=1");
test.setTimeout(120 * 60_000);
test.use({ viewport: { width: 1440, height: 900 } });

const shots = process.env.TCE_SHOTS_DIR;

const REGIMES: [string, string][] = [
  ["core:regime/household", "Household Fields"],
  ["core:regime/village", "Village Fields"],
];

/** Zooms the map about its centre to `scale` pixels a metre. */
async function zoomTo(page: Page, scale: number): Promise<void> {
  await page.evaluate((s) => window.__TCE__.zoomBy(s / window.__TCE__.map().camera.scale), scale);
}

test("one seed under both regimes, years on", async ({ page }) => {
  const saves = tempSaves();
  const seed = process.env.TCE_DEMO_SEED ?? "2";
  const band = process.env.TCE_DEMO_BAND ?? "125";
  const families = process.env.TCE_DEMO_FAMILIES ?? "20";
  const days = process.env.TCE_DEMO_DAYS ?? "1825";
  for (const [regime, name] of REGIMES) {
    const args = ["--seed", seed, "--band", band, "--families", families, "--days", days];
    makeWorld(saves, [...args, "--regime", regime, "--name", name], 60 * 60_000);
  }
  const host = await startHost(saves);
  try {
    page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");
    for (const [, name] of REGIMES) {
      await page.getByRole("button", { name: "Load", exact: true }).click();
      await page
        .locator("#load-body tbody tr")
        .filter({ hasText: name })
        .getByRole("button", { name: "Load" })
        .click();
      await page.waitForFunction(
        (n) =>
          window.__TCE__.state().world?.name === n &&
          (window.__TCE__.state().wealth?.settlements[0]?.history.length ?? 0) > 0 &&
          (window.__TCE__.state().markets?.length ?? 0) > 0 &&
          window.__TCE__.map().state === "ready",
        name,
        { timeout: 300_000 },
      );
      const state = await page.evaluate(() => window.__TCE__.state());
      console.log(`demo ${name}: ${JSON.stringify({ wealth: state.wealth, markets: state.markets })}`);
      const slug = name.toLowerCase().replace(/ /g, "-");
      const market = page.locator("#market-body");
      await expect(market.locator(".market-head")).toBeVisible();
      await market.locator(".market-head").getByRole("button").click();
      await zoomTo(page, 1.6);
      await page.waitForTimeout(3000);
      if (shots) {
        await page.screenshot({ path: `${shots}/m3a-${slug}-village.png` });
        await page.locator("#wealth-body").screenshot({ path: `${shots}/m3a-${slug}-wealth.png` });
        await market.screenshot({ path: `${shots}/m3a-${slug}-market.png` });
      }
    }
  } finally {
    await host.stop();
  }
});
