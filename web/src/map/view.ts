// The world map: a PixiJS scene in metres from the map's north-west corner.
//
// - A base texture covers the whole map at the finest level that fits in 1024² samples.
// - Zoomed in past the base texture's resolution, 256-cell detail tiles at full resolution are
//   fetched for the visible area and cached.
// - Rivers are vector polylines from the hydrography, drawn at their channel width but never
//   thinner than about a pixel.
// - Drag or arrow keys pan, the wheel or +/- zoom, 0 fits the map.
// - People are dots coloured by what they are doing, moved along their trips between snapshots;
//   a click selects one. Settlements show their hearth and name.
// - Fields are rectangles coloured by where they are in their year; the pointer readout names the
//   field under it.
// - Buildings show what their construction has reached: the plot, postholes and posts, the walls'
//   outline, and the thatch, from the shape the kernel expands; the readout names the one under
//   the pointer.
// - Worn ground shows as trodden earth, one texture a 64-cell tile, and the trails the kernel
//   traced through it as lines; the readout names a trail under the pointer.
// - Crossings over water show as their members laid from bank to bank, faint while being built
//   and dark once they gave way; the readout names the one under the pointer.
// - Wells show as their shafts, blue while water stands in them; springs flowing today as small
//   blue dots, and the places at the water's edge people drew at today as pale ones; the readout
//   names the one under the pointer, and the rivers' flow today over a river or lake.

import { Application, Container, Graphics, Sprite, Text, Texture } from "pixi.js";

import { buildingAt, buildingMarks, type Mark } from "../buildings.js";
import { crossingAt, crossingEnds, crossingLook } from "../crossings.js";
import { depositAt, depositLook } from "../deposits.js";
import { changedTiles, detailTilesOver, earthworkAt, earthworkLook } from "../earthworks.js";
import { fieldAt, fieldLook } from "../fields.js";
import type { HostClient } from "../net/client.js";
import { TRAIL_COLOUR, pathAt, smoothed, trailLook, wornPixels } from "../paths.js";
import { BANK_COLOUR, SPRING_COLOUR, bankAt, springAt, wellAt, wellLook } from "../water.js";
import {
  RasterLayer,
  type ActivityInfo,
  type BankDrawInfo,
  type BuildingInfo,
  type Clock,
  type CrossingInfo,
  type CrossingsInfo,
  type DepositInfo,
  type EarthworkInfo,
  type EarthworksInfo,
  type FieldInfo,
  type GoodInfo,
  type Hydrography,
  type PathsInfo,
  type PersonBrief,
  type SettlementBrief,
  type SpringInfo,
  type TripInfo,
  type WaterInfo,
  type WellInfo,
  type WorldInfo,
} from "../net/messages.js";
import {
  activityColour,
  estimateMinute,
  missingTrips,
  personPosition,
  pruneTrips,
  spread,
} from "../people.js";
import {
  WATER_LAKE,
  WATER_LAND,
  WATER_OCEAN,
  WATER_RIVER,
  baseLevel,
  classes,
  contourInterval,
  elevations,
  seasonKey,
  shade,
  type Season,
  type ShadeOptions,
} from "./shade.js";

export type MapState = "empty" | "loading" | "ready" | "error";

export interface MapStatus {
  state: MapState;
  message: string;
}

export interface PointerInfo {
  xM: number;
  yM: number;
  cellX: number;
  cellY: number;
  elevationM: number | null;
  water: "land" | "river" | "lake" | "ocean" | null;
  /** The field under the pointer, if any. */
  field: FieldInfo | null;
  /** The building under the pointer, if any. */
  building: BuildingInfo | null;
  /** The deposit under the pointer, if any (M3b slice Q). */
  deposit: DepositInfo | null;
  /** The earthwork under the pointer, if any (M3b slice Q). */
  earthwork: EarthworkInfo | null;
  /** The crossing under the pointer, if any (M5c slice AW). */
  crossing: CrossingInfo | null;
  /** The well, spring or place at the water's edge drawn at today under the pointer, if any, and
   * the rivers' flow today against their mean (null before the water is read; M6a slice AY). */
  well: WellInfo | null;
  spring: SpringInfo | null;
  bank: BankDrawInfo | null;
  flow: number | null;
  /** The worn ground under the pointer, if any. */
  path: { wear: number; trail: boolean } | null;
}

/** A shaded raster region with the samples behind it. */
interface Patch {
  /** Sample grid origin, in level-0 cells. */
  cellX0: number;
  cellY0: number;
  level: number;
  sw: number;
  sh: number;
  elev: Float32Array;
  water: Uint8Array;
  sprite: Sprite;
  /** Metres per sample, for shading it again. */
  cellM: number;
  /** The samples the sprite shows, inside a margin kept for slopes; null for all of them. */
  crop: { x: number; y: number; w: number; h: number } | null;
}

interface TileSlot {
  patch: Patch | null;
  used: number;
}

/** Least time between shading the map again for a change of season, milliseconds. */
const RESHADE_MS = 2000;

/** The `crop` of `w`-wide RGBA pixels. */
function cropPixels(
  rgba: Uint8ClampedArray,
  w: number,
  crop: { x: number; y: number; w: number; h: number },
): Uint8ClampedArray {
  const out = new Uint8ClampedArray(crop.w * crop.h * 4);
  for (let y = 0; y < crop.h; y++) {
    const from = ((y + crop.y) * w + crop.x) * 4;
    out.set(rgba.subarray(from, from + crop.w * 4), y * crop.w * 4);
  }
  return out;
}

/** Closest zoom, screen pixels per metre. */
const MAX_PX_PER_M = 8;

