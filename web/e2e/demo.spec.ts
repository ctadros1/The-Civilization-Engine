// The M1 demo (plan §7): watch a band found a village over ten years, then save and reload.
// It takes several minutes, so it runs only with TCE_DEMO=1; with TCE_DEMO_VIDEO=1 as well it
// records the video into test-results/. TCE_DEMO_SEED picks the seed (runs differ, and a band
// may fail; that is the model, not the demo). It logs `demo-mark` lines with the seconds since
// the test began, for cutting the video.

import { expect, test, type Page } from "@playwright/test";

import { startHost, tempSaves } from "./host.js";

test.skip(!process.env.TCE_DEMO, "the M1 demo runs only with TCE_DEMO=1");
test.setTimeout(60 * 60_000);
test.use({
  viewport: { width: 1280, height: 800 },
  video: process.env.TCE_DEMO_VIDEO ? { mode: "on", size: { width: 1280, height: 800 } } : "off",
});

const LONG = { timeout: 30 * 60_000 };

const state = (page: Page) => page.evaluate(() => window.__TCE__.state());

/** Runs ahead by one of the menu's spans and waits until the world is paused there. */
async function runAhead(page: Page, label: string, minutes: number): Promise<void> {
  const from = (await state(page)).clock!.minute;
  await page.getByLabel("Run ahead").selectOption({ label });
  await page.waitForFunction(
    (m) => {
      const s = window.__TCE__.state();
      return !s.task && (s.clock?.minute ?? 0) >= m;
    },
    from + minutes,
    LONG,
  );
  await page.waitForTimeout(2500);
}

/** Zooms the map about its centre to `scale` pixels a metre. */
async function zoomTo(page: Page, scale: number): Promise<void> {
  await page.evaluate((s) => window.__TCE__.zoomBy(s / window.__TCE__.map().camera.scale), scale);
}

test("a band founds a village over ten years, then the world is saved and loaded", async ({
  page,
}) => {
  const began = Date.now();
  const mark = (what: string) =>
    console.log(`demo-mark ${((Date.now() - began) / 1000).toFixed(1)} ${what}`);
  const host = await startHost(tempSaves());
  try {
    page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");
    await page.getByRole("button", { name: "New world" }).click();
    await page.getByLabel("Name").fill("Ten Years");
    await page.getByLabel("Size").selectOption("1024");
    await page.getByLabel("Seed").fill(process.env.TCE_DEMO_SEED ?? "2");
    await page.getByRole("button", { name: "Create", exact: true }).click();
    await page.waitForFunction(
      () => window.__TCE__.state().people > 0 && window.__TCE__.map().state === "ready",
      undefined,
      { timeout: 180_000 },
    );
    await page.waitForTimeout(1500);

    // Go to the camp and watch the first morning at 10×.
    await page.locator("#chronicle").getByRole("button").first().click();
    await zoomTo(page, 2.4);
    await page.getByRole("radio", { name: "10×" }).click();
    await page.waitForTimeout(8000);
    await page.getByRole("button", { name: "Pause" }).click();

    // The first month: fields are marked out and sown, huts begun.
    await runAhead(page, "a month", 30 * 1440);
    await zoomTo(page, 1.2);
    await page.waitForTimeout(1000);

    // The first year, and someone's day.
    await runAhead(page, "a year", 365 * 1440);
    const first = (await page.evaluate(() => window.__TCE__.briefs()))[0];
    if (first) {
      await page.evaluate((id) => window.__TCE__.select(id), first.id);
      await page.waitForTimeout(4000);
      await page.evaluate(() => window.__TCE__.select(null));
    }

    // The observer sends a family; it joins the village.
    const people = (await state(page)).people;
    await page.getByRole("button", { name: "Add a family" }).click();
    const box = await page.locator("#map canvas").boundingBox();
    if (!box) throw new Error("no map canvas");
    // Open dry ground a little way from the hearth.
    const at = await page.evaluate(
      ({ w, h }) => {
        for (let r = 60; r < Math.min(w, h) / 2; r += 8) {
          for (let a = 0; a < 48; a++) {
            const x = w / 2 + r * Math.cos((a / 48) * 2 * Math.PI);
            const y = h / 2 + r * Math.sin((a / 48) * 2 * Math.PI);
            const info = window.__TCE__.pointerAt(x, y);
            if (info?.water === "land" && !info.field && !info.building) return { x, y };
          }
        }
        return null;
      },
      { w: box.width, h: box.height },
    );
    if (!at) throw new Error("no open ground near the village");
    await page.mouse.click(box.x + at.x, box.y + at.y);
    await page.waitForFunction((n) => window.__TCE__.state().people > n, people, {
      timeout: 60_000,
    });
    await page.waitForTimeout(3000);
    mark("long run begins");

    // On to ten years from the founding.
    await runAhead(page, "5 years", 5 * 365 * 1440);
    for (let i = 0; i < 4; i++) await runAhead(page, "a year", 365 * 1440);
    mark("long run ends");
    await zoomTo(page, 3.5);
    await page.waitForTimeout(4000);
    await zoomTo(page, 0.9);
    await page.waitForTimeout(4000);

    // Save, then load the save back: the same village, the same day.
    const before = await state(page);
    await page.getByRole("button", { name: "Save", exact: true }).click();
    await page.getByLabel("Label").fill("Ten years on");
    await page.locator("#save-submit").click();
    await expect(page.locator("#save-dialog")).toBeHidden();
    await expect(page.locator("#events")).toContainText("Ten years on");
    await page.getByRole("button", { name: "Load", exact: true }).click();
    const row = page.locator("#load-body tr", { hasText: "Ten years on" });
    await expect(row).toBeVisible();
    await page.waitForTimeout(1500);
    await row.getByRole("button", { name: "Load" }).click();
    await page.waitForFunction(
      (e) => window.__TCE__.state().epoch > e && window.__TCE__.map().state === "ready",
      before.epoch,
      { timeout: 180_000 },
    );
    const loaded = await state(page);
    expect(loaded.clock?.minute).toBe(before.clock?.minute);
    expect(loaded.people).toBe(before.people);
    await page.locator("#chronicle").getByRole("button").first().click();
    await zoomTo(page, 2);
    await page.waitForTimeout(5000);
    mark("end");
    const c = loaded.clock;
    console.log(
      `demo: ${loaded.people} people on ${c?.day}/${c?.month}/${c?.year}; chronicle:\n` +
        loaded.chronicle.slice(-12).join("\n"),
    );
  } finally {
    await host.stop();
  }
});
