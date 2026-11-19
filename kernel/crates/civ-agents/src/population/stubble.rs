//! The stubble kept for those who cannot earn the harvest wage (§8 17, fourth, slice 2; research
//! 08-22 §1, §4): the Islip (1407) and Wymeswold bylaws barred from gleaning anyone who could earn
//! a penny, or two pence, a day at the harvest, so that what reaping leaves fed those who could
//! not, and the able went to the harvest wage. Here it is a law a polity may come to by its own
//! means: those who glean, and would glean what the able now take, may propose it; each household
//! weighs it by what its own gleaning would gain and what its own able would give up; once in
//! force, one of an age to work for hire who knows it weighs keeping it when they would glean, as
//! a curfew is kept, unless their household is going hungry.

use civ_core::PermanentId;

use super::{Ctx, Population, may_eat_reserve, reapers};
use crate::person::{Household, Person, Target};
use crate::polity::{LawStatus, PolicyKind};

impl Population {
    /// What keeping the stubble for those who cannot earn would bring each of `households` a year
    /// and cost it, kcal: what the polity's people who could earn the harvest wage gleaned, a
    /// year at the rate the record fades, shared among the households whose others gleaned by
    /// what each of them gleaned (they would glean it in the place of those barred); less what
    /// its own who could earn gleaned. None where content has no such law, nobody who could earn
    /// gleaned, or nobody else did.
    pub(super) fn stubble_forecasts(
        &self,
        ctx: &Ctx,
        households: &[PermanentId],
    ) -> Vec<(PermanentId, f64, f64)> {
        if !ctx
            .catalog
            .policies
            .iter()
            .any(|d| d.kind == PolicyKind::KeepStubble)
        {
            return Vec::new();
        }
        let (day, params) = (ctx.now.day_index(), ctx.params);
        let half = params.places.use_half_life_days;
        let kcal = ctx
            .catalog
            .crops
            .get(params.farm.crop)
            .and_then(|c| ctx.catalog.goods.get(c.good))
            .map_or(0.0, |g| g.kcal_per_kg);
        let held: Vec<(PermanentId, f64, f64)> = households
            .iter()
            .filter_map(|&h| {
                let i = self
                    .order
                    .gleaners
                    .binary_search_by_key(&h, |g| g.household)
                    .ok()?;
                let (able, other) = self.order.gleaners[i].at(day, half);
                let year = |kg: f64| crate::uses::per_year(kg, half) * kcal;
                Some((h, year(able), year(other)))
            })
            .collect();
        let able: f64 = held.iter().map(|g| g.1).sum();
        let others: f64 = held.iter().map(|g| g.2).sum();
        if able <= 0.0 || others <= 0.0 {
            return Vec::new();
        }
        held.iter()
            .map(|&(h, a, o)| (h, able * o / others, a))
            .collect()
    }

    /// Whether activity `def` at `target` by someone of `hh` is gleaning on its polity's own
    /// stubble, where a law keeping the stubble binds (§8 19, seventh; research 08-22 §1: the
    /// custom was the place's own, its bylaws its own fields'). Elsewhere it is not the polity's
    /// to keep.
    pub(super) fn stubble_binds(
        &self,
        ctx: &Ctx,
        hh: &Household,
        def: u16,
        target: Target,
    ) -> bool {
        let gleans = ctx
            .catalog
            .activities
            .get(usize::from(def))
            .filter(|a| a.behavior == crate::params::Behavior::Gather)
            .and_then(|a| a.resource)
            .filter(|&r| ctx.land_params.resources.get(r).is_some_and(|r| r.reaped));
        let pi = hh.settlement.and_then(|s| self.polity_of(s));
        match (gleans, pi) {
            (Some(r), Some(pi)) => self.own_stubble(ctx, pi, r, target),
            _ => false,
        }
    }

