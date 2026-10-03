// The M0 demo, end to end: generate a world from a seed, pan and zoom it, run the clock, save,
// load it back, and recover after the host is killed (plan §7, M0).

import { expect, test, type Page } from "@playwright/test";

import { startHost, tempSaves, type Host } from "./host.js";

// window.__TCE__ is typed by src/main.ts, which is part of the same TypeScript program.

const shots = process.env.TCE_SHOTS_DIR;

async function snap(page: Page, name: string): Promise<void> {
  if (shots) await page.screenshot({ path: `${shots}/${name}.png` });
}

const state = (page: Page) => page.evaluate(() => window.__TCE__.state());
const mapState = (page: Page) => page.evaluate(() => window.__TCE__.map());

async function open(page: Page, host: Host): Promise<void> {
  page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
  await page.goto(host.url);
  await page.waitForFunction(() => window.__TCE__?.state().connection === "open");
}

async function createWorld(page: Page, name: string, size: string, seed: string): Promise<void> {
  const before = (await state(page)).epoch;
  await page.getByRole("button", { name: "New world" }).click();
  await page.getByLabel("Name").fill(name);
  await page.getByLabel("Size").selectOption(size);
  await page.getByLabel("Seed").fill(seed);
  await page.getByRole("button", { name: "Create", exact: true }).click();
  await expect(page.locator("#task")).toBeVisible();
  await page.waitForFunction(
    (epoch) => window.__TCE__.state().epoch > epoch && window.__TCE__.map().state === "ready",
    before,
    { timeout: 180_000 },
  );
}

test("a world is generated, explored, saved and loaded back", async ({ page }) => {
  const host = await startHost(tempSaves());
  try {
    await open(page, host);
    await expect(page.getByText("No world yet")).toBeVisible();
    await snap(page, "01-empty");

    await createWorld(page, "Demo Valley", "1024", "7");
    await expect(page.locator("#world-title")).toHaveText("Demo Valley");
    await expect(page.locator("#world-body")).toContainText("1024 × 1024 cells");
    const world = (await state(page)).world;
    expect(world?.seed).toBe("7");
    expect((await mapState(page)).reaches).toBeGreaterThan(0);
    await snap(page, "02-world");

    // Pan by dragging, zoom with the wheel, read coordinates under the pointer.
    const canvas = page.locator("#map canvas");
    const box = await canvas.boundingBox();
    if (!box) throw new Error("no map canvas");
    const cx = box.x + box.width / 2;
    const cy = box.y + box.height / 2;
    const before = (await mapState(page)).camera;
    await page.mouse.move(cx, cy);
    await page.mouse.down();
    await page.mouse.move(cx - 120, cy - 60, { steps: 6 });
    await page.mouse.up();
    const panned = (await mapState(page)).camera;
    expect(panned.x).toBeLessThan(before.x - 100);
    await page.mouse.move(cx, cy);
    for (let i = 0; i < 6; i++) await page.mouse.wheel(0, -240);
    await page.waitForFunction((s) => window.__TCE__.map().camera.scale > s * 2, before.scale);
    await expect(page.locator("#readout")).toContainText("cell");
    await expect(page.locator("#readout")).toContainText(" m ·");
    await snap(page, "03-zoomed");
    await page.getByRole("button", { name: "Fit" }).click();

    // Run the clock at 10x for a moment.
    const startMinute = (await state(page)).clock?.minute ?? 0;
    await page.getByRole("radio", { name: "10×" }).click();
    await page.waitForFunction(
      (m) => (window.__TCE__.state().clock?.minute ?? 0) > m + 10,
      startMinute,
    );
    await page.getByRole("button", { name: "Pause" }).click();
    await page.waitForFunction(() => window.__TCE__.state().clock?.paused === true);
    const savedMinute = (await state(page)).clock?.minute ?? 0;

    // Save, then load that save back.
    await page.getByRole("button", { name: "Save", exact: true }).click();
    await page.getByLabel("Label").fill("Before the flood");
    await page.locator("#save-submit").click();
    await expect(page.locator("#save-dialog")).toBeHidden();
    await expect(page.locator("#events")).toContainText("Before the flood");

    const epoch = (await state(page)).epoch;
    await page.getByRole("button", { name: "Load", exact: true }).click();
    const row = page.locator("#load-body tr", { hasText: "Before the flood" });
    await expect(row).toBeVisible();
    await snap(page, "04-saves");
    await row.getByRole("button", { name: "Load" }).click();
    await page.waitForFunction(
      (e) => window.__TCE__.state().epoch > e && window.__TCE__.map().state === "ready",
      epoch,
    );
    const loaded = await state(page);
    expect(loaded.world?.name).toBe("Demo Valley");
    expect(loaded.clock?.minute).toBe(savedMinute);
    expect(loaded.clock?.paused).toBe(true);
    expect(loaded.events.some((e) => e.kind === "loaded")).toBe(true);
    expect(loaded.lastError).toBeNull();
    await snap(page, "05-loaded");
  } finally {
    await host.stop();
  }
});

test("after the host is killed, the next start offers to recover the world", async ({ page }) => {
  const saves = tempSaves();
  const first = await startHost(saves);
  try {
    await open(page, first);
    await createWorld(page, "Fragile Valley", "256", "21");
  } finally {
    await first.kill();
  }

  // The page notices and keeps retrying the old address.
  await expect(page.locator("#connection")).toContainText("No host");

  const second = await startHost(saves);
  try {
    await open(page, second);
    const dialog = page.getByRole("dialog", { name: "Recover your world?" });
    await expect(dialog).toBeVisible();
    await expect(dialog).toContainText("Fragile Valley");
    await snap(page, "06-recovery");
    await dialog.getByRole("button", { name: "Recover" }).click();
    await page.waitForFunction(
      () =>
        window.__TCE__.state().world?.name === "Fragile Valley" &&
        window.__TCE__.map().state === "ready",
    );
    await expect(dialog).toBeHidden();
    expect((await state(page)).recovery).toBeNull();
  } finally {
    await second.stop();
  }
});
