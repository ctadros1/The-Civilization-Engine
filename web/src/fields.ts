// How fields look on the map (M1 slice C): a colour for where each field is in its year, which
// field lies under a point, and who holds it (M3a slice K). Pure functions, so they can be tested
// without a renderer.

import { formatPercent, formatSimMinute } from "./format.js";
import type { FieldInfo, GoodInfo } from "./net/messages.js";

/** A field's look: fill colour and opacity. */
export interface FieldLook {
  fill: number;
  alpha: number;
}

/** The stages a field is drawn in, in the order the legend lists them. */
export const FIELD_LEGEND: { key: string; label: string; fill: number }[] = [
  { key: "woodland", label: "woodland being cleared", fill: 0x5f6b3c },
  { key: "new", label: "new ground", fill: 0x8f7454 },
  { key: "fallow", label: "fallow", fill: 0xa48a63 },
  { key: "prepared", label: "dug for sowing", fill: 0x6b4f37 },
  { key: "sown", label: "growing crop", fill: 0x86b552 },
  { key: "ripe", label: "ripe", fill: 0xd8b445 },
  { key: "reaped", label: "reaped, to thresh", fill: 0xe3d3a1 },
];

const FILLS: Record<string, number> = Object.fromEntries(FIELD_LEGEND.map((e) => [e.key, e.fill]));

/** Which legend entry a field is drawn as. */
export function fieldKey(f: FieldInfo): string {
  if (f.stage === "fallow" && f.newGround) return f.woodland ? "woodland" : "new";
  if (f.stage === "sown" && f.ripe) return "ripe";
  return f.stage;
}

/** How a field is drawn: ground still being broken shows fainter until the work is done. */
export function fieldLook(f: FieldInfo): FieldLook {
  const key = fieldKey(f);
  const fill = FILLS[key] ?? 0xa48a63;
  const alpha = key === "woodland" || key === "new" ? 0.45 + 0.4 * f.progress : 0.85;
  return { fill, alpha };
}

/** The field under a point in metres, if any (the last drawn wins where fields touch). */
export function fieldAt(fields: readonly FieldInfo[], x: number, y: number): FieldInfo | null {
  for (let i = fields.length - 1; i >= 0; i--) {
    const f = fields[i]!;
    if (x >= f.x && x < f.x + f.w && y >= f.y && y < f.y + f.h) return f;
  }
  return null;
}

/**
 * Who holds a field and on what terms, in words (M3a slice K): "the settlement's, given out to
 * work", "let for 25% of its grain until 1 Mar, year 3", "held by the household that works it".
 * `living` holds the households someone lives in.
 */
export function tenureText(f: FieldInfo, living: ReadonlySet<number>): string {
  const worked = living.has(f.household);
  if (f.holderSettlement !== 0) {
    return worked ? "the settlement's, given out to work" : "the settlement's, not given out";
  }
  if (!worked) return living.has(f.holder) ? "its tenant is gone" : "nobody holds it now";
  if (f.leaseUntilMinute >= 0) {
    return `let for ${formatPercent(f.leaseShare)} of its grain until ${formatSimMinute(f.leaseUntilMinute)}`;
  }
  return f.holder === f.household
    ? "held by the household that works it"
    : "worked for the household that holds it";
}

/** Wire 1.115: the field work offered for hire, by the field's hire task. */
const HIRE_WORK: Record<number, string> = {
  1: "reaping",
  2: "threshing",
  3: "preparing ground",
  4: "preparing and sowing",
};

/**
 * The field work its household would hire now for a field, in words (wire 1.115): "hiring 14 h
 * of reaping at 0.62 kg of emmer an hour"; empty when it offers none, or from an older kernel.
 */
export function hireText(f: FieldInfo, goods: readonly GoodInfo[]): string {
  const work = HIRE_WORK[f.hireTask];
  if (!work || f.hireHours <= 0) return "";
  const good = (goods[f.hireGood]?.name ?? "grain").toLowerCase();
  const hours = f.hireHours < 10 ? f.hireHours.toFixed(1) : Math.round(f.hireHours).toString();
  return `hiring ${hours} h of ${work} at ${f.hireKgPerHour.toFixed(2)} kg of ${good} an hour`;
}
