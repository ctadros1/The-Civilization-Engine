// More than one viewer (M12b, ADR-0039 §6): the first page on this machine drives, a second
// watches with its command controls disabled and the reason as their tooltip, and the driver hands
// over to it.

import { expect, test, type Page } from "@playwright/test";

import { startHost, tempSaves, type Host } from "./host.js";

const SPECTATOR_REASON = "You are watching: a spectator may not command; ask the driver to hand over";

async function open(page: Page, host: Host, name: string): Promise<void> {
  page.on("pageerror", (e) => console.log(`page error: ${e.message}`));
  await page.goto(`${host.url}?name=${name}`);
  await page.waitForFunction(() => window.__TCE__?.state().viewers !== null);
}

test("the first page drives, the second watches until the driver hands over", async ({ browser }) => {
  const host = await startHost(tempSaves());
  const ada = await browser.newPage();
  const bo = await browser.newPage();
  try {
    await open(ada, host, "Ada");
    await expect(ada.locator("#connection")).toContainText("You drive");
    await expect(ada.locator("#btn-new")).toBeEnabled();
    await expect(ada.locator("#hand-over")).toBeHidden();

    await open(bo, host, "Bo");
    await expect(bo.locator("#connection")).toContainText("You are watching; Ada drives");
    for (const id of ["#btn-new", "#btn-load", "#btn-run"]) {
      await expect(bo.locator(id)).toBeDisabled();
      await expect(bo.locator(id)).toHaveAttribute("title", SPECTATOR_REASON);
    }
    await expect(bo.locator("#hand-over")).toBeHidden();

    // The driver sees the spectator in its hand-over list, and hands over.
    const handOver = ada.locator("#hand-over");
    await expect(handOver).toBeVisible();
    await expect(handOver.locator("option")).toHaveText(["Hand over…", "to Bo"]);
    await handOver.selectOption({ label: "to Bo" });
    await expect(ada.locator("#connection")).toContainText("You are watching; Bo drives");
    await expect(ada.locator("#btn-new")).toBeDisabled();
    await expect(bo.locator("#connection")).toContainText("You drive");
    await expect(bo.locator("#btn-new")).toBeEnabled();
    await expect(bo.locator("#btn-new")).not.toHaveAttribute("title", SPECTATOR_REASON);

    // The driver leaves: the page left drives on.
    await bo.close();
    await expect(ada.locator("#connection")).toContainText("You drive");
    await expect(ada.locator("#btn-new")).toBeEnabled();
  } finally {
    await ada.close();
    if (!bo.isClosed()) await bo.close();
    await host.stop();
  }
});
