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

export interface ActivityInfo {
  id: string;
  /** "Gather plants" */
  name: string;
  /** "gathering wild plants" */
  doing: string;
}

export interface GoodInfo {
  id: string;
  /** "Meat" */
  name: string;
  /** "food" or "fuel" */
  purpose: string;
  /** Food energy, kcal per kilogram (0 for fuel). */
  kcalPerKg: number;
}

export interface CropInfo {
  id: string;
  /** "Emmer wheat" */
  name: string;
  /** Indexes into Welcome.goods: the grain it yields and the seed it is sown from. */
  good: number;
  seedGood: number;
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
  /** The activity catalogue, in the order people's activity indices refer to. */
  activities: ActivityInfo[];
  /** Labels of decision-receipt reasons, by code. */
  reasons: Record<number, string>;
  bandSizeMin: number;
  bandSizeMax: number;
  bandSizeDefault: number;
  /** The goods catalogue, in the order stores and loads refer to. */
  goods: GoodInfo[];
  /** The crops catalogue, in the order fields refer to. */
  crops: CropInfo[];
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

export type Sex = "female" | "male";

/** One living person as the snapshot carries them (ADR-0003 §2.3). */
export interface PersonBrief {
  id: number;
  /** Where they are at the snapshot's minute, metres from the north-west corner. */
  x: number;
  y: number;
  /** Index into Welcome.activities. */
  activity: number;
  /** The walk they are on (0 = none) and its revision. */
  trip: number;
  tripRev: number;
  sex: Sex;
  ageYears: number;
  household: number;
  asleep: boolean;
}

export interface SettlementBrief {
  id: number;
  name: string;
  x: number;
  y: number;
  foundedMinute: number;
  population: number;
  /** Days of food in its households' stores at its people's needs. */
  foodDays: number;
  /** Its food has run short and not yet recovered. */
  foodShort: boolean;
  /** Grain threshed from its fields this harvest, kilograms. */
  harvestKg: number;
}

export interface Snapshot {
  world: WorldInfo | null;
  clock: Clock | null;
  task: Task | null;
  recovery: Recovery | null;
  lastError: string | null;
  lastAutosaveUnixMs: number;
  people: PersonBrief[];
  settlements: SettlementBrief[];
  /** The newest chronicle entry's sequence number (0 = none). */
  chronicleHead: number;
  /** Changes whenever a field is marked out or changes stage (0 = no fields). */
  fieldsRev: number;
}

export type FieldStage = "fallow" | "prepared" | "sown" | "reaped";

/** A field people work (M1 slice C). */
export interface FieldInfo {
  id: number;
  household: number;
  settlement: number;
  /** North-west corner and size, metres. */
  x: number;
  y: number;
  w: number;
  h: number;
  /** An index into Welcome.crops. */
  crop: number;
  stage: FieldStage;
  stageSinceMinute: number;
  /** Share of the work of the stage under way that is done, 0–1. */
  progress: number;
  /** Ground not yet broken for a first crop, and whether it is woodland to clear first. */
  newGround: boolean;
  woodland: boolean;
  /** Sown and ripe, waiting to be reaped. */
  ripe: boolean;
  expectedKg: number;
  sheavesKg: number;
  harvests: number;
  /** Rendered by the kernel: "growing; ripe in about 20 days". */
  status: string;
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

/** A walk: a route with the minutes after departure at each vertex (ADR-0003 §2). */
export interface TripInfo {
  id: number;
  rev: number;
  person: number;
  departMinute: number;
  /** x, y pairs in metres. */
  points: Float32Array;
  minutes: Float32Array;
}

export interface Span {
  kind: "text" | "person" | "settlement";
  text: string;
  id: number;
}

export interface ChronicleEntry {
  seq: number;
  minute: number;
  spans: Span[];
}

export interface Term {
  reason: number;
  points: number;
}

export interface ScoredOption {
  /** Index into Welcome.activities; 65535 = an activity the content no longer has. */
  activity: number;
  /** Rendered by the kernel: "home", "the hearth", "woodland 600 m east of home". */
  target: string;
  total: number;
  terms: Term[];
}

/** Why a person chose what they did, recorded when they chose it. */
export interface Decision {
  minute: number;
  chosen: ScoredOption | null;
  runnerUp: ScoredOption | null;
  others: ScoredOption[];
  excluded: { activity: number; reason: number }[];
  probability: number;
  temperature: number;
  /** Hunger, sleep drive, loneliness (0–1), days of food, days of water. */
  needs: number[];
}

export interface StoreLine {
  /** An index into Welcome.goods. */
  good: number;
  kg: number;
}

export interface KinLink {
  id: number;
  name: string;
  relation: string;
  alive: boolean;
}

export interface PersonInfo {
  id: number;
  name: string;
  sex: Sex;
  bornMinute: number;
  ageYears: number;
  alive: boolean;
  diedMinute: number;
  cause: string;
  origin: string;
  household: number;
  settlement: number;
  settlementName: string;
  kin: KinLink[];
  activity: number;
  /** Rendered by the kernel: "walking to woodland 600 m east of home (gathering wild plants)". */
  doing: string;
  sinceMinute: number;
  untilMinute: number;
  hunger: number;
  sleepPressure: number;
  loneliness: number;
  energyKcal: number;
  /** Food energy of what they carry, kcal. */
  carryFoodKcal: number;
  /** The good carried (an index into Welcome.goods), or -1. */
  carryGood: number;
  carryKg: number;
  carryWaterL: number;
  householdFoodDays: number;
  householdWaterDays: number;
  householdFuelDays: number;
  /** The household's goods in store. */
  stores: StoreLine[];
  /** Newest first. */
  decisions: Decision[];
  traits: number[];
  x: number;
  y: number;
}

export type ResponseBody =
  | { kind: "ack"; message: string }
  | { kind: "raster"; tile: RasterTile }
  | { kind: "hydrography"; hydrography: Hydrography }
  | { kind: "saves"; saves: SaveEntry[] }
  | { kind: "trips"; trips: TripInfo[] }
  | { kind: "person"; person: PersonInfo }
  | { kind: "chronicle"; entries: ChronicleEntry[]; head: number }
  | { kind: "fields"; rev: number; fields: FieldInfo[] };

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
  /** People in the founding band; 0 = the content's default. */
  bandSize?: number;
}): Uint8Array {
  const b = new flatbuffers.Builder(128);
  const preset = b.createString(args.presetId);
  const name = b.createString(args.name);
  const body = W.NewWorld.createNewWorld(
    b,
    args.seed,
    preset,
    args.sizeCells,
    name,
    args.bandSize ?? 0,
  );
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

export function getTrips(ids: number[]): Uint8Array {
  const b = new flatbuffers.Builder(32 + ids.length * 8);
  const vector = W.GetTrips.createIdsVector(b, ids.map((id) => BigInt(id)));
  return query(b, W.QueryBody.GetTrips, W.GetTrips.createGetTrips(b, vector));
}

export function getPerson(id: number, decisions: number): Uint8Array {
  const b = new flatbuffers.Builder(32);
  return query(b, W.QueryBody.GetPerson, W.GetPerson.createGetPerson(b, BigInt(id), decisions));
}

export function getFields(): Uint8Array {
  const b = new flatbuffers.Builder(16);
  W.GetFields.startGetFields(b);
  return query(b, W.QueryBody.GetFields, W.GetFields.endGetFields(b));
}

export function getChronicle(afterSeq: number, limit: number): Uint8Array {
  const b = new flatbuffers.Builder(32);
  return query(
    b,
    W.QueryBody.GetChronicle,
    W.GetChronicle.createGetChronicle(b, BigInt(afterSeq), limit),
  );
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
  const activities: ActivityInfo[] = [];
  for (let i = 0; i < w.activitiesLength(); i++) {
    const a = w.activities(i);
    if (!a) continue;
    activities.push({ id: a.id() ?? "", name: a.name() ?? "", doing: a.doing() ?? "" });
  }
  const goods: GoodInfo[] = [];
  for (let i = 0; i < w.goodsLength(); i++) {
    const g = w.goods(i);
    if (!g) continue;
    goods.push({
      id: g.id() ?? "",
      name: g.name() ?? "",
      purpose: g.purpose() ?? "",
      kcalPerKg: g.kcalPerKg(),
    });
  }
  const crops: CropInfo[] = [];
  for (let i = 0; i < w.cropsLength(); i++) {
    const c = w.crops(i);
    if (!c) continue;
    crops.push({ id: c.id() ?? "", name: c.name() ?? "", good: c.good(), seedGood: c.seedGood() });
  }
  const reasons: Record<number, string> = {};
  for (let i = 0; i < w.reasonsLength(); i++) {
    const r = w.reasons(i);
    if (r) reasons[r.code()] = r.label() ?? "";
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
    activities,
    reasons,
    bandSizeMin: w.bandSizeMin(),
    bandSizeMax: w.bandSizeMax(),
    bandSizeDefault: w.bandSizeDefault(),
    goods,
    crops,
  };
}

const sexOf = (s: W.Sex): Sex => (s === W.Sex.Male ? "male" : "female");

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
    people: personBriefs(s),
    settlements: settlementBriefs(s),
    chronicleHead: Number(s.chronicleHead()),
    fieldsRev: Number(s.fieldsRev()),
  };
}