const DETAIL_TILE = 256;
const MAX_DETAIL_TILES = 96;
const MAX_IN_FLIGHT = 4;
const RIVER_COLOUR = 0x3a74ad;
const HEARTH_COLOUR = 0xd9653b;
/** Screen radius of an adult's dot, pixels. */
const PERSON_PX = 4.5;
/** How close a click must be to a person to select them, pixels. */
const PICK_PX = 12;

const WATER_NAMES: Record<number, PointerInfo["water"]> = {
  [WATER_LAND]: "land",
  [WATER_RIVER]: "river",
  [WATER_LAKE]: "lake",
  [WATER_OCEAN]: "ocean",
};

function texture(rgba: Uint8ClampedArray, w: number, h: number): Texture {
  const canvas = document.createElement("canvas");
  canvas.width = w;
  canvas.height = h;
  const ctx = canvas.getContext("2d");
  if (!ctx) throw new Error("no 2D canvas");
  ctx.putImageData(new ImageData(rgba as Uint8ClampedArray<ArrayBuffer>, w, h), 0, 0);
  const tex = Texture.from(canvas, true);
  tex.source.scaleMode = "linear";
  return tex;
}

/** Draws one of a building's marks: its shape, then its fill and its outline. */
function drawMark(g: Graphics, m: Mark): void {
  switch (m.kind) {
    case "rect":
      g.rect(m.x, m.y, m.w, m.h);
      break;
    case "poly":
      g.poly(m.points, true);
      break;
    case "line":
      g.moveTo(m.from[0], m.from[1]).lineTo(m.to[0], m.to[1]);
      break;
    case "circle":
      g.circle(m.x, m.y, m.r);
      break;
  }
  if (m.fill) g.fill(m.fill);
  if (m.stroke) g.stroke(m.stroke);
}

export class MapView {
  private app: Application | null = null;
  private readonly world = new Container();
  private readonly terrain = new Container();
  private readonly detail = new Container();
  private readonly rivers = new Graphics();
  private client: HostClient | null = null;
  private info: WorldInfo | null = null;
  private base: Patch | null = null;
  private hydro: Hydrography | null = null;
  private tiles = new Map<string, TileSlot>();
  private inFlight = 0;
  private generation = 0;
  private cameraChanged = true;
  private riverScale = 0;
  private riversVisible = true;
  private lastPointer: { x: number; y: number } | null = null;
  private shadeOptions: ShadeOptions | null = null;
  private season: Season | null = null;
  /** The season the map was last shaded for, and when. */
  private seasonShown = "";
  private seasonShownAt = -Infinity;
  private seasonTimer: ReturnType<typeof setTimeout> | null = null;
  private readonly fieldsLayer = new Graphics();
  private fields: FieldInfo[] = [];
  private readonly buildingsLayer = new Graphics();
  private buildings: BuildingInfo[] = [];
  private readonly depositsLayer = new Graphics();
  private deposits: DepositInfo[] = [];
  private readonly earthworksLayer = new Graphics();
  private earthworks: EarthworkInfo[] = [];
  private readonly crossingsLayer = new Graphics();
  private crossings: CrossingInfo[] = [];
  private crossingScale = 0;
  private readonly waterLayer = new Graphics();
  private water: WaterInfo | null = null;
  private waterScale = 0;
  /** Each changed ground tile's revision as last drawn, by index. */
  private groundSeen = new Map<number, number>();
  private readonly wornLayer = new Container();
  private readonly trailsLayer = new Graphics();
  private paths: PathsInfo | null = null;
  private trailScale = 0;
  private readonly settlementLayer = new Container();
  private readonly peopleLayer = new Graphics();
  private people: PersonBrief[] = [];
  private settlements: SettlementBrief[] = [];
  private settlementsKey = "";
  private clock: Clock | null = null;
  private clockAt = 0;
  private trips = new Map<number, TripInfo>();
  private tripsInFlight = false;
  private colours: number[] = [];
  private selected: number | null = null;

  status: MapStatus = { state: "empty", message: "" };
  onStatus: (status: MapStatus) => void = () => {};
  onPointer: (info: PointerInfo | null) => void = () => {};
  onCamera: () => void = () => {};
  /** A click on the map: the person under it, or null for empty ground. */
  onSelect: (id: number | null) => void = () => {};
  /** A click on the map while placing (see `setPlacing`): where, metres. */
  onPlace: (xM: number, yM: number) => void = () => {};
  private placing = false;

  /** Creates the renderer inside `el`. Throws when the browser cannot render (no WebGL). */
  async mount(el: HTMLElement): Promise<void> {
    const app = new Application();
    await app.init({
      resizeTo: el,
      background: "#1a1e22",
      antialias: true,
      autoDensity: true,
      resolution: Math.min(window.devicePixelRatio || 1, 2),
    });
    this.app = app;
    el.appendChild(app.canvas);
    this.world.addChild(
      this.terrain,
      this.detail,
      this.wornLayer,
      this.trailsLayer,
      this.rivers,
      this.fieldsLayer,
      this.depositsLayer,
      this.earthworksLayer,
      this.crossingsLayer,
      this.waterLayer,
      this.buildingsLayer,
      this.settlementLayer,
      this.peopleLayer,
    );
    app.stage.addChild(this.world);
    this.attachInput(app.canvas);
    app.renderer.on("resize", () => {
      this.cameraChanged = true;
    });
    app.ticker.add(() => this.frame());
  }

  get scale(): number {
    return this.world.scale.x;
  }

