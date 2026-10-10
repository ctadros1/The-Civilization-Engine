// M5b slice AP, step two, end to end (ADR-0019 §1): a household knows another settlement's
// offers only by what its people saw or were told, and the inspector says what it holds, of
// which seller, on what terms, and since when and from whom. And slice AQ, step three (ADR-0019
// §7): each market says how its asks stand against the other settlement's, month by month.

import { expect, test } from "@playwright/test";

import { makeWorld, startHost, tempSaves } from "./host.js";

const NAME = "Prices Heard";

test("the inspector lists the prices a household has heard of in another settlement", async ({ page }) => {
  const saves = tempSaves();
  // River valley seed 1, two groups of 30 that know where the other camped, lived 100 days. The
  // first visit between them comes in the third month (in every world identity tried), and the
  // visitor's companions at the hearth tell what their households offer.
  makeWorld(saves, [
    ...["--seed", "1", "--size", "768", "--band", "30", "--neighbour", "30", "--known"],
    ...["--days", "100", "--name", NAME],
  ]);
  const host = await startHost(saves);
  try {
    page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");
    await page.getByRole("button", { name: "Load", exact: true }).click();
    await page
      .locator("#load-body tbody tr")
      .filter({ hasText: NAME })
      .getByRole("button", { name: "Load" })
      .click();
    await page.waitForFunction(
      (n) => window.__TCE__.state().world?.name === n && (window.__TCE__.state().settlements?.length ?? 0) === 2,
      NAME,
      { timeout: 120_000 },
    );
    if (!(await page.evaluate(() => window.__TCE__.state().clock?.paused))) {
      await page.getByRole("button", { name: "Pause" }).click();
    }

    // Someone whose household has heard of prices elsewhere.
    const ids = (await page.evaluate(() => window.__TCE__.briefs())).map((b) => b.id);
    let heard: string[] = [];
    for (const id of ids) {
      await page.evaluate((x) => window.__TCE__.select(x), id);
      await page.waitForFunction(
        (x) => {
          const s = window.__TCE__.state().selected;
          return s?.id === x && s.name !== null && s.reports !== null;
        },
        id,
      );
      heard = await page.evaluate(() => window.__TCE__.state().selected?.reports ?? []);
      if (heard.length > 0) break;
    }
    expect(heard.length).toBeGreaterThan(0);
    const names = (await page.evaluate(() => window.__TCE__.state().settlements ?? [])).map((s) => s.name);
    for (const line of heard) {
      // "At Elmhollow, Bran's household: a sickle for 2.3 kg of grain, 8.5 sickles to be had;
      // told by Ada 12 days ago"
      expect(line).toMatch(/^At [^,]+, .+: .+; (seen|told by .+) (today|yesterday|\d+ days ago)$/);
      expect(names.some((n) => line.startsWith(`At ${n}, `))).toBe(true);
    }
    const block = page.locator("#people-body .reports");
    await expect(block).toContainText("Prices their household has heard of elsewhere");
    await expect(block).toContainText(heard[0]!);

    // M5b slice AQ, step three (wire 1.55): each market's trade with the other settlement, from
    // the monthly convergence record, in the market panel.
    await page.waitForFunction(
      () => (window.__TCE__.state().markets ?? []).some((m) => m.between.length > 0),
      undefined,
      { timeout: 60_000 },
    );
    const markets = await page.evaluate(() => window.__TCE__.state().markets ?? []);
    console.log(`between: ${JSON.stringify(markets.map((m) => [m.settlement, m.between, m.onTheWay]))}`);
    for (const m of markets) {
      for (const line of m.between) {
        // "With Elmhollow last month: a sickle 4.2 hours here and 3.1 there (30 points apart)"
        expect(line).toMatch(/^With .+ (last month|this month|in month \d+ of year \d+): .+$/);
        expect(names.some((n) => n !== m.settlement && line.startsWith(`With ${n} `))).toBe(true);
      }
    }
    await expect(page.locator("#market-body .market-between li").first()).toBeVisible();
  } finally {
    await host.stop();
  }
});
