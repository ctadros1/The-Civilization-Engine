// Wealth (M3a slice K; ADR-0007 §4): what the wealth panel says about a settlement's measures.
// Pure functions. The kernel measures each household (land held and worked, goods valued in hours
// of work, floor area) and how the measures spread over people; the observer only shows them.

import { formatPercent } from "./format.js";
import type { HouseholdWealth, WealthSpread } from "./net/messages.js";

/** Households the panel lists per settlement, the most goods per head first. */
export const HOUSEHOLDS_SHOWN = 20;
/** Years of history the panel's table lists, newest first. */
export const YEARS_SHOWN = 10;

/** A Gini coefficient: "0.42". */
export function giniText(g: number): string {
  return g.toFixed(2);
}

/** An area: "1.25 ha", or "none". */
export function areaText(ha: number): string {
  return ha < 0.005 ? "none" : `${ha.toFixed(2)} ha`;
}

/** Hours of work: "58 h", "0.4 h", or "none". */
export function hoursText(h: number): string {
  if (h < 0.05) return "none";
  return h < 10 ? `${h.toFixed(1)} h` : `${Math.round(h)} h`;
}

/** A floor area: "13 m²", or "no roof". */
export function floorText(m2: number): string {
  return m2 < 0.5 ? "no roof" : `${Math.round(m2)} m²`;
}

/** A row of a settlement's measures: how much there is and how unequally it is spread. */
export interface MeasureRow {
  measure: string;
  /** "58 h a head", "0.31 ha a head", "13 m² a house". */
  level: string;
  /** "0.42". */
  gini: string;
}

/** A settlement's measures as they stand, as table rows; `households` are its households'. */
export function measureRows(s: WealthSpread, households: HouseholdWealth[]): MeasureRow[] {
  const held = households.reduce((sum, h) => sum + h.heldHa, 0);
  return [
    { measure: "Goods", level: `${hoursText(s.goodsHPerHead)} a head`, gini: giniText(s.giniGoods) },
    {
      measure: "Land worked",
      level: `${areaText(s.workedHaPerHead)} a head`,
      gini: giniText(s.giniWorked),
    },
    {
      measure: "Land held",
      level: held < 0.005 ? "none" : `${areaText(s.people > 0 ? held / s.people : 0)} a head`,
      gini: giniText(s.giniHeld),
    },
    {
      measure: "Floor area",
      level: s.floorM2PerHouse > 0 ? `${floorText(s.floorM2PerHouse)} a house` : "no roofs yet",
      gini: giniText(s.giniFloor),
    },
    {
      measure: "Under all roofs",
      level: s.roofedM2PerHouse > 0 ? `${floorText(s.roofedM2PerHouse)} a household` : "no roofs yet",
      gini: "",
    },
    {
      measure: "Room for goods",
      level: s.storageKgPerHouse > 0 ? `${tonnesText(s.storageKgPerHouse)} a household` : "none yet",
      gini: "",
    },
  ];
}

/** A weight: "3.2 t", "450 kg". */
export function tonnesText(kg: number): string {
  return kg >= 1000 ? `${(kg / 1000).toFixed(1)} t` : `${Math.round(kg)} kg`;
}

/**
 * The spread in words: "The richest tenth of people have 21% of the goods. 2 of 6 households
 * hold no land; every household works some."
 */
export function spreadText(s: WealthSpread): string {
  if (s.households === 0) return "Nobody lives here now.";
  const holdingNone = Math.round(s.holdingNone * s.households);
  const workingNone = Math.round(s.workingNone * s.households);
  const held =
    holdingNone === 0
      ? "Every household holds land"
      : holdingNone === s.households
        ? s.commonHa > 0
          ? `No household holds land: the settlement holds ${areaText(s.commonHa)}`
          : "No household holds land"
        : `${holdingNone} of ${s.households} households hold no land`;
  const worked =
    workingNone === 0
      ? "every household works some"
      : workingNone === s.households
        ? "none works any"
        : `${workingNone} work${workingNone === 1 ? "s" : ""} none`;
  return `The richest tenth of people have ${formatPercent(s.topTenthGoods)} of the goods. ${held}; ${worked}.`;
}

/** A row of the households table. */
export interface HouseholdRow {
  household: number;
  name: string;
  people: string;
  holds: string;
  /** What it works, with its leases: "0.80 ha (rents 0.40 ha)". */
  works: string;
  /** Goods a head: "58 h". */
  goods: string;
  /** Its home's floor, and the floor under all its roofs when that is more: "30 m², 45 m² in
   * all". */
  floor: string;
}

/** What a household works, with its leases in words. */
export function worksText(h: HouseholdWealth): string {
  const notes: string[] = [];
  if (h.rentedHa >= 0.005) notes.push(`rents ${areaText(h.rentedHa)}`);
  if (h.letHa >= 0.005) notes.push(`lets ${areaText(h.letHa)}`);
  const worked = areaText(h.workedHa);
  return notes.length > 0 ? `${worked} (${notes.join(", ")})` : worked;
}

/**
 * The first `shown` households as table rows, in the order given (the kernel's: the most goods
 * per head first).
 */
export function householdRows(
  households: HouseholdWealth[],
  shown = HOUSEHOLDS_SHOWN,
): HouseholdRow[] {
  return households.slice(0, shown).map((h) => ({
    household: h.household,
    name: h.name,
    people: String(h.members),
    holds: areaText(h.heldHa),
    works: worksText(h),
    goods: hoursText(h.members > 0 ? h.goodsH / h.members : 0),
    floor: floorText(h.floorM2) + (h.roofedM2 >= h.floorM2 + 0.5 ? `, ${floorText(h.roofedM2)} in all` : ""),
  }));
}

/** A row of the yearly history table. */
export interface YearRow {
  year: number;
  goods: string;
  worked: string;
  floor: string;
  topTenth: string;
  holdingNone: string;
}

/** The latest `shown` years, newest first. */
export function yearRows(history: WealthSpread[], shown = YEARS_SHOWN): YearRow[] {
  return history
    .slice(-shown)
    .reverse()
    .map((y) => ({
      year: y.year,
      goods: giniText(y.giniGoods),
      worked: giniText(y.giniWorked),
      floor: giniText(y.giniFloor),
      topTenth: formatPercent(y.topTenthGoods),
      holdingNone: formatPercent(y.holdingNone),
    }));
}

/**
 * Each year's Ginis as SVG polyline points in a `width` × `height` box, on a fixed 0–1 scale so
 * that years and settlements compare: goods, land worked and floor area. Empty strings with
 * fewer than two years.
 */
export function giniLines(
  history: WealthSpread[],
  width: number,
  height: number,
): { goods: string; worked: string; floor: string } {
  if (history.length < 2) return { goods: "", worked: "", floor: "" };
  const step = width / (history.length - 1);
  const line = (pick: (y: WealthSpread) => number) =>
    history
      .map((y, i) => {
        const g = Math.min(1, Math.max(0, pick(y)));
        return `${(i * step).toFixed(1)},${(height - g * height).toFixed(1)}`;
      })
      .join(" ");
  return {
    goods: line((y) => y.giniGoods),
    worked: line((y) => y.giniWorked),
    floor: line((y) => y.giniFloor),
  };
}
