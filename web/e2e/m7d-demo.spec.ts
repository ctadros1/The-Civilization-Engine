// A live observer capture of the M7d four-year credit run. The run is prepared by
// civ-sim's capture example; nothing in this spec changes simulation state.
import { mkdirSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { resolve } from "node:path";
import { spawnSync } from "node:child_process";

import { expect, test } from "@playwright/test";

import { startHost, tempSaves } from "./host.js";

test.skip(!process.env.TCE_DEMO, "the M7d demo runs only with TCE_DEMO=1");
test.setTimeout(30 * 60_000);

const repo = resolve("..");
const name = "Three Fords";

test("M7d credit and coin in a four-year lived world", async ({ browser }) => {
  const saves = tempSaves();
  const shots = process.env.TCE_SHOTS_DIR;
  const run = spawnSync(
    process.env.CARGO ?? resolve(homedir(), ".cargo", "bin", process.platform === "win32" ? "cargo.exe" : "cargo"),
    ["run", "--release", "--quiet", "-p", "civ-sim", "--example", "capture", "--", saves, "1"],
    {
      cwd: resolve(repo, "kernel"),
      env: { ...process.env, TCE_SMELT_DAY: "30" },
      encoding: "utf8",
      timeout: 25 * 60_000,
    },
  );
  if (run.status !== 0) throw new Error(`M7d lived run failed (${run.status}): ${run.error?.message ?? "no process output"}\n${run.stdout ?? ""}\n${run.stderr ?? ""}`);
  if (shots) {
    mkdirSync(shots, { recursive: true });
    writeFileSync(resolve(shots, "m7d-demo.txt"), run.stdout);
  }

  const host = await startHost(saves);
  const videoDir = process.env.TCE_DEMO_VIDEO ? resolve("test-results", "m7d-raw") : undefined;
  if (videoDir) mkdirSync(videoDir, { recursive: true });
  const context = await browser.newContext({
    viewport: { width: 1920, height: 1080 },
    ...(videoDir ? { recordVideo: { dir: videoDir, size: { width: 1920, height: 1080 } } } : {}),
  });
  const page = await context.newPage();
  const video = page.video();
  try {
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");
    await page.getByRole("button", { name: "Load", exact: true }).click();
    await page.locator("#load-body tbody tr").filter({ hasText: name }).getByRole("button", { name: "Load" }).click();
    await page.waitForFunction(
      (n) => window.__TCE__.state().world?.name === n && window.__TCE__.map().state === "ready",
      name,
      { timeout: 300_000 },
    );
    if (!(await page.evaluate(() => window.__TCE__.state().clock?.paused))) {
      await page.getByRole("button", { name: "Pause" }).click();
    }
    if (process.env.TCE_DEMO_VIDEO) await page.waitForTimeout(3_000);
    if (shots) await page.screenshot({ path: resolve(shots, "m7d-world.png") });

    const wealth = page.locator("#wealth-panel");
    await wealth.scrollIntoViewIfNeeded();
    await expect(wealth).toContainText("Loans");
    await expect(wealth).toContainText("Coin");
    await expect(wealth).toContainText("Three Fords");
    if (shots) await page.screenshot({ path: resolve(shots, "m7d-credit-and-coin.png") });
    if (process.env.TCE_DEMO_VIDEO) await page.waitForTimeout(7_000);
  } finally {
    await context.close();
    if (video && shots) await video.saveAs(resolve(shots, "m7d-demo.webm"));
    await host.stop();
  }
});
