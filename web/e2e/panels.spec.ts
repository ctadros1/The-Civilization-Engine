// The panels alone (`?view=panels`), as Unreal's WebBrowser widget shows them beside its own
// view of the world (ADR-0005 §6): no map or WebGL, the panels fill a narrow page, and showing a
// place asks the host to look there with a `tce:focus` event.

import { expect, test } from "@playwright/test";

import { startHost, tempSaves } from "./host.js";

test.use({ viewport: { width: 400, height: 900 } });

test("the panels alone work beside a world drawn elsewhere", async ({ page }) => {
  const host = await startHost(tempSaves());
  try {
    page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
    await page.goto(`${host.url}?view=panels`);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");
    await expect(page.locator(".map-pane")).toBeHidden();
    expect(await page.locator("canvas").count()).toBe(0);
    await expect(page.getByRole("heading", { name: "People" })).toBeVisible();

    await page.getByRole("button", { name: "New world" }).click();
    await page.getByLabel("Name").fill("Panel Valley");
    await page.getByLabel("Size").selectOption("512");
    await page.getByLabel("Seed").fill("3");
    await page.getByRole("button", { name: "Create", exact: true }).click();
    await page.waitForFunction(() => window.__TCE__.state().people > 0, undefined, {
      timeout: 120_000,
    });

    // A settlement link asks the host to look at it.
    await page.evaluate(() => {
      window.addEventListener("tce:focus", (e) => {
        (window as unknown as { focused: unknown }).focused = (e as CustomEvent).detail;
      });
    });
    await page.locator("#chronicle").getByRole("button").first().click();
    const focused = await page.waitForFunction(
      () => (window as unknown as { focused?: { x: number; y: number } }).focused ?? null,
    );
    const at = (await focused.jsonValue()) as { x: number; y: number };
    expect(at.x).toBeGreaterThan(0);
    expect(at.y).toBeGreaterThan(0);

    // The host can ask the inspector about someone it picked in its own view.
    const someone = (await page.evaluate(() => window.__TCE__.briefs()))[0]!;
    await page.evaluate((id) => window.__TCE__.select(id), someone.id);
    await page.waitForFunction(() => window.__TCE__.state().selected?.name != null);
    await expect(page.locator(".person-head h3")).toBeVisible();
  } finally {
    await host.stop();
  }
});
