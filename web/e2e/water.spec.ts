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
