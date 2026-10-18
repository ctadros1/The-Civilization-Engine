//! Walkers at bottlenecks (M8b slice CN; ADR-0029 §4; see [`crate::traffic`]). A walk that passes
//! an open crossing or an open gate is planned free-flow as before, and its arrival at each one is
//! an event: there it takes the facility's next free slot in its direction, and a wait of a
//! minute or more holds it at the approach and re-times the rest of the walk as a new revision of
//! the trip (ADR-0003 §2). Its household remembers the wait. A world with no crossing and no
//! enclosure schedules nothing new.

use super::*;
use crate::traffic::{self, Ahead, Facility, Pending};

/// Waits a household holds at a facility, faded, below which it lets go of them.
const LEAST_WAITS: f32 = 0.5;

impl Population {
    /// The facilities on the walking grid as they stand: rebuilt when a crossing opens or fails
    /// or a gate opens or shuts.
    pub(super) fn facilities_now(&mut self, ctx: &Ctx) {
        let rev = (
            ctx.land.crossings.revision(),
            ctx.land.enclosures.revision(),
        );
        if self.facilities_rev == Some(rev) {
            return;
        }
        self.facilities_rev = Some(rev);
        self.facilities.clear();
        for c in ctx.land.crossings.list.iter().filter(|c| c.open()) {
            let width = f32::from(c.members.max(1)) * c.diameter_cm / 100.0;
            let f = Facility::Crossing(c.id);
            self.facilities
                .extend(c.cells.iter().map(|&cell| (cell, f, width)));
        }
        for e in &ctx.land.enclosures.list {
            let standing = e.standing();
            for (i, s) in e.sectors.iter().enumerate() {
                if i < standing && !s.gap() && s.gate.is_some_and(|g| g.open) {
                    let f = Facility::Gate {
                        enclosure: e.id,
                        sector: i as u16,
                    };
                    let width = traffic::GATE_OPENING_M as f32;
                    self.facilities
                        .extend(s.cells.iter().map(|&cell| (cell, f, width)));
                }
            }
        }
        self.facilities
            .sort_unstable_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        self.facilities.dedup_by_key(|f| f.0);
    }

    /// The next facility along `trip` reached at or after `from_min`, past `skip`, and the way it
    /// is passed.
    fn pending_pass(
        &self,
        ctx: &Ctx,
        trip: &Trip,
        from_min: f32,
        skip: Option<Facility>,
    ) -> Option<Pending> {
        if self.facilities.is_empty() {
            return None;
        }
        let list = &self.facilities;
        let at = |c: u32| {
            list.binary_search_by_key(&c, |f| f.0)
                .ok()
                .map(|i| list[i].1)
        };
        let nav = ctx.nav;
        let ahead = traffic::ahead(
            &trip.points,
            &trip.minutes,
            from_min,
            skip,
            ctx.map.cell_size_m,
            ctx.map.width,
            ctx.map.height,
            at,
            |a, b| crossings::better_corner(nav, a, b),
        )?;
        let width = list
            .iter()
            .find(|f| f.1 == ahead.facility)
            .map_or(traffic::FILE_M as f32, |f| f.2);
        let (shared, files) = traffic::files(f64::from(width));
        let dir = if shared {
            traffic::SHARED
        } else {
            direction(ctx, &ahead)
        };
        Some(Pending {
            ahead,
            dir,
            headway_s: traffic::headway_s(files) as f32,
        })
    }

    /// Sets the walk's next bottleneck from `from_min` on, past `skip`, and schedules its arrival
    /// there.
    pub(super) fn schedule_pass(
        &mut self,
        ctx: &mut Ctx,
        h: Handle<Person>,
        from_min: f32,
        skip: Option<Facility>,
    ) {
        let Some(pending) = self
            .people
            .get(h)
            .and_then(|p| p.trip.as_ref())
            .map(|t| self.pending_pass(ctx, t, from_min, skip))
        else {
            return;
        };
        let Some(p) = self.people.get_mut(h) else {
            return;
        };
        let Some(trip) = p.trip.as_mut() else {
            return;
        };
        trip.ahead = pending;
        if let Some(pd) = pending {
            let at = trip
                .depart
                .plus_minutes(pd.ahead.at_min.floor() as i64)
                .max(ctx.now.plus_minutes(1));
            let event = AgentEvent::Pass {
                person: p.id,
                version: p.act.version,
                trip: trip.id,
            };
            ctx.schedule.push((at, event));
        }
    }

