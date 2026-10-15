//! Fires in the world (M6b slice BC; ADR-0022 §4, §5, §13, §14): a building's fire stepped a
//! minute at a time while anything burns, its heat on the buildings near it, what it does to their
//! parts, the goods under their roofs and those asleep inside, and the record of it.
//!
//! Fires start from the observer's hand and, from slice BD, where fire is used: a household's
//! hearth and the fires it works at home escape now and then, keyed by building and minute, and
//! someone awake there puts most out at once. An escape left alone grows inside for some minutes
//! before it reaches the covering; someone coming by puts it out, as does someone asleep inside
//! who wakes to it in time.
//!
//! People notice a burning building by being in it or seeing it, and whoever comes to know of a
//! fire shouts once: those within earshot hear, wake if asleep, and stop what they are doing to
//! decide again (ADR-0022 §6). Each comes to know only what they saw or heard (§7), and someone
//! keeps away from home only once they know it burns. People carry water, strip threatened roofs
//! and salvage goods. Those awake in a
//! burning building get out; those still asleep wake when it is fully involved around them, and
//! a share of them do not get out.

use super::*;
use crate::decide::{FireFacts, FireTarget, FireWater};
use crate::fire::{
    Conflagration, Fire, FireParams, Hearth, Ignition, Kindled, Knew, Noticed, Outline, Salvaged,
    Stage, Stripped, Thrown, Weighed, charred_loss,
};
use crate::history::{Cause, FireStep};
use crate::influence::InfluenceKind;
use crate::person::FROM_STORE;
use civ_grammar::GroupKind;
use civ_land::{Building, GroupState, Land};

/// Purpose tag for a covering catching from its neighbours' heat, keyed by building and minute.
pub const PURPOSE_FIRE_CATCH: u64 = 0x6669_7265_6361_7431; // "firecat1"
/// Purpose tag for whether a sleeper caught by a fully involved building gets out, keyed by
/// person and fire.
pub const PURPOSE_FIRE_CAUGHT: u64 = 0x6669_7265_6361_7531; // "firecau1"

/// Purpose tag for the minutes at which a home's fires in use may escape on a day (slice BD),
/// keyed by building and day.
pub const PURPOSE_IGNITE: u64 = 0x6967_6e69_7465_3031; // "ignite01"
/// Purpose tag for whether a candidate ignition happens, and from which fire, keyed by building
/// and minute.
pub const PURPOSE_IGNITE_TAKES: u64 = 0x6967_6e74_616b_3031; // "igntak01"
/// Purpose tag for how long an escape left alone grows before it reaches the covering, keyed by
/// building and minute.
pub const PURPOSE_IGNITE_GROWS: u64 = 0x6967_6e67_7277_3031; // "igngrw01"
/// Purpose tag for when someone asleep in a building wakes to an escape there, keyed by person
/// and ignition.
pub const PURPOSE_FIRE_WAKE: u64 = 0x6669_7265_7761_6b31; // "firewak1"

/// The share of those still asleep in a building when it is fully involved around them who do
/// not get out (a design prior: ADR-0022 §5 has sleepers never woken die; a building fully
/// involved wakes anyone, and some still get out). Those woken earlier, by a shout or an escape
/// they wake to, are awake by then and get out.
pub const CAUGHT_ASLEEP_DIES: f64 = 0.5;

/// Building `b`'s outline as fire sees it: the edge of its roof, from its plot (the box its roof
/// covers); a round roof as the circle in that box.
pub fn outline_of(plots: &FastMap<PermanentId, civ_land::RectCm>, b: &Building) -> Option<Outline> {
    let r = plots.get(&b.plot)?;
    let (x0, y0) = (f64::from(r.x) / 100.0, f64::from(r.y) / 100.0);
    let (x1, y1) = (f64::from(r.x + r.w) / 100.0, f64::from(r.y + r.h) / 100.0);
    Some(match b.spec.footprint {
        civ_grammar::Footprint::Round { .. } => Outline::Round {
            x: (x0 + x1) / 2.0,
            y: (y0 + y1) / 2.0,
            r: (x1 - x0).min(y1 - y0) / 2.0,
        },
        civ_grammar::Footprint::Rect { .. } => Outline::Box { x0, y0, x1, y1 },
    })
}

/// A place outside the roof to pull from, on the side nearest the worker.
fn pulling_place(o: &Outline, from: (f32, f32), margin: f64) -> (f32, f32) {
    let (x, y) = o.centre();
    let (mut dx, mut dy) = (f64::from(from.0) - x, f64::from(from.1) - y);
    let distance = dx.hypot(dy);
    if distance <= 1e-6 {
        dx = 1.0;
        dy = 0.0;
    } else {
        dx /= distance;
        dy /= distance;
    }
    let radius = match *o {
        Outline::Round { r, .. } => r,
        Outline::Box { x0, y0, x1, y1 } => {
            let tx = if dx.abs() > 1e-6 {
                (x1 - x0) * 0.5 / dx.abs()
            } else {
                f64::INFINITY
            };
            let ty = if dy.abs() > 1e-6 {
                (y1 - y0) * 0.5 / dy.abs()
            } else {
                f64::INFINITY
            };
            tx.min(ty)
        }
    };
    (
        (x + dx * (radius + margin)) as f32,
        (y + dy * (radius + margin)) as f32,
    )
}

/// Whether point `p` lies within outline `o`.
fn within(o: &Outline, p: (f32, f32)) -> bool {
    let (x, y) = (f64::from(p.0), f64::from(p.1));
    match *o {
        Outline::Round { x: cx, y: cy, r } => (x - cx).hypot(y - cy) <= r,
        Outline::Box { x0, y0, x1, y1 } => x >= x0 && x <= x1 && y >= y0 && y <= y1,
    }
}

/// How far point `p` lies outside outline `o`, metres (0 within it).
fn outside(o: &Outline, p: (f32, f32)) -> f64 {
    let (x, y) = (f64::from(p.0), f64::from(p.1));
    match *o {
        Outline::Round { x: cx, y: cy, r } => ((x - cx).hypot(y - cy) - r).max(0.0),
        Outline::Box { x0, y0, x1, y1 } => {
            let dx = (x0 - x).max(x - x1).max(0.0);
            let dy = (y0 - y).max(y - y1).max(0.0);
            dx.hypot(dy)
        }
    }
}

/// The most a home's fires in use may give, ignitions an hour at the reference dryness: its
/// hearth in whichever state gives more, and a worked fire for each of its `members`. The day's
/// candidate minutes are drawn at this rate, and each kept by the rate at its minute.
fn ignition_bound(p: &FireParams, members: usize) -> f64 {
    p.hearth_lit_per_hour.max(p.hearth_banked_per_hour) + p.worked_per_hour * members as f64
}

/// Every plot's box, by its id.
pub fn plot_boxes(land: &Land) -> FastMap<PermanentId, civ_land::RectCm> {
    land.plots.iter().map(|p| (p.id, p.rect)).collect()
}

/// The group kinds of timber members that char while a building is fully involved.
fn chars(kind: GroupKind) -> bool {
    matches!(
        kind,
        GroupKind::Posts
            | GroupKind::TieBeams
            | GroupKind::LoftJoists
            | GroupKind::FloorJoists
            | GroupKind::RaisedFloor
            | GroupKind::RoofFrame
    )
}

impl Population {
    /// A direct observation becomes a dated account; an alarm alone names no building.
    fn fire_word(&mut self, knew: Knew) {
        if knew.how == Noticed::Heard {
            return;
        }
        let Some(f) = self.fires.get(knew.fire) else {
            return;
        };
        let (fire, building, household) = (f.id, f.building, f.household);
        let Some(settlement) = self.household(household).and_then(|h| h.settlement) else {
            return;
        };
        let claim = self
            .word
            .claims
            .iter()
            .find(|c| c.kind == crate::word::ClaimKind::Fire && c.fire == Some(fire))
            .map(|c| c.id)
            .unwrap_or_else(|| {
                self.word.make(crate::word::Claim {
                    id: 0,
                    kind: crate::word::ClaimKind::Fire,
                    settlement,
                    day: knew.at.day_index(),
                    subject: Some(building),
                    grievance: None,
                    suspected: None,
                    fire: Some(fire),
                    event: None,
                })
            });
        self.word.hear(
            knew.person,
            claim,
            knew.at.day_index(),
            None,
            Some(knew.person),
        );
    }

    /// On the month's first day, tell the previous month's household-fire escapes.
    pub(super) fn fire_month(&mut self, ctx: &Ctx) {
        let now = ctx.now;
        if now.date().day != 1 {
            return;
        }
        let previous = now.plus_minutes(-now.minute_of_day() - 1).date();
        // Escapes are activity-driven records; observer-set fires have no ignition to count.
        let mut counts = std::collections::BTreeMap::new();
        for g in &self.fires.ignitions {
            let date = g.at.date();
            if date.year != previous.year || date.month != previous.month {
                continue;
            }
            let c = counts.entry(g.settlement).or_insert([0u32; 5]);
            c[0] += 1;
            if g.put_out_by.is_some() {
                c[1] += 1;
            } else if g.fire.is_some() {
                c[2] += 1;
            } else if g.ended.is_some() {
                c[3] += 1;
            } else {
                c[4] += 1;
            }
        }
        for (settlement, [escaped, put_out, caught, went_out, growing]) in counts {
            let mut words = format!(
                "Last month's household fires: {escaped} escaped; {put_out} were put out; {caught} reached a roof; {went_out} went out with nothing catching."
            );
            if growing > 0 {
                words.push_str(&format!(
                    " {growing} still grow inside at this month's beginning."
                ));
            }
            self.chronicle_push(
                now,
                ChronicleKind::Fire,
                Vec::new(),
                settlement,
                None,
                f64::from(FireStep::Month as u8),
                words,
            );
        }
    }

