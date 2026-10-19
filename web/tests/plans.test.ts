import * as flatbuffers from "flatbuffers";
import { describe, expect, it } from "vitest";

import * as M from "../src/net/messages.js";
import { planAt, planLook, planWords } from "../src/plans.js";
import * as W from "../src/schema/generated/tce/wire.js";

function plan(over: Partial<M.PlanInfo>): M.PlanInfo {
  return {
    law: 9,
    version: 1,
    inForce: true,
    streets: [{ x: 100, y: 100, w: 80, h: 6 }],
    sites: [{ x: 100, y: 106, w: 80, h: 40 }],
    parcels: [{ x: 180, y: 106, w: 10, h: 20 }],
    built: [],
    occupied: [],
    words: "Ashford: a plan (version 1) of 1 by 1 blocks, in force",
    ...over,
  };
}

describe("plans", () => {
  it("draws one put forward at half the strength of one in force", () => {
    const shown = planLook(plan({}));
    const forward = planLook(plan({ inForce: false }));
    expect(forward.reservedAlpha).toBeCloseTo(shown.reservedAlpha / 2);
    expect(forward.parcelAlpha).toBeCloseTo(shown.parcelAlpha / 2);
    expect(forward.lineAlpha).toBeCloseTo(shown.lineAlpha / 2);
  });

  it("finds the topmost layer under the pointer, plans in force first", () => {
    const p = plan({
      built: [{ x: 120, y: 100, w: 10, h: 6 }],
      occupied: [{ x: 150, y: 110, w: 6, h: 5 }],
    });
    expect(planAt([p], 105, 102)).toEqual({ plan: p, layer: "street" });
    expect(planAt([p], 125, 102)).toEqual({ plan: p, layer: "built" });
    expect(planAt([p], 110, 120)).toEqual({ plan: p, layer: "square" });
    expect(planAt([p], 152, 112)).toEqual({ plan: p, layer: "occupied" });
    expect(planAt([p], 185, 110)).toEqual({ plan: p, layer: "parcel" });
    expect(planAt([p], 50, 50)).toBeNull();
    // Over the same ground, the one in force is found before one put forward.
    const forward = plan({ law: 10, inForce: false });
    expect(planAt([forward, p], 105, 102)?.plan.law).toBe(9);
    expect(planAt([forward], 105, 102)?.plan.law).toBe(10);
  });

  it("says which layer the pointer is on, in the kernel's words", () => {
    const p = plan({});
    expect(planWords({ plan: p, layer: "street" })).toBe(
      "a street the plan reserves: Ashford: a plan (version 1) of 1 by 1 blocks, in force",
    );
    expect(planWords({ plan: p, layer: "occupied" })).toMatch(/^a building on ground the plan reserves: /);
  });

  it("decodes the plans from the wire (1.88)", () => {
    const b = new flatbuffers.Builder(256);
    const rects = (start: (n: number) => void, list: M.PlanRect[]): flatbuffers.Offset => {
      start(list.length);
      for (const r of [...list].reverse()) W.PlanRect.createPlanRect(b, r.x, r.y, r.w, r.h);
      return b.endVector();
    };
    const words = b.createString("Ashford: a plan (version 2), put forward");
    const streets = rects(
      (n) => W.PlanInfo.startStreetsVector(b, n),
      [
        { x: 1, y: 2, w: 3, h: 4 },
        { x: 5, y: 6, w: 7, h: 8 },
      ],
    );
    const sites = rects((n) => W.PlanInfo.startSitesVector(b, n), [{ x: 10, y: 10, w: 80, h: 40 }]);
    const parcels = rects((n) => W.PlanInfo.startParcelsVector(b, n), [{ x: 90, y: 10, w: 10, h: 20 }]);
    const built = rects((n) => W.PlanInfo.startBuiltVector(b, n), [{ x: 1, y: 2, w: 3, h: 4 }]);
    const occupied = rects((n) => W.PlanInfo.startOccupiedVector(b, n), []);
    const info = W.PlanInfo.createPlanInfo(b, 77n, 2, false, streets, sites, parcels, built, occupied, words);
    const list = W.Plans.createPlansVector(b, [info]);
    b.finish(W.Response.createResponse(b, W.ResponseBody.Plans, W.Plans.createPlans(b, 5n, list)));
    const body = M.decodeResponse(b.asUint8Array());
    expect(body.kind).toBe("plans");
    if (body.kind !== "plans") return;
    expect(body.plans).toEqual({
      rev: 5,
      plans: [
        {
          law: 77,
          version: 2,
          inForce: false,
          streets: [
            { x: 1, y: 2, w: 3, h: 4 },
            { x: 5, y: 6, w: 7, h: 8 },
          ],
          sites: [{ x: 10, y: 10, w: 80, h: 40 }],
          parcels: [{ x: 90, y: 10, w: 10, h: 20 }],
          built: [{ x: 1, y: 2, w: 3, h: 4 }],
          occupied: [],
          words: "Ashford: a plan (version 2), put forward",
        },
      ],
    });
  });
});
