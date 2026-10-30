// M12c's viewer captures (ADR-0039 §6): one host, a driver page and a spectator page on the same
// world. The driver loads it and pauses; the spectator sees the same world with every command
// control disabled. Nothing in this spec changes simulation state beyond loading and pausing.

import { mkdirSync } from "node:fs";
import { resolve } from "node:path";

import { expect, test, type Browser, type Page } from "@playwright/test";

import { makeWorld, startHost, tempSaves, type Host } from "./host.js";

test.skip(!process.env.TCE_DEMO, "the M12 demo runs only with TCE_DEMO=1");
test.setTimeout(10 * 60_000);

const NAME = "Two Viewers";

async function viewer(browser: Browser, host: Host, name: string): Promise<Page> {
  const context = await browser.newContext({ viewport: { width: 1280, height: 720 } });
  const page = await context.newPage();
  page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
  await page.goto(`${host.url}?name=${name}`);
  await page.waitForFunction(() => window.__TCE__?.state().viewers !== null);
  return page;
}

test("M12: a driver and a spectator on one world", async ({ browser }) => {
  const saves = tempSaves();
  const shots = process.env.TCE_SHOTS_DIR ? resolve(process.env.TCE_SHOTS_DIR) : undefined;
  if (shots) mkdirSync(shots, { recursive: true });
  makeWorld(saves, ["--seed", "3", "--size", "512", "--days", "20", "--name", NAME]);
  const host = await startHost(saves);
  const pages: Page[] = [];
  try {
    const ada = await viewer(browser, host, "Ada");
    pages.push(ada);
    await expect(ada.locator("#connection")).toContainText("You drive");
    await ada.getByRole("button", { name: "Load", exact: true }).click();
    await ada
      .locator("#load-body tbody tr")
      .filter({ hasText: NAME })
      .getByRole("button", { name: "Load" })
      .click();
    await ada.waitForFunction((n) => window.__TCE__.state().world?.name === n, NAME, { timeout: 120_000 });
    if (!(await ada.evaluate(() => window.__TCE__.state().clock?.paused))) {
      await ada.getByRole("button", { name: "Pause" }).click();
    }

    const bo = await viewer(browser, host, "Bo");
    pages.push(bo);
    await bo.waitForFunction((n) => window.__TCE__.state().world?.name === n, NAME, { timeout: 120_000 });
    await expect(bo.locator("#connection")).toContainText("You are watching; Ada drives");
    for (const id of ["#btn-new", "#btn-load", "#btn-run"]) {
      await expect(bo.locator(id)).toBeDisabled();
    }
    await expect(ada.locator("#hand-over")).toBeVisible();
    await expect(ada.locator("#btn-new")).toBeEnabled();

    // Let both maps draw before the captures.
    await ada.waitForTimeout(2_000);
    if (shots) {
      await ada.screenshot({ path: resolve(shots, "m12-driver.png") });
      await bo.screenshot({ path: resolve(shots, "m12-spectator.png") });
    }
  } finally {
    for (const p of pages) await p.context().close();
    await host.stop();
  }
});
