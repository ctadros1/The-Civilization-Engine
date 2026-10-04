import * as flatbuffers from "flatbuffers";
import { describe, expect, it } from "vitest";

import * as M from "../src/net/messages.js";
import * as W from "../src/schema/generated/tce/wire.js";

const bb = (bytes: Uint8Array) => new flatbuffers.ByteBuffer(bytes);

describe("builders", () => {
  it("builds a NewWorld command the generated reader understands", () => {
    const bytes = M.newWorld({
      seed: 18446744073709551615n,
      presetId: "core:worldgen/river_valley",
      sizeCells: 1024,
      name: "Old River",
    });
    const command = W.Command.getRootAsCommand(bb(bytes));
    expect(command.bodyType()).toBe(W.CommandBody.NewWorld);
    const body = command.body(new W.NewWorld()) as W.NewWorld;
    expect(body.seed()).toBe(18446744073709551615n);
    expect(body.presetId()).toBe("core:worldgen/river_valley");
    expect(body.sizeCells()).toBe(1024);
    expect(body.name()).toBe("Old River");
    expect(body.regimeId()).toBe("");
    const village = W.Command.getRootAsCommand(
      bb(
        M.newWorld({
          seed: 1n,
          presetId: "core:worldgen/river_valley",
          sizeCells: 512,
          name: "",
          regimeId: "core:regime/village",
        }),
      ),
    );
    expect((village.body(new W.NewWorld()) as W.NewWorld).regimeId()).toBe("core:regime/village");
  });

  it("builds clock and raster requests", () => {
    const clock = W.Command.getRootAsCommand(bb(M.setClock(false, 288)));
    const set = clock.body(new W.SetClock()) as W.SetClock;
    expect([set.paused(), set.speed()]).toEqual([false, 288]);

    const query = W.Query.getRootAsQuery(
      bb(M.getRaster({ layer: M.RasterLayer.Water, level: 2, x0: 3, y0: 4, width: 5, height: 6 })),
    );
    expect(query.bodyType()).toBe(W.QueryBody.GetRaster);
    const raster = query.body(new W.GetRaster()) as W.GetRaster;
    expect([
      raster.layer(),
      raster.level(),
      raster.x0(),
      raster.y0(),
      raster.width(),
      raster.height(),
    ]).toEqual([W.RasterLayer.Water, 2, 3, 4, 5, 6]);
    expect(W.Query.getRootAsQuery(bb(M.listSaves())).bodyType()).toBe(W.QueryBody.ListSaves);
    expect(W.Command.getRootAsCommand(bb(M.cancelTask())).bodyType()).toBe(
      W.CommandBody.CancelTask,
    );
  });

  it("builds a family arrival with how many families come", () => {
    const one = W.Command.getRootAsCommand(bb(M.spawnFamily(120, 340)));
    expect(one.bodyType()).toBe(W.CommandBody.SpawnFamily);
    const single = one.body(new W.SpawnFamily()) as W.SpawnFamily;
    expect([single.at()?.x(), single.at()?.y(), single.families()]).toEqual([120, 340, 1]);
    const group = W.Command.getRootAsCommand(bb(M.spawnFamily(5, 6, 10)));
    expect((group.body(new W.SpawnFamily()) as W.SpawnFamily).families()).toBe(10);
  });

  it("builds the observer's introduction of a technique", () => {
    const command = W.Command.getRootAsCommand(bb(M.introduceTechnique(42, 3, true)));
    expect(command.bodyType()).toBe(W.CommandBody.IntroduceTechnique);
    const body = command.body(new W.IntroduceTechnique()) as W.IntroduceTechnique;
    expect([body.person(), body.technique(), body.awareOnly()]).toEqual([42n, 3, true]);
  });
});

function finish(b: flatbuffers.Builder, root: flatbuffers.Offset): Uint8Array {
  b.finish(root);
  return b.asUint8Array();
}

