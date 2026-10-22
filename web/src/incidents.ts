// Incidents in words (wire 1.64, M6a slice BB, ADR-0015 §8): every kind in one shape, its three
// layers kept apart. The kernel renders what happened, what people know and what the polity did;
// the observer only labels the layers, counts and draws.

import { lawDayText } from "./government.js";
import type { IncidentStep, IncidentTotal, OutbreakInfo } from "./net/messages.js";

/** The kinds the panel can show, by their wire code (0: every kind). */
export const INCIDENT_KINDS: { code: number; label: string }[] = [
  { code: 0, label: "Every kind" },
  { code: 1, label: "Takings" },
  { code: 2, label: "Illness" },
  { code: 4, label: "Fires" },
  { code: 10, label: "Waits at crossings and gates" },
  { code: 11, label: "Shaken buildings" },
];

/** "taking", "illness", "fire": the class a row carries. */
export function kindClass(kind: number): string {
  return kind === 1
    ? "taking"
    : kind === 2
      ? "illness"
      : kind === 4
        ? "fire"
        : kind === 10
          ? "wait"
          : kind === 11
            ? "shaken"
            : "other";
}

function count(n: number, one: string, many: string): string {
  return n === 1 ? `1 ${one}` : `${n} ${many}`;
}

/**
 * A kind's totals in a sentence, counts before anything else (ADR-0015 §8): "Takings: 12 in all;
 * 5 known to someone; 2 before a gathering", "Illness: 9 cases; 3 known to someone now; none named by a
 * law; 9 the observer's", "Fires: 7 buildings burned; 7 the observer's".
 */
export function totalsLine(t: IncidentTotal): string {
  if (t.kind === 1) {
    if (t.all === 0) return "Takings: none";
    return `Takings: ${t.all} in all; ${t.known} known to someone; ${t.polity} before a gathering`;
  }
  if (t.kind === 2) {
    if (t.all === 0) return "Illness: no case";
    const observer = t.observer > 0 ? `; ${t.observer} the observer's` : "";
    const law = t.polity === 0 ? "none named by a law" : `${t.polity} named by a law`;
    // Word of sickness fades as news does, so what is known of past cases is what is known now.
    return `Illness: ${count(t.all, "case", "cases")}; ${t.known} known to someone now; ${law}${observer}`;
  }
  if (t.kind === 4) {
    // Nobody in the world sees or tells of a fire yet (M6b slice BD), so only the truth is counted.
    if (t.all === 0) return "Fires: none";
    const observer = t.observer > 0 ? `; ${t.observer} the observer's` : "";
    return `Fires: ${count(t.all, "building burned", "buildings burned")}${observer}`;
  }
  if (t.kind === 10) {
    // A wait of a quarter of an hour or more at a crossing or a gate (wire 1.85).
    if (t.all === 0) return "Waits: none of a quarter of an hour";
    return `Waits: ${count(t.all, "of a quarter of an hour or more", "of a quarter of an hour or more")}`;
  }
  if (t.kind === 11) {
    // Buildings that fell or cracked when the ground shook (wire 1.91); nobody in the world tells
    // of them as such yet, so only the truth is counted.
    if (t.all === 0) return "Shaken buildings: none fell or cracked";
    const observer = t.observer > 0 ? `; ${t.observer} the observer's` : "";
    return `Shaken buildings: ${count(t.all, "fell or cracked", "fell or cracked")}${observer}`;
  }
  return `${t.all} in all`;
}

/** "14:05", a minute's time of day. */
function clock(minute: number): string {
  const m = ((minute % 1440) + 1440) % 1440;
  return `${String(Math.floor(m / 60)).padStart(2, "0")}:${String(m % 60).padStart(2, "0")}`;
}

/**
 * When a step was reached, as precisely as it is known: "3 May of year 2" for a day, "3 May of
 * year 2, 14:05" for a minute, "not reached" or "not recorded" otherwise (15-02 §1.9).
 */
export function stepWhen(s: IncidentStep): string {
  if (s.state !== "reached") return s.state;
  return s.byDay ? lawDayText(s.minute) : `${lawDayText(s.minute)}, ${clock(s.minute)}`;
}

/** A step in a line: "Fell ill: 3 May of year 2 — for 4 days, severely". */
export function stepText(s: IncidentStep): string {
  const words = s.words ? ` — ${s.words}` : "";
  return `${s.label}: ${stepWhen(s)}${words}`;
}

/**
 * The outbreak's curve as bars, one a day: each day's height for those who fell ill and for those
 * who took it, scaled to the tallest day. Days with none keep their place.
 */
export function curveBars(o: Pick<OutbreakInfo, "took" | "onset">): { took: number; onset: number }[] {
  const days = Math.max(o.took.length, o.onset.length);
  const tallest = Math.max(1, ...o.took, ...o.onset);
  return Array.from({ length: days }, (_, d) => ({
    took: (o.took[d] ?? 0) / tallest,
    onset: (o.onset[d] ?? 0) / tallest,
  }));
}
