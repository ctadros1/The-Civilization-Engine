import { describe, expect, it } from "vitest";

import {
  introducible,
  knowRows,
  knowText,
  namesText,
  summaryText,
  techniqueRows,
} from "../src/knowledge.js";
import type { KnowLine, PersonRef, TechniqueHere, TechniqueInfo } from "../src/net/messages.js";

function technique(name: string, over: Partial<TechniqueInfo> = {}): TechniqueInfo {
  return {
    id: `core:technique/${name.toLowerCase().replaceAll(" ", "_")}`,
    name,
    can: `do ${name.toLowerCase()}`,
    domain: -1,
    requires: "",
    learnH: 50,
    upbringing: true,
    ...over,
  };
}

const TECHNIQUES = [
  technique("Knapping"),
  technique("Growing emmer"),
  technique("Weaving", { upbringing: false, requires: "Spinning" }),
];

function line(over: Partial<KnowLine> = {}): KnowLine {
  return {
    technique: 0,
    state: "known",
    hours: 50,
    learnH: 50,
    sinceMinute: 0,
    source: "brought it",
    sourcePerson: 0,
    usedMinute: 0,
    ...over,
  };
}

const person = (name: string, ageYears: number, id = 1): PersonRef => ({ id, name, ageYears });

function here(over: Partial<TechniqueHere> = {}): TechniqueHere {
  return {
    technique: 0,
    known: true,
    knowers: [person("Wren", 61.7)],
    learners: [],
    heard: [],
    practisedLastYear: 1,
    status: "known by one, Wren, aged 61",
    history: ["Year 1: brought by Wren."],
    ...over,
  };
}

describe("knowledge", () => {
  it("names people with their ages, and says how many more", () => {
    expect(namesText([])).toBe("nobody");
    expect(namesText([person("Wren", 61.7)])).toBe("Wren (61)");
    expect(namesText([person("Wren", 61.7), person("Ash", 29.2)])).toBe("Wren (61) and Ash (29)");
    const many = Array.from({ length: 9 }, (_, i) => person(`P${i}`, 20 + i, i));
    expect(namesText(many, 2)).toBe("P0 (20), P1 (21) and 7 more");
  });

  it("says what a person knows, is learning and has heard of", () => {
    expect(knowText(line({ source: "brought up with it by Wren" }))).toBe("brought up with it by Wren");
    expect(knowText(line({ source: "" }))).toBe("known");
    expect(
      knowText(line({ state: "learning", hours: 12.34, learnH: 50, source: "taught by Ash" })),
    ).toBe("learning, 12 h of 50 h (taught by Ash)");
    expect(knowText(line({ state: "learning", hours: 0.45, learnH: 2, source: "" }))).toBe(
      "learning, 0.5 h of 2 h",
    );
    expect(knowText(line({ state: "heard", source: "introduced by the observer" }))).toBe(
      "heard of (introduced by the observer)",
    );
  });

  it("lists what is known first, then what is being learnt, then what is only heard of", () => {
    const rows = knowRows(
      [
        line({ technique: 2, state: "heard" }),
        line({ technique: 0 }),
        line({ technique: 1 }),
        line({ technique: 9, state: "learning", hours: 3 }),
      ],
      TECHNIQUES,
    );
    expect(rows.map((r) => [r.name, r.state])).toEqual([
      ["Growing emmer", "known"],
      ["Knapping", "known"],
      ["technique 9", "learning"],
      ["Weaving", "heard"],
    ]);
  });

  it("offers to introduce only what someone does not know", () => {
    const options = introducible(
      [line({ technique: 0 }), line({ technique: 2, state: "heard" })],
      TECHNIQUES,
    );
    expect(options).toEqual([
      { technique: 1, name: "Growing emmer", heard: false },
      { technique: 2, name: "Weaving", heard: true },
    ]);
    expect(introducible([], TECHNIQUES)).toHaveLength(3);
  });

  it("puts a settlement's known techniques first and sums them up", () => {
    const list = [
      here({
        technique: 2,
        known: false,
        knowers: [],
        heard: [person("Ash", 30)],
        status: "lost in year 9 with Wren; one still know of it",
        practisedLastYear: 0,
      }),
      here({ technique: 0, learners: [person("Moss", 15)] }),
    ];
    const rows = techniqueRows(list, TECHNIQUES);
    expect(rows.map((r) => r.name)).toEqual(["Knapping", "Weaving"]);
    expect(rows[0]).toMatchObject({
      known: true,
      knowers: "Wren (61)",
      learners: "Moss (15)",
      heard: null,
      practised: "1 of 1 in the last year",
      can: "do knapping",
      requires: "",
    });
    expect(rows[1]).toMatchObject({
      known: false,
      knowers: "nobody",
      learners: null,
      heard: "Ash (30)",
      practised: "",
      requires: "Spinning",
    });
    expect(summaryText(list)).toBe("Knows 1 technique; has lost 1.");
    expect(summaryText([here(), here({ technique: 1 })])).toBe("Knows 2 techniques.");
    expect(summaryText([])).toBe("Knows no technique.");
  });
});
