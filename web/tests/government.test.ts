import * as flatbuffers from "flatbuffers";
import { describe, expect, it } from "vitest";

import {
  brokenText,
  labelText,
  lawDayText,
  levyText,
  reliefText,
  stanceText,
  membersText,
  statusText,
} from "../src/government.js";
import * as M from "../src/net/messages.js";
import * as W from "../src/schema/generated/tce/wire.js";

const DAY = 24 * 60;

describe("a polity's laws in words", () => {
  it("say the day a thing happened", () => {
    // Day 0 of the engine's calendar is 1 January of year 1.
    expect(lawDayText(0)).toBe("1 January of year 1");
    expect(lawDayText((365 + 31 + 3) * DAY + 600)).toBe("4 February of year 2");
  });

  it("say where a law stands", () => {
    expect(statusText({ status: "in force", outcome: "passed", meetsMinute: 0 })).toBe("in force");
    expect(statusText({ status: "lapsed", outcome: "passed", meetsMinute: 0 })).toBe(
      "lapsed: the one it named is gone",
    );
    expect(statusText({ status: "rejected", outcome: "failed", meetsMinute: 0 })).toBe("turned down");
    expect(statusText({ status: "rejected", outcome: "tied", meetsMinute: 0 })).toBe(
      "failed: evenly split",
    );
    expect(statusText({ status: "rejected", outcome: "no quorum", meetsMinute: 0 })).toBe(
      "failed: too few came",
    );
    expect(statusText({ status: "proposed", outcome: null, meetsMinute: 40 * DAY })).toBe(
      "before the gathering on 10 February of year 1",
    );
  });

  it("say a law a later one replaced or a repeal ended, and how many may decide", () => {
    expect(statusText({ status: "superseded", outcome: "passed", meetsMinute: 0 })).toBe(
      "superseded by a later law",
    );
    expect(statusText({ status: "carried", outcome: "passed", meetsMinute: 0 })).toBe(
      "carried: the law it named is ended",
    );
    expect(membersText({ members: 24, bodyMembers: 24 })).toBe("24 adults");
    expect(membersText({ members: 24, bodyMembers: 9 })).toBe("24 adults, 9 of whom may decide");
  });

  it("join a stance and its reason", () => {
    expect(stanceText({ stance: "against", why: "their household stands to lose" })).toBe(
      "against: their household stands to lose",
    );
  });

  it("tell what the levy brought in and what people did", () => {
    const none = {
      complied: 0,
      couldNot: 0,
      evaded: 0,
      refused: 0,
      unaware: 0,
      leviedKg: 0,
      withheldKg: 0,
    };
    expect(levyText(none)).toBe("");
    expect(levyText({ ...none, complied: 1, leviedKg: 3.4 })).toBe("paid 1 time (3.4 kg)");
    expect(
      levyText({
        complied: 249,
        couldNot: 2,
        evaded: 16,
        refused: 0,
        unaware: 1,
        leviedKg: 326,
        withheldKg: 18,
      }),
    ).toBe(
      "paid 249 times (326 kg); kept back 16 times, could not pay 2, did not know of it 1 (18 kg withheld)",
    );
    expect(levyText({ ...none, complied: 2, refused: 4, leviedKg: 5, withheldKg: 9 })).toBe(
      "paid 2 times (5.0 kg); kept back in a refusal 4 times (9.0 kg withheld)",
    );
  });

  it("say what a polity would be called, and how sure", () => {
    const none = { label: "", labelModifiers: [], labelConfidence: 0 };
    expect(labelText(none)).toBe("");
    expect(labelText({ label: "Council community", labelModifiers: [], labelConfidence: 0.2 })).toBe(
      "Council community (confidence 0.20)",
    );
    expect(
      labelText({
        label: "Council community",
        labelModifiers: ["a storekeeper's office", "its levy mostly paid"],
        labelConfidence: 2 / 3,
      }),
    ).toBe("Council community: a storekeeper's office; its levy mostly paid (confidence 0.67)");
  });

  it("say how often a curfew was broken", () => {
    expect(brokenText({ broken: 0, brokenUnaware: 0 })).toBe("");
    expect(brokenText({ broken: 1, brokenUnaware: 0 })).toBe("broken 1 time by people who knew of it");
    expect(brokenText({ broken: 12, brokenUnaware: 3 })).toBe(
      "broken 12 times by people who knew of it, 3 by people who did not",
    );
    expect(brokenText({ broken: 0, brokenUnaware: 2 })).toBe(
      "broken 2 times by people who did not know of it",
    );
  });

  it("tell what the store gave", () => {
    expect(reliefText({ relieved: 0, reliefKg: 0, unanswered: 0 })).toBe("");
    expect(reliefText({ relieved: 3, reliefKg: 45, unanswered: 1 })).toBe(
      "the store answered 3 asks (45 kg); 1 it could not",
    );
  });
});

