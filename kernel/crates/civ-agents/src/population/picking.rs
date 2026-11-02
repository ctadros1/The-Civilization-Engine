//! Picking salvage (backlog C's second slice; ADR-0044 §2; brief backlog-c-salvage-sewage §1, rules
//! S2-S4; research 12-10 §1.6, §5.6, 12-02 §5.5): a member may sort its own household's heap or its
//! polity's open tip by hand, at the people profile's `sort_kg_h` of what lies there an hour, and
//! carry home the good the activity names: the ash of what burned, or the scrap a tool leaves. What
//! a session sorts yields the good's share of it, so the hours of a kilogram are `1 / (sort_kg_h ×
//! share)` and the walk there and back over a load; that is picking's own cost of the good
//! (`value::picked_costs`), which enters wants and the market as gathering's does. No price,
//! recovery rate or salvage share is authored. A household picks only what its work wants (the
//! materials of a tool it lacks, of what it builds), and not where a neighbour's posted terms give
//! the good for fewer of its own hours: a household whose food costs it many hours finds buying
//! dear and picks, one whose hour brings much food buys. Kilograms leave the heap or the tip
//! exactly as they are carried; what the heap or the tip holds of a disease stays there.

use super::*;
use crate::polity::{LawStatus, PolicyKind};
use crate::value::{self, PickSite};

/// How far from a heap's or a tip's place, cells, a picker may stand and sort it.
const AT_SITE_CELLS: f64 = 2.0;

/// A place a household may pick over (ADR-0044 §2): one of its polity's open tips, by id, or its
/// own heap (`tip` none); where it lies; and what lies there.
#[derive(Clone, Debug, PartialEq)]
pub struct PickPlace {
    pub tip: Option<PermanentId>,
    pub at: (f32, f32),
    pub site: PickSite,
}

impl Population {
    /// The places household `hh` may pick over at `now` (ADR-0044 §2): its own heap where it lies
    /// (at home, or where a law clearing middens had it carried), and each tip its polity keeps
    /// while the law that opened it is in force; each with its walk from home in a straight line
    /// on level ground, what lies there in all and the goods it holds whole (its discards and its
    /// ash, as the ash good). Picking another household's heap waits (ADR-0044 §7).
    pub fn pick_places(
        &self,
        catalog: &crate::params::Catalog,
        params: &PeopleParams,
        hh: &Household,
        now: SimTime,
    ) -> Vec<PickPlace> {
        let goods = &catalog.goods;
        let ash = goods.iter().position(|d| d.id == make::ASH);
        let walk_ms = params.nav.tobler_ms(0.0).max(1e-6);
        let walk_h = |at: (f32, f32)| {
            f64::from(at.0 - hh.home.0).hypot(f64::from(at.1 - hh.home.1)) / walk_ms / 3600.0
        };
        let whole = |ash_kg: f64, discards: &[(u16, f64)]| -> Vec<(usize, f64)> {
            let mut out: Vec<(usize, f64)> = discards
                .iter()
                .filter(|&&(_, kg)| kg > 0.0)
                .map(|&(g, kg)| (usize::from(g), kg))
                .collect();
            if let Some(a) = ash
                && ash_kg > 0.0
            {
                out.push((a, ash_kg));
            }
            out
        };
        let mut out = Vec::new();
        let day = now.day_index();
        let at = match catalog.policies.iter().find_map(|d| d.middens) {
            Some(def) if Self::midden_carried(hh, &def, day) => hh.midden.site,
            _ => hh.home,
        };
        let decaying = hh.midden.at_time(now, hh.members.len(), &params.midden);
        out.push(PickPlace {
            tip: None,
            at,
            site: PickSite {
                walk_h: walk_h(at),
                kg: decaying + hh.midden.inert_kg(),
                goods: whole(hh.midden.ash_kg, &hh.midden.discards),
            },
        });
        if let Some(pi) = hh.settlement.and_then(|s| self.polity_of(s)) {
            let p = &self.polities[pi];
            for t in p.tips.iter().filter(|t| Self::tip_open(p, t.law)) {
                out.push(PickPlace {
                    tip: Some(t.id),
                    at: t.place,
                    site: PickSite {
                        walk_h: walk_h(t.place),
                        kg: t.kg + t.ash_kg + crate::person::discards_kg(&t.discards),
                        goods: whole(t.ash_kg, &t.discards),
                    },
                });
            }
        }
        out
    }

    /// Whether a polity's tip opened by law `law` is open: that law is in force.
    fn tip_open(p: &crate::polity::Polity, law: PermanentId) -> bool {
        p.laws.iter().any(|l| {
            l.id == law && l.status == LawStatus::InForce && l.kind == PolicyKind::RefuseToTip
        })
    }

