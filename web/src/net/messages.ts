// Payloads of the observer protocol (ADR-0001), as plain TypeScript values. Builders produce the
// FlatBuffers bytes of commands and queries; decoders turn the host's payloads into objects the UI
// can keep. Generated code (src/schema/generated) never leaks past this module.

import * as flatbuffers from "flatbuffers";
import * as W from "../schema/generated/tce/wire.js";

export interface PresetInfo {
  id: string;
  name: string;
  description: string;
  isDefault: boolean;
}

export interface Welcome {
  host: string;
  version: string;
  presets: PresetInfo[];
  mapSizes: number[];
  cellSizeM: number;
  contentFingerprint: string;
  defaultMapSize: number;
  speed1x: number;
  speedMultipliers: number[];
}

export interface WorldInfo {
  worldId: string;
  name: string;
  seed: bigint;
  presetId: string;
  width: number;
  height: number;
  cellSizeM: number;
  seaLevelM: number;
  minElevationM: number;
  maxElevationM: number;
  generatorVersion: number;
  lakes: number;
  reaches: number;
  riverLengthKm: number;
  maxDischargeM3s: number;
  gentleLandFraction: number;
  landFraction: number;
  oceanFraction: number;
  generation: number;
  contentChanged: boolean;
  createdUnixMs: number;
}

export interface Clock {
  minute: number;
  year: number;
  month: number;
  day: number;
  hour: number;
  minuteOfHour: number;
  season: string;
  paused: boolean;
  speed: number;
}

export interface Task {
  name: string;
  stage: string;
  fraction: number;
  cancellable: boolean;
}

export interface Recovery {
  reason: string;
  saveFile: string;
  saveLabel: string;
  simMinute: number;
  worldName: string;
}

export interface Snapshot {
  world: WorldInfo | null;
  clock: Clock | null;
  task: Task | null;
  recovery: Recovery | null;
  lastError: string | null;
  lastAutosaveUnixMs: number;
}

export type EventKind =
  | "info"
  | "world-created"
  | "saved"
  | "loaded"
  | "autosaved"
  | "warning"
  | "failure";

export interface EventItem {
  id: number;
  simMinute: number;
  unixMs: number;
  kind: EventKind;
  text: string;
}

export enum RasterLayer {
  Elevation = W.RasterLayer.Elevation,
  Water = W.RasterLayer.Water,
  DrainageArea = W.RasterLayer.DrainageArea,
  LakeId = W.RasterLayer.LakeId,
}

export enum RasterFormat {
  U8 = W.RasterFormat.U8,
  U16 = W.RasterFormat.U16,
  U32 = W.RasterFormat.U32,
  F32 = W.RasterFormat.F32,
}

export interface RasterQuery {
  layer: RasterLayer;
  level: number;
  x0: number;
  y0: number;
  width: number;
  height: number;
}

export interface RasterTile extends RasterQuery {
  format: RasterFormat;
  scale: number;
  offset: number;
  /** Little-endian samples, row-major (a copy, safe to keep). */
  data: Uint8Array;
  fullWidth: number;
  fullHeight: number;
}

export interface Reach {
  id: number;
  order: number;
  dischargeM3s: number;
  widthM: number;
  downstream: number;
  /** x, y pairs in metres from the north-west corner, downstream order. */
  points: Float32Array;
}

export interface LakeInfo {
  id: number;
  levelM: number;
  areaM2: number;
  closed: boolean;
  centroidX: number;
  centroidY: number;
}

export interface Hydrography {
  reaches: Reach[];
  lakes: LakeInfo[];
}

export interface SaveEntry {
  file: string;
  worldName: string;
  worldId: string;
  label: string;
  kind: "manual" | "auto" | "crash" | string;
  generation: number;
  createdUnixMs: number;
  simMinute: number;
  sizeBytes: number;
  compatible: boolean;
  contentChanged: boolean;
  note: string;
}

export type ResponseBody =
  | { kind: "ack"; message: string }
  | { kind: "raster"; tile: RasterTile }
  | { kind: "hydrography"; hydrography: Hydrography }
  | { kind: "saves"; saves: SaveEntry[] };