    /// Runs the calendar-month fire summary over recorded escapes (tests).
    #[doc(hidden)]
    pub fn fire_month_for_tests(&mut self, ctx: &Ctx) {
        self.fire_month(ctx);
    }

    /// Whether person `person` knows that a building of household `household` burns now: they
    /// were in it, woke to it or saw it (ADR-0022 §7; hearing a shout names no building). A fire
    /// burns at most as long as its stages last, so only the newest records are looked at.
    pub fn knows_home_afire(
        &self,
        person: PermanentId,
        household: PermanentId,
        now: SimTime,
        params: &PeopleParams,
    ) -> bool {
        let p = &params.fire;
        let span = (p.involve_minutes + p.involved_minutes).ceil() as i64 + 1;
        let since = now.minutes() - span;
        self.fires
            .knew
            .iter()
            .rev()
            .take_while(|k| k.at.minutes() >= since)
            .any(|k| {
                k.person == person
                    && k.how != Noticed::Heard
                    && self
                        .fires
                        .get(k.fire)
                        .is_some_and(|f| f.burning() && f.household == household)
            })
    }

    /// Whether person `person` knows of fire `fire`, and of the conflagration (or lone fire) it
    /// is part of.
    fn knows_of(&self, fire: u32, person: PermanentId) -> (bool, bool) {
        let root = self.fires.root(fire);
        let since = self.fires.get(root).map_or(0, |f| f.began.minutes());
        let (mut knows_fire, mut knows_root) = (false, false);
        for k in self.fires.knew.iter().rev() {
            if k.at.minutes() < since {
                break;
            }
            if k.person == person && self.fires.root(k.fire) == root {
                knows_root = true;
                knows_fire |= k.fire == fire;
            }
        }
        (knows_fire, knows_root)
    }

    /// Someone awake in or within `within_m` of outline `o` at minute `t`: the nearest, then the
    /// first by id.
    fn awake_beside(&self, o: &Outline, t: f64, within_m: f64) -> Option<PermanentId> {
        let mut beside: Option<(f64, PermanentId)> = None;
        for (_, q) in self.people.iter() {
            if q.asleep {
                continue;
            }
            let d = outside(o, q.position_at(t));
            if d <= within_m && beside.is_none_or(|(bd, bid)| (d, q.id) < (bd, bid)) {
                beside = Some((d, q.id));
            }
        }
        beside.map(|(_, who)| who)
    }

    /// Person `h` stops where they stand and decides again in a minute: woken, or what they were
    /// doing ended. Someone awake carrying a load goes on to set it down where it was going, as
    /// nothing holds goods on the ground.
    fn stop_and_think(&mut self, ctx: &mut Ctx, h: Handle<Person>) {
        let t = ctx.now.minutes() as f64;
        let Some(p) = self.people.get_mut(h) else {
            return;
        };
        let laden = p.carrying.kg > 0.0 || p.carrying.water_l > 0.0;
        if laden && !p.asleep {
            return;
        }
        p.pos = p.position_at(t);
        self.wait(ctx, h, 1);
    }

    /// The index of the covering's good of building `b`, by the catalog's goods: the material of
    /// its first covering group.
    fn covering_good(&mut self, ctx: &Ctx, b: &Building) -> Option<usize> {
        let def = ctx
            .catalog
            .building_index(&b.spec.program)
            .and_then(|d| ctx.catalog.buildings.get(d))?;
        let e = self.expansion_of(b, def)?;
        let slot = e
            .groups
            .iter()
            .find(|g| g.kind == GroupKind::Covering)
            .and_then(|g| g.material)?;
        def.materials.get(usize::from(slot)).copied()
    }

    /// The threshold, kW/m², at which building `b`'s covering catches today, if it can: a roofed,
    /// standing building whose covering is not yet burnt away, of a good that catches.
    fn catch_threshold(&mut self, ctx: &Ctx, b: &Building) -> Option<f64> {
        if !b.roofed() {
            return None;
        }
        let burnt = b
            .condition
            .iter()
            .filter(|c| GroupKind::of_group(c.group) == Some(GroupKind::Covering))
            .all(|c| c.loss >= 1.0 || c.state == GroupState::Failed);
        if burnt {
            return None;
        }
        let good = self.covering_good(ctx, b)?;
        let catch = ctx.catalog.goods.get(good)?.fire?.catch_kw_m2;
        let today = &ctx.land.weather.today;
        ctx.params
            .fire
            .threshold(catch, f64::from(today.fuel_moisture))
    }

    /// At midnight, each household's roofed home draws the minutes of the day ahead at which one
    /// of its fires in use may escape (slice BD; ADR-0022 §3): candidates at the most its hearth
    /// and worked fires could give, scaled by dryness, from a key of its own and the day, so the
    /// same day lived at any speed has the same candidates. Each is weighed at its minute.
    pub(super) fn plan_ignitions(&mut self, ctx: &mut Ctx, day: i64) {
        let p = &ctx.params.fire;
        let scale = p.ignition_scale(f64::from(ctx.land.weather.today.fuel_moisture));
        if scale.is_nan() || scale <= 0.0 {
            return;
        }
        let start = day * 1440;
        for b in &ctx.land.buildings {
            if !b.roofed() {
                continue;
            }
            let home = ctx
                .catalog
                .building_index(&b.spec.program)
                .and_then(|d| ctx.catalog.buildings.get(d))
                .is_some_and(|d| d.use_ == civ_land::PlotUse::Dwelling);
            let members = self.household(b.household).map_or(0, |x| x.members.len());
            if !home || members == 0 {
                continue;
            }
            let mean = ignition_bound(p, members) * scale * 24.0;
            let mut rng = Rng64::from_key(&[ctx.seed, PURPOSE_IGNITE, b.id.get(), day as u64]);
            let count = civ_land::poisson_quantile(mean, rng.next_f64());
            for _ in 0..count {
                // Within the day, after midnight's own work and before the next.
                let minute = start + 1 + rng.below(1438) as i64;
                ctx.schedule.push((
                    SimTime::from_minutes(minute),
                    AgentEvent::Ignite { building: b.id },
                ));
            }
        }
    }

    /// Whether person `p` is working a fire: making with a recipe that burns fuel (baking,
    /// firing pots).
    fn works_fire(ctx: &Ctx, p: &Person) -> bool {
        let catalog = ctx.catalog;
        let Some(r) = catalog
            .activities
            .get(usize::from(p.act.def))
            .and_then(|a| a.recipe)
            .and_then(|r| catalog.recipes.get(r))
        else {
            return false;
        };
        r.inputs.iter().chain(&r.session_inputs).any(|&(g, _)| {
            catalog
                .goods
                .get(g)
                .is_some_and(|d| d.purpose == GoodUse::Fuel)
        })
    }

    /// A candidate ignition at home `building` comes due (slice BD; ADR-0022 §3). It happens by
    /// the rate of the home's fires in use now against the bound its candidates were drawn at:
    /// the hearth lit while any of its people is awake at home and banked otherwise, and a worked
    /// fire for each of them baking or firing pots there. Someone awake in or beside the building
    /// puts it out at once, and only the count records it; otherwise it grows inside, stepped a
    /// minute at a time, until it is put out or reaches the covering.
    pub(super) fn ignite(&mut self, ctx: &mut Ctx, building: PermanentId) {
        let now = ctx.now;
        let p = ctx.params.fire.clone();
        let Some(b) = ctx
            .land
            .buildings
            .iter()
            .find(|b| b.id == building)
            .cloned()
        else {
            return;
        };
        let growing = self
            .fires
            .ignitions
            .iter()
            .rev()
            .any(|g| g.growing() && g.building == building);
        if !b.roofed() || growing || self.fires.burning_in(building).is_some() {
            return;
        }
        let Some(x) = self.household(b.household) else {
            return;
        };
        let (members, settlement) = (x.members.clone(), x.settlement);
        let Some(o) = ctx
            .land
            .plots
            .iter()
            .find(|q| q.id == b.plot)
            .map(|q| (q.id, q.rect))
            .and_then(|(id, rect)| outline_of(&std::iter::once((id, rect)).collect(), &b))
        else {
            return;
        };
        let t = now.minutes() as f64;
        let (mut awake, mut workers) = (false, 0usize);
        for id in &members {
            let Some(m) = self.person(*id) else {
                continue;
            };
            if m.asleep || !within(&o, m.position_at(t)) {
                continue;
            }
            awake = true;
            workers += usize::from(Self::works_fire(ctx, m));
        }
        let hearth = if awake {
            p.hearth_lit_per_hour
        } else {
            p.hearth_banked_per_hour
        };
        let worked = workers as f64 * p.worked_per_hour;
        let bound = ignition_bound(&p, members.len());
        let mut rng = Rng64::from_key(&[
            ctx.seed,
            PURPOSE_IGNITE_TAKES,
            building.get(),
            now.minutes() as u64,
        ]);
        let u = rng.next_f64() * bound;
        if bound.is_nan() || bound <= 0.0 || u >= hearth + worked {
            return;
        }
        let kind = if u >= hearth {
            Hearth::Worked
        } else if awake {
            Hearth::Lit
        } else {
            Hearth::Banked
        };
        // Someone awake in or beside it puts it out at once; the nearest, then the first by id.
        let put_out_by = self.awake_beside(&o, t, p.beside_m);
        // Left alone, it grows inside for a while before it reaches the covering.
        let grows = if put_out_by.is_some() {
            0.0
        } else {
            let key = [
                ctx.seed,
                PURPOSE_IGNITE_GROWS,
                building.get(),
                now.minutes() as u64,
            ];
            let [lo, hi] = p.grow_minutes;
            lo.max(0.0) + Rng64::from_key(&key).next_f64() * (hi - lo).max(0.0)
        };
        let was = self.fires.active();
        self.fires.ignitions.push(Ignition {
            id: self.fires.ignitions.len() as u32 + 1,
            at: now,
            building,
            household: b.household,
            settlement,
            hearth: kind,
            moisture: ctx.land.weather.today.fuel_moisture,
            grows_min: grows as f32,
            ended: put_out_by.map(|_| now),
            put_out_by,
            fire: None,
        });
        if !was && self.fires.active() {
            ctx.schedule.push((now.plus_minutes(1), AgentEvent::Fire));
        }
    }

