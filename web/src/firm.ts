// Workshops (M3a slice J): what the workshops panel says about a workshop's books. Pure
// functions. The kernel decides who sets up a workshop, what it makes, sells and pays, and puts
// its record, wage and book lines in words; the observer only shows them.

import { formatKg } from "./format.js";
import { amountText, monthText } from "./market.js";
import type { BookKind, GoodInfo, MonthStatement, StoreLine } from "./net/messages.js";

/** Months of statements a workshop's page shows, newest first. */
export const MONTHS_SHOWN = 12;
/** Lines of its books a workshop's page lists, newest first. */
export const ENTRIES_SHOWN = 16;

/** What a workshop makes, in words: "sickles", "hoes and sickles". */
export function linesText(goods: GoodInfo[], lines: number[]): string {
  const names = lines.map((g) => {
    const good = goods[g];
    if (!good) return "goods no longer known";
    const name = good.name.toLowerCase();
    return good.purpose === "tool" ? `${name}s` : name;
  });
  if (names.length <= 1) return names[0] ?? "nothing";
  return `${names.slice(0, -1).join(", ")} and ${names.at(-1)}`;
}

/** How much of `good` a month's books moved by `kind`. */
export function monthAmount(m: MonthStatement, kind: BookKind, good: number): number {
  return m.lines
    .filter((l) => l.kind === kind && l.good === good)
    .reduce((sum, l) => sum + l.amount, 0);
}

/** Hours as a statement's cells show them: "3.5", or "—" for none. */
export function hoursText(h: number): string {
  if (Math.abs(h) < 0.05) return "—";
  return h.toFixed(1);
}

/** What came in less what went out, hours: "+2.1", "−0.6", or "—" for none. */
export function marginText(h: number): string {
  if (Math.abs(h) < 0.05) return "—";
  return `${h > 0 ? "+" : "−"}${Math.abs(h).toFixed(1)}`;
}

/** A count or weight without its noun, as a statement's cells show it: "3.2", "1", "12 kg". */
export function quantityText(good: GoodInfo | undefined, units: number): string {
  if (good?.purpose === "tool") {
    const whole = Math.round(units);
    return Math.abs(units - whole) < 0.05 ? String(whole) : units.toFixed(1);
  }
  return formatKg(units);
}

/** A month's row of a workshop's statements. */
export interface StatementRow {
  month: number;
  /** "Mar, year 2". */
  label: string;
  /**
   * What it made and sold of what it makes to sell: "2", "1.5", or "—"; with the goods named
   * when it makes more than one: "2 sickles, 1 hoe".
   */
  made: string;
  sold: string;
  /** Hours worked for it, and of them hired: "3.0", "4.5 (1.5)". */
  worked: string;
  /** Worth in hours of its owners' own work. */
  income: string;
  costs: string;
  margin: string;
  stock: string;
}

/** The latest `shown` months of a workshop's statements, newest first. */
export function statementRows(
  goods: GoodInfo[],
  lines: number[],
  months: MonthStatement[],
  shown = MONTHS_SHOWN,
): StatementRow[] {
  return months
    .slice(-shown)
    .reverse()
    .map((m) => {
      const of = (kind: BookKind) => {
        const parts = lines
          .map((g) => [g, monthAmount(m, kind, g)] as const)
          .filter(([, amount]) => amount >= 0.05)
          .map(([g, amount]) =>
            lines.length > 1 ? amountText(goods[g], amount) : quantityText(goods[g], amount),
          );
        return parts.length > 0 ? parts.join(", ") : "—";
      };
      const worked = m.ownerH + m.hiredH;
      return {
        month: m.month,
        label: monthText(m.month),
        made: of("made"),
        sold: of("sold"),
        worked: m.hiredH >= 0.05 ? `${hoursText(worked)} (${hoursText(m.hiredH)})` : hoursText(worked),
        income: hoursText(m.incomeH),
        costs: hoursText(m.costsH),
        margin: marginText(m.incomeH - m.costsH),
        stock: hoursText(m.stockH),
      };
    });
}

/** What a workshop holds: "2 sickles · 14 kg grain", or "nothing". */
export function holdingsText(goods: GoodInfo[], stores: StoreLine[]): string {
  const parts = stores
    .filter((s) => s.kg >= 0.005)
    .map((s) => amountText(goods[s.good], s.kg));
  return parts.length > 0 ? parts.join(" · ") : "nothing";
}

/**
 * A workshop's life in hours: "Its owners worked 8.0 h for it and hired 6.0 h. What it sold
 * brought in 23.5 h of their own work; its inputs and wages cost 12.1 h."
 */
export function lifeText(months: MonthStatement[]): string {
  const sum = (f: (m: MonthStatement) => number) => months.reduce((s, m) => s + f(m), 0);
  const owner = sum((m) => m.ownerH);
  const hired = sum((m) => m.hiredH);
  const income = sum((m) => m.incomeH);
  const costs = sum((m) => m.costsH);
  const work =
    hired >= 0.05
      ? `Its owners worked ${owner.toFixed(1)} h for it and hired ${hired.toFixed(1)} h.`
      : `Its owners worked ${owner.toFixed(1)} h for it and hired no one.`;
  const books =
    income >= 0.05
      ? `What it sold brought in ${income.toFixed(1)} h of their own work; its inputs and wages cost ${costs.toFixed(1)} h.`
      : `It has been paid nothing yet; its inputs and wages cost ${costs.toFixed(1)} h of their own work.`;
  return `${work} ${books}`;
}
