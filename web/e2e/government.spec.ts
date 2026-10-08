// M4a slice Z, end to end: each settlement's polity in the Government panel, with the custom it
// decides by, its common store, and a law's whole history in the kernel's words: who proposed it
// and why, how the gathering decided, where each who came stood and why, and what it asked and
// gave (ADR-0013).

import { expect, test } from "@playwright/test";

import { makeWorld, startHost, tempSaves } from "./host.js";

const shots = process.env.TCE_SHOTS_DIR;
const NAME = "Gathering Valley";

test("a polity, its custom and a law's history in the government panel", async ({ page }) => {
  const saves = tempSaves();
  // River valley seed 2, lived from 1 March to the end of the year, through the lean weeks
  // before the first harvest. A world's tie-breaks come from its identity, drawn afresh each time,
  // so each run is its own world; every run so far had a gathering decide by early September.
  makeWorld(saves, ["--seed", "2", "--size", "768", "--days", "300", "--name", NAME]);
  const host = await startHost(saves);
  try {
    page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");
    await expect(page.locator("#government-body")).toContainText("No world loaded.");
    await page.getByRole("button", { name: "Load", exact: true }).click();
    await page
      .locator("#load-body tbody tr")
      .filter({ hasText: NAME })
      .getByRole("button", { name: "Load" })
      .click();
    await page.waitForFunction(
      (n) =>
        window.__TCE__.state().world?.name === n &&
        (window.__TCE__.state().government?.polities.length ?? 0) > 0,
      NAME,
      { timeout: 120_000 },
    );
    if (!(await page.evaluate(() => window.__TCE__.state().clock?.paused))) {
      await page.getByRole("button", { name: "Pause" }).click();
    }

    const government = (await page.evaluate(() => window.__TCE__.state().government))!;
    console.log(`government: ${JSON.stringify(government)}`);
    const polity = government.polities[0]!;
    const panel = page.locator("#government-body");
    await expect(panel.locator(".polity h3").first()).toHaveText(polity.name);
    await expect(panel.locator(".custom").first()).toContainText("decide by acclamation");
    await expect(panel.locator(".polity .since").first()).toContainText(
      `${polity.members} adults; the common store holds`,
    );

    // A law the gathering decided, opened to its history.
    expect(polity.laws.length).toBeGreaterThan(0);
    const decided = polity.laws.find((l) => l.outcome !== null)!;
    expect(decided).toBeDefined();
    expect(decided.decision).toMatch(/adults came/);
    const law = panel.locator("details.law").filter({ hasText: decided.what }).first();
    await law.locator("summary").click();
    await expect(law.locator(".law-history")).toContainText("Proposed");
    await expect(law.locator(".law-history")).toContainText("because");
    await expect(law.locator(".law-history")).toContainText(decided.decision);
    await expect(law.locator(".stances li")).toHaveCount(decided.stances);
    await expect(law.locator(".stances li").first()).toContainText(/^.+ (for|against|abstained): /);
    if (shots) {
      await page.locator("#government-panel").screenshot({ path: `${shots}/m4a-government.png` });
    }
  } finally {
    await host.stop();
  }
});
