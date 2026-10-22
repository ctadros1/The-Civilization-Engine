// A lived M9 observer capture. The M9 example prepares a saved world with one detailed settlement,
// household resolution elsewhere, a followed person and a recorded climate influence. This spec
// only opens that save and captures what the observer presents.
import { mkdirSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { resolve } from "node:path";
import { spawnSync } from "node:child_process";

import { expect, test } from "@playwright/test";

import { demoPage, startHost, tempSaves } from "./host.js";

test.skip(!process.env.TCE_DEMO, "the M9 demo runs only with TCE_DEMO=1");
test.setTimeout(40 * 60_000);

const repo = resolve("..");
const name = "Many Hearths";

test("M9: one detailed settlement among household-resolved neighbours", async ({ browser }) => {
  const saves = tempSaves();
  const shots = process.env.TCE_SHOTS_DIR ? resolve(process.env.TCE_SHOTS_DIR) : undefined;
  const cargo = process.env.CARGO ?? resolve(homedir(), ".cargo", "bin", process.platform === "win32" ? "cargo.exe" : "cargo");

  const run = spawnSync(
    cargo,
    ["run", "--release", "--quiet", "-p", "civ-sim", "--example", "m9_world", "--", saves, "10", "50", "1", "2048"],
    { cwd: resolve(repo, "kernel"), encoding: "utf8", timeout: 30 * 60_000 },
  );
  if (run.status !== 0) {
    throw new Error(`M9 lived run failed (${run.status}):\n${run.stdout}\n${run.stderr}`);
  }
  if (shots) {
    mkdirSync(shots, { recursive: true });
    writeFileSync(resolve(shots, "m9-capture-run.txt"), run.stdout);
  }

  const host = await startHost(saves);
  const capture = await demoPage(browser, "m9-demo");
  try {
    const { page } = capture;
    page.on("pageerror", (error) => console.error(`page error: ${error.message}`));
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");
    await page.getByRole("button", { name: "Load", exact: true }).click();
    await page.locator("#load-body tbody tr").filter({ hasText: name }).getByRole("button", { name: "Load" }).click();
    await page.waitForFunction(
      (worldName) => window.__TCE__.state().world?.name === worldName && window.__TCE__.map().state === "ready",
      name,
      { timeout: 300_000 },
    );
    if (!(await page.evaluate(() => window.__TCE__.state().clock?.paused))) {
      await page.getByRole("button", { name: "Pause" }).click();
    }

    await page.getByRole("button", { name: "Fit the map" }).click();
    const people = page.locator("#people-body");
    await expect(people).toContainText("Live settlements away from the one you choose by household");
    const resolution = people.locator("#resolution-by-focus");
    if (!(await resolution.isChecked())) {
      const before = await page.evaluate(() => window.__TCE__.state().clock?.minute ?? 0);
      await resolution.check();
      // Focus-based resolution is a session setting and takes effect at midnight. Let one day
      // turn so the observer can show the household-resolved settlements in its saved view.
      await page.locator("#run-ahead").selectOption("1440");
      await page.waitForFunction(
        (minute) => {
          const clock = window.__TCE__.state().clock;
          return clock !== null && clock.minute >= minute + 1440 && clock.paused;
        },
        before,
        { timeout: 300_000 },
      );
    }
    await expect(people.locator(".badge").filter({ hasText: "lived by household" }).first()).toBeVisible();
    await people.scrollIntoViewIfNeeded();
    if (shots) await page.screenshot({ path: resolve(shots, "m9-settlements.png") });
    if (process.env.TCE_DEMO_VIDEO === "1") await page.waitForTimeout(5_000);

    const weather = page.locator("#weather-panel");
    await weather.scrollIntoViewIfNeeded();
    await expect(weather).toContainText("Climate");
    if (shots) await page.screenshot({ path: resolve(shots, "m9-climate.png") });
    if (process.env.TCE_DEMO_VIDEO === "1") await page.waitForTimeout(5_000);
    await people.scrollIntoViewIfNeeded();
    if (process.env.TCE_DEMO_VIDEO === "1") await page.waitForTimeout(3_000);
  } finally {
    await capture.finish();
    await host.stop();
  }
});