    /// The roofs a fire person `h` knows of threatens, and how they would bring water to each
    /// (M6b slice BD, step three; ADR-0022 §6): each roof the heat of a fire burning now reaches,
    /// worth saving by whose it is (their household's, their close kin's or a neighbour's) and by
    /// how near its exposure stands to what its covering needs to catch. The water is their
    /// household's store when it holds a load, else the nearest source they would draw at.
    /// Nothing for one who knows of no fire burning now.
    pub(super) fn fire_facts(
        &mut self,
        ctx: &Ctx,
        h: Handle<Person>,
        hh: &Household,
        water_l: f64,
        short: bool,
        field_key: PermanentId,
    ) -> Option<FireFacts> {
        if self.fires.exposure.is_empty() || !self.fires.any_burning() {
            return None;
        }
        let now = ctx.now;
        let p = self.people.get(h)?;
        let (me, age, pos) = (p.id, p.age_years(now), p.pos);
        let fp = &ctx.params.fire;
        // Burning groups identify goods already carried out; they do not supply perception.
        let roots: Vec<u32> = self
            .fires
            .list
            .iter()
            .filter(|f| f.burning())
            .map(|f| self.fires.root(f.id))
            .collect();
        let span = (fp.involve_minutes + fp.involved_minutes).ceil() as i64 + 1;
        let since = self
            .fires
            .list
            .iter()
            .filter(|f| f.burning())
            .map(|f| f.began.minutes())
            .min()
            .unwrap_or(now.minutes())
            - span;
        let known: std::collections::BTreeSet<u32> = self
            .fires
            .knew
            .iter()
            .rev()
            .take_while(|k| k.at.minutes() >= since)
            .filter(|k| k.person == me)
            .map(|k| k.fire)
            .collect();
        if known.is_empty() {
            return None;
        }
        // Where a load of water comes from.
        let carry = ctx.params.household.carry_water_l;
        let water_trip = if water_l >= carry {
            Some((FROM_STORE, hh.home, 1))
        } else {
            let home = self.homes.get(&field_key);
            self.water_source(ctx, home, Some((hh, short)), Some(me))
                .map(|water| {
                    let from = water
                        .well
                        .and_then(|w| ctx.land.wells.get(w))
                        .map_or(water.cell, |w| w.cell);
                    (from, water.at, 1 + water.lift_min.round().max(0.0) as u32)
                })
        };
        let speed = interpolate(&ctx.params.walk_speed_by_age, age).max(0.05) * 60.0;
        let kin = self.kin_households(hh);
        let plots = plot_boxes(ctx.land);
        let observed: Vec<Outline> = self
            .fires
            .list
            .iter()
            .filter(|f| known.contains(&f.id))
            .filter_map(|f| ctx.land.buildings.iter().find(|b| b.id == f.building))
            .filter_map(|b| outline_of(&plots, b))
            .collect();
        let burning: Vec<(u32, Outline)> = self
            .fires
            .list
            .iter()
            .filter(|f| f.burning())
            .filter_map(|f| {
                ctx.land
                    .buildings
                    .iter()
                    .find(|b| b.id == f.building)
                    .and_then(|b| outline_of(&plots, b))
                    .map(|o| (f.id, o))
            })
            .collect();
        // A connected physical scene, anchored in this person's recent records. Neither
        // causes, episode membership nor historical spread diagnostics supply knowledge.
        let mut included = std::collections::BTreeSet::new();
        let mut known_sources = Vec::new();
        loop {
            let before = included.len();
            for &(id, outline) in &burning {
                if !included.contains(&id)
                    && (known.contains(&id)
                        || observed
                            .iter()
                            .chain(&known_sources)
                            .any(|source| source.gap(&outline) <= fp.reach_m))
                {
                    included.insert(id);
                    known_sources.push(outline);
                }
            }
            if included.len() == before {
                break;
            }
        }
        if known_sources.is_empty() {
            return None;
        }
        let exposed: Vec<(PermanentId, f64)> = self
            .fires
            .exposure
            .iter()
            .filter(|&&(_, s)| s > 0.0)
            .copied()
            .collect();
        let mut targets = Vec::new();
        for (building, store) in exposed {
            let Some(b) = ctx
                .land
                .buildings
                .iter()
                .find(|b| b.id == building)
                .cloned()
            else {
                continue;
            };
            // Consider the physical scene of a fire they know, not unrelated distant heat.
            // Historical spread diagnostics never supply a person's task facts.
            let beside = outline_of(&plots, &b).is_some_and(|o| {
                known_sources
                    .iter()
                    .any(|source| source.gap(&o) <= fp.reach_m)
            });
            if !beside {
                continue;
            }
            let Some(threshold) = self.catch_threshold(ctx, &b) else {
                continue;
            };
            let near = (store / threshold.max(1e-6)).clamp(0.0, 1.0);
            let stake = if b.household == hh.id {
                1.0
            } else if kin.binary_search(&b.household).is_ok() {
                fp.kin_share
            } else {
                fp.neighbour_share
            };
            let points = fp.w_save * stake * near;
            if points <= 0.0 {
                continue;
            }
            let at = build::centre_m(&b.spec);
            let walk =
                |a: (f32, f32), b: (f32, f32)| f64::from(a.0 - b.0).hypot(f64::from(a.1 - b.1));
            // Stripping it: theirs, or another's with leave of one of its people there awake.
            let leave = b.household == hh.id
                || self.people.iter().any(|(_, q)| {
                    q.household == b.household
                        && !q.asleep
                        && walk(q.position_at(now.minutes() as f64), at) <= fp.reach_m
                });
            let strip_points = (leave && near > fp.strip_cost)
                .then_some(fp.w_save * stake * (near - fp.strip_cost));
            let outline = outline_of(&plot_boxes(ctx.land), &b);
            let empty = outline.is_some_and(|o| {
                !self
                    .people
                    .iter()
                    .any(|(_, q)| within(&o, q.position_at(now.minutes() as f64)))
            });
            let pull_def = ctx
                .catalog
                .activities
                .iter()
                .find(|d| d.behavior == Behavior::Pull);
            let equipped = pull_def.is_some_and(|d| {
                let stores = stores_now(hh, now, ctx.params, &ctx.catalog.goods);
                d.tools
                    .iter()
                    .all(|&g| stores.get(g).is_some_and(|&kg| kg > 0.0))
            });
            let place = self.pull_place(ctx, &b, pos);
            let pull_leave = b.household == hh.id
                || outline.is_some_and(|o| {
                    self.people.iter().any(|(_, q)| {
                        q.household == b.household
                            && !q.asleep
                            && outside(&o, q.position_at(now.minutes() as f64)) <= fp.reach_m
                    })
                });
            let pull_points =
                (pull_leave && empty && equipped && place.is_some() && near > fp.pull_cost)
                    .then_some(fp.w_save * stake * (near - fp.pull_cost));
            let pull_at = place.unwrap_or(at);
            // Carrying their household's goods out from under it.
            let salvage_points = if b.household == hh.id {
                let kg = self.burnable_under(ctx, &b, &roots);
                let carry = ctx.params.household.carry_kg;
                (kg > 0.0 && carry > 0.0)
                    .then(|| fp.w_save * fp.salvage_share * near * (kg / carry).min(1.0))
            } else {
                None
            };
            // Out of the fire's way: 15 m from it, away from the nearest fire burning.
            let away = self
                .fires
                .list
                .iter()
                .filter(|f| f.burning())
                .min_by(|a, c| walk(a.at, at).total_cmp(&walk(c.at, at)))
                .map_or(at, |f| {
                    let (dx, dy) = (at.0 - f.at.0, at.1 - f.at.1);
                    let d = dx.hypot(dy).max(0.1);
                    (at.0 + 15.0 * dx / d, at.1 + 15.0 * dy / d)
                });
            targets.push(FireTarget {
                building,
                at,
                points,
                water: water_trip.map(|(from, fill_at, fill_min)| FireWater {
                    from,
                    fill_at,
                    fill_min,
                    walk_min: (walk(pos, fill_at) + walk(fill_at, at)) / speed,
                }),
                strip_points,
                salvage_points,
                pull_points,
                pull_at,
                away,
                to_roof_min: walk(pos, at) / speed,
            });
        }
        targets.sort_by(|a, b| {
            b.points
                .total_cmp(&a.points)
                .then_with(|| a.building.cmp(&b.building))
        });
        targets.truncate(3);
        (!targets.is_empty()).then_some(FireFacts { targets })
    }

    /// Fire options from a specified known travel field (tests).
    #[doc(hidden)]
    pub fn fire_facts_for_tests(
        &mut self,
        ctx: &Ctx,
        who: PermanentId,
        field_key: PermanentId,
        water_l: f64,
    ) -> Option<FireFacts> {
        let h = *self.index.get(&who)?;
        let hh = self.household(self.people.get(h)?.household)?.clone();
        self.fire_facts(ctx, h, &hh, water_l, false, field_key)
    }

