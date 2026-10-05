// Terrain colouring: hypsometric tint, hillshade, contours and water, computed from the host's
// raster tiles. Pure functions; the map view turns the pixels into textures.

import { RasterFormat, type RasterTile } from "../net/messages.js";

/** Water classes of the water raster (civ-world). */
export const WATER_LAND = 0;
export const WATER_RIVER = 1;
export const WATER_LAKE = 2;
export const WATER_OCEAN = 3;

/** Largest side, in samples, of the whole-map texture. */
export const BASE_MAX_SIDE = 1024;

/** The finest level that shows the whole map in at most BASE_MAX_SIDE samples a side. */
export function baseLevel(width: number, height: number): number {
  let level = 0;
  while (Math.ceil(Math.max(width, height) / 2 ** level) > BASE_MAX_SIDE) level++;
  return level;
}

/** Elevations of a tile, metres. */
export function elevations(tile: RasterTile): Float32Array {
  const n = tile.width * tile.height;
  const out = new Float32Array(n);
  const view = new DataView(tile.data.buffer, tile.data.byteOffset, tile.data.byteLength);
  if (tile.format === RasterFormat.U16) {
    if (tile.data.byteLength < 2 * n) throw new Error("elevation tile is short");
    for (let i = 0; i < n; i++) out[i] = view.getUint16(2 * i, true) * tile.scale + tile.offset;
  } else if (tile.format === RasterFormat.F32) {
    if (tile.data.byteLength < 4 * n) throw new Error("elevation tile is short");
    for (let i = 0; i < n; i++) out[i] = view.getFloat32(4 * i, true);
  } else {
    throw new Error(`elevation tile has format ${tile.format}`);
  }
  return out;
}

/** Water classes of a tile. */
export function classes(tile: RasterTile): Uint8Array {
  const n = tile.width * tile.height;
  if (tile.format !== RasterFormat.U8 || tile.data.byteLength < n) {
    throw new Error("water tile is not U8 of the right size");
  }
  return tile.data.slice(0, n);
}

export interface ShadeOptions {
  /** Metres per sample. */
  cellM: number;
  seaLevelM: number;
  /** The whole map's elevation range, so every patch shares one colour scale. */
  minM: number;
  maxM: number;
  /** Slope exaggeration for the hillshade. */
  zFactor?: number;
  /** Contour interval, metres (0 = none). */
  contourM?: number;
  season?: Season;
  snowLineM?: number;
}

export type Season = "spring" | "summer" | "autumn" | "winter";

export function seasonOf(name: string | undefined | null): Season {
  if (name === "winter" || name === "autumn" || name === "spring") return name;
  return "summer";
}

export function seasonKey(season: string | undefined | null, snowLineM: number | undefined | null): string {
  const line = snowLineM === undefined || snowLineM === null || snowLineM < 0 ? -1 : Math.round(snowLineM);
  return `${seasonOf(season)}:${line}`;
}

type Rgb = [number, number, number];

const LAND_STOPS: [number, Rgb][] = [
  [0.0, [104, 133, 88]],
  [0.18, [128, 152, 98]],
  [0.4, [170, 168, 118]],
  [0.62, [165, 140, 106]],
  [0.82, [146, 132, 120]],
  [1.0, [222, 218, 210]],
];
const SPRING_STOPS: [number, Rgb][] = [
  [0.0, [118, 148, 98]],
  [0.18, [140, 164, 105]],
  [0.4, [178, 175, 120]],
  [0.62, [168, 145, 110]],
  [0.82, [148, 135, 122]],
  [1.0, [222, 218, 210]],
];
const AUTUMN_STOPS: [number, Rgb][] = [
  [0.0, [130, 125, 80]],
  [0.18, [150, 140, 85]],
  [0.4, [180, 160, 100]],
  [0.62, [170, 145, 110]],
  [0.82, [150, 135, 122]],
  [1.0, [222, 218, 210]],
];
const WINTER_STOPS: [number, Rgb][] = [
  [0.0, [95, 110, 90]],
  [0.18, [110, 120, 100]],
  [0.4, [140, 140, 115]],
  [0.62, [145, 135, 115]],
  [0.82, [140, 130, 120]],
  [1.0, [222, 218, 210]],
];
const RIVER: Rgb = [58, 112, 166];
const LAKE: Rgb = [70, 120, 160];
const OCEAN_SHALLOW: Rgb = [84, 138, 176];
const OCEAN_DEEP: Rgb = [36, 78, 118];

