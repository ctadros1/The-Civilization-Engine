//! Crossings in the day and underfoot (M5c slice AW; research 11-07 §1.3, §2.4, §6.2; the
//! settlements brief §1.7). Each midnight rot takes its share of every open crossing's members,
//! and one that can no longer carry its own weight gives way with nobody on it. Whoever sets off
//! on a walk over an open crossing steps onto it: one that cannot carry them gives way under them
//! (11-07 §6.2: loads are checked as agents step on). A crossing that gives way drops whoever is
//! on it into the water, who may die; everyone whose walk would have crossed it stops where they
//! are and decides again, as no route may cross a fallen bridge (01-08 §7).

use super::*;
use crate::bridge::{crossing_margin, labour_h, life_years, size_members};
use crate::decide::CrossingFacts;
use crate::history::{Cause, CrossingStep};
use crate::uses::per_year;
use civ_land::crossings::{Collapse, Crossing, CrossingOwner, CrossingState};
use civ_land::paths::cells_along_by;

/// Keyed-randomness purpose of who dies when a crossing gives way under them.
const PURPOSE_FALL: u64 = 0x6272_6964_6765_6661; // "bridgefa"
/// Keyed-randomness purpose of the day of the year a household weighs its crossings.
const PURPOSE_CROSSING_REVIEW: u64 = 0x6272_6964_6765_7276; // "bridgerv"
/// Wades a household holds of a cell, faded, below which it lets go of them.
const LEAST_WADES: f32 = 0.5;

/// Where a crossing could be laid over a river cell: the land on either side, east and west or
/// north and south, and the channel's width there.
#[derive(Clone, Copy)]
struct Site {
    cell: u32,
    banks: [u32; 2],
    width_m: f64,
}

/// What a household would build at a site, and what it is worth to it.
struct Proposal {
    site: Site,
    system: usize,
    diameter_cm: f64,
    labour_h: f64,
    /// Hours of walking it would save the household over its life, less the work it takes.
    net_h: f64,
    /// A crossing begun there by a household that is no more, taken over.
    taken: Option<usize>,
}

/// Which of the two cells `a` and `b` at a corner a walk turning through it went by: the better
/// ground, as the walking grid takes the step (`a` when they are alike).
pub(super) fn better_corner(nav: &civ_world::nav::NavGrid, a: u32, b: u32) -> u32 {
    if nav.ground(b as usize) > nav.ground(a as usize) {
        b
    } else {
        a
    }
}

/// The site at river cell `cell`, if a crossing could stand there.
fn site_at(ctx: &Ctx, cell: u32) -> Option<Site> {
    let map = ctx.map;
    if map.water.get(cell as usize) != Some(&civ_world::WATER_RIVER) {
        return None;
    }
    let w = map.width;
    let (x, y) = (cell % w, cell / w);
    if x == 0 || y == 0 || x + 1 >= w || y + 1 >= map.height {
        return None;
    }
    let land = |c: u32| {
        map.water.get(c as usize) == Some(&civ_world::WATER_LAND) && ctx.nav.walkable(c as usize)
    };
    let banks = [[cell - 1, cell + 1], [cell - w, cell + w]]
        .into_iter()
        .find(|b| b.iter().all(|&c| land(c)))?;
    let width_m = map
        .reaches
        .iter()
        .find(|r| r.cells.contains(&cell))?
        .width_m;
    Some(Site {
        cell,
        banks,
        width_m: f64::from(width_m),
    })
}

/// Seconds a wade of `cell`, stepping in from `bank`, takes over walking it on a deck walked at
/// `deck_factor` of dry ground's speed: the walking grid's own step, of which only the ground
/// underfoot changes.
fn wade_saving_s(ctx: &Ctx, bank: u32, cell: u32, deck_factor: f64) -> f64 {
    let wading = ctx.nav.params().wading_factor;
    let trail = ctx.land.wear.factor(cell as usize);
    ctx.nav
        .step_seconds(&ctx.map.elevation, bank as usize, cell as usize, 1.0, trail)
        .map_or(0.0, |s| {
            f64::from(s) * (1.0 - wading / deck_factor.max(wading))
        })
}