  private setStatus(state: MapState, message = ""): void {
    this.status = { state, message };
    this.onStatus(this.status);
  }

  private cellM(): number {
    return this.info?.cellSizeM ?? 8;
  }

  /** Removes the current world from the map. */
  clear(): void {
    this.generation++;
    for (const child of [...this.terrain.children, ...this.detail.children]) {
      child.destroy({ texture: true, textureSource: true });
    }
    this.terrain.removeChildren();
    this.detail.removeChildren();
    this.rivers.clear();
    this.tiles.clear();
    this.base = null;
    this.hydro = null;
    this.info = null;
    this.inFlight = 0;
    this.people = [];
    this.settlements = [];
    this.settlementsKey = "";
    for (const child of this.settlementLayer.removeChildren()) child.destroy();
    this.fields = [];
    this.fieldsLayer.clear();
    this.buildings = [];
    this.buildingsLayer.clear();
    this.earthworks = [];
    this.earthworksLayer.clear();
    this.crossings = [];
    this.crossingsLayer.clear();
    this.water = null;
    this.waterLayer.clear();
    this.groundSeen.clear();
    this.setPaths(null);
    this.peopleLayer.clear();
    this.trips.clear();
    this.clock = null;
    this.setStatus("empty");
  }

  /** The activity catalogue, for dot colours. */
  setActivities(activities: ActivityInfo[]): void {
    this.colours = activities.map((a) => activityColour(a));
  }

  /** The people, settlements and clock of the latest snapshot. */
  setPeople(people: PersonBrief[], settlements: SettlementBrief[], clock: Clock | null): void {
    this.people = people;
    this.clock = clock;
    this.clockAt = performance.now();
    pruneTrips(people, this.trips);
    void this.fetchTrips();
    const key = JSON.stringify(settlements.map((s) => [s.id, s.name, s.population]));
    if (key !== this.settlementsKey) {
      this.settlementsKey = key;
      this.settlements = settlements;
      this.drawSettlements();
    }
  }

  /** The fields, as the host last listed them. */
  setFields(fields: FieldInfo[]): void {
    this.fields = fields;
    const g = this.fieldsLayer;
    g.clear();
    for (const f of fields) {
      const look = fieldLook(f);
      g.rect(f.x, f.y, f.w, f.h).fill({ color: look.fill, alpha: look.alpha });
    }
    if (this.lastPointer) this.onPointer(this.pointerInfo(this.lastPointer.x, this.lastPointer.y));
  }

  /** The deposits, as the host last listed them: known ones solid, unfound ones faint. */
  setDeposits(deposits: DepositInfo[], goods: GoodInfo[]): void {
    this.deposits = deposits;
    const g = this.depositsLayer;
    g.clear();
    for (const d of deposits) {
      const look = depositLook(d, goods);
      g.circle(d.x, d.y, d.radiusM).fill({ color: look.colour, alpha: look.alpha });
      if (look.outline) g.circle(d.x, d.y, d.radiusM).stroke({ width: 1, color: look.colour, alpha: 0.6 });
    }
    if (this.lastPointer) this.onPointer(this.pointerInfo(this.lastPointer.x, this.lastPointer.y));
  }

  /** The earthworks, as the host last listed them (null for none): each platform drawn as bare
   * earth over its plot, and the map read again where the ground under it has changed. */
  setEarthworks(info: EarthworksInfo | null): void {
    this.earthworks = info?.works ?? [];
    const g = this.earthworksLayer;
    g.clear();
    for (const w of this.earthworks) {
      const look = earthworkLook(w);
      g.rect(w.x, w.y, w.w, w.h).fill({ color: look.colour, alpha: look.alpha });
      g.rect(w.x, w.y, w.w, w.h).stroke({ width: 0.3, color: look.colour, alpha: look.outlineAlpha });
    }
    if (info) {
      for (const index of changedTiles(this.groundSeen, info)) {
        for (const key of detailTilesOver(index, info.tileCells, info.tilesX, DETAIL_TILE)) {
          const slot = this.tiles.get(key);
          if (!slot) continue;
          slot.patch?.sprite.destroy({ texture: true, textureSource: true });
          this.tiles.delete(key);
          this.cameraChanged = true;
        }
      }
      this.groundSeen = new Map(info.tiles.map((t) => [t.index, t.rev]));
    } else {
      this.groundSeen.clear();
    }
    if (this.lastPointer) this.onPointer(this.pointerInfo(this.lastPointer.x, this.lastPointer.y));
  }

  /** The crossings over water, as the host last listed them (null for none). */
  setCrossings(info: CrossingsInfo | null): void {
    this.crossings = info?.crossings ?? [];
    this.drawCrossings();
    if (this.lastPointer) this.onPointer(this.pointerInfo(this.lastPointer.x, this.lastPointer.y));
  }

  /** Each crossing's members as a bar from bank to bank, never thinner than about two pixels so a
   * log over a brook shows at any zoom. */
  private drawCrossings(): void {
    const g = this.crossingsLayer;
    g.clear();
    this.crossingScale = this.scale;
    const pxM = 1 / Math.max(this.scale, 1e-6);
    for (const c of this.crossings) {
      const look = crossingLook(c);
      const [x0, y0, x1, y1] = crossingEnds(c);
      g.moveTo(x0, y0);
      g.lineTo(x1, y1);
      g.stroke({ width: Math.max(look.widthM, 2.2 * pxM), color: look.colour, alpha: look.alpha, cap: "butt" });
    }
  }

  /** The wells, springs and places drawn at today, as the host last listed them (null for none). */
  setWater(info: WaterInfo | null): void {
    this.water = info;
    this.drawWater();
    if (this.lastPointer) this.onPointer(this.pointerInfo(this.lastPointer.x, this.lastPointer.y));
  }

