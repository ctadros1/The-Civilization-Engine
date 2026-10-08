// M4a slice Y, end to end: people who keep company at the hearth come to know one another, and
// the inspector lists whom someone knows best and why, in the kernel's words; on the first of
// each month the standing panel sums what each settlement's adults think of one another and names
// its notables (ADR-0014); and the inspector says what they hold against whom and what they
// have heard (ADR-0016).

import { expect, test } from "@playwright/test";

import { makeWorld, startHost, tempSaves } from "./host.js";

const shots = process.env.TCE_SHOTS_DIR;
const NAME = "Standing Valley";

test("ties in the inspector and standing in its panel", async ({ page }) => {
  const saves = tempSaves();
  // From 1 March to mid April: a month of company, and standing worked out on 1 April.
  makeWorld(saves, ["--seed", "3", "--size", "512", "--days", "45", "--name", NAME]);
  const host = await startHost(saves);
  try {
    page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");
    await expect(page.locator("#standing-body")).toContainText("No world loaded.");
    await page.getByRole("button", { name: "Load", exact: true }).click();
    await page
      .locator("#load-body tbody tr")
      .filter({ hasText: NAME })
      .getByRole("button", { name: "Load" })
      .click();
    await page.waitForFunction(
      (n) =>
        window.__TCE__.state().world?.name === n &&
        (window.__TCE__.state().standing?.minute ?? 0) > 0,
      NAME,
      { timeout: 120_000 },
    );
    if (!(await page.evaluate(() => window.__TCE__.state().clock?.paused))) {
      await page.getByRole("button", { name: "Pause" }).click();
    }

    // The panel: the settlement's adults, with notables named, and the ties kept.
    const standing = (await page.evaluate(() => window.__TCE__.state().standing))!;
    expect(standing.ties).toBeGreaterThan(0);
    const village = standing.settlements[0]!;
    expect(village.adults).toBeGreaterThan(0);
    expect(village.notables.length).toBeGreaterThan(0);
    const panel = page.locator("#standing-body");
    await expect(panel.locator("h3")).toHaveText(village.name);
    await expect(panel.locator(".since")).toContainText(`${village.adults} adults; notables:`);
    await expect(panel.locator(".standing-rows li.notable").first()).toContainText("a notable");
    await expect(panel).toContainText("ties kept across the world");

    // The inspector: an adult knows people from other households, and says why.
    const adult = (await page.evaluate(() => window.__TCE__.briefs()))
      .filter((p) => p.ageYears >= 20)
      .sort((a, b) => a.id - b.id)[0]!;
    await page.evaluate((id) => window.__TCE__.select(id), adult.id);
    const ties = page.locator("#people-body .ties");
    await expect(ties.locator("h4")).toHaveText("Knows people");
    await expect(ties).toContainText("Standing:");
    await expect(ties.locator("li").first()).toContainText("last in");
    // And what they hold against whom and what they have heard (wire 1.34), even when it is
    // nothing yet.
    const word = page.locator("#people-body .word");
    await expect(word.locator("h4")).toHaveText("Grievances and news");
    await expect(word).toContainText(/Holds no grievance\.|Holds a grievance against/);
    await expect(word).toContainText(/Has heard no news\.|\((told by|they knew it first)/);
    console.log(
      `standing: ${JSON.stringify({
        village,
        ties: await ties.locator("li").allInnerTexts(),
        word: await word.innerText(),
      })}`,
    );
    if (shots) {
      await page.locator("#standing-panel").screenshot({ path: `${shots}/m4a-standing.png` });
      await page.locator("#people-body").screenshot({ path: `${shots}/m4a-ties.png` });
    }
  } finally {
    await host.stop();
  }
});
