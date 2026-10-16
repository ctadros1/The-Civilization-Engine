// The M6b demo's pictures (plan §7): a great fire, the fire tool's if no lived world shows one,
// and what followed. The world is made by civ-sim's `great_fire` example: a band of 100 lives a
// year and a half; if no fire of its own took four buildings, the observer sets alight the
// thatched building with the most others within 4 m, at noon of the first dry, windy day (the
// observer's choices of where and when, logged as NUDGEs), and the world lives a year and a half
// more. Each world draws its own identity, so the example makes up to four worlds and keeps the
// first whose fire took four buildings, or the last; its lines say which. The observer then shows
// the fires in the Incidents panel, the government panel's laws (any rule for building proposed,
// with its history), and what the chronicle tells. Whatever happened is what is shown: if nobody
// proposed a rule, the panel shows none. It takes a long while, so it runs only with TCE_DEMO=1;
// TCE_SHOTS_DIR saves the screenshots, and TCE_DEMO_SEED changes the seed.

import { mkdirSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";

import { expect, test } from "@playwright/test";

import { makeGreatFireWorld, startHost, tempSaves } from "./host.js";

test.skip(!process.env.TCE_DEMO, "the M6b demo runs only with TCE_DEMO=1");
test.setTimeout(180 * 60_000);

const shots = process.env.TCE_SHOTS_DIR;
const NAME = "Thatchford";

test("a great fire, who remembers it, and the rules for building it moved people to", async ({ browser }) => {
  const saves = tempSaves();
  const seen: string[] = [];
  const log = (line: string) => {
    seen.push(line);
    console.log(line);
  };
  const made = makeGreatFireWorld(saves, process.env.TCE_DEMO_SEED ?? "1");
  log(`demo world:\n${made.trim()}`);
  const host = await startHost(saves);
  const videoDir = process.env.TCE_DEMO_VIDEO ? resolve("test-results", "m6b-raw") : undefined;
  if (videoDir) mkdirSync(videoDir, { recursive: true });
  const context = await browser.newContext({
    viewport: { width: 1920, height: 1080 },
    ...(videoDir ? { recordVideo: { dir: videoDir, size: { width: 1920, height: 1080 } } } : {}),
  });
  const page = await context.newPage();
  const video = page.video();
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
        window.__TCE__.map().state === "ready",
      NAME,
      { timeout: 300_000 },
    );
    if (!(await page.evaluate(() => window.__TCE__.state().clock?.paused))) {
      await page.getByRole("button", { name: "Pause" }).click();
    }

    // The fires: how many, how many were the observer's, and each one's page.
    const panel = page.locator("#incidents-body");
    await panel.getByLabel("Kind of incident").selectOption({ label: "Fires" });
    await expect(panel.locator(".incident-totals")).toContainText("Fires");
    log(`fires: ${await panel.locator(".incident-totals").innerText()}`);
    if (shots) {
      await panel.locator(".incident-totals").scrollIntoViewIfNeeded();
      await page.screenshot({ path: `${shots}/m6b-incidents.png` });
    }
    if (process.env.TCE_DEMO_VIDEO) await page.waitForTimeout(3_000);

    // The laws: every rule for building proposed, opened to its history.
    const gov = page.locator("#government-body");
    const rules = gov.locator("details.law").filter({ hasText: "a rule for building" });
    const n = await rules.count();
    log(`rules for building proposed: ${n}`);
    for (let i = 0; i < n; i++) {
      const law = rules.nth(i);
      await law.locator("summary").click();
      log(`law: ${(await law.innerText()).replace(/\s+/g, " ").trim()}`);
    }
    if (shots) {
      if (n > 0) await rules.first().scrollIntoViewIfNeeded();
      await page.locator("#government-panel").screenshot({ path: `${shots}/m6b-government.png` });
    }
    if (process.env.TCE_DEMO_VIDEO) await page.waitForTimeout(3_000);

    // What the chronicle tells of fires and buildings that gave way.
    const told = await page.locator("#chronicle li").allInnerTexts();
    for (const line of told.filter((t) => /fire|burn|gave way|rule/i.test(t))) log(`chronicle: ${line}`);
    if (shots) {
      await page.locator(".map-pane").screenshot({ path: `${shots}/m6b-map.png` });
      writeFileSync(`${shots}/m6b-demo.txt`, `${seen.join("\n")}\n`);
    }
  } finally {
    await context.close();
    if (video && shots) await video.saveAs(`${shots}/m6b-demo.webm`);
    await host.stop();
  }
});
