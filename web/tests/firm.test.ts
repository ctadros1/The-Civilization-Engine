import { describe, expect, it } from "vitest";

import {
  holdingsText,
  hoursText,
  lifeText,
  linesText,
  marginText,
  monthAmount,
  quantityText,
  statementRows,
} from "../src/firm.js";
import type { GoodInfo, MonthStatement } from "../src/net/messages.js";

function good(name: string, purpose: "food" | "tool"): GoodInfo {
  return {
    id: `core:good/${name.toLowerCase()}`,
    name,
    purpose,
    kcalPerKg: 0,
    eaten: "never",
    toolLifeH: purpose === "tool" ? 300 : 0,
  } as GoodInfo;
}

/** Grain (0), toolstone (1), a sickle (2), a hoe (3). */
const GOODS = [good("Grain", "food"), good("Toolstone", "food"), good("Sickle", "tool"), good("Hoe", "tool")];

function month(m: number, patch: Partial<MonthStatement> = {}): MonthStatement {
  return { month: m, lines: [], ownerH: 0, hiredH: 0, incomeH: 0, costsH: 0, stockH: 0, ...patch };
}

describe("workshop words", () => {
  it("names what a workshop makes", () => {
    expect(linesText(GOODS, [2])).toBe("sickles");
    expect(linesText(GOODS, [3, 2])).toBe("hoes and sickles");
    expect(linesText(GOODS, [0, 3, 2])).toBe("grain, hoes and sickles");
    expect(linesText(GOODS, [])).toBe("nothing");
    expect(linesText(GOODS, [9])).toBe("goods no longer known");
  });

  it("shows hours and margins, and nothing as a dash", () => {
    expect(hoursText(3.04)).toBe("3.0");
    expect(hoursText(0.01)).toBe("—");
    expect(marginText(2.06)).toBe("+2.1");
    expect(marginText(-0.6)).toBe("−0.6");
    expect(marginText(0)).toBe("—");
  });

  it("counts tools and weighs goods without their nouns in a statement's cells", () => {
    expect(quantityText(GOODS[2], 1.02)).toBe("1");
    expect(quantityText(GOODS[2], 1.5)).toBe("1.5");
    expect(quantityText(GOODS[0], 12.4)).toBe("12 kg");
  });

  it("says what a workshop holds", () => {
    expect(
      holdingsText(GOODS, [
        { good: 2, kg: 2 },
        { good: 0, kg: 14.2 },
        { good: 1, kg: 0.001 },
      ]),
    ).toBe("2.0 sickles · 14 kg grain");
    expect(holdingsText(GOODS, [])).toBe("nothing");
  });
});

describe("workshop statements", () => {
  const months = [
    month(14, {
      lines: [
        { kind: "made", good: 2, amount: 1 },
        { kind: "used", good: 1, amount: 0.6 },
      ],
      ownerH: 3,
      costsH: 1.2,
      stockH: 6,
    }),
    month(15, {
      lines: [
        { kind: "made", good: 2, amount: 0.5 },
        { kind: "made", good: 2, amount: 1 },
        { kind: "sold", good: 2, amount: 1 },
        { kind: "paid", good: 0, amount: 12 },
      ],
      ownerH: 2,
      hiredH: 1.5,
      incomeH: 7.5,
      costsH: 2.5,
      stockH: 3,
    }),
  ];

  it("adds up a month's lines by kind and good", () => {
    expect(monthAmount(months[1]!, "made", 2)).toBe(1.5);
    expect(monthAmount(months[1]!, "sold", 2)).toBe(1);
    expect(monthAmount(months[1]!, "sold", 3)).toBe(0);
  });

  it("lists the latest months first, with what was made, sold, worked and earned", () => {
    const rows = statementRows(GOODS, [2], months);
    expect(rows.map((r) => r.label)).toEqual(["Apr, year 2", "Mar, year 2"]);
    expect(rows[0]).toMatchObject({
      made: "1.5",
      sold: "1",
      worked: "3.5 (1.5)",
      income: "7.5",
      costs: "2.5",
      margin: "+5.0",
      stock: "3.0",
    });
    expect(rows[1]).toMatchObject({ made: "1", sold: "—", worked: "3.0", margin: "−1.2" });
    expect(statementRows(GOODS, [2], months, 1)).toHaveLength(1);
    // A workshop that makes more than one good names them.
    expect(statementRows(GOODS, [3, 2], months)[0]).toMatchObject({ made: "1.5 sickles", sold: "1 sickle" });
  });

  it("sums a workshop's life in hours of its owners' work", () => {
    expect(lifeText(months)).toBe(
      "Its owners worked 5.0 h for it and hired 1.5 h. What it sold brought in 7.5 h of their own work; its inputs and wages cost 3.7 h.",
    );
    expect(lifeText([months[0]!])).toBe(
      "Its owners worked 3.0 h for it and hired no one. It has been paid nothing yet; its inputs and wages cost 1.2 h of their own work.",
    );
  });
});