    /// A walker reaches the bottleneck their walk was bound for: they pass at its next free slot,
    /// and a wait of a minute or more holds them at its approach and re-times the rest of the walk.
    pub(super) fn reach_bottleneck(&mut self, ctx: &mut Ctx, h: Handle<Person>) {
        let now = ctx.now;
        let Some(p) = self.people.get(h) else {
            return;
        };
        let Some((pd, depart)) = p.trip.as_ref().and_then(|t| Some((t.ahead?, t.depart))) else {
            return;
        };
        let (who, household) = (p.id, p.household);
        let at_s = (depart.minutes() as f64 + f64::from(pd.ahead.at_min)) * 60.0;
        let day = now.day_index();
        let hour = ((at_s / 3600.0).floor() as i64).rem_euclid(24) as usize;
        let passage = self.traffic.pass(
            pd.ahead.facility,
            pd.dir,
            at_s,
            f64::from(pd.headway_s),
            day,
            hour,
        );
        let waited = (passage.waited_s() / 60.0) as f32;
        let mut from = pd.ahead.at_min;
        if waited >= 1.0 {
            let half = ctx.params.places.use_half_life_days;
            self.traffic
                .waited(household, pd.ahead.facility, waited, day, half);
            if waited >= traffic::LONG_WAIT_MIN {
                let settlement = self.household(household).and_then(|x| x.settlement);
                let time = depart.plus_minutes(pd.ahead.at_min.floor() as i64);
                self.traffic.long_wait(traffic::LongWait {
                    id: 0,
                    person: who,
                    household,
                    settlement,
                    facility: pd.ahead.facility,
                    at: pd.ahead.at,
                    time,
                    minutes: waited,
                    ahead: passage.ahead,
                });
            }
            let Some(p) = self.people.get_mut(h) else {
                return;
            };
            let Some(trip) = p.trip.as_mut() else {
                return;
            };
            // Held at the approach: two vertices there, the wait between them.
            let k = (pd.ahead.seg + 1).min(trip.points.len());
            trip.points.splice(k..k, [pd.ahead.at, pd.ahead.at]);
            trip.minutes
                .splice(k..k, [pd.ahead.at_min, pd.ahead.at_min + waited]);
            for m in &mut trip.minutes[k + 2..] {
                *m += waited;
            }
            trip.rev += 1;
            let ends = trip
                .depart
                .plus_minutes(trip.duration_minutes().ceil().max(1.0) as i64);
            p.act.step_ends = ends;
            p.act.version = p.act.version.wrapping_add(1);
            let (id, version) = (p.id, p.act.version);
            ctx.schedule_step(ends, id, version);
            from = pd.ahead.at_min + waited;
        }
        self.schedule_pass(ctx, h, from, Some(pd.ahead.facility));
    }

    /// Lets go of facilities' directions nobody waits at, tallies long faded and waits households
    /// no longer hold (with the day's other forgetting).
    pub(super) fn forget_traffic(&mut self, ctx: &Ctx, day: i64) {
        let half = ctx.params.places.use_half_life_days;
        let now_s = ctx.now.minutes() as f64 * 60.0;
        let hh = &self.hh_index;
        self.traffic
            .forget(now_s, day, half, LEAST_WAITS, |h| hh.contains_key(&h));
    }
}

/// Which way a walker reaching `ahead` goes through it: across a crossing from its first bank
/// (0) or its second (1); through a gate from the side its sector's line has on its right (0) or
/// its left (1).
fn direction(ctx: &Ctx, ahead: &Ahead) -> u8 {
    let p = ahead.at;
    match ahead.facility {
        Facility::Crossing(id) => {
            ctx.land
                .crossings
                .list
                .iter()
                .find(|c| c.id == id)
                .map_or(0, |c| {
                    let b0 = cell_centre(ctx.map, c.banks[0] as usize);
                    let b1 = cell_centre(ctx.map, c.banks[1] as usize);
                    let d = |b: (f32, f32)| (b.0 - p.0).hypot(b.1 - p.1);
                    u8::from(d(b1) < d(b0))
                })
        }
        Facility::Gate { enclosure, sector } => ctx
            .land
            .enclosures
            .list
            .iter()
            .find(|e| e.id == enclosure)
            .and_then(|e| e.sectors.get(usize::from(sector)))
            .map_or(0, |s| {
                let cross = (s.b.0 - s.a.0) * (p.1 - s.a.1) - (s.b.1 - s.a.1) * (p.0 - s.a.0);
                u8::from(cross < 0.0)
            }),
    }
}
