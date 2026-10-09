// The M5a demo (plan §7): three founding groups that know where one another camped settle apart
// and live thirty years; at the end of year 10 the observer sends a migration wave to dry ground
// among them. `civ-host neighbours --keep-saves DIR` lives the world and keeps its saves at every
// tenth year's end; here its saves of years 10 and 30 are loaded in the observer. The settlements
// list shows each settlement, how it was founded, its past year's accounts, what passed between
// them and any coalition gathering or ended in the past year; the inspector shows someone who
// moved between settlements, with their residence history, and someone the wave brought, with the
// observer's hand. A splinter founding, if one happened, is shown by the settlement it founded;
// if none happened, that is reported. Nothing here is forced.
//
// It takes long, so it runs only with TCE_DEMO=1. TCE_DEMO_SAVES names the folder the command
// kept its saves in (with a `neighbours-1` folder); without it, the command is run here first
// (about half an hour). TCE_SHOTS_DIR saves the screenshots. Runs differ (the world draws its own
// identity); what it logs is one run's.

import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import path from "node:path";

import { expect, test, type Page } from "@playwright/test";

import { hostBinary, startHost, tempSaves } from "./host.js";

test.skip(!process.env.TCE_DEMO, "the M5a demo runs only with TCE_DEMO=1");
test.setTimeout(2 * 60 * 60_000);
test.use({ viewport: { width: 1440, height: 900 } });

const shots = process.env.TCE_SHOTS_DIR;
const WORLD = "Neighbours 1";

/** The folder the command kept its saves in: TCE_DEMO_SAVES, or a thirty-year run made here. */
function keptSaves(): string {
  const given = process.env.TCE_DEMO_SAVES;
  if (given) return given;
  const dir = tempSaves();
  const done = spawnSync(hostBinary(), ["neighbours", "--seed", "1", "--keep-saves", dir], {
    encoding: "utf8",
    timeout: 90 * 60_000,
  });
  console.log(done.stdout);
  if (!existsSync(path.join(dir, "neighbours-1"))) {
    throw new Error(`the command kept no saves (${done.status}):\n${done.stdout}\n${done.stderr}`);
  }
  return dir;
}

/** Loads the world's save of year `year` from the load dialog's list, and waits for it. */
async function loadYear(page: Page, year: number): Promise<boolean> {
  await page.getByRole("button", { name: "Load", exact: true }).click();
  const row = page
    .locator("#load-body tbody tr")
    .filter({ hasText: WORLD })
    .filter({ hasText: `year ${year}` });
  if ((await row.count()) === 0) {
    await page.keyboard.press("Escape");
    return false;
  }
  const before = await page.evaluate(() => window.__TCE__.state().clock?.minute ?? -1);
  await row.first().getByRole("button", { name: "Load" }).click();
  await page.waitForFunction(
    ({ name, before }) =>
      window.__TCE__.state().world?.name === name &&
      (window.__TCE__.state().clock?.minute ?? -1) !== before &&
      window.__TCE__.state().settlements.length > 0 &&
      window.__TCE__.map().state === "ready",
    { name: WORLD, before },
    { timeout: 300_000 },
  );
  if (!(await page.evaluate(() => window.__TCE__.state().clock?.paused))) {
    await page.getByRole("button", { name: "Pause" }).click();
  }
  return true;
}

/** The settlements list as the people panel shows it, one line each. */
async function settlementLines(page: Page): Promise<string[]> {
  await page.evaluate(() => window.__TCE__.select(null));
  const settlements = await page.evaluate(() => window.__TCE__.state().settlements);
  const body = page.locator("#people-body");
  for (const s of settlements) await expect(body).toContainText(s.name);
  return body.locator("p").filter({ hasText: " people · " }).allInnerTexts();
}

/**
 * Inspects people in turn until one's residence history matches `wanted`, and returns their
 * inspector's text (null if nobody of the first `most` does).
 */
async function inspectFirst(
  page: Page,
  wanted: RegExp,
  most: number,
): Promise<{ id: number; text: string } | null> {
  const ids = (await page.evaluate(() => window.__TCE__.briefs().map((b) => b.id))).slice(0, most);
  for (const id of ids) {
    await page.evaluate((x) => window.__TCE__.select(x), id);
    await page.waitForFunction(
      (x) => {
        const s = window.__TCE__.state().selected;
        return s?.id === x && (s.residence !== null || s.error !== null);
      },
      id,
      { timeout: 30_000 },
    );
    const residence = await page.evaluate(() => window.__TCE__.state().selected?.residence);
    if (residence?.some((r) => wanted.test(r))) {
      await expect(page.locator("#people-body .residence")).toContainText(
        /Where they (live|have lived)/,
      );
      return { id, text: await page.locator("#people-body").innerText() };
    }
  }
  return null;
}

test("three settlements thirty years on, a wave among them, their moves on record", async ({
  page,
}) => {
  const kept = keptSaves();
  const host = await startHost(kept);
  try {
    page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");
    const panel = page.locator(".people-panel");
    const seen: Record<string, unknown> = {};

    // Year 10: the wave has just come; its settlement is listed beside the three.
    if (await loadYear(page, 10)) {
      const lines = await settlementLines(page);
      seen.year10 = lines;
      expect(lines.some((l) => l.includes("by a migration wave"))).toBe(true);
      if (shots) await panel.screenshot({ path: `${shots}/m5a-demo-year-10.png` });
    }

    // Year 30: the settlements, someone who moved between them, someone the wave brought.
    expect(await loadYear(page, 30)).toBe(true);
    const state = await page.evaluate(() => window.__TCE__.state());
    const settlements = state.settlements;
    const lines = await settlementLines(page);
    expect(lines.length).toBe(settlements.length);
    expect(lines.filter((l) => l.includes("one of the groups the world began with")).length).toBe(
      3,
    );
    if (shots) await panel.screenshot({ path: `${shots}/m5a-demo-year-30.png` });
    const splinters = settlements.filter((s) => s.founding.startsWith("by households from"));
    seen.year30 = {
      people: state.people,
      settlements: settlements.map((s) => ({
        name: s.name,
        people: s.population,
        founding: s.founding,
        abandoned: s.abandoned,
        year: s.year,
        contacts: s.contacts,
        coalitions: s.coalitions,
      })),
      splinters: splinters.length
        ? splinters.map((s) => `${s.name}, founded ${s.founding}`)
        : "no splinter founding happened",
    };

    const mover = await inspectFirst(
      page,
      /moved there with their household|married into a household there|went with others to found it/,
      400,
    );
    seen.mover = mover?.text ?? "nobody living moved between settlements";
    if (mover && shots) await panel.screenshot({ path: `${shots}/m5a-demo-mover.png` });
    const brought = await inspectFirst(page, /came from beyond the map/, 400);
    if (brought) {
      await expect(page.locator("#people-body .observer-hand")).toContainText(
        /households (came|have come)/,
      );
      seen.wave = brought.text;
      if (shots) await panel.screenshot({ path: `${shots}/m5a-demo-wave.png` });
    } else {
      seen.wave = "nobody the wave brought lives in year 30";
    }
    console.log(`m5a demo: ${JSON.stringify(seen)}`);
  } finally {
    await host.stop();
  }
});
