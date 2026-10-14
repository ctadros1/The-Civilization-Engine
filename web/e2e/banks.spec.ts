// Controlled engineering fixture: saved geometry and display, not chosen public work.
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { expect, test } from "@playwright/test";
import { startHost, tempSaves } from "./host.js";

test("a saved bank displays its centre-line and borrow-earth account", async ({ page }) => {
  const saves = tempSaves();
  const made = spawnSync("cargo", ["run", "--release", "--quiet", "-p", "civ-sim", "--example", "bank_fixture", "--", saves], { cwd: resolve("../kernel"), encoding: "utf8", timeout: 600_000 });
  expect(made.status, `${made.stdout}\n${made.stderr}`).toBe(0);
  const host = await startHost(saves);
  try {
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__.state().connection === "open");
    await page.getByRole("button", { name: "Load", exact: true }).click();
    await page.locator("#load-body tbody tr").first().getByRole("button", { name: "Load" }).click();
    await page.waitForFunction(() => window.__TCE__.map().bankSegments.length === 1);
    const bank = await page.evaluate(() => window.__TCE__.map().bankSegments[0]);
    if (!bank) throw new Error("bank disappeared after loading");
    expect(bank.by - bank.ay).toBe(24);
    expect(bank.heightM).toBeGreaterThan(0.75);
    expect(bank.words).toContain("50% built");
    expect(bank.words).toContain("borrow pit");
    expect(bank.breached).toBe(false);
  } finally {
    await host.stop();
  }
});
