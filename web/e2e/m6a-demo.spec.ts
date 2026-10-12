// The M6a demo's pictures (plan §7): the observer gives cholera to one person; whether and where
// it spreads is the water's and people's. The world is made by civ-sim's `cholera_world` example:
// a band of 100 lives about a year and a half, digging wells; then, on the first wet day, the
// observer brings cholera to someone whose home's heap lies above a well other households draw at
// (the observer's choices of whom and when, logged as NUDGEs), and the world lives 90 days more.
// Each world draws its own identity and most introductions spread to nobody, so the example makes
// up to six worlds and keeps the first whose outbreak had five cases by water, or the last; its
// lines say which. The observer then shows the outbreak's page (each source with the cases among
// those who drew from it and among those who did not, what fouled it, who suspects it and what the
// polity did), and the Water lens's *did come*. Whatever happened is what is shown, and if nobody
// in the world linked the illness to the well, the page says so: the trace is then the observer's
// alone. It takes some minutes, so it runs only with TCE_DEMO=1; TCE_SHOTS_DIR saves the
// screenshots, and TCE_DEMO_SEED changes the seed.

import { writeFileSync } from "node:fs";

import { expect, test } from "@playwright/test";

import { makeCholeraWorld, startHost, tempSaves } from "./host.js";

test.skip(!process.env.TCE_DEMO, "the M6a demo runs only with TCE_DEMO=1");
test.setTimeout(90 * 60_000);
test.use({ viewport: { width: 1440, height: 900 } });

const shots = process.env.TCE_SHOTS_DIR;
const NAME = "Wellfield";

test("cholera brought to one person, the water its cases drank, and what people made of it", async ({ page }) => {
  const saves = tempSaves();
  // What the demo saw, printed and, with TCE_SHOTS_DIR, kept beside the screenshots.
  const seen: string[] = [];
  const log = (line: string) => {
    seen.push(line);
    console.log(line);
  };
  const made = makeCholeraWorld(saves, process.env.TCE_DEMO_SEED ?? "1");
  log(`demo world:\n${made.trim()}`);
  const host = await startHost(saves);
  try {
    page.on("pageerror", (e) => log(`page error: ${e.message}`));
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
        window.__TCE__.state().incidents !== null &&
        window.__TCE__.state().waterRev !== 0 &&
        window.__TCE__.map().state === "ready",
      NAME,
      { timeout: 300_000 },
    );
    if (!(await page.evaluate(() => window.__TCE__.state().clock?.paused))) {
      await page.getByRole("button", { name: "Pause" }).click();
    }

    // The cases of illness, all of them the observer's by the outbreak it began, and the outbreak.
    const panel = page.locator("#incidents-body");
    await panel.getByLabel("Kind of incident").selectOption("2");
    await expect(panel.locator(".incident-totals")).toContainText("Illness: ");
    await expect(panel.locator(".incident-totals")).toContainText("the observer's");
    const outbreaks = panel.locator("li.outbreak");
    await expect(outbreaks.first()).toBeVisible();
    log(`illness: ${await panel.locator(".incident-totals").innerText()}`);
    for (const o of await outbreaks.allInnerTexts()) log(`outbreak: ${o}`);
    if (shots) await page.locator("#incidents-panel").screenshot({ path: `${shots}/m6a-incidents.png` });

    // Its page: how it began, its curve, how its cases took it, and each source with the three
    // layers kept apart.
    await outbreaks.filter({ hasText: "the observer's" }).first().locator(".outbreak-open").click();
    await expect(panel.locator(".outbreak-cause")).toContainText("brought by the observer");
    await expect(panel.locator("svg.outbreak-curve")).toHaveCount(1);
    log(`cause: ${await panel.locator(".outbreak-cause").innerText()}`);
    log(`routes: ${await panel.locator(".outbreak-routes").innerText()}`);
    const sources = panel.locator(".outbreak-source");
    for (let i = 0; i < (await sources.count()); i++) {
      const s = sources.nth(i);
      log(`source: ${await s.locator(".source-name").innerText()}`);
      log(`  kernel: ${(await s.locator(".layer-truth li").allInnerTexts()).join(" | ")}`);
      log(`  people: ${(await s.locator(".layer-known li").allInnerTexts()).join(" | ")}`);
      log(`  polity: ${(await s.locator(".layer-polity li").allInnerTexts()).join(" | ")}`);
    }
    if (shots) await page.locator("#incidents-panel").screenshot({ path: `${shots}/m6a-outbreak.png` });

    // The Water lens: where each home's people drew water in the past 30 days.
    await page.getByLabel("Lens").selectOption("came");
    await page.waitForFunction(() => window.__TCE__.lens().homes.length > 0, undefined, { timeout: 60_000 });
    const lens = await page.evaluate(() => window.__TCE__.lens());
    log(`lens: ${lens.summary.join(" / ")}`);
    await expect(page.locator("#lens-legend .lens-counts")).toHaveText(lens.summary[1]!);
    if (shots) await page.locator(".map-pane").screenshot({ path: `${shots}/m6a-lens.png` });

    // What the chronicle tells of it: counts, never a source as the cause.
    const told = await page.locator("#chronicle li").allInnerTexts();
    for (const line of told.filter((t) => /cholera|outbreak|suspect|clos/i.test(t))) log(`chronicle: ${line}`);
    if (shots) writeFileSync(`${shots}/m6a-demo.txt`, `${seen.join("\n")}\n`);
  } finally {
    await host.stop();
  }
});
