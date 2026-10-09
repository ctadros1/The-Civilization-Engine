// The M4 demo (plan §7): the same start, five seeds lived forty years, their regimes side by side,
// and one law's full history. The five are the sanity dashboard's worlds (river valley, seeds 1 to
// 5, 1024 cells, the content's two property regimes in turn), whose saves at each tenth year's
// end `civ-host dashboard --keep-saves DIR` keeps. Each world's save of year 40 is loaded in the
// observer: the Government panel shows its custom, what it would be called and why, and its laws;
// the world whose gathering decided the most has its oldest decided law opened to its whole
// history. If fewer than two regimes arose, that is what is shown: nothing here is forced.
//
// It takes long, so it runs only with TCE_DEMO=1. TCE_DEMO_SAVES names a folder the dashboard
// kept its saves in (with one `dashboard-<seed>` folder a world); without it, the dashboard is run
// here for forty years first (about an hour). TCE_SHOTS_DIR saves the screenshots. Runs differ
// (each world draws its own identity); what it logs is one run's.

import { spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";

import { expect, test } from "@playwright/test";

import { hostBinary, startHost, tempSaves } from "./host.js";

test.skip(!process.env.TCE_DEMO, "the M4 demo runs only with TCE_DEMO=1");
test.setTimeout(3 * 60 * 60_000);
test.use({ viewport: { width: 1440, height: 900 } });

const shots = process.env.TCE_SHOTS_DIR;
const SEEDS = [1, 2, 3, 4, 5];

/** The folder the dashboard kept its saves in: TCE_DEMO_SAVES, or a forty-year run made here. */
function keptSaves(): string {
  const given = process.env.TCE_DEMO_SAVES;
  if (given) return given;
  const dir = tempSaves();
  const done = spawnSync(hostBinary(), ["dashboard", "--years", "40", "--keep-saves", dir], {
    encoding: "utf8",
    timeout: 2 * 60 * 60_000,
  });
  if (!existsSync(path.join(dir, "dashboard-1")) && !existsSync(path.join(dir, "dashboard-2"))) {
    throw new Error(`the dashboard kept no saves (${done.status}):\n${done.stdout}\n${done.stderr}`);
  }
  return dir;
}

interface Seen {
  seed: number;
  people: number;
  name: string;
  label: string;
  why: string[];
  custom: string;
  customHistory: string[];
  episodes: string[];
  laws: string[];
  decided: number;
}

/** Loads world `seed`'s save of year `year` from the load dialog's list, and waits for it. */
async function loadYear(
  page: import("@playwright/test").Page,
  seed: number,
  year: number,
): Promise<boolean> {
  await page.getByRole("button", { name: "Load", exact: true }).click();
  const row = page
    .locator("#load-body tbody tr")
    .filter({ hasText: `Dashboard ${seed}` })
    .filter({ hasText: `year ${year}` });
  if ((await row.count()) === 0) {
    await page.keyboard.press("Escape");
    return false;
  }
  await row.first().getByRole("button", { name: "Load" }).click();
  await page.waitForFunction(
    (name) =>
      window.__TCE__.state().world?.name === name &&
      (window.__TCE__.state().government?.polities.length ?? 0) > 0 &&
      window.__TCE__.map().state === "ready",
    `Dashboard ${seed}`,
    { timeout: 300_000 },
  );
  if (!(await page.evaluate(() => window.__TCE__.state().clock?.paused))) {
    await page.getByRole("button", { name: "Pause" }).click();
  }
  return true;
}

test("five worlds of one start, forty years on, side by side", async ({ page }) => {
  // The dashboard keeps each world's saves in a folder of its own, so the folder it kept them in
  // is a saves folder with one world in each.
  const kept = keptSaves();
  const host = await startHost(kept);
  try {
    page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");
    const seen: Seen[] = [];
    for (const seed of SEEDS) {
      if (!(await loadYear(page, seed, 40))) {
        console.log(`m4 demo: world ${seed} has no save of year 40 (its band may have ended)`);
        continue;
      }
      const state = await page.evaluate(() => window.__TCE__.state());
      const polity = state.government!.polities[0]!;
      const panel = page.locator("#government-body");
      await expect(panel.locator(".polity h3").first()).toHaveText(polity.name);
      await panel.locator("details.label").first().locator("summary").click();
      seen.push({
        seed,
        people: state.people,
        name: polity.name,
        label: polity.label,
        why: polity.labelWhy,
        custom: await panel.locator(".custom").first().innerText(),
        customHistory: polity.customHistory,
        episodes: [...polity.revolts, ...polity.coups, ...polity.petitions, ...polity.refusals],
        laws: polity.laws.map((l) => `${l.what}: ${l.status}; ${l.decision}`),
        decided: polity.laws.filter((l) => l.outcome !== null).length,
      });
      if (shots) {
        await page
          .locator("#government-panel")
          .screenshot({ path: `${shots}/m4-demo-world-${seed}.png` });
      }
    }
    expect(seen.length).toBeGreaterThan(0);

    // The world whose gathering decided the most: its oldest decided law, opened to its history.
    const most = [...seen].sort((a, b) => b.decided - a.decided || a.seed - b.seed)[0]!;
    let history = "no law was decided in any world";
    if (most.decided > 0 && (await loadYear(page, most.seed, 40))) {
      const polity = (await page.evaluate(() => window.__TCE__.state())).government!
        .polities[0]!;
      // Laws are listed newest first, in the panel as in the data; the oldest decided is found
      // by its place.
      let at = -1;
      polity.laws.forEach((l, i) => {
        if (l.outcome !== null) at = i;
      });
      const law = page.locator("#government-body .polity").first().locator("details.law").nth(at);
      await law.locator("summary").click();
      await expect(law.locator(".law-history")).toContainText("Proposed");
      history = await law.innerText();
      if (shots) {
        await law.screenshot({ path: `${shots}/m4-demo-law-history.png` });
      }
    }

    // A coda past the demo's forty years: a world whose regime changed by year 50 (the dashboard's
    // report says which), shown then, with the newest law its body decided opened.
    const report = path.join(kept, "dashboard.json");
    const later: { seed: number; label: string; customHistory: string[]; law: string }[] = [];
    if (existsSync(report)) {
      const worlds = (
        JSON.parse(readFileSync(report, "utf8")) as {
          worlds: { seed: number; regimes_by_decade: [number, string[]][] }[];
        }
      ).worlds;
      for (const w of worlds) {
        const at = (y: number) => JSON.stringify(w.regimes_by_decade.find((d) => d[0] === y)?.[1]);
        if (at(40) === at(50) || !(await loadYear(page, w.seed, 50))) continue;
        const polity = (await page.evaluate(() => window.__TCE__.state())).government!
          .polities[0]!;
        await page
          .locator("#government-body details.label")
          .first()
          .locator("summary")
          .click();
        if (shots) {
          await page
            .locator("#government-panel")
            .screenshot({ path: `${shots}/m4-demo-world-${w.seed}-year-50.png` });
        }
        const newest = polity.laws.findIndex((l) => l.outcome !== null);
        let law = "";
        if (newest >= 0) {
          const opened = page
            .locator("#government-body .polity")
            .first()
            .locator("details.law")
            .nth(newest);
          await opened.locator("summary").click();
          law = await opened.innerText();
          if (shots) {
            await opened.screenshot({ path: `${shots}/m4-demo-world-${w.seed}-year-50-law.png` });
          }
        }
        later.push({
          seed: w.seed,
          label: polity.label,
          customHistory: polity.customHistory,
          law,
        });
      }
    }

    // Side by side: what each would be called, and why. Fewer than two regimes is reported.
    const regimes = new Set(seen.map((s) => s.label.split(" (")[0]));
    console.log(
      `m4 demo: ${JSON.stringify({
        regimes: regimes.size,
        worlds: seen.map((s) => ({
          seed: s.seed,
          people: s.people,
          village: s.name,
          label: s.label,
          why: s.why,
          custom: s.custom,
          customHistory: s.customHistory,
          episodes: s.episodes,
          laws: s.laws.length,
          decided: s.decided,
        })),
        history: { world: most.seed, text: history },
        year50: later,
      })}`,
    );
  } finally {
    await host.stop();
  }
});
