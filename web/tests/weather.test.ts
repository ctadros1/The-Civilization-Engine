import { describe, expect, it } from "vitest";

import type { WeatherMonth, WeatherReport } from "../src/net/messages.js";
import { monthRows, monthText, rainText, rainWord, tempText, yearRows, years } from "../src/weather.js";

function month(over: Partial<WeatherMonth> = {}): WeatherMonth {
  return {
    year: 2,
    month: 4,
    days: 31,
    precipMm: 80,
    usualMm: 80,
    wetDays: 13,
    meanC: 13.5,
    usualC: 13.5,
    minC: 2,
    maxC: 25,
    frostDays: 0,
    snowDays: 0,
    soil: 0.8,
    ...over,
  };
}

function report(months: WeatherMonth[]): WeatherReport {
  return { rev: 7, heightM: 150, annualMm: 800, today: null, months, month: 0 };
}

describe("the weather panel's words", () => {
  it("names months and compares rain and warmth with the usual", () => {
    expect(monthText({ year: 3, month: 4 })).toBe("May 3");
    expect(rainText(91.6, 80.4)).toBe("92 (80)");
    expect(tempText(13.9, 13.5)).toBe("13.9 (+0.4)");
    expect(tempText(12.0, 13.5)).toBe("12.0 (−1.5)");
    expect(tempText(13.5, 13.5)).toBe("13.5 (±0.0)");
    expect(rainWord(30, 80)).toBe("very dry");
    expect(rainWord(60, 80)).toBe("dry");
    expect(rainWord(80, 80)).toBe("");
    expect(rainWord(110, 80)).toBe("wet");
    expect(rainWord(200, 80)).toBe("very wet");
  });

  it("lists the newest months first, the month under way against its usual so far", () => {
    const rows = monthRows(
      report([month({ month: 3, days: 30, precipMm: 30 }), month({ month: 4, days: 10, precipMm: 10, usualMm: 93 })]),
    );
    expect(rows.map((r) => r.month)).toEqual(["May 2", "Apr 2"]);
    // Ten days of May usually bring 30 mm of its 93.
    expect(rows[0]).toMatchObject({ rain: "10 (30)", rainWord: "", partial: true });
    expect(rows[1]).toMatchObject({ rainWord: "very dry", partial: false, soil: "80%" });
  });

  it("sums calendar years from their months, a year not lived whole saying how much was", () => {
    const all: WeatherMonth[] = [];
    for (let m = 2; m < 12; m++) all.push(month({ year: 0, month: m, days: [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][m] }));
    for (let m = 0; m < 12; m++) all.push(month({ year: 1, month: m, days: [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][m], precipMm: 40, usualMm: 80 }));
    const ys = years(all);
    expect(ys.map((y) => y.year)).toEqual([0, 1]);
    expect(ys[1]).toMatchObject({ months: 12, days: 365, precipMm: 480, usualMm: 960 });
    const rows = yearRows(report(all));
    // Half the usual is dry; under half, very dry.
    expect(rows[0]).toMatchObject({ year: "1", rainWord: "dry", partial: false });
    expect(rows[1]).toMatchObject({ year: "0 (306 days)", rainWord: "", partial: true });
  });
});
