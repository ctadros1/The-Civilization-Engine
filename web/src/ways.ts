// Ways people have worked on (M8b slice CM; ADR-0029 §1, §9): how a way's stretches are drawn and
// which way the pointer is on. The kernel decides everything, the rank included; this only shows it.

import type { WayInfo, WaySegmentInfo } from "./net/messages.js";

/** Bare earth and shaped earth, gravel, and a timber causeway. */
export const EARTH_COLOUR = 0xb59a6a;
export const GRAVEL_COLOUR = 0xc8c2b0;
export const TIMBER_COLOUR = 0x8a5a2b;

/** How a stretch is drawn: its usable width, coloured by its surface, more solid the better its
 * condition (never fainter than about a third, so an untended way still shows where it ran). A way
 * of higher rank is drawn a little bolder; the rank is shown, never acted on. */
export function wayLook(s: WaySegmentInfo, rank: number): { colour: number; alpha: number; widthM: number } {
  const colour =
    s.surface === "gravel" ? GRAVEL_COLOUR : s.surface === "timber causeway" ? TIMBER_COLOUR : EARTH_COLOUR;
  const condition = Math.min(1, Math.max(0, s.condition));
  const alpha = Math.min(1, (0.35 + 0.5 * condition) * (1 + 0.05 * Math.max(0, rank)));
  return { colour, alpha, widthM: Math.max(0.5, s.usableM) };
}

/** The way the pointer is on: the nearest whose centre line passes within half its usable width
 * and `slackM` of the point, and the stretch it is on. */
export function wayAt(
  list: WayInfo[],
  xM: number,
  yM: number,
  slackM: number,
): { way: WayInfo; segment: WaySegmentInfo } | null {
  let best: { way: WayInfo; segment: WaySegmentInfo } | null = null;
  let bestD = Infinity;
  for (const way of list) {
    for (const segment of way.segments) {
      const p = segment.points;
      for (let i = 0; i < p.length; i++) {
        const [x0, y0] = p[i]!;
        const [x1, y1] = p[Math.min(i + 1, p.length - 1)]!;
        const d = distanceToSegment(xM, yM, x0, y0, x1, y1);
        if (d <= segment.usableM / 2 + slackM && d < bestD) {
          best = { way, segment };
          bestD = d;
        }
      }
    }
  }
  return best;
}

function distanceToSegment(px: number, py: number, x0: number, y0: number, x1: number, y1: number): number {
  const dx = x1 - x0;
  const dy = y1 - y0;
  const len2 = dx * dx + dy * dy;
  const t = len2 === 0 ? 0 : Math.min(1, Math.max(0, ((px - x0) * dx + (py - y0) * dy) / len2));
  return Math.hypot(px - (x0 + t * dx), py - (y0 + t * dy));
}

/** A stretch in words for the pointer: the way in the kernel's words, then "This stretch gravel,
 * 3.0 m of a 6.0 m corridor, kept by Ada's household, in good condition." */
export function wayWords(way: WayInfo, s: WaySegmentInfo): string {
  const condition = s.condition >= 0.75 ? "good" : s.condition >= 0.4 ? "fair" : "poor";
  const kept = s.keeper ? `, kept by ${s.keeper}` : ", kept by nobody";
  return `${way.words} This stretch ${s.surface}, ${s.usableM.toFixed(1)} m of a ${s.corridorM.toFixed(1)} m corridor${kept}, in ${condition} condition.`;
}
