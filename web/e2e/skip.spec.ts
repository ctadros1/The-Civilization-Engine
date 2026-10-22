// M9c slice DL end to end (ADR-0032 §7): the observer skips ahead a month, at the fastest speed
// with every settlement lived by household, and the speed the clock had is put back where it
// stops; marked as stopping at a new settlement, a skip that meets none runs to its date.

import { expect, test } from "@playwright/test";

import { startHost, tempSaves } from "./host.js";

const state = (page: import("@playwright/test").Page) =>
  page.evaluate(() => window.__TCE__.state());

test("the observer skips ahead a month and the clock's speed is put back", async ({ page }) => {
  const host = await startHost(tempSaves());
  try {
    page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");
    await page.getByRole("button", { name: "New world" }).click();
    await page.getByLabel("Name").fill("Skipping Valley");
    await page.getByLabel("Size").selectOption("512");
    await page.getByLabel("Seed").fill("3");
    await page.getByRole("button", { name: "Create", exact: true }).click();
    await page.waitForFunction(
      () => window.__TCE__.state().people > 0 && window.__TCE__.map().state === "ready",
      undefined,
      { timeout: 120_000 },
    );
    if (!(await page.evaluate(() => window.__TCE__.state().clock?.paused))) {
      await page.getByRole("button", { name: "Pause" }).click();
    }

    // The stop choice waits on the skip box.
    const stopAt = page.getByLabel("Stop a skip at");
    await expect(stopAt).toBeDisabled();
    await page.locator("#skip-ahead").check();
    await expect(stopAt).toBeEnabled();
    await stopAt.selectOption({ label: "or at a new settlement" });

    const before = (await state(page)).clock!;
    await page.getByLabel("Run ahead").selectOption({ label: "a month" });
    await page.waitForFunction(
      (m) => {
        const s = window.__TCE__.state();
        return !s.task && (s.clock?.minute ?? 0) >= m;
      },
      before.minute + 43_200,
      { timeout: 180_000 },
    );
    // At the fastest speed the clock stops at midnights: the first at or after the month.
    const after = (await state(page)).clock!;
    expect(after.minute).toBe(Math.ceil((before.minute + 43_200) / 1440) * 1440);
    expect(after.paused).toBe(true);
    expect(after.speed).toBe(before.speed);
    await expect(page.locator("#events")).toContainText("Skipped to");
  } finally {
    await host.stop();
  }
});