    /// Hours per kilogram of picking each good for household `hh` at `now`, at the cheapest place
    /// it may pick over (ADR-0044 §2): for its own costs, beside gathering and growing.
    pub(crate) fn picked_costs_of(
        &self,
        catalog: &crate::params::Catalog,
        params: &PeopleParams,
        hh: &Household,
        now: SimTime,
    ) -> Vec<Option<f64>> {
        let sites: Vec<PickSite> = self
            .pick_places(catalog, params, hh, now)
            .into_iter()
            .map(|p| p.site)
            .collect();
        value::picked_costs(
            &sites,
            catalog.goods.len(),
            params.midden.sort_kg_h,
            params.household.carry_kg,
        )
    }

    /// Whether good `g` lies at a place household `hh` may pick over, `amount` kilograms of it or
    /// more: what a way that takes it may count on (ADR-0044 §2).
    pub(crate) fn lies_to_pick(&self, ctx: &Ctx, hh: &Household, g: usize, amount: f64) -> bool {
        if !make::is_salvage(&ctx.catalog.goods, g) {
            return false;
        }
        self.pick_places(ctx.catalog, ctx.params, hh, ctx.now)
            .iter()
            .any(|p| {
                p.site
                    .goods
                    .iter()
                    .any(|&(x, kg)| x == g && kg >= amount - 1e-9)
            })
    }

    /// Hours per kilogram of digging good `g` for household `hh`, which its own costs leave out: at
    /// the deposit of it its settlement knows that gives it cheapest, a capable adult's hours of
    /// digging a kilogram there and the walk there and back over a load, walked in a straight line
    /// on level ground. None where it knows no deposit of it. What picking competes with where a
    /// way to a material takes salvage in place of a dug good (ADR-0044 §2: ash for clay).
    pub(crate) fn dug_h(&self, ctx: &Ctx, hh: &Household, g: usize) -> Option<f64> {
        let settlement = hh.settlement?;
        let walk_ms = ctx.params.nav.tobler_ms(0.0).max(1e-6);
        let carry = ctx.params.household.carry_kg.max(1e-9);
        ctx.land
            .deposits
            .iter()
            .filter(|d| {
                usize::from(d.body.good) == g
                    && d.left_kg() > 0.0
                    && self.knows_deposit(settlement, d.id)
            })
            .filter_map(|d| {
                let rates =
                    super::digging::rates(ctx.land_params, &d.body, ctx.params.digging.h_per_m3);
                let kg_h = super::digging::dig_kg_per_hour(&d.body, rates);
                if kg_h <= 0.0 {
                    return None;
                }
                let at = (d.body.at_cm.0 as f64 / 100.0, d.body.at_cm.1 as f64 / 100.0);
                let metres = (at.0 - f64::from(hh.home.0)).hypot(at.1 - f64::from(hh.home.1));
                Some(1.0 / kg_h + 2.0 * metres / walk_ms / 3600.0 / carry)
            })
            .reduce(f64::min)
    }

    /// The fewest hours of household `hh`'s own work a kilogram of good `g` costs at the terms a
    /// neighbour of its settlement posts now (a household's or an open workshop's, not its own):
    /// the price in each good taken in payment at its own cost of that good. `None` where nobody
    /// offers it there.
    pub(crate) fn bought_h(&self, ctx: &Ctx, hh: &Household, g: usize) -> Option<f64> {
        let s = hh.settlement?;
        let rows: Vec<crate::market::Offer> = self
            .households
            .iter()
            .map(|(_, x)| x)
            .filter(|x| x.settlement == Some(s) && x.id != hh.id)
            .flat_map(|x| x.offers.iter().copied())
            .chain(
                self.firms
                    .iter()
                    .filter(|f| f.is_open() && f.settlement == Some(s) && f.owner != hh.id)
                    .flat_map(|f| f.offers.iter().copied()),
            )
            .filter(|o| usize::from(o.good) == g && o.units > 0.0 && o.price > 0.0)
            .collect();
        if rows.is_empty() {
            return None;
        }
        let costs = self.own_costs_of(ctx, hh);
        rows.iter()
            .filter_map(|o| {
                let c = costs.get(usize::from(o.payment)).copied().flatten()?;
                Some(f64::from(o.price) * c)
            })
            .reduce(f64::min)
    }

