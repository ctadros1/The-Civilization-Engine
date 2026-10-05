import * as flatbuffers from "flatbuffers";
import { describe, expect, it } from "vitest";

import * as M from "../src/net/messages.js";
import {
  activityColour,
  estimateMinute,
  missingTrips,
  personPosition,
  positionOnTrip,
  pruneTrips,
  spread,
} from "../src/people.js";
import * as W from "../src/schema/generated/tce/wire.js";

const bb = (bytes: Uint8Array) => new flatbuffers.ByteBuffer(bytes);

function trip(id: number, rev = 0): M.TripInfo {
  return {
    id,
    rev,
    person: 7,
    departMinute: 100,
    points: new Float32Array([0, 0, 10, 0, 10, 20]),
    minutes: new Float32Array([0, 1, 3]),
  };
}

function brief(id: number, tripId = 0, tripRev = 0): M.PersonBrief {
  return {
    id,
    x: 50,
    y: 60,
    activity: 0,
    trip: tripId,
    tripRev,
    sex: "female",
    ageYears: 30,
    household: 1,
    asleep: false,
  };
}

describe("trips", () => {
  it("interpolate between vertices like civ-agents Trip::position_at", () => {
    const t = trip(1);
    expect(positionOnTrip(t, 90)).toEqual([0, 0]);
    expect(positionOnTrip(t, 100.5)).toEqual([5, 0]);
    expect(positionOnTrip(t, 102)).toEqual([10, 10]);
    expect(positionOnTrip(t, 500)).toEqual([10, 20]);
  });

  it("place walkers on known trips and others where the snapshot put them", () => {
    const trips = new Map([[1, trip(1, 2)]]);
    expect(personPosition(brief(7, 1, 2), trips, 100.5)).toEqual([5, 0]);
    // A stale revision or an unknown trip falls back to the snapshot position.
    expect(personPosition(brief(7, 1, 3), trips, 100.5)).toEqual([50, 60]);
    expect(personPosition(brief(7, 9), trips, 100.5)).toEqual([50, 60]);
    expect(personPosition(brief(7), trips, 100.5)).toEqual([50, 60]);
  });

  it("are fetched when missing or stale and dropped when finished", () => {
    const trips = new Map([
      [1, trip(1, 0)],
      [2, trip(2, 0)],
    ]);
    const people = [brief(1, 1, 0), brief(2, 3, 0), brief(3, 2, 1), brief(4)];
    expect(missingTrips(people, trips)).toEqual([3, 2]);
    pruneTrips([brief(1, 1, 0)], trips);
    expect([...trips.keys()]).toEqual([1]);
  });
});

describe("spreading", () => {
  it("puts people at one spot on a ring and leaves others alone", () => {
    const out = spread(
      [
        [10, 10],
        [10.2, 10.1],
        [50, 50],
      ],
      2,
      0.75,
    );
    expect(out[2]).toEqual([50, 50]);
    expect(Math.hypot(out[0]![0] - 10, out[0]![1] - 10)).toBeCloseTo(2);
    expect(Math.hypot(out[0]![0] - out[1]![0], out[0]![1] - out[1]![1])).toBeGreaterThan(3);
  });
});

describe("the clock estimate", () => {
  const clock: M.Clock = {
    minute: 1000,
    year: 1,
    month: 3,
    day: 1,
    hour: 6,
    minuteOfHour: 0,
    season: "spring",
    paused: false,
    speed: 960,
    mode: "detailed",
  };
  it("runs on at the clock's speed and stops while paused", () => {
    expect(estimateMinute(clock, 0, 500)).toBeCloseTo(1008);
    expect(estimateMinute({ ...clock, paused: true }, 0, 500)).toBe(1000);
  });
  it("stands at midnight in Accelerated mode, until the next day's frame", () => {
    const daily = { ...clock, speed: Infinity, mode: "accelerated" as const };
    expect(estimateMinute(daily, 0, 500)).toBe(1000);
    expect(estimateMinute({ ...daily, speed: 5760 }, 0, 60_000)).toBe(1000);
  });
  it("never runs more than a real second ahead", () => {
    expect(estimateMinute(clock, 0, 60_000)).toBe(1016);
    expect(estimateMinute({ ...clock, speed: 96 }, 0, 60_000)).toBe(1001.6);
  });
});

