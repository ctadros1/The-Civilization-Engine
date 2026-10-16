// The M7a demo's pictures (plan §7; M7 design: each part's demo shows what its world reached,
// nudging only priors). The world is made by civ-sim's `council` example: a village of 120 lives
// until its gathering takes its custom to a council and the council is seated; if none has after
// 180 days, the observer nudges what an evening at the gathering costs one who comes (logged as a
// NUDGE; never an event). It saves the day a council is first seated ("A council is seated") and
// the end ("After the elections"). The observer then shows the Government panel: the custom, the
// label, the council's seats and the election page (who stood, where they said they stand, who
// backed them, the names counted, who sat), and what the chronicle tells. Whatever happened is
// what is shown: if no council sat, the panel shows the gathering. The example may live up to two
// worlds and keeps the one that went furthest; the spec loads the one it kept. It takes a long
// while, so it runs only with TCE_DEMO=1; TCE_SHOTS_DIR saves the screenshots, and TCE_DEMO_SEED
// changes the seed.

import { writeFileSync } from "node:fs";

import { expect, test, type Page } from "@playwright/test";

import { makeCouncilWorld, startHost, tempSaves } from "./host.js";

test.skip(!process.env.TCE_DEMO, "the M7a demo runs only with TCE_DEMO=1");
test.setTimeout(180 * 60_000);

const shots = process.env.TCE_SHOTS_DIR;
/** The kept world's name, as the example prints it. */
let NAME = "Moothill";

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

test("a village takes a council, and its elections", async ({ page }) => {
  const saves = tempSaves();
  const seen: string[] = [];
  const log = (line: string) => {
    seen.push(line);
    console.log(line);
  };
  const made = makeCouncilWorld(saves, process.env.TCE_DEMO_SEED ?? "1");
  log(`demo world:\n${made.trim()}`);
  NAME = [...made.matchAll(/^name: (.*)$/gm)].at(-1)?.[1] ?? NAME;
  const host = await startHost(saves);
  try {
    page.on("pageerror", (e) => log(`page error: ${e.message}`));
    await page.setViewportSize({ width: 1920, height: 1080 });
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");
    const gov = page.locator("#government-body");

    // The day a council is first seated: its seats and its first election.
    if (await loadSave(page, "A council is seated")) {
      const elections = gov.locator("details.elections");
      await expect(elections.first()).toBeVisible();
      await elections.first().locator("summary").click();
      for (const e of await elections.first().locator(".election").allInnerTexts()) {
        log(`election: ${e.replace(/\s+/g, " ").trim()}`);
      }
      if (shots) {
        await elections.first().scrollIntoViewIfNeeded();
        await page.locator("#government-panel").screenshot({ path: `${shots}/m7a-election.png` });
      }
    } else {
      log("no council sat: there is no election to show");
    }

    // After: the custom, the label, the seats and every election since.
    expect(await loadSave(page, "After the elections")).toBe(true);
    for (const p of await gov.locator(".polity").all()) {
      const name = await p.locator("h3").innerText();
      const custom = await p.locator(".custom").innerText();
      const offices = await p.locator(".offices li").allInnerTexts();
      log(`polity ${name}: ${custom}`);
      for (const o of offices) log(`  office: ${o}`);
      const elections = p.locator("details.elections");
      if ((await elections.count()) > 0) {
        await elections.locator("summary").click();
        for (const e of await elections.locator(".election").allInnerTexts()) {
          log(`  election: ${e.replace(/\s+/g, " ").trim()}`);
        }
      }
    }
    const amendments = gov.locator("details.law").filter({ hasText: /council|custom/i });
    const n = await amendments.count();
    log(`amendments of the custom proposed: ${n}`);
    for (let i = 0; i < n; i++) {
      const law = amendments.nth(i);
      await law.locator("summary").click();
      log(`law: ${(await law.innerText()).replace(/\s+/g, " ").trim()}`);
    }
    if (shots) {
      await page.locator("#government-panel").screenshot({ path: `${shots}/m7a-government.png` });
    }

    // What the chronicle tells of elections and alliances.
    const told = await page.locator("#chronicle li").allInnerTexts();
    for (const line of told.filter((t) => /election|council|slate/i.test(t))) {
      log(`chronicle: ${line}`);
    }
    if (shots) writeFileSync(`${shots}/m7a-demo.txt`, `${seen.join("\n")}\n`);
  } finally {
    await host.stop();
  }
});
