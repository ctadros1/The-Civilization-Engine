// M6b slice BC, end to end (ADR-0022 §13, §14): the observer sets a hut alight with the map tool.
// The host answers in words, the map draws the hut burning and then burnt out, the chronicle
// records the observer's hand and the fire, and the Incidents panel lists the fire with its page.
// What the fire does after it catches is the fire's: nothing here asks it to spread.

import { expect, test } from "@playwright/test";

import { makeWorld, startHost, tempSaves } from "./host.js";

const shots = process.env.TCE_SHOTS_DIR;

test("the observer sets a hut alight, and the map, chronicle and Incidents panel tell of it", async ({ page }) => {
  const saves = tempSaves();
  makeWorld(saves, ["--seed", "3", "--size", "512", "--days", "60", "--name", "Burning Valley"]);
  const host = await startHost(saves);
  try {
    page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");
    await page.getByRole("button", { name: "Load", exact: true }).click();
    await page.locator("#load-body tbody tr").first().getByRole("button", { name: "Load" }).click();
    await page.waitForFunction(
      () => window.__TCE__.map().state === "ready" && window.__TCE__.map().roofed > 0,
      undefined,
      { timeout: 120_000 },
    );
    if (!(await page.evaluate(() => window.__TCE__.state().clock?.paused))) {
      await page.getByRole("button", { name: "Pause" }).click();
    }
    // A roofed hut, centred and near enough to click.
    const roof = await page.evaluate(() => window.__TCE__.map().roofs.find((r) => r.fire === "none") ?? null);
    if (!roof) throw new Error("no roofed building");
    await page.evaluate(({ x, y }) => window.__TCE__.centreOn(x, y, 4), roof);
    const camera = () => page.evaluate(() => window.__TCE__.map().camera);
    let last = await camera();
    for (let i = 0; i < 50; i++) {
      await page.waitForTimeout(200);
      const now = await camera();
      if (now.x === last.x && now.y === last.y && now.scale === last.scale) break;
      last = now;
    }
    const tool = page.getByRole("button", { name: "Set alight" });
    await expect(tool).toBeEnabled();
    await tool.click();
    await expect(tool).toHaveAttribute("aria-pressed", "true");
    // Arming the tool may move the canvas, so find it, and the hut on it, only now.
    await page.waitForTimeout(300);
    const box = await page.locator("#map canvas").boundingBox();
    if (!box) throw new Error("no map canvas");
    const cam = await camera();
    const at = { x: cam.x + roof.x * cam.scale, y: cam.y + roof.y * cam.scale };
    const under = await page.evaluate(({ x, y }) => window.__TCE__.pointerAt(x, y)?.building?.id ?? null, at);
    expect(under).toBe(roof.id);
    await page.mouse.click(box.x + at.x, box.y + at.y);
    await expect(page.locator("#banner-text")).toContainText("The observer set alight the hut of");
    await expect(tool).toHaveAttribute("aria-pressed", "false");
    // The map draws it burning, and the readout says how.
    await page.waitForFunction(
      (id) => window.__TCE__.map().roofs.some((r) => r.id === id && r.fire !== "none"),
      roof.id,
      { timeout: 30_000 },
    );
    await expect(page.locator("#chronicle")).toContainText(
      "One recorded influence: the observer set alight the hut of",
    );
    if (shots) await page.locator("#map").screenshot({ path: `${shots}/fire-burning.png` });

    // The Incidents panel lists the fire as the observer's, and its page tells its stages.
    const panel = page.locator("#incidents-body");
    await panel.getByLabel("Kind of incident").selectOption({ label: "Fires" });
    await expect(panel.locator(".incident-totals")).toContainText("Fires: ");
    const row = panel.locator("li.incident.fire").first();
    await expect(row).toContainText("set alight by the observer");
    await expect(row).toContainText("(the observer's)");
    await row.getByRole("button", { name: "Open" }).click();
    await expect(panel.locator(".incident-steps")).toContainText("Caught:");
    await panel.getByRole("button", { name: "Back to the incidents" }).click();

    // A day on, it has burnt out: the map shows it charred and says when.
    await page.getByLabel("Run ahead").selectOption({ label: "a day" });
    await page.waitForFunction(
      (id) => window.__TCE__.map().roofs.some((r) => r.id === id && r.fire === "burnt"),
      roof.id,
      { timeout: 120_000 },
    );
    const words = await page.evaluate(
      (id) => window.__TCE__.map().roofs.find((r) => r.id === id)?.words ?? "",
      roof.id,
    );
    expect(words).toMatch(/^burnt out at \d\d:\d\d on /);
    // Alone, or as the first of fires it spread to: the chronicle tells which.
    await expect(page.locator("#chronicle")).toContainText(/Fire burned out the hut of|The fire that began at .* is out/);
  } finally {
    await host.stop();
  }
});