  /** Each well as its shaft, never smaller than about two pixels across; each spring and each
   * place drawn at as a dot about three pixels across, those drawn at more widely. */
  private drawWater(): void {
    const g = this.waterLayer;
    g.clear();
    this.waterScale = this.scale;
    const water = this.water;
    if (!water) return;
    const pxM = 1 / Math.max(this.scale, 1e-6);
    for (const b of water.banks) {
      g.circle(b.x, b.y, (1.2 + Math.min(2, Math.sqrt(b.trips) / 4)) * pxM);
      g.fill({ color: BANK_COLOUR, alpha: 0.7 });
    }
    for (const s of water.springs) {
      g.circle(s.x, s.y, 1.6 * pxM);
      g.fill({ color: SPRING_COLOUR, alpha: s.drawnL > 0 ? 0.95 : 0.6 });
    }
    for (const w of water.wells) {
      const look = wellLook(w);
      g.circle(w.x, w.y, Math.max(look.radiusM, 1.1 * pxM));
      g.fill({ color: look.colour, alpha: look.alpha });
    }
  }

  /** The buildings, as the host last listed them. */
  setBuildings(buildings: BuildingInfo[]): void {
    this.buildings = buildings;
    const g = this.buildingsLayer;
    g.clear();
    for (const b of buildings) {
      for (const m of buildingMarks(b)) drawMark(g, m);
    }
    if (this.lastPointer) this.onPointer(this.pointerInfo(this.lastPointer.x, this.lastPointer.y));
  }

  /** The worn ground and the trails, as the host last surveyed them (null for none). */
  setPaths(paths: PathsInfo | null): void {
    this.paths = paths;
    for (const child of this.wornLayer.removeChildren()) {
      child.destroy({ texture: true, textureSource: true });
    }
    if (paths && paths.tileCells > 0) {
      const tc = paths.tileCells;
      const sideM = tc * this.cellM();
      for (const tile of paths.worn) {
        const sprite = new Sprite(texture(wornPixels(tile, tc), tc, tc));
        sprite.x = (tile.index % paths.tilesX) * sideM;
        sprite.y = Math.floor(tile.index / paths.tilesX) * sideM;
        sprite.width = sideM;
        sprite.height = sideM;
        this.wornLayer.addChild(sprite);
      }
    }
    this.drawTrails();
    if (this.lastPointer) this.onPointer(this.pointerInfo(this.lastPointer.x, this.lastPointer.y));
  }

  private drawTrails(): void {
    const g = this.trailsLayer;
    g.clear();
    this.trailScale = this.scale;
    if (!this.paths) return;
    const pxM = 1 / Math.max(this.scale, 1e-6);
    for (const t of this.paths.trails) {
      const p = smoothed(t.points);
      if (p.length < 2) continue;
      const look = trailLook(t);
      g.moveTo(p[0]![0], p[0]![1]);
      for (let k = 1; k < p.length; k++) g.lineTo(p[k]![0], p[k]![1]);
      g.stroke({
        width: Math.max(look.widthM, 1.6 * pxM),
        color: TRAIL_COLOUR,
        alpha: look.alpha,
        cap: "round",
        join: "round",
      });
    }
  }

  /** While placing, a click on the map is a place (`onPlace`) rather than a selection. */
  setPlacing(on: boolean): void {
    this.placing = on;
    this.app?.canvas.classList.toggle("placing", on);
  }

  /** Marks a person as selected (or nobody). */
  setSelected(id: number | null): void {
    this.selected = id;
  }

  private async fetchTrips(): Promise<void> {
    const client = this.client;
    if (this.tripsInFlight || !client) return;
    const wanted = missingTrips(this.people, this.trips).slice(0, 256);
    if (wanted.length === 0) return;
    const generation = this.generation;
    this.tripsInFlight = true;
    try {
      const trips = await client.trips(wanted);
      if (generation !== this.generation) return;
      for (const trip of trips) this.trips.set(trip.id, trip);
    } catch (e) {
      console.warn(`tce: trips could not be loaded: ${String(e)}`);
    } finally {
      this.tripsInFlight = false;
    }
  }

  /** The simulation minute being drawn. */
  private minuteNow(): number {
    return this.clock ? estimateMinute(this.clock, this.clockAt, performance.now()) : 0;
  }

  private drawSettlements(): void {
    for (const child of this.settlementLayer.removeChildren()) child.destroy();
    for (const s of this.settlements) {
      const marker = new Container();
      marker.position.set(s.x, s.y);
      const hearth = new Graphics();
      hearth.circle(0, 0, 5).fill({ color: HEARTH_COLOUR }).stroke({ width: 1.5, color: 0x1a1e22 });
      const label = new Text({
        text: `${s.name} · ${s.population}`,
        style: {
          fontFamily: "system-ui, sans-serif",
          fontSize: 13,
          fill: 0xf3eee6,
          stroke: { color: 0x1a1e22, width: 3 },
        },
      });
      label.anchor.set(0.5, 1.6);
      marker.addChild(hearth, label);
      this.settlementLayer.addChild(marker);
    }
    this.scaleSettlements();
  }

  /** Keeps settlement markers the same size on screen at any zoom. */
  private scaleSettlements(): void {
    const s = 1 / this.scale;
    for (const marker of this.settlementLayer.children) marker.scale.set(s);
  }