describe("the government on the wire", () => {
  it("decodes a polity with a law's whole history", () => {
    const b = new flatbuffers.Builder(512);
    const stanceName = b.createString("Ada");
    const why = b.createString("their household stands to gain");
    const stance = W.StanceLine.createStanceLine(b, 7n, stanceName, 0, 2.5, 0.4, why, 0.3, -0.6);
    const stances = W.LawLine.createStancesVector(b, [stance]);
    const what = b.createString("a common store, taking a tenth of each harvest");
    const policy = b.createString("core:policy/common_store");
    const sponsorName = b.createString("Talia");
    const issue = b.createString("food would not last until the harvest");
    const decision = b.createString("agreed: 1 for, 0 against; 1 of 4 adults came, 1 needed");
    const law = W.LawLine.createLawLine(
      b,
      208n,
      what,
      policy,
      0.1,
      5,
      1,
      9n,
      sponsorName,
      BigInt(100 * DAY),
      issue,
      BigInt(101 * DAY),
      BigInt(102 * DAY),
      0,
      decision,
      4,
      1,
      stances,
      6,
      12,
      1,
      2,
      0,
      30,
      4,
      3,
      45,
      0,
      0n,
      0,
      7,
      2,
      3,
    );
    const laws = W.PolityLine.createLawsVector(b, [law]);
    const office = b.createString("Storekeeper: Ada, since 3 May of year 2");
    const offices = W.PolityLine.createOfficesVector(b, [office]);
    const label = b.createString("Council community");
    const modifier = b.createString("a storekeeper's office");
    const modifiers = W.PolityLine.createLabelModifiersVector(b, [modifier]);
    const reason = b.createString("All its adults may come and decide.");
    const labelWhy = W.PolityLine.createLabelWhyVector(b, [reason]);
    const gatheringCases = W.PolityLine.createGatheringCasesVector(b, [
      b.createString("Rilla's case against Tam"),
    ]);
    const customHistory = W.PolityLine.createCustomHistoryVector(b, [
      b.createString("Since 1 March of year 1, the founding custom: the adults decide."),
      b.createString("Since 9 June of year 2, by the amendment Ada proposed: the elders decide."),
    ]);
    const factions = W.PolityLine.createFactionsVector(b, [
      b.createString("Mira's faction, against the gathering, since 3 May of year 2: 5 members"),
    ]);
    const petitions = W.PolityLine.createPetitionsVector(b, [
      b.createString(
        "Mira's faction petitioned the gathering on 3 May of year 2 for an end to the common store's levy (the store gives what it holds): 9 came; the gathering turned it down",
      ),
    ]);
    const refusals = W.PolityLine.createRefusalsVector(b, [
      b.createString(
        "Mira's faction called on its members on 3 May of year 2 to keep back the levy of a common store, taking a tenth of each harvest, until 3 May of year 3: 6 kept back 240 kg",
      ),
    ]);
    const revolts = W.PolityLine.createRevoltsVector(b, [
      b.createString(
        "Mira's faction called on everyone on 3 May of year 2 to stand with it: from now on, the elders decide; it held on 12 May of year 2, 9 standing with it, 2 with the gathering, 1 with neither",
      ),
    ]);
    const coups = W.PolityLine.createCoupsVector(b, [
      b.createString(
        "Bo called on those who keep the watch on 3 May of year 2 to take the deciding with them; it held on 11 May of year 2, 2 of 3 watchers with it, none with the gathering",
      ),
    ]);
    const relationWhy = W.RelationLine.createWhyVector(b, [
      b.createString("3 of 5 of its households know Ashford"),
    ]);
    const payments = W.AgreementLine.createPaymentsVector(b, [
      b.createString(
        "100 kg of grain from Stonewick's store, owed from 20 May of year 3: 100 kg set aside, 99 kg arrived; met on 24 May of year 3",
      ),
    ]);
    const agreement = W.AgreementLine.createAgreementLine(
      b,
      901n,
      b.createString("leave for Ashford's people to use the places Stonewick claims, for a year"),
      b.createString("in force since 20 May of year 3"),
      b.createString("proposed by Ada on 2 May of year 3; passed on 3 May of year 3, 9 for, 2 against, 14 of 30 came"),
      b.createString("proposed by Bram on 5 May of year 3; passed on 6 May of year 3, 7 for, 0 against, 9 of 20 came"),
      payments,
    );
    const agreements = W.RelationLine.createAgreementsVector(b, [agreement]);
    const relation = W.RelationLine.createRelationLine(
      b,
      301n,
      b.createString("Ashford"),
      b.createString("under agreement"),
      relationWhy,
      agreements,
    );
    const relations = W.PolityLine.createRelationsVector(b, [relation]);
    const name = b.createString("Stonewick");
    const custom = b.createString("The adults who come to the hearth decide by acclamation.");
    const store = b.createString("grain 26 kg");
    const polity = W.PolityLine.createPolityLine(
      b,
      300n,
      12n,
      name,
      0n,
      custom,
      4,
      store,
      26,
      laws,
      0n,
      0n,
      0,
      offices,
      label,
      modifiers,
      labelWhy,
      0.5,
      gatheringCases,
      customHistory,
      3,
      factions,
      petitions,
      refusals,
      revolts,
      coups,
      relations,
    );
    const polities = W.Government.createPolitiesVector(b, [polity]);
    const government = W.Government.createGovernment(b, BigInt(103 * DAY), polities);
    const response = W.Response.createResponse(b, W.ResponseBody.Government, government);
    b.finish(response);
    const body = M.decodeResponse(b.asUint8Array());
    expect(body.kind).toBe("government");
    if (body.kind !== "government") return;
    const p = body.government.polities[0]!;
    expect(p).toMatchObject({
      name: "Stonewick",
      members: 4,
      store: "grain 26 kg",
      gatheringLaw: 0,
      offices: ["Storekeeper: Ada, since 3 May of year 2"],
      label: "Council community",
      labelModifiers: ["a storekeeper's office"],
      labelWhy: ["All its adults may come and decide."],
      labelConfidence: 0.5,
      gatheringCases: ["Rilla's case against Tam"],
      bodyMembers: 3,
    });
    expect(p.factions).toEqual([
      "Mira's faction, against the gathering, since 3 May of year 2: 5 members",
    ]);
    expect(p.petitions).toHaveLength(1);
    expect(p.petitions[0]).toContain("the gathering turned it down");
    expect(p.refusals).toHaveLength(1);
    expect(p.refusals[0]).toContain("6 kept back 240 kg");
    expect(p.revolts).toHaveLength(1);
    expect(p.revolts[0]).toContain("it held on 12 May of year 2");
    expect(p.coups).toEqual([
      "Bo called on those who keep the watch on 3 May of year 2 to take the deciding with them; it held on 11 May of year 2, 2 of 3 watchers with it, none with the gathering",
    ]);
    expect(p.relations).toEqual([
      {
        polity: 301,
        name: "Ashford",
        label: "under agreement",
        why: ["3 of 5 of its households know Ashford"],
        agreements: [
          {
            id: 901,
            terms: "leave for Ashford's people to use the places Stonewick claims, for a year",
            state: "in force since 20 May of year 3",
            ours: "proposed by Ada on 2 May of year 3; passed on 3 May of year 3, 9 for, 2 against, 14 of 30 came",
            theirs: "proposed by Bram on 5 May of year 3; passed on 6 May of year 3, 7 for, 0 against, 9 of 20 came",
            payments: [
              "100 kg of grain from Stonewick's store, owed from 20 May of year 3: 100 kg set aside, 99 kg arrived; met on 24 May of year 3",
            ],
          },
        ],
      },
    ]);
    expect(p.customHistory).toHaveLength(2);
    expect(p.customHistory[1]).toContain("by the amendment Ada proposed");
    const l = p.laws[0]!;
    expect(l).toMatchObject({
      id: 208,
      status: "in force",
      outcome: "passed",
      sponsorName: "Talia",
      known: 6,
      complied: 12,
      couldNot: 1,
      evaded: 2,
      refused: 3,
      relieved: 3,
      broken: 7,
      brokenUnaware: 2,
      reliefKg: 45,
    });
    expect(l.stances).toHaveLength(1);
    expect(l.stances[0]).toMatchObject({
      person: 7,
      name: "Ada",
      stance: "for",
      gain: 2.5,
      why: "their household stands to gain",
    });
    expect(l.stances[0]!.regard).toBeCloseTo(0.4, 5);
    expect(l.stances[0]!.opinion).toBeCloseTo(0.3, 5);
    expect(l.stances[0]!.values).toBeCloseTo(-0.6, 5);
  });
});
