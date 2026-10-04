// M3a slice J, end to end: a village with a workshop at work (made by civ-sim's workshop_world
// example) lists it in the workshops panel; its page shows what it holds, its wage, its monthly
// statements and its books, and the chronicle links to it.

import { expect, test } from "@playwright/test";

import { makeWorkshopWorld, startHost, tempSaves } from "./host.js";

const shots = process.env.TCE_SHOTS_DIR;

test("a workshop shows its books, its wage and what it sold", async ({ page }) => {
  const saves = tempSaves();
  makeWorkshopWorld(saves);
  const host = await startHost(saves);
  try {
    page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");
    const panel = page.locator("#firms-body");
    await expect(panel).toContainText("No world loaded.");
    await page.getByRole("button", { name: "Load", exact: true }).click();
    await page.locator("#load-body tbody tr").first().getByRole("button", { name: "Load" }).click();
    await page.waitForFunction(() => (window.__TCE__.state().firms?.length ?? 0) > 0, undefined, {
      timeout: 120_000,
    });
    const state = await page.evaluate(() => window.__TCE__.state());
    console.log(`workshops: ${JSON.stringify(state.firms)}`);
    expect(state.firmsRev).not.toBe(0);
    // Neighbours may set up sickle workshops of their own; the one the example waited for has
    // sold a sickle and posted a wage.
    const shown = state.firms!.find(
      (f) => f.open && f.name.endsWith("sickle workshop") && / and sold (?!none)/.test(f.record),
    );
    expect(shown, "an open sickle workshop that has sold").toBeTruthy();

    // The list: its name, open, what it makes and its record.
    const item = panel.locator(".firms > li").filter({ hasText: shown!.name });
    await expect(item.locator(".badge")).toHaveText("open");
    await expect(item).toContainText("makes sickles");
    await expect(item.locator(".record")).toHaveText(/^Made .+ and sold /);

    // Its page: holdings, wage, statements and books.
    await item.getByRole("button", { name: shown!.name }).click();
    await page.waitForFunction(() => window.__TCE__.state().firm?.name != null);
    await expect(panel.locator("h3.firm-head")).toContainText(shown!.name);
    const facts = panel.locator("dl.facts");
    await expect(facts).toContainText("Holds");
    await expect(facts.locator("dd").filter({ hasText: /^Pays / })).toBeVisible();
    await expect(panel.locator("table.statements tbody tr").first()).toBeVisible();
    await expect(panel.locator(".books li").filter({ hasText: /Sold .+ to / }).first()).toBeVisible();
    if (shots) await page.screenshot({ path: `${shots}/workshop.png` });

    // Back to the list, and to the page again from the chronicle.
    await panel.getByRole("button", { name: "All workshops" }).click();
    await expect(panel.locator(".firms")).toBeVisible();
    const opened = page.locator("#chronicle button").filter({ hasText: /sickle workshop$/ }).first();
    await opened.click();
    await page.waitForFunction(() => window.__TCE__.state().firm?.name != null);
    await expect(panel.locator("h3.firm-head")).toContainText("sickle workshop");
  } finally {
    await host.stop();
  }
});
