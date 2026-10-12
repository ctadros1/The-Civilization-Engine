// M6a slice AY, end to end: the water people draw (ADR-0021). The map reads the wells, the
// springs flowing today and the places at the water's edge drawn at, with the rivers' flow today;
// the inspector tells where a person's household went for water today.

import { expect, test } from "@playwright/test";

import { makeWorld, startHost, tempSaves } from "./host.js";

const shots = process.env.TCE_SHOTS_DIR;

test("the map reads the water and the inspector tells where a household drew it", async ({ page }) => {
  const saves = tempSaves();
  makeWorld(saves, ["--seed", "3", "--size", "512", "--days", "60", "--name", "Spring Valley"]);
  const host = await startHost(saves);
  try {
    page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");
    await page.getByRole("button", { name: "Load", exact: true }).click();
    await page.locator("#load-body tbody tr").first().getByRole("button", { name: "Load" }).click();
    // Wire 1.61: the snapshot carries the water's revision, and the map reads the water by it.
    await page.waitForFunction(
      () => window.__TCE__.state().waterRev !== 0 && window.__TCE__.map().flow !== null,
      undefined,
      { timeout: 60_000 },
    );
    const water = await page.evaluate(() => window.__TCE__.map());
    expect(water.flow).toBeGreaterThan(0);
    expect(water.springs + water.banks + water.wells.length).toBeGreaterThan(0);
    for (const w of water.wells) expect(w.words).toMatch(/well, (being dug|open since|given up|fell in)/);
    // The inspector: where the household went for water today, what each uses and what it holds.
    const someone = (await page.evaluate(() => window.__TCE__.briefs()))[0]!;
    await page.evaluate((id) => window.__TCE__.select(id), someone.id);
    await page.waitForFunction(() => !!window.__TCE__.state().selected?.water, undefined, {
      timeout: 30_000,
    });
    const words = await page.evaluate(() => window.__TCE__.state().selected?.water ?? "");
    expect(words).toMatch(/^Their household (went for water|has not gone for water yet today)/);
    expect(words).toMatch(/each of them uses \d+ L a day, and it holds \d+ L$/);
    await expect(page.locator("#people-body .water")).toContainText("Their household");
    if (shots) await page.locator("#people-body").screenshot({ path: `${shots}/water-inspector.png` });
  } finally {
    await host.stop();
  }
});

test("the Water lens shows how near each home's water is, and where its people did come", async ({ page }) => {
  const saves = tempSaves();
  makeWorld(saves, ["--seed", "3", "--size", "512", "--days", "40", "--name", "Lens Valley"]);
  const host = await startHost(saves);
  try {
    page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");
    await page.getByRole("button", { name: "Load", exact: true }).click();
    await page.locator("#load-body tbody tr").first().getByRole("button", { name: "Load" }).click();
    await page.waitForFunction(() => window.__TCE__.state().waterRev !== 0, undefined, { timeout: 60_000 });
    if (!(await page.evaluate(() => window.__TCE__.state().clock?.paused))) {
      await page.getByRole("button", { name: "Pause" }).click();
    }
    const legend = page.locator("#lens-legend");
    await expect(legend).toBeHidden();
    // Could reach: every home's walk to the nearest water, the legend naming its scope, period
    // and denominator (15-01 §3.1).
    await page.getByLabel("Lens").selectOption("reach");
    await page.waitForFunction(() => window.__TCE__.lens().homes.length > 0, undefined, { timeout: 30_000 });
    await expect(legend).toContainText("Water: could reach");
    await expect(legend).toContainText("Scope: Every settlement:");
    await expect(legend).toContainText("Period: now, on the paths as last surveyed");
    await expect(legend).toContainText("Per: homes");
    await expect(legend).toContainText("none within 60 minutes' walk");
    const lens = await page.evaluate(() => window.__TCE__.lens());
    expect(lens.summary[0]).toMatch(new RegExp(`^Could reach: of ${lens.homes.length} homes?, `));
    await expect(legend.locator(".lens-counts")).toHaveText(lens.summary[0]!);
    for (const h of lens.homes) expect(h.nearest).not.toBeNull();
    if (shots) await page.locator(".map-pane").screenshot({ path: `${shots}/water-lens-reach.png` });
    // Over a home, the readout says what the lens reads of it.
    const home = lens.homes[0]!;
    const box = await page.locator("#map canvas").boundingBox();
    if (!box) throw new Error("no map canvas");
    const cam = await page.evaluate(() => window.__TCE__.map().camera);
    const over = { x: box.x + cam.x + home.x * cam.scale, y: box.y + cam.y + home.y * cam.scale };
    await page.mouse.move(over.x + 1, over.y);
    await page.mouse.move(over.x, over.y);
    // Homes may lie close together at this zoom: the readout names whichever is nearest.
    await expect(page.locator("#readout")).toContainText("'s household: Could reach: ", { timeout: 15_000 });
    // Did come: where each home's people drew water in the past 30 days, from their own record.
    await page.getByLabel("Lens").selectOption("came");
    await expect(legend).toContainText("Water: did come");
    await expect(legend).toContainText("Period: the past 30 days, today included");
    await expect(legend.locator(".lens-counts")).toHaveText(lens.summary[1]!);
    expect(lens.summary[1]).toMatch(/^Did come: in the past 30 days, of \d+ homes?, /);
    // A household formed today may not have fetched yet; the rest drew somewhere.
    expect(lens.homes.filter((h) => h.drew > 0).length).toBeGreaterThan(lens.homes.length / 2);
    if (shots) await page.locator(".map-pane").screenshot({ path: `${shots}/water-lens-came.png` });
    // No lens: the legend goes with it.
    await page.getByLabel("Lens").selectOption("");
    await expect(legend).toBeHidden();
  } finally {
    await host.stop();
  }
});
