// Boats and carts (M8b slices CO and CP; ADR-0029 §3, §9): how each is drawn, where it is drawn
// and which one the pointer is on. The kernel decides everything; this only shows it.

import type { VehicleInfo } from "./net/messages.js";

/** A dugout's timber and a handcart's planks. */
export const BOAT_COLOUR = 0x6b4423;
export const CART_COLOUR = 0xa0703a;

/** How a vehicle is drawn: by its mode, solid once made and faint while it is being made (never
 * fainter than about a quarter, so a hull begun still shows), a little faded as it wears. */
export function vehicleLook(v: VehicleInfo): { colour: number; alpha: number; boat: boolean } {
  const boat = v.mode === 0;
  const made = Math.min(1, Math.max(0, v.made));
  const worn = Math.min(1, Math.max(0, v.wear));
  const alpha = made < 1 ? 0.25 + 0.45 * made : 1 - 0.35 * worn;
  return { colour: boat ? BOAT_COLOUR : CART_COLOUR, alpha, boat };
}

/** Where a vehicle is drawn, metres: beside whoever of its household has it out, where they are
 * drawn now; else where it stands. `positions` maps a person's id to where they are drawn. */
export function vehiclePosition(v: VehicleInfo, positions: Map<number, [number, number]>): [number, number] {
  if (v.outWith !== 0) {
    const at = positions.get(v.outWith);
    if (at) return at;
  }
  return [v.x, v.y];
}

/** The vehicle the pointer is on: the nearest drawn within `slackM` of the point. */
export function vehicleAt(
  list: VehicleInfo[],
  positions: Map<number, [number, number]>,
  xM: number,
  yM: number,
  slackM: number,
): VehicleInfo | null {
  let best: VehicleInfo | null = null;
  let bestD = slackM;
  for (const v of list) {
    const [x, y] = vehiclePosition(v, positions);
    const d = Math.hypot(xM - x, yM - y);
    if (d <= bestD) {
      best = v;
      bestD = d;
    }
  }
  return best;
}
