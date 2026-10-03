// M1, slices A to E, end to end: a new world begins with a founding band of families; people
// appear on the map, move along their trips while the clock runs, and the inspector says what
// someone is doing and why, what their household has in store and who their family are; the
// chronicle records the arrival; the band marks out fields and begins its huts, and the map shows
// them.

import { expect, test, type Page } from "@playwright/test";

import { startHost, tempSaves } from "./host.js";

const shots = process.env.TCE_SHOTS_DIR;

async function snap(page: Page, name: string): Promise<void> {
  if (shots) await page.screenshot({ path: `${shots}/${name}.png` });
}

const state = (page: Page) => page.evaluate(() => window.__TCE__.state());

test("a founding band lives on the map and explains itself", async ({ page }) => {
  const host = await startHost(tempSaves());
  try {
    page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");

    // The new-world dialog offers the band size, within the content's limits.
    await page.getByRole("button", { name: "New world" }).click();
    const band = page.getByLabel("Founding band");
    await expect(band).toHaveValue("40");
    await band.fill("400");
    await page.getByLabel("Name").fill("Band Valley");
    await page.getByLabel("Size").selectOption("512");
    await page.getByLabel("Seed").fill("3");
    await page.getByRole("button", { name: "Create", exact: true }).click();
    await expect(page.locator("#nw-error")).toContainText("30 to 50 people");
    await band.fill("30");
    await page.getByRole("button", { name: "Create", exact: true }).click();
    await page.waitForFunction(
      () => window.__TCE__.state().people === 30 && window.__TCE__.map().state === "ready",
      undefined,
      { timeout: 180_000 },
    );

    // The settlement, the legend and the chronicle.
    const s = await state(page);
    expect(s.settlements).toHaveLength(1);
    expect(s.settlements[0]?.population).toBe(30);
    await expect(page.locator("#people-body")).toContainText("30 people");
    await expect(page.locator("#people-body .legend.activities")).toContainText("Gather plants");
    await expect(page.locator("#people-body .legend.activities")).toContainText("Gather firewood");
    await expect(page.locator("#people-body")).toContainText(/food for \d[\d,]* days/);
    await expect(page.locator("#chronicle li")).toHaveCount(2);
    await expect(page.locator("#chronicle")).toContainText("A band of 30 people arrived");
    const settlementName = s.settlements[0]?.name ?? "";
    await expect(page.locator("#chronicle")).toContainText(`They made camp at ${settlementName}.`);

    // Following the settlement link centres the map on the camp; then click a person there.
    await page.locator("#chronicle").getByRole("button", { name: settlementName }).click();
    await page.getByRole("button", { name: "Zoom in" }).click();
    await page.getByRole("button", { name: "Zoom in" }).click();
    const canvas = page.locator("#map canvas");
    const box = await canvas.boundingBox();
    if (!box) throw new Error("no map canvas");
    const people = await page.evaluate(() => window.__TCE__.peopleOnScreen());
    const target = people.find(
      (p) => p.sx > 20 && p.sy > 20 && p.sx < box.width - 20 && p.sy < box.height - 20,
    );
    if (!target) throw new Error("nobody on screen");
    await page.mouse.click(box.x + target.sx, box.y + target.sy);
    await page.waitForFunction(() => window.__TCE__.state().selected?.name != null);
    const selected = (await state(page)).selected;
    expect(selected?.doing).toBeTruthy();
    const inspector = page.locator("#people-body");
    await expect(inspector.locator("h3")).toHaveText(selected?.name ?? "");
    await expect(inspector).toContainText("Why");
    await expect(inspector).toContainText("likely");
    await expect(inspector.getByRole("meter", { name: "Hunger" })).toBeVisible();
    await expect(inspector).toContainText("firewood for");
    await expect(inspector).toContainText(/Stores.*Provisions [\d,.]+ kg/);
    await snap(page, "m1-inspector");

    // The band arrives as families: a founding parent's inspector names their partner, in the
    // kernel's words, and their children.
    const parent = await page.evaluate(
      () => window.__TCE__.briefs().find((p) => p.ageYears >= 26 && p.ageYears <= 40)?.id,
    );
    if (parent == null) throw new Error("no founding parent");
    await page.evaluate((id) => window.__TCE__.select(id), parent);
    await page.waitForFunction((id) => window.__TCE__.state().selected?.id === id, parent);
    await expect(inspector.locator(".kin")).toContainText(/Partner of \w+ for (\d+ years|a year)\./);
    await expect(inspector.locator(".kin")).toContainText("partner");

    // Run the clock: the clock moves, decisions accumulate, and walkers move between snapshots.
    const start = (await state(page)).clock?.minute ?? 0;
    await page.getByRole("radio", { name: "10×" }).click();
    await page.waitForFunction((m) => (window.__TCE__.state().clock?.minute ?? 0) > m + 240, start, {
      timeout: 120_000,
    });
    const moved = await page.evaluate(async () => {
      const before = new Map(window.__TCE__.peopleOnScreen().map((p) => [p.id, [p.sx, p.sy]]));
      await new Promise((r) => setTimeout(r, 1500));
      let count = 0;
      for (const p of window.__TCE__.peopleOnScreen()) {
        const b = before.get(p.id);
        if (b && Math.hypot(p.sx - b[0]!, p.sy - b[1]!) > 0.5) count++;
      }
      return count;
    });
    expect(moved).toBeGreaterThan(0);
    await expect(inspector).toContainText("Why");
    // Arriving in March, they mark out fields and start breaking ground; the map shows them.
    await page.waitForFunction(() => window.__TCE__.map().fields > 0, undefined, {
      timeout: 120_000,
    });
    expect((await state(page)).fieldsRev).toBeGreaterThan(0);
    // Households claim ground for their homes and begin their huts.
    await page.waitForFunction(() => window.__TCE__.map().buildings > 0, undefined, {
      timeout: 120_000,
    });
    expect((await state(page)).buildingsRev).toBeGreaterThan(0);
    await snap(page, "m1-running");
    await page.getByRole("button", { name: "Pause" }).click();

    // Closing the inspector returns to the summary, with the field legend.
    await page.getByRole("button", { name: "Close the inspector" }).click();
    await expect(page.locator("#people-body")).toContainText("Click a person on the map");
    await expect(page.locator("#people-body .legend.fields")).toContainText("new ground");
    await expect(page.locator("#people-body .legend.huts")).toContainText("thatched hut");
    expect((await state(page)).lastError).toBeNull();
  } finally {
    await host.stop();
  }
});
