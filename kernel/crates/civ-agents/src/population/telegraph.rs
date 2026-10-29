//! The telegraph (M11c; ADR-0036 §8): an office is a household that holds a whole telegraph
//! instrument and someone who can write and knows every technique of a recipe making it, its
//! operator. On its day of the year an office weighs a wire to an office of another settlement,
//! as a station weighs a line: what the errands its settlement's households ran to that market in
//! a year would pay it in fees, had each asked first by wire, against stringing the wire along
//! ways with leave and its conductor. It begins the best that repays over a wire's life and strings
//! it as its way work. A wire carries messages only once whole (the second part: what is sent).

use civ_core::time::DAYS_PER_YEAR;
use civ_core::{PermanentId, Rng64};
use civ_land::wire::Telegraph;

use super::*;
use crate::decide::WayFacts;

/// What a message handed in at an office costs, in days of one person's food (a design prior, as
/// the meter's price is: some 125 kcal at 2,500 a day, an hour or so of a labourer's food).
pub const TELEGRAM_FEE_FOOD_DAYS: f64 = 0.05;

/// Keyed-randomness purpose of an office's day of the year for weighing a wire.
const PURPOSE_TELEGRAPH_REVIEW: u64 = 0x7465_6c65_6772_6170; // "telegrap"

/// What an office reckons of a wire to another office at its review (M11c): the far office, the
/// cells and length, metres, what it would repay a year and what stringing it takes, hours.
#[derive(Clone, Debug, PartialEq)]
pub struct TelegraphReckoning {
    pub to: PermanentId,
    pub cells: Vec<u32>,
    pub length_m: f64,
    pub worth_h_year: f64,
    pub cost_h: f64,
}

/// A message's fee, kcal of food.
pub(super) fn fee_kcal(ctx: &Ctx) -> f64 {
    TELEGRAM_FEE_FOOD_DAYS * ctx.params.household.daily_kcal_per_person
}

impl Population {
    /// The telegraph instruments of the catalogue, by good index, with the messages an hour each
    /// sends.
    pub(super) fn telegraph_goods(ctx: &Ctx) -> Vec<(usize, f64)> {
        ctx.catalog
            .goods
            .iter()
            .enumerate()
            .filter_map(|(g, d)| Some((g, d.telegraph?.messages_h)))
            .collect()
    }

    /// Household `office`'s operator and the messages an hour its office sends, if it is an
    /// office (M11c): it holds a whole telegraph instrument, and someone grown of it (the first by
    /// id) can write and knows every technique of a recipe making that instrument. The best of
    /// its instruments where it holds several kinds.
    pub fn telegraph_office(&self, ctx: &Ctx, office: PermanentId) -> Option<(PermanentId, f64)> {
        let x = self.household(office)?;
        let writing = Self::letter_technique(ctx.catalog, Behavior::Write)?;
        Self::telegraph_goods(ctx)
            .into_iter()
            .filter(|&(g, _)| x.stores.get(g).copied().unwrap_or(0.0) >= 1.0 - 1e-9)
            .filter_map(|(g, rate)| {
                let needs: Vec<usize> = ctx
                    .catalog
                    .recipes
                    .iter()
                    .filter(|r| r.outputs.iter().any(|&(o, _)| o == g))
                    .filter_map(|r| r.technique)
                    .collect();
                let operator = x.members.iter().copied().filter(|&m| {
                    self.person(m).is_some_and(|p| {
                        p.age_years(ctx.now) >= ctx.params.family.independent_age
                            && p.knows(writing)
                            && needs.iter().all(|&t| p.knows(t))
                    })
                });
                Some((operator.min()?, rate))
            })
            .max_by(|a, b| a.1.total_cmp(&b.1))
    }

    /// Each midnight, the offices whose day of the year it is weigh a wire to another office
    /// (M11c). Nothing where the land lays no wire or the catalogue has no instrument.
    pub(super) fn telegraph_reviews(&mut self, ctx: &mut Ctx, day: i64) {
        if ctx.land_params.wire.is_none() || Self::telegraph_goods(ctx).is_empty() {
            return;
        }
        let due: Vec<PermanentId> = self
            .households
            .iter()
            .map(|(_, x)| x.id)
            .filter(|&id| {
                Rng64::from_key(&[ctx.seed, PURPOSE_TELEGRAPH_REVIEW, id.get()]).next_u64()
                    % DAYS_PER_YEAR as u64
                    == day.rem_euclid(DAYS_PER_YEAR) as u64
            })
            .filter(|&id| self.telegraph_office(ctx, id).is_some())
            .collect();
        for office in due {
            self.review_telegraph(ctx, office);
        }
    }

