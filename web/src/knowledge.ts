// Knowledge (M3b slice M; ADR-0008): what the knowledge panel and the inspector say about what
// people know. Pure functions. The kernel keeps each person's knowledge and derives what a
// settlement knows from its people; the observer only shows it.

import type { KnowLine, PersonRef, TechniqueHere, TechniqueInfo } from "./net/messages.js";

/** Names a listing gives before "and 3 more". */
export const NAMES_SHOWN = 6;

/** Hours: "12 h", "0.5 h". */
function hoursText(h: number): string {
  return h < 10 ? `${(Math.round(h * 10) / 10).toString()} h` : `${Math.round(h)} h`;
}

/** People in words: "Wren (34), Ash (29) and 3 more", "Wren (34) and Ash (29)", or "nobody". */
export function namesText(people: PersonRef[], shown = NAMES_SHOWN): string {
  if (people.length === 0) return "nobody";
  const named = people.slice(0, shown).map((p) => `${p.name} (${Math.floor(p.ageYears)})`);
  const rest = people.length - named.length;
  if (rest > 0) return `${named.join(", ")} and ${rest} more`;
  if (named.length === 1) return named[0]!;
  return `${named.slice(0, -1).join(", ")} and ${named.at(-1)}`;
}

/** A technique's name, or a placeholder for one the welcome does not list. */
export function techniqueName(techniques: TechniqueInfo[], index: number): string {
  return techniques[index]?.name ?? `technique ${index}`;
}

/**
 * What a person knows of a technique, in words: "brought up with it by Wren", "learning, 12 of
 * 50 h (taught by Ash)", "heard of (introduced by the observer)".
 */
export function knowText(line: KnowLine): string {
  const source = line.source ? ` (${line.source})` : "";
  switch (line.state) {
    case "known":
      return line.source || "known";
    case "learning":
      return `learning, ${hoursText(line.hours)} of ${hoursText(line.learnH)}${source}`;
    case "heard":
      return `heard of${source}`;
  }
}

const STATE_ORDER: Record<KnowLine["state"], number> = { known: 0, learning: 1, heard: 2 };

/** A row of the inspector's "Knows" list. */
export interface KnowRow {
  technique: number;
  name: string;
  state: KnowLine["state"];
  text: string;
}

/** What a person knows, then is learning, then has heard of, each group by name. */
export function knowRows(lines: KnowLine[], techniques: TechniqueInfo[]): KnowRow[] {
  return lines
    .map((line) => ({
      technique: line.technique,
      name: techniqueName(techniques, line.technique),
      state: line.state,
      text: knowText(line),
    }))
    .sort((a, b) => STATE_ORDER[a.state] - STATE_ORDER[b.state] || a.name.localeCompare(b.name));
}

/** The techniques the observer can still introduce to someone: those they do not know, by name. */
export function introducible(
  lines: KnowLine[],
  techniques: TechniqueInfo[],
): { technique: number; name: string; heard: boolean }[] {
  const known = new Set(lines.filter((l) => l.state === "known").map((l) => l.technique));
  const aware = new Set(lines.map((l) => l.technique));
  return techniques
    .map((t, i) => ({ technique: i, name: t.name, heard: aware.has(i) }))
    .filter((t) => !known.has(t.technique))
    .sort((a, b) => a.name.localeCompare(b.name));
}

/** A row of a settlement's knowledge table. */
export interface TechniqueRow {
  technique: number;
  name: string;
  known: boolean;
  /** The kernel's words: "known by 12", "lost in year 9 with Wren". */
  status: string;
  /** "9 of 12 in the last year", or "" when nobody knows it. */
  practised: string;
  /** Who knows it ("nobody"); who is learning it and who has only heard of it (null for none). */
  knowers: string;
  learners: string | null;
  heard: string | null;
  /** What a competent person can do, and what it needs known first ("" for nothing). */
  can: string;
  requires: string;
  history: string[];
  /** What its builders there have seen of its buildings, in the kernel's words ("" for none). */
  trust: string;
}

/** A settlement's techniques as rows: those known there first, each group in the kernel's order. */
export function techniqueRows(here: TechniqueHere[], techniques: TechniqueInfo[]): TechniqueRow[] {
  const rows = here.map((t) => ({
    technique: t.technique,
    name: techniqueName(techniques, t.technique),
    known: t.known,
    status: t.status,
    practised:
      t.knowers.length > 0 ? `${t.practisedLastYear} of ${t.knowers.length} in the last year` : "",
    knowers: namesText(t.knowers),
    learners: t.learners.length > 0 ? namesText(t.learners) : null,
    heard: t.heard.length > 0 ? namesText(t.heard) : null,
    can: techniques[t.technique]?.can ?? "",
    requires: techniques[t.technique]?.requires ?? "",
    history: t.history,
    trust: t.trust,
  }));
  return [...rows.filter((r) => r.known), ...rows.filter((r) => !r.known)];
}

/** A settlement's knowledge in a sentence: "Knows 8 techniques; has lost 1." */
export function summaryText(here: TechniqueHere[]): string {
  const known = here.filter((t) => t.known).length;
  const lost = here.filter((t) => !t.known && t.status.startsWith("lost")).length;
  const word = (n: number) => `${n} technique${n === 1 ? "" : "s"}`;
  const knows = known === 0 ? "Knows no technique" : `Knows ${word(known)}`;
  return lost > 0 ? `${knows}; has lost ${lost}.` : `${knows}.`;
}
