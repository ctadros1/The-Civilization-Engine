// Weather (M3c slice U; ADR-0012): what the weather panel says about the months and years lived.
// Pure functions. The kernel draws the weather and keeps each month's record on the valley floor;
// the observer only shows it, beside what each month usually brings.

import type { WeatherMonth, WeatherReport } from "./net/messages.js";

const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
const DAYS_IN = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

/** Months the panel's table lists, newest first. */
export const MONTHS_SHOWN = 12;
/** Years the panel's table lists, newest first. */
export const YEARS_SHOWN = 20;

function daysIn(month: number): number {
  return DAYS_IN[month] ?? 30;
}

/** A month: "May 3", the month of the world's year 3. */
export function monthText(m: { year: number; month: number }): string {
  return `${MONTHS[m.month] ?? "?"} ${m.year}`;
}

/** Rain in mm against what is usual: "92 (81)". */
export function rainText(mm: number, usualMm: number): string {
  return `${Math.round(mm)} (${Math.round(usualMm)})`;
}

/** A temperature in °C against its normal: "13.9 (+0.4)". */
export function tempText(c: number, usualC: number): string {
  const d = c - usualC;
  const sign = d >= 0.05 ? "+" : d <= -0.05 ? "−" : "±";
  return `${c.toFixed(1)} (${sign}${Math.abs(d).toFixed(1)})`;
}

/** How rain compares with what is usual, in a word or two: "very dry" to "very wet", or "". */
export function rainWord(mm: number, usualMm: number): string {
  if (usualMm <= 0) return "";
  const r = mm / usualMm;
  if (r < 0.5) return "very dry";
  if (r < 0.8) return "dry";
  if (r <= 1.25) return "";
  if (r <= 1.6) return "wet";
  return "very wet";
}

/** One row of the months' table. */
export interface MonthRow {
  month: string;
  rain: string;
  /** Against the month's usual: "very dry", "dry", "", "wet", "very wet". */
  rainWord: string;
  temp: string;
  frost: number;
  snow: number;
  /** The soil water under the wild cover, on average: "62%". */
  soil: string;
  /** Still under way: its figures are so far, against the usual so far. */
  partial: boolean;
}

/** The newest months as table rows, newest first. */
export function monthRows(report: WeatherReport, shown = MONTHS_SHOWN): MonthRow[] {
  return report.months
    .slice(-shown)
    .reverse()
    .map((m) => {
      const partial = m.days < daysIn(m.month);
      const usual = (m.usualMm * m.days) / daysIn(m.month);
      return {
        month: monthText(m),
        rain: rainText(m.precipMm, usual),
        rainWord: partial ? "" : rainWord(m.precipMm, m.usualMm),
        temp: tempText(m.meanC, m.usualC),
        frost: m.frostDays,
        snow: m.snowDays,
        soil: `${Math.round(m.soil * 100)}%`,
        partial,
      };
    });
}

/** A calendar year's weather, summed from its months. */
export interface YearSummary {
  year: number;
  /** Months recorded: fewer than 12 for the year under way, or a world's first. */
  months: number;
  /** Days recorded. */
  days: number;
  precipMm: number;
  /** What its days usually bring, mm. */
  usualMm: number;
  wetDays: number;
  /** Mean temperature over its days, and the normal over the same days, °C. */
  meanC: number;
  usualC: number;
  frostDays: number;
  snowDays: number;
}

/** Each calendar year from its months, oldest first. */
export function years(months: WeatherMonth[]): YearSummary[] {
  const byYear = new Map<number, YearSummary>();
  for (const m of months) {
    let y = byYear.get(m.year);
    if (!y) {
      y = {
        year: m.year,
        months: 0,
        days: 0,
        precipMm: 0,
        usualMm: 0,
        wetDays: 0,
        meanC: 0,
        usualC: 0,
        frostDays: 0,
        snowDays: 0,
      };
      byYear.set(m.year, y);
    }
    y.months += 1;
    y.days += m.days;
    y.precipMm += m.precipMm;
    y.usualMm += (m.usualMm * m.days) / daysIn(m.month);
    y.wetDays += m.wetDays;
    // Day-weighted, then divided below.
    y.meanC += m.meanC * m.days;
    y.usualC += m.usualC * m.days;
    y.frostDays += m.frostDays;
    y.snowDays += m.snowDays;
  }
  const out = [...byYear.values()].sort((a, b) => a.year - b.year);
  for (const y of out) {
    const d = Math.max(1, y.days);
    y.meanC /= d;
    y.usualC /= d;
  }
  return out;
}

/** One row of the years' table. */
export interface YearRow {
  year: string;
  rain: string;
  rainWord: string;
  temp: string;
  wet: number;
  frost: number;
  snow: number;
  partial: boolean;
}

/** The newest years as table rows, newest first. A year not lived whole says how much was. */
export function yearRows(report: WeatherReport, shown = YEARS_SHOWN): YearRow[] {
  return years(report.months)
    .slice(-shown)
    .reverse()
    .map((y) => {
      const partial = y.days < 365;
      return {
        year: partial ? `${y.year} (${y.days} days)` : `${y.year}`,
        rain: rainText(y.precipMm, y.usualMm),
        rainWord: partial ? "" : rainWord(y.precipMm, y.usualMm),
        temp: tempText(y.meanC, y.usualC),
        wet: y.wetDays,
        frost: y.frostDays,
        snow: y.snowDays,
        partial,
      };
    });
}
