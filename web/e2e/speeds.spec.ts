// M3c slice S, end to end: the Accelerated speeds (ADR-0011). Max lives the world a day at a time,
// its frames standing at midnight; a Detailed speed chosen at a midnight takes effect at once.

import { expect, test } from "@playwright/test";

import { makeWorld, startHost, tempSaves } from "./host.js";

test("Max lives a day at a time and 10x goes back to the minute", async ({ page }) => {
  const saves = tempSaves();
  makeWorld(saves, ["--seed", "3", "--size", "512", "--days", "2", "--name", "Quick Valley"]);
  const host = await startHost(saves);
  try {
    page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");
    await page.getByRole("button", { name: "Load", exact: true }).click();
    await page.locator("#load-body tbody tr").first().getByRole("button", { name: "Load" }).click();
    await page.waitForFunction(() => window.__TCE__.state().clock != null, undefined, {
      timeout: 60_000,
    });
    const start = (await page.evaluate(() => window.__TCE__.state().clock))!;
    await expect(page.locator("#speeds button")).toHaveText(["1×", "3×", "10×", "60×", "600×", "Max"]);

    await page.locator("#speeds").getByRole("radio", { name: "Max" }).click();
    await page.waitForFunction(
      (from) => {
        const c = window.__TCE__.state().clock;
        return !!c && c.minute >= from + 3 * 1440 && c.mode === "accelerated";
      },
      start.minute,
      { timeout: 120_000 },
    );
    // Every Accelerated frame stands at a midnight.
    const daily = (await page.evaluate(() => window.__TCE__.state().clock))!;
    expect(daily.minute % 1440).toBe(0);
    expect([daily.hour, daily.minuteOfHour]).toEqual([0, 0]);
    await expect(page.locator("#speeds").getByRole("radio", { name: "Max" })).toHaveAttribute(
      "aria-checked",
      "true",
    );

    // 10x, chosen at a midnight, is Detailed at once, and the clock runs by the minute again.
    await page.locator("#speeds").getByRole("radio", { name: "10×" }).click();
    await page.waitForFunction(
      () => {
        const c = window.__TCE__.state().clock;
        return !!c && c.mode === "detailed" && c.minute % 1440 !== 0;
      },
      undefined,
      { timeout: 60_000 },
    );
  } finally {
    await host.stop();
  }
});
