// The M4b demo's pictures (plan §7): a theft from the act to its end. Takings come only where some
// households have food and others none, and the dashboard's valley worlds made no attempt in fifty
// years, so the world is made by civ-sim's `theft_world` example: a band settles for a week, then
// every other household loses the food in its store and the rest keep six days of it. Everything
// after is people's choice. The example lives the lean spell until a case has been found and all
// it imposed is settled, making up to twelve worlds (each draws its own identity, and most lean
// spells end without one) and saving the first where one did, or the last; its line says which.
// The observer then shows the Takings panel, each taking twice, as it happened and as people
// believe it, with what the gathering was told; and the Government panel, the law against taking
// and the gathering's cases. Whatever happened is what is shown. It takes some minutes, so it runs
// only with TCE_DEMO=1, and TCE_SHOTS_DIR saves the screenshots; TCE_DEMO_SEED changes the seed.

import { expect, test } from "@playwright/test";

import { makeTheftWorld, startHost, tempSaves } from "./host.js";

test.skip(!process.env.TCE_DEMO, "the M4b demo runs only with TCE_DEMO=1");
test.setTimeout(60 * 60_000);
test.use({ viewport: { width: 1440, height: 900 } });

const shots = process.env.TCE_SHOTS_DIR;
const NAME = "Lean Valley";

test("a theft from the act to its end: seen, told, brought, decided, owed", async ({ page }) => {
  const saves = tempSaves();
  const made = makeTheftWorld(saves, process.env.TCE_DEMO_SEED ?? "4");
  console.log(`demo world: ${made.trim()}`);
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
        (window.__TCE__.state().order?.minute ?? 0) > 0 &&
        (window.__TCE__.state().government?.polities.length ?? 0) > 0 &&
        window.__TCE__.map().state === "ready",
      NAME,
      { timeout: 300_000 },
    );
    if (!(await page.evaluate(() => window.__TCE__.state().clock?.paused))) {
      await page.getByRole("button", { name: "Pause" }).click();
    }

    // The takings: totals, then each shown as it happened and as people believe it.
    const state = await page.evaluate(() => window.__TCE__.state());
    const order = state.order!;
    const panel = page.locator("#order-body");
    await expect(panel.locator(".order-totals")).toContainText(`${order.attempts} attempt`);
    await expect(panel.locator("li.taking")).toHaveCount(order.incidents.length);
    // The taking brought before the gathering, if one was: its three layers.
    const brought = order.known.find((k) => k.case !== "");
    let story: Record<string, string> = { case: "no taking was brought before the gathering" };
    if (brought) {
      const at = order.incidents.findIndex((i) => i.id === brought.incident);
      expect(at, "the taking brought is among those shown").toBeGreaterThanOrEqual(0);
      const item = panel.locator("li.taking").nth(at);
      await expect(item.locator(".happened")).toContainText(order.incidents[at]!.actor);
      await expect(item.locator(".case")).toContainText(brought.case.slice(1, 40));
      await item.scrollIntoViewIfNeeded();
      story = {
        happened: await item.locator(".happened").innerText(),
        believed: await item.locator(".believed").innerText(),
        case: await item.locator(".case").innerText(),
      };
    }

    // The government: the law against taking, opened to its history, and the cases heard.
    const polity = state.government!.polities[0]!;
    const against = polity.laws.findIndex((l) => l.what.startsWith("a law against taking"));
    let law = "no law against taking was proposed";
    if (against >= 0) {
      const details = page
        .locator("#government-body .polity")
        .first()
        .locator("details.law")
        .nth(against);
      await details.locator("summary").click();
      await expect(details.locator(".law-history")).toContainText("Proposed");
      law = await details.innerText();
    }
    console.log(
      `demo: ${JSON.stringify({
        totals: await panel.locator(".order-totals").innerText(),
        story,
        law,
        laws: polity.laws.map((l) => `${l.what}: ${l.status}; ${l.decision}`),
        chronicle: state.chronicle.filter((t) =>
          ["took", "gave back", "case", "found", "against taking", "sent from"].some((w) =>
            t.includes(w),
          ),
        ),
      })}`,
    );
    if (shots) {
      await page.locator("#order-panel").screenshot({ path: `${shots}/m4b-demo-takings.png` });
      await page
        .locator("#government-panel")
        .screenshot({ path: `${shots}/m4b-demo-government.png` });
      await page.locator("#chronicle").screenshot({ path: `${shots}/m4b-demo-chronicle.png` });
      await page.screenshot({ path: `${shots}/m4b-demo-village.png` });
    }
  } finally {
    await host.stop();
  }
});
