// M8d slice DA, end to end (ADR-0031 §4): the observer shakes the ground from the observer's hand.
// The host answers in words with what fell and cracked, and the chronicle records the shaking as
// the observer's. What people make of what fell is theirs.

import { expect, test } from "@playwright/test";

import { makeWorld, startHost, tempSaves } from "./host.js";

test("the observer shakes the ground, and is told what fell", async ({ page }) => {
  const saves = tempSaves();
  makeWorld(saves, ["--seed", "3", "--size", "512", "--days", "60", "--name", "Shaken Valley"]);
  const host = await startHost(saves);
  try {
    page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");
    await page.getByRole("button", { name: "Load", exact: true }).click();
    await page.locator("#load-body tbody tr").first().getByRole("button", { name: "Load" }).click();
    await page.waitForFunction(
      () => window.__TCE__.map().state === "ready" && window.__TCE__.map().roofed > 0,
      undefined,
      { timeout: 120_000 },
    );
    if (!(await page.evaluate(() => window.__TCE__.state().clock?.paused))) {
      await page.getByRole("button", { name: "Pause" }).click();
    }
    const shake = page.getByRole("button", { name: "Shake the ground" });
    await page.getByLabel("How hard the ground shakes").selectOption("0.6");
    await expect(shake).toBeEnabled();
    await shake.click();
    await expect(page.locator("#banner-text")).toContainText(
      "The observer shook the ground with 0.60 of a building's weight:",
    );
    await expect(page.locator("#chronicle")).toContainText(
      "One recorded influence: the observer shook the ground with 0.60 of a building's weight:",
    );
  } finally {
    await host.stop();
  }
});
