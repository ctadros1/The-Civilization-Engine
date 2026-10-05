import { describe, expect, it } from "vitest";

import { FIELD_LEGEND, fieldAt, fieldKey, fieldLook, tenureText } from "../src/fields.js";
import type { FieldInfo } from "../src/net/messages.js";

function field(over: Partial<FieldInfo> = {}): FieldInfo {
  return {
    id: 1,
    household: 2,
    settlement: 3,
    x: 100,
    y: 200,
    w: 50,
    h: 50,
    crop: 0,
    stage: "fallow",
    stageSinceMinute: 0,
    progress: 0,
    newGround: false,
    woodland: false,
    ripe: false,
    expectedKg: 0,
    sheavesKg: 0,
    harvests: 1,
    status: "fallow",
    holder: 2,
    holderSettlement: 0,
    leaseUntilMinute: -1,
    leaseShare: 0,
    waterHad: -1,
    soilWater: 0,
    ...over,
  };
}

describe("fields on the map", () => {
  it("are drawn by where they are in their year", () => {
    expect(fieldKey(field({ newGround: true, woodland: true }))).toBe("woodland");
    expect(fieldKey(field({ newGround: true }))).toBe("new");
    expect(fieldKey(field())).toBe("fallow");
    expect(fieldKey(field({ stage: "prepared" }))).toBe("prepared");
    expect(fieldKey(field({ stage: "sown" }))).toBe("sown");
    expect(fieldKey(field({ stage: "sown", ripe: true }))).toBe("ripe");
    expect(fieldKey(field({ stage: "reaped" }))).toBe("reaped");
    // Every key has a legend entry.
    const keys = new Set(FIELD_LEGEND.map((e) => e.key));
    for (const k of ["woodland", "new", "fallow", "prepared", "sown", "ripe", "reaped"]) {
      expect(keys.has(k)).toBe(true);
    }
  });

  it("show ground being broken fainter until the work is done", () => {
    const start = fieldLook(field({ newGround: true, progress: 0 }));
    const half = fieldLook(field({ newGround: true, progress: 0.5 }));
    expect(half.alpha).toBeGreaterThan(start.alpha);
    expect(fieldLook(field({ stage: "sown" })).alpha).toBe(0.85);
  });

  it("are found under a point", () => {
    const fields = [field({ id: 1 }), field({ id: 2, x: 160 })];
    expect(fieldAt(fields, 120, 210)?.id).toBe(1);
    expect(fieldAt(fields, 170, 210)?.id).toBe(2);
    expect(fieldAt(fields, 155, 210)).toBeNull();
    expect(fieldAt(fields, 150, 210)).toBeNull();
  });
});

describe("who holds a field", () => {
  const living = new Set([2, 5]);
  it("says whether its holder works it, lets it or the settlement gives it out", () => {
    expect(tenureText(field(), living)).toBe("held by the household that works it");
    expect(tenureText(field({ holder: 0, holderSettlement: 3 }), living)).toBe(
      "the settlement's, given out to work",
    );
    expect(tenureText(field({ holder: 0, holderSettlement: 3, household: 9 }), living)).toBe(
      "the settlement's, not given out",
    );
    // Let by household 5 to household 2 for a quarter of its grain until 1 March, year 3.
    const until = (2 * 365 + 59) * 1440;
    expect(
      tenureText(field({ holder: 5, leaseUntilMinute: until, leaseShare: 0.25 }), living),
    ).toBe("let for 25% of its grain until Year 3 · Mar 1 · 00:00");
  });

  it("says when nobody works it", () => {
    expect(tenureText(field({ household: 9, holder: 5 }), living)).toBe("its tenant is gone");
    expect(tenureText(field({ household: 9, holder: 9 }), living)).toBe("nobody holds it now");
  });
});
