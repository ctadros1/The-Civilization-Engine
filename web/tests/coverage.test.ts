import { describe, expect, it } from "vitest";

import {
  NONE_COLOUR,
  REACH_BANDS,
  TRAFFIC_CLASSES,
  homeAt,
  kindColour,
  legend,
  lensId,
  lensMarks,
  placeAt,
  reachBand,
  trafficClass,
} from "../src/coverage.js";
import type { CoverageHome, CoverageInfo, CoveragePlace, CoverageSource } from "../src/net/messages.js";

function source(kind: number, x: number, minutes: number | null, days = 0): CoverageSource {
  return { name: `source ${kind}`, x, y: 0, kind, minutes, days };
}

function home(x: number, nearest: CoverageSource | null, drew: CoverageSource[]): CoverageHome {
  return { household: x, name: `home ${x}`, x, y: 0, settlement: 1, nearest, byRight: null, drew, words: [] };
}

const info: CoverageInfo = {
  lens: 1,
  title: "Water",
  scope: "Ashford: 3 homes",
  periods: ["now, on the paths as last surveyed", "the past 30 days, today included"],
  denominator: "homes: households with people, each at its home",
  homes: [
    home(0, source(1, 2, 0.3), [source(1, 2, 0.3, 30), source(5, 50, 3, 3)]),
    home(100, source(5, 140, 7), []),
    home(200, null, []),
  ],
  limitMin: 60,
  days: 30,
  summary: ["Could reach: of 3 homes, …", "Did come: in the past 30 days, of 3 homes, …"],
  places: [],
};

describe("the Water lens", () => {
  it("bands a walk on a fixed scale, and none within the search apart from any walk", () => {
    expect(reachBand(0.2)).toBe(0);
    expect(reachBand(1)).toBe(1);
    expect(reachBand(14.9)).toBe(2);
    expect(reachBand(40)).toBe(3);
    expect(reachBand(null)).toBe(-1);
  });

  it("draws could reach as each home by its band with a line to its nearest water, and a home without as hollow", () => {
    const marks = lensMarks(info, "reach");
    const homes = marks.filter((m) => m.type === "home");
    expect(homes.map((m) => (m.type === "home" ? [m.x, m.colour, m.hollow] : null))).toEqual([
      [0, REACH_BANDS[0]!.colour, false],
      [100, REACH_BANDS[2]!.colour, false],
      [200, NONE_COLOUR, true],
    ]);
    expect(marks.filter((m) => m.type === "line")).toHaveLength(2);
  });

  it("draws did come as a line to each source drawn at, as strong as its share of the days", () => {
    const marks = lensMarks(info, "came");
    const lines = marks.filter((m) => m.type === "line");
    expect(lines).toHaveLength(2);
    const [every, few] = lines;
    if (every?.type !== "line" || few?.type !== "line") throw new Error("lines");
    expect(every.alpha).toBeCloseTo(0.95);
    expect(every.weight).toBeCloseTo(3);
    expect(few.alpha).toBeCloseTo(0.25 + 0.07);
    expect(every.colour).toBe(kindColour(1));
    // A home its people's record has no draw for is hollow, never painted as a source.
    const hollow = marks.filter((m) => m.type === "home" && m.hollow);
    expect(hollow).toHaveLength(2);
    expect(lensMarks(info, "")).toEqual([]);
  });

  it("finds the home under the pointer and names its legend's scope, period and denominator", () => {
    expect(homeAt(info, 101, 2, 5)?.household).toBe(100);
    expect(homeAt(info, 150, 0, 5)).toBeNull();
    expect(homeAt(null, 0, 0, 5)).toBeNull();
    const reach = legend(info, "reach");
    expect(reach.title).toBe("Water: could reach");
    expect(reach.lines).toEqual([
      "Scope: Ashford: 3 homes",
      "Period: now, on the paths as last surveyed",
      "Per: homes: households with people, each at its home",
    ]);
    expect(reach.classes.at(-1)).toEqual({ label: "none within 60 minutes' walk", colour: NONE_COLOUR, hollow: true });
    expect(reach.counts).toBe("Could reach: of 3 homes, …");
    const came = legend(info, "came");
    expect(came.title).toBe("Water: did come");
    expect(came.lines[1]).toBe("Period: the past 30 days, today included");
    expect(came.classes.map((c) => c.label)).toContain("another's well");
    expect(came.counts).toBe("Did come: in the past 30 days, of 3 homes, …");
  });
});

function place(x: number, passages: number, waitedMin: number): CoveragePlace {
  return {
    name: `gate ${x}`,
    x,
    y: 0,
    settlement: 1,
    passages,
    waitedMin,
    heldShort: 0,
    longestMin: waitedMin > 0 ? 1 : 0,
    hourPassages: new Array<number>(24).fill(0),
    hourWaitS: new Array<number>(24).fill(0),
    words: [`${passages} passages`],
  };
}

describe("the Traffic lens", () => {
  const traffic: CoverageInfo = {
    ...info,
    lens: 3,
    title: "Traffic",
    scope: "Ashford: 3 crossings and gates that passed anyone lately",
    periods: ["about the last 30 days, each passage fading", "by the hour of the day it was reached"],
    denominator: "passages: each walker through a crossing or gate once",
    homes: [],
    summary: ["3 crossings and gates passed 700 walkers …", "Nobody waited a minute at any."],
    places: [place(0, 600, 3), place(100, 90, 0), place(200, 10, 20)],
  };

  it("is lens 3 and classes each place by its wait a passage, hollow where nobody waited", () => {
    expect(lensId("traffic")).toBe(3);
    expect(trafficClass(traffic.places[0]!)).toBe(0);
    expect(trafficClass(traffic.places[2]!)).toBe(3);
    const marks = lensMarks(traffic, "traffic");
    expect(marks).toHaveLength(3);
    expect(marks[1]).toMatchObject({ type: "home", x: 100, hollow: true });
    expect(marks[2]).toMatchObject({ colour: TRAFFIC_CLASSES[3]!.colour, hollow: false });
    expect(lensMarks(traffic, "reach")).toHaveLength(0);
  });

  it("finds the place under the pointer and names its scope, period and counts", () => {
    expect(placeAt(traffic, 98, 0, 5)?.name).toBe("gate 100");
    expect(placeAt(traffic, 50, 0, 5)).toBeNull();
    const l = legend(traffic, "traffic");
    expect(l.title).toBe("Traffic");
    expect(l.lines[0]).toContain("crossings and gates");
    expect(l.classes).toHaveLength(TRAFFIC_CLASSES.length + 1);
    expect(l.counts).toContain("Nobody waited a minute");
  });
});
