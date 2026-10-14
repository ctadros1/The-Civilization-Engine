import { expect, test } from "@playwright/test";

import { makeWorld, startHost, tempSaves } from "./host.js";

test("Help at a fire shows the kernel's current probe and recorded help, and switches back to Water", async ({ page }) => {
  const saves = tempSaves();
  makeWorld(saves, ["--seed", "3", "--size", "512", "--days", "40", "--name", "Help Valley"]);
  const host = await startHost(saves);
  try {
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");
    await page.getByRole("button", { name: "Load", exact: true }).click();
    await page.locator("#load-body tbody tr").first().getByRole("button", { name: "Load" }).click();
    await page.waitForFunction(() => window.__TCE__.state().waterRev !== 0);
    if (!(await page.evaluate(() => window.__TCE__.state().clock?.paused))) {
      await page.getByRole("button", { name: "Pause" }).click();
    }
    const legend = page.locator("#lens-legend");
    await page.getByLabel("Lens").selectOption("help-now");
    await expect(legend).toContainText("Help at a fire: now");
    await expect(legend).toContainText("one current probe");
    await expect(legend).toContainText("willingness is not forecast");
    const rows = await page.evaluate(() => window.__TCE__.lens().homes);
    expect(rows.length).toBeGreaterThan(0);
    for (const row of rows) {
      expect(row.fireHelp).toBeDefined();
      expect(row.fireHelp!.adultsNow).toBeGreaterThanOrEqual(0);
    }
    await page.getByLabel("Lens").selectOption("help-came");
    await expect(legend).toContainText("Help at a fire: did come");
    await expect(legend).toContainText("unserved exposures remain in the denominator");
    await page.getByLabel("Lens").selectOption("reach");
    await expect(legend).toContainText("Water: could reach");
    await expect(legend).toContainText("Could reach:");
    await page.getByLabel("Lens").selectOption("");
    await expect(legend).toBeHidden();
  } finally {
    await host.stop();
  }
});