    /// Whether `person` gleaning by activity `def` at `target` would weigh a law keeping the
    /// stubble now: for tests.
    #[doc(hidden)]
    pub fn stubble_binds_for_tests(
        &self,
        ctx: &Ctx,
        person: PermanentId,
        def: u16,
        target: Target,
    ) -> bool {
        let (Some(p), Some(hh)) = (
            self.person(person),
            self.person(person)
                .and_then(|p| self.household(p.household)),
        ) else {
            return false;
        };
        self.stubble_now(ctx, p, hh).is_some() && self.stubble_binds(ctx, hh, def, target)
    }

    /// Whether gleaning resource `r` at `target` is gleaning polity `pi`'s own stubble (§8 19,
    /// seventh; research 08-22 §1): a field held by one of its households lies within the
    /// gathering's block, whose gleanings these are.
    pub(super) fn own_stubble(&self, ctx: &Ctx, pi: usize, r: usize, target: Target) -> bool {
        let Target::Patch(centre) = target else {
            return false;
        };
        let Some(res) = ctx.land_params.resources.get(r) else {
            return false;
        };
        let patches = &ctx.land.patches;
        let cols = patches.cols.max(1);
        let k = res.range_patches;
        let (cx, cy) = (centre % cols, centre / cols);
        ctx.land.fields.iter().any(|f| {
            let (x, y) = f.rect.centre_m();
            let fp = patches.of_point(x, y) as u32;
            let (fx, fy) = (fp % cols, fp / cols);
            fx.abs_diff(cx) <= k
                && fy.abs_diff(cy) <= k
                && self
                    .household(f.household)
                    .and_then(|x| x.settlement)
                    .is_some_and(|s| self.polities[pi].holds(Some(s)))
        })
    }

    /// What keeping the stubble for those who cannot earn weighs with `p` of `hh` now, points,
    /// when they would glean: a law of it in force in their polity (or one struck down they still
    /// keep), known to them, and they of an age to work for hire. None while someone of their
    /// household is going hungry (need over the gathering's word, as for a claim).
    pub(super) fn stubble_now(&self, ctx: &Ctx, p: &Person, hh: &Household) -> Option<f64> {
        let (now, params) = (ctx.now, ctx.params);
        if !reapers::could_earn(ctx.catalog, p.age_years(now)) {
            return None;
        }
        let pi = self.polity_of(hh.settlement?)?;
        let law = self.polities[pi].laws.iter().find(|l| {
            l.kind == PolicyKind::KeepStubble
                && (l.status == LawStatus::InForce || self.keeps_struck(ctx, p.id, l))
        })?;
        if !law.knows(p.id) {
            return None;
        }
        let hungry = hh.members.iter().any(|&m| {
            self.person(m)
                .is_some_and(|q| may_eat_reserve(q, now, params))
        });
        if hungry {
            return None;
        }
        let stance = law
            .stances
            .iter()
            .find(|r| r.person == p.id)
            .map_or(0.0, |r| r.stance.sign());
        let regard = self
            .ties
            .regard(p.id, law.sponsor, now.day_index(), &params.ties);
        let norm = self.norm_points(ctx, p.id);
        Some(crate::polity::comply_points(norm, stance, regard, 0.0, &params.polity).max(0.0))
    }

    /// What a law keeping the stubble for those who cannot earn would bring each household of
    /// settlement `settlement` a year and cost it, kcal: for tests.
    #[doc(hidden)]
    pub fn stubble_forecasts_for_tests(
        &self,
        ctx: &Ctx,
        settlement: PermanentId,
    ) -> Vec<(PermanentId, f64, f64)> {
        let Some(pi) = self.polity_of(settlement) else {
            return Vec::new();
        };
        let mut households: Vec<PermanentId> = self
            .households
            .iter()
            .filter(|(_, x)| self.polities[pi].holds(x.settlement) && !x.members.is_empty())
            .map(|(_, x)| x.id)
            .collect();
        households.sort_unstable();
        self.stubble_forecasts(ctx, &households)
    }

    /// What keeping the stubble weighs with `person` now when they would glean, points: for
    /// tests.
    #[doc(hidden)]
    pub fn stubble_points_for_tests(&self, ctx: &Ctx, person: PermanentId) -> Option<f64> {
        let p = self.person(person)?;
        let hh = self.household(p.household)?;
        self.stubble_now(ctx, p, hh)
    }
}
