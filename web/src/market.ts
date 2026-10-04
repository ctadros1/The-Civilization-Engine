// Markets (M3a slice I): what the market panel says about terms, payments and prices. Pure
// functions. The kernel decides who offers what on which terms and what sold; the observer only
// shows it.

import { formatKg } from "./format.js";
import type { GoodInfo, MarketInfo, MonthOfTrade, OfferInfo } from "./net/messages.js";

/** Kilograms a price is quoted for, for goods counted in kilograms. */
export const KG_LOT = 10;
/** Months the price history shows. */
export const HISTORY_SHOWN = 12;

const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

function isTool(good: GoodInfo | undefined): boolean {
  return good?.purpose === "tool";
}

/** An amount of a good: "1 sickle", "0.8 hoes", "12 kg grain". */
export function amountText(good: GoodInfo | undefined, units: number): string {
  const name = good ? good.name.toLowerCase() : "a good no longer known";
  if (isTool(good)) {
    if (Math.abs(units - 1) < 0.05) return `1 ${name}`;
    return `${units.toFixed(1)} ${name}s`;
  }
  return `${formatKg(units)} ${name}`;
}

/**
 * A price, so much of `payment` for a unit of `good`, as people would quote it: by the tool, or
 * by ten kilograms: "12 kg grain each", "0.3 hoes per 10 kg".
 */
export function priceText(
  goods: GoodInfo[],
  good: number,
  payment: number,
  price: number,
): string {
  if (isTool(goods[good])) return `${amountText(goods[payment], price)} each`;
  return `${amountText(goods[payment], price * KG_LOT)} per ${KG_LOT} kg`;
}

/** The lowest terms asked for `good` in each payment good, cheapest payment index first. */
export function askingText(goods: GoodInfo[], offers: OfferInfo[], good: number): string {
  const best = new Map<number, number>();
  for (const o of offers) {
    if (o.good !== good) continue;
    const seen = best.get(o.payment);
    if (seen === undefined || o.price < seen) best.set(o.payment, o.price);
  }
  return [...best.entries()]
    .sort((a, b) => a[0] - b[0])
    .map(([payment, price]) => priceText(goods, good, payment, price))
    .join(" or ");
}

/** What payments settle in, largest share first: "grain 61% · hoes 25%". */
export function paymentsText(goods: GoodInfo[], market: MarketInfo): string {
  return market.goods
    .filter((g) => g.acceptance >= 0.005)
    .sort((a, b) => b.acceptance - a.acceptance || a.good - b.good)
    .map((g) => `${(goods[g.good]?.name ?? "?").toLowerCase()} ${Math.round(g.acceptance * 100)}%`)
    .join(" · ");
}

/** One household's terms for one good, as the offers list shows them. */
export interface OfferGroup {
  household: number;
  householdName: string;
  good: number;
  units: number;
  /** "12 kg grain each or 0.8 hoes each". */
  terms: string;
}

/** The offers grouped by household and good, by good and then household. */
export function offerGroups(goods: GoodInfo[], offers: OfferInfo[]): OfferGroup[] {
  const groups = new Map<string, OfferGroup & { rows: OfferInfo[] }>();
  for (const o of offers) {
    const key = `${o.good}:${o.household}`;
    let g = groups.get(key);
    if (!g) {
      g = {
        household: o.household,
        householdName: o.householdName,
        good: o.good,
        units: o.units,
        terms: "",
        rows: [],
      };
      groups.set(key, g);
    }
    g.units = Math.max(g.units, o.units);
    g.rows.push(o);
  }
  return [...groups.values()]
    .sort((a, b) => a.good - b.good || a.household - b.household)
    .map(({ rows, ...g }) => ({
      ...g,
      terms: rows
        .slice()
        .sort((a, b) => a.payment - b.payment)
        .map((o) => priceText(goods, o.good, o.payment, o.price))
        .join(" or "),
    }));
}

/** A month of the price history: what a unit was worth to its sellers, hours (null = no trade). */
export interface PricePoint {
  month: number;
  hoursPerUnit: number | null;
}

/** Months since the calendar's origin, as the kernel counts them. */
export function monthIndex(year: number, month: number): number {
  return (year - 1) * 12 + (month - 1);
}

/** "Mar, year 2" for a month index. */
export function monthText(month: number): string {
  return `${MONTHS[month % 12] ?? "?"}, year ${Math.floor(month / 12) + 1}`;
}

/** The last `shown` months of `good`'s price history up to month `last`, oldest first. */
export function priceSeries(
  history: MonthOfTrade[],
  good: number,
  last: number,
  shown = HISTORY_SHOWN,
): PricePoint[] {
  const out: PricePoint[] = [];
  for (let m = last - shown + 1; m <= last; m++) {
    if (m < 0) continue;
    const h = history.find((x) => x.month === m && x.good === good);
    out.push({
      month: m,
      hoursPerUnit: h && h.units > 0 ? h.paidH / h.units : null,
    });
  }
  return out;
}

/**
 * The polylines of a price series in a `width` × `height` box, months left to right, the dearest
 * month near the top: one run of points per stretch of months with trades.
 */
export function sparkline(points: PricePoint[], width: number, height: number): string[] {
  const values = points.map((p) => p.hoursPerUnit).filter((v): v is number => v !== null);
  if (values.length === 0) return [];
  const top = Math.max(...values) * 1.1;
  const step = points.length > 1 ? width / (points.length - 1) : 0;
  const runs: string[] = [];
  let run: string[] = [];
  points.forEach((p, i) => {
    if (p.hoursPerUnit === null) {
      if (run.length > 0) runs.push(run.join(" "));
      run = [];
      return;
    }
    const x = points.length > 1 ? i * step : width / 2;
    const y = height - (p.hoursPerUnit / top) * height;
    run.push(`${x.toFixed(1)},${y.toFixed(1)}`);
  });
  if (run.length > 0) runs.push(run.join(" "));
  return runs;
}

/** What a unit was worth to its sellers in the latest month it sold: "6.1 h of work each". */
export function worthText(goods: GoodInfo[], good: number, points: PricePoint[]): string {
  const latest = [...points].reverse().find((p) => p.hoursPerUnit !== null);
  if (!latest || latest.hoursPerUnit === null) return "";
  if (isTool(goods[good])) return `${latest.hoursPerUnit.toFixed(1)} h of work each`;
  return `${(latest.hoursPerUnit * KG_LOT).toFixed(1)} h of work per ${KG_LOT} kg`;
}
