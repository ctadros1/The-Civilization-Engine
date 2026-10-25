import { describe, expect, it } from "vitest";

import type { GasMainInfo, StreetLampInfo, WayInfo, WaySegmentInfo } from "../src/net/messages.js";
import {
  BED_COLOUR,
  BROKEN_MAIN_COLOUR,
  EARTH_COLOUR,
  GRAVEL_COLOUR,
  LAMP_DARK_COLOUR,
  LAMP_LIT_COLOUR,
  MAIN_COLOUR,
  lampAt,
  lampLook,
  lineLook,
  mainAt,
  mainLook,
  stationEnds,
  wayAt,
  wayLook,
  wayWords,
} from "../src/ways.js";

function segment(over: Partial<WaySegmentInfo> = {}): WaySegmentInfo {
  return {
    points: [
      [0, 0],
      [80, 0],
    ],
    corridorM: 6,
    builtM: 2,
    usableM: 2,
    surface: "cleared ground",
    condition: 1,
    grade: 0.02,
    keeper: "Ada's household",
    ...over,
  };
}

function way(over: Partial<WayInfo> = {}): WayInfo {
  return {
    id: 1,
    rank: 0,
    rankName: "lane",
    modes: 1,
    tripsADay: 4,
    segments: [segment()],
    words: "Lane of 80 m, 2.0 m wide at most, about 4 trips a day; begun in year 1.",
    gaugeM: 0,
    ruling: 0,
    laid: 1,
    stations: false,
    runsADay: 0,
    fareKg: 0,
    riders: 0,
    freightKgDay: 0,
    ...over,
  };
}

