// The Water lens (wire 1.66, M6a slice BB, step three; the observer brief §1.3): one lens on the
// map at a time, read per home, with its legend naming its scope, period and denominator
// (research 15-01 §3.1). The kernel measures and words everything; this only colours and places.

import type { CoverageHome, CoverageInfo, CoveragePlace, CoverageSource } from "./net/messages.js";

/** The lens's layers: none, *could reach*, *did come*. */
export type LensLayer = "" | "reach" | "came" | "help-now" | "help-came" | "traffic";

export const LENS_LAYERS: { value: LensLayer; label: string }[] = [
  { value: "", label: "No lens" },
  { value: "reach", label: "Water: could reach" },
  { value: "came", label: "Water: did come" },
  { value: "help-now", label: "Help at a fire: now" },
  { value: "help-came", label: "Help at a fire: did come" },
  { value: "traffic", label: "Traffic: waits at crossings and gates" },
];

/** The kernel lens a layer uses. */
export function lensId(layer: LensLayer): number {
  return layer === "traffic" ? 3 : layer.startsWith("help-") ? 2 : 1;
}

/** The Traffic lens's classes, by the seconds waited a passage (M8b slice CN, ADR-0029 §9). */
export const TRAFFIC_CLASSES: { maxS: number; label: string; colour: number }[] = [
  { maxS: 1, label: "under a second a passage", colour: 0x2f9e5a },
  { maxS: 10, label: "under ten seconds a passage", colour: 0xa6c93a },
  { maxS: 60, label: "under a minute a passage", colour: 0xe0a030 },
  { maxS: Infinity, label: "a minute or more a passage", colour: 0xd0502a },
];

/** The class of a place on the Traffic lens: its index in `TRAFFIC_CLASSES`. */
export function trafficClass(p: CoveragePlace): number {
  const perS = p.passages > 0 ? (p.waitedMin * 60) / p.passages : 0;
  const i = TRAFFIC_CLASSES.findIndex((c) => perS < c.maxS);
  return i < 0 ? TRAFFIC_CLASSES.length - 1 : i;
}

/** The place the pointer is on: the nearest within `slackM` of the point. */
export function placeAt(info: CoverageInfo | null, xM: number, yM: number, slackM: number): CoveragePlace | null {
  let best: CoveragePlace | null = null;
  let bestD = slackM;
  for (const p of info?.places ?? []) {
    const d = Math.hypot(p.x - xM, p.y - yM);
    if (d <= bestD) {
      bestD = d;
      best = p;
    }
  }
  return best;
}

const HELP_NOW = [
  { label: "route search inconclusive", colour: 0x8a8a8a, hollow: true },
  { label: "no adult in the current probe", colour: 0x92796f, hollow: true },
  { label: "adults nearby; no water by right or stored load", colour: 0xd6a458, hollow: true },
  { label: "one adult nearby, with water", colour: 0x659bc9, hollow: false },
  { label: "several adults nearby, with water", colour: 0x436a96, hollow: false },
];
const HELP_CAME = [
  { label: "no fire exposure in the period", colour: 0x8a8a8a, hollow: true },
  { label: "no useful work recorded", colour: 0x92796f, hollow: true },
  { label: "some exposures received work", colour: 0xd6a458, hollow: false },
  { label: "all exposures received work", colour: 0x659bc9, hollow: false },
];
function helpClass(h: CoverageHome, actual: boolean): number {
  const f = h.fireHelp;
  if (!f) return 0;
  if (actual) return f.fireEpisodes === 0 ? 0 : f.helpedEpisodes === 0 ? 1 : f.helpedEpisodes < f.fireEpisodes ? 2 : 3;
  return f.adultsNow === 0 ? (f.unknownRoutes > 0 ? 0 : 1) : !f.waterAvailable ? 2 : f.adultsNow === 1 ? 3 : 4;
}

/** *Could reach*'s fixed bands, by walking minutes to the nearest water, nearest first; a home
 * with none within the search is its own class, drawn hollow (never as zero). */
export const REACH_BANDS: { maxMin: number; label: string; colour: number }[] = [
  { maxMin: 1, label: "within a minute's walk", colour: 0x2f9e5a },
  { maxMin: 5, label: "within 5 minutes", colour: 0xa6c93a },
  { maxMin: 15, label: "within 15 minutes", colour: 0xe0a030 },
  { maxMin: Infinity, label: "farther, within an hour", colour: 0xd0502a },
];
/** A home with no water within the search. */
export const NONE_COLOUR = 0x8a8a8a;

/** *Did come*'s classes, by how the home stands to the source (wire codes 1-5). */
export const SOURCE_KINDS: { kind: number; label: string; colour: number }[] = [
  { kind: 1, label: "its own well", colour: 0x3d8fd6 },
  { kind: 2, label: "a well by right (its polity's or kin's)", colour: 0x7a6fd8 },
  { kind: 3, label: "another's well", colour: 0xd0502a },
  { kind: 4, label: "a spring", colour: 0x62c3e8 },
  { kind: 5, label: "the water's edge", colour: 0xe8e8d8 },
];

/** The band of a walk to the nearest water: its index in `REACH_BANDS`, or -1 for none within the
 * search. */
export function reachBand(minutes: number | null): number {
  if (minutes === null) return -1;
  const i = REACH_BANDS.findIndex((b) => minutes < b.maxMin);
  return i < 0 ? REACH_BANDS.length - 1 : i;
}

export function kindColour(kind: number): number {
  return SOURCE_KINDS.find((k) => k.kind === kind)?.colour ?? NONE_COLOUR;
}