    /// Where a member of household `hh` would pick good `g` now (ADR-0044 §2, S2-S3): of the
    /// places it may pick over that hold some of it within `max_walk_min` by its walking field
    /// `reach`, the one that brings most for the walk over a session of `session_h` hours; with
    /// the walk there, minutes, and what a capable adult sorts out of it an hour, kilograms. None
    /// where none lies within reach, or where a neighbour's terms give a kilogram for fewer of its
    /// hours than picking it there takes ([`Population::bought_h`]): picking pays where its hours
    /// are cheap or the good dear (S3).
    pub(crate) fn pick_place(
        &self,
        ctx: &Ctx,
        hh: &Household,
        g: usize,
        reach: &TravelField,
        max_walk_min: f64,
        session_h: f64,
    ) -> Option<(PickPlace, f64, f64)> {
        let (sort, carry) = (ctx.params.midden.sort_kg_h, ctx.params.household.carry_kg);
        let mut best: Option<(f64, PickPlace, f64, f64)> = None;
        for p in self.pick_places(ctx.catalog, ctx.params, hh, ctx.now) {
            let kg: f64 = p
                .site
                .goods
                .iter()
                .filter(|&&(x, _)| x == g)
                .map(|&(_, kg)| kg)
                .sum();
            if kg <= 0.0 || p.site.kg <= 0.0 {
                continue;
            }
            let Some(secs) = reach.seconds_to(cell_of(ctx.map, p.at)) else {
                continue;
            };
            let walk = f64::from(secs) / 60.0;
            if walk > max_walk_min {
                continue;
            }
            let kg_per_hour = sort * (kg / p.site.kg).min(1.0);
            let value = kg_per_hour * session_h / (session_h + 2.0 * walk / 60.0);
            if best.as_ref().is_none_or(|b| value > b.0) {
                best = Some((value, p, walk, kg_per_hour));
            }
        }
        let (_, place, walk, kg_per_hour) = best?;
        let kg: f64 = place
            .site
            .goods
            .iter()
            .filter(|&&(x, _)| x == g)
            .map(|&(_, kg)| kg)
            .sum();
        let picked_h = 1.0 / kg_per_hour.max(1e-9) + 2.0 * walk / 60.0 / carry.min(kg).max(1e-9);
        if self.bought_h(ctx, hh, g).is_some_and(|b| b < picked_h) {
            return None;
        }
        Some((place, walk, kg_per_hour))
    }

    /// Household `household`'s member, standing at `pos`, sorts `sorted_kg` of the heap or tip it
    /// stands at and takes good `g` out of it (ADR-0044 §2): the good's share of what was sorted,
    /// at most a load and what lies there, out of the place's discards or its ash. What the place
    /// holds of a disease stays there. Returns the kilograms taken; none where it stands at no
    /// place it may pick over.
    pub(super) fn pick_at(
        &mut self,
        ctx: &Ctx,
        household: PermanentId,
        pos: (f32, f32),
        g: usize,
        sorted_kg: f64,
    ) -> f64 {
        let Some(x) = self.household(household) else {
            return 0.0;
        };
        let ash = ctx.catalog.goods.iter().position(|d| d.id == make::ASH) == Some(g);
        let reach = AT_SITE_CELLS * f64::from(ctx.map.cell_size_m);
        let near = |at: (f32, f32)| f64::from(at.0 - pos.0).hypot(f64::from(at.1 - pos.1));
        let place = self
            .pick_places(ctx.catalog, ctx.params, x, ctx.now)
            .into_iter()
            .filter(|p| near(p.at) <= reach)
            .min_by(|a, b| near(a.at).total_cmp(&near(b.at)));
        let Some(place) = place else {
            return 0.0;
        };
        let carry = ctx.params.household.carry_kg;
        let take = |total: f64, held: f64| {
            if total <= 0.0 || held <= 0.0 {
                return 0.0;
            }
            (sorted_kg.max(0.0) * held / total).min(carry).min(held)
        };
        match place.tip {
            Some(tip) => {
                let Some(t) = self
                    .polities
                    .iter_mut()
                    .flat_map(|p| p.tips.iter_mut())
                    .find(|t| t.id == tip)
                else {
                    return 0.0;
                };
                let total = t.kg + t.ash_kg + crate::person::discards_kg(&t.discards);
                if ash {
                    let kg = take(total, t.ash_kg);
                    t.ash_kg -= kg;
                    kg
                } else {
                    let kg = take(total, crate::person::discard_of(&t.discards, g));
                    crate::person::take_discard(&mut t.discards, g, kg)
                }
            }
            None => {
                let Some(x) = self.household_mut(household) else {
                    return 0.0;
                };
                let members = x.members.len();
                x.midden.settle(ctx.now, members, &ctx.params.midden);
                let total = x.midden.kg + x.midden.inert_kg();
                let kg = if ash {
                    let kg = take(total, x.midden.ash_kg);
                    x.midden.ash_kg -= kg;
                    kg
                } else {
                    let kg = take(total, crate::person::discard_of(&x.midden.discards, g));
                    crate::person::take_discard(&mut x.midden.discards, g, kg)
                };
                self.changed(household);
                kg
            }
        }
    }

