// M1 slice G, end to end: the observer runs ahead a day in full detail (a task that pauses the
// clock where it arrives), then sends a family to the village with the map tool; the chronicle
// names it as the observer's doing.

import { expect, test } from "@playwright/test";

import { startHost, tempSaves } from "./host.js";

const state = (page: import("@playwright/test").Page) =>
  page.evaluate(() => window.__TCE__.state());

test("the observer runs ahead and sends a family", async ({ page }) => {
  const host = await startHost(tempSaves());
  try {
    page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");
    await page.getByRole("button", { name: "New world" }).click();
    await page.getByLabel("Name").fill("Godly Valley");
    await page.getByLabel("Size").selectOption("512");
    await page.getByLabel("Seed").fill("3");
    await page.getByRole("button", { name: "Create", exact: true }).click();
    await page.waitForFunction(
      () => window.__TCE__.state().people > 0 && window.__TCE__.map().state === "ready",
      undefined,
      { timeout: 120_000 },
    );

    // Run ahead a day: the clock moves on by a day and waits there.
    const before = (await state(page)).clock!;
    await page.getByLabel("Run ahead").selectOption({ label: "a day" });
    await page.waitForFunction(
      (m) => {
        const s = window.__TCE__.state();
        return !s.task && (s.clock?.minute ?? 0) >= m;
      },
      before.minute + 1440,
      { timeout: 120_000 },
    );
    const after = (await state(page)).clock!;
    expect(after.minute).toBe(before.minute + 1440);
    expect(after.paused).toBe(true);
    await expect(page.locator("#events")).toContainText("Ran ahead to");

    // Send a family to the village with the map tool.
    const people = (await state(page)).people;
    await page.locator("#chronicle").getByRole("button").first().click();
    await page.getByRole("button", { name: "Add a family" }).click();
    await expect(page.getByRole("button", { name: "Add a family" })).toHaveAttribute(
      "aria-pressed",
      "true",
    );
    const box = await page.locator("#map canvas").boundingBox();
    if (!box) throw new Error("no map canvas");
    // A little off the hearth, which the settlement link centred.
    await page.mouse.click(box.x + box.width / 2 + 25, box.y + box.height / 2 + 15);
    await page.waitForFunction((n) => window.__TCE__.state().people > n, people, {
      timeout: 30_000,
    });
    await expect(page.getByRole("button", { name: "Add a family" })).toHaveAttribute(
      "aria-pressed",
      "false",
    );
    await page.waitForFunction(
      () => window.__TCE__.state().chronicle.some((c) => c.includes("sent by the observer")),
      undefined,
      { timeout: 30_000 },
    );
  } finally {
    await host.stop();
  }
});
