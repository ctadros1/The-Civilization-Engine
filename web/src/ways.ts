// Ways people have worked on (M8b slice CM; ADR-0029 §1, §9): how a way's stretches are drawn and
// which way the pointer is on. The kernel decides everything, the rank included; this only shows it.

import type { WayInfo, WaySegmentInfo } from "./net/messages.js";

/** Bare earth and shaped earth, gravel, and a timber causeway. */
export const EARTH_COLOUR = 0xb59a6a;
export const GRAVEL_COLOUR = 0xc8c2b0;
export const TIMBER_COLOUR = 0x8a5a2b;
/** A line of track's graded bed, and its timber rails (M10b slice DV). */
export const BED_COLOUR = 0x9b8461;
export const RAIL_COLOUR = 0x4a3220;
/** A line's station at each of its ends, once its stations are open (M10b slice DX). */
export const STATION_COLOUR = 0x6b4f3a;

/** Where a line's stations stand, once open (M10b slice DX): the first point of its first stretch
 * and the last of its last; none for a way that is not a line or whose stations are shut. */
export function stationEnds(way: WayInfo): Array<[number, number]> {
  if (way.gaugeM <= 0 || !way.stations) return [];
  const first = way.segments[0]?.points[0];
  const lastSeg = way.segments[way.segments.length - 1];
  const last = lastSeg?.points[lastSeg.points.length - 1];
  return [first, last].filter((p): p is [number, number] => p !== undefined);
}

/** How a stretch of a line of track is drawn: its graded bed the width of its corridor, fainter
 * the less of it is laid (never fainter than about a quarter, so a line marked out shows), and
 * its rails at its gauge once the whole line is laid. */
export function lineLook(
  way: WayInfo,
  s: WaySegmentInfo,
): { colour: number; alpha: number; widthM: number; railColour: number; gaugeM: number | null } {
  const laid = Math.min(1, Math.max(0, way.laid));
  return {
    colour: BED_COLOUR,
    alpha: 0.25 + 0.6 * laid,
    widthM: Math.max(0.5, s.corridorM),
    railColour: RAIL_COLOUR,
    gaugeM: laid >= 1 ? way.gaugeM : null,
  };
}

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
        // A line of track is found over its bed, the rest over the width usable today.
        const half = (way.gaugeM > 0 ? segment.corridorM : segment.usableM) / 2;
        if (d <= half + slackM && d < bestD) {
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
 * 3.0 m of a 6.0 m corridor, kept by Ada's household, in good condition." A line of track: its
 * words, then "Timber track on a 2.0 m bed, kept by Ada's household." */
export function wayWords(way: WayInfo, s: WaySegmentInfo): string {
  const kept = s.keeper ? `, kept by ${s.keeper}` : ", kept by nobody";
  if (way.gaugeM > 0) {
    const surface = s.surface.charAt(0).toUpperCase() + s.surface.slice(1);
    return `${way.words} ${surface} on a ${s.corridorM.toFixed(1)} m bed${kept}.`;
  }
  const condition = s.condition >= 0.75 ? "good" : s.condition >= 0.4 ? "fair" : "poor";
  return `${way.words} This stretch ${s.surface}, ${s.usableM.toFixed(1)} m of a ${s.corridorM.toFixed(1)} m corridor${kept}, in ${condition} condition.`;
}
