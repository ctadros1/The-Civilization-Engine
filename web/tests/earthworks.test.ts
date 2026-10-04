import { describe, expect, it } from "vitest";

import { changedTiles, detailTilesOver, earthworkAt, earthworkLook } from "../src/earthworks.js";
import type { EarthworkInfo, EarthworksInfo } from "../src/net/messages.js";

function work(over: Partial<EarthworkInfo> = {}): EarthworkInfo {
  return {
    id: 1,
    kind: 0,
    x: 100,
    y: 200,
    w: 6,
    h: 6,
    levelM: 42.3,
    sideRun: 1.5,
    cutM3: 6.4,
    done: 0.4,
    household: 7,
    plot: 9,
    building: 11,
    words: "the plot of Ada's hut, being levelled: 40% of 6.4 m³ cut and filled",
    ...over,
  };
}

function info(tiles: { index: number; rev: number }[]): EarthworksInfo {
  return { rev: 1, works: [], tileCells: 64, tilesX: 16, tiles };
}

describe("earthworks", () => {
  it("finds the platform under a point, the smaller when two overlap", () => {
    const small = work({ id: 1, x: 100, y: 200, w: 6, h: 6 });
    const large = work({ id: 2, x: 98, y: 198, w: 12, h: 12 });
    expect(earthworkAt([large, small], 103, 203)?.id).toBe(1);
    expect(earthworkAt([large, small], 99, 199)?.id).toBe(2);
    expect(earthworkAt([large, small], 120, 203)).toBeNull();
  });

  it("draws a platform more solid as more of it is done, and outlines one not yet begun", () => {
    const begun = earthworkLook(work({ done: 0 }));
    const half = earthworkLook(work({ done: 0.5 }));
    const done = earthworkLook(work({ done: 1 }));
    expect(begun.alpha).toBeLessThan(half.alpha);
    expect(half.alpha).toBeLessThan(done.alpha);
    expect(begun.outlineAlpha).toBeGreaterThan(0);
    expect(begun.colour).toBe(done.colour);
  });

  it("names the ground tiles whose revision changed since last seen", () => {
    const seen = new Map([
      [3, 4],
      [5, 1],
    ]);
    expect(changedTiles(seen, info([{ index: 3, rev: 4 }, { index: 5, rev: 2 }, { index: 8, rev: 1 }]))).toEqual([
      5, 8,
    ]);
  });

  it("names the detail tiles whose shading reads a ground tile, a cell either side", () => {
    // Ground tile 0 is cells 0-63, inside detail tile 0,0 even with its margin.
    expect(detailTilesOver(0, 64, 16, 256)).toEqual(["0,0"]);
    // Ground tile 3 is cells 192-255; its margin reaches cell 256, in the next detail tile.
    expect(detailTilesOver(3, 64, 16, 256)).toEqual(["0,0", "1,0"]);
    // Ground tile 4 (cells 256-319) reaches back to cell 255.
    expect(detailTilesOver(4, 64, 16, 256)).toEqual(["0,0", "1,0"]);
    // Row 4's first tile, cells 256-319 down: detail rows 0 and 1.
    expect(detailTilesOver(4 * 16, 64, 16, 256)).toEqual(["0,0", "0,1"]);
    expect(detailTilesOver(0, 0, 16, 256)).toEqual([]);
  });
});
