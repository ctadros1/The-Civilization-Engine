//! Works on ways by a household (M8b slice CQ, second part; ADR-0029 §6; the transport brief
//! §1.8; research 10-07 §1.6, §2.2): a passing place where its carts wait. At its yearly review a
//! household weighs each single-cart stretch where it holds waits (`traffic::Cause::Width`): a
//! passing place cleared to two carts' width at the stretch's middle would spare it about half of
//! them, the stretch split in two, over the years it reckons such a place lasts; making one
//! takes the hours clearing and levelling its ground takes. Where the first is more, and the
//! place is within the work's walk from home, it begins one: a way of one cell, kept by it, its
//! built width growing with each hour of its people's work until it is as wide as two carts.
//!
//! The third part: where the waits the settlement's households hold at a stretch together would
//! repay the place's work, though no one household's might, a passing place is a work the polity
//! may build (`build_way`, shaped as `build_bank`): the issue `carts_wait`, each household's
//! forecast its own waits spared less its equal share of the work over the place's life, and,
//! once passed, a way of one cell kept by the polity whose clearing each household is asked an
//! equal share of.

use super::*;
use crate::decide::WayFacts;
use crate::polity::{LawStatus, PolicyKind};
use crate::traffic;
use civ_land::ways::{CART, Keeper, Rank, Segment, Surface, WALK, Way};

/// Where the settlement's people could clear a passing place together (M8b slice CQ, third
/// part): the cell at a single-cart stretch's middle, the hours clearing it takes, the years
/// they reckon it lasts, the hours of waiting a year it would spare each household that holds
/// waits there (by household), and how many households would share its work.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PublicWay {
    pub cell: u32,
    pub labour_h: f64,
    pub life: f64,
    pub saves: Vec<(PermanentId, f64)>,
    pub households: usize,
}

impl PublicWay {
    /// Each household's equal share of the work, hours.
    pub fn share_h(&self) -> f64 {
        self.labour_h / self.households.max(1) as f64
    }

    /// The hours of waiting a year it would spare household `h`.
    pub fn saves_of(&self, h: PermanentId) -> f64 {
        self.saves
            .binary_search_by_key(&h, |s| s.0)
            .map_or(0.0, |i| self.saves[i].1)
    }

    /// What it would spare all its households over its life, less its work, hours.
    pub fn pooled_net_h(&self) -> f64 {
        self.saves.iter().map(|s| s.1).sum::<f64>() * self.life - self.labour_h
    }
}

/// Purpose key for when in the year a household weighs a passing place.
const PURPOSE_WAY_REVIEW: u64 = 0x7761_7977_6f72_6b72; // "wayworkr"
/// The share of the waits at a single-cart stretch a passing place at its middle spares: the
/// stretch split in two, a cart waits for one coming the other way on half of it (a design prior).
pub const PASSING_SPARES: f64 = 0.5;
/// The width a way laid on worn trail is marked out at before any of its ground is cleared,
/// metres (M8b slice CQ, fourth part): no wheels take it so narrow.
pub const LAID_FROM_M: f32 = 0.05;

impl Population {
    /// The way household `household` is working on: one it keeps whose built width is short of
    /// its corridor.
    fn way_in_work(ctx: &Ctx, household: PermanentId) -> Option<usize> {
        ctx.land.ways.list.iter().position(|w| {
            w.segments.iter().any(|s| {
                s.keeper == Keeper::Household(household) && s.built_m + 1e-3 < s.corridor_m
            })
        })
    }

    /// The hours a year household `household` holds of waiting at the single-cart stretch whose
    /// middle is `cell`, as of `day`.
    fn width_hours_a_year(&self, ctx: &Ctx, household: PermanentId, cell: u32, day: i64) -> f64 {
        let half = ctx.params.places.use_half_life_days;
        self.traffic
            .losses_held(household, day, half)
            .filter(|l| l.cell == cell && l.cause == traffic::Cause::Width)
            .map(|l| crate::uses::per_year(f64::from(l.hours), half))
            .sum()
    }

    /// The hours clearing a passing place takes: the strip that widens a trail's tread to two
    /// carts' width, along a cell.
    fn passing_labour_h(ctx: &Ctx) -> f64 {
        let works = ctx.land_params.way_works;
        let tread = civ_land::carting::TRAIL_TREAD_M as f32;
        f64::from((works.passing_width_m - tread).max(0.0))
            * f64::from(ctx.map.cell_size_m)
            * works.clear_h_m2
    }

