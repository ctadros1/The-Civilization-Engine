// Takings in words (wire 1.30, ADR-0015): what happened beside what people believe of it. The
// kernel renders what was carried off, the household's name and what was chosen; the observer
// only joins the pieces, and never mixes the two layers in one sentence.

import { lawDayText } from "./government.js";
import type { IncidentLine, KnownLine, OrderInfo } from "./net/messages.js";

/** What happened after the taker's name: "took 12 kg of grain from the household of Rilla on 4 May of year 3." */
export function happenedRest(i: IncidentLine): string {
  const when = lawDayText(i.minute);
  switch (i.outcome) {
    case "turned back":
      return `went to take from ${i.targetName} on ${when} and turned back: someone was home.`;
    case "fled":
      return `went to take from ${i.targetName} on ${when} and fled with nothing when a sleeper woke.`;
    default:
      return `took ${i.what} from ${i.targetName} on ${when}.`;
  }
}

/** "Tam took 12 kg of grain from the household of Rilla on 4 May of year 3." */
export function happenedText(i: IncidentLine): string {
  return `${i.actorName} ${happenedRest(i)}`;
}

/** "Seen by Bram and Ada", or "Nobody saw it". */
export function seenText(i: IncidentLine): string {
  const n = i.seenBy.length;
  if (n === 0) return "Nobody saw it";
  if (n === 1) return `Seen by ${i.seenBy[0]}`;
  if (n === 2) return `Seen by ${i.seenBy[0]} and ${i.seenBy[1]}`;
  if (n === 3) return `Seen by ${i.seenBy[0]}, ${i.seenBy[1]} and 1 other`;
  return `Seen by ${i.seenBy[0]}, ${i.seenBy[1]} and ${n - 2} others`;
}

/**
 * What the living believe of it: "5 believe they know who took, from 1 account; 3 know only of a
 * loss", or "Nobody knows of it".
 */
export function believedText(k: KnownLine | undefined): string {
  if (!k || (k.knowTaker === 0 && k.knowLoss === 0)) return "Nobody knows of it";
  const parts: string[] = [];
  if (k.knowTaker > 0) {
    const accounts = k.sources === 1 ? "1 account" : `${k.sources} accounts`;
    parts.push(
      `${k.knowTaker} ${k.knowTaker === 1 ? "believes they know" : "believe they know"} who took, from ${accounts}`,
    );
  }
  if (k.knowLoss > 0) {
    parts.push(`${k.knowLoss} ${k.knowLoss === 1 ? "knows" : "know"} only of a loss`);
  }
  return parts.join("; ");
}

/** What the household taken from chose and where it stands: "Rilla demanded the food back: paid". */
export function choiceText(k: KnownLine | undefined): string {
  if (!k || k.response === "") return "";
  return k.owed === "" ? k.response : `${k.response}: ${k.owed}`;
}

/** The totals in a sentence. */
export function totalsText(o: OrderInfo): string {
  if (o.attempts === 0) return "Nobody has gone to take from another household's store.";
  const takings = o.takings === 1 ? "1 taking" : `${o.takings} takings`;
  const attempts = o.attempts === 1 ? "1 attempt" : `${o.attempts} attempts`;
  const seen = `${o.seen} seen`;
  const known = `${o.knownToVictims} known to the household taken from`;
  const demands = o.demands === 1 ? "1 demand" : `${o.demands} demands`;
  return `${attempts}, ${takings}; ${seen}; ${known}; ${demands} to give back (${o.met} met, ${o.refused} refused); ${o.refusals} asks refused to those believed to have taken.`;
}
