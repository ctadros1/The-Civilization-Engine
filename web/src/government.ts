// A settlement's polity and its laws in words (wire 1.27, ADR-0013). The kernel renders what a
// law is, how it was decided, why each member stood where they did, and what the polity would be
// called (1.29); the observer only joins the pieces.

import { formatKg, simDate } from "./format.js";
import type { LawLine, PolityLine, StanceLine } from "./net/messages.js";

const MONTHS = [
  "January",
  "February",
  "March",
  "April",
  "May",
  "June",
  "July",
  "August",
  "September",
  "October",
  "November",
  "December",
];

/** "4 May of year 3". */
export function lawDayText(minute: number): string {
  const d = simDate(minute);
  return `${d.day} ${MONTHS[d.month - 1] ?? "?"} of year ${d.year}`;
}

/** "in force", "turned down", "lapsed", "before the gathering on 9 May of year 3". */
export function statusText(l: Pick<LawLine, "status" | "outcome" | "meetsMinute">): string {
  if (l.status === "in force") return "in force";
  if (l.status === "lapsed") return "lapsed: the one it named is gone";
  if (l.status === "superseded") return "superseded by a later amendment";
  if (l.status === "proposed") return `before the gathering on ${lawDayText(l.meetsMinute)}`;
  switch (l.outcome) {
    case "tied":
      return "failed: evenly split";
    case "no quorum":
      return "failed: too few came";
    default:
      return "turned down";
  }
}

/** "24 adults", or when the custom admits fewer, "24 adults, 9 of whom may decide". */
export function membersText(p: Pick<PolityLine, "members" | "bodyMembers">): string {
  const adults = `${p.members} adults`;
  return p.bodyMembers < p.members ? `${adults}, ${p.bodyMembers} of whom may decide` : adults;
}

/** "for: their household stands to gain". */
export function stanceText(s: Pick<StanceLine, "stance" | "why">): string {
  return `${s.stance}: ${s.why}`;
}

/** Times, with the noun agreeing: "1 time", "3 times". */
function times(n: number): string {
  return `${n} time${n === 1 ? "" : "s"}`;
}

/**
 * What the levy brought in and what people did, in a line: "paid 249 times (326 kg); kept back
 * 16 times, could not pay 2, did not know of it 1 (18 kg withheld)". "" before anyone owed it.
 */
export function levyText(
  l: Pick<LawLine, "complied" | "couldNot" | "evaded" | "unaware" | "leviedKg" | "withheldKg">,
): string {
  if (l.complied + l.couldNot + l.evaded + l.unaware === 0) return "";
  const parts = [`paid ${times(l.complied)} (${formatKg(l.leviedKg)})`];
  const short: string[] = [];
  if (l.evaded > 0) short.push(`kept back ${times(l.evaded)}`);
  if (l.couldNot > 0) short.push(`could not pay ${l.couldNot}`);
  if (l.unaware > 0) short.push(`did not know of it ${l.unaware}`);
  if (short.length > 0) parts.push(`${short.join(", ")} (${formatKg(l.withheldKg)} withheld)`);
  return parts.join("; ");
}

/**
 * How often a curfew was broken (wire 1.33): "broken 12 times by people who knew of it, 3 by
 * people who did not". "" while nobody has broken it.
 */
export function brokenText(l: Pick<LawLine, "broken" | "brokenUnaware">): string {
  if (l.broken + l.brokenUnaware === 0) return "";
  const parts: string[] = [];
  if (l.broken > 0) parts.push(`${times(l.broken)} by people who knew of it`);
  if (l.brokenUnaware > 0) {
    const n = l.brokenUnaware;
    parts.push(
      parts.length > 0 ? `${n} by people who did not` : `${times(n)} by people who did not know of it`,
    );
  }
  return `broken ${parts.join(", ")}`;
}

/** "the store answered 3 asks (45 kg); 1 it could not". "" before anyone asked. */
export function reliefText(l: Pick<LawLine, "relieved" | "reliefKg" | "unanswered">): string {
  if (l.relieved + l.unanswered === 0) return "";
  const asks = `${l.relieved} ask${l.relieved === 1 ? "" : "s"}`;
  const text = `the store answered ${asks} (${formatKg(l.reliefKg)})`;
  return l.unanswered > 0 ? `${text}; ${l.unanswered} it could not` : text;
}

/**
 * What a polity would be called, with what qualifies it and how sure: "Council community: a
 * storekeeper's office; its levy mostly paid (confidence 0.67)". "" before the kernel labels it.
 */
export function labelText(
  p: Pick<PolityLine, "label" | "labelModifiers" | "labelConfidence">,
): string {
  if (p.label === "") return "";
  const qualified = p.labelModifiers.length > 0 ? `: ${p.labelModifiers.join("; ")}` : "";
  return `${p.label}${qualified} (confidence ${p.labelConfidence.toFixed(2)})`;
}
