import { describe, expect, it } from "vitest";

import type { GrievanceLine, HeardLine, IdeologyLine, NormLine, PositionLine, ValueLine } from "../src/net/messages.js";
import {
  claimText,
  grievanceFacts,
  grievanceHead,
  heardHow,
  heardWhen,
  ideologyText,
  keennessText,
  normText,
  positionText,
  valuesText,
} from "../src/word.js";

const DAY = 24 * 60;

function grievance(over: Partial<GrievanceLine> = {}): GrievanceLine {
  return {
    issue: 0,
    over: "food when their household was short",
    blamed: "the gathering",
    law: 9,
    harmDays: 5,
    unresolvedDays: 4,
    activation: 0.5,
    madeMinute: 62 * DAY,
    raisedMinute: 62 * DAY,
    reason: "the common store was empty when their household was short",
    ...over,
  };
}

function heard(over: Partial<HeardLine> = {}): HeardLine {
  return {
    kind: 0,
    what: "a gathering meets on 9 May of year 1 to hear a case",
    from: 7,
    fromName: "Wren",
    origin: 3,
    firstMinute: 62 * DAY,
    lastMinute: 62 * DAY,
    ...over,
  };
}

describe("grievances and news in words", () => {
  it("say how keenly a grievance is felt", () => {
    expect(keennessText(0.9)).toBe("keenly felt");
    expect(keennessText(0.3)).toBe("felt");
    expect(keennessText(0.05)).toBe("faintly felt");
  });

  it("name the party, the issue, the wrong and what is left to make good", () => {
    const g = grievance();
    expect(grievanceHead(g)).toBe("against the gathering, over food when their household was short");
    const facts = grievanceFacts(g);
    expect(facts).toMatch(/^the common store was empty when their household was short; /);
    expect(facts).toContain("4 of 5 days of food not made good");
    expect(facts).toContain("held since ");
    expect(facts).not.toContain("last raised");
    expect(facts).toMatch(/; felt$/);
  });

  it("say when a grievance was raised again, and when it was made good", () => {
    const again = grievanceFacts(grievance({ raisedMinute: 70 * DAY }));
    expect(again).toContain("last raised on ");
    expect(grievanceFacts(grievance({ unresolvedDays: 0 }))).toContain("5 days of food, all made good");
  });

  it("say who told of a claim and when, or that it began with them", () => {
    expect(heardHow(heard())).toMatch(/^told by Wren on \d+ \w+ of year \d+$/);
    expect(heardHow(heard({ from: 0, fromName: "" }))).toMatch(/^they knew it first, on /);
    expect(heardWhen(heard({ lastMinute: 64 * DAY }))).toMatch(/, last heard on /);
  });

  it("read a claim as a sentence", () => {
    expect(claimText(heard())).toBe("A gathering meets on 9 May of year 1 to hear a case.");
    expect(claimText(heard({ what: "" }))).toBe("");
  });

  it("say where someone stands and what moved them", () => {
    const p = (over: Partial<PositionLine> = {}): PositionLine => ({
      question: "whether to keep a common store",
      lean: "leaning for",
      x: 0.66,
      anchor: 0.58,
      salience: 1,
      heard: 4,
      ...over,
    });
    expect(positionText(p())).toBe(
      "leaning for (0.66); their household's lot and what they hold dear would make it 0.58; talk at the hearth, taken in 4 times, drew them toward it",
    );
    expect(positionText(p({ x: 0.5, anchor: 0.6, lean: "undecided", heard: 1 }))).toContain(
      "taken in once, turned them against it",
    );
    expect(positionText(p({ x: 0.58 }))).toBe(
      "leaning for (0.58); as their household's lot and what they hold dear make it",
    );
  });

  it("say how far someone holds a norm, what they believe others do and whether it holds them", () => {
    const n = (over: Partial<NormLine> = {}): NormLine => ({
      statement: "what the gathering decides binds everyone",
      holds: "holds firmly",
      believes: "most households pay what the gathering asks",
      endorse: 0.81,
      expect: 0.72,
      threshold: 0.4,
      activation: 0.96,
      heard: 3,
      ...over,
    });
    expect(normText(n())).toBe(
      "holds firmly (0.81); believes most households pay what the gathering asks (0.72), from 3 accounts at the hearth; enough do to hold them to it",
    );
    expect(normText(n({ heard: 0, activation: 0.1 }))).toContain(
      "from what their own people believed; too few do to hold them to it",
    );
    expect(normText(n({ heard: 1 }))).toContain("from one account at the hearth");
  });

  it("say what someone holds dear", () => {
    const vs: ValueLine[] = [
      { name: "safety from want and harm", words: "holds safety from want and harm dear", v: 0.624 },
      {
        name: "a household's say over what is its own",
        words: "cares as most do for a household's say over what is its own",
        v: -0.1,
      },
    ];
    expect(valuesText(vs)).toBe(
      "holds safety from want and harm dear (0.62); cares as most do for a household's say over what is its own (-0.10)",
    );
    expect(valuesText([])).toBe("");
  });

  it("say what someone holds to and whom they had it from", () => {
    const d = (over: Partial<IdeologyLine> = {}): IdeologyLine => ({
      name: "common provision",
      legitimacy: "what the village gathers, the village keeps against a lean year",
      sinceMinute: 62 * DAY,
      from: 0,
      fromName: "",
      ...over,
    });
    expect(ideologyText(d())).toBe(
      "'what the village gathers, the village keeps against a lean year'; brought with them",
    );
    expect(ideologyText(d({ from: 7, fromName: "Wren" }))).toMatch(/; from Wren since \d+ \w+ of year \d+$/);
  });
});
