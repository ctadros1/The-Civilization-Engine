// The M6c demo's pictures (plan §7): a siege, and what each gathering made of it. The world is
// made by civ-sim's `siege` example: two villages of 100, known to each other, live until a band
// camps by a line round a village or holds it; if none has after 120 days, the observer nudges a
// raider's valuation of a store they saw, and if a band then goes home before it held, how long
// members stay (each logged as a NUDGE; never an event). It saves the first day of a siege
// ("During the siege") and the end ("After the siege"). The observer then shows the siege page
// (each side's food at current use, what comes and goes by the line, the band's own food), the
// map, and afterwards each polity's settlements, the laws each gathering weighed (bands, lines,
// taking a village in, terms) and what the chronicle tells. Whatever happened is what is shown:
// if no gathering took a village in, the panel shows none. It takes a long while, so it runs only
// with TCE_DEMO=1; TCE_SHOTS_DIR saves the screenshots, and TCE_DEMO_SEED changes the seed.

import { writeFileSync } from "node:fs";

import { expect, test, type Page } from "@playwright/test";

import { makeSiegeWorld, startHost, tempSaves } from "./host.js";

test.skip(!process.env.TCE_DEMO, "the M6c demo runs only with TCE_DEMO=1");
test.setTimeout(180 * 60_000);

const shots = process.env.TCE_SHOTS_DIR;
const NAME = "Siegeford";

/** Loads the save of `NAME` labelled `label`, or returns false if there is none. */
async function loadSave(page: Page, label: string): Promise<boolean> {
  await page.getByRole("button", { name: "Load", exact: true }).click();
  const row = page.locator("#load-body tbody tr").filter({ hasText: NAME }).filter({ hasText: label });
  await page.locator("#load-body table, #load-body .empty").first().waitFor();
  if ((await row.count()) === 0) {
    await page.keyboard.press("Escape");
    return false;
  }
  await row.first().getByRole("button", { name: "Load" }).click();
  await page.waitForFunction(
    (n) =>
      window.__TCE__.state().world?.name === n &&
      window.__TCE__.state().government !== null &&
      window.__TCE__.map().state === "ready",
    NAME,
    { timeout: 300_000 },
  );
  if (!(await page.evaluate(() => window.__TCE__.state().clock?.paused))) {
    await page.getByRole("button", { name: "Pause" }).click();
  }
  return true;
}

test("a siege, each side's food at current use, and what each gathering made of it", async ({ page }) => {
  const saves = tempSaves();
  const seen: string[] = [];
  const log = (line: string) => {
    seen.push(line);
    console.log(line);
  };
  const made = makeSiegeWorld(saves, process.env.TCE_DEMO_SEED ?? "1");
  log(`demo world:\n${made.trim()}`);
  const host = await startHost(saves);
  try {
    page.on("pageerror", (e) => log(`page error: ${e.message}`));
    await page.setViewportSize({ width: 1920, height: 1080 });
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");

    // During the siege: the siege page and the map.
    const gov = page.locator("#government-body");
    if (await loadSave(page, "During the siege")) {
      const sieges = gov.locator(".sieges");
      await expect(sieges).toBeVisible();
      for (const s of await sieges.locator(".siege").allInnerTexts()) {
        log(`siege: ${s.replace(/\s+/g, " ").trim()}`);
      }
      if (shots) {
        await sieges.scrollIntoViewIfNeeded();
        await page.locator("#government-panel").screenshot({ path: `${shots}/m6c-siege.png` });
        await page.locator(".map-pane").screenshot({ path: `${shots}/m6c-map.png` });
      }
    } else {
      log("no band camped by a line or held a place: there is no siege to show");
    }

    // After: each polity's settlements, and the laws its gathering weighed.
    expect(await loadSave(page, "After the siege")).toBe(true);
    for (const p of await gov.locator(".polity").all()) {
      const name = await p.locator("h3").innerText();
      const ended = await p.locator(".ended-note").allInnerTexts();
      const held = await p.locator(".settlements li").allInnerTexts();
      log(`polity ${name}: ${[...ended, ...held].join("; ") || "one settlement, still deciding"}`);
    }
    const laws = gov
      .locator("details.law")
      .filter({ hasText: /band|raid|palisade|line|take in|held village|terms|agreement/i });
    const n = await laws.count();
    log(`laws of force, lines, holding and terms proposed: ${n}`);
    for (let i = 0; i < n; i++) {
      const law = laws.nth(i);
      await law.locator("summary").click();
      log(`law: ${(await law.innerText()).replace(/\s+/g, " ").trim()}`);
    }
    if (shots) {
      if (n > 0) await laws.first().scrollIntoViewIfNeeded();
      await page.locator("#government-panel").screenshot({ path: `${shots}/m6c-government.png` });
    }

    // What the chronicle tells of bands, holds and villages taken in.
    const told = await page.locator("#chronicle li").allInnerTexts();
    for (const line of told.filter((t) => /band|raid|held|took .* in|one body|broke from|palisade/i.test(t))) {
      log(`chronicle: ${line}`);
    }
    if (shots) writeFileSync(`${shots}/m6c-demo.txt`, `${seen.join("\n")}\n`);
  } finally {
    await host.stop();
  }
});
