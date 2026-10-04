// M3b slice Q, end to end: the observer lays down clay showing at the surface by the village with
// the map tool; the banner and the chronicle say so, and the map's readout describes it.

import { expect, test } from "@playwright/test";

import { startHost, tempSaves } from "./host.js";

test("the observer lays down a deposit and the map describes it", async ({ page }) => {
  const host = await startHost(tempSaves());
  try {
    page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");
    await page.getByRole("button", { name: "New world" }).click();
    await page.getByLabel("Name").fill("Clay Valley");
    await page.getByLabel("Size").selectOption("512");
    await page.getByLabel("Seed").fill("3");
    await page.getByRole("button", { name: "Create", exact: true }).click();
    await page.waitForFunction(
      () => window.__TCE__.state().people > 0 && window.__TCE__.map().state === "ready",
      undefined,
      { timeout: 120_000 },
    );
    // Centre the map on the village, as its chronicle link does, and let the camera settle.
    await page.locator("#chronicle").getByRole("button").first().click();
    const camera = () => page.evaluate(() => window.__TCE__.map().camera);
    let last = await camera();
    for (let i = 0; i < 50; i++) {
      await page.waitForTimeout(200);
      const now = await camera();
      if (now.x === last.x && now.y === last.y && now.scale === last.scale) break;
      last = now;
    }
    const rev = (await page.evaluate(() => window.__TCE__.state())).depositsRev;
    const drawn = (await page.evaluate(() => window.__TCE__.map())).deposits;

    await page.getByLabel("What the deposit is of").selectOption({ label: "Clay" });
    const lay = page.getByRole("button", { name: "Lay down a deposit" });
    await lay.click();
    await expect(lay).toHaveAttribute("aria-pressed", "true");
    const box = await page.locator("#map canvas").boundingBox();
    if (!box) throw new Error("no map canvas");
    const at = { x: box.x + box.width / 2 + 30, y: box.y + box.height / 2 + 20 };
    // Where that is on the ground, metres, so it can be found again wherever the camera goes.
    const cam = await camera();
    const ground = { x: (at.x - box.x - cam.x) / cam.scale, y: (at.y - box.y - cam.y) / cam.scale };
    await page.mouse.click(at.x, at.y);
    await expect(page.locator("#banner-text")).toContainText("The observer laid down clay showing at the surface");
    await expect(lay).toHaveAttribute("aria-pressed", "false");
    await page.waitForFunction((r) => window.__TCE__.state().depositsRev !== r, rev, { timeout: 30_000 });
    await page.waitForFunction(
      () => window.__TCE__.state().chronicle.some((c) => c.includes("The observer laid down clay")),
      undefined,
      { timeout: 30_000 },
    );
    // The map draws it beside the deposits the world began with.
    await page.waitForFunction((n) => window.__TCE__.map().deposits === n + 1, drawn, { timeout: 30_000 });
    // Over the clay, the readout says what it is and how much there is. The notice above the map
    // may have moved the canvas, so find it again.
    const now = await camera();
    const moved = await page.locator("#map canvas").boundingBox();
    if (!moved) throw new Error("no map canvas");
    const over = { x: moved.x + now.x + ground.x * now.scale, y: moved.y + now.y + ground.y * now.scale };
    await page.mouse.move(over.x + 1, over.y);
    await page.mouse.move(over.x, over.y);
    await expect(page.locator("#readout")).toContainText("clay, 20 m across, showing at the surface", {
      timeout: 15_000,
    });
  } finally {
    await host.stop();
  }
});