function landColour(t: number, season: Season): Rgb {
  const x = Math.min(1, Math.max(0, t));
  const stops = season === "spring" ? SPRING_STOPS :
                season === "autumn" ? AUTUMN_STOPS :
                season === "winter" ? WINTER_STOPS : LAND_STOPS;
  for (let i = 1; i < stops.length; i++) {
    const [t1, c1] = stops[i]!;
    if (x <= t1) {
      const [t0, c0] = stops[i - 1]!;
      const f = (x - t0) / (t1 - t0);
      return [c0[0] + (c1[0] - c0[0]) * f, c0[1] + (c1[1] - c0[1]) * f, c0[2] + (c1[2] - c0[2]) * f];
    }
  }
  return stops[stops.length - 1]![1];
}

/** A round contour interval giving about `lines` contours over `range` metres. */
export function contourInterval(range: number, lines = 16): number {
  if (!(range > 0)) return 0;
  const raw = range / lines;
  const pow = 10 ** Math.floor(Math.log10(raw));
  for (const step of [1, 2, 2.5, 5, 10]) {
    if (raw <= step * pow) return step * pow;
  }
  return 10 * pow;
}

// Light from the north-west, 45° above the horizon (x east, y south).
const SIN_ALT = Math.SQRT1_2;
const LIGHT_H = Math.SQRT1_2 * Math.SQRT1_2; // cos(alt) / √2 per horizontal axis

/** RGBA pixels for `w × h` samples of elevation and water. Slopes use Horn's 3×3 method, the
 * usual GIS hillshade, which also keeps the terrain's grid-aligned creases from dominating. */
export function shade(
  elev: Float32Array,
  water: Uint8Array,
  w: number,
  h: number,
  opts: ShadeOptions,
): Uint8ClampedArray {
  const out = new Uint8ClampedArray(w * h * 4);
  const z = opts.zFactor ?? 1.25;
  const landMin = Math.max(opts.minM, opts.seaLevelM);
  const span = Math.max(1, opts.maxM - landMin);
  const contour = opts.contourM ?? 0;
  const inv8c = z / (8 * opts.cellM);
  for (let y = 0; y < h; y++) {
    const up = (y > 0 ? y - 1 : y) * w;
    const row = y * w;
    const down = (y < h - 1 ? y + 1 : y) * w;
    for (let x = 0; x < w; x++) {
      const i = row + x;
      const e = elev[i]!;
      const cls = water[i]!;
      let r: number, g: number, b: number;
      if (cls === WATER_OCEAN) {
        const t = Math.min(1, Math.max(0, (opts.seaLevelM - e) / 60));
        r = OCEAN_SHALLOW[0] + (OCEAN_DEEP[0] - OCEAN_SHALLOW[0]) * t;
        g = OCEAN_SHALLOW[1] + (OCEAN_DEEP[1] - OCEAN_SHALLOW[1]) * t;
        b = OCEAN_SHALLOW[2] + (OCEAN_DEEP[2] - OCEAN_SHALLOW[2]) * t;
      } else if (cls === WATER_LAKE) {
        [r, g, b] = LAKE;
      } else {
        const xl = x > 0 ? x - 1 : x;
        const xr = x < w - 1 ? x + 1 : x;
        const a = elev[up + xl]!;
        const bb = elev[up + x]!;
        const c = elev[up + xr]!;
        const d = elev[row + xl]!;
        const f = elev[row + xr]!;
        const gg = elev[down + xl]!;
        const hh = elev[down + x]!;
        const ii = elev[down + xr]!;
        const dzdx = (c + 2 * f + ii - (a + 2 * d + gg)) * inv8c;
        const dzdy = (gg + 2 * hh + ii - (a + 2 * bb + c)) * inv8c;
        const lit = ((dzdx + dzdy) * LIGHT_H + SIN_ALT) / Math.sqrt(dzdx * dzdx + dzdy * dzdy + 1);
        const light = Math.min(1.35, Math.max(0.3, lit / SIN_ALT));
        let base = landColour((e - landMin) / span, opts.season ?? "summer");
        if (opts.snowLineM !== undefined && opts.snowLineM >= 0) {
          const snow = Math.min(1, Math.max(0, (e - opts.snowLineM) / 50));
          if (snow > 0) {
            base = [
              base[0] + (245 - base[0]) * snow,
              base[1] + (248 - base[1]) * snow,
              base[2] + (250 - base[2]) * snow,
            ];
          }
        }
        let k = 0.35 + 0.65 * light;
        if (contour > 0) {
          const band = Math.floor(e / contour);
          if (band !== Math.floor(f / contour) || band !== Math.floor(hh / contour)) k *= 0.9;
        }
        r = base[0] * k;
        g = base[1] * k;
        b = base[2] * k;
        if (cls === WATER_RIVER) {
          r = r * 0.2 + RIVER[0] * 0.8;
          g = g * 0.2 + RIVER[1] * 0.8;
          b = b * 0.2 + RIVER[2] * 0.8;
        }
      }
      const o = i * 4;
      out[o] = r;
      out[o + 1] = g;
      out[o + 2] = b;
      out[o + 3] = 255;
    }
  }
  return out;
}
