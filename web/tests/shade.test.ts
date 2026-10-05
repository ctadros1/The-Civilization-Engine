import { describe, it, expect } from "vitest";
import { seasonKey, seasonOf, shade } from "../src/map/shade.js";

describe("season resolution", () => {
  it("maps valid seasons to themselves", () => {
    expect(seasonOf("spring")).toBe("spring");
    expect(seasonOf("summer")).toBe("summer");
    expect(seasonOf("autumn")).toBe("autumn");
    expect(seasonOf("winter")).toBe("winter");
  });

  it("defaults unknown or missing seasons to summer", () => {
    expect(seasonOf("unknown")).toBe("summer");
    expect(seasonOf(undefined)).toBe("summer");
    expect(seasonOf(null)).toBe("summer");
  });

  it("forms a caching key from the season and snow line", () => {
    expect(seasonKey("winter", 1250.4)).toBe("winter:1250");
    expect(seasonKey("winter", 1250.6)).toBe("winter:1251");
    expect(seasonKey("spring", undefined)).toBe("spring:-1");
    expect(seasonKey("autumn", -50)).toBe("autumn:-1");
  });
});

describe("shading", () => {
  const w = 4;
  const h = 4;
  const elev = new Float32Array(w * h);
  const water = new Uint8Array(w * h);

  it("renders snow above the snow line and blends it", () => {
    elev.fill(100);
    // Make a flat snowy plateau so it's fully lit (k=1)
    elev[0] = 1000;
    elev[1] = 1025;
    elev[2] = 1050;
    elev[3] = 1100;
    for (let i = 4; i < 16; i++) elev[i] = elev[i % 4]!;

    const rgba = shade(elev, water, w, h, {
      cellM: 8,
      seaLevelM: 0,
      minM: 0,
      maxM: 2000,
      season: "winter",
      snowLineM: 1000,
    });

    // Check pixel 0 (at 1000, blend = 0) vs pixel 2 (at 1050, blend = 1)
    const row = 1; // row 1 to avoid edge effects
    const o0 = (row * w + 0) * 4;
    const o1 = (row * w + 1) * 4;
    const o2 = (row * w + 2) * 4;

    const r0 = rgba[o0]!;
    const r1 = rgba[o1]!;
    const r2 = rgba[o2]!;
    const b2 = rgba[o2 + 2]!;

    expect(r0).toBeLessThan(200); // Base color
    expect(r2).toBeGreaterThan(200); // Full snow, but shaded by slope
    expect(b2).toBeGreaterThan(200);
    expect(r1).toBeGreaterThan(r0); // Blended
    expect(r1).toBeLessThan(r2);
  });
});