    /// Kilograms of goods that burn its household keeps under building `b`'s roof, less what was
    /// carried out from under it against the fires begun with `roots` (M6b slice BD, step four).
    fn burnable_under(&mut self, ctx: &Ctx, b: &Building, roots: &[u32]) -> f64 {
        let Some(bi) = ctx.land.buildings.iter().position(|x| x.id == b.id) else {
            return 0.0;
        };
        let kept = self.kept_by(ctx, b.household, bi);
        let under: f64 = ctx
            .catalog
            .goods
            .iter()
            .enumerate()
            .filter(|(_, d)| d.fire.is_some_and(|f| f.burns > 0.0))
            .map(|(g, _)| {
                kept.iter()
                    .map(|k| k.get(g).copied().unwrap_or(0.0))
                    .sum::<f64>()
            })
            .sum();
        let out: f64 = self
            .fires
            .salvaged
            .iter()
            .filter(|s| s.building == b.id && roots.contains(&s.fire))
            .map(|s| f64::from(s.kg))
            .sum();
        (under - out).max(0.0)
    }

    /// The fire burning nearest point `at`, as the first of its conflagration (or itself).
    fn fire_against(&self, at: (f32, f32)) -> Option<u32> {
        let d = |f: &Fire| f64::from(f.at.0 - at.0).hypot(f64::from(f.at.1 - at.1));
        self.fires
            .list
            .iter()
            .filter(|f| f.burning())
            .min_by(|a, c| d(a).total_cmp(&d(c)).then_with(|| a.id.cmp(&c.id)))
            .map(|f| self.fires.root(f.id))
    }

    /// Person `who` has torn covering from building `building`'s roof for `minutes` (M6b slice
    /// BD, step four; ADR-0022 §6): its covering loses that share of its area, and once all of it
    /// is gone it has nothing to catch. Nothing is restored: the household rethatches it as any
    /// lost covering.
    pub(super) fn strip_roof(
        &mut self,
        ctx: &mut Ctx,
        who: PermanentId,
        building: PermanentId,
        minutes: u32,
    ) {
        let plots = plot_boxes(ctx.land);
        let Some(bi) = ctx.land.buildings.iter().position(|b| b.id == building) else {
            return;
        };
        let b = &ctx.land.buildings[bi];
        if !b.roofed() || self.fires.burning_in(building).is_some() {
            return;
        }
        let at = build::centre_m(&b.spec);
        let Some(fire) = self.fire_against(at) else {
            return;
        };
        let area = outline_of(&plots, b).map_or(0.0, |o| o.area_m2());
        let m2 = f64::from(minutes) * ctx.params.fire.strip_m2_per_minute;
        if area <= 0.0 || m2 <= 0.0 {
            return;
        }
        for c in &mut ctx.land.buildings[bi].condition {
            if GroupKind::of_group(c.group) == Some(GroupKind::Covering) {
                c.loss = (c.loss + (m2 / area) as f32).min(1.0);
                if c.loss >= 1.0 {
                    c.state = GroupState::Failed;
                }
            }
        }
        self.fires.stripped.push(Stripped {
            fire,
            building,
            person: who,
            at: ctx.now,
            m2: m2.min(area) as f32,
        });
    }

    /// Choose a working place outside every standing roof, nearest the worker first.
    fn pull_place(&self, ctx: &Ctx, b: &Building, from: (f32, f32)) -> Option<(f32, f32)> {
        let plots = plot_boxes(ctx.land);
        let outline = outline_of(&plots, b)?;
        let others: Vec<_> = ctx
            .land
            .buildings
            .iter()
            .filter(|other| other.id != b.id && other.standing())
            .filter_map(|other| outline_of(&plots, other))
            .collect();
        let (x, y) = outline.centre();
        let margin = ctx.params.fire.beside_m.max(1.0);
        let mut points = vec![pulling_place(&outline, from, margin)];
        for (dx, dy) in [
            (1., 0.),
            (1., 1.),
            (0., 1.),
            (-1., 1.),
            (-1., 0.),
            (-1., -1.),
            (0., -1.),
            (1., -1.),
        ] {
            points.push(pulling_place(
                &outline,
                ((x + dx) as f32, (y + dy) as f32),
                margin,
            ));
        }
        points
            .into_iter()
            .filter(|&at| others.iter().all(|o| !within(o, at)))
            .min_by(|a, b| {
                let distance = |p: &(f32, f32)| (p.0 - from.0).hypot(p.1 - from.1);
                distance(a)
                    .total_cmp(&distance(b))
                    .then(a.0.total_cmp(&b.0))
                    .then(a.1.total_cmp(&b.1))
            })
    }

    /// A minute of coordinated demolition, paid by actual workers on site.
    pub(super) fn pull_down(
        &mut self,
        ctx: &mut Ctx,
        who: PermanentId,
        building: PermanentId,
        minutes: u32,
    ) {
        let Some(bi) = ctx.land.buildings.iter().position(|b| b.id == building) else {
            return;
        };
        let b = &ctx.land.buildings[bi];
        if !b.roofed() || self.fires.burning_in(building).is_some() || minutes == 0 {
            return;
        }
        let at = build::centre_m(&b.spec);
        let Some(fire) = self.fire_against(at) else {
            return;
        };
        let Some(outline) = outline_of(&plot_boxes(ctx.land), b) else {
            return;
        };
        let Some(actor) = self.person(who) else {
            return;
        };
        let leave = actor.household == b.household
            || self.people.iter().any(|(_, p)| {
                p.household == b.household
                    && !p.asleep
                    && outside(&outline, p.position_at(ctx.now.minutes() as f64))
                        <= ctx.params.fire.reach_m
            });
        if !leave
            || !self
                .fires
                .exposure
                .iter()
                .any(|&(id, heat)| id == building && heat > 0.0)
        {
            return;
        }
        if self
            .people
            .iter()
            .any(|(_, p)| within(&outline, p.position_at(ctx.now.minutes() as f64)))
        {
            return;
        }
        let now = ctx.now.minutes();
        if b.demolition.is_some_and(|d| d.last_minute >= now) {
            return;
        }
        let mut crew: Vec<PermanentId> = self
            .people
            .iter()
            .filter_map(|(_, p)| {
                let def = ctx.catalog.activities.get(usize::from(p.act.def))?;
                if def.behavior != Behavior::Pull
                    || p.act.target != Target::Building(building)
                    || !matches!(
                        p.act.steps.get(p.act.step as usize),
                        Some(Step::Work { .. })
                    )
                    || p.act.step_started.minutes() > now - i64::from(minutes)
                    || p.act.step_ends.minutes() < now
                    || p.asleep
                    || ctx.now.day_index() <= self.order.hurt_until(p.id).unwrap_or(-1)
                    || self.sickness.ill(p.id, ctx.now.day_index())
                    || outside(&outline, p.position_at(now as f64))
                        > ctx.params.fire.beside_m.max(1.0) + 0.01
                {
                    return None;
                }
                let age = p.age_years(ctx.now);
                if age < def.min_age_years || age > def.max_age_years {
                    return None;
                }
                let hh = self.household(p.household)?;
                let stores = stores_now(hh, ctx.now, ctx.params, &ctx.catalog.goods);
                def.tools
                    .iter()
                    .all(|&g| stores.get(g).is_some_and(|&kg| kg > 0.0))
                    .then_some(p.id)
            })
            .collect();
        crew.sort_unstable();
        crew.dedup();
        if !crew.contains(&who) || crew.len() < usize::from(ctx.params.fire.pull_crew) {
            return;
        }
        let d = ctx.land.buildings[bi]
            .demolition
            .get_or_insert(civ_land::buildings::Demolition {
                fire,
                needed_h: (outline.area_m2() * ctx.params.fire.pull_h_per_m2) as f32,
                work_h: 0.0,
                last_minute: -1,
            });
        if d.needed_h <= 0.0 || !d.needed_h.is_finite() {
            return;
        }
        d.last_minute = now;
        d.work_h += (crew.len() as f64 * f64::from(minutes) / 60.0) as f32;
        if d.work_h < d.needed_h {
            return;
        }
        let work_h = d.work_h;
        let household = ctx.land.buildings[bi].household;
        let b = &mut ctx.land.buildings[bi];
        for c in &mut b.condition {
            c.loss = 1.0;
            c.state = GroupState::Failed;
        }
        b.state = civ_land::BuildingState::Ruin;
        b.repair = None;
        b.demolition = None;
        let words = self.building_words(ctx, &ctx.land.buildings[bi]);
        self.chronicle_push(
            ctx.now, ChronicleKind::Fire, crew.clone(),
            self.household(household).and_then(|h| h.settlement), Some(at),
            f64::from(FireStep::Pulled as u8),
            format!("A crew of {} pulled down {words} against the fire: {work_h:.2} hours of work; its parts were lost.", crew.len()),
        );
        self.fires.pulled.push(crate::fire::Pulled {
            fire,
            building,
            at: ctx.now,
            people: crew,
            work_h,
        });
        self.buildings_changed(ctx.now, ctx.land, ctx.catalog, ctx.params, household);
    }

    /// Starts the actual one-minute activity on site for a controlled crew test.
    #[doc(hidden)]
    pub fn start_pulling_for_tests(
        &mut self,
        ctx: &mut Ctx,
        who: PermanentId,
        building: PermanentId,
    ) -> bool {
        let Some(h) = self.index.get(&who).copied() else {
            return false;
        };
        let Some((def, d)) = ctx
            .catalog
            .activities
            .iter()
            .enumerate()
            .find(|(_, d)| d.behavior == Behavior::Pull)
        else {
            return false;
        };
        let minutes = d.min_minutes.max(1);
        let Some(b) = ctx.land.buildings.iter().find(|b| b.id == building) else {
            return false;
        };
        let Some(from) = self.people.get(h).map(|p| p.pos) else {
            return false;
        };
        let Some(pos) = self.pull_place(ctx, b, from) else {
            return false;
        };
        let Some(p) = self.people.get_mut(h) else {
            return false;
        };
        p.pos = pos;
        p.trip = None;
        p.act = Activity {
            def: def as u16,
            target: Target::Building(building),
            steps: vec![Step::Work { minutes }],
            step: 0,
            started: ctx.now,
            step_started: ctx.now,
            step_ends: ctx.now,
            version: p.act.version.wrapping_add(1),
        };
        self.start_work(ctx, h, minutes, Some(Behavior::Pull));
        true
    }

