import { describe, expect, it } from "vitest";
import { neighbourSizes } from "../src/settlements.js";

const limits = { bandSizeMin: 30, bandSizeMax: 125 };

describe("neighbouring groups", () => {
  it("are none when left blank", () => {
    expect(neighbourSizes("", limits)).toEqual([]);
    expect(neighbourSizes("   ", limits)).toEqual([]);
  });

  it("are sizes separated by commas or spaces", () => {
    expect(neighbourSizes("40, 30", limits)).toEqual([40, 30]);
    expect(neighbourSizes("50 60", limits)).toEqual([50, 60]);
  });

  it("are at most two, whole and within the band limits", () => {
    expect(neighbourSizes("40, 40, 40", limits)).toMatch(/At most 2/);
    expect(neighbourSizes("4.5", limits)).toMatch(/whole number/);
    expect(neighbourSizes("20", limits)).toMatch(/30 to 125/);
    expect(neighbourSizes("20", null)).toEqual([20]);
  });
});
