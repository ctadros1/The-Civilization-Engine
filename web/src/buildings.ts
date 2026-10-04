// How buildings look on the map (M1 slice D; M3b slice O): what each stage of a building's
// construction shows from above, how the readout names it, and which building lies under a
// point. A hut is round under a cone of thatch; a frame building is a rectangle of bays under a
// gabled roof with its ridge. The shape comes from the kernel, which expands each building's
// saved design; these are pure functions, so they can be tested without a renderer.

import type { BuildingInfo } from "./net/messages.js";

/** The stages a building is drawn in, in the order the legend lists them. */
export const BUILDING_LEGEND: { key: string; label: string; fill: number }[] = [
  { key: "marked", label: "ground marked out", fill: 0x4a3b2a },
  { key: "frame", label: "frame going up", fill: 0x8a6440 },
  { key: "walls", label: "walls going up", fill: 0xb89466 },
  { key: "roofed", label: "thatched roof", fill: 0xd6b25e },
];

const FILLS: Record<string, number> = Object.fromEntries(
  BUILDING_LEGEND.map((e) => [e.key, e.fill]),
);

/** Stage indexes (civ-grammar `Stage`). */
const FRAME = 1;
const WALLS = 2;
const ROOF = 3;

/** Bits set in `n`. */
function bitCount(n: number): number {
  let c = 0;
  for (let v = n; v > 0; v >>>= 1) c += v & 1;
  return c;
}

/** "1 bay", "3 bays". */
function bays(n: number): string {
  return `${n} ${n === 1 ? "bay" : "bays"}`;
}

/**
 * A building in the map's readout: "hut of 38 m², room for 7: walls going up, 40% done", or for
 * a frame building its bays, storeys and lofts and its floor by use: "longhouse of 3 bays, a
 * loft over 1 bay, 50 m², room for 6, 13 m² to store: finished", then what its household keeps
 * in it ("loft over 1 bay: 1.2 t of 1.9 t, mostly grain"). Its size shows whether its household
 * built for more than live there (M3a slice L: house size by wealth).
 */
export function buildingWords(b: BuildingInfo): string {
  const kept = b.stored ? `; ${b.stored}` : "";
  return `${shapeWords(b)}: ${b.status}${kept}`;
}

/** A building's program and shape: "hut of 30 m², room for 5", or a frame building's bays,
 * storeys, lofts and floor by use. */
function shapeWords(b: BuildingInfo): string {
  const name = b.program.toLowerCase();
  if (b.grammar !== "frame") {
    const size = b.floorM2 > 0 ? ` of ${Math.round(b.floorM2)} m², room for ${b.sleeps}` : "";
    return `${name}${size}`;
  }
  const shape = [bays(b.bays)];
  if (b.storeys > 1) shape.push(`${b.storeys} storeys`);
  const lofts = bitCount(b.loftBays);
  if (lofts > 0) shape.push(lofts === b.bays ? "a loft over every bay" : `a loft over ${bays(lofts)}`);
  const [living = 0, store = 0, work = 0] = b.floorByUse;
  const uses: string[] = [`${Math.round(b.floorM2)} m²`];
  if (living > 0) uses.push(`room for ${b.sleeps}`);
  if (store > 0) uses.push(`${Math.round(store)} m² to store`);
  if (work > 0) uses.push(`${b.workPlaces} ${b.workPlaces === 1 ? "place" : "places"} to work`);
  return `${name} of ${shape.join(", ")}, ${uses.join(", ")}`;
}

/** Whether `(x, y)` lies inside the polygon `poly` (even-odd rule). */
function inside(poly: readonly [number, number][], x: number, y: number): boolean {
  let hit = false;
  for (let i = 0, j = poly.length - 1; i < poly.length; j = i++) {
    const [xi, yi] = poly[i]!;
    const [xj, yj] = poly[j]!;
    if (yi > y !== yj > y && x < ((xj - xi) * (y - yi)) / (yj - yi) + xi) hit = !hit;
  }
  return hit;
}

/** Whether a point in metres lies under a building's roof: a frame's gabled roof, or within the
 * reach of a hut's cone. */
export function underRoof(b: BuildingInfo, x: number, y: number): boolean {
  if (b.roofOutline.length >= 3) return inside(b.roofOutline, x, y);
  return Math.hypot(x - b.x, y - b.y) <= b.roofRadiusM;
}

/** Which legend entry a building is drawn as. */
export function buildingKey(b: BuildingInfo): string {
  if (b.roofed) return "roofed";
  if (b.stage >= WALLS) return "walls";
  if (b.stage === FRAME) return "frame";
  return "marked";
}

/** How a building is drawn: what of it stands, and how strongly each part shows. */
export interface BuildingLook {
  key: string;
  /** How many of its posts (or, while the foundation is dug, postholes) show. */
  posts: number;
  /** Colour of the posts or holes. */
  postFill: number;
  /** Opacity of the walls (0 = none yet). */
  wallAlpha: number;
  wallFill: number;
  /** Opacity of the thatch (0 = none yet). */
  roofAlpha: number;
  roofFill: number;
}

