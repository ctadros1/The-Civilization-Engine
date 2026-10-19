import * as flatbuffers from "flatbuffers";
import { describe, expect, it } from "vitest";

import { NOTHING_COLOUR, PAID_COLOUR, RECKONED_COLOUR, landAt, landKind, landLook, landWords } from "../src/land.js";
import * as M from "../src/net/messages.js";
import * as W from "../src/schema/generated/tce/wire.js";

function plot(over: Partial<M.LandPlot>): M.LandPlot {
  return { rect: { x: 100, y: 100, w: 10, h: 20 }, paidKcal: 0, reckonedH: 0, weighed: 0, ...over };
}

describe("land", () => {
  it("labels a plot by what is recorded of it, paid before reckoned", () => {
    expect(landKind(plot({}))).toBe("nothing");
    expect(landKind(plot({ reckonedH: 4, weighed: 12 }))).toBe("reckoned");
    expect(landKind(plot({ paidKcal: 3000, reckonedH: 4, weighed: 12 }))).toBe("paid");
  });

  it("draws paid in brown, reckoned greener as it saved more, nothing only outlined", () => {
    expect(landLook(plot({ paidKcal: 1 }), 10)).toEqual({ colour: PAID_COLOUR, fill: 0.55 });
    expect(landLook(plot({}), 10)).toEqual({ colour: NOTHING_COLOUR, fill: 0 });
    const less = landLook(plot({ reckonedH: 2, weighed: 3 }), 10);
    const most = landLook(plot({ reckonedH: 10, weighed: 3 }), 10);
    expect(less.colour).toBe(RECKONED_COLOUR);
    expect(most.fill).toBeGreaterThan(less.fill);
    expect(most.fill).toBeCloseTo(0.7);
  });

  it("finds the plot under the point, the last laid first", () => {
    const a = plot({});
    const b = plot({ rect: { x: 105, y: 105, w: 10, h: 10 }, paidKcal: 5 });
    expect(landAt([a, b], 106, 106)).toBe(b);
    expect(landAt([a, b], 101, 101)).toBe(a);
    expect(landAt([a, b], 99, 101)).toBeNull();
  });

  it("says what is recorded, and never more", () => {
    expect(landWords(plot({}))).toBe("land: nothing recorded");
    expect(landWords(plot({ paidKcal: 3000 }))).toBe("land: paid 3,000 kcal for it");
    expect(landWords(plot({ paidKcal: 3000, reckonedH: 4.25, weighed: 12 }))).toBe(
      "land: paid 3,000 kcal for it; reckoned worth 4.3 hours a year over the next best of 12 weighed",
    );
  });

  it("decodes the land map from the wire (1.89)", () => {
    const b = new flatbuffers.Builder(256);
    const plots = [
      { rect: { x: 1, y: 2, w: 3, h: 4 }, paidKcal: 1500, reckonedH: 0, weighed: 0 },
      { rect: { x: 5, y: 6, w: 7, h: 8 }, paidKcal: 0, reckonedH: 2.5, weighed: 9 },
    ];
    const offsets = plots.map((p) => {
      W.LandPlot.startLandPlot(b);
      W.LandPlot.addRect(b, W.PlanRect.createPlanRect(b, p.rect.x, p.rect.y, p.rect.w, p.rect.h));
      W.LandPlot.addPaidKcal(b, p.paidKcal);
      W.LandPlot.addReckonedH(b, p.reckonedH);
      W.LandPlot.addWeighed(b, p.weighed);
      return W.LandPlot.endLandPlot(b);
    });
    const list = W.Land.createPlotsVector(b, offsets);
    b.finish(W.Response.createResponse(b, W.ResponseBody.Land, W.Land.createLand(b, 7n, list)));
    const body = M.decodeResponse(b.asUint8Array());
    expect(body.kind).toBe("land");
    if (body.kind !== "land") return;
    expect(body.land).toEqual({ rev: 7, plots });
  });
});