describe("activity colours", () => {
  it("are stable per content id and distinct for the core activities", () => {
    const a = activityColour({ id: "core:activity/sleep", name: "Sleep", doing: "sleeping" });
    const b = activityColour({ id: "core:activity/eat", name: "Eat", doing: "eating" });
    const modded = { id: "mod:activity/weave", name: "Weave", doing: "weaving" };
    expect(a).not.toBe(b);
    expect(activityColour(modded)).toBe(activityColour(modded));
    expect(activityColour(undefined)).toBe(0xffffff);
  });
});

describe("people payloads", () => {
  it("decode from a snapshot", () => {
    const b = new flatbuffers.Builder(256);
    W.PersonBrief.startPersonBrief(b);
    W.PersonBrief.addId(b, 12n);
    W.PersonBrief.addPos(b, W.Vec2.createVec2(b, 3.5, 4.5));
    W.PersonBrief.addActivity(b, 2);
    W.PersonBrief.addTrip(b, 9n);
    W.PersonBrief.addTripRev(b, 1);
    W.PersonBrief.addSex(b, W.Sex.Male);
    W.PersonBrief.addAgeYears(b, 31.5);
    W.PersonBrief.addHousehold(b, 4n);
    W.PersonBrief.addAsleep(b, true);
    const person = W.PersonBrief.endPersonBrief(b);
    const people = W.Snapshot.createPeopleVector(b, [person]);
    const name = b.createString("Alderford");
    W.SettlementBrief.startSettlementBrief(b);
    W.SettlementBrief.addId(b, 3n);
    W.SettlementBrief.addName(b, name);
    W.SettlementBrief.addHearth(b, W.Vec2.createVec2(b, 100, 200));
    W.SettlementBrief.addFoundedMinute(b, 85_320n);
    W.SettlementBrief.addPopulation(b, 40);
    W.SettlementBrief.addFoodDays(b, 1.5);
    W.SettlementBrief.addFoodShort(b, true);
    const settlement = W.SettlementBrief.endSettlementBrief(b);
    const settlements = W.Snapshot.createSettlementsVector(b, [settlement]);
    W.Snapshot.startSnapshot(b);
    W.Snapshot.addPeople(b, people);
    W.Snapshot.addSettlements(b, settlements);
    W.Snapshot.addChronicleHead(b, 2n);
    b.finish(W.Snapshot.endSnapshot(b));
    const s = M.decodeSnapshot(b.asUint8Array());
    expect(s.people).toEqual([
      {
        id: 12,
        x: 3.5,
        y: 4.5,
        activity: 2,
        trip: 9,
        tripRev: 1,
        sex: "male",
        ageYears: 31.5,
        household: 4,
        asleep: true,
      },
    ]);
    expect(s.settlements).toEqual([
      {
        id: 3,
        name: "Alderford",
        x: 100,
        y: 200,
        foundedMinute: 85_320,
        population: 40,
        foodDays: 1.5,
        foodShort: true,
        harvestKg: 0,
      },
    ]);
    expect(s.chronicleHead).toBe(2);
  });

  it("build people queries the generated reader understands", () => {
    const trips = W.Query.getRootAsQuery(bb(M.getTrips([4, 5])));
    expect(trips.bodyType()).toBe(W.QueryBody.GetTrips);
    const ids = trips.body(new W.GetTrips()) as W.GetTrips;
    expect([ids.ids(0), ids.ids(1), ids.idsLength()]).toEqual([4n, 5n, 2]);
    const person = W.Query.getRootAsQuery(bb(M.getPerson(12, 8)));
    const p = person.body(new W.GetPerson()) as W.GetPerson;
    expect([p.id(), p.decisions()]).toEqual([12n, 8]);
    const chronicle = W.Query.getRootAsQuery(bb(M.getChronicle(3, 50)));
    const c = chronicle.body(new W.GetChronicle()) as W.GetChronicle;
    expect([c.afterSeq(), c.limit()]).toEqual([3n, 50]);
    const nw = W.Command.getRootAsCommand(
      bb(M.newWorld({ seed: 1n, presetId: "p", sizeCells: 256, name: "n", bandSize: 30 })),
    );
    expect((nw.body(new W.NewWorld()) as W.NewWorld).bandSize()).toBe(30);
  });

  it("decode a person's stores and load, and the goods they refer to", () => {
    const b = new flatbuffers.Builder(256);
    const name = b.createString("Ilse");
    W.PersonInfo.startStoresVector(b, 2);
    // Structs are written back to front.
    W.StoreLine.createStoreLine(b, 3, 14.5);
    W.StoreLine.createStoreLine(b, 1, 812.25);
    const stores = b.endVector();
    W.PersonInfo.startSkillsVector(b, 2);
    W.SkillLine.createSkillLine(b, 4, 0.25);
    W.SkillLine.createSkillLine(b, 0, 0.75);
    const skills = b.endVector();
    W.PersonInfo.startPersonInfo(b);
    W.PersonInfo.addId(b, 12n);
    W.PersonInfo.addName(b, name);
    W.PersonInfo.addAlive(b, true);
    W.PersonInfo.addCarryGood(b, 2);
    W.PersonInfo.addCarryKg(b, 12.5);
    W.PersonInfo.addCarryFoodKcal(b, 7500);
    W.PersonInfo.addHouseholdFuelDays(b, 2.5);
    W.PersonInfo.addHouseholdReadyDays(b, 1.5);
    W.PersonInfo.addStores(b, stores);
    W.PersonInfo.addSkills(b, skills);
    const body = W.PersonInfo.endPersonInfo(b);
    b.finish(W.Response.createResponse(b, W.ResponseBody.PersonInfo, body));
    const r = M.decodeResponse(b.asUint8Array());
    if (r.kind !== "person") throw new Error(r.kind);
    expect(r.person.stores).toEqual([
      { good: 1, kg: 812.25 },
      { good: 3, kg: 14.5 },
    ]);
    expect([r.person.carryGood, r.person.carryKg, r.person.carryFoodKcal]).toEqual([2, 12.5, 7500]);
    expect(r.person.householdFuelDays).toBe(2.5);
    expect(r.person.householdReadyDays).toBe(1.5);
    expect(r.person.skills).toEqual([
      { skill: 0, level: 0.75 },
      { skill: 4, level: 0.25 },
    ]);

    // Someone carrying nothing has no good: -1, not the first good.
    const e = new flatbuffers.Builder(64);
    W.PersonInfo.startPersonInfo(e);
    W.PersonInfo.addId(e, 13n);
    const empty = W.PersonInfo.endPersonInfo(e);
    e.finish(W.Response.createResponse(e, W.ResponseBody.PersonInfo, empty));
    const nobody = M.decodeResponse(e.asUint8Array());
    if (nobody.kind !== "person") throw new Error(nobody.kind);
    expect([nobody.person.carryGood, nobody.person.stores, nobody.person.skills]).toEqual([
      -1,
      [],
      [],
    ]);
    expect([nobody.person.partner, nobody.person.family]).toEqual([0, []]);
    expect([nobody.person.householdTaste, nobody.person.householdAdmired]).toEqual(["", 0]);
  });

  it("decode a person's partner and the kernel's sentences about their family", () => {
    const b = new flatbuffers.Builder(256);
    const lines = [
      b.createString("Partner of Bram for 12 years."),
      b.createString("Expecting a child, due in about 3 months."),
    ];
    const family = W.PersonInfo.createFamilyVector(b, lines);
    const taste = b.createString(
      "roofs pitched 48°, walls 1.9 m to the eaves, eaves 0.5 m out; admiring Bo's hut",
    );
    W.PersonInfo.startPersonInfo(b);
    W.PersonInfo.addId(b, 21n);
    W.PersonInfo.addPartner(b, 22n);
    W.PersonInfo.addFamily(b, family);
    W.PersonInfo.addHouseholdTaste(b, taste);
    W.PersonInfo.addHouseholdAdmired(b, 12n);
    const body = W.PersonInfo.endPersonInfo(b);
    b.finish(W.Response.createResponse(b, W.ResponseBody.PersonInfo, body));
    const r = M.decodeResponse(b.asUint8Array());
    if (r.kind !== "person") throw new Error(r.kind);
    expect(r.person.partner).toBe(22);
    // Their household's taste in building, and the building it admires (M3b slice R).
    expect(r.person.householdTaste).toBe(
      "roofs pitched 48°, walls 1.9 m to the eaves, eaves 0.5 m out; admiring Bo's hut",
    );
    expect(r.person.householdAdmired).toBe(12);
    expect(r.person.family).toEqual([
      "Partner of Bram for 12 years.",
      "Expecting a child, due in about 3 months.",
    ]);

    const w = new flatbuffers.Builder(256);
    const id = w.createString("core:good/meat");
    const goodName = w.createString("Meat");
    const purpose = w.createString("food");
    const eaten = w.createString("cooked");
    const good = W.GoodInfo.createGoodInfo(w, id, goodName, purpose, 1500, eaten, 0);
    const sickleId = w.createString("core:good/sickle");
    const sickleName = w.createString("Sickle");
    const tool = w.createString("tool");
    const never = w.createString("never");
    const sickle = W.GoodInfo.createGoodInfo(w, sickleId, sickleName, tool, 0, never, 100);
    const goods = W.Welcome.createGoodsVector(w, [good, sickle]);
    const skillId = w.createString("core:skill/milling");
    const skillName = w.createString("Milling");
    const skill = W.SkillInfo.createSkillInfo(w, skillId, skillName);
    const skills = W.Welcome.createSkillsVector(w, [skill]);
    const regimeId = w.createString("core:regime/village");
    const regimeName = w.createString("Village fields");
    const regimeText = w.createString("The village holds the land.");
    const rule = w.createString("The settlement holds the ground its households break.");
    const rules = W.RegimeInfo.createRulesVector(w, [rule]);
    const regime = W.RegimeInfo.createRegimeInfo(w, regimeId, regimeName, regimeText, false, rules);
    const regimes = W.Welcome.createRegimesVector(w, [regime]);
    W.Welcome.startWelcome(w);
    W.Welcome.addGoods(w, goods);
    W.Welcome.addSkills(w, skills);
    W.Welcome.addRegimes(w, regimes);
    w.finish(W.Welcome.endWelcome(w));
    const welcome = M.decodeWelcome(w.asUint8Array());
    expect(welcome.goods).toEqual([
      { id: "core:good/meat", name: "Meat", purpose: "food", kcalPerKg: 1500, eaten: "cooked", toolLifeH: 0 },
      { id: "core:good/sickle", name: "Sickle", purpose: "tool", kcalPerKg: 0, eaten: "never", toolLifeH: 100 },
    ]);
    expect(welcome.skills).toEqual([{ id: "core:skill/milling", name: "Milling" }]);
    expect(welcome.regimes).toEqual([
      {
        id: "core:regime/village",
        name: "Village fields",
        description: "The village holds the land.",
        isDefault: false,
        rules: ["The settlement holds the ground its households break."],
      },
    ]);
  });

  it("decode a chronicle with links", () => {
    const b = new flatbuffers.Builder(256);
    const t1 = b.createString("They made camp at ");
    const s1 = W.Span.createSpan(b, W.SpanKind.Text, t1, 0n);
    const t2 = b.createString("Alderford");
    const s2 = W.Span.createSpan(b, W.SpanKind.Settlement, t2, 3n);
    const spans = W.ChronicleEntry.createSpansVector(b, [s1, s2]);
    const entry = W.ChronicleEntry.createChronicleEntry(b, 2n, 85_320n, spans);
    const entries = W.Chronicle.createEntriesVector(b, [entry]);
    const body = W.Chronicle.createChronicle(b, entries, 2n);
    b.finish(W.Response.createResponse(b, W.ResponseBody.Chronicle, body));
    const r = M.decodeResponse(b.asUint8Array());
    expect(r).toEqual({
      kind: "chronicle",
      head: 2,
      entries: [
        {
          seq: 2,
          minute: 85_320,
          spans: [
            { kind: "text", text: "They made camp at ", id: 0 },
            { kind: "settlement", text: "Alderford", id: 3 },
          ],
        },
      ],
    });
  });
});