  /** Where each person is drawn now, metres: on their trips, with people at one spot spread
   * apart by a few pixels. */
  private drawnPositions(): [number, number][] {
    const t = this.minuteNow();
    const pxM = 1 / this.scale;
    const at = this.people.map((p) => personPosition(p, this.trips, t));
    return spread(at, 2.2 * PERSON_PX * pxM, 0.75);
  }

  private drawPeople(): void {
    const g = this.peopleLayer;
    g.clear();
    if (this.people.length === 0) return;
    const pxM = 1 / this.scale;
    const positions = this.drawnPositions();
    let selectedAt: [number, number, number] | null = null;
    for (const [i, p] of this.people.entries()) {
      const [x, y] = positions[i]!;
      const r = (p.ageYears < 3 ? 0.55 : p.ageYears < 12 ? 0.72 : 1) * PERSON_PX * pxM;
      g.circle(x, y, r).fill({
        color: this.colours[p.activity] ?? 0xffffff,
        alpha: p.asleep ? 0.6 : 1,
      });
      g.stroke({ width: pxM, color: 0x14181b, alpha: 0.9 });
      if (p.id === this.selected) selectedAt = [x, y, r];
    }
    if (selectedAt) {
      const [x, y, r] = selectedAt;
      g.circle(x, y, r + 3 * pxM).stroke({ width: 2 * pxM, color: 0xffffff });
    }
  }

  /** Screen positions of the people being drawn, for picking and tests. */
  peopleOnScreen(): { id: number; sx: number; sy: number }[] {
    const positions = this.drawnPositions();
    return this.people.map((p, i) => {
      const [x, y] = positions[i]!;
      return { id: p.id, sx: this.world.x + x * this.scale, sy: this.world.y + y * this.scale };
    });
  }

  /** The person nearest a screen point, within picking distance. */
  personAt(sx: number, sy: number): number | null {
    let best: number | null = null;
    let bestD = PICK_PX;
    for (const p of this.peopleOnScreen()) {
      const d = Math.hypot(p.sx - sx, p.sy - sy);
      if (d <= bestD) {
        best = p.id;
        bestD = d;
      }
    }
    return best;
  }

  /** Centres the view on a point, zooming in to at least `minScale` pixels per metre. */
  centreOn(x: number, y: number, minScale = 1): void {
    const app = this.app;
    if (!app || !this.info) return;
    const { max } = this.limits();
    const s = Math.min(max, Math.max(this.scale, minScale));
    this.world.scale.set(s);
    this.world.position.set(app.screen.width / 2 - x * s, app.screen.height / 2 - y * s);
    this.clampPosition();
    this.cameraChanged = true;
  }

  /** Loads and shows a world. */
  async show(info: WorldInfo, client: HostClient): Promise<void> {
    this.clear();
    const generation = this.generation;
    this.info = info;
    this.client = client;
    this.setStatus("loading", "Drawing the map…");
    try {
      const level = baseLevel(info.width, info.height);
      const lw = Math.ceil(info.width / 2 ** level);
      const lh = Math.ceil(info.height / 2 ** level);
      const region = { level, x0: 0, y0: 0, width: lw, height: lh };
      const [elevTile, waterTile, hydro] = await Promise.all([
        client.raster({ layer: RasterLayer.Elevation, ...region }),
        client.raster({ layer: RasterLayer.Water, ...region }),
        client.hydrography(info.cellSizeM),
      ]);
      if (generation !== this.generation) return;
      const range = info.maxElevationM - Math.max(info.minElevationM, info.seaLevelM);
      this.shadeOptions = {
        cellM: info.cellSizeM,
        seaLevelM: info.seaLevelM,
        minM: info.minElevationM,
        maxM: info.maxElevationM,
        contourM: contourInterval(range),
      };
      const elev = elevations(elevTile);
      const water = classes(waterTile);
      const cellM = info.cellSizeM * 2 ** level;
      const rgba = shade(elev, water, elevTile.width, elevTile.height, {
        ...this.shadeOptions,
        cellM,
        season: this.season ?? undefined,
      });
      const sprite = new Sprite(texture(rgba, elevTile.width, elevTile.height));
      const sampleM = info.cellSizeM * 2 ** level;
      sprite.width = elevTile.width * sampleM;
      sprite.height = elevTile.height * sampleM;
      this.terrain.addChild(sprite);
      this.base = {
        cellX0: 0,
        cellY0: 0,
        level,
        sw: elevTile.width,
        sh: elevTile.height,
        elev,
        water,
        sprite,
        cellM,
        crop: null,
      };
      this.seasonShown = seasonKey(this.season);
      this.hydro = hydro;
      this.riverScale = 0;
      this.fit();
      this.setStatus("ready");
    } catch (e) {
      if (generation !== this.generation) return;
      this.setStatus("error", e instanceof Error ? e.message : String(e));
    }
  }

  /** The season the clock tells (null for none): the land is tinted by it and whitened above the
   * snow line. The map is shaded again when what shows has changed, at most every two seconds. */
  setSeason(season: Season | null): void {
    this.season = season;
    const key = seasonKey(season);
    if (key === this.seasonShown) return;
    const wait = this.seasonShownAt + RESHADE_MS - performance.now();
    if (wait > 0) {
      // Too soon: shade for whatever the season is then.
      this.seasonTimer ??= setTimeout(() => {
        this.seasonTimer = null;
        this.setSeason(this.season);
      }, wait);
      return;
    }
    this.seasonShown = key;
    this.seasonShownAt = performance.now();
    this.reshade();
  }

