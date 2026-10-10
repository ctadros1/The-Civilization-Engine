// Crossings over water (M5c slice AW; ADR-0004 §7, ADR-0009 §9): where a footbridge's members
// lie, how one is drawn, and which one the pointer is on. The kernel decides everything; this only
// shows it.

import type { CrossingInfo } from "./net/messages.js";

/** Where a crossing's members lie, metres, as [x0, y0, x1, y1]: `lengthM` long, centred between
 * its banks' centres and along the line from one to the other. */
export function crossingEnds(c: CrossingInfo): [number, number, number, number] {
  const mx = (c.ax + c.bx) / 2;
  const my = (c.ay + c.by) / 2;
  const dx = c.bx - c.ax;
  const dy = c.by - c.ay;
  const d = Math.hypot(dx, dy);
  if (d === 0) return [mx, my, mx, my];
  const half = (c.lengthM > 0 ? c.lengthM : d) / 2;
  const ux = dx / d;
  const uy = dy / d;
  return [mx - ux * half, my - uy * half, mx + ux * half, my + uy * half];
}

/** How wide a crossing's deck is, metres: its members side by side. */
export function crossingWidthM(c: CrossingInfo): number {
  return (Math.max(1, c.members) * c.diameterCm) / 100;
}

/** The crossing the pointer is on: the nearest whose members pass within half their width and
 * `slackM` of the point. */
export function crossingAt(list: CrossingInfo[], xM: number, yM: number, slackM: number): CrossingInfo | null {
  let best: CrossingInfo | null = null;
  let bestD = Infinity;
  for (const c of list) {
    const [x0, y0, x1, y1] = crossingEnds(c);
    const d = distanceToSegment(xM, yM, x0, y0, x1, y1);
    if (d <= crossingWidthM(c) / 2 + slackM && d < bestD) {
      best = c;
      bestD = d;
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

/** Sound timber. */
export const TIMBER_COLOUR = 0x8a5a2b;
/** Timber gone grey with rot. */
export const ROTTEN_COLOUR = 0x6e6a5e;
/** What is left of one that gave way. */
export const WRECK_COLOUR = 0x3e342b;

/** How a crossing is drawn: one being built fainter the less of its work is done, an open one solid
 * timber greying as rot takes its section, and one that gave way as a dark wreck. */
export function crossingLook(c: CrossingInfo): { colour: number; alpha: number; widthM: number } {
  const widthM = crossingWidthM(c);
  if (c.state === 0) {
    const done = c.labourH > 0 ? Math.min(1, Math.max(0, c.workH / c.labourH)) : 0;
    return { colour: TIMBER_COLOUR, alpha: 0.25 + 0.45 * done, widthM };
  }
  if (c.state === 2) return { colour: WRECK_COLOUR, alpha: 0.5, widthM };
  const loss = Math.min(1, Math.max(0, c.loss));
  return { colour: mix(TIMBER_COLOUR, ROTTEN_COLOUR, loss), alpha: 0.95, widthM };
}

function mix(a: number, b: number, t: number): number {
  const channel = (shift: number): number => {
    const x = (a >> shift) & 0xff;
    const y = (b >> shift) & 0xff;
    return Math.round(x + (y - x) * t) << shift;
  };
  return channel(16) | channel(8) | channel(0);
}