/** What a building's construction shows: holes dug as the foundation goes, posts once the frame
 * is up, walls as they are woven and daubed, and the thatch as it goes on. */
export function buildingLook(b: BuildingInfo): BuildingLook {
  const key = buildingKey(b);
  const all = b.posts.length;
  const done = Math.min(1, Math.max(0, b.progress));
  const posts = b.stage === 0 ? Math.round(all * done) : all;
  const wallAlpha = b.stage < WALLS ? 0 : b.stage === WALLS ? 0.35 + 0.6 * done : 0.95;
  const roofAlpha = b.roofed ? 0.95 : b.stage === ROOF ? 0.25 + 0.6 * done : 0;
  return {
    key,
    posts,
    postFill: b.stage === 0 ? FILLS.marked! : FILLS.frame!,
    wallAlpha,
    wallFill: FILLS.walls!,
    roofAlpha,
    roofFill: FILLS.roofed!,
  };
}

/** How a mark is filled: colour and opacity. */
export interface Fill {
  color: number;
  alpha: number;
}

/** How a mark is outlined: colour, opacity and width in metres. */
export interface Stroke extends Fill {
  width: number;
}

/** One shape drawn for a building, in metres from the map's north-west corner. */
export type Mark =
  | { kind: "rect"; x: number; y: number; w: number; h: number; fill?: Fill; stroke?: Stroke }
  | { kind: "poly"; points: number[]; fill?: Fill; stroke?: Stroke }
  | { kind: "line"; from: [number, number]; to: [number, number]; fill?: Fill; stroke?: Stroke }
  | { kind: "circle"; x: number; y: number; r: number; fill?: Fill; stroke?: Stroke };

const PLOT_EDGE = 0xe8dcc0;
const EAVES = 0x6b5426;
const DOORWAY = 0x2b2117;

/**
 * What is drawn for a building, bottom first: its plot's edge; its roof once thatching starts, a
 * frame building's gabled roof with its ridge or a hut's cone; its walls' outline and its posts
 * until the thatch hides them; and once roofed, the doorway as a dark notch at the eaves.
 */
export function buildingMarks(b: BuildingInfo): Mark[] {
  const look = buildingLook(b);
  const marks: Mark[] = [];
  if (b.plot) {
    const { x, y, w, h } = b.plot;
    marks.push({ kind: "rect", x, y, w, h, stroke: { width: 0.25, color: PLOT_EDGE, alpha: 0.5 } });
  }
  const gabled = b.roofOutline.length >= 3;
  if (look.roofAlpha > 0) {
    const fill = { color: look.roofFill, alpha: look.roofAlpha };
    const stroke = { width: 0.2, color: EAVES, alpha: look.roofAlpha };
    if (gabled) {
      // A gabled roof, its two slopes meeting at the ridge.
      marks.push({ kind: "poly", points: b.roofOutline.flat(), fill, stroke });
      const [r0, r1] = b.ridge;
      if (r0 && r1) {
        marks.push({ kind: "line", from: r0, to: r1, stroke: { ...stroke, width: 0.25 } });
      }
    } else {
      marks.push({ kind: "circle", x: b.x, y: b.y, r: b.roofRadiusM, fill, stroke });
    }
  }
  if (look.wallAlpha > 0 && b.outline.length > 2 && look.roofAlpha < 0.9) {
    marks.push({
      kind: "poly",
      points: b.outline.flat(),
      stroke: { width: 0.3, color: look.wallFill, alpha: look.wallAlpha },
    });
  }
  if (look.roofAlpha < 0.9) {
    for (const [x, y] of b.posts.slice(0, look.posts)) {
      marks.push({ kind: "circle", x, y, r: 0.15, fill: { color: look.postFill, alpha: 0.9 } });
    }
  }
  if (b.roofed) {
    // The doorway: just outside a frame building's door, at the eaves of a hut's cone.
    const [dx, dy] = [Math.cos(b.doorDir), Math.sin(b.doorDir)];
    const [x, y] = gabled
      ? [b.door[0] + dx * 0.3, b.door[1] + dy * 0.3]
      : [b.x + dx * b.roofRadiusM * 0.92, b.y + dy * b.roofRadiusM * 0.92];
    marks.push({ kind: "circle", x, y, r: 0.35, fill: { color: DOORWAY, alpha: 0.9 } });
  }
  return marks;
}

/** The building under a point in metres, if any: within its roof's reach, or on its plot while it
 * has no roof yet (the last drawn wins where they touch). */
export function buildingAt(
  buildings: readonly BuildingInfo[],
  x: number,
  y: number,
): BuildingInfo | null {
  for (let i = buildings.length - 1; i >= 0; i--) {
    const b = buildings[i]!;
    if (underRoof(b, x, y)) return b;
    const p = b.plot;
    if (p && x >= p.x && x < p.x + p.w && y >= p.y && y < p.y + p.h) return b;
  }
  return null;
}