  /** Shades the base map and every detail tile again from the samples they keep. */
  private reshade(): void {
    const opts = this.shadeOptions;
    if (!opts) return;
    const season = this.season ?? undefined;
    const patches = [this.base, ...[...this.tiles.values()].map((t) => t.patch)];
    for (const p of patches) {
      if (!p) continue;
      const full = shade(p.elev, p.water, p.sw, p.sh, { ...opts, cellM: p.cellM, season });
      const rgba = p.crop ? cropPixels(full, p.sw, p.crop) : full;
      const old = p.sprite.texture;
      p.sprite.texture = texture(rgba, p.crop?.w ?? p.sw, p.crop?.h ?? p.sh);
      old.destroy(true);
    }
  }

  /** Fits the whole map in the view. */
  fit(): void {
    const app = this.app;
    const info = this.info;
    if (!app || !info) return;
    const wM = info.width * info.cellSizeM;
    const hM = info.height * info.cellSizeM;
    const s = Math.min(app.screen.width / wM, app.screen.height / hM) * 0.94;
    this.world.scale.set(s);
    this.world.position.set((app.screen.width - wM * s) / 2, (app.screen.height - hM * s) / 2);
    this.cameraChanged = true;
  }

  private limits(): { min: number; max: number } {
    const app = this.app;
    const info = this.info;
    if (!app || !info) return { min: 1e-4, max: 1 };
    const fit = Math.min(
      app.screen.width / (info.width * info.cellSizeM),
      app.screen.height / (info.height * info.cellSizeM),
    );
    // Close enough to see a hut's posts: eight pixels a metre.
    return { min: fit * 0.5, max: MAX_PX_PER_M };
  }

  /** Zooms by `factor` around a screen point (the centre by default). */
  zoomBy(factor: number, sx?: number, sy?: number): void {
    const app = this.app;
    if (!app || !this.info) return;
    const px = sx ?? app.screen.width / 2;
    const py = sy ?? app.screen.height / 2;
    const { min, max } = this.limits();
    const s0 = this.scale;
    const s1 = Math.min(max, Math.max(min, s0 * factor));
    const wx = (px - this.world.x) / s0;
    const wy = (py - this.world.y) / s0;
    this.world.scale.set(s1);
    this.world.position.set(px - wx * s1, py - wy * s1);
    this.clampPosition();
    this.cameraChanged = true;
  }

  panBy(dx: number, dy: number): void {
    this.world.position.set(this.world.x + dx, this.world.y + dy);
    this.clampPosition();
    this.cameraChanged = true;
  }

  /** Keeps part of the map on screen. */
  private clampPosition(): void {
    const app = this.app;
    const info = this.info;
    if (!app || !info) return;
    const wPx = info.width * info.cellSizeM * this.scale;
    const hPx = info.height * info.cellSizeM * this.scale;
    const keep = 80;
    this.world.x = Math.min(app.screen.width - keep, Math.max(keep - wPx, this.world.x));
    this.world.y = Math.min(app.screen.height - keep, Math.max(keep - hPx, this.world.y));
  }

  setRiversVisible(visible: boolean): void {
    this.riversVisible = visible;
    this.rivers.visible = visible;
  }

  private attachInput(canvas: HTMLCanvasElement): void {
    canvas.tabIndex = 0;
    canvas.setAttribute("role", "img");
    canvas.setAttribute(
      "aria-label",
      "World map. Drag or use the arrow keys to pan; scroll or press + and - to zoom; 0 fits the map.",
    );
    let drag: { x: number; y: number; wx: number; wy: number } | null = null;
    let moved = 0;
    canvas.addEventListener("pointerdown", (e) => {
      canvas.setPointerCapture(e.pointerId);
      canvas.focus();
      drag = { x: e.clientX, y: e.clientY, wx: this.world.x, wy: this.world.y };
      moved = 0;
      canvas.classList.add("dragging");
    });
    canvas.addEventListener("pointermove", (e) => {
      if (drag) {
        moved = Math.max(moved, Math.hypot(e.clientX - drag.x, e.clientY - drag.y));
        this.world.position.set(drag.wx + e.clientX - drag.x, drag.wy + e.clientY - drag.y);
        this.clampPosition();
        this.cameraChanged = true;
      }
      this.lastPointer = { x: e.offsetX, y: e.offsetY };
      this.onPointer(this.pointerInfo(e.offsetX, e.offsetY));
    });
    const end = (e: PointerEvent) => {
      if (canvas.hasPointerCapture(e.pointerId)) canvas.releasePointerCapture(e.pointerId);
      drag = null;
      canvas.classList.remove("dragging");
    };
    canvas.addEventListener("pointerup", (e) => {
      const click = drag !== null && moved < 5;
      end(e);
      if (!click || !this.info) return;
      if (this.placing) {
        this.onPlace((e.offsetX - this.world.x) / this.scale, (e.offsetY - this.world.y) / this.scale);
      } else {
        this.onSelect(this.personAt(e.offsetX, e.offsetY));
      }
    });
    canvas.addEventListener("pointercancel", end);
    canvas.addEventListener("pointerleave", () => {
      this.lastPointer = null;
      this.onPointer(null);
    });
    canvas.addEventListener(
      "wheel",
      (e) => {
        e.preventDefault();
        this.zoomBy(Math.exp(-e.deltaY * 0.0015), e.offsetX, e.offsetY);
      },
      { passive: false },
    );
    canvas.addEventListener("keydown", (e) => {
      const step = 80;
      const keys: Record<string, () => void> = {
        ArrowLeft: () => this.panBy(step, 0),
        ArrowRight: () => this.panBy(-step, 0),
        ArrowUp: () => this.panBy(0, step),
        ArrowDown: () => this.panBy(0, -step),
        "+": () => this.zoomBy(1.25),
        "=": () => this.zoomBy(1.25),
        "-": () => this.zoomBy(0.8),
        "0": () => this.fit(),
      };
      const action = keys[e.key];
      if (action) {
        e.preventDefault();
        action();
      }
    });
  }