impl Population {
    /// The crossings' midnight: rot takes its daily share of each open crossing's members, and
    /// one that can no longer carry its own weight gives way.
    pub(super) fn crossings_day(&mut self, ctx: &mut Ctx) {
        for i in 0..ctx.land.crossings.list.len() {
            let c = &mut ctx.land.crossings.list[i];
            if !c.open() {
                continue;
            }
            let Some(def) = ctx.catalog.bridges.get(usize::from(c.system)) else {
                continue;
            };
            c.loss = (c.loss + (def.loss_per_year / 365.0) as f32).min(1.0);
            let Some(timber) = ctx.catalog.goods.get(def.good).and_then(|g| g.timber) else {
                continue;
            };
            if crossing_margin(c, def, &timber, 0) < 1.0 {
                self.crossing_gives_way(ctx, i, Collapse::OwnWeight, None);
            }
        }
    }

    /// `who` sets off along the cells `along`: each open crossing on the way is stepped onto.
    /// One that cannot carry them gives way under them, and they do not go (`false`).
    pub(super) fn step_on(&mut self, ctx: &mut Ctx, who: PermanentId, along: &[u32]) -> bool {
        let mut crossed: Vec<usize> = along
            .iter()
            .filter_map(|&c| ctx.land.crossings.open_at(c))
            .collect();
        crossed.sort_unstable();
        crossed.dedup();
        for i in crossed {
            let c = &ctx.land.crossings.list[i];
            let Some(def) = ctx.catalog.bridges.get(usize::from(c.system)) else {
                continue;
            };
            let Some(timber) = ctx.catalog.goods.get(def.good).and_then(|g| g.timber) else {
                continue;
            };
            if crossing_margin(c, def, &timber, 1) < 1.0 {
                self.crossing_gives_way(ctx, i, Collapse::UnderWalker, Some(who));
                return false;
            }
        }
        true
    }

    /// Households whose day of the year it is weigh building a crossing where their people wade
    /// (M5c slice AW, step two); before that, what households hold of streams long unwaded, and
    /// what households no more held, is let go of.
    pub(super) fn crossing_reviews(&mut self, ctx: &mut Ctx, day: i64) {
        let half = ctx.params.places.use_half_life_days;
        let hh = &self.hh_index;
        self.fords
            .forget(day, half, LEAST_WADES, |h| hh.contains_key(&h));
        if ctx.catalog.bridges.is_empty() {
            return;
        }
        let due: Vec<PermanentId> = self
            .fords
            .households
            .keys()
            .copied()
            .filter(|h| {
                Rng64::from_key(&[ctx.seed, PURPOSE_CROSSING_REVIEW, h.get()]).next_u64()
                    % DAYS_PER_YEAR as u64
                    == day.rem_euclid(DAYS_PER_YEAR) as u64
            })
            .collect();
        for h in due {
            self.review_crossing(ctx, h, day);
        }
    }

    /// Household `household` weighs a crossing now, whatever its day of the year: for tests.
    #[doc(hidden)]
    pub fn review_crossing_for_tests(&mut self, ctx: &mut Ctx, household: PermanentId) {
        let day = ctx.now.day_index();
        self.review_crossing(ctx, household, day);
    }

    /// The travel field from household `x`'s settlement's hearth (from its home, for one without
    /// a hearth), as its people's decisions keep it, and its key.
    fn reach_of(&self, ctx: &Ctx, x: &Household) -> Option<(&TravelField, PermanentId)> {
        let hearth = x.settlement.and_then(|s| {
            ctx.land
                .settlements
                .iter()
                .find(|t| t.id == s)
                .map(|t| t.hearth_m)
        });
        let key = match (x.settlement, hearth) {
            (Some(s), Some(_)) => s,
            _ => x.id,
        };
        self.homes.get(&key).map(|f| (&f.reach, key))
    }

