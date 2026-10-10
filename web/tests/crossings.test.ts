import { describe, expect, it } from "vitest";

import { crossingAt, crossingEnds, crossingLook, ROTTEN_COLOUR, TIMBER_COLOUR, WRECK_COLOUR } from "../src/crossings.js";
import type { CrossingInfo } from "../src/net/messages.js";

function crossing(over: Partial<CrossingInfo> = {}): CrossingInfo {
  return {
    id: 1,
    system: "Log footbridge",
    ax: 100,
    ay: 200,
    bx: 116,
    by: 200,
    spanM: 2.4,
    members: 2,
    diameterCm: 20,
    state: 1,
    labourH: 72,
    workH: 72,
    quality: 0.9,
    loss: 0,
    margin: 19,
    ownerKind: 0,
    owner: 7,
    words: "Ada's household's log footbridge, open since year 2: two members 20 cm thick over 2.4 m, sound",
    lengthM: 4.4,
    ...over,
  };
}

describe("crossings", () => {
  it("lays the members their length, centred between the banks and along the line between them", () => {
    const [x0, y0, x1, y1] = crossingEnds(crossing());
    expect(x0).toBeCloseTo(105.8);
    expect(x1).toBeCloseTo(110.2);
    expect(y0).toBe(200);
    expect(y1).toBe(200);
    const [dx0, dy0, dx1, dy1] = crossingEnds(crossing({ bx: 108, by: 208, lengthM: Math.SQRT2 * 4 }));
    expect(dx0).toBeCloseTo(102);
    expect(dy0).toBeCloseTo(202);
    expect(dx1).toBeCloseTo(106);
    expect(dy1).toBeCloseTo(206);
  });

  it("finds the crossing under a point, the nearer when two are close, none off the members' ends", () => {
    const a = crossing({ id: 1 });
    const b = crossing({ id: 2, ay: 203, by: 203 });
    expect(crossingAt([a, b], 108, 200.1, 0.5)?.id).toBe(1);
    expect(crossingAt([a, b], 108, 202.8, 0.5)?.id).toBe(2);
    expect(crossingAt([a, b], 108, 201.5, 0.5)).toBeNull();
    expect(crossingAt([a, b], 101, 200, 0.5)).toBeNull();
  });

  it("draws one being built fainter the less is done, an open one greying as rot takes it, a wreck dark", () => {
    const begun = crossingLook(crossing({ state: 0, workH: 0, quality: 0, margin: 0 }));
    const half = crossingLook(crossing({ state: 0, workH: 36, quality: 0, margin: 0 }));
    const open = crossingLook(crossing());
    expect(begun.alpha).toBeLessThan(half.alpha);
    expect(half.alpha).toBeLessThan(open.alpha);
    expect(open.colour).toBe(TIMBER_COLOUR);
    expect(crossingLook(crossing({ loss: 1 })).colour).toBe(ROTTEN_COLOUR);
    const rotting = crossingLook(crossing({ loss: 0.5 })).colour;
    expect(rotting).not.toBe(TIMBER_COLOUR);
    expect(rotting).not.toBe(ROTTEN_COLOUR);
    expect(crossingLook(crossing({ state: 2 })).colour).toBe(WRECK_COLOUR);
    expect(open.widthM).toBeCloseTo(0.4);
  });
});