function personBriefs(s: W.Snapshot): PersonBrief[] {
  const out: PersonBrief[] = [];
  const brief = new W.PersonBrief();
  const v = new W.Vec2();
  for (let i = 0; i < s.peopleLength(); i++) {
    const p = s.people(i, brief);
    if (!p) continue;
    const pos = p.pos(v);
    out.push({
      id: Number(p.id()),
      x: pos?.x() ?? 0,
      y: pos?.y() ?? 0,
      activity: p.activity(),
      trip: Number(p.trip()),
      tripRev: p.tripRev(),
      sex: sexOf(p.sex()),
      ageYears: p.ageYears(),
      household: Number(p.household()),
      asleep: p.asleep(),
    });
  }
  return out;
}

function settlementBriefs(s: W.Snapshot): SettlementBrief[] {
  const out: SettlementBrief[] = [];
  for (let i = 0; i < s.settlementsLength(); i++) {
    const t = s.settlements(i);
    if (!t) continue;
    const hearth = t.hearth();
    out.push({
      id: Number(t.id()),
      name: t.name() ?? "",
      x: hearth?.x() ?? 0,
      y: hearth?.y() ?? 0,
      foundedMinute: Number(t.foundedMinute()),
      population: t.population(),
      foodDays: t.foodDays(),
      foodShort: t.foodShort(),
      harvestKg: t.harvestKg(),
    });
  }
  return out;
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

function points(n: number, at: (k: number, v: W.Vec2) => W.Vec2 | null): Float32Array {
  const out = new Float32Array(n * 2);
  const v = new W.Vec2();
  for (let k = 0; k < n; k++) {
    const p = at(k, v);
    if (!p) continue;
    out[2 * k] = p.x();
    out[2 * k + 1] = p.y();
  }
  return out;
}

function trips(t: W.Trips): TripInfo[] {
  const out: TripInfo[] = [];
  for (let i = 0; i < t.tripsLength(); i++) {
    const trip = t.trips(i);
    if (!trip) continue;
    out.push({
      id: Number(trip.id()),
      rev: trip.rev(),
      person: Number(trip.person()),
      departMinute: Number(trip.departMinute()),
      points: points(trip.pointsLength(), (k, v) => trip.points(k, v)),
      minutes: (trip.minutesArray() ?? new Float32Array()).slice(),
    });
  }
  return out;
}

const SPAN_KINDS: Record<number, Span["kind"]> = {
  [W.SpanKind.Text]: "text",
  [W.SpanKind.Person]: "person",
  [W.SpanKind.Settlement]: "settlement",
};

function chronicle(c: W.Chronicle): { entries: ChronicleEntry[]; head: number } {
  const entries: ChronicleEntry[] = [];
  for (let i = 0; i < c.entriesLength(); i++) {
    const e = c.entries(i);
    if (!e) continue;
    const spans: Span[] = [];
    for (let k = 0; k < e.spansLength(); k++) {
      const s = e.spans(k);
      if (!s) continue;
      spans.push({ kind: SPAN_KINDS[s.kind()] ?? "text", text: s.text() ?? "", id: Number(s.id()) });
    }
    entries.push({ seq: Number(e.seq()), minute: Number(e.minute()), spans });
  }
  return { entries, head: Number(c.head()) };
}

const FIELD_STAGES: Record<number, FieldStage> = {
  [W.FieldStage.Fallow]: "fallow",
  [W.FieldStage.Prepared]: "prepared",
  [W.FieldStage.Sown]: "sown",
  [W.FieldStage.Reaped]: "reaped",
};

function fields(f: W.Fields): { rev: number; fields: FieldInfo[] } {
  const out: FieldInfo[] = [];
  const info = new W.FieldInfo();
  const v = new W.Vec2();
  for (let i = 0; i < f.fieldsLength(); i++) {
    const x = f.fields(i, info);
    if (!x) continue;
    const min = x.min(v);
    const [mx, my] = [min?.x() ?? 0, min?.y() ?? 0];
    const size = x.size(v);
    out.push({
      id: Number(x.id()),
      household: Number(x.household()),
      settlement: Number(x.settlement()),
      x: mx,
      y: my,
      w: size?.x() ?? 0,
      h: size?.y() ?? 0,
      crop: x.crop(),
      stage: FIELD_STAGES[x.stage()] ?? "fallow",
      stageSinceMinute: Number(x.stageSinceMinute()),
      progress: x.progress(),
      newGround: x.newGround(),
      woodland: x.woodland(),
      ripe: x.ripe(),
      expectedKg: x.expectedKg(),
      sheavesKg: x.sheavesKg(),
      harvests: x.harvests(),
      status: x.status() ?? "",
    });
  }
  return { rev: Number(f.rev()), fields: out };
}

function scoredOption(o: W.ScoredOption): ScoredOption {
  const terms: Term[] = [];
  for (let k = 0; k < o.termsLength(); k++) {
    const t = o.terms(k);
    if (t) terms.push({ reason: t.reason(), points: t.points() });
  }
  return { activity: o.activity(), target: o.target() ?? "", total: o.total(), terms };
}

function decision(d: W.Decision): Decision {
  const others: ScoredOption[] = [];
  for (let k = 0; k < d.othersLength(); k++) {
    const o = d.others(k);
    if (o) others.push(scoredOption(o));
  }
  const excluded: Decision["excluded"] = [];
  for (let k = 0; k < d.excludedLength(); k++) {
    const x = d.excluded(k);
    if (x) excluded.push({ activity: x.activity(), reason: x.reason() });
  }
  const chosen = d.chosen();
  const runnerUp = d.runnerUp();
  return {
    minute: Number(d.minute()),
    chosen: chosen ? scoredOption(chosen) : null,
    runnerUp: runnerUp ? scoredOption(runnerUp) : null,
    others,
    excluded,
    probability: d.probability(),
    temperature: d.temperature(),
    needs: Array.from(d.needsArray() ?? []),
  };
}

function personInfo(p: W.PersonInfo): PersonInfo {
  const kin: KinLink[] = [];
  for (let k = 0; k < p.kinLength(); k++) {
    const l = p.kin(k);
    if (l) {
      kin.push({
        id: Number(l.id()),
        name: l.name() ?? "",
        relation: l.relation() ?? "",
        alive: l.alive(),
      });
    }
  }
  const decisions: Decision[] = [];
  for (let k = 0; k < p.decisionsLength(); k++) {
    const d = p.decisions(k);
    if (d) decisions.push(decision(d));
  }
  const stores: StoreLine[] = [];
  for (let k = 0; k < p.storesLength(); k++) {
    const line = p.stores(k);
    if (line) stores.push({ good: line.good(), kg: line.kg() });
  }
  const pos = p.pos();
  return {
    id: Number(p.id()),
    name: p.name() ?? "",
    sex: sexOf(p.sex()),
    bornMinute: Number(p.bornMinute()),
    ageYears: p.ageYears(),
    alive: p.alive(),
    diedMinute: Number(p.diedMinute()),
    cause: p.cause() ?? "",
    origin: p.origin() ?? "",
    household: Number(p.household()),
    settlement: Number(p.settlement()),
    settlementName: p.settlementName() ?? "",
    kin,
    activity: p.activity(),
    doing: p.doing() ?? "",
    sinceMinute: Number(p.sinceMinute()),
    untilMinute: Number(p.untilMinute()),
    hunger: p.hunger(),
    sleepPressure: p.sleepPressure(),
    loneliness: p.loneliness(),
    energyKcal: p.energyKcal(),
    carryFoodKcal: p.carryFoodKcal(),
    carryGood: p.carryGood(),
    carryKg: p.carryKg(),
    carryWaterL: p.carryWaterL(),
    householdFoodDays: p.householdFoodDays(),
    householdWaterDays: p.householdWaterDays(),
    householdFuelDays: p.householdFuelDays(),
    stores,
    decisions,
    traits: Array.from(p.traitsArray() ?? []),
    x: pos?.x() ?? 0,
    y: pos?.y() ?? 0,
  };
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
    case W.ResponseBody.Trips: {
      const t = r.body(new W.Trips()) as W.Trips | null;
      if (!t) break;
      return { kind: "trips", trips: trips(t) };
    }
    case W.ResponseBody.PersonInfo: {
      const p = r.body(new W.PersonInfo()) as W.PersonInfo | null;
      if (!p) break;
      return { kind: "person", person: personInfo(p) };
    }
    case W.ResponseBody.Chronicle: {
      const c = r.body(new W.Chronicle()) as W.Chronicle | null;
      if (!c) break;
      return { kind: "chronicle", ...chronicle(c) };
    }
    case W.ResponseBody.Fields: {
      const f = r.body(new W.Fields()) as W.Fields | null;
      if (!f) break;
      return { kind: "fields", ...fields(f) };
    }
    default:
      break;
  }
  throw new Error(`unexpected response body ${r.bodyType()}`);
}