    /// How skilled at building crossings of system `def` household `x`'s people old enough to
    /// do the work are, on average: the skill it practises, else a middling builder's.
    fn crossing_skill(&self, ctx: &Ctx, x: &Household, def: &crate::params::BridgeDef) -> f64 {
        let Some(k) = def.skill else {
            return crate::condition::MIDDLING_SKILL;
        };
        let min_age = ctx
            .catalog
            .activities
            .iter()
            .find(|a| a.behavior == Behavior::Bridge)
            .map_or(0.0, |a| a.min_age_years);
        let levels: Vec<f64> = x
            .members
            .iter()
            .filter_map(|&m| self.person(m))
            .filter(|p| p.age_years(ctx.now) >= min_age)
            .map(|p| p.skill(k))
            .collect();
        if levels.is_empty() {
            crate::condition::MIDDLING_SKILL
        } else {
            levels.iter().sum::<f64>() / levels.len() as f64
        }
    }

    /// Household `household` weighs a crossing at each river cell its people wade (M5c slice
    /// AW, step two; the settlements brief §1.7; research 11-07 §1.1): what one would save is the
    /// time each wade there takes over walking a deck, times the wades it holds a year, over the
    /// years the crossing would last at the quality its people would lay it; what it would cost
    /// is the work it takes. Of the systems its people know the technique of that span the
    /// channel there, it begins the crossing whose saving most exceeds its work, if any does:
    /// one at a time, and none where a crossing stands or is being built, except one begun by a
    /// household that is no more, which it takes over with the work done on it.
    fn review_crossing(&mut self, ctx: &mut Ctx, household: PermanentId, day: i64) {
        let Some(x) = self.household(household) else {
            return;
        };
        if x.members.is_empty() {
            return;
        }
        let owner = CrossingOwner::Household(household);
        let building = |c: &Crossing| matches!(c.state, CrossingState::Building { .. });
        if ctx
            .land
            .crossings
            .list
            .iter()
            .any(|c| c.owner == owner && building(c))
        {
            return;
        }
        let Some(act) = ctx
            .catalog
            .activities
            .iter()
            .find(|a| a.behavior == Behavior::Bridge)
        else {
            return;
        };
        let Some((reach, _)) = self.reach_of(ctx, x) else {
            return;
        };
        let half = ctx.params.places.use_half_life_days;
        let spread = ctx.params.build.quality_spread;
        let knows = |t: Option<usize>| {
            t.is_none_or(|t| {
                x.members
                    .iter()
                    .filter_map(|&m| self.person(m))
                    .any(|p| p.knows(t))
            })
        };
        let gone = |c: &Crossing| match c.owner {
            CrossingOwner::Household(h) => self.household(h).is_none_or(|y| y.members.is_empty()),
            CrossingOwner::Polity(_) => false,
        };
        let mut best: Option<Proposal> = None;
        for w in self.fords.held(household, day, half) {
            let wades = per_year(f64::from(w.wades), half);
            let Some(site) = site_at(ctx, w.cell) else {
                continue;
            };
            // The work is done from the bank nearer the hearth, within the activity's walk.
            let Some(near) = site
                .banks
                .iter()
                .filter_map(|&b| reach.seconds_to(b as usize))
                .reduce(f32::min)
            else {
                continue;
            };
            if f64::from(near) / 60.0 > f64::from(act.max_walk_minutes) {
                continue;
            }
            let standing = ctx.land.crossings.list.iter().position(|c| {
                !matches!(c.state, CrossingState::Failed { .. }) && c.cells.contains(&w.cell)
            });
            let taken = match standing {
                Some(i)
                    if building(&ctx.land.crossings.list[i])
                        && gone(&ctx.land.crossings.list[i]) =>
                {
                    Some(i)
                }
                Some(_) => continue,
                None => None,
            };
            for (system, def) in ctx.catalog.bridges.iter().enumerate() {
                if taken.is_some_and(|i| usize::from(ctx.land.crossings.list[i].system) != system)
                    || !knows(def.technique)
                {
                    continue;
                }
                let Some(timber) = ctx.catalog.goods.get(def.good).and_then(|g| g.timber) else {
                    continue;
                };
                let (diameter_cm, labour, span) = match taken {
                    Some(i) => {
                        let c = &ctx.land.crossings.list[i];
                        let done = match c.state {
                            CrossingState::Building { work_h } => f64::from(work_h),
                            _ => 0.0,
                        };
                        (
                            f64::from(c.diameter_cm),
                            (f64::from(c.labour_h) - done).max(0.0),
                            f64::from(c.span_m),
                        )
                    }
                    None => {
                        let Some(d) = size_members(def, &timber, site.width_m) else {
                            continue;
                        };
                        (d, labour_h(def, site.width_m), site.width_m)
                    }
                };
                let quality =
                    crate::condition::expected_quality(self.crossing_skill(ctx, x, def), spread);
                let life = life_years(def, &timber, span, diameter_cm, quality);
                let saved_h = wades * wade_saving_s(ctx, site.banks[0], site.cell, def.deck_factor)
                    / 3600.0
                    * life;
                let net_h = saved_h - labour;
                if net_h > 0.0 && best.as_ref().is_none_or(|b| net_h > b.net_h) {
                    best = Some(Proposal {
                        site,
                        system,
                        diameter_cm,
                        labour_h: labour,
                        net_h,
                        taken,
                    });
                }
            }
        }
        let Some(p) = best else {
            return;
        };
        match p.taken {
            Some(i) => ctx.land.crossings.list[i].owner = owner,
            None => {
                let def = &ctx.catalog.bridges[p.system];
                let id = ctx.ids.allocate();
                ctx.land.crossings.list.push(Crossing {
                    id,
                    system: p.system as u16,
                    banks: p.site.banks,
                    cells: vec![p.site.cell],
                    span_m: p.site.width_m as f32,
                    members: def.members.min(u32::from(u8::MAX)) as u8,
                    diameter_cm: p.diameter_cm as f32,
                    quality: 0.0,
                    loss: 0.0,
                    owner,
                    labour_h: p.labour_h as f32,
                    skill_h: 0.0,
                    begun: ctx.now,
                    state: CrossingState::Building { work_h: 0.0 },
                });
            }
        }
    }