    /// Person `who` of `household` takes up a load of its goods from under building
    /// `building`'s roof, those that burn and the most first, to carry out of the fire's way
    /// (M6b slice BD, step four; ADR-0022 §5). They stay the household's; its fire, if it comes,
    /// does not reach them.
    pub(super) fn salvage(
        &mut self,
        ctx: &mut Ctx,
        who: PermanentId,
        household: PermanentId,
        building: PermanentId,
    ) {
        let Some(bi) = ctx.land.buildings.iter().position(|b| b.id == building) else {
            return;
        };
        let b = ctx.land.buildings[bi].clone();
        if b.household != household {
            return;
        }
        let Some(fire) = self.fire_against(build::centre_m(&b.spec)) else {
            return;
        };
        let kept = self.kept_by(ctx, household, bi);
        let mut under: Vec<(f64, usize)> = ctx
            .catalog
            .goods
            .iter()
            .enumerate()
            .filter(|(_, d)| d.fire.is_some_and(|f| f.burns > 0.0))
            .map(|(g, _)| {
                let kg: f64 = kept.iter().map(|k| k.get(g).copied().unwrap_or(0.0)).sum();
                let out: f64 = self
                    .fires
                    .salvaged
                    .iter()
                    .filter(|s| {
                        s.building == building && s.fire == fire && usize::from(s.good) == g
                    })
                    .map(|s| f64::from(s.kg))
                    .sum();
                ((kg - out).max(0.0), g)
            })
            .filter(|&(kg, _)| kg > 0.01)
            .collect();
        under.sort_by(|a, c| c.0.total_cmp(&a.0).then_with(|| a.1.cmp(&c.1)));
        let mut room = ctx.params.household.carry_kg;
        for (kg, g) in under {
            if room <= 0.0 {
                break;
            }
            let taken = kg.min(room);
            room -= taken;
            self.fires.salvaged.push(Salvaged {
                fire,
                building,
                person: who,
                at: ctx.now,
                good: g as u16,
                kg: taken as f32,
            });
        }
    }

    /// Person `h` of `household` draws a load of water for a fire where `from` says (M6b slice
    /// BD, step three): from the household's store at home, as much of a load as it holds; at a
    /// well, what its column holds of a load; at a bank or spring, a load.
    pub(super) fn draw_for_fire(
        &mut self,
        ctx: &mut Ctx,
        h: Handle<Person>,
        who: PermanentId,
        household: PermanentId,
        from: u32,
    ) {
        let (now, params) = (ctx.now, ctx.params);
        let carry = params.household.carry_water_l;
        let got = if from == FROM_STORE {
            let Some(x) = self
                .hh_index
                .get(&household)
                .copied()
                .and_then(|i| self.households.get_mut(i))
            else {
                return;
            };
            let members = x.members.len().max(1) as f64;
            let litres_day = members * x.water_use(&params.household);
            x.settle_water(now, litres_day);
            let got = x.water_l.min(carry).max(0.0);
            x.water_l -= got;
            got
        } else if let Some(well) = ctx
            .land
            .wells
            .list
            .iter()
            .find(|w| w.cell == from)
            .map(|w| w.id)
        {
            let got = self.drew_from_well(ctx, well, household, who, now);
            self.drew_at(who, household, from);
            got
        } else {
            self.drew_water(ctx, from, household, who, now);
            self.drew_at(who, household, from);
            carry
        };
        if let Some(p) = self.people.get_mut(h) {
            p.carrying.water_l = got as f32;
        }
    }

    /// Person `h` throws the water they carry on building `building`'s roof (M6b slice BD, step
    /// three; ADR-0022 §6): the share that stays on it wets it, while a fire burns near. Each
    /// load is recorded against the fire burning then.
    pub(super) fn throw_water(
        &mut self,
        ctx: &mut Ctx,
        h: Handle<Person>,
        who: PermanentId,
        building: PermanentId,
    ) {
        let Some(p) = self.people.get_mut(h) else {
            return;
        };
        let litres = f64::from(p.carrying.water_l);
        p.carrying.water_l = 0.0;
        let from = match p.act.target {
            Target::Douse { from, .. } => from,
            _ => FROM_STORE,
        };
        if litres <= 0.0 {
            return;
        }
        // Against the fire burning nearest it: the first of its conflagration, or itself.
        let Some(b) = ctx.land.buildings.iter().find(|b| b.id == building) else {
            return;
        };
        let at = build::centre_m(&b.spec);
        let fire = self
            .fires
            .list
            .iter()
            .filter(|f| f.burning())
            .min_by(|a, c| {
                let d = |f: &Fire| f64::from(f.at.0 - at.0).hypot(f64::from(f.at.1 - at.1));
                d(a).total_cmp(&d(c)).then_with(|| a.id.cmp(&c.id))
            })
            .map(|f| self.fires.root(f.id));
        let Some(fire) = fire else {
            // Nothing burns any more: the water is spent on nothing.
            return;
        };
        let stays = litres * ctx.params.fire.water_stays.clamp(0.0, 1.0);
        if self.fires.burning_in(building).is_none() && stays > 0.0 {
            match self.fires.wet.iter_mut().find(|(x, _)| *x == building) {
                Some(w) => w.1 += stays,
                None => self.fires.wet.push((building, stays)),
            }
        }
        self.fires.thrown.push(Thrown {
            fire,
            building,
            person: who,
            at: ctx.now,
            litres: litres as f32,
            from,
        });
    }

    /// The observer sets building `building`'s covering alight (ADR-0022 §14): it burns as any
    /// fire does, and what follows is the fire's and people's. Refused for a building that is not
    /// roofed, already burns, or has no covering left to burn.
    pub fn set_fire(
        &mut self,
        ctx: &mut Ctx,
        building: PermanentId,
    ) -> Result<super::influence::Reached, String> {
        let b = ctx
            .land
            .buildings
            .iter()
            .find(|b| b.id == building)
            .cloned()
            .ok_or_else(|| format!("no building {building} stands here"))?;
        if self.fires.burning_in(building).is_some() {
            return Err("it is already burning".to_owned());
        }
        if !b.roofed() {
            return Err("it has no roof to set alight".to_owned());
        }
        let covered = b
            .condition
            .iter()
            .any(|c| GroupKind::of_group(c.group) == Some(GroupKind::Covering) && c.loss < 1.0);
        if !covered {
            return Err("its covering is gone".to_owned());
        }
        let id = self.fires.list.len() as u32 + 1;
        let influence = self
            .influences
            .add(ctx.now, InfluenceKind::Fire, building, id);
        let words = self.building_words(ctx, &b);
        let settlement = self.household(b.household).and_then(|x| x.settlement);
        let people = self
            .elder_of(b.household, ctx.now, ctx.params)
            .into_iter()
            .collect();
        self.chronicle_push(
            ctx.now,
            ChronicleKind::Influence,
            people,
            settlement,
            Some(build::centre_m(&b.spec)),
            f64::from(InfluenceKind::Fire.code()),
            format!(" {words}."),
        );
        let was = self.fires.active();
        self.kindle(ctx, &b, Kindled::Observer { influence });
        if !was {
            ctx.schedule
                .push((ctx.now.plus_minutes(1), AgentEvent::Fire));
        }
        // Whoever sees it now knows of it and shouts.
        let plots = plot_boxes(ctx.land);
        self.notice(ctx, &plots);
        Ok(super::influence::Reached {
            id: influence,
            repeat: false,
            holds: false,
            fit: 0.0,
            chance: 0.0,
        })
    }

    /// The words for building `b`: "the hut of Wren's household".
    fn building_words(&self, ctx: &Ctx, b: &Building) -> String {
        let what = ctx
            .catalog
            .building_index(&b.spec.program)
            .and_then(|d| ctx.catalog.buildings.get(d))
            .map_or_else(|| "building".to_owned(), |d| d.name.to_lowercase());
        match self.elder_of(b.household, ctx.now, ctx.params) {
            Some(e) => format!("the {what} of {}'s household", self.name_of(e)),
            None => format!("a {what}"),
        }
    }

    /// Building `b`'s covering catches now, as `kindled` says: a fire begins. The caller steps it:
    /// a minute's stepping is under way, or begun by the caller if nothing was active before.
    fn kindle(&mut self, ctx: &mut Ctx, b: &Building, kindled: Kindled) -> u32 {
        let id = self.fires.list.len() as u32 + 1;
        let settlement = self.household(b.household).and_then(|x| x.settlement);
        self.fires.list.push(Fire {
            id,
            building: b.id,
            household: b.household,
            settlement,
            at: build::centre_m(&b.spec),
            began: ctx.now,
            kindled,
            episode: None,
            ended: None,
            goods_kg: 0.0,
            deaths: Vec::new(),
        });
        self.fires.exposure.retain(|(x, _)| *x != b.id);
        id
    }

