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
  /** "food", "fuel", "material" or "tool" */
  purpose: string;
  /** Food energy, kcal per kilogram (0 for fuel). */
  kcalPerKg: number;
  /** How it is eaten: "raw", "cooked" or "never" (a recipe makes it food first). */
  eaten: string;
  /** For a tool, hours of use a standard tool lasts; stores count tools in standard tools. 0 for
   * other goods, counted in kilograms. */
  toolLifeH: number;
}

export interface SkillInfo {
  id: string;
  /** "Milling" */
  name: string;
}

/** A person's level in a skill. */
export interface SkillLine {
  /** An index into Welcome.skills. */
  skill: number;
  /** 0–1. */
  level: number;
}

/** A technique people know, learn and can lose (M3b slice M; ADR-0008). */
export interface TechniqueInfo {
  id: string;
  /** "Shaping stone" */
  name: string;
  /** What a competent person can do, completing "A competent person can ...". */
  can: string;
  /** An index into Welcome.skills of the skill whose practice it is, or -1. */
  domain: number;
  /** Its prerequisites in words; empty for none. */
  requires: string;
  /** Hours of work beside someone who knows it that teach it. */
  learnH: number;
  /** Children brought up in a household that knows it learn it at the work's age. */
  upbringing: boolean;
}

/** What a person knows of a technique (M3b slice M). */
export interface KnowLine {
  /** An index into Welcome.techniques. */
  technique: number;
  state: "heard" | "learning" | "known";
  /** Hours learnt beside someone who knows it, and the hours that teach it. */
  hours: number;
  learnH: number;
  sinceMinute: number;
  /** How it came, in words rendered by the kernel ("taught by Wren"). */
  source: string;
  /** Who it came from (0 = nobody). */
  sourcePerson: number;
  usedMinute: number;
}

export interface CropInfo {
  id: string;
  /** "Emmer wheat" */
  name: string;
  /** Indexes into Welcome.goods: the grain it yields and the seed it is sown from. */
  good: number;
  seedGood: number;
}

/** A property regime a new world can be made under (M3a slice K; ADR-0007). */
export interface RegimeInfo {
  id: string;
  /** "Household fields" */
  name: string;
  description: string;
  isDefault: boolean;
  /** Its rules, rendered by the kernel, one sentence each. */
  rules: string[];
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
  /** The skills catalogue, in the order people's skills refer to. */
  skills: SkillInfo[];
  /** The property regimes a new world can be made under. */
  regimes: RegimeInfo[];
  /** The techniques, in the order people's knowledge refers to. */
  techniques: TechniqueInfo[];
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
  /** The property regime it lives under, fixed when it was made (M3a slice K). */
  regimeId: string;
  regimeName: string;
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
  /** Changes whenever ground is claimed for a building or work on one moves on (0 = none). */
  buildingsRev: number;
  /** Changes whenever the paths are surveyed: monthly, and when a world is made or loaded. */
  pathsRev: number;
  /**
   * Changes whenever a household posts terms, a trade is made or a want goes unmet (0 = nothing
   * offered or traded yet).
   */
  marketsRev: number;
  /** Changes whenever a workshop opens, closes, makes, sells, pays or is reviewed (0 = none yet). */
  firmsRev: number;
  /**
   * Changes at the start of each month, when households form or end, and when a year's wealth
   * measures are recorded (0 = no households).
   */
  wealthRev: number;
  /**
   * Changes whenever someone comes to know, learns toward, hears of or loses a technique, and when
   * people arrive, leave or die (0 = no people).
   */
  knowledgeRev: number;
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
  /** Who holds it (M3a slice K): a household's id, or else a settlement's; `household` works it. */
  holder: number;
  holderSettlement: number;
  /** When it is let: when the lease's term ends (-1 = not let), and the holder's share of the grain. */
  leaseUntilMinute: number;
  leaseShare: number;
}

/** Ground worn by walking in one tile of cells (M1 slice F), as last surveyed. */
export interface WornTile {
  /** Tile row × tiles per row + tile column. */
  index: number;
  /** Wear of each cell, row by row, 0–255 for 0–1. */
  wear: Uint8Array;
  /** 1 where the cell is trail, row by row. */
  trail: Uint8Array;
}

/** A trail traced through trail cells (M1 slice F). */
export interface TrailInfo {
  /** Vertices, metres. */
  points: [number, number][];
  /** Mean wear of its cells, 0–1. */
  wear: number;
  lengthM: number;
}

/** The worn ground and the trails, as last surveyed. */
export interface PathsInfo {
  rev: number;
  /** Tiles per row, and cells per tile side. */
  tilesX: number;
  tileCells: number;
  worn: WornTile[];
  trails: TrailInfo[];
}

