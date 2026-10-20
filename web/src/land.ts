// The land map (M8c slice CX; ADR-0030 §1, §8): every plot, drawn from what is recorded of it
// and labelled so: what was paid for it, what its household reckoned it worth, or nothing
// recorded. The kernel keeps no land value; this only shows what was paid and reckoned.

import type { LandPlot } from "./net/messages.js";

/** A plot paid for, one reckoned worth something, and one with nothing recorded. */
export const PAID_COLOUR = 0xb5651d;
export const RECKONED_COLOUR = 0x2e7d32;
export const NOTHING_COLOUR = 0x8a8a8a;

/** What is recorded of a plot: paid for (whatever was also reckoned), reckoned, or nothing. */
export type LandKind = "paid" | "reckoned" | "nothing";

export function landKind(p: LandPlot): LandKind {
  if (p.paidKcal > 0) return "paid";
  if (p.weighed > 0) return "reckoned";
  return "nothing";
}

/** How a plot is drawn: paid in brown, reckoned in green deeper as it saved more of the most any
 * plot shown saved (`mostH`), nothing recorded only outlined. */
export function landLook(p: LandPlot, mostH: number): { colour: number; fill: number } {
  switch (landKind(p)) {
    case "paid":
      return { colour: PAID_COLOUR, fill: 0.55 };
    case "reckoned":
      return { colour: RECKONED_COLOUR, fill: 0.15 + 0.55 * Math.min(1, p.reckonedH / Math.max(mostH, 1e-6)) };
    default:
      return { colour: NOTHING_COLOUR, fill: 0 };
  }
}

/** The plot under the point, if any, the last laid first. */
export function landAt(plots: LandPlot[], xM: number, yM: number): LandPlot | null {
  for (let i = plots.length - 1; i >= 0; i--) {
    const r = plots[i]!.rect;
    if (xM >= r.x && xM < r.x + r.w && yM >= r.y && yM < r.y + r.h) return plots[i]!;
  }
  return null;
}

/** A plot in words, labelled by what is recorded: "land: paid 3,000 kcal for it", "land: reckoned
 * worth 4.2 hours a year over the next best of 12 weighed", or "land: nothing recorded". */
export function landWords(p: LandPlot): string {
  const parts: string[] = [];
  if (p.paidKcal > 0) parts.push(`paid ${Math.round(p.paidKcal).toLocaleString("en-GB")} kcal for it`);
  if (p.weighed > 0) {
    parts.push(
      `reckoned worth ${p.reckonedH.toFixed(1)} hours a year over the next best of ${p.weighed} weighed`,
    );
  }
  return `land: ${parts.length > 0 ? parts.join("; ") : "nothing recorded"}`;
}
