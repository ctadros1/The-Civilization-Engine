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
  /** The Accelerated speeds as multiples of 1x: 60, 600 and Max (Infinity). Wire 1.23. */
  acceleratedMultipliers: number[];
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
  /** Wire 1.47 (M4c slice AJ): the ideologies, which the observer may tell someone of. */
  ideologies: IdeologyInfo[];
}

/** An ideology content names (wire 1.47). */
export interface IdeologyInfo {
  id: string;
  name: string;
  legitimacy: string;
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
  /** Simulated seconds per real second; Infinity for Max. */
  speed: number;
  /** How the kernel advances: by the minute, or a day at a time with frames only at midnight
   * (ADR-0011). Wire 1.23. */
  mode: "detailed" | "accelerated";
  /** Today's weather on the valley floor (wire 1.24, ADR-0012); null from an older host. */
  weather: DayWeather | null;
}

/** A day's weather on the valley floor (wire 1.24, M3c slice U). */
export interface DayWeather {
  /** Rain and snow, mm of water. */
  precipMm: number;
  meanC: number;
  minC: number;
  maxC: number;
  /** Snow lying, mm of water. */
  snowMm: number;
  /** The soil water under the wild cover, as a share of what the soil holds. */
  soil: number;
  /** Rendered by the kernel: "6 °C, light rain; snow lying". */
  words: string;
  /** The lowest height at which snow lies, metres; Infinity when it lies nowhere, or from a
   * host before wire 1.25. */
  snowLineM: number;
}

/** One month's weather on the valley floor, beside what the month usually brings (wire 1.24). */
export interface WeatherMonth {
  /** The world's first year is 1. */
  year: number;
  /** 0 for January. */
  month: number;
  /** Days recorded: fewer than the month has while it is under way. */
  days: number;
  precipMm: number;
  usualMm: number;
  wetDays: number;
  meanC: number;
  usualC: number;
  minC: number;
  maxC: number;
  frostDays: number;
  snowDays: number;
  /** The soil water under the wild cover, on average, as a share of what the soil holds. */
  soil: number;
}