export type ErrorCode =
  | "unknown"
  | "bad-request"
  | "busy"
  | "no-world"
  | "not-found"
  | "incompatible"
  | "internal";

export interface ErrorInfo {
  code: ErrorCode;
  message: string;
}

// ---- Builders ---------------------------------------------------------------------------------

function finish(builder: flatbuffers.Builder, root: flatbuffers.Offset): Uint8Array {
  builder.finish(root);
  return builder.asUint8Array().slice();
}

export function hello(client: string): Uint8Array {
  const b = new flatbuffers.Builder(64);
  const name = b.createString(client);
  return finish(b, W.Hello.createHello(b, name));
}

function command(
  b: flatbuffers.Builder,
  type: W.CommandBody,
  body: flatbuffers.Offset,
): Uint8Array {
  return finish(b, W.Command.createCommand(b, type, body));
}

function query(b: flatbuffers.Builder, type: W.QueryBody, body: flatbuffers.Offset): Uint8Array {
  return finish(b, W.Query.createQuery(b, type, body));
}

export function newWorld(args: {
  seed: bigint;
  presetId: string;
  sizeCells: number;
  name: string;
}): Uint8Array {
  const b = new flatbuffers.Builder(128);
  const preset = b.createString(args.presetId);
  const name = b.createString(args.name);
  const body = W.NewWorld.createNewWorld(b, args.seed, preset, args.sizeCells, name);
  return command(b, W.CommandBody.NewWorld, body);
}

export function saveWorld(label: string): Uint8Array {
  const b = new flatbuffers.Builder(64);
  const text = b.createString(label);
  return command(b, W.CommandBody.SaveWorld, W.SaveWorld.createSaveWorld(b, text));
}

export function loadWorld(file: string): Uint8Array {
  const b = new flatbuffers.Builder(128);
  const text = b.createString(file);
  return command(b, W.CommandBody.LoadWorld, W.LoadWorld.createLoadWorld(b, text));
}

export function setClock(paused: boolean, speed: number): Uint8Array {
  const b = new flatbuffers.Builder(32);
  return command(b, W.CommandBody.SetClock, W.SetClock.createSetClock(b, paused, speed));
}

export function cancelTask(): Uint8Array {
  const b = new flatbuffers.Builder(16);
  W.CancelTask.startCancelTask(b);
  return command(b, W.CommandBody.CancelTask, W.CancelTask.endCancelTask(b));
}

export function recoverWorld(accept: boolean): Uint8Array {
  const b = new flatbuffers.Builder(16);
  return command(b, W.CommandBody.RecoverWorld, W.RecoverWorld.createRecoverWorld(b, accept));
}

export function getRaster(q: RasterQuery): Uint8Array {
  const b = new flatbuffers.Builder(64);
  const body = W.GetRaster.createGetRaster(
    b,
    q.layer as number as W.RasterLayer,
    q.level,
    q.x0,
    q.y0,
    q.width,
    q.height,
  );
  return query(b, W.QueryBody.GetRaster, body);
}

export function getHydrography(toleranceM: number): Uint8Array {
  const b = new flatbuffers.Builder(32);
  return query(
    b,
    W.QueryBody.GetHydrography,
    W.GetHydrography.createGetHydrography(b, toleranceM),
  );
}

export function listSaves(): Uint8Array {
  const b = new flatbuffers.Builder(16);
  W.ListSaves.startListSaves(b);
  return query(b, W.QueryBody.ListSaves, W.ListSaves.endListSaves(b));
}

// ---- Decoders ---------------------------------------------------------------------------------

function bb(payload: Uint8Array): flatbuffers.ByteBuffer {
  return new flatbuffers.ByteBuffer(payload);
}

