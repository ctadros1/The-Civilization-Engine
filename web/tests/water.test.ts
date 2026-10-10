import { describe, expect, it } from "vitest";

import type { SpringInfo, WellInfo } from "../src/net/messages.js";
import {
  DIGGING_COLOUR,
  DRY_COLOUR,
  FALLEN_COLOUR,
  WELL_WATER_COLOUR,
  bankAt,
  bankWords,
  flowWords,
  springAt,
  springWords,
  wellAt,
  wellLook,
} from "../src/water.js";

function well(over: Partial<WellInfo> = {}): WellInfo {
  return {
    id: 1,
    system: "Timber-lined well",
    x: 100,
    y: 200,
    radiusM: 0.75,
    state: 1,
    depthM: 4,
    targetM: 4,
    waterM: 1,
    belowM: 3,
    quality: 0.6,
    loss: 0,
    household: 7,
    words: "Ada's household's timber-lined well, open since year 2: 4.0 m deep",
    ...over,
  };
}

function spring(over: Partial<SpringInfo> = {}): SpringInfo {
  return { x: 300, y: 300, flowM3Day: 2.5, drawnL: 0, ...over };
}

describe("water", () => {
  it("draws a shaft being dug fainter the shallower it is, an open one blue, and the rest dark", () => {
    const begun = wellLook(well({ state: 0, depthM: 0, waterM: 0 }));
    const half = wellLook(well({ state: 0, depthM: 2, waterM: 0 }));
    expect(begun.colour).toBe(DIGGING_COLOUR);
    expect(half.alpha).toBeGreaterThan(begun.alpha);
    const open = wellLook(well());
    expect(open.colour).toBe(WELL_WATER_COLOUR);
    expect(open.alpha).toBeGreaterThan(wellLook(well({ waterM: 0 })).alpha);
    expect(open.radiusM).toBe(0.75);
    expect(wellLook(well({ state: 2 })).colour).toBe(DRY_COLOUR);
    expect(wellLook(well({ state: 3 })).colour).toBe(FALLEN_COLOUR);
  });

  it("finds the well, spring or place drawn at under a point, the nearer when two are close", () => {
    const a = well({ id: 1 });
    const b = well({ id: 2, x: 102 });
    expect(wellAt([a, b], 100.2, 200, 0.1)?.id).toBe(1);
    expect(wellAt([a, b], 101.8, 200, 0.1)?.id).toBe(2);
    expect(wellAt([a, b], 100, 201, 0.1)).toBeNull();
    expect(springAt([spring()], 302, 300, 3)).not.toBeNull();
    expect(springAt([spring()], 305, 300, 3)).toBeNull();
    expect(bankAt([{ x: 10, y: 10, trips: 3 }], 11, 10, 2)?.trips).toBe(3);
    expect(bankAt([], 11, 10, 2)).toBeNull();
  });

  it("tells a spring, a place drawn at and the rivers' flow in words", () => {
    expect(springWords(spring())).toBe("a spring: 2.5 m³ flows to it a day, none drawn today");
    expect(springWords(spring({ flowM3Day: 12.4, drawnL: 450.2 }))).toBe(
      "a spring: 12 m³ flows to it a day, 450 L drawn today",
    );
    expect(bankWords({ x: 0, y: 0, trips: 1 })).toBe("people came for water here once today");
    expect(bankWords({ x: 0, y: 0, trips: 6 })).toBe("people came for water here 6 times today");
    expect(flowWords(1.02)).toBe("flowing at its mean today");
    expect(flowWords(0.4)).toBe("flowing at 40% of its mean today");
    expect(flowWords(2.24)).toBe("flowing at 2.2 times its mean today");
  });
});
