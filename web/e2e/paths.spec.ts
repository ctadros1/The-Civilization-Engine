// M1 slice F, end to end: a village some weeks old, made from the command line, shows the ground
// its people wore and the trails traced through it; the legend explains them and the readout
// names a trail under the pointer.

import { expect, test } from "@playwright/test";

import { makeWorld, startHost, tempSaves } from "./host.js";

const shots = process.env.TCE_SHOTS_DIR;

test("a village shows the trails its people wore", async ({ page }) => {
  const saves = tempSaves();
  makeWorld(saves, ["--seed", "3", "--size", "512", "--days", "40", "--name", "Trodden Valley"]);
  const host = await startHost(saves);
  try {
    page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");
    await page.getByRole("button", { name: "Load", exact: true }).click();
    await page.locator("#load-body tbody tr").first().getByRole("button", { name: "Load" }).click();
    await page.waitForFunction(
      () => window.__TCE__.map().state === "ready" && window.__TCE__.map().trails > 0,
      undefined,
      { timeout: 120_000 },
    );
    expect((await page.evaluate(() => window.__TCE__.map())).wornTiles).toBeGreaterThan(0);
    await expect(page.locator(".legend.paths")).toContainText("Worn ground");
    await expect(page.locator(".legend.paths")).toContainText("Trail");

    // Close in on the settlement and point at a trail.
    await page.locator("#chronicle").getByRole("button").first().click();
    for (let i = 0; i < 6; i++) await page.getByRole("button", { name: "Zoom in" }).click();
    const box = await page.locator("#map canvas").boundingBox();
    if (!box) throw new Error("no map canvas");
    const at = await page.evaluate(
      ({ w, h }) => {
        for (let r = 0; r < Math.min(w, h) / 2; r += 3) {
          for (let a = 0; a < 96; a++) {
            const x = w / 2 + r * Math.cos((a / 96) * 2 * Math.PI);
            const y = h / 2 + r * Math.sin((a / 96) * 2 * Math.PI);
            const info = window.__TCE__.pointerAt(x, y);
            if (info?.path?.trail && !info.building) return [x, y];
          }
        }
        return null;
      },
      { w: box.width, h: box.height },
    );
    expect(at, "a trail near the settlement").not.toBeNull();
    await page.mouse.move(box.x + at![0]!, box.y + at![1]!);
    await expect(page.locator("#readout")).toContainText("a trail");
    if (shots) await page.screenshot({ path: `${shots}/trails.png` });
  } finally {
    await host.stop();
  }
});
