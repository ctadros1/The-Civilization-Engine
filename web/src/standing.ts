// Ties and standing in words (wire 1.26, ADR-0014). The kernel works out who regards whom and
// renders each tie's reason; the observer only says it plainly.

import type { StandingLine, TieLine } from "./net/messages.js";

/** "warm, well known", "known by sight": how well and how warmly someone is known. */
export function closenessText(t: Pick<TieLine, "familiarity" | "warmth">): string {
  const known =
    t.familiarity >= 0.6 ? "well known" : t.familiarity >= 0.25 ? "known" : "known by sight";
  const warmth = t.warmth >= 0.5 ? "close" : t.warmth >= 0.15 ? "warm" : "";
  return warmth ? `${warmth}, ${known}` : known;
}

/** One number as the panels show it: "4", "1.5", "0.3". */
function amount(x: number): string {
  const r = Math.round(x * 10) / 10;
  return Number.isInteger(r) ? String(r) : r.toFixed(1);
}

/** "provision 4, word 1.5" for the domains with esteem of 0.05 or more either way; "" for none. */
export function esteemText(esteem: number[], domains: string[]): string {
  return esteem
    .map((e, k) => ({ e, name: domains[k] ?? `domain ${k}` }))
    .filter(({ e }) => Math.abs(e) >= 0.05)
    .map(({ e, name }) => `${name} ${amount(e)}`)
    .join(", ");
}

/** One tie in a line: "close, well known; esteemed for provision 2; owes them 3 h of help". */
export function tieText(t: TieLine, domains: string[]): string {
  const parts = [closenessText(t)];
  const esteem = esteemText(t.esteem, domains);
  if (esteem) parts.push(`esteemed for ${esteem}`);
  if (t.helpH >= 0.5) parts.push(`owes them ${amount(t.helpH)} h of help`);
  else if (t.helpH <= -0.5) parts.push(`is owed ${amount(-t.helpH)} h of help`);
  if (!t.mutual) parts.push("not known back");
  return parts.join("; ");
}

/** "a notable; 3 count them among those they esteem most; esteemed for provision 4". */
export function standingText(s: StandingLine, domains: string[]): string {
  const parts: string[] = [];
  if (s.notable) parts.push("a notable");
  parts.push(
    s.influence === 0
      ? "nobody counts them among those they esteem most"
      : s.influence === 1
        ? "one counts them among those they esteem most"
        : `${s.influence} count them among those they esteem most`,
  );
  const esteem = esteemText(s.esteem, domains);
  parts.push(esteem ? `esteemed for ${esteem}` : "no standing yet");
  return parts.join("; ");
}
