import { describe, expect, it } from "vitest";

import {
  formatBytes,
  formatDistance,
  formatSimMinute,
  formatSpeed,
  simDate,
} from "../src/format.js";
import {
  WATER_LAKE,
  WATER_LAND,
  WATER_OCEAN,
  baseLevel,
  classes,
  contourInterval,
  elevations,
  shade,
} from "../src/map/shade.js";
import {
  RasterFormat,
  RasterLayer,
  type EventItem,
  type RasterTile,
} from "../src/net/messages.js";
import { MAX_EVENTS, mergeEvents } from "../src/state.js";

describe("calendar", () => {
  it("matches civ-core's 365-day calendar", () => {
    // civ-core DEFAULT_WORLD_START: year 1, March 1, 06:00.
    expect(simDate(85_320)).toEqual({ year: 1, month: 3, day: 1, hour: 6, minute: 0 });
    expect(formatSimMinute(85_320)).toBe("Year 1 · Mar 1 · 06:00");
    expect(simDate(0)).toEqual({ year: 1, month: 1, day: 1, hour: 0, minute: 0 });
    // The last minute of February and of the year.
    expect(simDate(59 * 1440 - 1)).toEqual({ year: 1, month: 2, day: 28, hour: 23, minute: 59 });
    expect(simDate(365 * 1440 - 1)).toEqual({
      year: 1,
      month: 12,
      day: 31,
      hour: 23,
      minute: 59,
    });
    expect(simDate(365 * 1440)).toEqual({ year: 2, month: 1, day: 1, hour: 0, minute: 0 });
  });

  it("formats speeds, distances and sizes", () => {
    expect(formatSpeed(96, 96)).toBe("1×");
    expect(formatSpeed(960, 96)).toBe("10×");
    expect(formatDistance(850)).toBe("850 m");
    expect(formatDistance(16_384)).toBe("16.4 km");
    expect(formatBytes(15_121_098)).toBe("14.4 MB");
    expect(formatBytes(900)).toBe("900 B");
  });
});

function tile(format: RasterFormat, width: number, height: number, data: number[]): RasterTile {
  return {
    layer: RasterLayer.Elevation,
    level: 0,
    x0: 0,
    y0: 0,
    width,
    height,
    format,
    scale: 0.5,
    offset: 10,
    data: new Uint8Array(data),
    fullWidth: width,
    fullHeight: height,
  };
}

describe("map shading", () => {
  it("chooses a base level that fits 1024 samples a side", () => {
    expect(baseLevel(256, 256)).toBe(0);
    expect(baseLevel(1024, 1024)).toBe(0);
    expect(baseLevel(2048, 2048)).toBe(1);
    expect(baseLevel(4096, 1024)).toBe(2);
  });

  it("dequantises elevation and reads water classes", () => {
    // U16 little-endian: 4 and 258.
    expect(Array.from(elevations(tile(RasterFormat.U16, 2, 1, [4, 0, 2, 1])))).toEqual([12, 139]);
    expect(() => elevations(tile(RasterFormat.U8, 2, 1, [1, 2]))).toThrow();
    expect(Array.from(classes(tile(RasterFormat.U8, 2, 1, [0, 3])))).toEqual([0, 3]);
  });

  it("picks round contour intervals", () => {
    expect(contourInterval(550)).toBe(50);
    expect(contourInterval(160)).toBe(10);
    expect(contourInterval(0)).toBe(0);
  });

  it("colours water by class and lights slopes facing the north-west", () => {
    const w = 5;
    const h = 1;
    const opts = { cellM: 8, seaLevelM: 0, minM: -20, maxM: 100 };
    const water = new Uint8Array([WATER_OCEAN, WATER_LAKE, WATER_LAND, WATER_LAND, WATER_LAND]);
    const flat = shade(new Float32Array([-10, 50, 50, 50, 50]), water, w, h, opts);
    expect(flat.length).toBe(w * h * 4);
    // Ocean and lake are blue-dominant; land is not.
    expect(flat[2]!).toBeGreaterThan(flat[0]!);
    expect(flat[6]!).toBeGreaterThan(flat[4]!);
    expect(flat[13]!).toBeGreaterThanOrEqual(flat[14]!);
    // Ground rising to the east faces west and is lit more than ground falling to the east.
    const rising = shade(new Float32Array([0, 0, 40, 80, 120]), new Uint8Array(5), w, h, opts);
    const falling = shade(new Float32Array([120, 80, 40, 0, 0]), new Uint8Array(5), w, h, opts);
    expect(rising[12]! / falling[12]!).toBeGreaterThan(1);
  });
});

describe("event log", () => {
  const event = (id: number): EventItem => ({
    id,
    simMinute: 0,
    unixMs: 0,
    kind: "info",
    text: `${id}`,
  });

  it("skips events it already has and keeps the newest", () => {
    const merged = mergeEvents([event(1), event(2)], [event(2), event(3)]);
    expect(merged.map((e) => e.id)).toEqual([1, 2, 3]);
    const many = Array.from({ length: MAX_EVENTS + 10 }, (_, i) => event(i + 1));
    const capped = mergeEvents([], many);
    expect(capped.length).toBe(MAX_EVENTS);
    expect(capped[0]!.id).toBe(11);
  });
});
