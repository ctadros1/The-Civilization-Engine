// The M3c demo's pictures (plan §7): one valley lived from the command line through a dry year and
// a wet one (river valley seed 2 draws a dry year 4 and a wet year 5 on its valley floor), then
// its weather panel (the years against what they usually bring), its market (what grain is asked
// now) and its chronicle (the weather that stood out); then a year lived at Max, a day at a time.
// It takes some time, so it runs only with TCE_DEMO=1, and TCE_SHOTS_DIR saves the screenshots.
// TCE_DEMO_SEED and TCE_DEMO_DAYS change the seed and the days lived. Runs differ (each world draws
// its own identity), though its weather does not; what it logs is one run's.

import { expect, test } from "@playwright/test";

import { makeWorld, startHost, tempSaves } from "./host.js";

test.skip(!process.env.TCE_DEMO, "the M3c demo runs only with TCE_DEMO=1");
test.setTimeout(60 * 60_000);
test.use({ viewport: { width: 1440, height: 900 } });

const shots = process.env.TCE_SHOTS_DIR;
const NAME = "Weather Valley";

test("a village through a dry year and a wet one, then a year at Max", async ({ page }) => {
  const saves = tempSaves();
  const seed = process.env.TCE_DEMO_SEED ?? "2";
  // From 1 March of year 1 to 1 November of year 5: year 4's harvest and year 5's are in.
  const days = process.env.TCE_DEMO_DAYS ?? "1705";
  makeWorld(
    saves,
    [
      ...["--seed", seed, "--preset", "core:worldgen/river_valley", "--size", "768"],
      ...["--days", days, "--name", NAME],
    ],
    45 * 60_000,
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
        (window.__TCE__.state().weather?.months ?? 0) > 12 &&
        (window.__TCE__.state().markets?.length ?? 0) > 0 &&
        window.__TCE__.map().state === "ready",
      NAME,
      { timeout: 300_000 },
    );
    if (!(await page.evaluate(() => window.__TCE__.state().clock?.paused))) {
      await page.getByRole("button", { name: "Pause" }).click();
    }

    // The years on the valley floor against what they usually bring: year 4 dry, year 5 wet.
    const weather = page.locator("#weather-body");
    const years = weather.locator(".weather-years tbody tr");
    await expect(years.first()).toBeVisible();
    const rows = await years.allInnerTexts();
    expect(rows.some((r) => /^4\s/.test(r))).toBe(true);
    expect(rows.some((r) => /^5\s/.test(r))).toBe(true);
    // What grain is asked now, and what the chronicle says of the weather.
    const state = await page.evaluate(() => window.__TCE__.state());
    const grain = await page
      .locator("#market-body .market-goods li")
      .filter({ hasText: /^Grain/ })
      .allInnerTexts();
    const weatherWords = ["dry", "wet", "winter", "warm", "cold"];
    console.log(
      `demo: ${JSON.stringify({
        people: state.people,
        years: rows,
        grain,
        chronicle: state.chronicle.filter((t) => weatherWords.some((w) => t.includes(w))),
      })}`,
    );
    if (shots) {
      await page.locator("#weather-panel").screenshot({ path: `${shots}/m3c-weather.png` });
      await page.locator("#market-body").screenshot({ path: `${shots}/m3c-market.png` });
      await page.locator("#chronicle").screenshot({ path: `${shots}/m3c-chronicle.png` });
      await page.screenshot({ path: `${shots}/m3c-village.png` });
    }

    // A year at Max: Accelerated mode, the clock standing at a midnight each time it shows.
    const start = (await page.evaluate(() => window.__TCE__.state().clock))!.minute;
    await page.locator("#speeds").getByRole("radio", { name: "Max" }).click();
    await page.getByRole("button", { name: "Run" }).click();
    await page.waitForFunction(
      (from) => (window.__TCE__.state().clock?.minute ?? from) >= from + 365 * 1440,
      start,
      { timeout: 20 * 60_000 },
    );
    // Read at once: at Max the clock runs on, and the bar is drawn anew at every frame.
    const after = await page.evaluate(() => window.__TCE__.state());
    expect(after.clock?.mode).toBe("accelerated");
    expect((after.clock?.minute ?? 1) % 1440).toBe(0);
    console.log(
      `demo after a year at Max: ${JSON.stringify({
        people: after.people,
        years: await years.allInnerTexts(),
      })}`,
    );
    if (shots) await page.screenshot({ path: `${shots}/m3c-a-year-on.png` });
  } finally {
    await host.stop();
  }
});
