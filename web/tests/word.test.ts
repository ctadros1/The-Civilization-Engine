import { describe, expect, it } from "vitest";

import type { GrievanceLine, HeardLine } from "../src/net/messages.js";
import {
  claimText,
  grievanceFacts,
  grievanceHead,
  heardHow,
  heardWhen,
  keennessText,
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
});
