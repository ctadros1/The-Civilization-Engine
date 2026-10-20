import { describe, expect, it } from "vitest";

import type { VehicleInfo } from "../src/net/messages.js";
import { BOAT_COLOUR, CART_COLOUR, vehicleAt, vehicleLook, vehiclePosition } from "../src/vehicles.js";

function vehicle(over: Partial<VehicleInfo>): VehicleInfo {
  return {
    id: 1,
    mode: 0,
    name: "Dugout",
    owner: 7,
    ownerName: "Ada's household",
    x: 100,
    y: 200,
    made: 1,
    wear: 0,
    outWith: 0,
    loadKg: 0,
    words: "Ada's household's dugout, made in year 1",
    ...over,
  };
}

describe("vehicles", () => {
  it("draws a boat and a cart by their mode, faint while being made and faded as they wear", () => {
    expect(vehicleLook(vehicle({}))).toEqual({ colour: BOAT_COLOUR, alpha: 1, boat: true });
    const cart = vehicleLook(vehicle({ mode: 1, made: 0.3 }));
    expect(cart.colour).toBe(CART_COLOUR);
    expect(cart.boat).toBe(false);
    expect(cart.alpha).toBeCloseTo(0.25 + 0.45 * 0.3);
    expect(vehicleLook(vehicle({ mode: 1, made: 0 })).alpha).toBeCloseTo(0.25);
    expect(vehicleLook(vehicle({ wear: 1 })).alpha).toBeCloseTo(0.65);
  });

  it("is drawn beside whoever has it out, else where it stands", () => {
    const positions = new Map<number, [number, number]>([[42, [5, 6]]]);
    expect(vehiclePosition(vehicle({}), positions)).toEqual([100, 200]);
    expect(vehiclePosition(vehicle({ outWith: 42 }), positions)).toEqual([5, 6]);
    // One out with someone not drawn stays where it stood.
    expect(vehiclePosition(vehicle({ outWith: 9 }), positions)).toEqual([100, 200]);
  });

  it("finds the nearest vehicle under the pointer", () => {
    const list = [vehicle({ id: 1 }), vehicle({ id: 2, x: 104, y: 200 }), vehicle({ id: 3, outWith: 42 })];
    const positions = new Map<number, [number, number]>([[42, [500, 500]]]);
    expect(vehicleAt(list, positions, 103, 200, 5)?.id).toBe(2);
    expect(vehicleAt(list, positions, 99, 200, 5)?.id).toBe(1);
    expect(vehicleAt(list, positions, 501, 500, 5)?.id).toBe(3);
    expect(vehicleAt(list, positions, 300, 300, 5)).toBeNull();
  });
});
