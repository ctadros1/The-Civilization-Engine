import { describe, expect, it } from "vitest";

import {
  amountText,
  askingText,
  monthIndex,
  monthText,
  offerGroups,
  paymentsText,
  priceSeries,
  priceText,
  sparkline,
  worthText,
} from "../src/market.js";
import type { GoodInfo, MarketInfo, OfferInfo } from "../src/net/messages.js";

function good(name: string, purpose: string): GoodInfo {
  return {
    id: `core:good/${name.toLowerCase()}`,
    name,
    purpose,
    kcalPerKg: 0,
    eaten: "never",
    toolLifeH: purpose === "tool" ? 300 : 0,
  } as GoodInfo;
}

/** Grain (0), flour (1), a sickle (2), a hoe (3). */
const GOODS = [good("Grain", "food"), good("Flour", "food"), good("Sickle", "tool"), good("Hoe", "tool")];

function offer(household: number, good: number, payment: number, price: number, units = 2): OfferInfo {
  return { household, householdName: `H${household}'s household`, good, payment, price, units, firm: false };
}

describe("market words", () => {
  it("says amounts as people would", () => {
    expect(amountText(GOODS[2], 1)).toBe("1 sickle");
    expect(amountText(GOODS[3], 0.8)).toBe("0.8 hoes");
    expect(amountText(GOODS[0], 12.4)).toBe("12 kg grain");
    expect(amountText(undefined, 2)).toContain("no longer known");
  });

  it("quotes tools each and goods by weight per ten kilograms", () => {
    expect(priceText(GOODS, 2, 0, 12)).toBe("12 kg grain each");
    // 0.03 hoes a kilogram of grain is 0.3 hoes for ten.
    expect(priceText(GOODS, 0, 3, 0.03)).toBe("0.3 hoes per 10 kg");
  });

  it("asks the lowest terms in each payment", () => {
    const offers = [offer(1, 2, 0, 14), offer(2, 2, 0, 12), offer(2, 2, 3, 0.8), offer(1, 0, 3, 0.03)];
    expect(askingText(GOODS, offers, 2)).toBe("12 kg grain each or 0.8 hoes each");
    expect(askingText(GOODS, offers, 1)).toBe("");
  });

  it("groups each household's terms for a good", () => {
    const offers = [offer(2, 2, 3, 0.8, 1.5), offer(1, 2, 0, 14), offer(2, 2, 0, 12, 1.5)];
    expect(offerGroups(GOODS, offers)).toEqual([
      { household: 1, householdName: "H1's household", firm: false, good: 2, units: 2, terms: "14 kg grain each" },
      {
        household: 2,
        householdName: "H2's household",
        firm: false,
        good: 2,
        units: 1.5,
        terms: "12 kg grain each or 0.8 hoes each",
      },
    ]);
  });

  it("lists what payments settle in, largest share first", () => {
    const market = {
      goods: [
        { good: 3, acceptance: 0.25 },
        { good: 0, acceptance: 0.75 },
        { good: 2, acceptance: 0 },
      ],
    } as unknown as MarketInfo;
    expect(paymentsText(GOODS, market)).toBe("grain 75% · hoe 25%");
  });
});

describe("price history", () => {
  const history = [
    { month: monthIndex(1, 11), good: 2, trades: 1, units: 1, paidH: 10 },
    { month: monthIndex(2, 1), good: 2, trades: 2, units: 2, paidH: 24 },
    { month: monthIndex(2, 1), good: 0, trades: 1, units: 20, paidH: 12 },
  ];

  it("counts months as the kernel does", () => {
    expect(monthIndex(1, 1)).toBe(0);
    expect(monthIndex(2, 3)).toBe(14);
    expect(monthText(14)).toBe("Mar, year 2");
  });

  it("gives each month what a unit was worth, with gaps where nothing sold", () => {
    const points = priceSeries(history, 2, monthIndex(2, 2), 4);
    expect(points.map((p) => p.month)).toEqual([10, 11, 12, 13]);
    expect(points.map((p) => p.hoursPerUnit)).toEqual([10, null, 12, null]);
    // Months before the calendar began are left out.
    expect(priceSeries(history, 2, 1, 12)).toHaveLength(2);
    expect(worthText(GOODS, 2, points)).toBe("12.0 h of work each");
    expect(worthText(GOODS, 0, priceSeries(history, 0, 12, 3))).toBe("6.0 h of work per 10 kg");
    expect(worthText(GOODS, 1, priceSeries(history, 1, 12, 3))).toBe("");
  });

  it("draws a run of points per stretch of months with trades", () => {
    const runs = sparkline(
      [
        { month: 0, hoursPerUnit: 10 },
        { month: 1, hoursPerUnit: 5 },
        { month: 2, hoursPerUnit: null },
        { month: 3, hoursPerUnit: 11 },
      ],
      90,
      22,
    );
    expect(runs).toHaveLength(2);
    // The dearest month sits near the top, 10% below it.
    expect(runs[0]).toBe("0.0,3.8 30.0,12.9");
    expect(runs[1]).toBe("90.0,2.0");
    expect(sparkline([{ month: 0, hoursPerUnit: null }], 90, 22)).toEqual([]);
  });
});
