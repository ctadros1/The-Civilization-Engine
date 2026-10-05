import { describe, expect, it } from "vitest";

import { depositAt, depositLook, depositWords, materialGoods } from "../src/deposits.js";
import type { DepositInfo, GoodInfo } from "../src/net/messages.js";

const good = (id: string, name: string, purpose: string): GoodInfo =>
  ({ id, name, purpose, kcalPerKg: 0, eaten: "never", toolLifeH: 0 }) as GoodInfo;

const GOODS = [
  good("core:good/clay", "Clay", "material"),
  good("core:good/meat", "Meat", "food"),
  good("core:good/stone", "Stone", "material"),
];

function deposit(over: Partial<DepositInfo> = {}): DepositInfo {
  return {
    id: 1,
    good: 0,
    x: 100,
    y: 200,
    radiusM: 8,
    exposed: true,
    coverM: 0,
    thicknessM: 1.2,
    quality: 0.6,
    leftKg: 975_000,
    takenKg: 0,
    knownBy: [],
    finds: [],
    ...over,
  };
}

describe("deposits", () => {
  it("finds the deposit under a point, the nearest when two overlap", () => {
    const a = deposit({ id: 1, x: 100, y: 200, radiusM: 8 });
    const b = deposit({ id: 2, x: 106, y: 200, radiusM: 8 });
    expect(depositAt([a, b], 101, 200)?.id).toBe(1);
    expect(depositAt([a, b], 105, 200)?.id).toBe(2);
    expect(depositAt([a, b], 100, 220)).toBeNull();
  });

  it("says what a deposit is, where it lies, what is left and who found it", () => {
    expect(depositWords(deposit(), GOODS)).toBe("clay, 16 m across, showing at the surface; 975 t left; not yet found");
    const buried = deposit({
      good: 2,
      exposed: false,
      coverM: 1.25,
      leftKg: 4_200,
      finds: ["found by Ash of Alderford in year 3"],
      knownBy: [12],
    });
    expect(depositWords(buried, GOODS)).toBe(
      "stone, 16 m across, under 1.3 m of ground; 4.2 t left; found by Ash of Alderford in year 3",
    );
  });

  it("draws found deposits solid and unfound ones faint and outlined", () => {
    const found = depositLook(deposit({ knownBy: [12] }), GOODS);
    const unfound = depositLook(deposit(), GOODS);
    expect(found.outline).toBe(false);
    expect(unfound.outline).toBe(true);
    expect(found.alpha).toBeGreaterThan(unfound.alpha);
    expect(found.colour).toBe(unfound.colour);
  });

  it("offers the materials, by name, for the observer's tool", () => {
    expect(materialGoods(GOODS).map((g) => g.name)).toEqual(["Clay", "Stone"]);
  });
});
