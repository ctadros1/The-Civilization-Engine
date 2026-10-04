// Earthworks (M3b slice Q; ADR-0010 §2-3): where people have changed the ground, how it is drawn,
// what the readout says of it, and which map tiles to read again when the ground changes. The
// kernel decides everything; this only shows it.

import type { EarthworkInfo, EarthworksInfo } from "./net/messages.js";

/** The earthwork under a point, metres: the smallest whose rectangle holds it. */
export function earthworkAt(works: EarthworkInfo[], xM: number, yM: number): EarthworkInfo | null {
  let best: EarthworkInfo | null = null;
  for (const w of works) {
    const inside = xM >= w.x && xM <= w.x + w.w && yM >= w.y && yM <= w.y + w.h;
    if (inside && (!best || w.w * w.h < best.w * best.h)) best = w;
  }
  return best;
}

/** Bare earth: a platform's. */
export const EARTH_COLOUR = 0x9c7a52;
/** A pit's open ground, darker and deeper. */
export const PIT_COLOUR = 0x4e3b28;
/** A spoil heap's loose earth, paler. */
export const SPOIL_COLOUR = 0xc2a57c;

/** How an earthwork is drawn: a platform as bare earth, more solid the more of it is done and
 * always outlined, so one not yet begun still shows where it will be; a pit dark and a spoil heap
 * pale, both solid once there is earth in them. */
export function earthworkLook(w: EarthworkInfo): { colour: number; alpha: number; outlineAlpha: number } {
  if (w.kind === 1) return { colour: PIT_COLOUR, alpha: w.cutM3 > 0 ? 0.7 : 0.2, outlineAlpha: 0.9 };
  if (w.kind === 2) return { colour: SPOIL_COLOUR, alpha: w.cutM3 > 0 ? 0.6 : 0.15, outlineAlpha: 0.6 };
  const done = Math.min(1, Math.max(0, w.done));
  return { colour: EARTH_COLOUR, alpha: 0.12 + 0.33 * done, outlineAlpha: 0.75 };
}

/** The ground tiles whose revision is not the one last seen, `seen` keyed by tile index. */
export function changedTiles(seen: ReadonlyMap<number, number>, now: EarthworksInfo): number[] {
  return now.tiles.filter((t) => seen.get(t.index) !== t.rev).map((t) => t.index);
}

/** The keys ("tx,ty") of the map's detail tiles, `detailCells` cells a side, whose shading reads
 * ground tile `index` of `tileCells` cells a side on a map `tilesX` ground tiles across. A detail
 * tile is shaded with a cell of margin, so the ground tile is widened by a cell either way. */
export function detailTilesOver(index: number, tileCells: number, tilesX: number, detailCells: number): string[] {
  if (tileCells <= 0 || tilesX <= 0 || detailCells <= 0) return [];
  const gx = (index % tilesX) * tileCells;
  const gy = Math.floor(index / tilesX) * tileCells;
  const x0 = Math.floor(Math.max(0, gx - 1) / detailCells);
  const y0 = Math.floor(Math.max(0, gy - 1) / detailCells);
  const x1 = Math.floor((gx + tileCells) / detailCells);
  const y1 = Math.floor((gy + tileCells) / detailCells);
  const keys: string[] = [];
  for (let ty = y0; ty <= y1; ty++) for (let tx = x0; tx <= x1; tx++) keys.push(`${tx},${ty}`);
  return keys;
}