/** A building and the ground it stands on (M1 slice D), as the kernel expands its design. */
export interface BuildingInfo {
  id: number;
  household: number;
  settlement: number;
  /** The program's name: "Hut". */
  program: string;
  /** Centre, metres; radius of the wall line and of the roof's edge, metres. */
  x: number;
  y: number;
  radiusM: number;
  roofRadiusM: number;
  /** The doorway's middle, metres, and the direction it faces, radians from east toward south. */
  door: [number, number];
  doorDir: number;
  /** The stage under way (0 foundation … 4 finish; 5 once finished) and its name. */
  stage: number;
  stageName: string;
  /** Share of the stage's work done, 0–1. */
  progress: number;
  roofed: boolean;
  /** The outer face of the walls, metres, round from the door. */
  outline: [number, number][];
  /** Where the posts stand, metres. */
  posts: [number, number][];
  /** The plot it stands on: north-west corner and size, metres. */
  plot: { x: number; y: number; w: number; h: number } | null;
  floorM2: number;
  sleeps: number;
  startedMinute: number;
  /** Rendered by the kernel: "walls going up, 40% done; waiting for timber". */
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
  kind: "text" | "person" | "settlement" | "firm";
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
  /** They left the valley alive. */
  left: boolean;
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
  /** All the household's food, grain and flour included, in days of its needs. */
  householdFoodDays: number;
  /** Its food ready to eat, in days. */
  householdReadyDays: number;
  householdWaterDays: number;
  householdFuelDays: number;
  /** The household's goods in store. */
  stores: StoreLine[];
  /** Their skills. */
  skills: SkillLine[];
  /** What they know, are learning or have heard of. */
  knows: KnowLine[];
  /** Newest first. */
  decisions: Decision[];
  traits: number[];
  x: number;
  y: number;
  /** Their partner (0 = none). */
  partner: number;
  /** Their family life in sentences rendered by the kernel ("Nursing Wren."). */
  family: string[];
  /** When they left the valley alive (0 = they did not). */
  leftMinute: number;
}

/** One good in a settlement's market (M3a slice I). Tallies fade by half every memory. */
export interface MarketGood {
  /** An index into Welcome.goods. */
  good: number;
  /** Units offered, and by how many sellers (households and workshops). */
  offered: number;
  sellers: number;
  /** How many of the sellers are workshops. */
  workshops: number;
  sold: number;
  /** Units wanted that found no offer, and what a unit was worth to those buyers, hours. */
  unmet: number;
  unmetWorthH: number;
  /** Its share of the worth of the payments made, 0–1. */
  acceptance: number;
  /** The last trade's payment good (-1 = not sold yet) and units of it for a unit. */
  lastPayment: number;
  lastPrice: number;
}

/** A household's or workshop's posted terms for a good: so much of a payment good for a unit. */
export interface OfferInfo {
  /** The household's id, or the workshop's when `firm` is set. */
  household: number;
  /** "Ada's household", "Wren's sickle workshop". */
  householdName: string;
  good: number;
  payment: number;
  price: number;
  units: number;
  /** The terms are a workshop's. */
  firm: boolean;
}

/** A trade a settlement's market remembers. */
export interface TradeInfo {
  minute: number;
  seller: number;
  buyer: number;
  good: number;
  units: number;
  payment: number;
  paid: number;
  /** Paid in the settlement's money, not goods for goods. */
  sale: boolean;
  /** Rendered by the kernel: "Ada's household sold a sickle to Bran's household for …". */
  text: string;
  /** The seller was a workshop (`seller` is its id). */
  sellerFirm: boolean;
}

/** A month of trade in one good: a line of the price history. */
export interface MonthOfTrade {
  /** Months since the calendar's origin: (year − 1) × 12 + month − 1. */
  month: number;
  good: number;
  trades: number;
  units: number;
  /** What the payments were worth to the sellers, hours of their own work. */
  paidH: number;
}

/** A settlement's market (M3a slice I). */
export interface MarketInfo {
  settlement: number;
  settlementName: string;
  /** Its money, an index into Welcome.goods, or -1 while it trades by barter. */
  money: number;
  /** Rendered by the kernel: "Barter: no good settles most of what is paid; …". */
  summary: string;
  /** Trades remembered, and the half-life of what the market remembers, days. */
  trades: number;
  memoryDays: number;
  goods: MarketGood[];
  offers: OfferInfo[];
  /** Newest first. */
  recent: TradeInfo[];
  /** Oldest first. */
  history: MonthOfTrade[];
}

/** What a line of a workshop's books records (M3a slice J). */
export type BookKind =
  | "put-in"
  | "drawn"
  | "made"
  | "used"
  | "sold"
  | "paid"
  | "wages"
  | "lost"
  | "unknown";