    /// The errands people of settlement `from` ran to the market of settlement `to` in a year,
    /// bought or not: the more of this year's and last year's counts.
    pub(super) fn errands_between(&self, ctx: &Ctx, from: PermanentId, to: PermanentId) -> u32 {
        let year = ctx.now.day_index().div_euclid(DAYS_PER_YEAR);
        [year - 1, year]
            .iter()
            .map(|&y| {
                self.contacts
                    .years
                    .get(&(y, from, to))
                    .map_or(0, |c| c.bought + c.missed.iter().sum::<u32>())
            })
            .max()
            .unwrap_or(0)
    }

    /// What wires from office `office` to an office of another settlement would repay it, the best
    /// first (M11c): to each such office it has no wire with, routed along ways with leave from
    /// its home to the other's, worth a year the fee in its own hours of food for each errand its
    /// settlement's people ran to that market in a year, as though each had asked by wire first;
    /// costing a metre's stringing and conductor at its own cost (or in hand) each metre. None
    /// while one of its wires is being strung.
    pub(super) fn telegraph_reckonings(
        &self,
        ctx: &Ctx,
        office: PermanentId,
    ) -> Vec<TelegraphReckoning> {
        let mut out = Vec::new();
        let Some(params) = ctx.land_params.wire else {
            return out;
        };
        let Some(x) = self.household(office) else {
            return out;
        };
        let Some(home) = x.settlement else {
            return out;
        };
        let wires = &ctx.land.wires;
        if self.telegraph_office(ctx, office).is_none()
            || wires.telegraphs.iter().any(|t| t.a == office && !t.whole())
        {
            return out;
        }
        let costs = self.own_costs_of(ctx, x);
        let Some(per_kcal) = super::wires::hours_per_kcal(&ctx.catalog.goods, &costs) else {
            return out;
        };
        let held = x.stores.get(params.good).copied().unwrap_or(0.0);
        let conductor_h = costs.get(params.good).copied().flatten();
        let from = cell_of(ctx.map, x.home) as u32;
        let fars: Vec<(PermanentId, PermanentId, u32)> = self
            .households
            .iter()
            .map(|(_, y)| y)
            .filter(|y| y.id != office && y.settlement.is_some_and(|s| s != home))
            .filter(|y| !wires.telegraphs.iter().any(|t| t.joins(office, y.id)))
            .filter(|y| self.telegraph_office(ctx, y.id).is_some())
            .filter_map(|y| Some((y.id, y.settlement?, cell_of(ctx.map, y.home) as u32)))
            .collect();
        for (to, market, cell) in fars {
            let errands = self.errands_between(ctx, home, market);
            if errands == 0 {
                continue;
            }
            let Some((cells, length_m)) = self
                .wire_route(ctx, office, from, cell)
                .filter(|r| r.1 >= 1e-3)
            else {
                continue;
            };
            let kg = length_m * params.good_kg_m;
            let conductor = match conductor_h {
                Some(h) => kg * h,
                None if held + 1e-9 >= kg => 0.0,
                None => continue,
            };
            out.push(TelegraphReckoning {
                to,
                cells,
                length_m,
                worth_h_year: f64::from(errands) * fee_kcal(ctx) * per_kcal,
                cost_h: length_m * params.lay_h_m + conductor,
            });
        }
        let net = |r: &TelegraphReckoning| r.worth_h_year * params.life_years - r.cost_h;
        out.sort_by(|a, b| net(b).total_cmp(&net(a)).then(a.to.cmp(&b.to)));
        out
    }

    /// What wires from office `office` would repay it, the best first: for tests.
    #[doc(hidden)]
    pub fn telegraph_reckonings_for_tests(
        &self,
        ctx: &Ctx,
        office: PermanentId,
    ) -> Vec<TelegraphReckoning> {
        self.telegraph_reckonings(ctx, office)
    }

    /// Office `office` weighs a wire (M11c): it begins the one that repays it most where that
    /// repays more over a wire's life than stringing it takes, marked out with what it reckoned.
    fn review_telegraph(&mut self, ctx: &mut Ctx, office: PermanentId) {
        let Some(params) = ctx.land_params.wire else {
            return;
        };
        let Some(best) = self.telegraph_reckonings(ctx, office).into_iter().next() else {
            return;
        };
        if best.worth_h_year * params.life_years <= best.cost_h {
            return;
        }
        if let Some(t) = self.begin_telegraph(ctx, office, best.to)
            && let Some(w) = ctx.land.wires.telegraphs.iter_mut().find(|w| w.id == t)
        {
            w.worth_h = best.worth_h_year;
        }
    }

