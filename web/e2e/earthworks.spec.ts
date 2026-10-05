// M3b slice Q, end to end: in a coast world whose founders level their sloping plots on their first
// day, the map draws the platforms, reads the changed ground again, and the readout says what
// each platform is.

import { expect, test } from "@playwright/test";

import { startHost, tempSaves } from "./host.js";

test("the map draws levelled plots and the readout describes them", async ({ page }) => {
  const host = await startHost(tempSaves());
  try {
    page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");
    await page.getByRole("button", { name: "New world" }).click();
    await page.getByLabel("Name").fill("Terrace Cove");
    await page.getByLabel("Landscape").selectOption("core:worldgen/ria_coast");
    await page.getByLabel("Size").selectOption("512");
    await page.getByLabel("Seed").fill("7");
    await page.getByRole("button", { name: "Create", exact: true }).click();
    await page.waitForFunction(
      () => window.__TCE__.state().people > 0 && window.__TCE__.map().state === "ready",
      undefined,
      { timeout: 120_000 },
    );
    // Run ahead a day: the founders choose their plots and level the sloping ones first.
    await page.getByLabel("Run ahead").selectOption({ label: "a day" });
    await page.waitForFunction(
      () => window.__TCE__.state().earthworksRev !== 0 && window.__TCE__.map().earthworks > 0,
      undefined,
      { timeout: 120_000 },
    );
    expect((await page.evaluate(() => window.__TCE__.map())).groundTiles).toBeGreaterThan(0);

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
    // Over a platform, the readout says whose plot it is and how far it is levelled.
    const platform = (await page.evaluate(() => window.__TCE__.map())).platforms[0]!;
    const cam = await camera();
    const box = await page.locator("#map canvas").boundingBox();
    if (!box) throw new Error("no map canvas");
    const over = { x: box.x + cam.x + platform.x * cam.scale, y: box.y + cam.y + platform.y * cam.scale };
    await page.mouse.move(over.x + 1, over.y);
    await page.mouse.move(over.x, over.y);
    await expect(page.locator("#readout")).toContainText(/the plot of .*, (to be levelled|being levelled|levelled)/, {
      timeout: 15_000,
    });
    // The building on it is built to its household's taste (M3b slice R), and the readout says so.
    const style = await page.evaluate(
      ([x, y]) => window.__TCE__.pointerAt(x!, y!)?.building?.style,
      [cam.x + platform.x * cam.scale, cam.y + platform.y * cam.scale],
    );
    expect(style).toMatch(/^roof pitched \d+°/);
    await expect(page.locator("#readout")).toContainText(/roof pitched \d+°/);
  } finally {
    await host.stop();
  }
});
