// Grievances and word of mouth in words (wire 1.34, ADR-0016 §2-3). The kernel renders what a
// grievance is over, whom it blames, what raised it and what a claim says; the observer only
// joins the pieces.

import { lawDayText } from "./government.js";
import type { GrievanceLine, HeardLine, NormLine, PositionLine, ValueLine } from "./net/messages.js";

/** One number of days as the panels show it: "4", "1.5". */
function days(x: number): string {
  const r = Math.round(x * 10) / 10;
  return Number.isInteger(r) ? String(r) : r.toFixed(1);
}

/** How keenly a grievance is felt now: "keenly felt", "felt", "faintly felt". */
export function keennessText(activation: number): string {
  return activation >= 0.6 ? "keenly felt" : activation >= 0.2 ? "felt" : "faintly felt";
}

/** The grievance itself: "against the gathering, over food when their household was short". */
export function grievanceHead(g: Pick<GrievanceLine, "blamed" | "over">): string {
  return `against ${g.blamed}, over ${g.over}`;
}

/** Why and how much: "the common store was empty when their household was short; 4 of 5 days
 * of food not made good; held since 3 May of year 2, last raised on 9 May of year 2; felt". */
export function grievanceFacts(g: GrievanceLine): string {
  const parts = [g.reason];
  parts.push(
    g.unresolvedDays > 0
      ? `${days(g.unresolvedDays)} of ${days(g.harmDays)} days of food not made good`
      : `${days(g.harmDays)} days of food, all made good`,
  );
  const since = lawDayText(g.madeMinute);
  parts.push(
    g.raisedMinute > g.madeMinute
      ? `held since ${since}, last raised on ${lawDayText(g.raisedMinute)}`
      : `held since ${since}`,
  );
  parts.push(keennessText(g.activation));
  return parts.join("; ");
}

/** When they heard a claim: "on 3 May of year 2", and ", last heard on 5 May of year 2" when
 * heard again since. */
export function heardWhen(h: Pick<HeardLine, "firstMinute" | "lastMinute">): string {
  const first = `on ${lawDayText(h.firstMinute)}`;
  return h.lastMinute > h.firstMinute ? `${first}, last heard on ${lawDayText(h.lastMinute)}` : first;
}

/** How they heard a claim: "told by Wren on 3 May of year 2", "they knew it first, on 3 May of
 * year 2". */
export function heardHow(h: HeardLine): string {
  return h.from === 0 ? `they knew it first, ${heardWhen(h)}` : `told by ${h.fromName} ${heardWhen(h)}`;
}

/** The claim itself, as a sentence: "A gathering meets on 9 May of year 2 to hear a case." */
export function claimText(h: Pick<HeardLine, "what">): string {
  const w = h.what.trim();
  if (!w) return "";
  return `${w.charAt(0).toUpperCase()}${w.slice(1)}.`;
}

/** Where someone stands, with what moved them: "leaning for (0.66); their household's lot alone
 * would make it 0.58; talk at the hearth, heard 4 times, drew them toward it". */
export function positionText(p: PositionLine): string {
  const at = (x: number) => (Math.round(x * 100) / 100).toFixed(2);
  const parts = [`${p.lean} (${at(p.x)})`];
  const moved = p.x - p.anchor;
  if (Math.abs(moved) < 0.01) {
    parts.push("as their household's lot and what they hold dear make it");
  } else {
    parts.push(`their household's lot and what they hold dear would make it ${at(p.anchor)}`);
    const times = p.heard === 1 ? "once" : `${p.heard} times`;
    parts.push(
      `talk at the hearth, taken in ${times}, ${moved > 0 ? "drew them toward it" : "turned them against it"}`,
    );
  }
  return parts.join("; ");
}

/** What someone holds of a norm, after its statement: "holds firmly (0.81); believes most
 * households pay what the gathering asks (0.72), from 3 accounts at the hearth; enough do to hold
 * them to it". */
export function normText(n: NormLine): string {
  const at = (x: number) => (Math.round(x * 100) / 100).toFixed(2);
  const from =
    n.heard === 0 ? "from what their own people believed" : n.heard === 1 ? "from one account at the hearth" : `from ${n.heard} accounts at the hearth`;
  const moved = n.activation >= 0.5 ? "enough do to hold them to it" : "too few do to hold them to it";
  return `${n.holds} (${at(n.endorse)}); believes ${n.believes} (${at(n.expect)}), ${from}; ${moved}`;
}

/** What someone holds dear, in one line: "holds safety from want and harm dear (0.62); cares as
 * most do for a household's say over what is its own (0.10)". */
export function valuesText(vs: ValueLine[]): string {
  const at = (x: number) => (Math.round(x * 100) / 100).toFixed(2);
  return vs.map((v) => `${v.words} (${at(v.v)})`).join("; ");
}