describe("ways", () => {
  it("draws a stretch its usable width, by its surface, fading with its condition", () => {
    const good = wayLook(segment(), 0);
    expect(good.colour).toBe(EARTH_COLOUR);
    expect(good.widthM).toBe(2);
    const worn = wayLook(segment({ condition: 0.1 }), 0);
    expect(worn.alpha).toBeLessThan(good.alpha);
    expect(worn.alpha).toBeGreaterThan(0.3);
    expect(wayLook(segment({ surface: "gravel" }), 0).colour).toBe(GRAVEL_COLOUR);
    expect(wayLook(segment(), 3).alpha).toBeGreaterThan(good.alpha);
  });

  it("finds the way under a point within its width, the nearer of two, none off its line", () => {
    const a = way();
    const b = way({ id: 2, segments: [segment({ points: [[0, 6], [80, 6]] })] });
    expect(wayAt([a, b], 40, 1, 0.5)?.way.id).toBe(1);
    expect(wayAt([a, b], 40, 5, 0.5)?.way.id).toBe(2);
    expect(wayAt([a, b], 40, 20, 0.5)).toBeNull();
    expect(wayAt([a], 90, 0, 0.5)).toBeNull();
  });

  it("tells a stretch in words with its keeper and condition", () => {
    const a = way();
    expect(wayWords(a, a.segments[0]!)).toBe(
      `${a.words} This stretch cleared ground, 2.0 m of a 6.0 m corridor, kept by Ada's household, in good condition.`,
    );
    expect(wayWords(a, segment({ keeper: "", condition: 0.2 }))).toContain("kept by nobody, in poor condition.");
  });

  it("marks a line's stations at its two ends once they are open, and none before", () => {
    const first = segment({ points: [[0, 0], [40, 0]] });
    const second = segment({ points: [[40, 0], [80, 10]] });
    const line = way({ modes: 4, gaugeM: 1, segments: [first, second] });
    expect(stationEnds(line)).toEqual([]);
    expect(stationEnds({ ...line, stations: true })).toEqual([
      [0, 0],
      [80, 10],
    ]);
    // A way that is not a line has none.
    expect(stationEnds(way({ stations: true }))).toEqual([]);
  });

  it("draws a line of track as its bed, faint while being laid, with rails once whole", () => {
    const bed = segment({ surface: "timber track", corridorM: 2, builtM: 1, usableM: 0.05 });
    const marked = way({ modes: 4, gaugeM: 1, ruling: 0.003, laid: 0, segments: [bed] });
    const half = lineLook({ ...marked, laid: 0.5 }, bed);
    expect(half.colour).toBe(BED_COLOUR);
    expect(half.widthM).toBe(2);
    expect(half.gaugeM).toBeNull();
    expect(lineLook(marked, bed).alpha).toBeCloseTo(0.25);
    expect(half.alpha).toBeLessThan(lineLook({ ...marked, laid: 1 }, bed).alpha);
    expect(lineLook({ ...marked, laid: 1 }, bed).gaugeM).toBe(1);
  });

  it("finds a line of track over its bed and tells it with its bed and keeper", () => {
    const bed = segment({ surface: "timber track", corridorM: 2, builtM: 2, usableM: 2 });
    const line = way({
      modes: 4,
      gaugeM: 1,
      ruling: 0.003,
      laid: 0.4,
      segments: [{ ...bed, usableM: 0.05 }],
      words: "Wagonway of 80 m, gauge 1.0 m, graded to 1 in 333 at its steepest, 40% laid; begun in year 1.",
    });
    // Not yet usable, it is still found across its bed.
    expect(wayAt([line], 40, 0.9, 0.1)?.way.id).toBe(1);
    expect(wayWords(line, line.segments[0]!)).toBe(`${line.words} Timber track on a 2.0 m bed, kept by Ada's household.`);
  });

  it("reads out a line's timetable, fare and freight as the kernel words them", () => {
    const bed = segment({ surface: "timber track", corridorM: 2, builtM: 2, usableM: 2 });
    const line = way({
      modes: 4,
      gaugeM: 1,
      laid: 1,
      segments: [bed],
      stations: true,
      runsADay: 2,
      fareKg: 0.6,
      riders: 14,
      freightKgDay: 380,
      words:
        "Wagonway of 80 m, gauge 1.0 m, laid; begun in year 1. Stations at both ends; 2 trains a day; a ride costs 0.6 kg of coal; 14 rides and 380 kg brought a day since its review.",
    });
    const readout = wayWords(line, bed);
    expect(readout).toContain("2 trains a day; a ride costs 0.6 kg of coal; 14 rides and 380 kg brought a day");
    expect(readout.endsWith("Timber track on a 2.0 m bed, kept by Ada's household.")).toBe(true);
  });

  it("draw a gas main fainter while it is laid and red once broken, and street lamps by whether they burn (M10b slice DY)", () => {
    const main = (over: Partial<GasMainInfo> = {}): GasMainInfo => ({
      id: 81,
      works: 7,
      points: [
        [10, 10],
        [70, 10],
      ],
      lengthM: 60,
      laid: 1,
      capacityKgH: 1,
      broken: false,
      homes: 1,
      lit: 1,
      words: "Gas main of 60 m from Ada's gasworks, laid, carrying up to 1.0 kg of coal gas an hour; 1 home connected, 1 lit now.",
      ...over,
    });
    const whole = mainLook(main());
    const marked = mainLook(main({ laid: 0 }));
    expect(whole.colour).toBe(MAIN_COLOUR);
    expect(marked.alpha).toBeLessThan(whole.alpha);
    expect(marked.alpha).toBeGreaterThan(0.2);
    expect(mainLook(main({ broken: true })).colour).toBe(BROKEN_MAIN_COLOUR);
    const lamp = (lit: boolean): StreetLampInfo => ({ x: 40, y: 12, lit, words: lit ? "burning" : "dark" });
    expect(lampLook(lamp(true))).toEqual({ colour: LAMP_LIT_COLOUR, haloM: 6 });
    expect(lampLook(lamp(false))).toEqual({ colour: LAMP_DARK_COLOUR, haloM: 0 });
    // Found under the pointer: the main along its route, the lamp at its post.
    expect(mainAt([main()], 40, 11, 2)?.id).toBe(81);
    expect(mainAt([main()], 40, 20, 2)).toBeNull();
    expect(lampAt([lamp(true)], 41, 12, 2)?.words).toBe("burning");
    expect(lampAt([lamp(true)], 40, 20, 2)).toBeNull();
  });
});
