// Water people draw (M6a slice AY; ADR-0021 §1-§3): how a well, a spring and a place at the
// water's edge drawn at today are drawn, which one the pointer is on, and their words. The kernel
// decides everything; this only shows it.

import type { BankDrawInfo, SpringInfo, WellInfo } from "./net/messages.js";

/** Fresh earth round a shaft being dug. */
export const DIGGING_COLOUR = 0x9a7b4f;
/** Water standing in an open well. */
export const WELL_WATER_COLOUR = 0x3d8fd6;
/** A shaft given up dry. */
export const DRY_COLOUR = 0x6b5a45;
/** What is left of a shaft that fell in. */
export const FALLEN_COLOUR = 0x3e342b;
/** A spring. */
export const SPRING_COLOUR = 0x62c3e8;
/** Where people stood at the water's edge to draw. */
export const BANK_COLOUR = 0xd8eef8;

/** How a well is drawn: a shaft being dug fainter the shallower it still is against its target,
 * an open one blue with the water standing in it, one given up dry as a brown pit, and one that
 * fell in as a dark scar. */
export function wellLook(w: WellInfo): { colour: number; alpha: number; radiusM: number } {
  const radiusM = Math.max(0.5, w.radiusM);
  switch (w.state) {
    case 0: {
      const done = w.targetM > 0 ? Math.min(1, Math.max(0, w.depthM / w.targetM)) : 0;
      return { colour: DIGGING_COLOUR, alpha: 0.35 + 0.45 * done, radiusM };
    }
    case 1:
      return { colour: WELL_WATER_COLOUR, alpha: w.waterM > 0.05 ? 0.95 : 0.55, radiusM };
    case 2:
      return { colour: DRY_COLOUR, alpha: 0.75, radiusM };
    default:
      return { colour: FALLEN_COLOUR, alpha: 0.6, radiusM };
  }
}

/** The well the pointer is on: the nearest whose shaft is within `slackM` of the point. */
export function wellAt(list: WellInfo[], xM: number, yM: number, slackM: number): WellInfo | null {
  return nearest(list, xM, yM, (w) => Math.max(0.5, w.radiusM) + slackM);
}

/** The spring the pointer is on: the nearest within `slackM` of the point. */
export function springAt(list: SpringInfo[], xM: number, yM: number, slackM: number): SpringInfo | null {
  return nearest(list, xM, yM, () => slackM);
}

/** The place at the water's edge drawn at today the pointer is on: the nearest within `slackM`. */
export function bankAt(list: BankDrawInfo[], xM: number, yM: number, slackM: number): BankDrawInfo | null {
  return nearest(list, xM, yM, () => slackM);
}

function nearest<T extends { x: number; y: number }>(
  list: T[],
  xM: number,
  yM: number,
  reach: (item: T) => number,
): T | null {
  let best: T | null = null;
  let bestD = Infinity;
  for (const item of list) {
    const d = Math.hypot(item.x - xM, item.y - yM);
    if (d <= reach(item) && d < bestD) {
      best = item;
      bestD = d;
    }
  }
  return best;
}

/** A spring in words: "a spring: 3.2 m³ flows to it a day, 450 L drawn today". */
export function springWords(s: SpringInfo): string {
  const flow = s.flowM3Day >= 10 ? s.flowM3Day.toFixed(0) : s.flowM3Day.toFixed(1);
  const drawn = s.drawnL > 0 ? `${Math.round(s.drawnL)} L drawn today` : "none drawn today";
  return `a spring: ${flow} m³ flows to it a day, ${drawn}`;
}

/** A place at the water's edge in words: "people came for water here 6 times today". */
export function bankWords(b: BankDrawInfo): string {
  const times = b.trips === 1 ? "once" : b.trips === 2 ? "twice" : `${b.trips} times`;
  return `people came for water here ${times} today`;
}

/** The rivers' flow today against their mean in words: "flowing at 40% of its mean today". */
export function flowWords(flow: number): string {
  if (!(flow >= 0)) return "";
  if (Math.abs(flow - 1) < 0.05) return "flowing at its mean today";
  if (flow >= 1.95) return `flowing at ${flow.toFixed(1)} times its mean today`;
  return `flowing at ${Math.round(flow * 100)}% of its mean today`;
}
