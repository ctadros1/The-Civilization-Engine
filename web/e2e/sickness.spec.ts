// M6a slice AZ, end to end (ADR-0021 §5, §8): the observer brings cholera to someone from the
// inspector, as if they took it elsewhere. The inspector tells their infection in the kernel's
// words and the influence it was, the chronicle records it, and a second time is refused while it
// runs in them. Whether it goes further is the disease's and people's.

import { expect, test } from "@playwright/test";

import { makeWorld, startHost, tempSaves } from "./host.js";

const shots = process.env.TCE_SHOTS_DIR;

test("the observer brings cholera to someone, and the inspector tells of it", async ({ page }) => {
  const saves = tempSaves();
  makeWorld(saves, ["--seed", "3", "--size", "512", "--days", "2", "--name", "Fever Valley"]);
  const host = await startHost(saves);
  try {
    page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
    await page.goto(host.url);
    await page.waitForFunction(() => window.__TCE__?.state().connection === "open");
    await page.getByRole("button", { name: "Load", exact: true }).click();
    await page.locator("#load-body tbody tr").first().getByRole("button", { name: "Load" }).click();
    await page.waitForFunction(() => window.__TCE__.state().world?.name === "Fever Valley", undefined, {
      timeout: 60_000,
    });
    if (!(await page.evaluate(() => window.__TCE__.state().clock?.paused))) {
      await page.getByRole("button", { name: "Pause" }).click();
    }

    // Someone who has never been ill.
    const someone = (await page.evaluate(() => window.__TCE__.briefs())).find((p) => p.ageYears >= 20)!;
    await page.evaluate((id) => window.__TCE__.select(id), someone.id);
    const inspector = page.locator("#people-body");
    await expect(inspector.locator(".observer-hand")).toBeVisible();
    await expect(inspector.locator(".sickness")).toHaveCount(0);
    const name = await inspector.locator(".person-head h3").textContent();

    // The observer brings them cholera.
    await inspector.getByLabel("Bring").selectOption({ label: "Cholera" });
    await inspector.getByRole("button", { name: "Bring it" }).click();
    await expect(page.locator("#banner-text")).toContainText(`${name} has cholera, as if they took it elsewhere`);
    const sickness = inspector.locator(".sickness li");
    await expect(sickness).toHaveCount(1);
    // Most who take cholera are never ill (content's prior: a quarter fall ill); either way the
    // kernel says which.
    await expect(sickness).toContainText(/^Took cholera today( and is not ill|; ill (today|tomorrow|in \d+ days))/);
    await expect(sickness).toContainText("brought by the observer, as if they took it elsewhere");
    await expect(sickness).toContainText(/the only case so far of an outbreak at \w+\.$/);
    const reached = inspector.locator(".observer-hand .influences li");
    await expect(reached).toHaveCount(1);
    await expect(reached).toContainText("Brought cholera");
    await expect(page.locator("#chronicle")).toContainText(
      `One recorded influence: the observer brought a disease to ${name}: cholera, as if they took it elsewhere.`,
    );
    if (shots) await inspector.screenshot({ path: `${shots}/sickness-inspector.png` });

    // A second time is refused while it runs in them, and nothing more is recorded.
    await inspector.getByLabel("Bring").selectOption({ label: "Cholera" });
    await inspector.getByRole("button", { name: "Bring it" }).click();
    await expect(page.locator("#banner-text")).toContainText("cannot take cholera now");
    await expect(reached).toHaveCount(1);
  } finally {
    await host.stop();
  }
});
