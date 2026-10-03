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
    });
  });

  it("builds a buildings query and decodes the buildings", () => {
    const query = W.Query.getRootAsQuery(bb(M.getBuildings()));
    expect(query.bodyType()).toBe(W.QueryBody.GetBuildings);

    const b = new flatbuffers.Builder(512);
    const program = b.createString("Hut");
    const stageName = b.createString("walls");
    const status = b.createString("walls going up, 40% done");
    // Struct vectors are written back to front.
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
        },
      ],
    });
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