/** Every month's weather (wire 1.24). */
export interface WeatherReport {
  rev: number;
  /** The height the weather is for, metres. */
  heightM: number;
  /** The landscape's mean annual precipitation, mm. */
  annualMm: number;
  today: DayWeather | null;
  /** Oldest first; the last is the month under way. */
  months: WeatherMonth[];
  /** Today's month, 0 for January. */
  month: number;
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
  /** How it was founded, in the kernel's words (wire 1.49, ADR-0018 §1). */
  founding: string;
  /** Its accounts over the past year, in the kernel's words; empty when nothing changed. */
  year: string;
  /** When its last resident died or left; -1 while it is lived in. */
  abandonedMinute: number;
  /** Wire 1.51 (M5a slice AM): visits and marriages between it and other settlements last
   * year and this year so far, in the kernel's words; empty when there were none. */
  contacts: string;
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
  /** Wire 1.19: changes whenever a deposit is laid down, found or dug from (0 = no deposits). */
  depositsRev: number;
  /** Wire 1.20: changes whenever an earthwork is begun or advanced (0 = none). */
  earthworksRev: number;
  /** Wire 1.24: changes each day lived (0 = no world). Fetch the weather with GetWeather. */
  weatherRev: number;
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
  /** Wire 1.24: the share of the water its growing crop needed that it has had (-1 before it has
   * needed any), and its root zone's water as a share of what the soil holds. */
  waterHad: number;
  soilWater: number;
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

/** How one component group of a building stands (wire 1.17). */
export type GroupState = "sound" | "symptom" | "failed";

/** How a building stands as a whole (wire 1.17). */
export type BuildingState = "standing" | "damaged" | "ruin";

/** The condition of one component group of a building (wire 1.17, ADR-0009 §4). */
export interface GroupInfo {
  /** Its semantic id in the building's expansion, and its kind in words: "posts", "covering". */
  id: number;
  kind: string;
  /** A share of what sound members of its sizes carry, drawn once from its builders' skill. */
  quality: number;
  /** The share of its members' section lost, or of a covering worn, 0–1. */
  loss: number;
  state: GroupState;
  installedMinute: number;
  repairedMinute: number;
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
  /** Wire 1.14 (M3b slice O): the grammar that expands it, "hut" or "frame" ("" from an older
   * host), and what it is for: "dwelling", "store" or "work". */
  grammar: string;
  purpose: string;
  /** Length and width between the wall lines, metres (a hut: its diameter both ways), and the
   * direction the length runs, radians from east toward south. */
  size: [number, number];
  angle: number;
  storeys: number;
  bays: number;
  /** The bays floored as a loft, a bit each. */
  loftBays: number;
  /** A gabled roof's corners and its ridge's two ends, metres; both empty for a hut. */
  roofOutline: [number, number][];
  ridge: [number, number][];
  /** Height of the roof's apex or ridge, metres. */
  apexM: number;
  /** Floor by use, square metres: living, store and work. */
  floorByUse: number[];
  /** Goods it can hold under its roof, kilograms: on a raised floor, in lofts, on other floors. */
  storageKg: number[];
  workPlaces: number;
  /** Wire 1.15: what its household keeps in it now, kilograms by kind of room, and in words
   * rendered by the kernel: "loft over 1 bay: 1.2 t of 1.9 t, mostly grain". */
  storedKg: number[];
  stored: string;
  /** Wire 1.16: the firm it is the workshop of (0 for none), and its name: "Wren's sickle
   * workshop". */
  firm: number;
  firmName: string;
  /** Wire 1.17 (M3b slice P): how it stands; what it shows in words, worst first ("the thatch
   * leaks; rot at the posts' foot"; "" while nothing shows); the share of its roof that leaks,
   * 0–1; each of its component groups in place; and the upkeep under way in words ("mending the
   * covering, 40% done"; "" for none). */
  state: BuildingState;
  symptoms: string;
  leak: number;
  groups: GroupInfo[];
  upkeep: string;
  /** Wire 1.22 (M3b slice R): how it was built, in words rendered by the kernel, with the
   * building its household's taste followed ("roof pitched 49°, walls 1.9 m to the eaves, after
   * Bo's hut"); and that building (0 for none; it may since be gone). */
  style: string;
  styleFrom: number;
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
  /** Wire 1.22 (M3b slice R): how their household would build, in words rendered by the kernel
   * ("roofs pitched 48°, walls 1.9 m to the eaves, eaves 0.5 m out; admiring Bo's hut"; "" for
   * the dead), and the building that moved its taste most (0 for none). */
  householdTaste: string;
  householdAdmired: number;
  /** Wire 1.26 (M4a slice Y): their strongest ties, the most salient first (the living only). */
  ties: TieLine[];
  /** Their standing in their settlement as last worked out (null before the first time). */
  standing: StandingLine | null;
  /** Wire 1.34 (M4c slice AE, ADR-0016): the grievances they hold, the most keenly felt first,
   * and what they have heard that is still news, the latest first (the living only). */
  grievances: GrievanceLine[];
  heard: HeardLine[];
  /** Wire 1.36 (M4c slice AG): where they stand on each question content names. */
  positions: PositionLine[];
  /** Wire 1.37 (M4c slice AG): what they hold of each norm content names. */
  norms: NormLine[];
  /** Wire 1.38 (M4c slice AG): what they hold of each value content names. */
  values: ValueLine[];
  /** Wire 1.39 (M4c slice AG): the ideologies they hold. */
  ideologies: IdeologyLine[];
  /** Wire 1.40 (M4c slice AH): the faction they belong to, if any. */
  faction: FactionLine | null;
  /** Wire 1.47 (M4c slice AJ, ADR-0016 §5): true claims their settlement's word holds that they
   * have not heard, which the observer may whisper to them, newest first. */
  news: NewsLine[];
  /** Wire 1.47: the observer's interventions that reached them, newest first, with what came of
   * each, in the kernel's words. */
  influences: InfluenceLine[];
  /** Wire 1.49 (ADR-0018 §2): where they have lived, oldest first, in the kernel's words. */
  residence: string[];
  /** Wire 1.50 (ADR-0018 §4): the other settlements their household knows, and how, in the
   * kernel's words ("Ashford: told of it by Wren in spring of year 2"). */
  places: string[];
}

/** A claim the observer may whisper (wire 1.47). */
export interface NewsLine {
  claim: number;
  what: string;
}

/** One recorded influence and what came of it (wire 1.47, research 15-05 §6). */
export interface InfluenceLine {
  id: number;
  /** 0 a whisper, 1 an ideology told of. */
  kind: number;
  minute: number;
  what: string;
}

/** The faction someone belongs to (wire 1.40, ADR-0017 §2, §6). */
export interface FactionLine {
  faction: number;
  /** "Mira's faction", and what it stands against ("the gathering"). */
  name: string;
  against: string;
  organizer: number;
  organizerName: string;
  /** The start of the day they joined. */
  sinceMinute: number;
  /** Why they belong, as they last reviewed it, in the kernel's words. */
  why: string;
}

/** An ideology someone holds (wire 1.39, ADR-0016 §4). */
export interface IdeologyLine {
  /** What it is called and its legitimacy story, in the kernel's words. */
  name: string;
  legitimacy: string;
  /** When they took it up, and from whom (0 and "": they brought it with them). */
  sinceMinute: number;
  from: number;
  fromName: string;
  /** Wire 1.47: the observer's intervention they took it up by (0: none). */
  influence: number;
}

/** What someone holds of a value (wire 1.38, ADR-0016 §4). */
export interface ValueLine {
  /** The value, and what they hold of it, in the kernel's words. */
  name: string;
  words: string;
  /** -1 (less than most) to 1 (more than most). */
  v: number;
}

/** What someone holds of a norm (wire 1.37, ADR-0016 §4). */
export interface NormLine {
  /** What it says, how far they hold it and what they believe others do, in the kernel's words. */
  statement: string;
  holds: string;
  believes: string;
  /** 0-1: their endorsement; the share of households they believe abide; where others' doing it
   * starts to move them; how far it moves them now. */
  endorse: number;
  expect: number;
  threshold: number;
  activation: number;
  /** Accounts of what households did that they have taken in. */
  heard: number;
}

/** Where someone stands on a question (wire 1.36, ADR-0016 §4). */
export interface PositionLine {
  /** The question and where they stand, in the kernel's words. */
  question: string;
  lean: string;
  /** 0 against to 1 for; what their household's own lot makes of it; how much it matters now. */
  x: number;
  anchor: number;
  salience: number;
  /** What they have heard said of it at the hearth and taken in. */
  heard: number;
}

/** A grievance someone holds (wire 1.34, ADR-0016 §2). */
export interface GrievanceLine {
  /** 0 subsistence, 1 extraction, 2 treatment, 3 a collective claim. */
  issue: number;
  /** What it is over and whom it is held against, in words rendered by the kernel ("food when
   * their household was short"; "the gathering"). */
  over: string;
  blamed: string;
  /** The law whose terms it broke (0 = none). */
  law: number;
  /** The harm, and what of it nothing has yet made good, days of their household's food. */
  harmDays: number;
  unresolvedDays: number;
  /** How keenly it is felt now, 0-1: it fades, and only a reminder raises it. */
  activation: number;
  madeMinute: number;
  raisedMinute: number;
  /** What last raised it, in words rendered by the kernel. */
  reason: string;
}

/** A claim someone has heard (wire 1.34, ADR-0016 §3). */
export interface HeardLine {
  /** 0 a gathering called, 1 a grievance told. */
  kind: number;
  /** What it says, in words rendered by the kernel. */
  what: string;
  /** Who told them (0 = nobody: it began with them), and the one the account began with. */
  from: number;
  fromName: string;
  origin: number;
  firstMinute: number;
  lastMinute: number;
  /** Wire 1.47: the observer's intervention that placed it in their hearing (0: none). */
  influence: number;
}

/** One person's view of another (wire 1.26, ADR-0014). */
export interface TieLine {
  person: number;
  name: string;
  /** 0-1 each. */
  familiarity: number;
  warmth: number;
  /** Esteem by domain, in StandingInfo.domains' order: good acts remembered less bad ones. */
  esteem: number[];
  /** Hours of help received from them less help given to them. */
  helpH: number;
  /** Why, in words rendered by the kernel ("gave their household food 3 times, last in May of
   * year 4"). */
  reason: string;
  /** Whether they hold a tie back. */
  mutual: boolean;
}

/** An adult's standing in their settlement (wire 1.26). */
export interface StandingLine {
  person: number;
  name: string;
  household: number;
  /** The esteem the settlement's other adults hold for them, summed, by domain. */
  esteem: number[];
  /** How many of the settlement's adults count them among those they esteem most. */
  influence: number;
  /** Considers the settlement's affairs weekly; it grants nothing. */
  notable: boolean;
}

/** A settlement's standing: its notables first, then those most esteemed. */
export interface SettlementStanding {
  settlement: number;
  name: string;
  adults: number;
  rows: StandingLine[];
}

/** Every settlement's standing as worked out on `minute` (0 before the first time). */
export interface StandingInfo {
  minute: number;
  /** The domains' names, in the order of every `esteem`. */
  domains: string[];
  settlements: SettlementStanding[];
  /** Ties kept across the world, and how many were let go for want of room. */
  ties: number;
  letGo: number;
}

/** Where a member stood at a gathering (wire 1.27, ADR-0013 §3). */
export type StanceKind = "for" | "against" | "abstained";

/** One member's stance at a gathering, with what moved it and why in the kernel's words. */
export interface StanceLine {
  person: number;
  name: string;
  stance: StanceKind;
  /** Their household's forecast of the law, and what their regard for its sponsor added, points. */
  gain: number;
  regard: number;
  why: string;
  /** Wire 1.36: what talk at the hearth had moved them from their household's lot, points. */
  opinion: number;
  /** Wire 1.38: what the law does to what they hold dear, points. */
  values: number;
}

/**
 * Where a law stands ("lapsed", wire 1.28: the one it named died or left; "carried", wire 1.44: a
 * repeal that passed, ending the law it named).
 */
export type LawStatus = "proposed" | "in force" | "rejected" | "lapsed" | "superseded" | "carried";

/** How a gathering decided. */
export type LawOutcome = "passed" | "failed" | "tied" | "no quorum";

/** A law and its whole history (wire 1.27, ADR-0013 §3). Sentences are the kernel's. */
export interface LawLine {
  id: number;
  /** "a common store, taking a tenth of each harvest". */
  what: string;
  policy: string;
  levyShare: number;
  reliefDays: number;
  status: LawStatus;
  sponsor: number;
  sponsorName: string;
  proposedMinute: number;
  /** The issue it answered: "food would not last until the harvest". */
  issue: string;
  /** The start of the day the gathering meets. */
  meetsMinute: number;
  /** 0 before it was decided. */
  decidedMinute: number;
  outcome: LawOutcome | null;
  /** "agreed: 20 for, 3 against; 23 of 24 adults came, 6 needed" ("" before). */
  decision: string;
  eligible: number;
  quorum: number;
  stances: StanceLine[];
  /** Living people who know it. */
  known: number;
  complied: number;
  couldNot: number;
  /** Levies kept back unannounced; `refused` (wire 1.42) counts those kept back openly in a
   * faction's refusal. */
  evaded: number;
  refused: number;
  unaware: number;
  leviedKg: number;
  withheldKg: number;
  relieved: number;
  reliefKg: number;
  unanswered: number;
  /** Wire 1.28: the one it names (who keeps the store; 0 for none), and their name. */
  holder: number;
  holderName: string;
  /** Wire 1.33: for a curfew, the times someone broke it knowing of it, and not. */
  broken: number;
  brokenUnaware: number;
}

/** A settlement's polity (wire 1.27): its custom, members, store, gathering called and laws. */
export interface PolityLine {
  polity: number;
  settlement: number;
  name: string;
  foundedMinute: number;
  /** The body in the kernel's words. */
  custom: string;
  members: number;
  /** What the store holds in words ("grain 322 kg", "nothing"), and its food in kilograms. */
  store: string;
  storeKg: number;
  /** Every law proposed there, newest first. */
  laws: LawLine[];
  /** The gathering called: its law (0 for none), the day it meets and those come so far. */
  gatheringLaw: number;
  gatheringMinute: number;
  gatheringPresent: number;
  /** Wire 1.28: its offices and who holds them, in the kernel's words. */
  offices: string[];
  /**
   * Wire 1.29: what it would be called, worked out afterwards from its history and read by
   * nothing in the world (ADR-0013 §6): a name, what qualifies it, why, and how much evidence
   * stands behind it (0-1, uncalibrated).
   */
  label: string;
  labelModifiers: string[];
  labelWhy: string[];
  labelConfidence: number;
  /** Wire 1.31: the cases the gathering called is to hear, in the kernel's words. */
  gatheringCases: string[];
  /** Wire 1.35 (M4c slice AF): every version of its custom, oldest first, in the kernel's
   * words, and how many its body admits now (`members` counts every adult). */
  customHistory: string[];
  bodyMembers: number;
  /** Wire 1.40 (M4c slice AH): its factions, those with members first, in the kernel's words. */
  factions: string[];
  /** Wire 1.41 (M4c slice AH): its petitions, newest first, in the kernel's words. */
  petitions: string[];
  /** Wire 1.42 (M4c slice AH): its refusals of a levy, newest first, in the kernel's words. */
  refusals: string[];
  /** Wire 1.43 (M4c slice AI): its revolts, newest first, in the kernel's words. */
  revolts: string[];
  /** Wire 1.45 (M4c slice AI, step three): its coups, newest first, in the kernel's words. */
  coups: string[];
}

/** Every settlement's polity at `minute` (wire 1.27). */
export interface GovernmentInfo {
  minute: number;
  polities: PolityLine[];
}

/** How an attempt to take ended (wire 1.30). */
export type TakingOutcome = "taken" | "turned back" | "fled";

/**
 * What happened (the truth layer, ADR-0015 §1): one attempt to take from a household's store.
 * Kept apart from what anyone believes of it, which is a {@link KnownLine} with the same number.
 */
export interface IncidentLine {
  id: number;
  minute: number;
  /** 0 when the household taken from has none. */
  settlement: number;
  actor: number;
  actorName: string;
  /** The household taken from, and its name for the one who stands for it. */
  target: number;
  targetName: string;
  outcome: TakingOutcome;
  /** What was carried off ("12 kg of grain"), or how it ended, in the kernel's words. */
  what: string;
  kcal: number;
  /** Who saw the taker at it. */
  seenBy: string[];
  /**
   * Wire 1.32: what the one who keeps the watch did with it, if they saw it, in the kernel's
   * words, or "". The truth: nobody in the world reads it.
   */
  watch: string;
}

/** What the living believe of one incident, and what was chosen (the knowledge layer). */
export interface KnownLine {
  incident: number;
  /** People who believe they know who took, and people who know only of a loss. */
  knowTaker: number;
  knowLoss: number;
  /** Distinct first-hand accounts behind what they believe. */
  sources: number;
  /** Someone of the household taken from knows who took. */
  victimKnows: boolean;
  /** What the household taken from chose, in words, or "". */
  response: string;
  /** Where what is owed stands, in words, or "". */
  owed: string;
  /** Wire 1.31: the case brought for it and how the gathering decided, in words, or "". */
  case: string;
}

/** Takings at `minute` (wire 1.30): the most recent, newest first, and totals over all. */
export interface OrderInfo {
  minute: number;
  incidents: IncidentLine[];
  known: KnownLine[];
  attempts: number;
  takings: number;
  seen: number;
  knownToVictims: number;
  demands: number;
  met: number;
  refused: number;
  /** Asks refused because the giver believed the asker took, since the world was loaded. */
  refusals: number;
  /** Wire 1.31: cases brought before the gathering, and how they ended. */
  cases: number;
  found: number;
  notFound: number;
  unheard: number;
  /**
   * Wire 1.46 (M4c slice AI, step four): watchers come to take what a refused finding owed,
   * newest first, in the kernel's words.
   */
  encounters: string[];
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
  /** Wire 1.16: floor under all a household's roofs (homes, stores, workshops), square metres,
   * and room for goods under them, kilograms, a household with a roof. */
  roofedM2PerHouse: number;
  storageKgPerHouse: number;
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
  /** Floor area of its home. */
  floorM2: number;
  /** Wire 1.16: floor under all its roofs, square metres, and room for goods under them,
   * kilograms. */
  roofedM2: number;
  storageKg: number;
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
  /**
   * Wire 1.18 (M3b slice P): how many times their usual strength builders there make its frame
   * buildings' joists and posts after the failures they have seen (1 as usual; ADR-0009 §6), and
   * what they have seen in words, rendered by the kernel ("" when nothing is built by it there).
   */
  caution: number;
  trust: string;
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

/** A deposit in the ground (wire 1.19, M3b slice Q; ADR-0010 §1). */
export interface DepositInfo {
  id: number;
  /** An index into Welcome.goods. */
  good: number;
  /** Its centre, metres from the map's north-west corner, and its radius. */
  x: number;
  y: number;
  radiusM: number;
  /** It shows at the surface; otherwise it lies under `coverM` of ground. */
  exposed: boolean;
  coverM: number;
  thicknessM: number;
  /** How much of what is dug is fit for use, 0 to 1. */
  quality: number;
  leftKg: number;
  takenKg: number;
  /** The settlements that know it, and how each came to, in the kernel's words. */
  knownBy: number[];
  finds: string[];
}

export interface DepositsInfo {
  rev: number;
  deposits: DepositInfo[];
}

/** An earthwork (wire 1.20, M3b slice Q; ADR-0010 §2): a platform levelling a plot, a pit dug for
 * a deposit's goods, or the spoil heap beside a pit. */
export interface EarthworkInfo {
  id: number;
  /** What it is: 0 a platform; wire 1.21: 1 a pit, 2 a spoil heap. */
  kind: number;
  /** The rectangle it levels, metres from the map's north-west corner. */
  x: number;
  y: number;
  w: number;
  h: number;
  /** The level it is cut and filled to, metres of height, and its sides' run across per metre. */
  levelM: number;
  sideRun: number;
  /** Earth it cuts when done, cubic metres as it lay in the ground, and the share done, 0 to 1. */
  cutM3: number;
  done: number;
  household: number;
  /** The plot it levels and the building on it (0 for none). */
  plot: number;
  building: number;
  /** In the kernel's words: "the plot of Ada's hut, being levelled: 40% of 6.4 m³ cut and filled". */
  words: string;
  /** Wire 1.21: for a pit or its heap, the deposit the pit is dug for (0 for none). */
  deposit: number;
}

export interface EarthworksInfo {
  rev: number;
  works: EarthworkInfo[];
  /** Cells a side of a tile of the ground's changes, tiles across the map, and each changed tile's
   * index (row by row) and revision. */
  tileCells: number;
  tilesX: number;
  tiles: { index: number; rev: number }[];
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
  | { kind: "knowledge"; knowledge: KnowledgeInfo }
  | { kind: "deposits"; deposits: DepositsInfo }
  | { kind: "earthworks"; earthworks: EarthworksInfo }
  | { kind: "weather"; weather: WeatherReport }
  | { kind: "standing"; standing: StandingInfo }
  | { kind: "government"; government: GovernmentInfo }
  | { kind: "order"; order: OrderInfo };

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
  /** People in each further founding group (wire 1.49); 0 = the content's default. */
  neighbours?: number[];
  /** The founding groups know where each other camped (wire 1.50, ADR-0018 §6). */
  neighboursKnown?: boolean;
}): Uint8Array {
  const b = new flatbuffers.Builder(128);
  const preset = b.createString(args.presetId);
  const name = b.createString(args.name);
  const regime = b.createString(args.regimeId ?? "");
  const neighbours = W.NewWorld.createNeighboursVector(b, args.neighbours ?? []);
  const body = W.NewWorld.createNewWorld(
    b,
    args.seed,
    preset,
    args.sizeCells,
    name,
    args.bandSize ?? 0,
    regime,
    neighbours,
    args.neighboursKnown ?? false,
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

/**
 * The observer whispers a true claim to a living adult (god tool, M4c slice AJ, ADR-0016 §5):
 * `claim` is a number from their PersonInfo.news. A repeat refreshes it and adds nothing.
 */
export function whisper(person: number, claim: number): Uint8Array {
  const b = new flatbuffers.Builder(32);
  const body = W.Whisper.createWhisper(b, BigInt(person), claim);
  return command(b, W.CommandBody.Whisper, body);
}

/**
 * The observer tells a living adult of an ideology (god tool, M4c slice AJ, ADR-0016 §5):
 * `ideology` is an index into Welcome.ideologies. They weigh it as one heard of from no one.
 */
export function tellOfIdeology(person: number, ideology: number): Uint8Array {
  const b = new flatbuffers.Builder(32);
  const body = W.TellOfIdeology.createTellOfIdeology(b, BigInt(person), ideology);
  return command(b, W.CommandBody.TellOfIdeology, body);
}

/**
 * The observer sends an agitator (god tool, M4c slice AJ, ADR-0016 §5): one adult newcomer
 * holding an ideology (an index into Welcome.ideologies) arrives where a family would.
 */
export function sendAgitator(xM: number, yM: number, ideology: number): Uint8Array {
  const b = new flatbuffers.Builder(32);
  W.SendAgitator.startSendAgitator(b);
  W.SendAgitator.addAt(b, W.Vec2.createVec2(b, xM, yM));
  W.SendAgitator.addIdeology(b, ideology);
  return command(b, W.CommandBody.SendAgitator, W.SendAgitator.endSendAgitator(b));
}

/**
 * The observer blesses a living person, or with `curse` curses them, for `days` days, moving
 * their own draws for illness or accident and for finding things out by `share` of their way
 * (god tool, M4c slice AJ, ADR-0016 §5).
 */
export function bless(person: number, curse: boolean, days: number, share: number): Uint8Array {
  const b = new flatbuffers.Builder(32);
  const body = W.Bless.createBless(b, BigInt(person), curse, days, share);
  return command(b, W.CommandBody.Bless, body);
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

export function getDeposits(): Uint8Array {
  const b = new flatbuffers.Builder(16);
  W.GetDeposits.startGetDeposits(b);
  return query(b, W.QueryBody.GetDeposits, W.GetDeposits.endGetDeposits(b));
}

export function getEarthworks(): Uint8Array {
  const b = new flatbuffers.Builder(16);
  W.GetEarthworks.startGetEarthworks(b);
  return query(b, W.QueryBody.GetEarthworks, W.GetEarthworks.endGetEarthworks(b));
}

/** The observer lays down a deposit of `good` (a content id) at a point, metres (god tool). */
export function placeDeposit(x: number, y: number, good: string, radiusM: number, exposed: boolean): Uint8Array {
  const b = new flatbuffers.Builder(64);
  const id = b.createString(good);
  W.PlaceDeposit.startPlaceDeposit(b);
  W.PlaceDeposit.addAt(b, W.Vec2.createVec2(b, x, y));
  W.PlaceDeposit.addGood(b, id);
  W.PlaceDeposit.addRadiusM(b, radiusM);
  W.PlaceDeposit.addExposed(b, exposed);
  return command(b, W.CommandBody.PlaceDeposit, W.PlaceDeposit.endPlaceDeposit(b));
}

export function getKnowledge(): Uint8Array {
  const b = new flatbuffers.Builder(16);
  W.GetKnowledge.startGetKnowledge(b);
  return query(b, W.QueryBody.GetKnowledge, W.GetKnowledge.endGetKnowledge(b));
}

export function getStanding(): Uint8Array {
  const b = new flatbuffers.Builder(16);
  W.GetStanding.startGetStanding(b);
  return query(b, W.QueryBody.GetStanding, W.GetStanding.endGetStanding(b));
}

export function getGovernment(): Uint8Array {
  const b = new flatbuffers.Builder(16);
  W.GetGovernment.startGetGovernment(b);
  return query(b, W.QueryBody.GetGovernment, W.GetGovernment.endGetGovernment(b));
}

export function getOrder(): Uint8Array {
  const b = new flatbuffers.Builder(16);
  W.GetOrder.startGetOrder(b);
  return query(b, W.QueryBody.GetOrder, W.GetOrder.endGetOrder(b));
}

export function getWeather(): Uint8Array {
  const b = new flatbuffers.Builder(16);
  W.GetWeather.startGetWeather(b);
  return query(b, W.QueryBody.GetWeather, W.GetWeather.endGetWeather(b));
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
  const ideologies: IdeologyInfo[] = [];
  for (let i = 0; i < w.ideologiesLength(); i++) {
    const d = w.ideologies(i);
    if (!d) continue;
    ideologies.push({ id: d.id() ?? "", name: d.name() ?? "", legitimacy: d.legitimacy() ?? "" });
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
    acceleratedMultipliers: Array.from(w.acceleratedMultipliersArray() ?? []),
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
    ideologies,
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
          mode: clock.mode() === W.ClockMode.Accelerated ? "accelerated" : "detailed",
          weather: dayWeather(clock.weather()),
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
    depositsRev: Number(s.depositsRev()),
    earthworksRev: Number(s.earthworksRev()),
    weatherRev: Number(s.weatherRev()),
  };
}

function deposits(w: W.Deposits): DepositsInfo {
  const list: DepositInfo[] = [];
  for (let i = 0; i < w.depositsLength(); i++) {
    const d = w.deposits(i);
    if (!d) continue;
    list.push({
      id: Number(d.id()),
      good: d.good(),
      x: d.x(),
      y: d.y(),
      radiusM: d.radiusM(),
      exposed: d.exposed(),
      coverM: d.coverM(),
      thicknessM: d.thicknessM(),
      quality: d.quality(),
      leftKg: d.leftKg(),
      takenKg: d.takenKg(),
      knownBy: Array.from({ length: d.knownByLength() }, (_, j) => Number(d.knownBy(j) ?? 0n)),
      finds: Array.from({ length: d.findsLength() }, (_, j) => d.finds(j) ?? ""),
    });
  }
  return { rev: Number(w.rev()), deposits: list };
}

function earthworks(w: W.Earthworks): EarthworksInfo {
  const works: EarthworkInfo[] = [];
  for (let i = 0; i < w.worksLength(); i++) {
    const e = w.works(i);
    if (!e) continue;
    works.push({
      id: Number(e.id()),
      kind: e.kind(),
      x: e.x(),
      y: e.y(),
      w: e.w(),
      h: e.h(),
      levelM: e.levelM(),
      sideRun: e.sideRun(),
      cutM3: e.cutM3(),
      done: e.done(),
      household: Number(e.household()),
      plot: Number(e.plot()),
      building: Number(e.building()),
      words: e.words() ?? "",
      deposit: Number(e.deposit()),
    });
  }
  const tiles: { index: number; rev: number }[] = [];
  const t = new W.GroundTileRev();
  for (let i = 0; i < w.tilesLength(); i++) {
    const tile = w.tiles(i, t);
    if (tile) tiles.push({ index: tile.index(), rev: tile.rev() });
  }
  return { rev: Number(w.rev()), works, tileCells: w.tileCells(), tilesX: w.tilesX(), tiles };
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
      founding: t.founding() ?? "",
      year: t.year() ?? "",
      abandonedMinute: Number(t.abandonedMinute()),
      contacts: t.contacts() ?? "",
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
      waterHad: x.waterHad(),
      soilWater: x.soilWater(),
    });
  }
  return { rev: Number(f.rev()), fields: out };
}

function dayWeather(w: W.DayWeather | null): DayWeather | null {
  if (!w) return null;
  return {
    precipMm: w.precipMm(),
    meanC: w.meanC(),
    minC: w.minC(),
    maxC: w.maxC(),
    snowMm: w.snowMm(),
    soil: w.soil(),
    words: w.words() ?? "",
    snowLineM: w.snowLineM(),
  };
}

function weatherReport(r: W.WeatherReport): WeatherReport {
  const months: WeatherMonth[] = [];
  const m = new W.WeatherMonthInfo();
  for (let i = 0; i < r.monthsLength(); i++) {
    const x = r.months(i, m);
    if (!x) continue;
    months.push({
      year: Number(x.year()),
      month: x.month(),
      days: x.days(),
      precipMm: x.precipMm(),
      usualMm: x.usualMm(),
      wetDays: x.wetDays(),
      meanC: x.meanC(),
      usualC: x.usualC(),
      minC: x.minC(),
      maxC: x.maxC(),
      frostDays: x.frostDays(),
      snowDays: x.snowDays(),
      soil: x.soil(),
    });
  }
  return {
    rev: Number(r.rev()),
    heightM: r.heightM(),
    annualMm: r.annualMm(),
    today: dayWeather(r.today()),
    months,
    month: r.month(),
  };
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

const GROUP_STATES: Record<number, GroupState> = {
  [W.GroupState.Sound]: "sound",
  [W.GroupState.Symptom]: "symptom",
  [W.GroupState.Failed]: "failed",
};

const BUILDING_STATES: Record<number, BuildingState> = {
  [W.BuildingState.Standing]: "standing",
  [W.BuildingState.Damaged]: "damaged",
  [W.BuildingState.Ruin]: "ruin",
};

function groups(x: W.BuildingInfo): GroupInfo[] {
  const out: GroupInfo[] = [];
  const g = new W.GroupInfo();
  for (let k = 0; k < x.groupsLength(); k++) {
    const y = x.groups(k, g);
    if (!y) continue;
    out.push({
      id: y.id(),
      kind: y.kind() ?? "",
      quality: y.quality(),
      loss: y.loss(),
      state: GROUP_STATES[y.state()] ?? "sound",
      installedMinute: Number(y.installedMinute()),
      repairedMinute: Number(y.repairedMinute()),
    });
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
      grammar: x.grammar() ?? "",
      purpose: x.purpose() ?? "",
      size: [x.size(v)?.x() ?? 0, x.size(v)?.y() ?? 0],
      angle: x.angle(),
      storeys: x.storeys(),
      bays: x.bays(),
      loftBays: x.loftBays(),
      roofOutline: vec2List(x.roofOutlineLength(), (k, p) => x.roofOutline(k, p)),
      ridge: vec2List(x.ridgeLength(), (k, p) => x.ridge(k, p)),
      apexM: x.apexM(),
      floorByUse: Array.from(x.floorByUseArray() ?? []),
      storageKg: Array.from(x.storageKgArray() ?? []),
      workPlaces: x.workPlaces(),
      storedKg: Array.from(x.storedKgArray() ?? []),
      stored: x.stored() ?? "",
      firm: Number(x.firm()),
      firmName: x.firmName() ?? "",
      state: BUILDING_STATES[x.state()] ?? "standing",
      symptoms: x.symptoms() ?? "",
      leak: x.leak(),
      groups: groups(x),
      upkeep: x.upkeep() ?? "",
      style: x.style() ?? "",
      styleFrom: Number(x.styleFrom()),
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
    roofedM2PerHouse: s.roofedM2PerHouse(),
    storageKgPerHouse: s.storageKgPerHouse(),
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
        roofedM2: h.roofedM2(),
        storageKg: h.storageKg(),
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
        caution: t.caution(),
        trust: t.trust() ?? "",
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
  const ties: TieLine[] = [];
  for (let k = 0; k < p.tiesLength(); k++) {
    const t = p.ties(k);
    if (!t) continue;
    ties.push({
      person: Number(t.person()),
      name: t.name() ?? "",
      familiarity: t.familiarity(),
      warmth: t.warmth(),
      esteem: Array.from(t.esteemArray() ?? []),
      helpH: t.helpH(),
      reason: t.reason() ?? "",
      mutual: t.mutual(),
    });
  }
  const grievances: GrievanceLine[] = [];
  for (let k = 0; k < p.grievancesLength(); k++) {
    const g = p.grievances(k);
    if (!g) continue;
    grievances.push({
      issue: g.issue(),
      over: g.over() ?? "",
      blamed: g.blamed() ?? "",
      law: Number(g.law()),
      harmDays: g.harmDays(),
      unresolvedDays: g.unresolvedDays(),
      activation: g.activation(),
      madeMinute: Number(g.madeMinute()),
      raisedMinute: Number(g.raisedMinute()),
      reason: g.reason() ?? "",
    });
  }
  const heard: HeardLine[] = [];
  for (let k = 0; k < p.heardLength(); k++) {
    const h = p.heard(k);
    if (!h) continue;
    heard.push({
      kind: h.kind(),
      what: h.what() ?? "",
      from: Number(h.from()),
      fromName: h.fromName() ?? "",
      origin: Number(h.origin()),
      firstMinute: Number(h.firstMinute()),
      lastMinute: Number(h.lastMinute()),
      influence: h.influence(),
    });
  }
  const positions: PositionLine[] = [];
  for (let k = 0; k < p.positionsLength(); k++) {
    const q = p.positions(k);
    if (!q) continue;
    positions.push({
      question: q.question() ?? "",
      lean: q.lean() ?? "",
      x: q.x(),
      anchor: q.anchor(),
      salience: q.salience(),
      heard: q.heard(),
    });
  }
  const norms: NormLine[] = [];
  for (let k = 0; k < p.normsLength(); k++) {
    const n = p.norms(k);
    if (!n) continue;
    norms.push({
      statement: n.statement() ?? "",
      holds: n.holds() ?? "",
      believes: n.believes() ?? "",
      endorse: n.endorse(),
      expect: n.expect(),
      threshold: n.threshold(),
      activation: n.activation(),
      heard: n.heard(),
    });
  }
  const values: ValueLine[] = [];
  for (let k = 0; k < p.valuesLength(); k++) {
    const v = p.values(k);
    if (!v) continue;
    values.push({ name: v.name() ?? "", words: v.words() ?? "", v: v.v() });
  }
  const ideologies: IdeologyLine[] = [];
  for (let k = 0; k < p.ideologiesLength(); k++) {
    const d = p.ideologies(k);
    if (!d) continue;
    ideologies.push({
      name: d.name() ?? "",
      legitimacy: d.legitimacy() ?? "",
      sinceMinute: Number(d.sinceMinute()),
      from: Number(d.from()),
      fromName: d.fromName() ?? "",
      influence: d.influence(),
    });
  }
  const news: NewsLine[] = [];
  for (let k = 0; k < p.newsLength(); k++) {
    const n = p.news(k);
    if (!n) continue;
    news.push({ claim: n.claim(), what: n.what() ?? "" });
  }
  const influences: InfluenceLine[] = [];
  for (let k = 0; k < p.influencesLength(); k++) {
    const i = p.influences(k);
    if (!i) continue;
    influences.push({
      id: i.id(),
      kind: i.kind(),
      minute: Number(i.minute()),
      what: i.what() ?? "",
    });
  }
  const residence: string[] = [];
  for (let k = 0; k < p.residenceLength(); k++) {
    residence.push(p.residence(k) ?? "");
  }
  const places: string[] = [];
  for (let k = 0; k < p.placesLength(); k++) {
    places.push(p.places(k) ?? "");
  }
  const f = p.faction();
  const faction: FactionLine | null = f
    ? {
        faction: Number(f.faction()),
        name: f.name() ?? "",
        against: f.against() ?? "",
        organizer: Number(f.organizer()),
        organizerName: f.organizerName() ?? "",
        sinceMinute: Number(f.sinceMinute()),
        why: f.why() ?? "",
      }
    : null;
  const standing = p.standing();
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
    householdTaste: p.householdTaste() ?? "",
    householdAdmired: Number(p.householdAdmired()),
    ties,
    standing: standing ? standingLine(standing) : null,
    grievances,
    heard,
    positions,
    norms,
    values,
    ideologies,
    faction,
    news,
    influences,
    residence,
    places,
  };
}

function standingLine(l: W.StandingLine): StandingLine {
  return {
    person: Number(l.person()),
    name: l.name() ?? "",
    household: Number(l.household()),
    esteem: Array.from(l.esteemArray() ?? []),
    influence: l.influence(),
    notable: l.notable(),
  };
}

const STANCES: StanceKind[] = ["for", "against", "abstained"];
const LAW_STATUSES: LawStatus[] = [
  "proposed",
  "in force",
  "rejected",
  "lapsed",
  "superseded",
  "carried",
];
const OUTCOMES: LawOutcome[] = ["passed", "failed", "tied", "no quorum"];

function lawLine(l: W.LawLine): LawLine {
  const stances: StanceLine[] = [];
  for (let k = 0; k < l.stancesLength(); k++) {
    const r = l.stances(k);
    if (!r) continue;
    stances.push({
      person: Number(r.person()),
      name: r.name() ?? "",
      stance: STANCES[r.stance()] ?? "abstained",
      gain: r.gain(),
      regard: r.regard(),
      why: r.why() ?? "",
      opinion: r.opinion(),
      values: r.values(),
    });
  }
  return {
    id: Number(l.id()),
    what: l.what() ?? "",
    policy: l.policy() ?? "",
    levyShare: l.levyShare(),
    reliefDays: l.reliefDays(),
    status: LAW_STATUSES[l.status()] ?? "proposed",
    sponsor: Number(l.sponsor()),
    sponsorName: l.sponsorName() ?? "",
    proposedMinute: Number(l.proposedMinute()),
    issue: l.issue() ?? "",
    meetsMinute: Number(l.meetsMinute()),
    decidedMinute: Number(l.decidedMinute()),
    outcome: OUTCOMES[l.outcome()] ?? null,
    decision: l.decision() ?? "",
    eligible: l.eligible(),
    quorum: l.quorum(),
    stances,
    known: l.known(),
    complied: l.complied(),
    couldNot: l.couldNot(),
    evaded: l.evaded(),
    refused: l.refused(),
    unaware: l.unaware(),
    leviedKg: l.leviedKg(),
    withheldKg: l.withheldKg(),
    relieved: l.relieved(),
    reliefKg: l.reliefKg(),
    unanswered: l.unanswered(),
    holder: Number(l.holder()),
    holderName: l.holderName() ?? "",
    broken: l.broken(),
    brokenUnaware: l.brokenUnaware(),
  };
}

function governmentInfo(w: W.Government): GovernmentInfo {
  const polities: PolityLine[] = [];
  for (let k = 0; k < w.politiesLength(); k++) {
    const p = w.polities(k);
    if (!p) continue;
    const laws: LawLine[] = [];
    for (let j = 0; j < p.lawsLength(); j++) {
      const l = p.laws(j);
      if (l) laws.push(lawLine(l));
    }
    polities.push({
      polity: Number(p.polity()),
      settlement: Number(p.settlement()),
      name: p.name() ?? "",
      foundedMinute: Number(p.foundedMinute()),
      custom: p.custom() ?? "",
      members: p.members(),
      store: p.store() ?? "",
      storeKg: p.storeKg(),
      laws,
      gatheringLaw: Number(p.gatheringLaw()),
      gatheringMinute: Number(p.gatheringMinute()),
      gatheringPresent: p.gatheringPresent(),
      offices: Array.from({ length: p.officesLength() }, (_, k) => p.offices(k) ?? ""),
      label: p.label() ?? "",
      labelModifiers: Array.from(
        { length: p.labelModifiersLength() },
        (_, k) => p.labelModifiers(k) ?? "",
      ),
      labelWhy: Array.from({ length: p.labelWhyLength() }, (_, k) => p.labelWhy(k) ?? ""),
      labelConfidence: p.labelConfidence(),
      gatheringCases: Array.from(
        { length: p.gatheringCasesLength() },
        (_, k) => p.gatheringCases(k) ?? "",
      ),
      customHistory: Array.from(
        { length: p.customHistoryLength() },
        (_, k) => p.customHistory(k) ?? "",
      ),
      bodyMembers: p.bodyMembers(),
      factions: Array.from({ length: p.factionsLength() }, (_, k) => p.factions(k) ?? ""),
      petitions: Array.from({ length: p.petitionsLength() }, (_, k) => p.petitions(k) ?? ""),
      refusals: Array.from({ length: p.refusalsLength() }, (_, k) => p.refusals(k) ?? ""),
      revolts: Array.from({ length: p.revoltsLength() }, (_, k) => p.revolts(k) ?? ""),
      coups: Array.from({ length: p.coupsLength() }, (_, k) => p.coups(k) ?? ""),
    });
  }
  return { minute: Number(w.minute()), polities };
}

const TAKING_OUTCOMES: TakingOutcome[] = ["taken", "turned back", "fled"];

/** Takings, the truth and what is believed kept in their own lists (wire 1.30). */
export function orderInfo(w: W.Order): OrderInfo {
  const incidents: IncidentLine[] = [];
  for (let k = 0; k < w.incidentsLength(); k++) {
    const i = w.incidents(k);
    if (!i) continue;
    incidents.push({
      id: i.id(),
      minute: Number(i.minute()),
      settlement: Number(i.settlement()),
      actor: Number(i.actor()),
      actorName: i.actorName() ?? "",
      target: Number(i.target()),
      targetName: i.targetName() ?? "",
      outcome: TAKING_OUTCOMES[i.outcome()] ?? "taken",
      what: i.what() ?? "",
      kcal: i.kcal(),
      seenBy: Array.from({ length: i.seenByLength() }, (_, j) => i.seenBy(j) ?? ""),
      watch: i.watch() ?? "",
    });
  }
  const known: KnownLine[] = [];
  for (let k = 0; k < w.knownLength(); k++) {
    const l = w.known(k);
    if (!l) continue;
    known.push({
      incident: l.incident(),
      knowTaker: l.knowTaker(),
      knowLoss: l.knowLoss(),
      sources: l.sources(),
      victimKnows: l.victimKnows(),
      response: l.response() ?? "",
      owed: l.owed() ?? "",
      case: l.case_() ?? "",
    });
  }
  return {
    minute: Number(w.minute()),
    incidents,
    known,
    attempts: w.attempts(),
    takings: w.takings(),
    seen: w.seen(),
    knownToVictims: w.knownToVictims(),
    demands: w.demands(),
    met: w.met(),
    refused: w.refused(),
    refusals: Number(w.refusals()),
    cases: w.cases(),
    found: w.found(),
    notFound: w.notFound(),
    unheard: w.unheard(),
    encounters: Array.from({ length: w.encountersLength() }, (_, k) => w.encounters(k) ?? ""),
  };
}

function standingInfo(w: W.Standing): StandingInfo {
  const settlements: SettlementStanding[] = [];
  for (let k = 0; k < w.settlementsLength(); k++) {
    const s = w.settlements(k);
    if (!s) continue;
    const rows: StandingLine[] = [];
    for (let j = 0; j < s.rowsLength(); j++) {
      const l = s.rows(j);
      if (l) rows.push(standingLine(l));
    }
    settlements.push({
      settlement: Number(s.settlement()),
      name: s.name() ?? "",
      adults: s.adults(),
      rows,
    });
  }
  return {
    minute: Number(w.minute()),
    domains: Array.from({ length: w.domainsLength() }, (_, k) => w.domains(k) ?? ""),
    settlements,
    ties: w.ties(),
    letGo: Number(w.letGo()),
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
    case W.ResponseBody.Deposits: {
      const f = r.body(new W.Deposits()) as W.Deposits | null;
      if (!f) break;
      return { kind: "deposits", deposits: deposits(f) };
    }
    case W.ResponseBody.Earthworks: {
      const f = r.body(new W.Earthworks()) as W.Earthworks | null;
      if (!f) break;
      return { kind: "earthworks", earthworks: earthworks(f) };
    }
    case W.ResponseBody.WeatherReport: {
      const f = r.body(new W.WeatherReport()) as W.WeatherReport | null;
      if (!f) break;
      return { kind: "weather", weather: weatherReport(f) };
    }
    case W.ResponseBody.Standing: {
      const f = r.body(new W.Standing()) as W.Standing | null;
      if (!f) break;
      return { kind: "standing", standing: standingInfo(f) };
    }
    case W.ResponseBody.Government: {
      const f = r.body(new W.Government()) as W.Government | null;
      if (!f) break;
      return { kind: "government", government: governmentInfo(f) };
    }
    case W.ResponseBody.Order: {
      const f = r.body(new W.Order()) as W.Order | null;
      if (!f) break;
      return { kind: "order", order: orderInfo(f) };
    }
    default:
      break;
  }
  throw new Error(`unexpected response body ${r.bodyType()}`);
}
