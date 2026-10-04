// M3a slice I, end to end: a village some weeks old, made from the command line, shows its market:
// what its households offer and on what terms, whether it has a money yet, and the trades made.

import { expect, test } from "@playwright/test";

import { makeWorld, startHost, tempSaves } from "./host.js";

const shots = process.env.TCE_SHOTS_DIR;

test("a village shows what its households offer and trade", async ({ page }) => {
  const saves = tempSaves();
  makeWorld(saves, ["--seed", "3", "--size", "512", "--days", "60", "--name", "Market Valley"]);
  const host = await startHost(saves);
  try {
    page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");
    const market = page.locator("#market-body");
    await expect(market).toContainText("No world loaded.");
    await page.getByRole("button", { name: "Load", exact: true }).click();
    await page.locator("#load-body tbody tr").first().getByRole("button", { name: "Load" }).click();
    await page.waitForFunction(() => (window.__TCE__.state().markets?.length ?? 0) > 0, undefined, {
      timeout: 120_000,
    });
    const state = await page.evaluate(() => window.__TCE__.state());
    console.log(`markets: ${JSON.stringify(state.markets)}`);
    expect(state.marketsRev).not.toBe(0);
    const shown = state.markets![0]!;
    expect(shown.offers).toBeGreaterThan(0);

    // The settlement's market: barter or a money, what is offered, and on what terms.
    await expect(market.locator(".market-head")).toContainText(shown.settlement);
    await expect(market.locator(".market-head .badge")).toHaveText(/barter|money: /);
    await expect(market.locator(".market-summary")).not.toBeEmpty();
    await expect(market.locator(".market-goods li").filter({ hasText: "offered by" }).first()).toBeVisible();
    await expect(market.locator(".market-goods").getByText(/^Asking /).first()).toBeVisible();
    await market.getByText(/^Every offer/).click();
    await expect(market.locator(".offers li").first()).toContainText("household");
    if (shown.trades > 0) {
      await expect(market.locator(".trades li").first()).toContainText(" sold ");
    }
    // The settlement's name shows it on the map.
    await market.locator(".market-head").getByRole("button").click();
    if (shots) await page.screenshot({ path: `${shots}/market.png` });
  } finally {
    await host.stop();
  }
});
