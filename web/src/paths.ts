// Worn ground and trails on the map (M1 slice F): how they look and what lies under a point.
// Pure functions. The kernel decides where the ground is worn and traces the trails; the
// observer only draws them.

import type { PathsInfo, TrailInfo, WornTile } from "./net/messages.js";

/** Trodden earth. */
export const WORN_COLOUR = 0xb69b69;
/** A trail's line. */
export const TRAIL_COLOUR = 0x86653a;

/** The map legend's entries for paths. */
export const PATH_LEGEND: { label: string; fill: number }[] = [
  { label: "Worn ground", fill: WORN_COLOUR },
  { label: "Trail", fill: TRAIL_COLOUR },
];

/** Wear (0–1) below which ground does not show as worn. */
export const SHOWN_WEAR = 0.06;
/** Wear at which worn ground is as plain as it gets. */
const FULL_WEAR = 0.6;
const MAX_ALPHA = 0.6;

/** Opacity of ground with wear `w` (0–1). */
export function wornAlpha(w: number): number {
  if (!(w >= SHOWN_WEAR)) return 0;
  return Math.min(1, (w - SHOWN_WEAR) / (FULL_WEAR - SHOWN_WEAR)) * MAX_ALPHA;
}

/** A tile's worn ground as RGBA pixels, one per cell, row by row. */
export function wornPixels(tile: WornTile, tileCells: number): Uint8ClampedArray {
  const n = tileCells * tileCells;
  const out = new Uint8ClampedArray(n * 4);
  const [r, g, b] = [(WORN_COLOUR >> 16) & 255, (WORN_COLOUR >> 8) & 255, WORN_COLOUR & 255];
  for (let i = 0; i < n; i++) {
    const a = wornAlpha((tile.wear[i] ?? 0) / 255);
    if (a === 0) continue;
    out[i * 4] = r;
    out[i * 4 + 1] = g;
    out[i * 4 + 2] = b;
    out[i * 4 + 3] = Math.round(a * 255);
  }
  return out;
}

/** How a trail is drawn: its width in metres (wider and plainer the more worn) and opacity. */
export function trailLook(t: TrailInfo): { widthM: number; alpha: number } {
  const w = Math.min(1, Math.max(0, t.wear));
  return { widthM: 1.2 + 1.3 * w, alpha: 0.45 + 0.45 * w };
}

/**
 * A polyline with its corners rounded for drawing (Chaikin's corner cutting, `rounds` times); the
 * ends stay where they are.
 */
export function smoothed(points: [number, number][], rounds = 2): [number, number][] {
  let pts = points;
  for (let r = 0; r < rounds && pts.length > 2; r++) {
    const out: [number, number][] = [pts[0]!];
    for (let i = 0; i + 1 < pts.length; i++) {
      const [ax, ay] = pts[i]!;
      const [bx, by] = pts[i + 1]!;
      if (i > 0) out.push([0.75 * ax + 0.25 * bx, 0.75 * ay + 0.25 * by]);
      if (i + 2 < pts.length) out.push([0.25 * ax + 0.75 * bx, 0.25 * ay + 0.75 * by]);
    }
    out.push(pts[pts.length - 1]!);
    pts = out;
  }
  return pts;
}

/** The worn ground under a cell, or null where it is untrodden. */
export function pathAt(
  paths: PathsInfo | null,
  cellX: number,
  cellY: number,
): { wear: number; trail: boolean } | null {
  if (!paths || paths.tileCells <= 0 || cellX < 0 || cellY < 0) return null;
  const tc = paths.tileCells;
  const index = Math.floor(cellY / tc) * paths.tilesX + Math.floor(cellX / tc);
  const tile = paths.worn.find((t) => t.index === index);
  if (!tile) return null;
  const k = (cellY % tc) * tc + (cellX % tc);
  const wear = (tile.wear[k] ?? 0) / 255;
  const trail = tile.trail[k] === 1;
  if (!trail && wear < SHOWN_WEAR) return null;
  return { wear, trail };
}

/** The readout's words for worn ground. */
export function pathWords(p: { wear: number; trail: boolean }): string {
  return p.trail ? "a trail" : "worn ground";
}

/** Total length of the trails, metres. */
export function trailMetres(paths: PathsInfo | null): number {
  return paths ? paths.trails.reduce((m, t) => m + t.lengthM, 0) : 0;
}