/** A workshop in brief (M3a slice J): in M3a, every firm is a household's workshop. */
export interface FirmBrief {
  id: number;
  /** "Wren's sickle workshop". */
  name: string;
  /** The household that owns it. */
  owner: number;
  ownerName: string;
  settlement: number;
  settlementName: string;
  foundedMinute: number;
  open: boolean;
  /** When it closed, and why: "it sold nothing for months" (closed workshops only). */
  closedMinute: number;
  closedWhy: string;
  /** What it makes to sell: indexes into Welcome.goods. */
  lines: number[];
  /** Hours of work it would hire before its next review (0 = it hires no one now). */
  hiringH: number;
  /** Rendered by the kernel: "Made 3.0 sickles and sold a sickle." */
  record: string;
}

/** What a workshop pays for work. */
export interface WageInfo {
  /** The work (an index into Welcome.activities) and what it pays in (into Welcome.goods). */
  activity: number;
  pay: number;
  /** Units of the pay good for an hour's work, and what that is worth to its owners, hours. */
  perHour: number;
  hourH: number;
  /** Hours of work it wants before its next review, and the hours taken of those. */
  hours: number;
  taken: number;
  /** Rendered by the kernel: "Pays 0.90 kg of grain an hour; wants 6 more hours of work …". */
  text: string;
}

/** A line of a workshop's books. */
export interface BookEntry {
  minute: number;
  kind: BookKind;
  /** An index into Welcome.goods, and how much of it, in its unit. */
  good: number;
  amount: number;
  /** The household on the other side (0 = none). */
  other: number;
  /** Rendered by the kernel: "Sold a sickle to Bran's household." */
  text: string;
}

/** How much of a good a workshop's books moved in a month, by kind. */
export interface BookLine {
  kind: BookKind;
  good: number;
  amount: number;
}

/** A month of a workshop's books. Worth is in hours of its owners' own work. */
export interface MonthStatement {
  /** Months since the calendar's origin: (year − 1) × 12 + month − 1. */
  month: number;
  lines: BookLine[];
  /** Hours worked for it by its owners' household, and by people it hired. */
  ownerH: number;
  hiredH: number;
  /** What its payments were worth, and its inputs and wages. */
  incomeH: number;
  costsH: number;
  /** Its stock as the month ended (as it stands, for the month under way). */
  stockH: number;
}

/** A workshop's page (M3a slice J). */
export interface FirmInfo {
  brief: FirmBrief;
  founder: number;
  founderName: string;
  ownerSinceMinute: number;
  /** When it last sold anything (-1 = never). */
  lastSaleMinute: number;
  /** What it holds: its stock and what it was paid. */
  stores: StoreLine[];
  offers: OfferInfo[];
  /** What it pays for work (null if it never hired). */
  wage: WageInfo | null;
  /** The latest lines of its books, newest first. */
  entries: BookEntry[];
  /** Every month of its life, oldest first. */
  months: MonthStatement[];
}

/**
 * How a settlement's wealth measures spread over its people (M3a slice K; ADR-0007 §4). Ginis of
 * goods and land are person-weighted per head; floor area is compared house by house. Goods are
 * valued in hours of work at the settlement's prices.
 */
export interface WealthSpread {
  /** The year that ended (a year's record), or the year under way (as they stand). */
  year: number;
  households: number;
  people: number;
  giniGoods: number;
  giniHeld: number;
  giniWorked: number;
  giniFloor: number;
  /** The share of all goods the richest tenth of people have. */
  topTenthGoods: number;
  /** The shares of households that hold no land, and that work none. */
  holdingNone: number;
  workingNone: number;
  goodsHPerHead: number;
  workedHaPerHead: number;
  floorM2PerHouse: number;
  /** Land the settlement itself holds, hectares. */
  commonHa: number;
}

/** A household's wealth measures (M3a slice K). */
export interface HouseholdWealth {
  household: number;
  /** "Wren's household" */
  name: string;
  members: number;
  heldHa: number;
  workedHa: number;
  letHa: number;
  rentedHa: number;
  /** Its goods and its workshops' stock, hours of work at the settlement's prices. */
  goodsH: number;
  floorM2: number;
}

export interface SettlementWealth {
  settlement: number;
  name: string;
  /** As they stand (null for a settlement with nobody left). */
  now: WealthSpread | null;
  /** Its households, the most goods per head first. */
  households: HouseholdWealth[];
  /** At the end of each year, oldest first. */
  history: WealthSpread[];
}

export interface WealthInfo {
  rev: number;
  /** The world's property regime, by name. */
  regimeName: string;
  settlements: SettlementWealth[];
}

/** Someone named in a knowledge listing. */
export interface PersonRef {
  id: number;
  name: string;
  ageYears: number;
}