  /** What is under a screen point. */
  pointerInfo(sx: number, sy: number): PointerInfo | null {
    const info = this.info;
    if (!info || !this.base) return null;
    const xM = (sx - this.world.x) / this.scale;
    const yM = (sy - this.world.y) / this.scale;
    const cellX = Math.floor(xM / info.cellSizeM);
    const cellY = Math.floor(yM / info.cellSizeM);
    if (cellX < 0 || cellY < 0 || cellX >= info.width || cellY >= info.height) return null;
    const key = `${Math.floor(cellX / DETAIL_TILE)},${Math.floor(cellY / DETAIL_TILE)}`;
    const patch = this.tiles.get(key)?.patch ?? this.base;
    const scale = 2 ** patch.level;
    const px = Math.min(patch.sw - 1, Math.floor((cellX - patch.cellX0) / scale));
    const py = Math.min(patch.sh - 1, Math.floor((cellY - patch.cellY0) / scale));
    const i = py * patch.sw + px;
    return {
      xM,
      yM,
      cellX,
      cellY,
      elevationM: patch.elev[i] ?? null,
      water: WATER_NAMES[patch.water[i] ?? -1] ?? null,
      field: fieldAt(this.fields, xM, yM),
      building: buildingAt(this.buildings, xM, yM),
      deposit: depositAt(this.deposits, xM, yM),
      earthwork: earthworkAt(this.earthworks, xM, yM),
      crossing: crossingAt(this.crossings, xM, yM, 4 / this.scale),
      well: wellAt(this.water?.wells ?? [], xM, yM, 3 / this.scale),
      spring: springAt(this.water?.springs ?? [], xM, yM, 4 / this.scale),
      bank: bankAt(this.water?.banks ?? [], xM, yM, 4 / this.scale),
      flow: this.water?.flow ?? null,
      path: pathAt(this.paths, cellX, cellY),
    };
  }

  private frame(): void {
    if (!this.info || !this.base) return;
    if (this.cameraChanged) {
      this.cameraChanged = false;
      const scale = this.scale;
      if (this.riverScale === 0 || Math.abs(Math.log(scale / this.riverScale)) > 0.15) {
        this.drawRivers();
      }
      if (this.trailScale === 0 || Math.abs(Math.log(scale / this.trailScale)) > 0.15) {
        this.drawTrails();
      }
      if (this.crossings.length > 0 && Math.abs(Math.log(scale / this.crossingScale)) > 0.15) {
        this.drawCrossings();
      }
      if (this.water && Math.abs(Math.log(scale / this.waterScale)) > 0.15) {
        this.drawWater();
      }
      this.updateDetail();
      this.scaleSettlements();
      if (this.lastPointer) {
        this.onPointer(this.pointerInfo(this.lastPointer.x, this.lastPointer.y));
      }
      this.onCamera();
    }
    this.drawPeople();
  }

  private drawRivers(): void {
    const g = this.rivers;
    g.clear();
    this.riverScale = this.scale;
    if (!this.hydro) return;
    const pxM = 1 / this.scale;
    for (const reach of this.hydro.reaches) {
      const p = reach.points;
      if (p.length < 4) continue;
      g.moveTo(p[0]!, p[1]!);
      for (let k = 2; k < p.length; k += 2) g.lineTo(p[k]!, p[k + 1]!);
      const minWidth = pxM * Math.min(3, 0.8 + 0.45 * reach.order);
      g.stroke({
        width: Math.max(reach.widthM, minWidth),
        color: RIVER_COLOUR,
        alpha: 0.95,
        cap: "round",
        join: "round",
      });
    }
    g.visible = this.riversVisible;
  }

  private wantsDetail(): boolean {
    if (!this.base || this.base.level === 0) return false;
    return this.scale * this.cellM() * 2 ** this.base.level > 1.5;
  }

  private updateDetail(): void {
    const app = this.app;
    const info = this.info;
    if (!app || !info) return;
    if (!this.wantsDetail()) {
      this.detail.visible = false;
      return;
    }
    this.detail.visible = true;
    const cell = info.cellSizeM * this.scale;
    const x0 = Math.max(0, Math.floor(-this.world.x / cell / DETAIL_TILE));
    const y0 = Math.max(0, Math.floor(-this.world.y / cell / DETAIL_TILE));
    const x1 = Math.min(
      Math.ceil(info.width / DETAIL_TILE) - 1,
      Math.floor((app.screen.width - this.world.x) / cell / DETAIL_TILE),
    );
    const y1 = Math.min(
      Math.ceil(info.height / DETAIL_TILE) - 1,
      Math.floor((app.screen.height - this.world.y) / cell / DETAIL_TILE),
    );
    const now = performance.now();
    for (let ty = y0; ty <= y1; ty++) {
      for (let tx = x0; tx <= x1; tx++) {
        const key = `${tx},${ty}`;
        const slot = this.tiles.get(key);
        if (slot) {
          slot.used = now;
        } else if (this.inFlight < MAX_IN_FLIGHT) {
          const fresh: TileSlot = { patch: null, used: now };
          this.tiles.set(key, fresh);
          void this.fetchTile(tx, ty, key, fresh);
        } else {
          this.cameraChanged = true; // try again next frame
        }
      }
    }
    this.evictTiles();
  }

