import { describe, expect, it } from "vitest";

import {
  BUILDING_LEGEND,
  buildingAt,
  buildingKey,
  buildingLook,
  buildingMarks,
  buildingWords,
  underRoof,
  type Mark,
} from "../src/buildings.js";
import type { BuildingInfo } from "../src/net/messages.js";

function hut(over: Partial<BuildingInfo> = {}): BuildingInfo {
  const posts: [number, number][] = Array.from({ length: 20 }, (_, i) => {
    const a = (i / 20) * 2 * Math.PI;
    return [100 + 3.1 * Math.cos(a), 200 + 3.1 * Math.sin(a)];
  });
  return {
    id: 1,
    household: 2,
    settlement: 3,
    program: "Hut",
    x: 100,
    y: 200,
    radiusM: 3.1,
    roofRadiusM: 3.6,
    door: [103, 200],
    doorDir: 0,
    stage: 0,
    stageName: "foundation",
    progress: 0,
    roofed: false,
    outline: [],
    posts,
    plot: { x: 96.4, y: 196.4, w: 7.2, h: 7.2 },
    floorM2: 30.2,
    sleeps: 5,
    startedMinute: 0,
    status: "ground marked out",
    grammar: "hut",
    purpose: "dwelling",
    size: [6.2, 6.2],
    angle: 0,
    storeys: 1,
    bays: 1,
    loftBays: 0,
    roofOutline: [],
    ridge: [],
    apexM: 4.9,
    floorByUse: [30.2, 0, 0],
    storageKg: [0, 0, 1812],
    workPlaces: 0,
    storedKg: [0, 0, 0],
    stored: "",
    firm: 0,
    firmName: "",
    state: "standing",
    symptoms: "",
    leak: 0,
    groups: [],
    upkeep: "",
    style: "",
    styleFrom: 0,
    fire: "none",
    fireId: 0,
    fireWords: "",
    site: "",
    gas: "",
    records: "",
    wire: "",
    road: "",
    ...over,
  };
}

/** A longhouse of three bays, 7.5 by 5 m, along the east-west axis, with a loft over one bay. */
function longhouse(over: Partial<BuildingInfo> = {}): BuildingInfo {
  return hut({
    id: 4,
    program: "Longhouse",
    grammar: "frame",
    x: 200,
    y: 100,
    radiusM: 4.5,
    roofRadiusM: 5.2,
    outline: [
      [196.175, 97.425],
      [203.825, 97.425],
      [203.825, 102.575],
      [196.175, 102.575],
    ],
    roofOutline: [
      [195.75, 97],
      [204.25, 97],
      [204.25, 103],
      [195.75, 103],
    ],
    ridge: [
      [195.75, 100],
      [204.25, 100],
    ],
    plot: { x: 195.75, y: 97, w: 8.5, h: 6 },
    size: [7.5, 5],
    bays: 3,
    loftBays: 0b001,
    floorM2: 50,
    floorByUse: [37.5, 12.5, 0],
    storageKg: [0, 1875, 2250],
    sleeps: 6,
    stage: 5,
    roofed: true,
    status: "finished",
    ...over,
  });
}