    /// One minute of every fire burning and every escape growing (ADR-0022 §3-§6): each fire
    /// burns on and does its damage, its heat reaches the buildings near it and may catch them,
    /// each escape grows, is put out or reaches its covering, those burnt out end, and people
    /// notice, shout and hear. Another minute follows while anything burns or grows.
    pub(super) fn fire_minute(&mut self, ctx: &mut Ctx) {
        let now = ctx.now;
        if !self.fires.active() {
            return;
        }
        let p = ctx.params.fire.clone();
        let burning: Vec<usize> = (0..self.fires.list.len())
            .filter(|&i| self.fires.list[i].burning())
            .collect();
        let plots = plot_boxes(ctx.land);
        // Each fire burns on, and those burnt out end.
        let mut ended = Vec::new();
        for &i in &burning {
            if self.burn_minute(ctx, i, &plots, &p) {
                ended.push(i);
            }
        }
        // Heat on the buildings near those still burning; some catch.
        if !burning.is_empty() {
            self.spread(ctx, &plots, &p);
        }
        // Escapes grow, are put out, or reach their coverings.
        self.grow(ctx, &plots, &p);
        for i in ended {
            self.burn_out(ctx, i);
        }
        // Who sees what burns now, and the shouts.
        self.notice(ctx, &plots);
        if self.fires.active() {
            ctx.schedule.push((now.plus_minutes(1), AgentEvent::Fire));
        }
        // Water on roofs runs off and dries.
        let drains = (1.0 - p.wet_drains_per_minute).clamp(0.0, 1.0);
        for w in &mut self.fires.wet {
            w.1 *= drains;
        }
        self.fires.wet.retain(|&(_, l)| l > 0.01);
        if !self.fires.any_burning() {
            self.fires.exposure.clear();
            self.fires.wet.clear();
        }
    }

    /// A minute of every escape growing inside (slice BD; 12-03 §1.2): someone awake in or
    /// beside its building puts it out, as does someone asleep inside who wakes to it by a draw
    /// of their own; left alone for its minutes, it reaches the covering, which catches if it
    /// can.
    fn grow(
        &mut self,
        ctx: &mut Ctx,
        plots: &FastMap<PermanentId, civ_land::RectCm>,
        p: &FireParams,
    ) {
        let now = ctx.now;
        let t = now.minutes() as f64;
        let growing: Vec<usize> = (0..self.fires.ignitions.len())
            .filter(|&i| self.fires.ignitions[i].growing())
            .collect();
        for gi in growing {
            let g = self.fires.ignitions[gi];
            let b = ctx
                .land
                .buildings
                .iter()
                .find(|b| b.id == g.building)
                .cloned();
            let o = b.as_ref().and_then(|b| outline_of(plots, b));
            let (Some(b), Some(o)) = (b, o) else {
                // Its building is gone: it goes out with it.
                self.fires.ignitions[gi].ended = Some(now);
                continue;
            };
            if !b.roofed() || self.fires.burning_in(b.id).is_some() {
                // Its roof is gone, or it burns already from elsewhere.
                self.fires.ignitions[gi].ended = Some(now);
                continue;
            }
            // Someone awake in or beside it finds it and puts it out.
            if let Some(who) = self.awake_beside(&o, t, p.beside_m) {
                let g = &mut self.fires.ignitions[gi];
                g.ended = Some(now);
                g.put_out_by = Some(who);
                continue;
            }
            // Someone asleep inside wakes to it in time: the first by their draw, then by id.
            let minutes = (now.minutes() - g.at.minutes()) as f64;
            let [lo, hi] = p.wake_minutes;
            let mut woke: Option<(f64, PermanentId, Handle<Person>)> = None;
            for (h, q) in self.people.iter() {
                if !q.asleep || q.trip.is_some() || !within(&o, q.position_at(t)) {
                    continue;
                }
                let key = [ctx.seed, PURPOSE_FIRE_WAKE, q.id.get(), u64::from(g.id)];
                let w = lo.max(0.0) + Rng64::from_key(&key).next_f64() * (hi - lo).max(0.0);
                if minutes >= w && woke.is_none_or(|(bw, bid, _)| (w, q.id) < (bw, bid)) {
                    woke = Some((w, q.id, h));
                }
            }
            if let Some((_, who, h)) = woke {
                self.stop_and_think(ctx, h);
                let g = &mut self.fires.ignitions[gi];
                g.ended = Some(now);
                g.put_out_by = Some(who);
                continue;
            }
            if minutes >= f64::from(g.grows_min) {
                // It reaches the covering, which catches if it can.
                self.fires.ignitions[gi].ended = Some(now);
                if self.catch_threshold(ctx, &b).is_some() {
                    let fire = self.kindle(ctx, &b, Kindled::Ignition { ignition: g.id });
                    self.fires.ignitions[gi].fire = Some(fire);
                }
            }
        }
    }

    /// Who comes to know of the fires burning now, and the shouts (ADR-0022 §6-§7). Anyone awake
    /// within sight of a burning building notices it, or is inside it; each fire they see or are
    /// in is a record of theirs. Whoever comes to know of a conflagration (or lone fire) they did
    /// not know of shouts, those who saw it at once and those who heard a shout the minute after:
    /// anyone within earshot who did not know of it hears, waking if asleep. Anyone awake who
    /// comes to know of one stops what they are doing and decides again.
    fn notice(&mut self, ctx: &mut Ctx, plots: &FastMap<PermanentId, civ_land::RectCm>) {
        let now = ctx.now;
        let t = now.minutes() as f64;
        let (sight, earshot) = (ctx.params.fire.sight_m, ctx.params.fire.earshot_m);
        // The fires burning now, with their outlines and the conflagrations they are part of.
        let burning: Vec<(u32, u32, Outline)> = self
            .fires
            .list
            .iter()
            .filter(|f| f.burning())
            .filter_map(|f| {
                let b = ctx.land.buildings.iter().find(|b| b.id == f.building)?;
                Some((f.id, self.fires.root(f.id), outline_of(plots, b)?))
            })
            .collect();
        if burning.is_empty() {
            return;
        }
        // Who knows of what already: each record since the earliest of these fires began.
        let since = burning
            .iter()
            .filter_map(|&(id, _, _)| self.fires.get(id).map(|f| f.began.minutes()))
            .min()
            .unwrap_or(now.minutes());
        let roots: Vec<u32> = burning.iter().map(|&(_, r, _)| r).collect();
        let mut knows_fire: std::collections::HashSet<(u32, PermanentId)> = Default::default();
        let mut knows_root: std::collections::HashSet<(u32, PermanentId)> = Default::default();
        let start = self
            .fires
            .knew
            .iter()
            .rposition(|k| k.at.minutes() < since)
            .map_or(0, |i| i + 1);
        for k in &self.fires.knew[start..] {
            let root = self.fires.root(k.fire);
            if roots.contains(&root) {
                // An unnamed alarm is not a sighting; approaching it can add a direct account.
                if k.how != Noticed::Heard {
                    knows_fire.insert((k.fire, k.person));
                }
                knows_root.insert((root, k.person));
            }
        }
        // Those in or within sight of a fire who did not know of it.
        let mut seen: Vec<(Handle<Person>, Knew, bool)> = Vec::new();
        for (h, q) in self.people.iter() {
            if q.asleep {
                continue;
            }
            let at = q.position_at(t);
            for &(fire, root, ref o) in &burning {
                let d = outside(o, at);
                if d > sight || knows_fire.contains(&(fire, q.id)) {
                    continue;
                }
                let how = if q.trip.is_none() && within(o, at) {
                    Noticed::Inside
                } else {
                    Noticed::Saw
                };
                let new_to_it = knows_root.insert((root, q.id));
                knows_fire.insert((fire, q.id));
                let knew = Knew {
                    fire,
                    person: q.id,
                    at: now,
                    how,
                    shouted: !new_to_it,
                };
                seen.push((h, knew, new_to_it));
            }
        }
        for (h, knew, new_to_it) in seen {
            self.fires.knew.push(knew);
            self.fire_word(knew);
            if new_to_it {
                self.stop_and_think(ctx, h);
            }
        }
        // The shouts: everyone who came to know of a fire and has not shouted of it.
        let shouts: Vec<(usize, u32, u32, (f32, f32))> = (start..self.fires.knew.len())
            .filter(|&i| !self.fires.knew[i].shouted)
            .filter_map(|i| {
                let k = self.fires.knew[i];
                let root = self.fires.root(k.fire);
                let at = self.person(k.person)?.position_at(t);
                roots.contains(&root).then_some((i, k.fire, root, at))
            })
            .collect();
        for &(i, _, _, _) in &shouts {
            self.fires.knew[i].shouted = true;
        }
        let mut heard: Vec<(Handle<Person>, Knew)> = Vec::new();
        for (_, fire, root, from) in shouts {
            for (h, q) in self.people.iter() {
                if knows_root.contains(&(root, q.id)) {
                    continue;
                }
                let at = q.position_at(t);
                let d = f64::from(at.0 - from.0).hypot(f64::from(at.1 - from.1));
                if d > earshot {
                    continue;
                }
                knows_root.insert((root, q.id));
                heard.push((
                    h,
                    Knew {
                        fire,
                        person: q.id,
                        at: now,
                        how: Noticed::Heard,
                        shouted: false,
                    },
                ));
            }
        }
        for (h, knew) in heard {
            self.fires.knew.push(knew);
            self.stop_and_think(ctx, h);
        }
    }