/** A technique in one settlement (M3b slice M). */
export interface TechniqueHere {
  /** An index into Welcome.techniques. */
  technique: number;
  /** Known there now. */
  known: boolean;
  /** Who knows it, eldest first; who is learning it; who has only heard of it. */
  knowers: PersonRef[];
  learners: PersonRef[];
  heard: PersonRef[];
  /** Knowers who did its work in the last year. */
  practisedLastYear: number;
  /** Its state in words, rendered by the kernel. */
  status: string;
  /** Its history there, oldest first, in sentences. */
  history: string[];
}

export interface SettlementKnowledge {
  settlement: number;
  name: string;
  techniques: TechniqueHere[];
}

export interface KnowledgeInfo {
  rev: number;
  settlements: SettlementKnowledge[];
}

export type ResponseBody =
  | { kind: "ack"; message: string }
  | { kind: "raster"; tile: RasterTile }
  | { kind: "hydrography"; hydrography: Hydrography }
  | { kind: "saves"; saves: SaveEntry[] }
  | { kind: "trips"; trips: TripInfo[] }
  | { kind: "person"; person: PersonInfo }
  | { kind: "chronicle"; entries: ChronicleEntry[]; head: number }
  | { kind: "fields"; rev: number; fields: FieldInfo[] }
  | { kind: "buildings"; rev: number; buildings: BuildingInfo[] }
  | { kind: "paths"; paths: PathsInfo }
  | { kind: "markets"; rev: number; markets: MarketInfo[] }
  | { kind: "firms"; rev: number; firms: FirmBrief[] }
  | { kind: "firm"; firm: FirmInfo }
  | { kind: "wealth"; wealth: WealthInfo }
  | { kind: "knowledge"; knowledge: KnowledgeInfo };

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
  /** The property regime, by content id; empty or absent = the content's default. */
  regimeId?: string;
}): Uint8Array {
  const b = new flatbuffers.Builder(128);
  const preset = b.createString(args.presetId);
  const name = b.createString(args.name);
  const regime = b.createString(args.regimeId ?? "");
  const body = W.NewWorld.createNewWorld(
    b,
    args.seed,
    preset,
    args.sizeCells,
    name,
    args.bandSize ?? 0,
    regime,
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

/**
 * Sends `families` families (1 to 20; the host refuses more) to a point on the map, metres
 * (god tool). They arrive together and settle side by side.
 */
export function spawnFamily(x: number, y: number, families = 1): Uint8Array {
  const b = new flatbuffers.Builder(32);
  W.SpawnFamily.startSpawnFamily(b);
  W.SpawnFamily.addAt(b, W.Vec2.createVec2(b, x, y));
  W.SpawnFamily.addFamilies(b, families);
  return command(b, W.CommandBody.SpawnFamily, W.SpawnFamily.endSpawnFamily(b));
}

/**
 * The observer introduces a technique (god tool, ADR-0008 §6): a living person comes to know it,
 * or with `awareOnly` only hears of it. `technique` is an index into Welcome.techniques.
 */
export function introduceTechnique(person: number, technique: number, awareOnly: boolean): Uint8Array {
  const b = new flatbuffers.Builder(32);
  const body = W.IntroduceTechnique.createIntroduceTechnique(
    b,
    BigInt(person),
    technique,
    awareOnly,
  );
  return command(b, W.CommandBody.IntroduceTechnique, body);
}

/** Runs ahead to a simulation minute, unpaced and in full detail. */
export function runUntil(minute: number): Uint8Array {
  const b = new flatbuffers.Builder(32);
  return command(b, W.CommandBody.RunUntil, W.RunUntil.createRunUntil(b, BigInt(minute)));
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

export function getBuildings(): Uint8Array {
  const b = new flatbuffers.Builder(16);
  W.GetBuildings.startGetBuildings(b);
  return query(b, W.QueryBody.GetBuildings, W.GetBuildings.endGetBuildings(b));
}

export function getPaths(): Uint8Array {
  const b = new flatbuffers.Builder(16);
  W.GetPaths.startGetPaths(b);
  return query(b, W.QueryBody.GetPaths, W.GetPaths.endGetPaths(b));
}

export function getMarkets(): Uint8Array {
  const b = new flatbuffers.Builder(16);
  W.GetMarkets.startGetMarkets(b);
  return query(b, W.QueryBody.GetMarkets, W.GetMarkets.endGetMarkets(b));
}

export function getFirms(): Uint8Array {
  const b = new flatbuffers.Builder(16);
  W.GetFirms.startGetFirms(b);
  return query(b, W.QueryBody.GetFirms, W.GetFirms.endGetFirms(b));
}

export function getFirm(id: number): Uint8Array {
  const b = new flatbuffers.Builder(32);
  return query(b, W.QueryBody.GetFirm, W.GetFirm.createGetFirm(b, BigInt(id)));
}

export function getKnowledge(): Uint8Array {
  const b = new flatbuffers.Builder(16);
  W.GetKnowledge.startGetKnowledge(b);
  return query(b, W.QueryBody.GetKnowledge, W.GetKnowledge.endGetKnowledge(b));
}

export function getWealth(): Uint8Array {
  const b = new flatbuffers.Builder(16);
  W.GetWealth.startGetWealth(b);
  return query(b, W.QueryBody.GetWealth, W.GetWealth.endGetWealth(b));
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
      eaten: g.eaten() ?? "",
      toolLifeH: g.toolLifeH(),
    });
  }
  const skills: SkillInfo[] = [];
  for (let i = 0; i < w.skillsLength(); i++) {
    const k = w.skills(i);
    if (k) skills.push({ id: k.id() ?? "", name: k.name() ?? "" });
  }
  const crops: CropInfo[] = [];
  for (let i = 0; i < w.cropsLength(); i++) {
    const c = w.crops(i);
    if (!c) continue;
    crops.push({ id: c.id() ?? "", name: c.name() ?? "", good: c.good(), seedGood: c.seedGood() });
  }
  const regimes: RegimeInfo[] = [];
  for (let i = 0; i < w.regimesLength(); i++) {
    const r = w.regimes(i);
    if (!r) continue;
    regimes.push({
      id: r.id() ?? "",
      name: r.name() ?? "",
      description: r.description() ?? "",
      isDefault: r.isDefault(),
      rules: Array.from({ length: r.rulesLength() }, (_, k) => r.rules(k) ?? ""),
    });
  }
  const techniques: TechniqueInfo[] = [];
  for (let i = 0; i < w.techniquesLength(); i++) {
    const t = w.techniques(i);
    if (!t) continue;
    techniques.push({
      id: t.id() ?? "",
      name: t.name() ?? "",
      can: t.can() ?? "",
      domain: t.domain(),
      requires: t.requires() ?? "",
      learnH: t.learnH(),
      upbringing: t.upbringing(),
    });
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
    skills,
    regimes,
    techniques,
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
    regimeId: w.regimeId() ?? "",
    regimeName: w.regimeName() ?? "",
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
    buildingsRev: Number(s.buildingsRev()),
    pathsRev: Number(s.pathsRev()),
    marketsRev: Number(s.marketsRev()),
    firmsRev: Number(s.firmsRev()),
    wealthRev: Number(s.wealthRev()),
    knowledgeRev: Number(s.knowledgeRev()),
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
  [W.SpanKind.Firm]: "firm",
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
      holder: Number(x.holder()),
      holderSettlement: Number(x.holderSettlement()),
      leaseUntilMinute: Number(x.leaseUntilMinute()),
      leaseShare: x.leaseShare(),
    });
  }
  return { rev: Number(f.rev()), fields: out };
}

