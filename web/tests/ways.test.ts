import { describe, expect, it } from "vitest";

import type { WayInfo, WaySegmentInfo } from "../src/net/messages.js";
import { EARTH_COLOUR, GRAVEL_COLOUR, wayAt, wayLook, wayWords } from "../src/ways.js";

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
});
