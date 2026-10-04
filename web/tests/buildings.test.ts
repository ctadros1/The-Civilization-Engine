import { describe, expect, it } from "vitest";

import {
  BUILDING_LEGEND,
  buildingAt,
  buildingKey,
  buildingLook,
  buildingWords,
} from "../src/buildings.js";
import type { BuildingInfo } from "../src/net/messages.js";

function hut(over: Partial<BuildingInfo> = {}): BuildingInfo {
  const posts: [number, number][] = Array.from({ length: 20 }, (_, i) => {
    const a = (i / 20) * 2 * Math.PI;
    return [100 + 3.1 * Math.cos(a), 200 + 3.1 * Math.sin(a)];
  });
  return {
    id: 1,
    household: 2,
    settlement: 3,
    program: "Hut",
    x: 100,
    y: 200,
    radiusM: 3.1,
    roofRadiusM: 3.6,
    door: [103, 200],
    doorDir: 0,
    stage: 0,
    stageName: "foundation",
    progress: 0,
    roofed: false,
    outline: [],
    posts,
    plot: { x: 96.4, y: 196.4, w: 7.2, h: 7.2 },
    floorM2: 30.2,
    sleeps: 5,
    startedMinute: 0,
    status: "ground marked out",
    ...over,
  };
}

describe("buildings on the map", () => {
  it("are named in the readout with their size and state", () => {
    expect(buildingWords(hut())).toBe("hut of 30 m², room for 5: ground marked out");
    expect(buildingWords(hut({ floorM2: 0, status: "finished" }))).toBe("hut: finished");
  });

  it("are drawn by how far their construction has gone", () => {
    expect(buildingKey(hut())).toBe("marked");
    expect(buildingKey(hut({ stage: 1 }))).toBe("frame");
    expect(buildingKey(hut({ stage: 2 }))).toBe("walls");
    expect(buildingKey(hut({ stage: 3 }))).toBe("walls");
    expect(buildingKey(hut({ stage: 4, roofed: true }))).toBe("roofed");
    expect(buildingKey(hut({ stage: 5, roofed: true }))).toBe("roofed");
    const keys = new Set(BUILDING_LEGEND.map((e) => e.key));
    for (const k of ["marked", "frame", "walls", "roofed"]) expect(keys.has(k)).toBe(true);
  });

  it("show postholes as they are dug, then posts, walls and thatch", () => {
    expect(buildingLook(hut({ progress: 0 })).posts).toBe(0);
    expect(buildingLook(hut({ progress: 0.5 })).posts).toBe(10);
    const frame = buildingLook(hut({ stage: 1, progress: 0.2 }));
    expect([frame.posts, frame.wallAlpha, frame.roofAlpha]).toEqual([20, 0, 0]);
    const walls = buildingLook(hut({ stage: 2, progress: 0.5 }));
    expect(walls.wallAlpha).toBeGreaterThan(buildingLook(hut({ stage: 2 })).wallAlpha);
    expect(walls.roofAlpha).toBe(0);
    const thatching = buildingLook(hut({ stage: 3, progress: 0.5 }));
    expect(thatching.roofAlpha).toBeGreaterThan(0);
    expect(thatching.roofAlpha).toBeLessThan(buildingLook(hut({ stage: 4, roofed: true })).roofAlpha);
  });

  it("are found under a point, on their plot or under their roof", () => {
    const buildings = [hut({ id: 1 }), hut({ id: 2, x: 120, plot: null })];
    expect(buildingAt(buildings, 101, 201)?.id).toBe(1);
    // A corner of the plot, outside the roof's reach.
    expect(buildingAt(buildings, 96.6, 196.6)?.id).toBe(1);
    expect(buildingAt(buildings, 122, 200)?.id).toBe(2);
    expect(buildingAt(buildings, 110, 200)).toBeNull();
  });
});