function vec2List(n: number, at: (i: number, v: W.Vec2) => W.Vec2 | null): [number, number][] {
  const out: [number, number][] = [];
  const v = new W.Vec2();
  for (let i = 0; i < n; i++) {
    const p = at(i, v);
    if (p) out.push([p.x(), p.y()]);
  }
  return out;
}

function buildings(f: W.Buildings): { rev: number; buildings: BuildingInfo[] } {
  const out: BuildingInfo[] = [];
  const info = new W.BuildingInfo();
  const v = new W.Vec2();
  for (let i = 0; i < f.buildingsLength(); i++) {
    const x = f.buildings(i, info);
    if (!x) continue;
    const centre = x.centre(v);
    const [cx, cy] = [centre?.x() ?? 0, centre?.y() ?? 0];
    const door = x.door(v);
    const doorAt: [number, number] = [door?.x() ?? cx, door?.y() ?? cy];
    const min = x.plotMin(v);
    const minAt = min ? [min.x(), min.y()] : null;
    const size = x.plotSize(v);
    const plot =
      minAt && size ? { x: minAt[0]!, y: minAt[1]!, w: size.x(), h: size.y() } : null;
    out.push({
      id: Number(x.id()),
      household: Number(x.household()),
      settlement: Number(x.settlement()),
      program: x.program() ?? "",
      x: cx,
      y: cy,
      radiusM: x.radiusM(),
      roofRadiusM: x.roofRadiusM(),
      door: doorAt,
      doorDir: x.doorDir(),
      stage: x.stage(),
      stageName: x.stageName() ?? "",
      progress: x.progress(),
      roofed: x.roofed(),
      outline: vec2List(x.outlineLength(), (k, p) => x.outline(k, p)),
      posts: vec2List(x.postsLength(), (k, p) => x.posts(k, p)),
      plot,
      floorM2: x.floorM2(),
      sleeps: x.sleeps(),
      startedMinute: Number(x.startedMinute()),
      status: x.status() ?? "",
    });
  }
  return { rev: Number(f.rev()), buildings: out };
}

