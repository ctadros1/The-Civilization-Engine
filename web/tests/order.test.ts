import * as flatbuffers from "flatbuffers";
import { describe, expect, it } from "vitest";

import * as M from "../src/net/messages.js";
import {
  believedText,
  choiceText,
  happenedRest,
  happenedText,
  seenText,
  totalsText,
} from "../src/order.js";
import * as W from "../src/schema/generated/tce/wire.js";

const DAY = 24 * 60;

function incident(over: Partial<M.IncidentLine> = {}): M.IncidentLine {
  return {
    id: 1,
    minute: (31 + 3) * DAY + 600,
    settlement: 12,
    actor: 7,
    actorName: "Tam",
    target: 40,
    targetName: "the household of Rilla",
    outcome: "taken",
    what: "12 kg of grain",
    kcal: 40_800,
    seenBy: [],
    ...over,
  };
}

function known(over: Partial<M.KnownLine> = {}): M.KnownLine {
  return {
    incident: 1,
    knowTaker: 0,
    knowLoss: 0,
    sources: 0,
    victimKnows: false,
    response: "",
    owed: "",
    ...over,
  };
}

describe("takings in words", () => {
  it("say what happened, plainly", () => {
    expect(happenedText(incident())).toBe(
      "Tam took 12 kg of grain from the household of Rilla on 4 February of year 1.",
    );
    expect(happenedRest(incident({ outcome: "turned back" }))).toBe(
      "went to take from the household of Rilla on 4 February of year 1 and turned back: someone was home.",
    );
    expect(happenedRest(incident({ outcome: "fled" }))).toContain("fled with nothing");
  });

  it("say who saw it", () => {
    expect(seenText(incident())).toBe("Nobody saw it");
    expect(seenText(incident({ seenBy: ["Bram"] }))).toBe("Seen by Bram");
    expect(seenText(incident({ seenBy: ["Bram", "Ada"] }))).toBe("Seen by Bram and Ada");
    expect(seenText(incident({ seenBy: ["Bram", "Ada", "Cole"] }))).toBe(
      "Seen by Bram, Ada and 1 other",
    );
    expect(seenText(incident({ seenBy: ["Bram", "Ada", "Cole", "Wren"] }))).toBe(
      "Seen by Bram, Ada and 2 others",
    );
  });

  it("keep what people believe apart from what happened", () => {
    expect(believedText(undefined)).toBe("Nobody knows of it");
    expect(believedText(known({ knowLoss: 3 }))).toBe("3 know only of a loss");
    expect(believedText(known({ knowTaker: 5, sources: 1, knowLoss: 1 }))).toBe(
      "5 believe they know who took, from 1 account; 1 knows only of a loss",
    );
    expect(believedText(known({ knowTaker: 1, sources: 2 }))).toBe(
      "1 believes they know who took, from 2 accounts",
    );
  });

  it("say what the household taken from chose", () => {
    expect(choiceText(known())).toBe("");
    expect(choiceText(known({ response: "Rilla let it go" }))).toBe("Rilla let it go");
    expect(choiceText(known({ response: "Rilla demanded the food back", owed: "paid" }))).toBe(
      "Rilla demanded the food back: paid",
    );
  });

  it("sum the takings up", () => {
    const none: M.OrderInfo = {
      minute: 0,
      incidents: [],
      known: [],
      attempts: 0,
      takings: 0,
      seen: 0,
      knownToVictims: 0,
      demands: 0,
      met: 0,
      refused: 0,
      refusals: 0,
    };
    expect(totalsText(none)).toBe("Nobody has gone to take from another household's store.");
    expect(
      totalsText({ ...none, attempts: 5, takings: 3, seen: 2, knownToVictims: 2, demands: 1, met: 1 }),
    ).toBe(
      "5 attempts, 3 takings; 2 seen; 2 known to the household taken from; 1 demand to give back (1 met, 0 refused); 0 asks refused to those believed to have taken.",
    );
  });

  it("decode truth and knowledge into their own lists", () => {
    const b = new flatbuffers.Builder(256);
    const actorName = b.createString("Tam");
    const targetName = b.createString("the household of Rilla");
    const what = b.createString("12 kg of grain");
    const seen = W.IncidentLine.createSeenByVector(b, [b.createString("Bram")]);
    const line = W.IncidentLine.createIncidentLine(
      b,
      3,
      BigInt(40 * DAY),
      12n,
      7n,
      actorName,
      40n,
      targetName,
      0,
      what,
      40_800,
      seen,
    );
    const incidents = W.Order.createIncidentsVector(b, [line]);
    const response = b.createString("Rilla demanded the food back");
    const owed = b.createString("paid");
    const k = W.KnownLine.createKnownLine(b, 3, 4, 0, 1, true, response, owed);
    const knownList = W.Order.createKnownVector(b, [k]);
    const order = W.Order.createOrder(
      b,
      BigInt(41 * DAY),
      incidents,
      knownList,
      4,
      3,
      1,
      1,
      1,
      1,
      0,
      2n,
    );
    b.finish(W.Response.createResponse(b, W.ResponseBody.Order, order));
    const body = M.decodeResponse(b.asUint8Array());
    expect(body.kind).toBe("order");
    if (body.kind !== "order") return;
    const o = body.order;
    expect(o).toMatchObject({ attempts: 4, takings: 3, seen: 1, demands: 1, met: 1, refusals: 2 });
    expect(o.incidents[0]).toMatchObject({
      id: 3,
      actor: 7,
      actorName: "Tam",
      target: 40,
      outcome: "taken",
      what: "12 kg of grain",
      seenBy: ["Bram"],
    });
    expect(o.known[0]).toMatchObject({
      incident: 3,
      knowTaker: 4,
      sources: 1,
      victimKnows: true,
      response: "Rilla demanded the food back",
      owed: "paid",
    });
  });
});