describe("buildings on the map", () => {
  it("are named in the readout with their size and state", () => {
    expect(buildingWords(hut())).toBe("hut of 30 m², room for 5: ground marked out");
    expect(buildingWords(hut({ floorM2: 0, status: "finished" }))).toBe("hut: finished");
  });

  it("name a frame building's bays, lofts and floor by use", () => {
    expect(buildingWords(longhouse())).toBe(
      "longhouse of 3 bays, a loft over 1 bay, 50 m², room for 6, 13 m² to store: finished",
    );
    expect(
      buildingWords(
        longhouse({ storeys: 2, loftBays: 0, bays: 4, floorM2: 100, floorByUse: [100, 0, 0], sleeps: 19 }),
      ),
    ).toBe("longhouse of 4 bays, 2 storeys, 100 m², room for 19: finished");
    expect(
      buildingWords(
        longhouse({
          program: "Granary",
          purpose: "store",
          bays: 2,
          loftBays: 0,
          floorM2: 15,
          floorByUse: [0, 15, 0],
          storageKg: [7500, 0, 0],
          sleeps: 0,
        }),
      ),
    ).toBe("granary of 2 bays, raised on posts, 15 m², 15 m² to store: finished");
    expect(
      buildingWords(
        longhouse({
          program: "Workshop",
          bays: 2,
          loftBays: 0b11,
          floorM2: 60,
          floorByUse: [0, 30, 30],
          workPlaces: 5,
          sleeps: 0,
        }),
      ),
    ).toBe("workshop of 2 bays, a loft over every bay, 60 m², 30 m² to store, 5 places to work: finished");
    // A firm's workshop goes by the firm's name.
    expect(
      buildingWords(
        longhouse({
          program: "Workshop",
          bays: 2,
          loftBays: 0,
          floorM2: 30,
          floorByUse: [0, 0, 30],
          workPlaces: 5,
          sleeps: 0,
          firm: 41,
          firmName: "Wren's sickle workshop",
        }),
      ),
    ).toBe("Wren's sickle workshop of 2 bays, 30 m², 5 places to work: finished");
  });

  it("say what their household keeps in them", () => {
    const b = longhouse({ stored: "loft over 1 bay: 1.2 t of 1.9 t, mostly grain" });
    expect(buildingWords(b)).toBe(
      "longhouse of 3 bays, a loft over 1 bay, 50 m², room for 6, 13 m² to store: finished; loft over 1 bay: 1.2 t of 1.9 t, mostly grain",
    );
    expect(buildingWords(hut({ status: "finished", stored: "floor: 1.8 t of 3.0 t, mostly provisions" }))).toBe(
      "hut of 30 m², room for 5: finished; floor: 1.8 t of 3.0 t, mostly provisions",
    );
  });

  it("say how they were built and what their builders' taste followed", () => {
    const followed = hut({
      stage: 5,
      roofed: true,
      status: "finished",
      style: "roof pitched 49°, walls 1.9 m to the eaves, after Bo's hut",
      styleFrom: 12,
    });
    expect(buildingWords(followed)).toBe(
      "hut of 30 m², room for 5: finished; roof pitched 49°, walls 1.9 m to the eaves, after Bo's hut",
    );
  });

  it("say what they show of their condition and what is being mended", () => {
    const leaking = hut({
      stage: 5,
      roofed: true,
      status: "finished",
      symptoms: "the thatch leaks; rot at the posts' foot",
      leak: 0.3,
      upkeep: "mending the covering, 40% done",
      stored: "floor: 1.8 t of 3.0 t, mostly provisions",
    });
    expect(buildingWords(leaking)).toBe(
      "hut of 30 m², room for 5: finished; the thatch leaks; rot at the posts' foot; mending the covering, 40% done; floor: 1.8 t of 3.0 t, mostly provisions",
    );
    // A ruin says so once: its status is its state.
    const ruin = hut({
      stage: 5,
      status: "a ruin",
      state: "ruin",
      symptoms: "a ruin: its posts gave way",
    });
    expect(buildingWords(ruin)).toBe("hut of 30 m², room for 5: a ruin");
  });

  it("darken a leaking roof and leave a ruin its posts", () => {
    const sound = hut({ stage: 5, roofed: true });
    const leaking = hut({ stage: 5, roofed: true, leak: 0.6 });
    expect(buildingKey(sound)).toBe("roofed");
    expect(buildingKey(hut({ stage: 5, roofed: true, leak: 0.01 }))).toBe("roofed");
    expect(buildingKey(leaking)).toBe("leaking");
    const brightness = (c: number) => ((c >> 16) & 0xff) + ((c >> 8) & 0xff) + (c & 0xff);
    expect(brightness(buildingLook(leaking).roofFill)).toBeLessThan(
      brightness(buildingLook(sound).roofFill),
    );
    const ruin = hut({ stage: 5, roofed: false, state: "ruin" });
    expect(buildingKey(ruin)).toBe("ruin");
    const look = buildingLook(ruin);
    expect([look.posts, look.roofAlpha]).toEqual([20, 0]);
    const marks = buildingMarks(ruin);
    expect(marks.some((m) => m.kind === "circle" && m.r > 1)).toBe(false);
  });

  it("says why its household built a home where it stands (wire 1.87)", () => {
    const chosen = hut({
      stage: 5,
      roofed: true,
      site: "chosen over 5 others: 34 hours less walking and levelling over 5 years than the next best",
    });
    expect(buildingWords(chosen)).toMatch(/; chosen over 5 others: 34 hours less walking/);
    expect(buildingWords(hut({ stage: 5, roofed: true }))).not.toMatch(/chosen over/);
  });

  it("say how their gas serves them, connected to a main or as a gasworks (wire 1.103)", () => {
    const dark = hut({
      stage: 5,
      roofed: true,
      gas: "Gas: 1 burner on Ada's main; not lit on its last evening: the main could not carry its burners",
    });
    expect(buildingWords(dark)).toMatch(/; Gas: 1 burner on Ada's main; not lit on its last evening: the main could not/);
    expect(buildingWords(hut({ stage: 5, roofed: true }))).not.toMatch(/Gas/);
  });

  it("say how the wire serves them, hooked up to a line or as a station (wire 1.107)", () => {
    const cut = hut({ stage: 5, roofed: true, wire: "Current: 0.2 kW from Ada's station; cut off: its account went unpaid" });
    expect(buildingWords(cut)).toMatch(/; Current: 0\.2 kW from Ada's station; cut off: its account went unpaid$/);
    expect(buildingWords(hut({ stage: 5, roofed: true }))).not.toMatch(/Current/);
  });

  it("say the runs a motor cart's keeper posts (wire 1.109)", () => {
    const words = "Motor cart runs to Elmhollow: 1 a day, leaving at 06:00, back at 07:12; fare 210 kcal; 4 rides since day 120";
    expect(buildingWords(hut({ stage: 5, roofed: true, road: words }))).toMatch(/; Motor cart runs to Elmhollow: .* since day 120$/);
    expect(buildingWords(hut({ stage: 5, roofed: true }))).not.toMatch(/Motor cart/);
  });

  it("glow while they burn, show charred once burnt out, and say so in the readout", () => {
    const sound = hut({ stage: 5, roofed: true });
    const alight = hut({ stage: 5, roofed: true, fire: "alight", fireId: 1, fireWords: "its roof alight since 02:14" });
    const involved = hut({ stage: 5, roofed: true, fire: "involved", fireId: 1 });
    const burnt = hut({ stage: 5, roofed: true, fire: "burnt", fireId: 1, fireWords: "burnt out at 02:45 on 3 May of year 2" });
    expect([alight, involved, burnt].map(buildingKey)).toEqual(["burning", "burning", "burnt"]);
    const red = (c: number) => (c >> 16) & 0xff;
    expect(red(buildingLook(involved).roofFill)).toBeGreaterThan(red(buildingLook(sound).roofFill));
    expect(buildingLook(involved).roofFill).not.toBe(buildingLook(alight).roofFill);
    // The covering is gone: no roof is drawn, the charred posts show.
    const look = buildingLook(burnt);
    expect([look.roofAlpha, look.posts]).toEqual([0, 20]);
    expect(buildingMarks(burnt).some((m) => m.kind === "circle" && m.r > 1)).toBe(false);
    expect(buildingWords(alight)).toContain("; its roof alight since 02:14");
    expect(buildingWords(burnt)).toContain("; burnt out at 02:45 on 3 May of year 2");
    expect(BUILDING_LEGEND.map((e) => e.key)).toEqual(expect.arrayContaining(["burning", "burnt"]));
  });

  it("find a frame building under its gabled roof, not its circle", () => {
    const b = longhouse();
    expect(underRoof(b, 204, 102.5)).toBe(true);
    // Within the circle round the roof, but past its edge.
    expect(Math.hypot(205 - b.x, 100 - b.y)).toBeLessThan(b.roofRadiusM);
    expect(underRoof(b, 205, 100)).toBe(false);
    expect(buildingAt([b], 200, 100)?.id).toBe(4);
    expect(buildingAt([b], 200, 104)).toBeNull();
    expect(underRoof(hut(), 103, 200)).toBe(true);
  });

  it("are drawn by how far their construction has gone", () => {
    expect(buildingKey(hut())).toBe("marked");
    expect(buildingKey(hut({ stage: 1 }))).toBe("frame");
    expect(buildingKey(hut({ stage: 2 }))).toBe("walls");
    expect(buildingKey(hut({ stage: 3 }))).toBe("walls");
    expect(buildingKey(hut({ stage: 4, roofed: true }))).toBe("roofed");
    expect(buildingKey(hut({ stage: 5, roofed: true }))).toBe("roofed");
    const keys = new Set(BUILDING_LEGEND.map((e) => e.key));
    for (const k of ["marked", "frame", "walls", "roofed", "leaking", "ruin"]) {
      expect(keys.has(k)).toBe(true);
    }
  });

  it("show postholes as they are dug, then posts, walls and thatch", () => {
    expect(buildingLook(hut({ progress: 0 })).posts).toBe(0);
    expect(buildingLook(hut({ progress: 0.5 })).posts).toBe(10);
    const frame = buildingLook(hut({ stage: 1, progress: 0.2 }));
    expect([frame.posts, frame.wallAlpha, frame.roofAlpha]).toEqual([20, 0, 0]);
    const walls = buildingLook(hut({ stage: 2, progress: 0.5 }));
    expect(walls.wallAlpha).toBeGreaterThan(buildingLook(hut({ stage: 2 })).wallAlpha);
    expect(walls.roofAlpha).toBe(0);
    const thatching = buildingLook(hut({ stage: 3, progress: 0.5 }));
    expect(thatching.roofAlpha).toBeGreaterThan(0);
    expect(thatching.roofAlpha).toBeLessThan(buildingLook(hut({ stage: 4, roofed: true })).roofAlpha);
  });

  it("draw a frame building's gabled roof, its ridge and its doorway once roofed", () => {
    const b = longhouse({ door: [202, 102.575], doorDir: Math.PI / 2 });
    const marks = buildingMarks(b);
    const of = <K extends Mark["kind"]>(kind: K) =>
      marks.filter((m): m is Extract<Mark, { kind: K }> => m.kind === kind);
    // The plot first, then the roof over its four corners, and the ridge along its length.
    expect(marks[0]).toMatchObject({ kind: "rect", x: 195.75, y: 97, w: 8.5, h: 6 });
    const [roof, ...others] = of("poly");
    expect(roof?.points).toEqual(b.roofOutline.flat());
    expect(roof?.fill).toEqual({ color: buildingLook(b).roofFill, alpha: 0.95 });
    expect(others).toEqual([]);
    expect(of("line")).toEqual([
      expect.objectContaining({ from: [195.75, 100], to: [204.25, 100] }),
    ]);
    // No cone, no posts under the thatch: one circle, the doorway just outside the door.
    const circles = of("circle");
    expect(circles).toHaveLength(1);
    expect(circles[0]?.x).toBeCloseTo(202, 9);
    expect(circles[0]?.y).toBeCloseTo(102.875, 9);
    expect(underRoof(b, circles[0]!.x, circles[0]!.y)).toBe(true);
  });

  it("draw a frame building going up by its walls' outline and its posts, without a roof", () => {
    const posts: [number, number][] = [
      [196.175, 97.425],
      [198.725, 97.425],
      [201.275, 97.425],
      [203.825, 97.425],
    ];
    const b = longhouse({ stage: 2, progress: 0.5, roofed: false, status: "walls going up", posts });
    const marks = buildingMarks(b);
    expect(marks.some((m) => m.kind === "line")).toBe(false);
    const polys = marks.filter((m) => m.kind === "poly");
    expect(polys).toHaveLength(1);
    expect(polys[0]).toMatchObject({ points: b.outline.flat(), stroke: { width: 0.3 } });
    expect(marks.filter((m) => m.kind === "circle")).toHaveLength(posts.length);
  });

  it("draw a roofed hut as a cone with its doorway at the eaves", () => {
    const b = hut({ stage: 4, roofed: true, status: "roofed", doorDir: 0 });
    const circles = buildingMarks(b).filter((m) => m.kind === "circle");
    expect(circles[0]).toMatchObject({ x: 100, y: 200, r: 3.6 });
    expect(circles[1]).toMatchObject({ y: 200, r: 0.35 });
    expect(circles[1]?.kind === "circle" && circles[1].x).toBeCloseTo(100 + 3.6 * 0.92, 9);
    expect(circles).toHaveLength(2);
  });

  it("are found under a point, on their plot or under their roof", () => {
    const buildings = [hut({ id: 1 }), hut({ id: 2, x: 120, plot: null })];
    expect(buildingAt(buildings, 101, 201)?.id).toBe(1);
    // A corner of the plot, outside the roof's reach.
    expect(buildingAt(buildings, 96.6, 196.6)?.id).toBe(1);
    expect(buildingAt(buildings, 122, 200)?.id).toBe(2);
    expect(buildingAt(buildings, 110, 200)).toBeNull();
  });
});