    /// Fire `i` burns a minute: its covering goes, and while it is fully involved its timber
    /// chars, the goods under its roof burn and sleepers inside are caught. Whether it has burnt
    /// out.
    fn burn_minute(
        &mut self,
        ctx: &mut Ctx,
        i: usize,
        plots: &FastMap<PermanentId, civ_land::RectCm>,
        p: &crate::fire::FireParams,
    ) -> bool {
        let now = ctx.now;
        let f = &self.fires.list[i];
        let (building, id) = (f.building, f.id);
        let m = f.minutes_at(now);
        let stage = p.stage_at(m);
        let was = p.stage_at(m - 1.0);
        let Some(bi) = ctx.land.buildings.iter().position(|b| b.id == building) else {
            // The building is gone: its fire ends.
            return true;
        };
        let b = ctx.land.buildings[bi].clone();
        let def = ctx
            .catalog
            .building_index(&b.spec.program)
            .and_then(|d| ctx.catalog.buildings.get(d));
        let e = def.and_then(|d| self.expansion_of(&b, d));
        // The covering burns away.
        let burnt = p.covering_burnt(m) as f32;
        // Timber chars for the minute it was fully involved.
        let involved = |t: f64| (t - p.involve_minutes).clamp(0.0, p.involved_minutes);
        let char = p.char_mm(involved(m)) - p.char_mm(involved(m - 1.0));
        let out = stage == Stage::Out;
        {
            let b = &mut ctx.land.buildings[bi];
            for c in &mut b.condition {
                match GroupKind::of_group(c.group) {
                    Some(GroupKind::Covering) => {
                        c.loss = c.loss.max(burnt);
                        if out || c.loss >= 1.0 {
                            c.loss = 1.0;
                            c.state = GroupState::Failed;
                        }
                    }
                    Some(kind) if chars(kind) && char > 0.0 => {
                        let size_mm = e
                            .as_ref()
                            .and_then(|e| e.groups.iter().find(|g| g.id == c.group))
                            .map_or(0.0, |g| {
                                f64::from(g.section_cm[0].min(g.section_cm[1])) * 10.0
                            });
                        c.loss = charred_loss(c.loss, size_mm, char);
                    }
                    Some(GroupKind::Infill) if out => {
                        c.loss = (c.loss + p.infill_loss as f32).min(1.0);
                    }
                    _ => {}
                }
            }
        }
        // Fully involved: the goods under the roof burn, and sleepers inside are caught.
        let caught = stage == Stage::Involved && was == Stage::Covering;
        if caught {
            let root = self.fires.root(id);
            let lost = self.burn_goods(ctx, bi, root);
            self.fires.list[i].goods_kg += lost as f32;
        }
        let o = outline_of(plots, &b);
        self.occupants(ctx, o.as_ref(), id, i, caught);
        out
    }

    /// The goods household of building `bi` keeps under its roof burn, each good its share
    /// (ADR-0022 §5): lost from its stores. Goods carried out from under it against the fires
    /// begun with `root` do not (slice BD, step four). Kilograms lost.
    fn burn_goods(&mut self, ctx: &mut Ctx, bi: usize, root: u32) -> f64 {
        let (household, building) = (ctx.land.buildings[bi].household, ctx.land.buildings[bi].id);
        let Some(hd) = self.hh_index.get(&household).copied() else {
            return 0.0;
        };
        let mut kept = self.kept_by(ctx, household, bi);
        for s in self
            .fires
            .salvaged
            .iter()
            .filter(|s| s.fire == root && s.building == building)
        {
            let mut out = f64::from(s.kg);
            for k in &mut kept {
                if let Some(v) = k.get_mut(usize::from(s.good)) {
                    let taken = out.min(*v);
                    *v -= taken;
                    out -= taken;
                }
            }
        }
        let goods = &ctx.catalog.goods;
        let Some(x) = self.households.get_mut(hd) else {
            return 0.0;
        };
        let mut lost_kg = 0.0;
        for (g, def) in goods.iter().enumerate() {
            let Some(fire) = def.fire else {
                continue;
            };
            let under: f64 = kept.iter().map(|k| k.get(g).copied().unwrap_or(0.0)).sum();
            let held = x.stores.get(g).copied().unwrap_or(0.0).max(0.0);
            let lost = (under * fire.burns.clamp(0.0, 1.0)).min(held);
            if lost > 0.0 {
                x.stores[g] -= lost;
                x.flows.add(Flow::Lost, g, lost);
                lost_kg += lost;
            }
        }
        lost_kg
    }

    /// Those in a burning building with outline `o` (fire `id`, at index `i`): the awake get out
    /// and do nothing at home until it is out; when it is fully involved around them (`caught`),
    /// those still asleep are woken by it, and a share of them do not get out.
    fn occupants(&mut self, ctx: &mut Ctx, o: Option<&Outline>, id: u32, i: usize, caught: bool) {
        let Some(o) = o else {
            return;
        };
        let t = ctx.now.minutes() as f64;
        let inside: Vec<(Handle<Person>, PermanentId, bool)> = self
            .people
            .iter()
            .filter(|(_, p)| p.trip.is_none() && within(o, p.position_at(t)))
            .map(|(h, p)| (h, p.id, p.asleep))
            .collect();
        let mut dead = Vec::new();
        for (h, who, asleep) in inside {
            if asleep && !caught {
                continue;
            }
            if asleep {
                let key = [ctx.seed, PURPOSE_FIRE_CAUGHT, who.get(), u64::from(id)];
                if Rng64::from_key(&key).next_f64() < CAUGHT_ASLEEP_DIES {
                    dead.push(who);
                    continue;
                }
                // Woken by it, they know of it, and shout if it is news to them.
                let (fire, root) = self.knows_of(id, who);
                if !fire {
                    let knew = Knew {
                        fire: id,
                        person: who,
                        at: ctx.now,
                        how: Noticed::Woke,
                        shouted: root,
                    };
                    self.fires.knew.push(knew);
                    self.fire_word(knew);
                }
            }
            // Out, and deciding again; their home is not where they will go.
            let waiting = self.people.get(h).is_some_and(|p| {
                p.act.target == Target::None
                    && matches!(p.act.steps.first(), Some(Step::Wait { .. }))
            });
            if !waiting || asleep {
                self.wait(ctx, h, 1);
            }
        }
        for who in dead {
            self.fires.list[i].deaths.push(who);
            self.die(ctx, who, Cause::Burned);
        }
    }

    /// The heat of every fire burning on the buildings near it, a minute's worth: each weighed
    /// building's exposure moves toward it, and its covering may catch (ADR-0022 §4).
    fn spread(
        &mut self,
        ctx: &mut Ctx,
        plots: &FastMap<PermanentId, civ_land::RectCm>,
        p: &crate::fire::FireParams,
    ) {
        let now = ctx.now;
        let today = ctx.land.weather.today;
        let (wind, wind_ms, moisture) = (
            today.wind_toward(),
            f64::from(today.wind_ms),
            today.fuel_moisture,
        );
        // Fires burning now, with their outlines and stages.
        let sources: Vec<(u32, Outline, Stage)> = self
            .fires
            .list
            .iter()
            .filter(|f| f.burning())
            .filter_map(|f| {
                let b = ctx.land.buildings.iter().find(|b| b.id == f.building)?;
                let stage = p.stage_at(f.minutes_at(now));
                (stage != Stage::Out).then_some((f.id, outline_of(plots, b)?, stage))
            })
            .collect();
        if sources.is_empty() {
            return;
        }
        let burning: Vec<PermanentId> = self
            .fires
            .list
            .iter()
            .filter(|f| f.burning())
            .map(|f| f.building)
            .collect();
        // Every other roofed building within reach of one of them, in the land's order, with the
        // fire whose heat on it is greatest.
        type Most = (u32, f64, f64, f64);
        let mut near: Vec<(usize, f64, Most)> = Vec::new();
        for (bi, b) in ctx.land.buildings.iter().enumerate() {
            if burning.contains(&b.id) || !b.roofed() {
                continue;
            }
            let Some(ob) = outline_of(plots, b) else {
                continue;
            };
            let mut heat = 0.0;
            let mut most: Option<Most> = None;
            for (fid, os, stage) in &sources {
                let gap = os.gap(&ob);
                if gap > p.reach_m {
                    continue;
                }
                let toward = os.toward(&ob, wind);
                let q = p.heat_on(*stage, gap, wind_ms, toward);
                heat += q;
                if most.is_none_or(|(_, mq, _, _)| q > mq) {
                    most = Some((*fid, q, gap, toward));
                }
            }
            if let Some(most) = most {
                near.push((bi, heat, most));
            }
        }
        for (bi, heat, (source, _, gap, toward)) in near {
            let b = ctx.land.buildings[bi].clone();
            let before = self
                .fires
                .exposure
                .iter()
                .find(|(x, _)| *x == b.id)
                .map_or(0.0, |&(_, s)| s);
            // Water standing on its roof takes the heat first (M6b slice BD, step three).
            let area = outline_of(plots, &b).map_or(0.0, |o| o.area_m2());
            let heat = self.wet_takes(b.id, heat, area, p);
            let store = p.expose(before, heat, 1.0);
            match self.fires.exposure.iter_mut().find(|(x, _)| *x == b.id) {
                Some(e) => e.1 = store,
                None => self.fires.exposure.push((b.id, store)),
            }
            let threshold = self.catch_threshold(ctx, &b);
            let root = self.fires.root(source);
            let caught = threshold.is_some_and(|th| {
                let key = [
                    ctx.seed,
                    PURPOSE_FIRE_CATCH,
                    b.id.get(),
                    now.minutes() as u64,
                ];
                Rng64::from_key(&key).next_f64() < p.catch_chance(store, th, 1.0)
            });
            self.weigh_spread(root, b.id, store, threshold, caught);
            if caught {
                let fire = self.kindle(
                    ctx,
                    &b,
                    Kindled::Neighbour {
                        fire: source,
                        gap_m: gap as f32,
                        wind_ms: wind_ms as f32,
                        toward: toward as f32,
                        moisture,
                    },
                );
                self.join(ctx, source, fire);
            }
        }
        // Exposure that has cooled to nothing is let go.
        self.fires.exposure.retain(|&(_, s)| s > 0.01);
    }