    /// Office `office` weighs a wire now, whatever its day of the year: for tests.
    #[doc(hidden)]
    pub fn review_telegraph_for_tests(&mut self, ctx: &mut Ctx, office: PermanentId) {
        self.review_telegraph(ctx, office);
    }

    /// Begins a telegraph wire for household `a` from its home to household `b`'s, routed as a
    /// line of `a`'s is ([`Population::begin_line`]), marked out at its length with nothing yet
    /// strung. Returns its id; `None` where the land lays no wire or no route keeps to such ways.
    pub fn begin_telegraph(
        &self,
        ctx: &mut Ctx,
        a: PermanentId,
        b: PermanentId,
    ) -> Option<PermanentId> {
        ctx.land_params.wire?;
        let from = cell_of(ctx.map, self.household(a)?.home) as u32;
        let to = cell_of(ctx.map, self.household(b)?.home) as u32;
        let (cells, length_m) = self.wire_route(ctx, a, from, to)?;
        let id = ctx.ids.allocate();
        ctx.land.wires.telegraphs.push(Telegraph {
            id,
            a,
            b,
            cells,
            length_m,
            laid_m: 0.0,
            broken: false,
            worth_h: 0.0,
        });
        Some(id)
    }

    /// The telegraph wire household `x` is stringing, as way work for its people (M11c, as a
    /// line's): where its work stands, the walk there, the hours still to string, and each hour's
    /// worth, the hours a year it was reckoned to repay over a wire's life spread over its
    /// stringing. `None` without one, or without a metre's conductor in hand.
    pub(super) fn telegraph_facts(&self, ctx: &Ctx, x: &Household) -> Option<WayFacts> {
        let params = ctx.land_params.wire?;
        let t = ctx
            .land
            .wires
            .telegraphs
            .iter()
            .find(|t| t.a == x.id && !t.whole() && !t.broken)?;
        if x.stores.get(params.good).copied().unwrap_or(0.0) < params.good_kg_m {
            return None;
        }
        let n = t.cells.len();
        let share = t.laid_m / t.length_m.max(1e-3);
        let cell = *t
            .cells
            .get(((share * n as f64) as usize).min(n.checked_sub(1)?))?;
        let at = cell_centre(ctx.map, cell as usize);
        let pace = ctx.params.nav.tobler_ms(0.0).max(1e-6);
        let labour_h = t.length_m * params.lay_h_m;
        Some(WayFacts {
            way: t.id,
            at,
            walk_min: f64::from((at.0 - x.home.0).hypot(at.1 - x.home.1)) / pace / 60.0,
            left_h: (t.length_m - t.laid_m).max(0.0) * params.lay_h_m,
            saves_per_hour: t.worth_h * params.life_years / labour_h.max(1e-6),
            duty: 0.0,
            full: false,
        })
    }

    /// `hours` of work by someone of `household` on its telegraph wire `wire`: a metre strung each
    /// `lay_h_m` hours, each taking its conductor from the household's store, no more than the
    /// conductor held allows or the wire has left.
    pub(super) fn telegraph_work(
        &mut self,
        ctx: &mut Ctx,
        household: PermanentId,
        wire: PermanentId,
        hours: f64,
    ) {
        let Some(params) = ctx.land_params.wire else {
            return;
        };
        let Some(t) = ctx
            .land
            .wires
            .telegraphs
            .iter_mut()
            .find(|t| t.id == wire && t.a == household && !t.whole())
        else {
            return;
        };
        let Some(x) = self.household_mut(household) else {
            return;
        };
        let held = x.stores.get(params.good).copied().unwrap_or(0.0).max(0.0);
        let metres = (hours.max(0.0) / params.lay_h_m.max(1e-9))
            .min(held / params.good_kg_m.max(1e-9))
            .min((t.length_m - t.laid_m).max(0.0));
        t.laid_m = (t.laid_m + metres).min(t.length_m);
        if t.length_m - t.laid_m < 1e-3 {
            t.laid_m = t.length_m;
        }
        let used = (metres * params.good_kg_m).min(held);
        if used > 0.0 {
            x.stores[params.good] = held - used;
            x.flows.add(Flow::Used, params.good, used);
        }
    }

    /// `hours` of work on a telegraph wire: for tests.
    #[doc(hidden)]
    pub fn telegraph_work_for_tests(
        &mut self,
        ctx: &mut Ctx,
        household: PermanentId,
        wire: PermanentId,
        hours: f64,
    ) {
        self.telegraph_work(ctx, household, wire, hours);
    }
}
