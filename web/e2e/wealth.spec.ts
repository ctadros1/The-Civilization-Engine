// M3a slice K, end to end: a new world chooses its land tenure in the new-world dialog and the
// world panel names it; a village world lived past its first year's end shows the wealth panel's
// measures, the year's record and its households, with every field held by the settlement.

import { expect, test } from "@playwright/test";

import { makeWorld, startHost, tempSaves } from "./host.js";

const shots = process.env.TCE_SHOTS_DIR;

test("a world lives under the tenure it chose, and its wealth is measured", async ({ page }) => {
  const saves = tempSaves();
  // 1 March of year 1 to past 1 January of year 2: one year's end recorded.
  makeWorld(saves, [
    "--seed",
    "3",
    "--size",
    "512",
    "--days",
    "320",
    "--regime",
    "core:regime/village",
    "--name",
    "Common Ground",
  ]);
  const host = await startHost(saves);
  try {
    page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");
    const panel = page.locator("#wealth-body");
    await expect(panel).toContainText("No world loaded.");

    // The new-world dialog offers the content's regimes, the default first chosen, with rules.
    await page.getByRole("button", { name: "New world" }).click();
    const tenure = page.getByLabel("Land tenure");
    await expect(tenure).toHaveValue("core:regime/household");
    await expect(page.locator("#nw-regime-rules li").first()).toContainText(
      "A household holds the ground it breaks.",
    );
    await tenure.selectOption({ label: "Village fields" });
    await expect(page.locator("#nw-regime-rules")).toContainText(
      "The settlement holds the ground its households break.",
    );
    await page.getByLabel("Name").fill("Village Test");
    await page.getByLabel("Size").selectOption("256");
    await page.getByLabel("Seed").fill("5");
    await page.getByRole("button", { name: "Create", exact: true }).click();
    await page.waitForFunction(
      () => window.__TCE__.state().world?.name === "Village Test" && !window.__TCE__.state().task,
      undefined,
      { timeout: 180_000 },
    );
    const world = page.locator("#world-body");
    await expect(world.locator("dd").filter({ hasText: "Village fields" })).toBeVisible();
    await expect(world.locator("ul.rules")).toContainText("back to the settlement");
    // As things stand: one settlement, its households, no year ended yet.
    await page.waitForFunction(() => window.__TCE__.state().wealth?.settlements.length === 1);
    await expect(panel).toContainText("as things stand");
    await expect(panel).toContainText("none has ended here yet");

    // The lived village: a year's record, and the village holds every field.
    await page.getByRole("button", { name: "Load", exact: true }).click();
    const row = page.locator("#load-body tbody tr").filter({ hasText: "Common Ground" });
    await row.getByRole("button", { name: "Load" }).click();
    await page.waitForFunction(
      () =>
        window.__TCE__.state().world?.name === "Common Ground" &&
        (window.__TCE__.state().wealth?.settlements[0]?.history.length ?? 0) > 0,
      undefined,
      { timeout: 120_000 },
    );
    const state = await page.evaluate(() => window.__TCE__.state());
    console.log(`wealth: ${JSON.stringify(state.wealth)}`);
    const settlement = state.wealth!.settlements[0]!;
    expect(state.wealth!.regime).toBe("Village fields");
    expect(settlement.history).toEqual([1]);
    expect(settlement.holdingNone).toBe(1);
    expect(settlement.commonHa).toBeGreaterThan(0);
    expect(state.wealthRev).not.toBe(0);

    await expect(panel.locator("h3")).toHaveText(settlement.name);
    const measures = panel.locator("table.measures tbody tr");
    await expect(measures).toHaveCount(4);
    await expect(measures.filter({ hasText: "Land held" })).toContainText("none");
    await expect(panel.locator(".record")).toContainText(
      "No household holds land: the settlement holds",
    );
    const years = panel.locator('h4:has-text("At each year") + .table-scroll tbody tr');
    await expect(years).toHaveCount(1);
    await expect(years.first().locator("td").first()).toHaveText("1");
    const households = panel.locator('h4:has-text("Households") + .table-scroll tbody tr');
    await expect(households.first()).toContainText("household");
    expect(await households.count()).toBe(Math.min(settlement.households, 20));
    if (shots) await page.locator("#wealth-panel").screenshot({ path: `${shots}/wealth.png` });
  } finally {
    await host.stop();
  }
});
