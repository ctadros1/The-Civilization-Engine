// M6a slice BA, end to end (ADR-0021 §8): the observer holds back the rain from the observer's
// hand. The host answers in words, the weather panel tells the drought while it holds and marks
// the month it holds, and the chronicle records it. What people make of a dry year is theirs.

import { expect, test } from "@playwright/test";

import { makeWorld, startHost, tempSaves } from "./host.js";

const shots = process.env.TCE_SHOTS_DIR;

test("the observer holds back the rain, and the weather panel tells of it", async ({ page }) => {
  const saves = tempSaves();
  makeWorld(saves, ["--seed", "3", "--size", "512", "--days", "2", "--name", "Dry Valley"]);
  const host = await startHost(saves);
  try {
    page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");
    await page.getByRole("button", { name: "Load", exact: true }).click();
    await page.locator("#load-body tbody tr").first().getByRole("button", { name: "Load" }).click();
    await page.waitForFunction(() => (window.__TCE__.state().weather?.months ?? 0) > 12, undefined, {
      timeout: 60_000,
    });
    if (!(await page.evaluate(() => window.__TCE__.state().clock?.paused))) {
      await page.getByRole("button", { name: "Pause" }).click();
    }
    const hold = page.getByRole("button", { name: "Hold back the rain" });
    const panel = page.locator("#weather-body");
    await expect(panel.locator(".weather-drought")).toHaveCount(0);
    await expect(panel.locator(".weather-months tr.held")).toHaveCount(0);

    // Six months of a hard drought.
    await page.getByLabel("Months the rain is held back").selectOption("6");
    await page.getByLabel("How dry the months are held").selectOption("2");
    await expect(hold).toBeEnabled();
    await hold.click();
    await expect(page.locator("#banner-text")).toContainText(
      "The observer holds back the rain for 6 months, 2.0 standard deviations dry, until",
    );
    // The panel asks again at once: the drought shows before the next day is lived.
    await expect(panel.locator(".weather-drought")).toContainText(
      "The observer holds back the rain, 2.0 standard deviations dry, until",
    );
    await expect(panel.locator(".weather-months tbody tr").first()).toHaveClass(/held/);
    await expect(panel.locator(".weather-months tbody tr").first()).toContainText("(held)");
    await expect(panel).toContainText("the observer's drought held that month's weather");
    await expect(page.locator("#chronicle")).toContainText(
      "One recorded influence: the observer held back the rain: for 6 months the weather runs dry, until",
    );
    if (shots) await page.locator("#weather-panel").screenshot({ path: `${shots}/drought-weather.png` });

    // Holding it again refreshes the same record, and the host says so.
    await hold.click();
    await expect(page.locator("#banner-text")).toContainText("The observer again holds back the rain for 6 months");
  } finally {
    await host.stop();
  }
});