    /// Person `who` works `minutes` of pick activity `def` where they stand (ADR-0044 §2): they
    /// sort what a capable adult sorts in that time at their age's capacity and the activity's
    /// rate, and carry what they take out of it. Returns the kilograms taken.
    pub(super) fn pick_work(
        &mut self,
        ctx: &Ctx,
        h: Handle<Person>,
        def: &ActivityDef,
        minutes: u32,
    ) -> f64 {
        let Some(g) = def.picks else {
            return 0.0;
        };
        let params = ctx.params;
        let Some((household, pos, eff)) = self.people.get(h).map(|p| {
            (
                p.household,
                p.pos,
                interpolate(&params.capacity_by_age, p.age_years(ctx.now)) * def.rate,
            )
        }) else {
            return 0.0;
        };
        let sorted = params.midden.sort_kg_h * f64::from(minutes) / 60.0 * eff;
        let kg = self.pick_at(ctx, household, pos, g, sorted);
        if kg > 0.0
            && let Some(p) = self.people.get_mut(h)
        {
            p.carrying.good = Some(g as u16);
            p.carrying.kg = kg as f32;
        }
        kg
    }

    /// Person `who` stands at `at` and works `minutes` of the pick activity for good `g`, as its
    /// work step does: for tests. The kilograms they carry from it.
    #[doc(hidden)]
    pub fn pick_for_tests(
        &mut self,
        ctx: &Ctx,
        who: PermanentId,
        at: (f32, f32),
        g: usize,
        minutes: u32,
    ) -> f64 {
        let Some(def) = ctx
            .catalog
            .activities
            .iter()
            .find(|a| a.behavior == Behavior::Pick && a.picks == Some(g))
            .cloned()
        else {
            return 0.0;
        };
        let Some(h) = self
            .people
            .iter()
            .find(|(_, p)| p.id == who)
            .map(|(h, _)| h)
        else {
            return 0.0;
        };
        if let Some(p) = self.people.get_mut(h) {
            p.pos = at;
        }
        self.pick_work(ctx, h, &def, minutes)
    }

    /// Where a member of household `household` would pick good `g` now, as the decision weighs
    /// it, and what a kilogram costs it there and at a neighbour's terms: for tests.
    #[doc(hidden)]
    pub fn pick_place_for_tests(
        &self,
        ctx: &Ctx,
        household: PermanentId,
        g: usize,
    ) -> Option<(PickPlace, f64, f64)> {
        let hh = self.household(household)?;
        let reach = self.homes.get(&hh.settlement.unwrap_or(hh.id))?;
        self.pick_place(ctx, hh, g, &reach.reach, f64::from(u16::MAX), 4.0)
    }

    /// The fewest hours of household `household`'s own work a kilogram of good `g` costs at a
    /// neighbour's terms now: for tests.
    #[doc(hidden)]
    pub fn bought_h_for_tests(&self, ctx: &Ctx, household: PermanentId, g: usize) -> Option<f64> {
        self.bought_h(ctx, self.household(household)?, g)
    }

    /// The recipe household `household` would make good `g` by toward a tool it lacks, as its
    /// wants reckon it: for tests.
    #[doc(hidden)]
    pub fn material_way_for_tests(
        &self,
        ctx: &Ctx,
        household: PermanentId,
        g: usize,
    ) -> Option<String> {
        let hh = self.household(household)?;
        let stores = stores_now(hh, ctx.now, ctx.params, &ctx.catalog.goods);
        let can_work = |r: &crate::params::RecipeDef| self.can_work_way(ctx, hh, &stores, r);
        let costs = || self.own_costs_of(ctx, hh);
        let dug = |x: usize| self.dug_h(ctx, hh, x);
        super::market::way_to_material(ctx.catalog, g, &can_work, &costs, &dug)
            .map(|(r, _)| r.id.clone())
    }

    /// Hours per kilogram of digging good `g` for household `household`: for tests.
    #[doc(hidden)]
    pub fn dug_h_for_tests(&self, ctx: &Ctx, household: PermanentId, g: usize) -> Option<f64> {
        self.dug_h(ctx, self.household(household)?, g)
    }

    /// What an hour of household `household`'s work brings it in food, kcal: for tests.
    #[doc(hidden)]
    pub fn hour_kcal_for_tests(&self, ctx: &Ctx, household: PermanentId) -> f64 {
        self.household(household)
            .map_or(0.0, |x| self.hour_kcal(ctx, x))
    }
}
