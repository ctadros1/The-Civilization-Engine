// The M4a demo's pictures (plan §7): one village through its lean seasons, as its gathering lived
// them. River valley seed 2 (the earlier demos' seed, which draws a dry year 4) is lived from the
// command line to 1 November of year 5; then the Government panel shows what the village would be
// called and why, who proposed what and why, who came and where each stood, what was decided, and
// what moved through its store; one law opens to its whole history; and the Standing panel shows
// its notables and what put them there. Whatever the village decided, or failed to, is what is
// shown. It takes some time, so it runs only with TCE_DEMO=1, and TCE_SHOTS_DIR saves the
// screenshots. TCE_DEMO_SEED and TCE_DEMO_DAYS change the seed and the days lived. Runs differ
// (each world draws its own identity); what it logs is one run's.

import { expect, test } from "@playwright/test";

import { makeWorld, startHost, tempSaves } from "./host.js";

test.skip(!process.env.TCE_DEMO, "the M4a demo runs only with TCE_DEMO=1");
test.setTimeout(60 * 60_000);
test.use({ viewport: { width: 1440, height: 900 } });

const shots = process.env.TCE_SHOTS_DIR;
const NAME = "Council Valley";

test("a village's gathering, its laws and its notables, as they were", async ({ page }) => {
  const saves = tempSaves();
  const seed = process.env.TCE_DEMO_SEED ?? "2";
  // From 1 March of year 1 to 1 November of year 5: the lean weeks before the first harvest, the
  // dry year 4 and the harvest after it.
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
        (window.__TCE__.state().government?.polities.length ?? 0) > 0 &&
        (window.__TCE__.state().standing?.minute ?? 0) > 0 &&
        window.__TCE__.map().state === "ready",
      NAME,
      { timeout: 300_000 },
    );
    if (!(await page.evaluate(() => window.__TCE__.state().clock?.paused))) {
      await page.getByRole("button", { name: "Pause" }).click();
    }

    // The polity: its custom, what it would be called and why, its offices and its laws.
    const state = await page.evaluate(() => window.__TCE__.state());
    const polity = state.government!.polities[0]!;
    const panel = page.locator("#government-body");
    await expect(panel.locator(".polity h3").first()).toHaveText(polity.name);
    await expect(panel.locator(".custom").first()).toContainText("decide by acclamation");
    const label = panel.locator("details.label").first();
    await label.locator("summary").click();
    await expect(label.locator("li")).toHaveCount(polity.labelWhy.length);

    // The oldest law a gathering decided, opened to its whole history. Laws are listed newest
    // first, in the panel as in the data, and two may read alike, so it is found by its place.
    let at = -1;
    polity.laws.forEach((l, i) => {
      if (l.outcome !== null) at = i;
    });
    const decided = at >= 0 ? polity.laws[at] : undefined;
    let history = "no law was decided";
    if (decided) {
      const law = panel.locator(".polity").first().locator("details.law").nth(at);
      await expect(law.locator("summary")).toContainText(decided.what);
      await law.locator("summary").click();
      await expect(law.locator(".law-history")).toContainText("Proposed");
      await expect(law.locator(".law-history")).toContainText(decided.decision);
      await expect(law.locator(".stances li")).toHaveCount(decided.stances);
      history = await law.innerText();
    }

    // Its notables, and what put them there.
    const standing = state.standing!;
    const village = standing.settlements[0]!;
    const notables = await page
      .locator("#standing-body .standing-rows li.notable")
      .allInnerTexts();
    console.log(
      `demo: ${JSON.stringify({
        people: state.people,
        polity: {
          name: polity.name,
          label: polity.label,
          why: polity.labelWhy,
          offices: polity.offices,
          store: polity.store,
          laws: polity.laws.map((l) => `${l.what}: ${l.status}; ${l.decision}`),
        },
        history,
        notables,
        village,
        chronicle: state.chronicle.filter((t) =>
          ["proposed", "gathering", "keeps the common store", "no longer keeps"].some((w) =>
            t.includes(w),
          ),
        ),
      })}`,
    );
    if (shots) {
      await page
        .locator("#government-panel")
        .screenshot({ path: `${shots}/m4a-demo-government.png` });
      await page.locator("#standing-panel").screenshot({ path: `${shots}/m4a-demo-standing.png` });
      await page.locator("#chronicle").screenshot({ path: `${shots}/m4a-demo-chronicle.png` });
      await page.screenshot({ path: `${shots}/m4a-demo-village.png` });
    }
  } finally {
    await host.stop();
  }
});