    /// Households whose day of the year it is weigh a passing place where their carts wait.
    pub(super) fn way_work_reviews(&mut self, ctx: &mut Ctx, day: i64) {
        if !ctx
            .catalog
            .activities
            .iter()
            .any(|a| a.behavior == Behavior::Way)
        {
            return;
        }
        let due: Vec<PermanentId> = self
            .traffic
            .losses
            .keys()
            .copied()
            .filter(|h| {
                Rng64::from_key(&[ctx.seed, PURPOSE_WAY_REVIEW, h.get()]).next_u64()
                    % DAYS_PER_YEAR as u64
                    == day.rem_euclid(DAYS_PER_YEAR) as u64
            })
            .collect();
        for h in due {
            self.review_way_work(ctx, h, day);
        }
    }

    /// Household `household` weighs its works on ways now, whatever its day of the year: for
    /// tests.
    #[doc(hidden)]
    pub fn review_way_work_for_tests(&mut self, ctx: &mut Ctx, household: PermanentId) {
        let day = ctx.now.day_index();
        self.review_way_work(ctx, household, day);
    }

    /// Household `household` weighs a passing place at each single-cart stretch where it holds
    /// waits, and laying a way over the worn trail its loaded carts drag on near home, and begins
    /// the one whose saving most exceeds its work, if any does: one at a time, and none where a
    /// way already runs.
    fn review_way_work(&mut self, ctx: &mut Ctx, household: PermanentId, day: i64) {
        let Some(x) = self.household(household) else {
            return;
        };
        if x.members.is_empty() || Self::way_in_work(ctx, household).is_some() {
            return;
        }
        let Some(act) = ctx
            .catalog
            .activities
            .iter()
            .find(|a| a.behavior == Behavior::Way)
        else {
            return;
        };
        let works = ctx.land_params.way_works;
        let labour_h = Self::passing_labour_h(ctx);
        let pace = ctx.params.nav.tobler_ms(0.0).max(1e-6);
        let home = x.home;
        let half = ctx.params.places.use_half_life_days;
        let mut best: Option<(u32, f64)> = None;
        for l in self.traffic.losses_held(household, day, half) {
            if l.cause != traffic::Cause::Width || ctx.land.ways.at(l.cell).next().is_some() {
                continue;
            }
            let at = cell_centre(ctx.map, l.cell as usize);
            let walk_min = f64::from((at.0 - home.0).hypot(at.1 - home.1)) / pace / 60.0;
            if walk_min > f64::from(act.max_walk_minutes) {
                continue;
            }
            let saved_h = crate::uses::per_year(f64::from(l.hours), half)
                * PASSING_SPARES
                * works.passing_life_years;
            let gain = saved_h - labour_h;
            if gain > 0.0 && best.is_none_or(|(_, g)| gain > g) {
                best = Some((l.cell, gain));
            }
        }
        // Or its approach: each cell of worn trail within the work's walk where it holds what
        // its loaded carts lost to the trail's roughness, and no way runs, whose roughness spared
        // over the years it reckons a way lasts is more than clearing a trail's tread there takes
        // (M8b slice CQ, fourth part).
        let lay_h = f64::from(civ_land::carting::TRAIL_TREAD_M as f32)
            * f64::from(ctx.map.cell_size_m)
            * works.clear_h_m2;
        let mut approach: Vec<u32> = Vec::new();
        let mut approach_gain = 0.0;
        for l in self.traffic.losses_held(household, day, half) {
            if l.cause != traffic::Cause::Rough || ctx.land.ways.at(l.cell).next().is_some() {
                continue;
            }
            let at = cell_centre(ctx.map, l.cell as usize);
            let walk_min = f64::from((at.0 - home.0).hypot(at.1 - home.1)) / pace / 60.0;
            if walk_min > f64::from(act.max_walk_minutes) {
                continue;
            }
            let gain =
                crate::uses::per_year(f64::from(l.hours), half) * works.passing_life_years - lay_h;
            if gain > 0.0 {
                approach.push(l.cell);
                approach_gain += gain;
            }
        }
        match best {
            Some((cell, gain)) if gain >= approach_gain => {
                Self::lay_passing_place(ctx, cell, Keeper::Household(household), day);
            }
            _ if !approach.is_empty() => {
                Self::lay_approach(ctx, approach, Keeper::Household(household), day);
            }
            _ => {}
        }
    }