describe("decoders", () => {
  it("decodes a snapshot with a world, clock, task and recovery offer", () => {
    const b = new flatbuffers.Builder(256);
    const id = b.createString("ab".repeat(16));
    const name = b.createString("Test Valley");
    const preset = b.createString("core:worldgen/ria_coast");
    const regimeId = b.createString("core:regime/village");
    const regimeName = b.createString("Village fields");
    W.WorldInfo.startWorldInfo(b);
    W.WorldInfo.addWorldId(b, id);
    W.WorldInfo.addName(b, name);
    W.WorldInfo.addPresetId(b, preset);
    W.WorldInfo.addSeed(b, 42n);
    W.WorldInfo.addWidth(b, 512);
    W.WorldInfo.addHeight(b, 512);
    W.WorldInfo.addCellSizeM(b, 8);
    W.WorldInfo.addGeneration(b, 3n);
    W.WorldInfo.addContentChanged(b, true);
    W.WorldInfo.addRegimeId(b, regimeId);
    W.WorldInfo.addRegimeName(b, regimeName);
    const world = W.WorldInfo.endWorldInfo(b);
    const season = b.createString("spring");
    const clock = W.Clock.createClock(b, 85_320n, 1n, 3, 1, 6, 0, season, true, 96);
    const taskName = b.createString("Loading");
    const stage = b.createString("Reading");
    const task = W.Task.createTask(b, taskName, stage, 0.5, false);
    const reason = b.createString("The last session did not shut down cleanly.");
    const file = b.createString("w/g0000000002-auto.tcesave");
    const label = b.createString("Autosave");
    const worldName = b.createString("Test Valley");
    const recovery = W.Recovery.createRecovery(b, reason, file, label, 85_400n, worldName);
    const error = b.createString("Autosave failed: disk full");
    W.Snapshot.startSnapshot(b);
    W.Snapshot.addWorld(b, world);
    W.Snapshot.addClock(b, clock);
    W.Snapshot.addTask(b, task);
    W.Snapshot.addRecovery(b, recovery);
    W.Snapshot.addLastError(b, error);
    W.Snapshot.addLastAutosaveUnixMs(b, 1_700_000_000_000n);
    const root = W.Snapshot.endSnapshot(b);
    const snapshot = M.decodeSnapshot(finish(b, root));

    expect(snapshot.world?.name).toBe("Test Valley");
    expect(snapshot.world?.seed).toBe(42n);
    expect(snapshot.world?.generation).toBe(3);
    expect(snapshot.world?.contentChanged).toBe(true);
    expect([snapshot.world?.regimeId, snapshot.world?.regimeName]).toEqual([
      "core:regime/village",
      "Village fields",
    ]);
    expect(snapshot.clock).toMatchObject({
      minute: 85_320,
      year: 1,
      month: 3,
      paused: true,
      season: "spring",
    });
    expect(snapshot.task).toEqual({
      name: "Loading",
      stage: "Reading",
      fraction: 0.5,
      cancellable: false,
    });
    expect(snapshot.recovery?.saveFile).toBe("w/g0000000002-auto.tcesave");
    expect(snapshot.lastError).toBe("Autosave failed: disk full");
    expect(snapshot.lastAutosaveUnixMs).toBe(1_700_000_000_000);
  });

  it("decodes an empty snapshot", () => {
    const b = new flatbuffers.Builder(16);
    W.Snapshot.startSnapshot(b);
    const snapshot = M.decodeSnapshot(finish(b, W.Snapshot.endSnapshot(b)));
    expect(snapshot).toEqual({
      world: null,
      clock: null,
      task: null,
      recovery: null,
      lastError: null,
      lastAutosaveUnixMs: 0,
      people: [],
      settlements: [],
      chronicleHead: 0,
      fieldsRev: 0,
      buildingsRev: 0,
      pathsRev: 0,
      marketsRev: 0,
      firmsRev: 0,
      wealthRev: 0,
      knowledgeRev: 0,
    });
  });

  it("builds a paths query and decodes the worn ground and trails", () => {
    const query = W.Query.getRootAsQuery(bb(M.getPaths()));
    expect(query.bodyType()).toBe(W.QueryBody.GetPaths);

    const b = new flatbuffers.Builder(8192);
    const cells = new Uint8Array(64 * 64);
    cells[5] = 200;
    cells[64 + 1] = 40;
    const wear = W.WornTile.createWearVector(b, cells);
    // Cell 5 is trail (bit 5 of word 0), and cell 100 (bit 36 of word 1).
    const words = new Array<bigint>(64).fill(0n);
    words[0] = 1n << 5n;
    words[1] = 1n << 36n;
    const trail = W.WornTile.createTrailVector(b, words);
    const tile = W.WornTile.createWornTile(b, 9, wear, trail);
    const worn = W.Paths.createWornVector(b, [tile]);
    W.TrailInfo.startPointsVector(b, 2);
    W.Vec2.createVec2(b, 300, 40);
    W.Vec2.createVec2(b, 100, 40);
    const points = b.endVector();
    const info = W.TrailInfo.createTrailInfo(b, points, 0.75, 200);
    const trails = W.Paths.createTrailsVector(b, [info]);
    const paths = W.Paths.createPaths(b, 12n, 4, 64, worn, trails);
    const response = W.Response.createResponse(b, W.ResponseBody.Paths, paths);
    const body = M.decodeResponse(finish(b, response));
    expect(body.kind).toBe("paths");
    if (body.kind !== "paths") return;
    const p = body.paths;
    expect([p.rev, p.tilesX, p.tileCells]).toEqual([12, 4, 64]);
    expect(p.worn).toHaveLength(1);
    expect(p.worn[0]!.index).toBe(9);
    expect(p.worn[0]!.wear[5]).toBe(200);
    expect(p.worn[0]!.wear[65]).toBe(40);
    const trailCells = [...p.worn[0]!.trail.entries()].filter(([, v]) => v === 1).map(([k]) => k);
    expect(trailCells).toEqual([5, 100]);
    expect(p.trails).toEqual([{ points: [[100, 40], [300, 40]], wear: 0.75, lengthM: 200 }]);
  });

  it("builds a markets query and decodes a settlement's market", () => {
    const query = W.Query.getRootAsQuery(bb(M.getMarkets()));
    expect(query.bodyType()).toBe(W.QueryBody.GetMarkets);

    const b = new flatbuffers.Builder(1024);
    const sickle = W.MarketGood.createMarketGood(b, 4, 2, 2, 1.5, 3, 12, 0.6, 0, 11.5, 1);
    const goods = W.MarketInfo.createGoodsVector(b, [sickle]);
    const ada = b.createString("Ada's household");
    const offer = W.OfferInfo.createOfferInfo(b, 7n, ada, 4, 0, 12.25, 2, false);
    const offers = W.MarketInfo.createOffersVector(b, [offer]);
    const text = b.createString("Ada's household sold 1 sickle to Bran's household for 12 kg of grain.");
    const trade = W.TradeInfo.createTradeInfo(b, 1440n, 7n, 9n, 4, 1, 0, 12, false, text, false);
    const recent = W.MarketInfo.createRecentVector(b, [trade]);
    W.MarketInfo.startHistoryVector(b, 1);
    W.MonthOfTrade.createMonthOfTrade(b, 14, 2, 1.5, 18, 4);
    const history = b.endVector();
    const name = b.createString("Hearth");
    const summary = b.createString("Barter: no good settles most of what is paid; grain settles 60%.");
    const info = W.MarketInfo.createMarketInfo(
      b,
      3n,
      name,
      -1,
      summary,
      2.5,
      30,
      goods,
      offers,
      recent,
      history,
    );
    const list = W.Markets.createMarketsVector(b, [info]);
    const markets = W.Markets.createMarkets(b, 41n, list);
    const response = W.Response.createResponse(b, W.ResponseBody.Markets, markets);
    const body = M.decodeResponse(finish(b, response));
    expect(body.kind).toBe("markets");
    if (body.kind !== "markets") return;
    expect(body.rev).toBe(41);
    expect(body.markets).toHaveLength(1);
    const m = body.markets[0]!;
    expect([m.settlement, m.settlementName, m.money, m.trades, m.memoryDays]).toEqual([
      3,
      "Hearth",
      -1,
      2.5,
      30,
    ]);
    expect(m.summary).toContain("Barter");
    expect(m.goods).toEqual([
      {
        good: 4,
        offered: 2,
        sellers: 2,
        sold: 1.5,
        unmet: 3,
        unmetWorthH: 12,
        acceptance: expect.closeTo(0.6, 5),
        lastPayment: 0,
        lastPrice: 11.5,
        workshops: 1,
      },
    ]);
    expect(m.offers).toEqual([
      { household: 7, householdName: "Ada's household", good: 4, payment: 0, price: 12.25, units: 2, firm: false },
    ]);
    expect(m.recent[0]).toMatchObject({
      minute: 1440,
      seller: 7,
      buyer: 9,
      good: 4,
      sale: false,
      sellerFirm: false,
    });
    expect(m.recent[0]!.text).toContain("sold 1 sickle");
    expect(m.history).toEqual([{ month: 14, good: 4, trades: 2, units: 1.5, paidH: 18 }]);
  });

  it("builds workshop queries and decodes the workshops and a workshop's page", () => {
    expect(W.Query.getRootAsQuery(bb(M.getFirms())).bodyType()).toBe(W.QueryBody.GetFirms);
    const ask = W.Query.getRootAsQuery(bb(M.getFirm(77)));
    expect(ask.bodyType()).toBe(W.QueryBody.GetFirm);
    expect(Number((ask.body(new W.GetFirm()) as W.GetFirm).id())).toBe(77);

    const brief = (b: flatbuffers.Builder) => {
      const name = b.createString("Wren's sickle workshop");
      const owner = b.createString("Wren's household");
      const place = b.createString("Hearth");
      const why = b.createString("it sold nothing for months");
      const lines = W.FirmBrief.createLinesVector(b, [4]);
      const record = b.createString("Made 3.0 sickles and sold a sickle.");
      return W.FirmBrief.createFirmBrief(b, 77n, name, 7n, owner, 3n, place, 1440n, false, 90_000n, why, lines, 0, record);
    };

    let b = new flatbuffers.Builder(1024);
    const list = W.Firms.createFirmsVector(b, [brief(b)]);
    let response = W.Response.createResponse(b, W.ResponseBody.Firms, W.Firms.createFirms(b, 5n, list));
    const firms = M.decodeResponse(finish(b, response));
    expect(firms.kind).toBe("firms");
    if (firms.kind !== "firms") return;
    expect(firms.rev).toBe(5);
    expect(firms.firms).toEqual([
      {
        id: 77,
        name: "Wren's sickle workshop",
        owner: 7,
        ownerName: "Wren's household",
        settlement: 3,
        settlementName: "Hearth",
        foundedMinute: 1440,
        open: false,
        closedMinute: 90_000,
        closedWhy: "it sold nothing for months",
        lines: [4],
        hiringH: 0,
        record: "Made 3.0 sickles and sold a sickle.",
      },
    ]);

    b = new flatbuffers.Builder(2048);
    const head = brief(b);
    const founder = b.createString("Wren");
    W.FirmInfo.startStoresVector(b, 1);
    W.StoreLine.createStoreLine(b, 4, 2);
    const stores = b.endVector();
    const seller = b.createString("Wren's sickle workshop");
    const offer = W.OfferInfo.createOfferInfo(b, 77n, seller, 4, 0, 12, 2, true);
    const offers = W.FirmInfo.createOffersVector(b, [offer]);
    const wageText = b.createString("Pays 0.90 kg of grain an hour; wants 6 more hours of work before its next review.");
    const wage = W.WageInfo.createWageInfo(b, 12, 0, 0.9, 0.5, 8, 2, wageText);
    const sold = b.createString("Sold a sickle to Bran's household.");
    const entry = W.BookEntryInfo.createBookEntryInfo(b, 2000n, W.BookKind.Sold, 4, 1, 9n, sold);
    const entries = W.FirmInfo.createEntriesVector(b, [entry]);
    W.MonthStatement.startLinesVector(b, 2);
    W.BookLine.createBookLine(b, 12, 0, W.BookKind.Paid);
    W.BookLine.createBookLine(b, 1, 4, W.BookKind.Sold);
    const lines = b.endVector();
    const statement = W.MonthStatement.createMonthStatement(b, 14, lines, 3, 1.5, 7.5, 2.5, 6);
    const months = W.FirmInfo.createMonthsVector(b, [statement]);
    W.FirmInfo.startFirmInfo(b);
    W.FirmInfo.addBrief(b, head);
    W.FirmInfo.addFounder(b, 21n);
    W.FirmInfo.addFounderName(b, founder);
    W.FirmInfo.addOwnerSinceMinute(b, 1440n);
    W.FirmInfo.addLastSaleMinute(b, 2000n);
    W.FirmInfo.addStores(b, stores);
    W.FirmInfo.addOffers(b, offers);
    W.FirmInfo.addWage(b, wage);
    W.FirmInfo.addEntries(b, entries);
    W.FirmInfo.addMonths(b, months);
    response = W.Response.createResponse(b, W.ResponseBody.FirmInfo, W.FirmInfo.endFirmInfo(b));
    const page = M.decodeResponse(finish(b, response));
    expect(page.kind).toBe("firm");
    if (page.kind !== "firm") return;
    const f = page.firm;
    expect(f.brief.name).toBe("Wren's sickle workshop");
    expect([f.founder, f.founderName, f.ownerSinceMinute, f.lastSaleMinute]).toEqual([21, "Wren", 1440, 2000]);
    expect(f.stores).toEqual([{ good: 4, kg: 2 }]);
    expect(f.offers).toEqual([
      { household: 77, householdName: "Wren's sickle workshop", good: 4, payment: 0, price: 12, units: 2, firm: true },
    ]);
    expect(f.wage).toMatchObject({ activity: 12, pay: 0, hours: 8, taken: 2 });
    expect(f.wage?.perHour).toBeCloseTo(0.9, 5);
    expect(f.wage?.text).toContain("Pays 0.90 kg of grain");
    expect(f.entries).toEqual([
      { minute: 2000, kind: "sold", good: 4, amount: 1, other: 9, text: "Sold a sickle to Bran's household." },
    ]);
    expect(f.months).toEqual([
      {
        month: 14,
        lines: [
          { kind: "sold", good: 4, amount: 1 },
          { kind: "paid", good: 0, amount: 12 },
        ],
        ownerH: 3,
        hiredH: 1.5,
        incomeH: 7.5,
        costsH: 2.5,
        stockH: 6,
      },
    ]);

    // A workshop that never sold says so; one with no wage has none.
    b = new flatbuffers.Builder(512);
    const bare = brief(b);
    W.FirmInfo.startFirmInfo(b);
    W.FirmInfo.addBrief(b, bare);
    response = W.Response.createResponse(b, W.ResponseBody.FirmInfo, W.FirmInfo.endFirmInfo(b));
    const fresh = M.decodeResponse(finish(b, response));
    if (fresh.kind !== "firm") throw new Error("expected a workshop");
    expect([fresh.firm.lastSaleMinute, fresh.firm.wage, fresh.firm.entries]).toEqual([-1, null, []]);
  });

  it("decodes chronicle links to people, settlements and workshops", () => {
    const b = new flatbuffers.Builder(512);
    const span = (kind: W.SpanKind, text: string, id: bigint) =>
      W.Span.createSpan(b, kind, b.createString(text), id);
    const spans = W.ChronicleEntry.createSpansVector(b, [
      span(W.SpanKind.Person, "Wren", 21n),
      span(W.SpanKind.Text, " set up ", 0n),
      span(W.SpanKind.Firm, "Wren's sickle workshop", 77n),
    ]);
    const entry = W.ChronicleEntry.createChronicleEntry(b, 4n, 1440n, spans);
    const entries = W.Chronicle.createEntriesVector(b, [entry]);
    const response = W.Response.createResponse(
      b,
      W.ResponseBody.Chronicle,
      W.Chronicle.createChronicle(b, entries, 4n),
    );
    const body = M.decodeResponse(finish(b, response));
    if (body.kind !== "chronicle") throw new Error("expected the chronicle");
    expect(body.entries[0]!.spans.map((x) => [x.kind, x.id])).toEqual([
      ["person", 21],
      ["text", 0],
      ["firm", 77],
    ]);
  });

  it("builds a buildings query and decodes the buildings", () => {
    const query = W.Query.getRootAsQuery(bb(M.getBuildings()));
    expect(query.bodyType()).toBe(W.QueryBody.GetBuildings);

    const b = new flatbuffers.Builder(512);
    const program = b.createString("Hut");
    const stageName = b.createString("walls");
    const status = b.createString("walls going up, 40% done");
    // Struct vectors are written back to front.
    const firmName = b.createString("Wren's sickle workshop");
    W.BuildingInfo.startOutlineVector(b, 2);
    W.Vec2.createVec2(b, 100, 203);
    W.Vec2.createVec2(b, 103, 200);
    const outline = b.endVector();
    W.BuildingInfo.startPostsVector(b, 1);
    W.Vec2.createVec2(b, 103.1, 200);
    const posts = b.endVector();
    W.BuildingInfo.startBuildingInfo(b);
    W.BuildingInfo.addId(b, 61n);
    W.BuildingInfo.addHousehold(b, 7n);
    W.BuildingInfo.addSettlement(b, 3n);
    W.BuildingInfo.addProgram(b, program);
    W.BuildingInfo.addCentre(b, W.Vec2.createVec2(b, 100, 200));
    W.BuildingInfo.addRadiusM(b, 3);
    W.BuildingInfo.addRoofRadiusM(b, 3.5);
    W.BuildingInfo.addDoor(b, W.Vec2.createVec2(b, 103, 200));
    W.BuildingInfo.addDoorDir(b, 0.5);
    W.BuildingInfo.addStage(b, 2);
    W.BuildingInfo.addStageName(b, stageName);
    W.BuildingInfo.addProgress(b, 0.4);
    W.BuildingInfo.addOutline(b, outline);
    W.BuildingInfo.addPosts(b, posts);
    W.BuildingInfo.addPlotMin(b, W.Vec2.createVec2(b, 96.5, 196.5));
    W.BuildingInfo.addPlotSize(b, W.Vec2.createVec2(b, 7, 7));
    W.BuildingInfo.addFloorM2(b, 28.25);
    W.BuildingInfo.addSleeps(b, 4);
    W.BuildingInfo.addStartedMinute(b, 86_400n);
    W.BuildingInfo.addStatus(b, status);
    W.BuildingInfo.addFirm(b, 41n);
    W.BuildingInfo.addFirmName(b, firmName);
    const info = W.BuildingInfo.endBuildingInfo(b);
    const list = W.Buildings.createBuildingsVector(b, [info]);
    const buildings = W.Buildings.createBuildings(b, 77n, list);
    const response = W.Response.createResponse(b, W.ResponseBody.Buildings, buildings);
    const body = M.decodeResponse(finish(b, response));
    expect(body).toEqual({
      kind: "buildings",
      rev: 77,
      buildings: [
        {
          id: 61,
          household: 7,
          settlement: 3,
          program: "Hut",
          x: 100,
          y: 200,
          radiusM: 3,
          roofRadiusM: 3.5,
          door: [103, 200],
          doorDir: 0.5,
          stage: 2,
          stageName: "walls",
          progress: expect.closeTo(0.4, 6),
          roofed: false,
          outline: [
            [103, 200],
            [100, 203],
          ],
          posts: [[expect.closeTo(103.1, 4), 200]],
          plot: { x: 96.5, y: 196.5, w: 7, h: 7 },
          floorM2: 28.25,
          sleeps: 4,
          startedMinute: 86_400,
          status: "walls going up, 40% done",
          grammar: "",
          purpose: "",
          size: [0, 0],
          angle: 0,
          storeys: 0,
          bays: 0,
          loftBays: 0,
          roofOutline: [],
          ridge: [],
          apexM: 0,
          floorByUse: [],
          storageKg: [],
          workPlaces: 0,
          storedKg: [],
          stored: "",
          firm: 41,
          firmName: "Wren's sickle workshop",
        },
      ],
    });
  });

  it("builds a fields query and decodes the fields", () => {
    const query = W.Query.getRootAsQuery(bb(M.getFields()));
    expect(query.bodyType()).toBe(W.QueryBody.GetFields);

    const b = new flatbuffers.Builder(256);
    const status = b.createString("growing; ripe in about 20 days");
    W.FieldInfo.startFieldInfo(b);
    W.FieldInfo.addId(b, 41n);
    W.FieldInfo.addHousehold(b, 7n);
    W.FieldInfo.addSettlement(b, 3n);
    W.FieldInfo.addMin(b, W.Vec2.createVec2(b, 100, 250));
    W.FieldInfo.addSize(b, W.Vec2.createVec2(b, 50, 50));
    W.FieldInfo.addStage(b, W.FieldStage.Sown);
    W.FieldInfo.addStageSinceMinute(b, 129_600n);
    W.FieldInfo.addProgress(b, 0.5);
    W.FieldInfo.addExpectedKg(b, 220);
    W.FieldInfo.addHarvests(b, 1);
    W.FieldInfo.addStatus(b, status);
    W.FieldInfo.addHolder(b, 8n);
    W.FieldInfo.addLeaseUntilMinute(b, 700_000n);
    W.FieldInfo.addLeaseShare(b, 0.25);
    const field = W.FieldInfo.endFieldInfo(b);
    const list = W.Fields.createFieldsVector(b, [field]);
    const fields = W.Fields.createFields(b, 99n, list);
    const response = W.Response.createResponse(b, W.ResponseBody.Fields, fields);
    const body = M.decodeResponse(finish(b, response));
    expect(body).toEqual({
      kind: "fields",
      rev: 99,
      fields: [
        {
          id: 41,
          household: 7,
          settlement: 3,
          x: 100,
          y: 250,
          w: 50,
          h: 50,
          crop: 0,
          stage: "sown",
          stageSinceMinute: 129_600,
          progress: 0.5,
          newGround: false,
          woodland: false,
          ripe: false,
          expectedKg: 220,
          sheavesKg: 0,
          harvests: 1,
          status: "growing; ripe in about 20 days",
          holder: 8,
          holderSettlement: 0,
          leaseUntilMinute: 700_000,
          leaseShare: 0.25,
        },
      ],
    });
  });

  it("builds the wealth query and decodes each settlement's measures", () => {
    expect(W.Query.getRootAsQuery(bb(M.getWealth())).bodyType()).toBe(W.QueryBody.GetWealth);
    const b = new flatbuffers.Builder(512);
    const spread = (year: bigint, gini: number) => {
      W.WealthSpread.startWealthSpread(b);
      W.WealthSpread.addYear(b, year);
      W.WealthSpread.addHouseholds(b, 2);
      W.WealthSpread.addPeople(b, 7);
      W.WealthSpread.addGiniGoods(b, gini);
      W.WealthSpread.addTopTenthGoods(b, 0.25);
      W.WealthSpread.addHoldingNone(b, 1);
      W.WealthSpread.addCommonHa(b, 1.5);
      W.WealthSpread.addRoofedM2PerHouse(b, 41.5);
      W.WealthSpread.addStorageKgPerHouse(b, 4150);
      return W.WealthSpread.endWealthSpread(b);
    };
    const now = spread(3n, 0.5);
    const name = b.createString("Wren's household");
    const household = W.HouseholdWealth.createHouseholdWealth(b, 9n, name, 4, 0, 0.75, 0, 0.25, 120, 24, 39, 6400);
    const households = W.SettlementWealth.createHouseholdsVector(b, [household]);
    const history = W.SettlementWealth.createHistoryVector(b, [spread(1n, 0.25), spread(2n, 0.5)]);
    const place = b.createString("Alderford");
    W.SettlementWealth.startSettlementWealth(b);
    W.SettlementWealth.addSettlement(b, 3n);
    W.SettlementWealth.addName(b, place);
    W.SettlementWealth.addNow(b, now);
    W.SettlementWealth.addHouseholds(b, households);
    W.SettlementWealth.addHistory(b, history);
    const settlement = W.SettlementWealth.endSettlementWealth(b);
    const regime = b.createString("Village fields");
    const list = W.Wealth.createSettlementsVector(b, [settlement]);
    const wealth = W.Wealth.createWealth(b, 5n, regime, list);
    const body = M.decodeResponse(finish(b, W.Response.createResponse(b, W.ResponseBody.Wealth, wealth)));
    expect(body.kind).toBe("wealth");
    if (body.kind !== "wealth") return;
    expect(body.wealth.rev).toBe(5);
    expect(body.wealth.regimeName).toBe("Village fields");
    const s = body.wealth.settlements[0]!;
    expect([s.settlement, s.name]).toEqual([3, "Alderford"]);
    expect(s.now).toMatchObject({ year: 3, households: 2, people: 7, giniGoods: 0.5, holdingNone: 1 });
    expect(s.now?.commonHa).toBe(1.5);
    expect([s.now?.roofedM2PerHouse, s.now?.storageKgPerHouse]).toEqual([41.5, 4150]);
    expect(s.history.map((y) => [y.year, y.giniGoods])).toEqual([
      [1, 0.25],
      [2, 0.5],
    ]);
    expect(s.households).toEqual([
      {
        household: 9,
        name: "Wren's household",
        members: 4,
        heldHa: 0,
        workedHa: 0.75,
        letHa: 0,
        rentedHa: 0.25,
        goodsH: 120,
        floorM2: 24,
        roofedM2: 39,
        storageKg: 6400,
      },
    ]);
  });

  it("builds the knowledge query and decodes what each settlement knows", () => {
    expect(W.Query.getRootAsQuery(bb(M.getKnowledge())).bodyType()).toBe(W.QueryBody.GetKnowledge);
    const b = new flatbuffers.Builder(512);
    const ref = (id: bigint, name: string, age: number) =>
      W.PersonRef.createPersonRef(b, id, b.createString(name), age);
    const knowers = W.TechniqueHere.createKnowersVector(b, [ref(7n, "Wren", 61.5)]);
    const learners = W.TechniqueHere.createLearnersVector(b, [ref(9n, "Ash", 15.2)]);
    const heard = W.TechniqueHere.createHeardVector(b, []);
    const status = b.createString("known by one, Wren, aged 61; one learning");
    const history = W.TechniqueHere.createHistoryVector(b, [b.createString("Year 1: brought by Wren.")]);
    const here = W.TechniqueHere.createTechniqueHere(b, 2, true, knowers, learners, heard, 1, status, history);
    const techniques = W.SettlementKnowledge.createTechniquesVector(b, [here]);
    const settlement = W.SettlementKnowledge.createSettlementKnowledge(
      b,
      3n,
      b.createString("Alderford"),
      techniques,
    );
    const list = W.Knowledge.createSettlementsVector(b, [settlement]);
    const knowledge = W.Knowledge.createKnowledge(b, 11n, list);
    const body = M.decodeResponse(
      finish(b, W.Response.createResponse(b, W.ResponseBody.Knowledge, knowledge)),
    );
    expect(body.kind).toBe("knowledge");
    if (body.kind !== "knowledge") return;
    expect(body.knowledge.rev).toBe(11);
    const s = body.knowledge.settlements[0]!;
    expect([s.settlement, s.name]).toEqual([3, "Alderford"]);
    const t = s.techniques[0]!;
    expect(t).toMatchObject({
      technique: 2,
      known: true,
      practisedLastYear: 1,
      status: "known by one, Wren, aged 61; one learning",
      history: ["Year 1: brought by Wren."],
      heard: [],
    });
    expect(t.knowers).toEqual([{ id: 7, name: "Wren", ageYears: 61.5 }]);
    expect(t.learners.map((p) => p.name)).toEqual(["Ash"]);
  });

  it("decodes a raster tile response", () => {
    const b = new flatbuffers.Builder(64);
    const data = W.RasterTile.createDataVector(b, new Uint8Array([1, 0, 2, 0]));
    const tile = W.RasterTile.createRasterTile(
      b,
      W.RasterLayer.Elevation,
      1,
      0,
      0,
      2,
      1,
      W.RasterFormat.U16,
      0.5,
      100,
      data,
      64,
      64,
    );
    const root = W.Response.createResponse(b, W.ResponseBody.RasterTile, tile);
    const body = M.decodeResponse(finish(b, root));
    expect(body.kind).toBe("raster");
    if (body.kind !== "raster") return;
    expect(body.tile).toMatchObject({
      width: 2,
      height: 1,
      format: M.RasterFormat.U16,
      scale: 0.5,
      offset: 100,
    });
    expect(Array.from(body.tile.data)).toEqual([1, 0, 2, 0]);
  });

  it("decodes errors and events", () => {
    const b = new flatbuffers.Builder(64);
    const message = b.createString("there is no world yet");
    const error = M.decodeError(
      finish(b, W.ErrorInfo.createErrorInfo(b, W.ErrorCode.NoWorld, message)),
    );
    expect(error).toEqual({ code: "no-world", message: "there is no world yet" });

    const e = new flatbuffers.Builder(64);
    const text = e.createString("Saved");
    const event = W.Event.createEvent(e, 7n, 85_320n, 1n, W.EventKind.Saved, text);
    const list = W.Events.createEventsVector(e, [event]);
    const events = M.decodeEvents(finish(e, W.Events.createEvents(e, list)));
    expect(events).toEqual([{ id: 7, simMinute: 85_320, unixMs: 1, kind: "saved", text: "Saved" }]);
  });
});