export function decodeWelcome(payload: Uint8Array): Welcome {
  const w = W.Welcome.getRootAsWelcome(bb(payload));
  const presets: PresetInfo[] = [];
  for (let i = 0; i < w.presetsLength(); i++) {
    const p = w.presets(i);
    if (!p) continue;
    presets.push({
      id: p.id() ?? "",
      name: p.name() ?? "",
      description: p.description() ?? "",
      isDefault: p.isDefault(),
    });
  }
  return {
    host: w.host() ?? "",
    version: w.version() ?? "",
    presets,
    mapSizes: Array.from(w.mapSizesArray() ?? []),
    cellSizeM: w.cellSizeM(),
    contentFingerprint: w.contentFingerprint() ?? "",
    defaultMapSize: w.defaultMapSize(),
    speed1x: w.speed1x(),
    speedMultipliers: Array.from(w.speedMultipliersArray() ?? []),
  };
}

function worldInfo(w: W.WorldInfo): WorldInfo {
  return {
    worldId: w.worldId() ?? "",
    name: w.name() ?? "",
    seed: w.seed(),
    presetId: w.presetId() ?? "",
    width: w.width(),
    height: w.height(),
    cellSizeM: w.cellSizeM(),
    seaLevelM: w.seaLevelM(),
    minElevationM: w.minElevationM(),
    maxElevationM: w.maxElevationM(),
    generatorVersion: w.generatorVersion(),
    lakes: w.lakes(),
    reaches: w.reaches(),
    riverLengthKm: w.riverLengthKm(),
    maxDischargeM3s: w.maxDischargeM3s(),
    gentleLandFraction: w.gentleLandFraction(),
    landFraction: w.landFraction(),
    oceanFraction: w.oceanFraction(),
    generation: Number(w.generation()),
    contentChanged: w.contentChanged(),
    createdUnixMs: Number(w.createdUnixMs()),
  };
}

export function decodeSnapshot(payload: Uint8Array): Snapshot {
  const s = W.Snapshot.getRootAsSnapshot(bb(payload));
  const world = s.world();
  const clock = s.clock();
  const task = s.task();
  const recovery = s.recovery();
  return {
    world: world ? worldInfo(world) : null,
    clock: clock
      ? {
          minute: Number(clock.minute()),
          year: Number(clock.year()),
          month: clock.month(),
          day: clock.day(),
          hour: clock.hour(),
          minuteOfHour: clock.minuteOfHour(),
          season: clock.season() ?? "",
          paused: clock.paused(),
          speed: clock.speed(),
        }
      : null,
    task: task
      ? {
          name: task.name() ?? "",
          stage: task.stage() ?? "",
          fraction: task.fraction(),
          cancellable: task.cancellable(),
        }
      : null,
    recovery: recovery
      ? {
          reason: recovery.reason() ?? "",
          saveFile: recovery.saveFile() ?? "",
          saveLabel: recovery.saveLabel() ?? "",
          simMinute: Number(recovery.simMinute()),
          worldName: recovery.worldName() ?? "",
        }
      : null,
    lastError: s.lastError() || null,
    lastAutosaveUnixMs: Number(s.lastAutosaveUnixMs()),
  };
}

const EVENT_KINDS: Record<number, EventKind> = {
  [W.EventKind.Info]: "info",
  [W.EventKind.WorldCreated]: "world-created",
  [W.EventKind.Saved]: "saved",
  [W.EventKind.Loaded]: "loaded",
  [W.EventKind.Autosaved]: "autosaved",
  [W.EventKind.Warning]: "warning",
  [W.EventKind.Failure]: "failure",
};

export function decodeEvents(payload: Uint8Array): EventItem[] {
  const e = W.Events.getRootAsEvents(bb(payload));
  const out: EventItem[] = [];
  for (let i = 0; i < e.eventsLength(); i++) {
    const item = e.events(i);
    if (!item) continue;
    out.push({
      id: Number(item.id()),
      simMinute: Number(item.simMinute()),
      unixMs: Number(item.unixMs()),
      kind: EVENT_KINDS[item.kind()] ?? "info",
      text: item.text() ?? "",
    });
  }
  return out;
}

