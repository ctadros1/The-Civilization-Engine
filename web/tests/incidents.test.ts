import { describe, expect, it } from "vitest";

import { curveBars, kindClass, stepText, stepWhen, totalsLine } from "../src/incidents.js";

const DAY = 1440;

describe("the Incidents panel's words", () => {
  it("counts each kind before anything else, and the observer's share", () => {
    expect(totalsLine({ kind: 1, all: 0, known: 0, polity: 0, observer: 0 })).toBe("Takings: none");
    expect(totalsLine({ kind: 1, all: 12, known: 5, polity: 2, observer: 0 })).toBe(
      "Takings: 12 in all; 5 known to someone; 2 before a gathering",
    );
    expect(totalsLine({ kind: 2, all: 1, known: 0, polity: 0, observer: 0 })).toBe(
      "Illness: 1 case; 0 known to someone; none named by a law",
    );
    expect(totalsLine({ kind: 2, all: 9, known: 3, polity: 2, observer: 9 })).toBe(
      "Illness: 9 cases; 3 known to someone; 2 named by a law; 9 the observer's",
    );
    expect(kindClass(1)).toBe("taking");
    expect(kindClass(2)).toBe("illness");
  });

  it("tells a step as precisely as it is known, and never invents an hour", () => {
    const day = { label: "Fell ill", minute: 400 * DAY, state: "reached" as const, byDay: true, words: "for 4 days" };
    expect(stepWhen(day)).not.toContain(":");
    expect(stepText(day)).toMatch(/^Fell ill: \d+ \w+ of year \d+ — for 4 days$/);
    const minute = { ...day, label: "Went to take", minute: 400 * DAY + 14 * 60 + 5, byDay: false, words: "" };
    expect(stepWhen(minute)).toMatch(/, 14:05$/);
    expect(stepText(minute)).not.toContain("—");
    const never = { ...day, label: "Seen", minute: -1, state: "not reached" as const, words: "nobody saw them abed" };
    expect(stepText(never)).toBe("Seen: not reached — nobody saw them abed");
    expect(stepWhen({ ...never, state: "not recorded" })).toBe("not recorded");
  });

  it("draws an outbreak's days to the tallest, a day with none keeping its place", () => {
    expect(curveBars({ took: [2, 0, 4], onset: [0, 1, 2] })).toEqual([
      { took: 0.5, onset: 0 },
      { took: 0, onset: 0.25 },
      { took: 1, onset: 0.5 },
    ]);
    expect(curveBars({ took: [], onset: [] })).toEqual([]);
  });
});