  private evictTiles(): void {
    if (this.tiles.size <= MAX_DETAIL_TILES) return;
    const loaded = [...this.tiles.entries()]
      .filter(([, slot]) => slot.patch)
      .sort((a, b) => a[1].used - b[1].used);
    for (const [key, slot] of loaded.slice(0, this.tiles.size - MAX_DETAIL_TILES)) {
      slot.patch?.sprite.destroy({ texture: true, textureSource: true });
      this.tiles.delete(key);
    }
  }

  /** Reads detail tile `key` for `slot`, filling it only while it is still that key's slot: one
   * dropped (evicted, or read again since the ground changed) meanwhile gets nothing stale. */
  private async fetchTile(tx: number, ty: number, key: string, slot: TileSlot): Promise<void> {
    const info = this.info;
    const client = this.client;
    const opts = this.shadeOptions;
    if (!info || !client || !opts) return;
    const generation = this.generation;
    this.inFlight++;
    try {
      const x0 = tx * DETAIL_TILE;
      const y0 = ty * DETAIL_TILE;
      const w = Math.min(DETAIL_TILE, info.width - x0);
      const h = Math.min(DETAIL_TILE, info.height - y0);
      // One cell of margin so slopes are continuous across tile edges.
      const mx0 = Math.max(0, x0 - 1);
      const my0 = Math.max(0, y0 - 1);
      const mx1 = Math.min(info.width, x0 + w + 1);
      const my1 = Math.min(info.height, y0 + h + 1);
      const region = { level: 0, x0: mx0, y0: my0, width: mx1 - mx0, height: my1 - my0 };
      const [elevTile, waterTile] = await Promise.all([
        client.raster({ layer: RasterLayer.Elevation, ...region }),
        client.raster({ layer: RasterLayer.Water, ...region }),
      ]);
      if (generation !== this.generation) return;
      const elev = elevations(elevTile);
      const water = classes(waterTile);
      const full = shade(elev, water, elevTile.width, elevTile.height, {
        ...opts,
        season: this.season ?? undefined,
      });
      const ox = x0 - mx0;
      const oy = y0 - my0;
      const rgba = cropPixels(full, elevTile.width, { x: ox, y: oy, w, h });
      const sprite = new Sprite(texture(rgba, w, h));
      sprite.position.set(x0 * info.cellSizeM, y0 * info.cellSizeM);
      sprite.width = w * info.cellSizeM;
      sprite.height = h * info.cellSizeM;
      this.detail.addChild(sprite);
      if (this.tiles.get(key) === slot) {
        slot.patch = {
          cellX0: mx0,
          cellY0: my0,
          level: 0,
          sw: elevTile.width,
          sh: elevTile.height,
          elev,
          water,
          sprite,
          cellM: opts.cellM,
          crop: { x: ox, y: oy, w, h },
        };
      } else {
        sprite.destroy({ texture: true, textureSource: true });
      }
    } catch (e) {
      if (generation === this.generation && this.tiles.get(key) === slot) {
        this.tiles.delete(key);
        console.warn(`tce: a map tile could not be loaded: ${String(e)}`);
      }
    } finally {
      if (generation === this.generation) {
        this.inFlight--;
        this.cameraChanged = true;
      }
    }
  }

  /** Map facts for tests and the debug hook. */
  debugState() {
    return {
      state: this.status.state,
      message: this.status.message,
      baseLevel: this.base?.level ?? null,
      baseSamples: this.base ? [this.base.sw, this.base.sh] : null,
      detailTiles: [...this.tiles.values()].filter((t) => t.patch).length,
      detailVisible: this.detail.visible && this.wantsDetail(),
      reaches: this.hydro?.reaches.length ?? 0,
      people: this.people.length,
      tripsCached: this.trips.size,
      settlements: this.settlements.map((s) => s.name),
      fields: this.fields.length,
      buildings: this.buildings.length,
      roofed: this.buildings.filter((b) => b.roofed).length,
      // M3b slice R: buildings that followed an admired one, and frame buildings, where they stand.
      followed: this.buildings
        .filter((b) => b.styleFrom !== 0)
        .map((b) => ({ x: b.x, y: b.y, style: b.style })),
      frames: this.buildings
        .filter((b) => b.grammar === "frame")
        .map((b) => ({ x: b.x, y: b.y, program: b.program })),
      deposits: this.deposits.length,
      earthworks: this.earthworks.length,
      platforms: this.earthworks.map((w) => ({ x: w.x + w.w / 2, y: w.y + w.h / 2, done: w.done })),
      // M5c slice AW: crossings over water, where each lies and how far along it is.
      crossings: this.crossings.map((c) => ({
        x: (c.ax + c.bx) / 2,
        y: (c.ay + c.by) / 2,
        state: c.state,
        words: c.words,
      })),
      // M6a slice AY: wells, where each is and what has become of it; springs and places drawn at.
      wells: (this.water?.wells ?? []).map((w) => ({
        x: w.x,
        y: w.y,
        state: w.state,
        words: w.words,
        fouled: w.fouled,
      })),
      springs: this.water?.springs.length ?? 0,
      banks: this.water?.banks.length ?? 0,
      flow: this.water?.flow ?? null,
      groundTiles: this.groundSeen.size,
      wornTiles: this.paths?.worn.length ?? 0,
      trails: this.paths?.trails.length ?? 0,
      selected: this.selected,
      camera: { x: this.world.x, y: this.world.y, scale: this.scale },
      screen: this.app ? [this.app.screen.width, this.app.screen.height] : null,
      // M3c slice U: the season the land was last shaded for ("" for none).
      season: this.seasonShown,
    };
  }
}
