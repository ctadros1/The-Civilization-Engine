import { describe, expect, it } from "vitest";

import type { StandingLine, TieLine } from "../src/net/messages.js";
import { closenessText, esteemText, standingText, tieText } from "../src/standing.js";

const DOMAINS = ["provision", "craft", "word", "counsel"];

function tie(over: Partial<TieLine> = {}): TieLine {
  return {
    person: 7,
    name: "Wren",
    familiarity: 0.3,
    warmth: 0.05,
    esteem: [0, 0, 0, 0],
    helpH: 0,
    reason: "kept them company at the hearth, last in May of year 2",
    mutual: true,
    ...over,
  };
}

describe("ties and standing in words", () => {
  it("say how well and how warmly someone is known", () => {
    expect(closenessText({ familiarity: 0.1, warmth: 0 })).toBe("known by sight");
    expect(closenessText({ familiarity: 0.3, warmth: 0.2 })).toBe("warm, known");
    expect(closenessText({ familiarity: 0.9, warmth: 0.6 })).toBe("close, well known");
  });

  it("name only the domains someone is esteemed in", () => {
    expect(esteemText([2, 0, 1.54, 0.01], DOMAINS)).toBe("provision 2, word 1.5");
    expect(esteemText([0, 0, 0, 0], DOMAINS)).toBe("");
    expect(esteemText([0, -1, 0, 0], DOMAINS)).toBe("craft -1");
  });

  it("put a tie in a line, with help owed either way and a tie not returned", () => {
    expect(tieText(tie(), DOMAINS)).toBe("known");
    expect(tieText(tie({ esteem: [3, 0, 0, 0], helpH: 4.2, mutual: false }), DOMAINS)).toBe(
      "known; esteemed for provision 3; owes them 4.2 h of help; not known back",
    );
    expect(tieText(tie({ helpH: -2 }), DOMAINS)).toBe("known; is owed 2 h of help");
  });

  it("put standing in a line, notables first", () => {
    const line: StandingLine = {
      person: 3,
      name: "Ash",
      household: 9,
      esteem: [4, 0, 0.5, 0],
      influence: 3,
      notable: true,
    };
    expect(standingText(line, DOMAINS)).toBe(
      "a notable; 3 count them among those they esteem most; esteemed for provision 4, word 0.5",
    );
    expect(standingText({ ...line, notable: false, influence: 0, esteem: [0, 0, 0, 0] }, DOMAINS)).toBe(
      "nobody counts them among those they esteem most; no standing yet",
    );
  });
});
