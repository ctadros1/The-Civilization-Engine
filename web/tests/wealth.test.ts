import { describe, expect, it } from "vitest";

import type { HouseholdWealth, WealthSpread } from "../src/net/messages.js";
import {
  areaText,
  floorText,
  giniLines,
  giniText,
  householdRows,
  hoursText,
  measureRows,
  spreadText,
  worksText,
  yearRows,
} from "../src/wealth.js";

function spread(over: Partial<WealthSpread> = {}): WealthSpread {
  return {
    year: 2,
    households: 6,
    people: 24,
    giniGoods: 0.21,
    giniHeld: 0.3,
    giniWorked: 0.12,
    giniFloor: 0.05,
    topTenthGoods: 0.18,
    holdingNone: 1 / 3,
    workingNone: 0,
    goodsHPerHead: 58.4,
    workedHaPerHead: 0.15,
    floorM2PerHouse: 13.2,
    commonHa: 0,
    roofedM2PerHouse: 13.2,
    storageKgPerHouse: 1320,
    ...over,
  };
}

function household(over: Partial<HouseholdWealth> = {}): HouseholdWealth {
  return {
    household: 7,
    name: "Wren's household",
    members: 4,
    heldHa: 1,
    workedHa: 0.6,
    letHa: 0.4,
    rentedHa: 0,
    goodsH: 240,
    floorM2: 22.6,
    roofedM2: 22.6,
    storageKg: 2260,
    ...over,
  };
}

describe("wealth in words", () => {
  it("formats Ginis, areas, hours and floor areas", () => {
    expect(giniText(0.4567)).toBe("0.46");
    expect(areaText(0)).toBe("none");
    expect(areaText(1.234)).toBe("1.23 ha");
    expect(hoursText(0.01)).toBe("none");
    expect(hoursText(4.25)).toBe("4.3 h");
    expect(hoursText(58.4)).toBe("58 h");
    expect(floorText(0)).toBe("no roof");
    expect(floorText(22.6)).toBe("23 m²");
  });

  it("lists a settlement's measures with how unequally each is spread", () => {
    const rows = measureRows(spread(), [household(), household({ household: 8, heldHa: 0.2 })]);
    expect(rows.map((r) => r.measure)).toEqual([
      "Goods",
      "Land worked",
      "Land held",
      "Floor area",
      "Under all roofs",
      "Room for goods",
    ]);
    expect(rows[0]).toEqual({ measure: "Goods", level: "58 h a head", gini: "0.21" });
    expect(rows[1]!.level).toBe("0.15 ha a head");
    expect(rows[2]!.level).toBe("0.05 ha a head"); // 1.2 ha over 24 people
    expect(rows[3]!.level).toBe("13 m² a house");
    // Beside the houses: stores' and workshops' floors, and room for goods (no Gini of either).
    expect(rows[4]).toEqual({ measure: "Under all roofs", level: "13 m² a household", gini: "" });
    expect(rows[5]!.level).toBe("1.3 t a household");
    const village = measureRows(spread({ floorM2PerHouse: 0, roofedM2PerHouse: 0, storageKgPerHouse: 0 }), [
      household({ heldHa: 0 }),
    ]);
    expect(village[2]!.level).toBe("none");
    expect(village[3]!.level).toBe("no roofs yet");
    expect(village[4]!.level).toBe("no roofs yet");
    expect(village[5]!.level).toBe("none yet");
  });

  it("says who holds land and who works it", () => {
    expect(spreadText(spread())).toBe(
      "The richest tenth of people have 18% of the goods. 2 of 6 households hold no land; every household works some.",
    );
    expect(spreadText(spread({ holdingNone: 1, commonHa: 4.2, workingNone: 1 / 6 }))).toBe(
      "The richest tenth of people have 18% of the goods. No household holds land: the settlement holds 4.20 ha; 1 works none.",
    );
    expect(spreadText(spread({ holdingNone: 0 }))).toContain("Every household holds land;");
    expect(spreadText(spread({ households: 0, people: 0 }))).toBe("Nobody lives here now.");
  });

  it("lists households with their leases, in the order given", () => {
    expect(worksText(household())).toBe("0.60 ha (lets 0.40 ha)");
    expect(worksText(household({ rentedHa: 0.25, letHa: 0 }))).toBe("0.60 ha (rents 0.25 ha)");
    expect(worksText(household({ letHa: 0 }))).toBe("0.60 ha");
    const rows = householdRows(
      [household(), household({ household: 8, name: "Bran's household", members: 0, goodsH: 0 })],
      1,
    );
    expect(rows).toEqual([
      {
        household: 7,
        name: "Wren's household",
        people: "4",
        holds: "1.00 ha",
        works: "0.60 ha (lets 0.40 ha)",
        goods: "60 h",
        floor: "23 m²",
      },
    ]);
    // A granary beside the house counts in all the floor it has under roofs.
    expect(householdRows([household({ roofedM2: 37.6 })])[0]!.floor).toBe("23 m², 38 m² in all");
  });

  it("lists the latest years first and draws each Gini on a fixed scale", () => {
    const history = [
      spread({ year: 1, giniGoods: 0 }),
      spread({ year: 2, giniGoods: 0.5 }),
      spread({ year: 3, giniGoods: 1 }),
    ];
    expect(yearRows(history, 2).map((r) => [r.year, r.goods])).toEqual([
      [3, "1.00"],
      [2, "0.50"],
    ]);
    const lines = giniLines(history, 100, 50);
    expect(lines.goods).toBe("0.0,50.0 50.0,25.0 100.0,0.0");
    expect(lines.floor.split(" ")).toHaveLength(3);
    expect(giniLines(history.slice(0, 1), 100, 50)).toEqual({ goods: "", worked: "", floor: "" });
  });
});
