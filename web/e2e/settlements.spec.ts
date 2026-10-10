// M5a slices AK and AM end to end (ADR-0018): a world made with a neighbouring group has two
// settlements, each founded by one of the groups the world began with, and the inspector says
// where a person lives, how they came there, and the other places their household knows; each
// settlement's way of building (M5b slice AR); and how each polity stands toward the other (M5c
// slice AT).

import { expect, test } from "@playwright/test";

import { startHost, tempSaves } from "./host.js";

test("a world made with neighbours has a settlement for each group", async ({ page }) => {
  const host = await startHost(tempSaves());
  try {
    page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");

    await page.getByRole("button", { name: "New world" }).click();
    await page.getByLabel("Name").fill("Two Camps");
    await page.getByLabel("Size").selectOption("1024");
    await page.getByLabel("Seed").fill("3");
    await page.getByLabel("Founding band").fill("30");
    const neighbours = page.getByLabel("Neighbouring groups");
    await neighbours.fill("30, 30, 30");
    await page.getByRole("button", { name: "Create", exact: true }).click();
    await expect(page.locator("#nw-error")).toContainText("At most 2 neighbouring groups");
    await neighbours.fill("30");
    await page.getByLabel("The groups know where the others camped").check();
    await page.getByRole("button", { name: "Create", exact: true }).click();
    await page.waitForFunction(
      () => window.__TCE__.state().people === 60 && window.__TCE__.map().state === "ready",
      undefined,
      { timeout: 180_000 },
    );
    if (!(await page.evaluate(() => window.__TCE__.state().clock?.paused))) {
      await page.getByRole("button", { name: "Pause" }).click();
    }

    const s = await page.evaluate(() => window.__TCE__.state());
    expect(s.settlements).toHaveLength(2);
    const body = page.locator("#people-body");
    await expect(body).toContainText("one of the groups the world began with");
    for (const settlement of s.settlements) {
      expect(settlement.population).toBe(30);
      // Its way of building on the three clocks (wire 1.56, M5b slice AR): how it was founded
      // to build and how its households would build now; nothing is built yet.
      expect(settlement.style).toMatch(/^founded to build \d+° roofs/);
      expect(settlement.style).toContain("households would build");
    }
    await expect(body).toContainText("Building: founded to build");

    const someone = (await page.evaluate(() => window.__TCE__.briefs()))[0]?.id ?? 0;
    await page.evaluate((x) => window.__TCE__.select(x), someone);
    const residence = page.locator("#people-body .residence");
    await expect(residence).toContainText("Where they live");
    await expect(residence).toContainText("came with a founding group");
    const places = page.locator("#people-body .places");
    await expect(places).toContainText("Places their household knows");
    await expect(places).toContainText("came to the valley alongside its founders");

    // Each polity's standing toward the other, from its own people's side (wire 1.57, M5c slice
    // AT, ADR-0020 §1): the groups knew where the others camped, so each knows the other, and
    // nobody yet holds a view of the other's polity.
    await page.evaluate(() => window.__TCE__.select(null));
    await page.locator("#speeds").getByRole("radio", { name: "Max" }).click();
    const run = page.getByRole("button", { name: "Run", exact: true });
    if (await run.count()) {
      await run.click();
    }
    await page.waitForFunction(
      () => {
        const polities = window.__TCE__.state().government?.polities ?? [];
        return polities.length === 2 && polities.every((p) => p.relations.length === 1);
      },
      undefined,
      { timeout: 120_000 },
    );
    if (!(await page.evaluate(() => window.__TCE__.state().clock?.paused))) {
      await page.getByRole("button", { name: "Pause" }).click();
    }
    const government = (await page.evaluate(() => window.__TCE__.state().government))!;
    for (const p of government.polities) {
      expect(p.relations[0]!.label).toBe("known");
      expect(p.relations[0]!.why[0]).toContain("of its households know");
    }
    await expect(page.locator("#government-body .relations").first()).toContainText(
      "Toward its neighbours:",
    );
  } finally {
    await host.stop();
  }
});
