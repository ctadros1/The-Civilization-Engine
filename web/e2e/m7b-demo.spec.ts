// The M7b demo's pictures (plan §7: "a court strikes down a curfew"; the M7 courts brief §3: the
// demo shows what its world reached, nudging only priors). The world is made by civ-sim's `courts`
// example: a village of 120 through two lean spells (a scaffold of conditions, as theft_world's),
// where every law, case, forum, limit and challenge is people's; each step not reached by its time
// box nudges a prior (logged as a NUDGE; never an event). It saves the day a law is first struck
// down ("A law is struck down") and the end ("After the rulings"). The observer then shows the
// Government panel (each law with its status, a law struck down as such) and the Incidents panel's
// order: the cases by how they closed, challenges and laws struck down, and the latest cases and
// challenges in the kernel's words. Whatever happened is what is shown. It takes a long while, so
// it runs only with TCE_DEMO=1; TCE_SHOTS_DIR saves the screenshots, and TCE_DEMO_SEED changes the
// seed.

import { writeFileSync } from "node:fs";

import { expect, test, type Page } from "@playwright/test";

import { makeCourtsWorld, startHost, tempSaves } from "./host.js";

test.skip(!process.env.TCE_DEMO, "the M7b demo runs only with TCE_DEMO=1");
test.setTimeout(240 * 60_000);

const shots = process.env.TCE_SHOTS_DIR;
/** The kept world's name, as the example prints it. */
let NAME = "Hallowford";

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
      window.__TCE__.state().order !== null &&
      window.__TCE__.map().state === "ready",
    NAME,
    { timeout: 300_000 },
  );
  if (!(await page.evaluate(() => window.__TCE__.state().clock?.paused))) {
    await page.getByRole("button", { name: "Pause" }).click();
  }
  return true;
}

test("a village goes to law, and what its forum makes of a curfew", async ({ page }) => {
  const saves = tempSaves();
  const seen: string[] = [];
  const log = (line: string) => {
    seen.push(line);
    console.log(line);
  };
  const made = makeCourtsWorld(saves, process.env.TCE_DEMO_SEED ?? "1");
  log(`demo world:\n${made.trim()}`);
  NAME = [...made.matchAll(/^name: (.*)$/gm)].at(-1)?.[1] ?? NAME;
  const host = await startHost(saves);
  try {
    page.on("pageerror", (e) => log(`page error: ${e.message}`));
    await page.setViewportSize({ width: 1920, height: 1080 });
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");
    const gov = page.locator("#government-body");
    const order = page.locator("#incidents-body");

    // The day a law is first struck down: the law, as the Government panel shows it.
    if (await loadSave(page, "A law is struck down")) {
      const struck = gov.locator("details.law.struck-down");
      await expect(struck.first()).toBeVisible();
      await struck.first().locator("summary").click();
      log(`struck down: ${(await struck.first().innerText()).replace(/\s+/g, " ").trim()}`);
      if (shots) {
        await struck.first().scrollIntoViewIfNeeded();
        await page.locator("#government-panel").screenshot({ path: `${shots}/m7b-struck-down.png` });
      }
    } else {
      log("no law was struck down: there is no ruling to show");
    }

    // After: every law and its status, and the order's cases and challenges.
    expect(await loadSave(page, "After the rulings")).toBe(true);
    for (const p of await gov.locator(".polity").all()) {
      log(`polity ${await p.locator("h3").innerText()}: ${await p.locator(".custom").innerText()}`);
      for (const law of await p.locator("details.law summary").allInnerTexts()) {
        log(`  law: ${law.replace(/\s+/g, " ").trim()}`);
      }
    }
    if (shots) {
      await page.locator("#government-panel").screenshot({ path: `${shots}/m7b-government.png` });
    }
    await order.getByLabel("Kind of incident").selectOption("1");
    await expect(order.locator(".order-totals")).toBeVisible();
    log(`order: ${await order.locator(".order-totals").innerText()}`);
    const lines = order.locator("details.case-lines");
    if ((await lines.count()) > 0) {
      await lines.locator("summary").click();
      for (const c of await lines.locator("li").allInnerTexts()) log(`  case: ${c}`);
      if (shots) {
        await lines.scrollIntoViewIfNeeded();
        await page.locator("#incidents-panel").screenshot({ path: `${shots}/m7b-cases.png` });
      }
    } else {
      log("no case was brought: there are no case lines to show");
    }

    // What the chronicle tells of cases, forums, limits and rulings.
    const told = await page.locator("#chronicle li").allInnerTexts();
    for (const line of told.filter((t) => /case|hear|limit|struck|challeng|curfew/i.test(t))) {
      log(`chronicle: ${line}`);
    }
    if (shots) writeFileSync(`${shots}/m7b-demo.txt`, `${seen.join("\n")}\n`);
  } finally {
    await host.stop();
  }
});
