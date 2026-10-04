import { describe, expect, it } from "vitest";

import type { PathsInfo, WornTile } from "../src/net/messages.js";
import {
  PATH_LEGEND,
  SHOWN_WEAR,
  TRAIL_COLOUR,
  pathAt,
  pathWords,
  smoothed,
  trailLook,
  trailMetres,
  wornAlpha,
  wornPixels,
} from "../src/paths.js";

function tile(index: number, set: [number, number, boolean][]): WornTile {
  const wear = new Uint8Array(64 * 64);
  const trail = new Uint8Array(64 * 64);
  for (const [k, w, t] of set) {
    wear[k] = w;
    trail[k] = t ? 1 : 0;
  }
  return { index, wear, trail };
}

function paths(worn: WornTile[]): PathsInfo {
  return {
    rev: 3,
    tilesX: 4,
    tileCells: 64,
    worn,
    trails: [
      { points: [[0, 0], [30, 40]], wear: 0.4, lengthM: 50 },
      { points: [[0, 0], [0, 10]], wear: 0.9, lengthM: 10 },
    ],
  };
}

describe("worn ground and trails", () => {
  it("shows ground more plainly the more it is worn, and not at all when barely trodden", () => {
    expect(wornAlpha(0)).toBe(0);
    expect(wornAlpha(SHOWN_WEAR / 2)).toBe(0);
    expect(wornAlpha(0.2)).toBeGreaterThan(0);
    expect(wornAlpha(0.5)).toBeGreaterThan(wornAlpha(0.2));
    expect(wornAlpha(1)).toBe(wornAlpha(0.9));
    expect(wornAlpha(Number.NaN)).toBe(0);
  });

  it("paints a tile cell by cell", () => {
    const px = wornPixels(tile(0, [[0, 255, true], [1, 2, false]]), 64);
    expect(px.length).toBe(64 * 64 * 4);
    expect(px[3]).toBeGreaterThan(0);
    expect(px[4 + 3]).toBe(0);
  });

  it("draws busier trails wider and plainer", () => {
    const busy = trailLook({ points: [], wear: 0.9, lengthM: 1 });
    const faint = trailLook({ points: [], wear: 0.3, lengthM: 1 });
    expect(busy.widthM).toBeGreaterThan(faint.widthM);
    expect(busy.alpha).toBeGreaterThan(faint.alpha);
    expect(PATH_LEGEND.map((p) => p.fill)).toContain(TRAIL_COLOUR);
  });

  it("rounds a trail's corners but keeps its ends", () => {
    const corner: [number, number][] = [
      [0, 0],
      [8, 0],
      [8, 8],
    ];
    const once = smoothed(corner, 1);
    expect(once[0]).toEqual([0, 0]);
    expect(once[once.length - 1]).toEqual([8, 8]);
    expect(once).not.toContainEqual([8, 0]);
    expect(once).toEqual([
      [0, 0],
      [6, 0],
      [8, 2],
      [8, 8],
    ]);
    expect(smoothed([[0, 0], [5, 5]])).toEqual([[0, 0], [5, 5]]);
  });

  it("names the worn ground under the pointer", () => {
    // Tile 5 is the second tile of the second row: cells 64-127 across and down.
    const p = paths([tile(5, [[0, 200, true], [1, 30, false], [2, 2, false]])]);
    expect(pathAt(p, 64, 64)).toEqual({ wear: 200 / 255, trail: true });
    expect(pathWords(pathAt(p, 64, 64)!)).toBe("a trail");
    expect(pathWords(pathAt(p, 65, 64)!)).toBe("worn ground");
    expect(pathAt(p, 66, 64)).toBeNull();
    expect(pathAt(p, 0, 0)).toBeNull();
    expect(pathAt(null, 64, 64)).toBeNull();
    expect(trailMetres(p)).toBe(60);
    expect(trailMetres(null)).toBe(0);
  });
});
