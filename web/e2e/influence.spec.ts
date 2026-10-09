// M4c slice AJ, end to end (ADR-0016 §5): the observer tells an adult of an ideology from the
// inspector; they weigh it by what they hold dear, the inspector says what came of it as one
// recorded influence, the chronicle records it, and telling them again adds nothing.

import { expect, test } from "@playwright/test";

import { startHost, tempSaves } from "./host.js";

test("the observer tells someone of an ideology, and a repeat adds nothing", async ({ page }) => {
  const host = await startHost(tempSaves());
  try {
    page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");
    await page.getByRole("button", { name: "New world" }).click();
    await page.getByLabel("Name").fill("Whispering Valley");
    await page.getByLabel("Size").selectOption("512");
    await page.getByLabel("Seed").fill("3");
    await page.getByRole("button", { name: "Create", exact: true }).click();
    await page.waitForFunction(
      () => window.__TCE__.state().world?.name === "Whispering Valley" && !window.__TCE__.state().task,
      undefined,
      { timeout: 180_000 },
    );
    if (!(await page.evaluate(() => window.__TCE__.state().clock?.paused))) {
      await page.getByRole("button", { name: "Pause" }).click();
    }

    // An adult who holds to no ideology.
    const adults = (await page.evaluate(() => window.__TCE__.briefs()))
      .filter((p) => p.ageYears >= 20)
      .map((p) => p.id);
    const inspector = page.locator("#people-body");
    let chosen = 0;
    for (const id of adults) {
      await page.evaluate((x) => window.__TCE__.select(x), id);
      await expect(inspector.locator(".observer-hand")).toBeVisible();
      if ((await inspector.locator(".opinions .ideologies.empty").count()) > 0) {
        chosen = id;
        break;
      }
    }
    expect(chosen).not.toBe(0);
    const name = await inspector.locator(".person-head h3").textContent();
    await expect(inspector.locator(".observer-hand")).toContainText("The observer has not reached them.");

    // The observer tells them of common provision.
    await inspector.getByLabel("Tell of").selectOption({ label: "Common provision" });
    await inspector.getByRole("button", { name: "Tell them" }).click();
    const told = inspector.locator(".observer-hand .influences li");
    await expect(told).toHaveCount(1);
    await expect(told).toContainText("Told of common provision; weighed it once");
    await expect(told).toContainText("one recorded influence");
    await expect(page.locator("#chronicle")).toContainText(
      `One recorded influence: the observer told ${name} of common provision.`,
    );

    // Again: the record is refreshed, nothing is drawn anew.
    await inspector.getByLabel("Tell of").selectOption({ label: "Common provision" });
    await inspector.getByRole("button", { name: "Tell them" }).click();
    await expect(told).toHaveCount(1);
    await expect(told).toContainText("Told of common provision (2 times); weighed it once");

    // A blessing for a year, a quarter of the way.
    await inspector.getByRole("button", { name: "Bless" }).click();
    const lines = inspector.locator(".observer-hand .influences li");
    await expect(lines).toHaveCount(2);
    await expect(lines.first()).toContainText("Blessed until");
    await expect(lines.first()).toContainText("illness or accident at 0.75 of what it was");
    await expect(page.locator("#chronicle")).toContainText(`One recorded influence: the observer blessed ${name} for 365 days`);

    // An agitator, sent with the map tool to the village.
    // The settlement's link, the chronicle's first, centres the map on its hearth.
    await page.locator("#chronicle").getByRole("button").first().click();
    await page.getByLabel("The ideology the agitator holds").selectOption({ label: "Order kept by all" });
    await page.getByRole("button", { name: "Send an agitator" }).click();
    await expect(page.getByRole("button", { name: "Send an agitator" })).toHaveAttribute("aria-pressed", "true");
    const village = await page.evaluate(() => window.__TCE__.state().world?.name ?? "");
    expect(village).toBe("Whispering Valley");
    const box = await page.locator("#map canvas").boundingBox();
    if (!box) throw new Error("no map canvas");
    const people = await page.evaluate(() => window.__TCE__.state().people);
    await page.mouse.click(box.x + box.width / 2 + 25, box.y + box.height / 2 + 15);
    await page.waitForFunction((n) => window.__TCE__.state().people > n, people, { timeout: 30_000 });
    await expect(page.getByRole("button", { name: "Send an agitator" })).toHaveAttribute("aria-pressed", "false");
    await expect(page.locator("#chronicle")).toContainText(/One recorded influence: the observer sent \w+, who holds to order kept by all, to /);
  } finally {
    await host.stop();
  }
});
