// Plans (M8c slice CT, fourth part; ADR-0030 §3, §8): each plan put forward or in force in three
// layers kept apart (research 10-04 §6.1): the ground it reserves and the parcels it lays out,
// the reserved ground a way already runs over, and the buildings standing on reserved ground.
// The kernel decides everything; this only shows it.

import type { PlanInfo, PlanRect } from "./net/messages.js";

/** Reserved ground (streets and the square), parcels, ground a way runs over, and buildings on
 * reserved ground. */
export const RESERVED_COLOUR = 0xc9a227;
export const PARCEL_COLOUR = 0x7a6a48;
export const BUILT_COLOUR = 0x5f5f5f;
export const OCCUPIED_COLOUR = 0xb03a2e;

/** How a plan is drawn: one put forward at half the strength of one in force. */
export function planLook(p: PlanInfo): { reservedAlpha: number; parcelAlpha: number; lineAlpha: number } {
  const k = p.inForce ? 1 : 0.5;
  return { reservedAlpha: 0.28 * k, parcelAlpha: 0.45 * k, lineAlpha: 0.8 * k };
}

/** Which of a plan's layers a point is on. */
export type PlanLayer = "occupied" | "built" | "square" | "street" | "parcel";

/** Whether the point lies within the rectangle. */
function within(r: PlanRect, xM: number, yM: number): boolean {
  return xM >= r.x && xM < r.x + r.w && yM >= r.y && yM < r.y + r.h;
}

/** The plan and layer the pointer is on, the topmost first: a building on reserved ground, then
 * reserved ground a way runs over, the square, a street, a parcel; plans in force before those
 * put forward. */
export function planAt(list: PlanInfo[], xM: number, yM: number): { plan: PlanInfo; layer: PlanLayer } | null {
  const ordered = [...list.filter((p) => p.inForce), ...list.filter((p) => !p.inForce)];
  const layers: [PlanLayer, (p: PlanInfo) => PlanRect[]][] = [
    ["occupied", (p) => p.occupied],
    ["built", (p) => p.built],
    ["square", (p) => p.sites],
    ["street", (p) => p.streets],
    ["parcel", (p) => p.parcels],
  ];
  for (const [layer, rects] of layers) {
    for (const plan of ordered) {
      if (rects(plan).some((r) => within(r, xM, yM))) return { plan, layer };
    }
  }
  return null;
}

/** What the pointer is on, in words: "a street the plan reserves: Ashford: a plan (version 1) …". */
export function planWords(hit: { plan: PlanInfo; layer: PlanLayer }): string {
  const what: Record<PlanLayer, string> = {
    occupied: "a building on ground the plan reserves",
    built: "reserved ground a way already runs over",
    square: "the square the plan reserves",
    street: "a street the plan reserves",
    parcel: "a parcel the plan lays out",
  };
  return `${what[hit.layer]}: ${hit.plan.words}`;
}