/** Trail bits, 64 cells a word, as one byte a cell (1 = trail). */
function trailMask(t: W.WornTile, cells: number): Uint8Array {
  const out = new Uint8Array(cells);
  for (let k = 0; k < t.trailLength(); k++) {
    const word = t.trail(k) ?? 0n;
    const lo = Number(word & 0xffffffffn);
    const hi = Number(word >> 32n);
    for (let b = 0; b < 32; b++) {
      if ((lo >>> b) & 1) out[k * 64 + b] = 1;
      if ((hi >>> b) & 1) out[k * 64 + 32 + b] = 1;
    }
  }
  return out;
}

function paths(f: W.Paths): PathsInfo {
  const tileCells = f.tileCells();
  const cells = tileCells * tileCells;
  const worn: WornTile[] = [];
  const tile = new W.WornTile();
  for (let i = 0; i < f.wornLength(); i++) {
    const t = f.worn(i, tile);
    if (!t) continue;
    // Copied: the view points into a buffer the client reuses.
    const wear = new Uint8Array(t.wearArray() ?? new Uint8Array(cells));
    worn.push({ index: t.index(), wear, trail: trailMask(t, cells) });
  }
  const trails: TrailInfo[] = [];
  const info = new W.TrailInfo();
  for (let i = 0; i < f.trailsLength(); i++) {
    const t = f.trails(i, info);
    if (!t) continue;
    trails.push({
      points: vec2List(t.pointsLength(), (k, p) => t.points(k, p)),
      wear: t.wear(),
      lengthM: t.lengthM(),
    });
  }
  return { rev: Number(f.rev()), tilesX: f.tilesX(), tileCells, worn, trails };
}

function offerList(n: number, at: (i: number, o: W.OfferInfo) => W.OfferInfo | null): OfferInfo[] {
  const out: OfferInfo[] = [];
  const offer = new W.OfferInfo();
  for (let i = 0; i < n; i++) {
    const o = at(i, offer);
    if (!o) continue;
    out.push({
      household: Number(o.household()),
      householdName: o.householdName() ?? "",
      good: o.good(),
      payment: o.payment(),
      price: o.price(),
      units: o.units(),
      firm: o.firm(),
    });
  }
  return out;
}

function marketInfo(m: W.MarketInfo): MarketInfo {
  const goods: MarketGood[] = [];
  const line = new W.MarketGood();
  for (let i = 0; i < m.goodsLength(); i++) {
    const g = m.goods(i, line);
    if (!g) continue;
    goods.push({
      good: g.good(),
      offered: g.offered(),
      sellers: g.sellers(),
      sold: g.sold(),
      unmet: g.unmet(),
      unmetWorthH: g.unmetWorthH(),
      acceptance: g.acceptance(),
      lastPayment: g.lastPayment(),
      lastPrice: g.lastPrice(),
      workshops: g.workshops(),
    });
  }
  const offers = offerList(m.offersLength(), (i, o) => m.offers(i, o));
  const recent: TradeInfo[] = [];
  const trade = new W.TradeInfo();
  for (let i = 0; i < m.recentLength(); i++) {
    const t = m.recent(i, trade);
    if (!t) continue;
    recent.push({
      minute: Number(t.minute()),
      seller: Number(t.seller()),
      buyer: Number(t.buyer()),
      good: t.good(),
      units: t.units(),
      payment: t.payment(),
      paid: t.paid(),
      sale: t.sale(),
      text: t.text() ?? "",
      sellerFirm: t.sellerFirm(),
    });
  }
  const history: MonthOfTrade[] = [];
  const month = new W.MonthOfTrade();
  for (let i = 0; i < m.historyLength(); i++) {
    const h = m.history(i, month);
    if (!h) continue;
    history.push({
      month: h.month(),
      good: h.good(),
      trades: h.trades(),
      units: h.units(),
      paidH: h.paidH(),
    });
  }
  return {
    settlement: Number(m.settlement()),
    settlementName: m.settlementName() ?? "",
    money: m.money(),
    summary: m.summary() ?? "",
    trades: m.trades(),
    memoryDays: m.memoryDays(),
    goods,
    offers,
    recent,
    history,
  };
}

function markets(f: W.Markets): { rev: number; markets: MarketInfo[] } {
  const out: MarketInfo[] = [];
  const info = new W.MarketInfo();
  for (let i = 0; i < f.marketsLength(); i++) {
    const m = f.markets(i, info);
    if (m) out.push(marketInfo(m));
  }
  return { rev: Number(f.rev()), markets: out };
}

const BOOK_KINDS: Record<number, BookKind> = {
  [W.BookKind.PutIn]: "put-in",
  [W.BookKind.Drawn]: "drawn",
  [W.BookKind.Made]: "made",
  [W.BookKind.Used]: "used",
  [W.BookKind.Sold]: "sold",
  [W.BookKind.Paid]: "paid",
  [W.BookKind.Wages]: "wages",
  [W.BookKind.Lost]: "lost",
};

