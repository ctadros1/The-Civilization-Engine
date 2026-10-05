// M3b slice M, end to end: a new village's founders bring their techniques, and the knowledge
// panel lists each with who knows it and its history there; the inspector says what someone
// knows, and the observer teaches a young child a technique and tells them of another, which the
// chronicle records.

import { expect, test } from "@playwright/test";

import { startHost, tempSaves } from "./host.js";

const shots = process.env.TCE_SHOTS_DIR;

test("people carry what they know, and the observer can teach them", async ({ page }) => {
  const host = await startHost(tempSaves());
  try {
    page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");
    const panel = page.locator("#knowledge-body");
    await expect(panel).toContainText("No world loaded.");

    await page.getByRole("button", { name: "New world" }).click();
    await page.getByLabel("Name").fill("Knowing Valley");
    await page.getByLabel("Size").selectOption("512");
    await page.getByLabel("Seed").fill("3");
    await page.getByRole("button", { name: "Create", exact: true }).click();
    await page.waitForFunction(
      () =>
        window.__TCE__.state().world?.name === "Knowing Valley" &&
        !window.__TCE__.state().task &&
        (window.__TCE__.state().knowledge?.length ?? 0) === 1,
      undefined,
      { timeout: 180_000 },
    );
    if (!(await page.evaluate(() => window.__TCE__.state().clock?.paused))) {
      await page.getByRole("button", { name: "Pause" }).click();
    }

    // The founders brought every technique of the core content, and each is recorded so.
    const state = await page.evaluate(() => window.__TCE__.state());
    const village = state.knowledge![0]!;
    expect(village.techniques).toHaveLength(10);
    for (const t of village.techniques) {
      expect(t.known).toBe(true);
      expect(t.knowers).toBeGreaterThan(0);
      expect(t.history).toHaveLength(1);
      expect(t.history[0]).toMatch(/^Year 1: brought by /);
    }
    expect(state.knowledgeRev).not.toBe(0);
    await expect(panel.locator("h3")).toHaveText(village.name);
    await expect(panel.locator(".since")).toHaveText("Knows 10 techniques.");
    const emmer = panel.locator("details").filter({ hasText: "Growing emmer" });
    await expect(emmer.locator("summary .status")).toContainText("known by");
    await emmer.locator("summary").click();
    await expect(emmer.locator("dl")).toContainText("Known by");
    await expect(emmer.locator(".history")).toContainText("Year 1: brought by");

    // A child too young for any work knows nothing yet.
    const child = (await page.evaluate(() => window.__TCE__.briefs()))
      .filter((p) => p.ageYears < 6)
      .sort((a, b) => a.ageYears - b.ageYears)[0]!;
    expect(child).toBeTruthy();
    await page.evaluate((id) => window.__TCE__.select(id), child.id);
    const inspector = page.locator("#people-body");
    await expect(inspector.locator(".knows")).toContainText("Nothing yet.");
    const name = await inspector.locator(".person-head h3").textContent();

    // The observer teaches them to grow emmer.
    await inspector.getByLabel("Introduce").selectOption({ label: "Growing emmer" });
    await inspector.getByRole("button", { name: "Teach" }).click();
    await expect(inspector.locator(".knows li.know-known")).toContainText(
      "Growing emmer introduced by the observer",
    );
    await expect(page.locator("#chronicle")).toContainText(
      `The observer taught ${name} growing emmer.`,
    );

    // And tells them of knapping, which they have then only heard of.
    await inspector.getByLabel("Introduce").selectOption({ label: "Knapping sickle blades" });
    await inspector.getByRole("button", { name: "Tell of it" }).click();
    await expect(inspector.locator(".knows li.know-heard")).toContainText(
      "Knapping sickle blades heard of (introduced by the observer)",
    );
    await expect(page.locator("#chronicle")).toContainText(
      `The observer told ${name} of knapping sickle blades.`,
    );
    const selected = (await page.evaluate(() => window.__TCE__.state())).selected!;
    expect(selected.knows).toEqual([
      { id: "core:technique/emmer_growing", state: "known", source: "introduced by the observer" },
      { id: "core:technique/knapping", state: "heard", source: "introduced by the observer" },
    ]);
    // The village panel counts them among those who have heard of knapping.
    const knapping = panel.locator("details").filter({ hasText: "Knapping sickle blades" });
    await knapping.locator("summary").click();
    await expect(knapping.locator("dl")).toContainText(`Heard of${name}`);
    if (shots) {
      await page.locator("#knowledge-panel").screenshot({ path: `${shots}/knowledge.png` });
      await inspector.screenshot({ path: `${shots}/knowledge-inspector.png` });
    }
  } finally {
    await host.stop();
  }
});