    /// Lays a way on worn trail over `cells` (ascending), kept by `keeper` (M8b slice CQ, fourth
    /// part): one segment of cleared ground, marked out for now, to be cleared as wide as a
    /// trail's tread. Returns its id.
    fn lay_approach(ctx: &mut Ctx, cells: Vec<u32>, keeper: Keeper, day: i64) -> PermanentId {
        let tread = civ_land::carting::TRAIL_TREAD_M as f32;
        let id = ctx.ids.allocate();
        ctx.land.ways.add(Way {
            id,
            begun: day,
            segments: vec![Segment {
                cells,
                corridor_m: tread,
                built_m: LAID_FROM_M,
                usable_m: LAID_FROM_M,
                surface: Surface::Cleared,
                keeper,
                kept: day,
            }],
            modes: WALK | CART,
            trips: 0.0,
            trips_day: day,
            rank: Rank::Lane,
        });
        id
    }

    /// Lays a passing place at `cell`, kept by `keeper`: a way of one cell, its built width the
    /// trail's tread, its corridor two carts' width. Returns its id.
    fn lay_passing_place(ctx: &mut Ctx, cell: u32, keeper: Keeper, day: i64) -> PermanentId {
        let works = ctx.land_params.way_works;
        let tread = civ_land::carting::TRAIL_TREAD_M as f32;
        let id = ctx.ids.allocate();
        ctx.land.ways.add(Way {
            id,
            begun: day,
            segments: vec![Segment {
                cells: vec![cell],
                corridor_m: works.passing_width_m.max(tread),
                built_m: tread,
                usable_m: tread,
                surface: Surface::Cleared,
                keeper,
                kept: day,
            }],
            modes: WALK | CART,
            trips: 0.0,
            trips_day: day,
            rank: Rank::Lane,
        });
        id
    }

    /// Where settlement `settlement`'s households `households` (sorted) could clear a passing
    /// place together, if anywhere (M8b slice CQ, third part): of the single-cart stretches where
    /// they hold waits and no way runs yet, the one whose waits spared over its life most exceed
    /// its work, if any do; none while their polity is clearing one (one at a time), or without
    /// a template that builds one.
    pub(crate) fn public_way(
        &self,
        ctx: &Ctx,
        settlement: PermanentId,
        households: &[PermanentId],
    ) -> Option<PublicWay> {
        if self.traffic.losses.is_empty()
            || !ctx
                .catalog
                .policies
                .iter()
                .any(|d| d.kind == PolicyKind::BuildWay)
        {
            return None;
        }
        let polity = self.polities.get(self.polity_of(settlement)?)?.id;
        if ctx.land.ways.list.iter().any(|w| {
            w.segments
                .iter()
                .any(|s| s.keeper == Keeper::Polity(polity) && s.built_m + 1e-3 < s.corridor_m)
        }) {
            return None;
        }
        let day = ctx.now.day_index();
        let half = ctx.params.places.use_half_life_days;
        let mut by_cell: BTreeMap<u32, Vec<(PermanentId, f64)>> = BTreeMap::new();
        for &h in households {
            for l in self.traffic.losses_held(h, day, half) {
                if l.cause != traffic::Cause::Width || ctx.land.ways.at(l.cell).next().is_some() {
                    continue;
                }
                let spared = crate::uses::per_year(f64::from(l.hours), half) * PASSING_SPARES;
                let saves = by_cell.entry(l.cell).or_default();
                match saves.last_mut() {
                    Some(last) if last.0 == h => last.1 += spared,
                    _ => saves.push((h, spared)),
                }
            }
        }
        let labour_h = Self::passing_labour_h(ctx);
        let life = ctx.land_params.way_works.passing_life_years;
        let mut best: Option<PublicWay> = None;
        for (cell, saves) in by_cell {
            let p = PublicWay {
                cell,
                labour_h,
                life,
                saves,
                households: households.len(),
            };
            if p.pooled_net_h() > 0.0
                && best
                    .as_ref()
                    .is_none_or(|b| p.pooled_net_h() > b.pooled_net_h())
            {
                best = Some(p);
            }
        }
        best
    }