    /// The heat `heat` (kW/m²) on building `building`'s roof of `area_m2` for a minute, less what
    /// the water standing on it boils off: a wet covering cannot pass the boiling point until it
    /// is dry, so only the heat its water could not take reaches its exposure.
    fn wet_takes(&mut self, building: PermanentId, heat: f64, area_m2: f64, p: &FireParams) -> f64 {
        let Some(w) = self.fires.wet.iter_mut().find(|(x, _)| *x == building) else {
            return heat;
        };
        let needed = p.boiled_l(heat, area_m2, 1.0);
        if needed <= 0.0 {
            return heat;
        }
        let used = needed.min(w.1.max(0.0));
        w.1 -= used;
        heat * (1.0 - used / needed)
    }

    /// The spread weighed building `building` for the fires begun with `root`: the most its
    /// exposure stood, and whether it caught.
    fn weigh_spread(
        &mut self,
        root: u32,
        building: PermanentId,
        store: f64,
        threshold: Option<f64>,
        caught: bool,
    ) {
        let th = threshold.map_or(f32::INFINITY, |t| t as f32);
        match self
            .fires
            .weighed
            .iter_mut()
            .find(|w| w.fire == root && w.building == building)
        {
            Some(w) => {
                if store as f32 > w.peak_kw {
                    w.peak_kw = store as f32;
                    w.threshold_kw = th;
                }
                w.caught |= caught;
            }
            None => self.fires.weighed.push(Weighed {
                fire: root,
                building,
                peak_kw: store as f32,
                threshold_kw: th,
                caught,
            }),
        }
    }

    /// Fire `fire` was caught from fire `source`: both are part of one conflagration, begun now
    /// if `source` was burning alone (ADR-0022 §13).
    fn join(&mut self, ctx: &mut Ctx, source: u32, fire: u32) {
        let episode = match self.fires.get(source).and_then(|f| f.episode) {
            Some(e) => e,
            None => {
                let Some(s) = self.fires.get(source).cloned() else {
                    return;
                };
                let e = self.fires.episodes.len() as u32 + 1;
                self.fires.episodes.push(Conflagration {
                    id: e,
                    settlement: s.settlement,
                    first: source,
                    began: s.began,
                    ended: None,
                });
                if let Some(f) = self.fires.list.get_mut(source as usize - 1) {
                    f.episode = Some(e);
                }
                let words = ctx
                    .land
                    .buildings
                    .iter()
                    .find(|b| b.id == s.building)
                    .cloned()
                    .map_or_else(|| "a building".to_owned(), |b| self.building_words(ctx, &b));
                let people = self
                    .elder_of(s.household, ctx.now, ctx.params)
                    .into_iter()
                    .collect();
                self.chronicle_push(
                    ctx.now,
                    ChronicleKind::Fire,
                    people,
                    s.settlement,
                    Some(s.at),
                    f64::from(FireStep::Spread as u8),
                    format!("Fire spread from {words} to the buildings beside it."),
                );
                e
            }
        };
        if let Some(f) = self.fires.list.get_mut(fire as usize - 1) {
            f.episode = Some(episode);
        }
    }

    /// The work putting building `b` right after a fire or a failure would take, hours, as
    /// anyone could see it: building it again if it fell, else mending every part it lost (M6b
    /// slice BH).
    pub(super) fn fire_cost_h(ctx: &Ctx, b: &Building) -> f64 {
        let Some(def) = ctx
            .catalog
            .building_index(&b.spec.program)
            .and_then(|i| ctx.catalog.buildings.get(i))
        else {
            return 0.0;
        };
        if b.state == civ_land::BuildingState::Ruin {
            return crate::build::building_hours(&b.spec, def, ctx.params.household.carry_kg)
                .unwrap_or(0.0);
        }
        let Ok(e) = civ_grammar::expand(&b.spec, &def.rules) else {
            return 0.0;
        };
        b.condition
            .iter()
            .filter(|g| g.loss > 0.0)
            .filter_map(|g| crate::condition::mend_needs(&e, g.group, f64::from(g.loss)))
            .map(|n| n.labour_h)
            .sum()
    }

    /// Fire `i` has burnt out: the household whose building it was, and the household of everyone
    /// who was in it or saw it burn, remember it (M6b slice BH; ADR-0022 §12: attribution comes
    /// from what people saw). Those who only heard the shout remember nothing of it.
    fn households_remember_fire(&mut self, ctx: &Ctx, i: usize) {
        let (fire, building, owner) = {
            let f = &self.fires.list[i];
            (f.id, f.building, f.household)
        };
        let Some(b) = ctx.land.buildings.iter().find(|b| b.id == building) else {
            return;
        };
        let hours = Self::fire_cost_h(ctx, b) as f32;
        let burnable = !crate::rules::roof_unburnt(&b.spec);
        let mut saw: Vec<PermanentId> = self
            .fires
            .knew
            .iter()
            .filter(|k| k.fire == fire && k.how != Noticed::Heard)
            .filter_map(|k| self.person(k.person).map(|p| p.household))
            .collect();
        saw.push(owner);
        saw.sort_unstable();
        saw.dedup();
        let day = ctx.now.day_index();
        for h in saw {
            if let Some(&hd) = self.hh_index.get(&h)
                && let Some(x) = self.households.get_mut(hd)
            {
                x.remember_fire_loss(crate::person::FireLoss {
                    day,
                    building,
                    own: h == owner,
                    burnable,
                    hours,
                });
            }
        }
    }

    /// Fire `i` has burnt out: it ends, its household's buildings are weighed against what they
    /// carry now, and the chronicle tells of it, or of its conflagration when that was its last
    /// fire.
    fn burn_out(&mut self, ctx: &mut Ctx, i: usize) {
        let now = ctx.now;
        let (building, household, episode, settlement) = {
            let f = &mut self.fires.list[i];
            f.ended = Some(now);
            (f.building, f.household, f.episode, f.settlement)
        };
        // The settlement remembers a building lost to fire by covering and wall kind (ADR-0022 §7).
        if let Some(b) = ctx.land.buildings.iter().find(|b| b.id == building) {
            let tile = b
                .spec
                .materials
                .get(usize::from(civ_grammar::frame::frame_materials::COVERING))
                .is_some_and(|m| m.contains("tile"));
            let wall_kind = b.spec.params[civ_grammar::frame::frame_params::WALL_KIND] as u8;
            self.remember_fire_loss(settlement, tile, wall_kind, now, &ctx.params.build.caution);
        }
        // Its household and everyone who saw it burn remember it (M6b slice BH).
        self.households_remember_fire(ctx, i);
        // What the fire left is weighed now: charred posts and beams may give way.
        if ctx.land.buildings.iter().any(|b| b.id == building) {
            self.buildings_changed(now, ctx.land, ctx.catalog, ctx.params, household);
            self.check_household(ctx, household, 0.0);
        }
        let f = self.fires.list[i].clone();
        let words_of = |me: &Self, ctx: &Ctx, building: PermanentId| -> String {
            ctx.land
                .buildings
                .iter()
                .find(|b| b.id == building)
                .map_or_else(|| "a building".to_owned(), |b| me.building_words(ctx, b))
        };
        match episode {
            None => {
                let mut s = format!("Fire burned out {}", words_of(self, ctx, building));
                s.push_str(&losses(&f.deaths, f.goods_kg, &|id| self.name_of(id)));
                s.push('.');
                let people = self
                    .elder_of(household, now, ctx.params)
                    .into_iter()
                    .collect();
                self.chronicle_push(
                    now,
                    ChronicleKind::Fire,
                    people,
                    f.settlement,
                    Some(f.at),
                    f64::from(FireStep::Burned as u8),
                    s,
                );
            }
            Some(e) => {
                let still = self
                    .fires
                    .list
                    .iter()
                    .any(|x| x.episode == Some(e) && x.burning());
                if still {
                    return;
                }
                let Some(ep) = self.fires.episodes.get_mut(e as usize - 1) else {
                    return;
                };
                ep.ended = Some(now);
                let ep = ep.clone();
                let fires: Vec<Fire> = self
                    .fires
                    .list
                    .iter()
                    .filter(|x| x.episode == Some(e))
                    .cloned()
                    .collect();
                let deaths: Vec<PermanentId> = fires
                    .iter()
                    .flat_map(|x| x.deaths.iter().copied())
                    .collect();
                let goods: f32 = fires.iter().map(|x| x.goods_kg).sum();
                let first = self.fires.get(ep.first).cloned();
                let at = first.as_ref().map_or_else(
                    || "a building".to_owned(),
                    |f| words_of(self, ctx, f.building),
                );
                let mut s = format!(
                    "The fire that began at {at} is out: it burned {} buildings",
                    fires.len()
                );
                s.push_str(&losses(&deaths, goods, &|id| self.name_of(id)));
                s.push('.');
                let people = first
                    .as_ref()
                    .and_then(|f| self.elder_of(f.household, now, ctx.params))
                    .into_iter()
                    .collect();
                self.chronicle_push(
                    now,
                    ChronicleKind::Fire,
                    people,
                    ep.settlement,
                    first.map(|f| f.at),
                    f64::from(FireStep::Ended as u8),
                    s,
                );
            }
        }
    }
}

/// The losses of a fire in words: "; Ada and Bram died; 120 kg of goods burned".
fn losses(deaths: &[PermanentId], goods_kg: f32, name: &dyn Fn(PermanentId) -> String) -> String {
    let mut s = String::new();
    if !deaths.is_empty() {
        let names: Vec<String> = deaths.iter().map(|&d| name(d)).collect();
        let list = match names.len() {
            1 => names[0].clone(),
            n => format!("{} and {}", names[..n - 1].join(", "), names[n - 1]),
        };
        s.push_str(&format!("; {list} died"));
    }
    if goods_kg >= 1.0 {
        s.push_str(&format!("; {goods_kg:.0} kg of goods burned"));
    }
    s
}