function firmBrief(f: W.FirmBrief): FirmBrief {
  return {
    id: Number(f.id()),
    name: f.name() ?? "",
    owner: Number(f.owner()),
    ownerName: f.ownerName() ?? "",
    settlement: Number(f.settlement()),
    settlementName: f.settlementName() ?? "",
    foundedMinute: Number(f.foundedMinute()),
    open: f.open(),
    closedMinute: Number(f.closedMinute()),
    closedWhy: f.closedWhy() ?? "",
    lines: Array.from(f.linesArray() ?? []),
    hiringH: f.hiringH(),
    record: f.record() ?? "",
  };
}

function firms(f: W.Firms): { rev: number; firms: FirmBrief[] } {
  const out: FirmBrief[] = [];
  const brief = new W.FirmBrief();
  for (let i = 0; i < f.firmsLength(); i++) {
    const b = f.firms(i, brief);
    if (b) out.push(firmBrief(b));
  }
  return { rev: Number(f.rev()), firms: out };
}

function firmInfo(f: W.FirmInfo): FirmInfo {
  const brief = f.brief();
  if (!brief) throw new Error("a workshop's page came without the workshop");
  const stores: StoreLine[] = [];
  for (let k = 0; k < f.storesLength(); k++) {
    const line = f.stores(k);
    if (line) stores.push({ good: line.good(), kg: line.kg() });
  }
  const w = f.wage();
  const entries: BookEntry[] = [];
  const entry = new W.BookEntryInfo();
  for (let k = 0; k < f.entriesLength(); k++) {
    const e = f.entries(k, entry);
    if (!e) continue;
    entries.push({
      minute: Number(e.minute()),
      kind: BOOK_KINDS[e.kind()] ?? "unknown",
      good: e.good(),
      amount: e.amount(),
      other: Number(e.other()),
      text: e.text() ?? "",
    });
  }
  const months: MonthStatement[] = [];
  const month = new W.MonthStatement();
  const line = new W.BookLine();
  for (let k = 0; k < f.monthsLength(); k++) {
    const m = f.months(k, month);
    if (!m) continue;
    const lines: BookLine[] = [];
    for (let i = 0; i < m.linesLength(); i++) {
      const l = m.lines(i, line);
      if (l) lines.push({ kind: BOOK_KINDS[l.kind()] ?? "unknown", good: l.good(), amount: l.amount() });
    }
    months.push({
      month: m.month(),
      lines,
      ownerH: m.ownerH(),
      hiredH: m.hiredH(),
      incomeH: m.incomeH(),
      costsH: m.costsH(),
      stockH: m.stockH(),
    });
  }
  return {
    brief: firmBrief(brief),
    founder: Number(f.founder()),
    founderName: f.founderName() ?? "",
    ownerSinceMinute: Number(f.ownerSinceMinute()),
    lastSaleMinute: Number(f.lastSaleMinute()),
    stores,
    offers: offerList(f.offersLength(), (i, o) => f.offers(i, o)),
    wage: w
      ? {
          activity: w.activity(),
          pay: w.pay(),
          perHour: w.perHour(),
          hourH: w.hourH(),
          hours: w.hours(),
          taken: w.taken(),
          text: w.text() ?? "",
        }
      : null,
    entries,
    months,
  };
}

function wealthSpread(s: W.WealthSpread): WealthSpread {
  return {
    year: Number(s.year()),
    households: s.households(),
    people: s.people(),
    giniGoods: s.giniGoods(),
    giniHeld: s.giniHeld(),
    giniWorked: s.giniWorked(),
    giniFloor: s.giniFloor(),
    topTenthGoods: s.topTenthGoods(),
    holdingNone: s.holdingNone(),
    workingNone: s.workingNone(),
    goodsHPerHead: s.goodsHPerHead(),
    workedHaPerHead: s.workedHaPerHead(),
    floorM2PerHouse: s.floorM2PerHouse(),
    commonHa: s.commonHa(),
  };
}

function wealth(w: W.Wealth): WealthInfo {
  const settlements: SettlementWealth[] = [];
  for (let i = 0; i < w.settlementsLength(); i++) {
    const s = w.settlements(i);
    if (!s) continue;
    const households: HouseholdWealth[] = [];
    for (let k = 0; k < s.householdsLength(); k++) {
      const h = s.households(k);
      if (!h) continue;
      households.push({
        household: Number(h.household()),
        name: h.name() ?? "",
        members: h.members(),
        heldHa: h.heldHa(),
        workedHa: h.workedHa(),
        letHa: h.letHa(),
        rentedHa: h.rentedHa(),
        goodsH: h.goodsH(),
        floorM2: h.floorM2(),
      });
    }
    const history: WealthSpread[] = [];
    for (let k = 0; k < s.historyLength(); k++) {
      const y = s.history(k);
      if (y) history.push(wealthSpread(y));
    }
    const now = s.now();
    settlements.push({
      settlement: Number(s.settlement()),
      name: s.name() ?? "",
      now: now ? wealthSpread(now) : null,
      households,
      history,
    });
  }
  return { rev: Number(w.rev()), regimeName: w.regimeName() ?? "", settlements };
}