/** A mark of the lens: a home's disc (hollow when it has no water within the search) or a line
 * from a home to a source. */
export type LensMark =
  | { type: "home"; x: number; y: number; colour: number; hollow: boolean }
  | { type: "line"; x0: number; y0: number; x1: number; y1: number; colour: number; alpha: number; weight: number };

/** The marks of `layer` over `info`: *could reach*, each home by its band with a thin line to the
 * nearest water; *did come*, a line from each home to each source drawn at, as opaque as the share
 * of the period's days it was drawn there, and each home by the source drawn at most. */
export function lensMarks(info: CoverageInfo, layer: LensLayer): LensMark[] {
  const out: LensMark[] = [];
  if (!layer || info.lens !== lensId(layer)) return out;
  if (layer === "traffic") {
    // Each place a disc, coloured by its wait a passage; hollow when nobody waited at all.
    for (const p of info.places) {
      const c = TRAFFIC_CLASSES[trafficClass(p)]!;
      out.push({ type: "home", x: p.x, y: p.y, colour: c.colour, hollow: p.waitedMin <= 0 });
    }
    return out;
  }
  if (layer.startsWith("help-")) {
    const actual = layer === "help-came";
    const classes = actual ? HELP_CAME : HELP_NOW;
    for (const h of info.homes) {
      const c = classes[helpClass(h, actual)]!;
      out.push({ type: "home", x: h.x, y: h.y, colour: c.colour, hollow: c.hollow });
    }
    return out;
  }
  if (layer === "reach") {
    for (const h of info.homes) {
      const n = h.nearest;
      const band = reachBand(n?.minutes ?? null);
      const colour = band < 0 ? NONE_COLOUR : REACH_BANDS[band]!.colour;
      if (n) out.push({ type: "line", x0: h.x, y0: h.y, x1: n.x, y1: n.y, colour, alpha: 0.6, weight: 1 });
      out.push({ type: "home", x: h.x, y: h.y, colour, hollow: band < 0 });
    }
  } else if (layer === "came") {
    const days = Math.max(1, info.days);
    for (const h of info.homes) {
      for (const s of h.drew) {
        const share = Math.min(1, s.days / days);
        out.push({
          type: "line",
          x0: h.x,
          y0: h.y,
          x1: s.x,
          y1: s.y,
          colour: kindColour(s.kind),
          alpha: 0.25 + 0.7 * share,
          weight: 1 + 2 * share,
        });
      }
      const most = h.drew[0];
      out.push({
        type: "home",
        x: h.x,
        y: h.y,
        colour: most ? kindColour(most.kind) : NONE_COLOUR,
        hollow: !most,
      });
    }
  }
  return out;
}

/** The home the pointer is on: the nearest within `slackM` of the point. */
export function homeAt(info: CoverageInfo | null, xM: number, yM: number, slackM: number): CoverageHome | null {
  let best: CoverageHome | null = null;
  let bestD = slackM;
  for (const h of info?.homes ?? []) {
    const d = Math.hypot(h.x - xM, h.y - yM);
    if (d <= bestD) {
      bestD = d;
      best = h;
    }
  }
  return best;
}

/** The legend of `layer` over `info`: its title, scope, period and denominator, its classes in
 * order, and its counts in the kernel's words. */
export function legend(
  info: CoverageInfo,
  layer: Exclude<LensLayer, "">,
): { title: string; lines: string[]; classes: { label: string; colour: number; hollow: boolean }[]; counts: string } {
  if (layer === "traffic") {
    return {
      title: info.title,
      lines: [`Scope: ${info.scope}`, `Period: ${info.periods[0] ?? ""}`, `Per: ${info.denominator}`],
      classes: [
        ...TRAFFIC_CLASSES.map((c) => ({ label: c.label, colour: c.colour, hollow: false })),
        { label: "nobody waited at all", colour: TRAFFIC_CLASSES[0]!.colour, hollow: true },
      ],
      counts: info.summary.filter((l) => l).join(" "),
    };
  }
  if (layer.startsWith("help-")) {
    const actual = layer === "help-came";
    const index = actual ? 1 : 0;
    return {
      title: `${info.title}: ${actual ? "did come" : "now"}`,
      lines: [`Scope: ${info.scope}`, `Period: ${info.periods[index] ?? ""}`, `Per: ${info.denominator}`],
      classes: actual ? HELP_CAME : HELP_NOW,
      counts: info.summary[index] ?? "",
    };
  }
  const reach = layer === "reach";
  const classes = reach
    ? [
        ...REACH_BANDS.map((b) => ({ label: b.label, colour: b.colour, hollow: false })),
        {
          label: `none within ${Math.round(info.limitMin)} minutes' walk`,
          colour: NONE_COLOUR,
          hollow: true,
        },
      ]
    : [
        ...SOURCE_KINDS.map((k) => ({ label: k.label, colour: k.colour, hollow: false })),
        { label: "no draw recorded", colour: NONE_COLOUR, hollow: true },
      ];
  const counts = info.summary[reach ? 0 : 1] ?? "";
  return {
    title: `${info.title}: ${reach ? "could reach" : "did come"}`,
    lines: [`Scope: ${info.scope}`, `Period: ${info.periods[reach ? 0 : 1] ?? ""}`, `Per: ${info.denominator}`],
    classes,
    counts,
  };
}

/** A source in a line: "Bo's household's well, 0.3 min". */
export function sourceText(s: CoverageSource): string {
  return s.minutes === null ? s.name : `${s.name}, ${s.minutes.toFixed(1)} min`;
}