    /// Where settlement `settlement`'s people could clear a passing place together now: for
    /// tests.
    #[doc(hidden)]
    #[allow(clippy::type_complexity)]
    pub fn public_way_for_tests(
        &self,
        ctx: &Ctx,
        settlement: PermanentId,
    ) -> Option<(u32, f64, Vec<(PermanentId, f64)>)> {
        let mut households: Vec<PermanentId> = self
            .households
            .iter()
            .filter(|(_, x)| x.settlement == Some(settlement) && !x.members.is_empty())
            .map(|(_, x)| x.id)
            .collect();
        households.sort_unstable();
        self.public_way(ctx, settlement, &households)
            .map(|p| (p.cell, p.labour_h, p.saves))
    }

    /// Polity `pi` lays out the passing place its law `law` names, as its law passed would (M8b
    /// slice CQ, third part): unless another way has come to run there since.
    pub(super) fn begin_public_way(&mut self, ctx: &mut Ctx, pi: usize, law: PermanentId) {
        let polity = self.polities[pi].id;
        let Some(site) = self.polities[pi].way_site_of(law) else {
            return;
        };
        if site.work.is_some() || ctx.land.ways.at(site.cell).next().is_some() {
            return;
        }
        let day = ctx.now.day_index();
        let id = Self::lay_passing_place(ctx, site.cell, Keeper::Polity(polity), day);
        if let Some(s) = self.polities[pi].ways.iter_mut().find(|s| s.0 == law) {
            s.1.work = Some(id);
        }
    }

    /// Polity `pi` lays out the passing place its law `law` names: for tests.
    #[doc(hidden)]
    pub fn begin_public_way_for_tests(&mut self, ctx: &mut Ctx, pi: usize, law: PermanentId) {
        self.begin_public_way(ctx, pi, law);
    }

    /// The passing place household `x`'s people may work on, if it is light (M8b slice CQ): the
    /// one it is making, or else one its polity is clearing under a law in force `person` knows
    /// of; where its work is done, the walk there from home, the hours of a capable adult's work
    /// it still takes, the hours of waiting each hour of that work spares the household over the
    /// place's life (for the polity's, what it reckoned when the law was proposed), and, while
    /// the household has not given its equal share of the polity's, what keeping to the
    /// gathering's word is worth to `person`, as for a flood bank built together.
    pub(super) fn way_facts(
        &self,
        ctx: &Ctx,
        x: &Household,
        person: PermanentId,
        dark: bool,
    ) -> Option<WayFacts> {
        if dark || ctx.land.ways.list.is_empty() {
            return None;
        }
        let works = ctx.land_params.way_works;
        let pace = ctx.params.nav.tobler_ms(0.0).max(1e-6);
        let left_of = |s: &Segment| {
            let metres = f64::from(ctx.map.cell_size_m) * s.cells.len() as f64;
            f64::from(s.corridor_m - s.built_m).max(0.0) * metres * works.clear_h_m2
        };
        let walk_of = |cell: u32| {
            let at = cell_centre(ctx.map, cell as usize);
            (
                at,
                f64::from((at.0 - x.home.0).hypot(at.1 - x.home.1)) / pace / 60.0,
            )
        };
        if let Some(wi) = Self::way_in_work(ctx, x.id) {
            let w = &ctx.land.ways.list[wi];
            let s = w.segments.first()?;
            let cell = *s.cells.first()?;
            let metres = f64::from(ctx.map.cell_size_m) * s.cells.len() as f64;
            let tread = civ_land::carting::TRAIL_TREAD_M as f32;
            let day = ctx.now.day_index();
            // A way laid on trail is cleared from nothing to a tread's width, sparing the
            // roughness its carts lost on each of its cells; a passing place is widened from the
            // tread, sparing half the waits at its cell.
            let (labour_h, saved_h) = if s.corridor_m <= tread + 1e-3 {
                let half = ctx.params.places.use_half_life_days;
                let rough: f64 = self
                    .traffic
                    .losses_held(x.id, day, half)
                    .filter(|l| l.cause == traffic::Cause::Rough && s.cells.contains(&l.cell))
                    .map(|l| crate::uses::per_year(f64::from(l.hours), half))
                    .sum();
                (
                    f64::from(tread) * metres * works.clear_h_m2,
                    rough * works.passing_life_years,
                )
            } else {
                (
                    f64::from(s.corridor_m - tread).max(0.1) * metres * works.clear_h_m2,
                    self.width_hours_a_year(ctx, x.id, cell, day)
                        * PASSING_SPARES
                        * works.passing_life_years,
                )
            };
            let (at, walk_min) = walk_of(cell);
            return Some(WayFacts {
                way: w.id,
                at,
                walk_min,
                left_h: left_of(s),
                saves_per_hour: saved_h / labour_h,
                duty: 0.0,
                full: false,
            });
        }
        let pi = self.polity_of(x.settlement?)?;
        let polity = &self.polities[pi];
        let (law, site, way, left_h) = polity.ways.iter().find_map(|(l, site)| {
            let law = polity.laws.iter().find(|k| {
                k.id == *l
                    && k.status == LawStatus::InForce
                    && k.kind == PolicyKind::BuildWay
                    && k.knows(person)
            })?;
            let w = ctx
                .land
                .ways
                .list
                .iter()
                .find(|w| Some(w.id) == site.work)?;
            let s = w.segments.iter().find(|s| {
                s.keeper == Keeper::Polity(polity.id) && s.built_m + 1e-3 < s.corridor_m
            })?;
            Some((law, *site, w.id, left_of(s)))
        })?;
        let households = self
            .households
            .iter()
            .filter(|(_, y)| y.settlement == x.settlement && !y.members.is_empty())
            .count()
            .max(1);
        let share = site.labour_h / households as f64;
        let mut duty = 0.0;
        if f64::from(polity.way_given_by(law.id, x.id)) + 1e-6 < share {
            let stance = law
                .stances
                .iter()
                .find(|r| r.person == person)
                .map_or(0.0, |r| r.stance.sign());
            let regard =
                self.ties
                    .regard(person, law.sponsor, ctx.now.day_index(), &ctx.params.ties);
            let norm = self.norm_points(ctx, person);
            duty = crate::polity::comply_points(norm, stance, regard, 0.0, &ctx.params.polity)
                .max(0.0);
        }
        let (at, walk_min) = walk_of(site.cell);
        Some(WayFacts {
            way,
            at,
            walk_min,
            left_h,
            saves_per_hour: f64::from(polity.way_saves_of(law.id, x.id)) * works.passing_life_years
                / site.labour_h.max(1e-6),
            duty,
            full: false,
        })
    }