function personRefs(n: number, at: (k: number) => W.PersonRef | null): PersonRef[] {
  const out: PersonRef[] = [];
  for (let k = 0; k < n; k++) {
    const r = at(k);
    if (r) out.push({ id: Number(r.id()), name: r.name() ?? "", ageYears: r.ageYears() });
  }
  return out;
}

function knowledge(w: W.Knowledge): KnowledgeInfo {
  const settlements: SettlementKnowledge[] = [];
  for (let i = 0; i < w.settlementsLength(); i++) {
    const s = w.settlements(i);
    if (!s) continue;
    const techniques: TechniqueHere[] = [];
    for (let k = 0; k < s.techniquesLength(); k++) {
      const t = s.techniques(k);
      if (!t) continue;
      techniques.push({
        technique: t.technique(),
        known: t.known(),
        knowers: personRefs(t.knowersLength(), (j) => t.knowers(j)),
        learners: personRefs(t.learnersLength(), (j) => t.learners(j)),
        heard: personRefs(t.heardLength(), (j) => t.heard(j)),
        practisedLastYear: t.practisedLastYear(),
        status: t.status() ?? "",
        history: Array.from({ length: t.historyLength() }, (_, j) => t.history(j) ?? ""),
      });
    }
    settlements.push({ settlement: Number(s.settlement()), name: s.name() ?? "", techniques });
  }
  return { rev: Number(w.rev()), settlements };
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
        left: l.left(),
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
  const skills: SkillLine[] = [];
  for (let k = 0; k < p.skillsLength(); k++) {
    const line = p.skills(k);
    if (line) skills.push({ skill: line.skill(), level: line.level() });
  }
  const knows: KnowLine[] = [];
  for (let k = 0; k < p.knowsLength(); k++) {
    const line = p.knows(k);
    if (!line) continue;
    const state = line.state();
    knows.push({
      technique: line.technique(),
      state: state === 2 ? "known" : state === 1 ? "learning" : "heard",
      hours: line.hours(),
      learnH: line.learnH(),
      sinceMinute: Number(line.sinceMinute()),
      source: line.source() ?? "",
      sourcePerson: Number(line.sourcePerson()),
      usedMinute: Number(line.usedMinute()),
    });
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
    householdReadyDays: p.householdReadyDays(),
    householdWaterDays: p.householdWaterDays(),
    householdFuelDays: p.householdFuelDays(),
    stores,
    skills,
    knows,
    decisions,
    traits: Array.from(p.traitsArray() ?? []),
    x: pos?.x() ?? 0,
    y: pos?.y() ?? 0,
    partner: Number(p.partner()),
    family: Array.from({ length: p.familyLength() }, (_, k) => p.family(k) ?? ""),
    leftMinute: Number(p.leftMinute()),
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
    case W.ResponseBody.Buildings: {
      const f = r.body(new W.Buildings()) as W.Buildings | null;
      if (!f) break;
      return { kind: "buildings", ...buildings(f) };
    }
    case W.ResponseBody.Paths: {
      const f = r.body(new W.Paths()) as W.Paths | null;
      if (!f) break;
      return { kind: "paths", paths: paths(f) };
    }
    case W.ResponseBody.Markets: {
      const f = r.body(new W.Markets()) as W.Markets | null;
      if (!f) break;
      return { kind: "markets", ...markets(f) };
    }
    case W.ResponseBody.Firms: {
      const f = r.body(new W.Firms()) as W.Firms | null;
      if (!f) break;
      return { kind: "firms", ...firms(f) };
    }
    case W.ResponseBody.FirmInfo: {
      const f = r.body(new W.FirmInfo()) as W.FirmInfo | null;
      if (!f) break;
      return { kind: "firm", firm: firmInfo(f) };
    }
    case W.ResponseBody.Wealth: {
      const f = r.body(new W.Wealth()) as W.Wealth | null;
      if (!f) break;
      return { kind: "wealth", wealth: wealth(f) };
    }
    case W.ResponseBody.Knowledge: {
      const f = r.body(new W.Knowledge()) as W.Knowledge | null;
      if (!f) break;
      return { kind: "knowledge", knowledge: knowledge(f) };
    }
    default:
      break;
  }
  throw new Error(`unexpected response body ${r.bodyType()}`);
}
