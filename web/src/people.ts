// Where people are between snapshots (ADR-0003 §2): a walker's position is interpolated along the
// route and timings of their trip; the simulation minute is estimated from the last clock and its
// speed. Pure functions, so they can be tested without a renderer.

import type { ActivityInfo, Clock, PersonBrief, TripInfo } from "./net/messages.js";

/** Position on a trip at simulation minute `t` (fractional), metres. */
export function positionOnTrip(trip: TripInfo, t: number): [number, number] {
  const p = trip.points;
  const m = trip.minutes;
  const n = Math.min(m.length, p.length / 2);
  if (n === 0) return [0, 0];
  const since = t - trip.departMinute;
  if (since <= 0) return [p[0]!, p[1]!];
  for (let k = 1; k < n; k++) {
    const m0 = m[k - 1]!;
    const m1 = m[k]!;
    if (since <= m1) {
      const f = m1 > m0 ? (since - m0) / (m1 - m0) : 1;
      const x0 = p[2 * k - 2]!;
      const y0 = p[2 * k - 1]!;
      return [x0 + (p[2 * k]! - x0) * f, y0 + (p[2 * k + 1]! - y0) * f];
    }
  }
  return [p[2 * n - 2]!, p[2 * n - 1]!];
}

/**
 * The simulation minute now, estimated from the latest clock: its minute plus the real time since
 * it arrived at its speed, never more than a real second's worth ahead (the next snapshot
 * corrects it).
 */
export function estimateMinute(clock: Clock, receivedAtMs: number, nowMs: number): number {
  if (clock.paused) return clock.minute;
  const perSecond = clock.speed / 60;
  const ahead = ((nowMs - receivedAtMs) / 1000) * perSecond;
  return clock.minute + Math.min(Math.max(0, ahead), Math.max(1, perSecond));
}

/** Where to draw a person at minute `t`: along their trip when it is known, else where the
 * snapshot placed them. */
export function personPosition(
  person: PersonBrief,
  trips: ReadonlyMap<number, TripInfo>,
  t: number,
): [number, number] {
  if (person.trip !== 0) {
    const trip = trips.get(person.trip);
    if (trip && trip.rev === person.tripRev) return positionOnTrip(trip, t);
  }
  return [person.x, person.y];
}

/** Trips the snapshot mentions that are not cached at their current revision. */
export function missingTrips(people: PersonBrief[], trips: ReadonlyMap<number, TripInfo>): number[] {
  const out: number[] = [];
  for (const p of people) {
    if (p.trip === 0) continue;
    const known = trips.get(p.trip);
    if (!known || known.rev !== p.tripRev) out.push(p.trip);
  }
  return out;
}

/** Drops cached trips that no living person is on any more. */
export function pruneTrips(people: PersonBrief[], trips: Map<number, TripInfo>): void {
  const live = new Set(people.map((p) => p.trip));
  for (const id of trips.keys()) if (!live.has(id)) trips.delete(id);
}

/** Colours of the core activities, by content id. Other activities get a stable hue. */
const CORE_COLOURS: Record<string, number> = {
  "core:activity/sleep": 0x7480c4,
  "core:activity/eat": 0xe6c75a,
  "core:activity/fetch_water": 0x52bde0,
  "core:activity/gather_plants": 0x74c463,
  "core:activity/gather_wood": 0xa8794a,
  "core:activity/hunt": 0xd6544a,
  "core:activity/fish": 0x3f8fd1,
  "core:activity/socialize": 0xeb8d4c,
  "core:activity/rest": 0xbdb6ab,
  "core:activity/play": 0xe796c6,
};

function hashHue(text: string): number {
  let h = 2166136261;
  for (let i = 0; i < text.length; i++) {
    h ^= text.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return (h >>> 0) % 360;
}

function hsl(h: number, s: number, l: number): number {
  const k = (n: number) => (n + h / 30) % 12;
  const a = s * Math.min(l, 1 - l);
  const f = (n: number) => l - a * Math.max(-1, Math.min(k(n) - 3, Math.min(9 - k(n), 1)));
  return (
    (Math.round(f(0) * 255) << 16) | (Math.round(f(8) * 255) << 8) | Math.round(f(4) * 255)
  );
}

/** The map colour of an activity. */
export function activityColour(activity: ActivityInfo | undefined): number {
  if (!activity) return 0xffffff;
  return CORE_COLOURS[activity.id] ?? hsl(hashHue(activity.id), 0.55, 0.62);
}

/** A CSS colour from a 0xRRGGBB number. */
export function cssColour(colour: number): string {
  return `#${colour.toString(16).padStart(6, "0")}`;
}

/**
 * Spreads people who stand at the same spot (a household at home) around a small ring of
 * `radius` metres, in the order given, so each dot can be seen and clicked. Presentation only:
 * the kernel's positions are unchanged. Points closer than `near` metres count as one spot.
 */
export function spread(
  points: [number, number][],
  radius: number,
  near: number,
): [number, number][] {
  // Each spot is anchored at its first person; tens of people make this quadratic pass cheap.
  const spots: { x: number; y: number; members: number[] }[] = [];
  points.forEach(([x, y], i) => {
    const spot = spots.find((s) => Math.hypot(s.x - x, s.y - y) < near);
    if (spot) spot.members.push(i);
    else spots.push({ x, y, members: [i] });
  });
  const out = points.map(([x, y]) => [x, y] as [number, number]);
  for (const spot of spots) {
    if (spot.members.length < 2) continue;
    spot.members.forEach((i, k) => {
      const angle = (2 * Math.PI * k) / spot.members.length;
      out[i] = [spot.x + radius * Math.cos(angle), spot.y + radius * Math.sin(angle)];
    });
  }
  return out;
}
