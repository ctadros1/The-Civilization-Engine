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
//! keeps away from home only once they know it burns. Nobody fights a fire yet: those awake in a
//! burning building get out; those still asleep wake when it is fully involved around them, and
//! a share of them do not get out.

use super::*;
use crate::fire::{
    Conflagration, Fire, FireParams, Hearth, Ignition, Kindled, Knew, Noticed, Outline, Stage,
    Weighed, charred_loss,
};
use crate::history::{Cause, FireStep};
use crate::influence::InfluenceKind;
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
        if !self.fires.any_burning() {
            self.fires.exposure.clear();
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
                knows_fire.insert((k.fire, k.person));
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
            let lost = self.burn_goods(ctx, bi);
            self.fires.list[i].goods_kg += lost as f32;
        }
        let o = outline_of(plots, &b);
        self.occupants(ctx, o.as_ref(), id, i, caught);
        out
    }

    /// The goods household of building `bi` keeps under its roof burn, each good its share
    /// (ADR-0022 §5): lost from its stores. Kilograms lost.
    fn burn_goods(&mut self, ctx: &mut Ctx, bi: usize) -> f64 {
        let household = ctx.land.buildings[bi].household;
        let Some(hd) = self.hh_index.get(&household).copied() else {
            return 0.0;
        };
        let kept = self.kept_by(ctx, household, bi);
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
                    self.fires.knew.push(Knew {
                        fire: id,
                        person: who,
                        at: ctx.now,
                        how: Noticed::Woke,
                        shouted: root,
                    });
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

    /// Fire `i` has burnt out: it ends, its household's buildings are weighed against what they
    /// carry now, and the chronicle tells of it, or of its conflagration when that was its last
    /// fire.
    fn burn_out(&mut self, ctx: &mut Ctx, i: usize) {
        let now = ctx.now;
        let (building, household, episode) = {
            let f = &mut self.fires.list[i];
            f.ended = Some(now);
            (f.building, f.household, f.episode)
        };
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