    /// The crossing household `x` is building, if `person`'s decision may work on it: where on
    /// its nearer bank the work is done, the walk there, the work left, and the walking each hour
    /// of it saves the household (M5c slice AW, step two).
    pub(super) fn crossing_facts(
        &self,
        ctx: &Ctx,
        x: &Household,
        field_key: PermanentId,
        dark: bool,
    ) -> Option<CrossingFacts> {
        if dark || ctx.land.crossings.list.is_empty() {
            return None;
        }
        let owner = CrossingOwner::Household(x.id);
        let c = ctx
            .land
            .crossings
            .list
            .iter()
            .find(|c| c.owner == owner && matches!(c.state, CrossingState::Building { .. }))?;
        let CrossingState::Building { work_h } = c.state else {
            return None;
        };
        let def = ctx.catalog.bridges.get(usize::from(c.system))?;
        let timber = ctx.catalog.goods.get(def.good).and_then(|g| g.timber)?;
        let reach = &self.homes.get(&field_key)?.reach;
        let (secs, bank) = c
            .banks
            .iter()
            .filter_map(|&b| reach.seconds_to(b as usize).map(|s| (s, b)))
            .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)))?;
        let half = ctx.params.places.use_half_life_days;
        let cell = *c.cells.first()?;
        let wades = self
            .fords
            .held(x.id, ctx.now.day_index(), half)
            .find(|w| w.cell == cell)
            .map_or(0.0, |w| per_year(f64::from(w.wades), half));
        let quality = crate::condition::expected_quality(
            self.crossing_skill(ctx, x, def),
            ctx.params.build.quality_spread,
        );
        let life = life_years(
            def,
            &timber,
            f64::from(c.span_m),
            f64::from(c.diameter_cm),
            quality,
        );
        let saved_h = wades * wade_saving_s(ctx, c.banks[0], cell, def.deck_factor) / 3600.0 * life;
        Some(CrossingFacts {
            crossing: c.id,
            at: cell_centre(ctx.map, bank as usize),
            walk_min: f64::from(secs) / 60.0,
            left_h: (f64::from(c.labour_h) - f64::from(work_h)).max(0.0),
            saves_per_hour: saved_h / f64::from(c.labour_h).max(1.0),
        })
    }

    /// `hours` of a capable adult's work by `who` (handle and id) of `household` on crossing
    /// `crossing`: it counts toward the crossing's labour and, weighted by their building skill,
    /// toward the quality of its members, and they practise the skill. Once its labour is done
    /// its members' quality is drawn from how skilled its builders were on average (ADR-0009
    /// §6), it opens to walkers, and the chronicle says so (M5c slice AW, step two).
    pub(super) fn crossing_work(
        &mut self,
        ctx: &mut Ctx,
        (h, who): (Handle<Person>, PermanentId),
        household: PermanentId,
        crossing: PermanentId,
        hours: f64,
    ) {
        let Some(i) = ctx
            .land
            .crossings
            .list
            .iter()
            .position(|c| c.id == crossing && c.owner == CrossingOwner::Household(household))
        else {
            return;
        };
        let CrossingState::Building { work_h } = ctx.land.crossings.list[i].state else {
            return;
        };
        let Some(def) = ctx
            .catalog
            .bridges
            .get(usize::from(ctx.land.crossings.list[i].system))
        else {
            return;
        };
        let skill = def
            .skill
            .and_then(|k| ctx.catalog.skills.get(k).map(|s| (k, s)));
        let level = match skill {
            Some((k, _)) => self.people.get(h).map_or(0.0, |p| p.skill(k)),
            None => crate::condition::MIDDLING_SKILL,
        };
        let (now, seed, spread) = (ctx.now, ctx.seed, ctx.params.build.quality_spread);
        let c = &mut ctx.land.crossings.list[i];
        let left = (f64::from(c.labour_h) - f64::from(work_h)).max(0.0);
        let done = hours.clamp(0.0, left);
        let work_h = work_h + done as f32;
        c.skill_h += (done * level) as f32;
        c.state = CrossingState::Building { work_h };
        if let (Some((k, s)), Some(p)) = (skill, self.people.get_mut(h)) {
            p.set_skill(k, s.practised(p.skill(k), done));
        }
        if f64::from(work_h) + 1e-6 < f64::from(c.labour_h) {
            return;
        }
        // Its labour done: its members laid at the quality its builders' skill gives, and open.
        let skill = if c.labour_h > 0.0 {
            (f64::from(c.skill_h) / f64::from(c.labour_h)).clamp(0.0, 1.0)
        } else {
            level
        };
        c.quality = crate::condition::draw_quality(seed, c.id, 0, skill, spread);
        c.state = CrossingState::Open {
            since: now.day_index(),
        };
        let (cells, name) = (c.cells.clone(), def.name.to_lowercase());
        ctx.land.crossings.changed();
        let settlement = self.household(household).and_then(|x| x.settlement);
        let middle = cells[cells.len() / 2] as usize;
        let sentence = format!(
            "{}'s household finished a {name} over a stream, and it is open to walkers.",
            self.name_of(who)
        );
        self.chronicle_push(
            now,
            ChronicleKind::Crossing,
            vec![who],
            settlement,
            Some(cell_centre(ctx.map, middle)),
            f64::from(CrossingStep::Opened as u8),
            sentence,
        );
    }

    /// Someone of `household` sets off along the cells `along`: each river cell they wade, not
    /// walked on a crossing's deck, is one more wade there for the household.
    pub(super) fn wade(&mut self, ctx: &Ctx, household: PermanentId, along: &[u32]) {
        let mut waded: Vec<u32> = along
            .iter()
            .copied()
            .filter(|&c| {
                ctx.map.water.get(c as usize) == Some(&civ_world::WATER_RIVER)
                    && ctx.nav.walkable(c as usize)
                    && ctx.land.crossings.open_at(c).is_none()
            })
            .collect();
        if waded.is_empty() {
            return;
        }
        waded.sort_unstable();
        waded.dedup();
        let (day, half) = (ctx.now.day_index(), ctx.params.places.use_half_life_days);
        for c in waded {
            self.fords.waded(household, c, day, half);
        }
    }

    /// Crossing `i` gives way for `why`, under `under` if someone stepped onto it: whoever is on
    /// it falls, and may die; whoever would have walked over it stops where they are.
    fn crossing_gives_way(
        &mut self,
        ctx: &mut Ctx,
        i: usize,
        why: Collapse,
        under: Option<PermanentId>,
    ) {
        let (now, day) = (ctx.now, ctx.now.day_index());
        let crossing = &mut ctx.land.crossings.list[i];
        crossing.state = CrossingState::Failed { day, why };
        let (id, cells, banks, system) = (
            crossing.id,
            crossing.cells.clone(),
            crossing.banks,
            crossing.system,
        );
        ctx.land.crossings.changed();
        let kills = ctx
            .catalog
            .bridges
            .get(usize::from(system))
            .map_or(0.0, |d| d.fall_kills);
        let map = ctx.map;
        let t = now.minutes() as f64;
        // Who is on it now, and whose walk would cross it.
        let mut on: Vec<(Handle<Person>, PermanentId)> = Vec::new();
        let mut stop: Vec<(Handle<Person>, (f32, f32))> = Vec::new();
        for (h, p) in self.people.iter() {
            let Some(trip) = &p.trip else {
                continue;
            };
            let at = trip.position_at(t);
            if cells.contains(&(cell_of(map, at) as u32)) {
                on.push((h, p.id));
                continue;
            }
            // What is left of the walk, from where they are.
            let since = (t - trip.depart.minutes() as f64) as f32;
            let mut rest = vec![at];
            rest.extend(
                trip.points
                    .iter()
                    .zip(&trip.minutes)
                    .filter(|&(_, &m)| m > since)
                    .map(|(&pt, _)| pt),
            );
            if rest.len() > 1
                && cells_along_by(&rest, map.cell_size_m, map.width, map.height, |a, b| {
                    better_corner(ctx.nav, a, b)
                })
                .iter()
                .any(|c| cells.contains(c))
            {
                stop.push((h, at));
            }
        }
        if let Some(who) = under
            && let Some(&h) = self.index.get(&who)
        {
            on.push((h, who));
        }
        // Those who fell: some die; the rest scramble out onto the nearer bank.
        let mut dead = Vec::new();
        let mut fell = Vec::new();
        for &(h, who) in &on {
            fell.push(who);
            let key = [ctx.seed, PURPOSE_FALL, id.get(), who.get(), day as u64];
            if Rng64::from_key(&key).next_f64() < kills {
                dead.push(who);
                continue;
            }
            let Some(p) = self.people.get_mut(h) else {
                continue;
            };
            if p.trip.is_some() {
                let at = p.trip.as_ref().map_or(p.pos, |trip| trip.position_at(t));
                let near = |b: u32| {
                    let c = cell_centre(map, b as usize);
                    (c.0 - at.0).powi(2) + (c.1 - at.1).powi(2)
                };
                let bank = if near(banks[0]) <= near(banks[1]) {
                    banks[0]
                } else {
                    banks[1]
                };
                p.pos = cell_centre(map, bank as usize);
                self.wait(ctx, h, 1);
            }
        }
        for (h, at) in stop {
            if let Some(p) = self.people.get_mut(h) {
                p.pos = at;
                self.wait(ctx, h, 1);
            }
        }
        // The chronicle, then the dead.
        let name = ctx
            .catalog
            .bridges
            .get(usize::from(system))
            .map_or_else(|| "crossing".to_owned(), |d| d.name.to_lowercase());
        let cause = match why {
            Collapse::OwnWeight => "its rotten members could no longer carry their own weight",
            Collapse::UnderWalker => "its rotten members broke under someone stepping onto it",
        };
        let mut sentence = format!("A {name} gave way: {cause}.");
        if !fell.is_empty() {
            let names: Vec<String> = fell.iter().map(|&p| self.name_of(p)).collect();
            sentence += &format!(
                " {} fell into the water{}.",
                names.join(" and "),
                if dead.is_empty() {
                    String::new()
                } else {
                    let d: Vec<String> = dead.iter().map(|&p| self.name_of(p)).collect();
                    format!("; {} died", d.join(" and "))
                }
            );
        }
        let middle = cells[cells.len() / 2] as usize;
        self.chronicle_push(
            now,
            ChronicleKind::Crossing,
            fell,
            None,
            Some(cell_centre(map, middle)),
            f64::from(CrossingStep::Failed as u8),
            sentence,
        );
        for who in dead {
            self.die(ctx, who, Cause::Fell);
        }
    }
}
