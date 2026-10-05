// Deposits in the ground (M3b slice Q; ADR-0010 §1): where they lie on the map, how they look,
// and what the readout says of them. The kernel decides everything; this only shows it.

import type { DepositInfo, GoodInfo } from "./net/messages.js";

/** The deposit under a point, metres: the one whose centre is nearest, within its radius. */
export function depositAt(deposits: DepositInfo[], xM: number, yM: number): DepositInfo | null {
  let best: DepositInfo | null = null;
  let bestD = Infinity;
  for (const d of deposits) {
    const dist = Math.hypot(d.x - xM, d.y - yM);
    if (dist <= d.radiusM && dist < bestD) {
      best = d;
      bestD = dist;
    }
  }
  return best;
}

/** A colour for a good, the same each time for the same good id. */
export function goodColour(id: string): number {
  let h = 2166136261;
  for (let i = 0; i < id.length; i++) h = Math.imul(h ^ id.charCodeAt(i), 16777619) >>> 0;
  // Earthy hues: ochres, greys, browns and slates, never the water's blue.
  const palette = [0xb5804a, 0x8f8a80, 0x6b5a46, 0x5f6a72, 0xa86f3c, 0x7d7462];
  return palette[h % palette.length]!;
}

/** How a deposit is drawn: its colour, how solid, and whether it is outlined only. Known ones are
 * solid; ones no settlement has found are faint, as only the observer sees them. */
export function depositLook(d: DepositInfo, goods: GoodInfo[]): { colour: number; alpha: number; outline: boolean } {
  const colour = goodColour(goods[d.good]?.id ?? String(d.good));
  return d.knownBy.length > 0 ? { colour, alpha: 0.55, outline: false } : { colour, alpha: 0.18, outline: true };
}

/** A mass in words: "850 kg", "1.2 t", "3,400 t". */
function mass(kg: number): string {
  if (kg < 1000) return `${Math.round(kg)} kg`;
  const t = kg / 1000;
  return t < 10 ? `${t.toFixed(1)} t` : `${Math.round(t).toLocaleString("en")} t`;
}

/** The readout's words for a deposit: "clay, 16 m across, showing at the surface; 975 t left;
 * found by Ash of Alderford in year 1". */
export function depositWords(d: DepositInfo, goods: GoodInfo[]): string {
  const what = (goods[d.good]?.name ?? "a deposit").toLowerCase();
  const across = `${Math.round(d.radiusM * 2)} m across`;
  const where = d.exposed ? "showing at the surface" : `under ${d.coverM.toFixed(1)} m of ground`;
  const known = d.finds.length > 0 ? d.finds.join("; ") : "not yet found";
  return `${what}, ${across}, ${where}; ${mass(d.leftKg)} left; ${known}`;
}

/** The goods a deposit can be of, for the observer's tool: the materials, by name. */
export function materialGoods(goods: GoodInfo[]): GoodInfo[] {
  return goods.filter((g) => g.purpose === "material").sort((a, b) => a.name.localeCompare(b.name));
}
