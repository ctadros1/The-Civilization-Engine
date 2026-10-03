// How buildings look on the map (M1 slice D): what each stage of a hut's construction shows from
// above, and which building lies under a point. The shape comes from the kernel, which expands
// each building's saved design; these are pure functions, so they can be tested without a
// renderer.

import type { BuildingInfo } from "./net/messages.js";

/** The stages a building is drawn in, in the order the legend lists them. */
export const BUILDING_LEGEND: { key: string; label: string; fill: number }[] = [
  { key: "marked", label: "ground marked out", fill: 0x4a3b2a },
  { key: "frame", label: "frame going up", fill: 0x8a6440 },
  { key: "walls", label: "walls going up", fill: 0xb89466 },
  { key: "roofed", label: "thatched hut", fill: 0xd6b25e },
];

const FILLS: Record<string, number> = Object.fromEntries(
  BUILDING_LEGEND.map((e) => [e.key, e.fill]),
);

/** Stage indexes (civ-grammar `Stage`). */
const FRAME = 1;
const WALLS = 2;
const ROOF = 3;

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

/** The building under a point in metres, if any: within its roof's reach, or on its plot while it
 * has no roof yet (the last drawn wins where they touch). */
export function buildingAt(
  buildings: readonly BuildingInfo[],
  x: number,
  y: number,
): BuildingInfo | null {
  for (let i = buildings.length - 1; i >= 0; i--) {
    const b = buildings[i]!;
    if (Math.hypot(x - b.x, y - b.y) <= b.roofRadiusM) return b;
    const p = b.plot;
    if (p && x >= p.x && x < p.x + p.w && y >= p.y && y < p.y + p.h) return b;
  }
  return null;
}
