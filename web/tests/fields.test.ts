import { describe, expect, it } from "vitest";

import { FIELD_LEGEND, fieldAt, fieldKey, fieldLook } from "../src/fields.js";
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
