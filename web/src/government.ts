// A settlement's polity and its laws in words (wire 1.27, ADR-0013). The kernel renders what a
// law is, how it was decided and why each member stood where they did; the observer only joins
// the pieces.

import { formatKg, simDate } from "./format.js";
import type { LawLine, StanceLine } from "./net/messages.js";

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

/** "in force", "turned down", "before the gathering on 9 May of year 3". */
export function statusText(l: Pick<LawLine, "status" | "outcome" | "meetsMinute">): string {
  if (l.status === "in force") return "in force";
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

/** "the store answered 3 asks (45 kg); 1 it could not". "" before anyone asked. */
export function reliefText(l: Pick<LawLine, "relieved" | "reliefKg" | "unanswered">): string {
  if (l.relieved + l.unanswered === 0) return "";
  const asks = `${l.relieved} ask${l.relieved === 1 ? "" : "s"}`;
  const text = `the store answered ${asks} (${formatKg(l.reliefKg)})`;
  return l.unanswered > 0 ? `${text}; ${l.unanswered} it could not` : text;
}