const ERROR_CODES: Record<number, ErrorCode> = {
  [W.ErrorCode.Unknown]: "unknown",
  [W.ErrorCode.BadRequest]: "bad-request",
  [W.ErrorCode.Busy]: "busy",
  [W.ErrorCode.NoWorld]: "no-world",
  [W.ErrorCode.NotFound]: "not-found",
  [W.ErrorCode.Incompatible]: "incompatible",
  [W.ErrorCode.Internal]: "internal",
};

export function decodeError(payload: Uint8Array): ErrorInfo {
  const e = W.ErrorInfo.getRootAsErrorInfo(bb(payload));
  return { code: ERROR_CODES[e.code()] ?? "unknown", message: e.message() ?? "" };
}

function rasterTile(t: W.RasterTile): RasterTile {
  return {
    layer: t.layer() as number as RasterLayer,
    level: t.level(),
    x0: t.x0(),
    y0: t.y0(),
    width: t.width(),
    height: t.height(),
    format: t.format() as number as RasterFormat,
    scale: t.scale(),
    offset: t.offset(),
    data: (t.dataArray() ?? new Uint8Array()).slice(),
    fullWidth: t.fullWidth(),
    fullHeight: t.fullHeight(),
  };
}

function hydrography(h: W.Hydrography): Hydrography {
  const reaches: Reach[] = [];
  for (let i = 0; i < h.reachesLength(); i++) {
    const r = h.reaches(i);
    if (!r) continue;
    const n = r.pointsLength();
    const points = new Float32Array(n * 2);
    const v = new W.Vec2();
    for (let k = 0; k < n; k++) {
      const p = r.points(k, v);
      if (!p) continue;
      points[2 * k] = p.x();
      points[2 * k + 1] = p.y();
    }
    reaches.push({
      id: r.id(),
      order: r.order(),
      dischargeM3s: r.dischargeM3s(),
      widthM: r.widthM(),
      downstream: r.downstream(),
      points,
    });
  }
  const lakes: LakeInfo[] = [];
  for (let i = 0; i < h.lakesLength(); i++) {
    const l = h.lakes(i);
    if (!l) continue;
    const c = l.centroid();
    lakes.push({
      id: l.id(),
      levelM: l.levelM(),
      areaM2: l.areaM2(),
      closed: l.closed(),
      centroidX: c?.x() ?? 0,
      centroidY: c?.y() ?? 0,
    });
  }
  return { reaches, lakes };
}

function saveEntries(list: W.SaveList): SaveEntry[] {
  const out: SaveEntry[] = [];
  for (let i = 0; i < list.savesLength(); i++) {
    const s = list.saves(i);
    if (!s) continue;
    out.push({
      file: s.file() ?? "",
      worldName: s.worldName() ?? "",
      worldId: s.worldId() ?? "",
      label: s.label() ?? "",
      kind: s.kind() ?? "",
      generation: Number(s.generation()),
      createdUnixMs: Number(s.createdUnixMs()),
      simMinute: Number(s.simMinute()),
      sizeBytes: Number(s.sizeBytes()),
      compatible: s.compatible(),
      contentChanged: s.contentChanged(),
      note: s.note() ?? "",
    });
  }
  return out;
}

export function decodeResponse(payload: Uint8Array): ResponseBody {
  const r = W.Response.getRootAsResponse(bb(payload));
  switch (r.bodyType()) {
    case W.ResponseBody.Ack:
      return { kind: "ack", message: (r.body(new W.Ack()) as W.Ack | null)?.message() ?? "" };
    case W.ResponseBody.RasterTile: {
      const t = r.body(new W.RasterTile()) as W.RasterTile | null;
      if (!t) break;
      return { kind: "raster", tile: rasterTile(t) };
    }
    case W.ResponseBody.Hydrography: {
      const h = r.body(new W.Hydrography()) as W.Hydrography | null;
      if (!h) break;
      return { kind: "hydrography", hydrography: hydrography(h) };
    }
    case W.ResponseBody.SaveList: {
      const l = r.body(new W.SaveList()) as W.SaveList | null;
      if (!l) break;
      return { kind: "saves", saves: saveEntries(l) };
    }
    default:
      break;
  }
  throw new Error(`unexpected response body ${r.bodyType()}`);
}
