// Plain-words formatting for the panels. The calendar mirrors civ-core's: 365-day years numbered
// from 1, no leap years, one tick per minute.

const MINUTES_PER_DAY = 24 * 60;
const MINUTES_PER_YEAR = 365 * MINUTES_PER_DAY;
const MONTH_STARTS = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365];
const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

export interface SimDate {
  year: number;
  month: number;
  day: number;
  hour: number;
  minute: number;
}

/** The calendar date of a simulation minute. */
export function simDate(minutes: number): SimDate {
  const year = Math.floor(minutes / MINUTES_PER_YEAR) + 1;
  const inYear = minutes - (year - 1) * MINUTES_PER_YEAR;
  const dayOfYear = Math.floor(inYear / MINUTES_PER_DAY);
  let month = 0;
  while (month < 11 && dayOfYear >= MONTH_STARTS[month + 1]!) month++;
  const minuteOfDay = inYear - dayOfYear * MINUTES_PER_DAY;
  return {
    year,
    month: month + 1,
    day: dayOfYear - MONTH_STARTS[month]! + 1,
    hour: Math.floor(minuteOfDay / 60),
    minute: minuteOfDay % 60,
  };
}

const two = (n: number) => String(n).padStart(2, "0");

/** "Year 1 · Mar 4 · 06:24" */
export function formatSimDate(d: SimDate): string {
  return `Year ${d.year} · ${MONTHS[d.month - 1] ?? "?"} ${d.day} · ${two(d.hour)}:${two(d.minute)}`;
}

export function formatSimMinute(minutes: number): string {
  return formatSimDate(simDate(minutes));
}

/** "1×", "3×", "10×" for a speed in simulated seconds per real second. */
export function formatSpeed(speed: number, speed1x: number): string {
  const multiple = speed / speed1x;
  const text = Number.isInteger(multiple) ? String(multiple) : multiple.toFixed(1);
  return `${text}×`;
}

export function formatDistance(metres: number): string {
  if (Math.abs(metres) >= 1000) {
    const km = metres / 1000;
    return `${km >= 100 ? km.toFixed(0) : km.toFixed(1)} km`;
  }
  return `${Math.round(metres)} m`;
}

export function formatBytes(bytes: number): string {
  if (bytes >= 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  if (bytes >= 1024) return `${Math.round(bytes / 1024)} kB`;
  return `${bytes} B`;
}

export function formatPercent(fraction: number): string {
  return `${Math.round(fraction * 100)}%`;
}

/** Local wall-clock time of Unix milliseconds. */
export function formatWhen(unixMs: number): string {
  if (!unixMs) return "never";
  return new Date(unixMs).toLocaleString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

/** "2 min ago" style age of Unix milliseconds relative to `now`. */
export function formatAge(unixMs: number, now = Date.now()): string {
  if (!unixMs) return "never";
  const seconds = Math.max(0, Math.round((now - unixMs) / 1000));
  if (seconds < 45) return "just now";
  const minutes = Math.round(seconds / 60);
  if (minutes < 60) return `${minutes} min ago`;
  const hours = Math.round(minutes / 60);
  if (hours < 48) return `${hours} h ago`;
  return formatWhen(unixMs);
}

/** A size label for a map side: "2048 cells · 16 km". */
export function formatMapSize(cells: number, cellSizeM: number): string {
  return `${cells} cells · ${formatDistance(cells * cellSizeM)}`;
}
