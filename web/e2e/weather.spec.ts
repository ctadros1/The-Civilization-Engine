// M3c slice U, end to end: the weather (ADR-0012). The clock tells today's weather in words, and
// the weather panel lists the months and years lived on the valley floor against what each
// usually brings.

import { expect, test } from "@playwright/test";

import { makeWorld, startHost, tempSaves } from "./host.js";

const shots = process.env.TCE_SHOTS_DIR;

test("the clock and the weather panel tell the weather the world lives", async ({ page }) => {
  const saves = tempSaves();
  // 1 March of year 1 to mid-May: crops sown and growing.
  makeWorld(saves, ["--seed", "3", "--size", "512", "--days", "75", "--name", "Rainy Valley"]);
  const host = await startHost(saves);
  try {
    page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");
    const panel = page.locator("#weather-body");
    await expect(panel).toContainText("No world loaded.");
    await page.getByRole("button", { name: "Load", exact: true }).click();
    await page.locator("#load-body tbody tr").first().getByRole("button", { name: "Load" }).click();
    await page.waitForFunction(() => (window.__TCE__.state().weather?.months ?? 0) > 12, undefined, {
      timeout: 60_000,
    });
    // The clock says the day's weather in the kernel's words.
    await expect(page.locator("#clock")).toContainText("°C");
    const today = await page.evaluate(() => window.__TCE__.state().weather?.today);
    expect(today).toMatch(/°C, /);
    // The panel: today, then the months, newest first, against the usual, then the years.
    await expect(panel).toContainText("Today:");
    await expect(panel.locator(".weather-months tbody tr").first()).toContainText("May 1");
    await expect(panel.locator(".weather-months tbody tr")).toHaveCount(12);
    await expect(panel).toContainText("what it usually brings");
    // A new world has lived the year before its founding: year 0 from March, and year 1 so far.
    await expect(panel.locator(".weather-years tbody tr").first()).toContainText("1 (");
    await expect(panel.locator(".weather-years")).toContainText("0 (306 days)");
    if (shots) {
      await page.locator("#weather-panel").screenshot({ path: `${shots}/weather.png` });
      await page.locator(".topbar").screenshot({ path: `${shots}/weather-clock.png` });
    }
  } finally {
    await host.stop();
  }
});
