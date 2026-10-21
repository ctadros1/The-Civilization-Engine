// M9b slice DK end to end (ADR-0032 §6): in a world of more than three settlements the observer
// may have those away from the one it chooses lived by household; the settlements list says
// which are, someone of one is shown in a block of their day, the person selected is followed, and
// turning it off brings every settlement back to detail at the next midnight.

import { expect, test } from "@playwright/test";

import { startHost, tempSaves } from "./host.js";

test("settlements away from the chosen one are lived by household while the observer asks", async ({
  page,
}) => {
  const host = await startHost(tempSaves());
  try {
    page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");

    await page.getByRole("button", { name: "New world" }).click();
    await page.getByLabel("Name").fill("Five Camps");
    await page.getByLabel("Size").selectOption("1024");
    await page.getByLabel("Seed").fill("3");
    await page.getByLabel("Founding band").fill("30");
    await page.getByLabel("Neighbouring groups").fill("30, 30, 30, 30");
    await page.getByRole("button", { name: "Create", exact: true }).click();
    await page.waitForFunction(
      () => window.__TCE__.state().settlements.length > 3 && window.__TCE__.map().state === "ready",
      undefined,
      { timeout: 180_000 },
    );
    if (!(await page.evaluate(() => window.__TCE__.state().clock?.paused))) {
      await page.getByRole("button", { name: "Pause" }).click();
    }
    // Off, as every world begins: nothing is lived by household.
    const toggle = page.locator("#resolution-by-focus");
    await expect(toggle).not.toBeChecked();
    expect(
      (await page.evaluate(() => window.__TCE__.state().settlements)).some(
        (s) => s.householdResolved,
      ),
    ).toBe(false);

    // On, the first settlement chosen: the rest are lived by household from the next midnight.
    await toggle.check();
    await page.waitForFunction(() => window.__TCE__.state().clock?.resolutionByFocus === true);
    const names = (await page.evaluate(() => window.__TCE__.state().settlements)).map((s) => s.name);
    const body = page.locator("#people-body");
    await body.getByText(names[0]!, { exact: true }).first().click();
    await page.locator("#speeds").getByRole("radio", { name: "Max" }).click();
    const run = page.getByRole("button", { name: "Run", exact: true });
    if (await run.count()) {
      await run.click();
    }
    await page.waitForFunction(
      () => {
        const s = window.__TCE__.state().settlements;
        return s.length > 3 && !s[0]!.householdResolved && s.slice(1).every((x) => x.householdResolved);
      },
      undefined,
      { timeout: 120_000 },
    );
    await expect(body).toContainText("lived by household");
    await expect(body.getByRole("button", { name: "Hold in detail" }).first()).toBeVisible();

    // Wire 1.94: someone of a settlement lived by household is shown in a block of their day by
    // the clock, and the inspector says so.
    if (!(await page.evaluate(() => window.__TCE__.state().clock?.paused))) {
      await page.getByRole("button", { name: "Pause" }).click();
    }
    const inBlock = await page.evaluate(
      () => window.__TCE__.briefs().find((p) => p.blockUntilMinute > 0)?.id ?? 0,
    );
    expect(inBlock).toBeGreaterThan(0);
    await page.evaluate((x) => window.__TCE__.select(x), inBlock);
    await expect(body).toContainText("their day lived by household");
    // The person selected is followed: their settlement is lived in detail from the next midnight.
    const theirs = await page.waitForFunction(() => window.__TCE__.state().selected?.settlement ?? 0);
    const settlement = (await theirs.jsonValue()) as number;
    await page.getByRole("button", { name: "Run", exact: true }).click();
    await page.waitForFunction(
      (id) => window.__TCE__.state().settlements.find((x) => x.id === id)?.householdResolved === false,
      settlement,
      { timeout: 120_000 },
    );
    await page.evaluate(() => window.__TCE__.select(null));

    // Off: every settlement in detail again from the next midnight.
    await page.locator("#resolution-by-focus").uncheck();
    await page.waitForFunction(
      () =>
        window.__TCE__.state().clock?.resolutionByFocus === false &&
        window.__TCE__.state().settlements.every((x) => !x.householdResolved),
      undefined,
      { timeout: 120_000 },
    );
  } finally {
    await host.stop();
  }
});
