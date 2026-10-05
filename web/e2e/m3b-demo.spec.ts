// The M3b demo's pictures (plan §7): one valley lived years from the command line, the observer
// having brought jointed timber framing to one founder as the world began. Then its knowledge
// panel (who came to know framing and how, and any craft found or lost), its chronicle (what gave
// way), and the readout of a building built after an admired one. It takes some time, so it runs
// only with TCE_DEMO=1, and TCE_SHOTS_DIR saves the screenshots. TCE_DEMO_SEED, TCE_DEMO_BAND and
// TCE_DEMO_DAYS change the seed, the band and the days lived. Runs differ (each world draws its
// own identity), so what it logs is one run's; the decision log compares several.

import { expect, test, type Page } from "@playwright/test";

import { makeWorld, startHost, tempSaves } from "./host.js";

test.skip(!process.env.TCE_DEMO, "the M3b demo runs only with TCE_DEMO=1");
test.setTimeout(120 * 60_000);
test.use({ viewport: { width: 1440, height: 900 } });

const shots = process.env.TCE_SHOTS_DIR;
const NAME = "Framing Valley";

/** Zooms the map about its centre to `scale` pixels a metre. */
async function zoomTo(page: Page, scale: number): Promise<void> {
  await page.evaluate((s) => window.__TCE__.zoomBy(s / window.__TCE__.map().camera.scale), scale);
}

test("a craft brought to one founder, what gave way, and homes after an admired one", async ({
  page,
}) => {
  const saves = tempSaves();
  const seed = process.env.TCE_DEMO_SEED ?? "4";
  const band = process.env.TCE_DEMO_BAND ?? "40";
  const days = process.env.TCE_DEMO_DAYS ?? "3650";
  makeWorld(
    saves,
    [
      ...["--seed", seed, "--preset", "core:worldgen/river_valley", "--band", band],
      ...["--days", days, "--introduce", "core:technique/jointed_frame", "--name", NAME],
    ],
    90 * 60_000,
  );
  const host = await startHost(saves);
  try {
    page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");
    await page.getByRole("button", { name: "Load", exact: true }).click();
    await page
      .locator("#load-body tbody tr")
      .filter({ hasText: NAME })
      .getByRole("button", { name: "Load" })
      .click();
    await page.waitForFunction(
      (n) =>
        window.__TCE__.state().world?.name === n &&
        (window.__TCE__.state().knowledge?.length ?? 0) > 0 &&
        window.__TCE__.map().state === "ready" &&
        window.__TCE__.map().buildings > 0,
      NAME,
      { timeout: 300_000 },
    );
    if (!(await page.evaluate(() => window.__TCE__.state().clock?.paused))) {
      await page.getByRole("button", { name: "Pause" }).click();
    }

    // What the village came to know: framing, brought to one founder, and how it went on.
    const state = await page.evaluate(() => window.__TCE__.state());
    const village = state.knowledge![0]!;
    const framing = village.techniques.find((t) => t.id === "core:technique/jointed_frame");
    const kinds = ["found", "taught", "lost", "gave way", "introduced"];
    const chronicle = state.chronicle.filter((t) => kinds.some((k) => t.includes(k)));
    const map = await page.evaluate(() => window.__TCE__.map());
    console.log(
      `demo: ${JSON.stringify({
        people: state.people,
        framing,
        techniques: village.techniques.map((t) => [t.id, t.known, t.knowers, t.history]),
        chronicle,
        buildings: map.buildings,
        frames: map.frames,
        followed: map.followed,
      })}`,
    );
    expect(framing?.history[0]).toMatch(/the observer taught/);

    const panel = page.locator("#knowledge-body");
    const framed = panel.locator("details").filter({ hasText: "Jointed timber framing" });
    await framed.locator("summary").click();
    await expect(framed.locator(".history")).toContainText("the observer taught");
    if (shots) {
      await panel.screenshot({ path: `${shots}/m3b-knowledge.png` });
      await page.locator("#chronicle").screenshot({ path: `${shots}/m3b-chronicle.png` });
    }

    // The village, and a home built after an admired one, if any was: the readout says so.
    await page.locator("#chronicle").getByRole("button").first().click();
    await zoomTo(page, 6);
    await page.waitForTimeout(3000);
    const after = map.followed[0];
    if (after) {
      const cam = (await page.evaluate(() => window.__TCE__.map())).camera;
      const box = await page.locator("#map canvas").boundingBox();
      if (!box) throw new Error("no map canvas");
      await page.evaluate(
        ([x, y]) => window.__TCE__.panBy(x!, y!),
        [box.width / 2 - (cam.x + after.x * cam.scale), box.height / 2 - (cam.y + after.y * cam.scale)],
      );
      await page.waitForTimeout(1000);
      await page.mouse.move(box.x + box.width / 2 + 1, box.y + box.height / 2);
      await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
      await expect(page.locator("#readout")).toContainText(", after ", { timeout: 15_000 });
    }
    if (shots) await page.screenshot({ path: `${shots}/m3b-village.png` });
  } finally {
    await host.stop();
  }
});