    /// `hours` of a capable adult's work by someone of `household` on the way `way`: each clears
    /// and levels its share of a square metre a `clear_h_m2`, widening the stretch along its
    /// length, until it is as wide as its corridor. It is kept that day. On a polity's, the hours
    /// it took are the household's given to the law that laid it, and to the polity's works.
    pub(super) fn way_work(
        &mut self,
        ctx: &mut Ctx,
        household: PermanentId,
        way: PermanentId,
        hours: f64,
    ) {
        let works = ctx.land_params.way_works;
        let cell_m = f64::from(ctx.map.cell_size_m);
        let day = ctx.now.day_index();
        let Some(w) = ctx.land.ways.list.iter_mut().find(|w| w.id == way) else {
            return;
        };
        let mut used = None;
        for s in &mut w.segments {
            let ours = match s.keeper {
                Keeper::Household(h) => h == household,
                Keeper::Polity(p) => self
                    .household(household)
                    .and_then(|x| self.polity_of(x.settlement?))
                    .is_some_and(|pi| self.polities[pi].id == p),
                Keeper::Nobody => false,
            };
            if !ours || s.built_m + 1e-3 >= s.corridor_m {
                continue;
            }
            let area = cell_m * s.cells.len() as f64;
            let wider = (hours / works.clear_h_m2 / area.max(1e-6)) as f32;
            let before = s.built_m;
            s.built_m = (s.built_m + wider).min(s.corridor_m);
            s.usable_m = s.built_m;
            s.kept = day;
            let took = f64::from(s.built_m - before) * area * works.clear_h_m2;
            used = Some((s.keeper, took.min(hours)));
            break;
        }
        let Some((keeper, took)) = used else {
            return;
        };
        ctx.land.ways.changed();
        if let Keeper::Polity(p) = keeper
            && let Some(pi) = self.polities.iter().position(|q| q.id == p)
            && let Some(law) = self.polities[pi]
                .ways
                .iter()
                .find(|s| s.1.work == Some(way))
                .map(|s| s.0)
            && took > 0.0
        {
            self.polities[pi].give_to_way(law, household, took as f32);
            self.work_given.push((p, household, took));
        }
    }

    /// `hours` of work by someone of `household` on way `way`: for tests.
    #[doc(hidden)]
    pub fn way_work_for_tests(
        &mut self,
        ctx: &mut Ctx,
        household: PermanentId,
        way: PermanentId,
        hours: f64,
    ) {
        self.way_work(ctx, household, way, hours);
    }
}
