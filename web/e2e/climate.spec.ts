// M9c slice DM, end to end (ADR-0032 §9): the observer changes the climate from the observer's
// hand. The host answers in words, the weather panel tells where the change stands and where it
// is going, and the chronicle records it. What people make of a warmer, drier valley is theirs.

import { expect, test } from "@playwright/test";

import { makeWorld, startHost, tempSaves } from "./host.js";

test("the observer changes the climate, and the weather panel tells of it", async ({ page }) => {
  const saves = tempSaves();
  makeWorld(saves, ["--seed", "3", "--size", "512", "--days", "2", "--name", "Warm Valley"]);
  const host = await startHost(saves);
  try {
    page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");
    await page.getByRole("button", { name: "Load", exact: true }).click();
    await page.locator("#load-body tbody tr").first().getByRole("button", { name: "Load" }).click();
    await page.waitForFunction(() => (window.__TCE__.state().weather?.months ?? 0) > 12, undefined, {
      timeout: 60_000,
    });
    if (!(await page.evaluate(() => window.__TCE__.state().clock?.paused))) {
      await page.getByRole("button", { name: "Pause" }).click();
    }
    const panel = page.locator("#weather-body");
    await expect(panel.locator(".weather-climate")).toHaveCount(0);

    // Four degrees warmer and a third less rain, from tomorrow.
    await page
      .getByLabel("Change the climate")
      .selectOption({ label: "4 °C warmer and 30% drier from tomorrow" });
    await expect(page.locator("#banner-text")).toContainText(
      "The observer changes the climate: +4.0 °C and wet days' rain ×0.70, from tomorrow",
    );
    await expect(panel.locator(".weather-climate")).toContainText(
      "The observer has changed the climate: +0.0 °C and wet days' rain ×1.00 today, reaching +4.0 °C and ×0.70 by",
    );
    await expect(page.locator("#chronicle")).toContainText(
      "One recorded influence: the observer changed the climate: the climate grows 4.0 °C warmer and wet days bring 30% less rain, from tomorrow.",
    );
  } finally {
    await host.stop();
  }
});
