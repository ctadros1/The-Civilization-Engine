//! Rules for building (M6b slice BH; ADR-0022 §12): what a household believes a rule would bring
//! it, whether its builders keep one in force, and what each building begun under one did.
//!
//! A household that lost a building to fire, or whose people saw one burn, holds the issue
//! `burned`; someone of the settlement who knows the technique a rule needs may propose it. A
//! household weighs it by the loss it believes averted, from the fires its people remember among
//! the buildings they know of, against what its next building would cost more under it. A
//! household that builds nothing soon pays nothing yet (research 11-09 §1.7: grandfathering
//! lowers opposition). A rule binds only buildings begun once it is in force, and keeping it is
//! the builders' choice, weighed as a curfew's keeping is, with no sanction.

use super::*;
use crate::polity::{Law, PolicyKind, RuleTemplate};

/// What each rule for building would bring each household: by template index, then (household,
/// food a year it believes averted, food a year its next building would cost more), kcal.
pub(super) type RuleForecasts = Vec<(u16, Vec<(PermanentId, f64, f64)>)>;

impl Population {
    /// The households among `households` that remember a building lost to fire or seen burn
    /// (M6b slice BH): those for whom the issue `burned` holds.
    pub(super) fn burned_homes(&self, households: &[PermanentId]) -> Vec<PermanentId> {
        households
            .iter()
            .copied()
            .filter(|&h| self.household(h).is_some_and(|x| !x.fire_losses.is_empty()))
            .collect()
    }

    /// What each rule for building in the catalog would bring each of `households` of polity
    /// `pi`, by template index and then household: the food a year the fire loss it believes
    /// averted could bring, and the food a year the work its next building would take more under
    /// the rule, spread over the years it weighs, could (kcal). Empty when no household remembers
    /// a fire.
    pub(super) fn rule_forecasts(
        &self,
        ctx: &Ctx,
        pi: usize,
        households: &[PermanentId],
    ) -> RuleForecasts {
        if self.burned_homes(households).is_empty() {
            return Vec::new();
        }
        let settlement = self.polities[pi].settlement;
        let rp = &ctx.params.rules;
        // The buildings of the settlement that stand: what a household's remembered fires are
        // reckoned among.
        let standing = ctx
            .land
            .buildings
            .iter()
            .filter(|b| {
                b.standing()
                    && self
                        .household(b.household)
                        .is_some_and(|x| x.settlement == Some(settlement))
            })
            .count()
            .max(1) as f64;
        let mut out = Vec::new();
        for (k, def) in ctx.catalog.policies.iter().enumerate() {
            let Some(rule) = def.rule.as_ref() else {
                continue;
            };
            let of = households
                .iter()
                .filter_map(|&h| {
                    let x = self.household(h)?;
                    let kcal = self.hour_kcal(ctx, x);
                    let (averted_h, extra_h) = match rule.template {
                        RuleTemplate::RoofsUnburnt { .. } => {
                            let seen = x.fire_losses.iter().filter(|l| l.burnable).count() as f64;
                            let rate = seen / (standing * rp.memory_years.max(1e-9));
                            let at_risk: f64 = ctx
                                .land
                                .buildings
                                .iter()
                                .filter(|b| {
                                    b.household == h
                                        && b.standing()
                                        && !crate::rules::roof_unburnt(&b.spec)
                                })
                                .filter_map(|b| self.rebuild_hours(ctx, b))
                                .sum::<f64>();
                            // A household about to build reckons its next home at risk too, as
                            // its newest home took to build, and what tiling it would add.
                            let (next, extra) = if self.builds_soon(ctx, x) {
                                (self.newest_home_hours(ctx, x), self.tiling_hours(ctx, x))
                            } else {
                                (0.0, 0.0)
                            };
                            (rate * (at_risk + next) * rp.avoided_share, extra)
                        }
                    };
                    Some((
                        h,
                        averted_h * kcal,
                        extra_h / rp.horizon_years.max(1e-9) * kcal,
                    ))
                })
                .collect();
            out.push((k as u16, of));
        }
        out
    }

    /// The work building `b` again would take, hours.
    fn rebuild_hours(&self, ctx: &Ctx, b: &civ_land::Building) -> Option<f64> {
        let def = ctx
            .catalog
            .building_index(&b.spec.program)
            .and_then(|i| ctx.catalog.buildings.get(i))?;
        build::building_hours(&b.spec, def, ctx.params.household.carry_kg)
    }

    /// The work its newest home took to build, hours: what a household about to build reckons
    /// its next one will (M6b slice BH).
    fn newest_home_hours(&self, ctx: &Ctx, x: &Household) -> f64 {
        ctx.land
            .buildings
            .iter()
            .filter(|b| b.household == x.id && ctx.catalog.is_dwelling(&b.spec.program))
            .max_by_key(|b| (b.started, b.id))
            .and_then(|b| self.rebuild_hours(ctx, b))
            .unwrap_or(0.0)
    }

    /// Whether household `x` is about to build (M6b slice BH): it has no roof enough for its
    /// people, or its home has given way in part or whole.
    fn builds_soon(&self, ctx: &Ctx, x: &Household) -> bool {
        !x.sheltered
            || ctx.land.buildings.iter().any(|b| {
                b.household == x.id
                    && ctx.catalog.is_dwelling(&b.spec.program)
                    && b.state != civ_land::BuildingState::Standing
            })
    }

    /// The work its newest home built again with a tile roof would take more than it took,
    /// hours: building and carrying, and making what it is built of at the household's own cost
    /// in hours. Nothing for a hut, whose roof cannot be tiled, or a household that cannot make
    /// tile: either keeps such a rule by building apart from others, which it reckons costs it
    /// nothing.
    fn tiling_hours(&self, ctx: &Ctx, x: &Household) -> f64 {
        let Some(b) = ctx
            .land
            .buildings
            .iter()
            .filter(|b| b.household == x.id && ctx.catalog.is_dwelling(&b.spec.program))
            .max_by_key(|b| (b.started, b.id))
        else {
            return 0.0;
        };
        let Some(def) = ctx
            .catalog
            .building_index(&b.spec.program)
            .and_then(|i| ctx.catalog.buildings.get(i))
        else {
            return 0.0;
        };
        let covering = usize::from(civ_grammar::frame::frame_materials::COVERING);
        if !matches!(def.rules, civ_grammar::ProgramRules::Frame(_))
            || crate::rules::roof_unburnt(&b.spec)
        {
            return 0.0;
        }
        let Some(tile) = ctx
            .catalog
            .goods
            .iter()
            .find(|g| g.id == "core:good/roof_tile")
        else {
            return 0.0;
        };
        let mut tiled = b.spec.clone();
        let Some(slot) = tiled.materials.get_mut(covering) else {
            return 0.0;
        };
        *slot = tile.id.clone();
        let costs = self.own_costs_of(ctx, x);
        let cost = |spec: &civ_grammar::BuildingSpec| -> Option<f64> {
            let e = civ_grammar::expand(spec, &def.rules).ok()?;
            let kg = e.total_materials_kg();
            let mut h = e.total_labour_h()
                + kg.iter().sum::<f64>() / ctx.params.household.carry_kg.max(1e-6)
                    * build::HAUL_H_PER_LOAD;
            for (slot, kg) in kg.iter().enumerate() {
                if *kg <= 0.0 {
                    continue;
                }
                let g = spec
                    .materials
                    .get(slot)
                    .and_then(|id| ctx.catalog.good_index(id))?;
                h += kg * costs.get(g).copied().flatten()?;
            }
            Some(h)
        };
        match (cost(&tiled), cost(&b.spec)) {
            (Some(with), Some(without)) => (with - without).max(0.0),
            _ => 0.0,
        }
    }

    /// The rules for building in force in settlement `settlement` (M6b slice BH), with their
    /// templates, in the order passed.
    pub(crate) fn rules_in_force(
        &self,
        ctx: &Ctx,
        settlement: Option<PermanentId>,
    ) -> Vec<(PermanentId, RuleTemplate)> {
        let Some(pi) = settlement.and_then(|s| self.polity_of(s)) else {
            return Vec::new();
        };
        self.polities[pi]
            .in_force(&ctx.catalog.policies, PolicyKind::BuildingRule)
            .filter_map(|l| {
                let d = ctx.catalog.policies.get(usize::from(l.policy))?;
                Some((l.id, d.rule.as_ref()?.template))
            })
            .collect()
    }

    /// The least distance from others a roof that burns keeps under the rules household `hh`'s
    /// builders keep (M6b slice BH), metres: the farthest a rule in force where it lives binds,
    /// of those its eldest knows and chooses to keep, as a curfew's keeping is weighed (research
    /// 09-06 §1.5: the norms they hold that the gathering binds, where they stood on it and their
    /// regard for its sponsor). None when it keeps none.
    pub(crate) fn roof_rule_kept(&self, ctx: &Ctx, hh: &Household) -> Option<f64> {
        let rules = self.rules_in_force(ctx, hh.settlement);
        if rules.is_empty() {
            return None;
        }
        let pi = hh.settlement.and_then(|s| self.polity_of(s))?;
        let person = self.elder_of(hh.id, ctx.now, ctx.params)?;
        rules
            .into_iter()
            .filter_map(|(law, t)| {
                let RuleTemplate::RoofsUnburnt { within_m } = t;
                let l = self.polities[pi].laws.iter().find(|l| l.id == law)?;
                self.keeps(ctx, person, l).then_some(f64::from(within_m))
            })
            .max_by(f64::total_cmp)
    }

    /// Whether `person`, knowing law `l`, chooses to keep it (M6b slice BH).
    fn keeps(&self, ctx: &Ctx, person: PermanentId, l: &Law) -> bool {
        if !l.knows(person) {
            return false;
        }
        let stance = l
            .stances
            .iter()
            .find(|r| r.person == person)
            .map_or(0.0, |r| r.stance.sign());
        let regard = self
            .ties
            .regard(person, l.sponsor, ctx.now.day_index(), &ctx.params.ties);
        let norm = self.norm_points(ctx, person);
        crate::polity::comply_points(norm, stance, regard, 0.0, &ctx.params.polity) > 0.0
    }

    /// Household `household` has begun building `building` (M6b slice BH): for each rule for
    /// building in force where it lives that binds it, the law counts whether it was kept, and
    /// whether whoever broke it knew of it.
    pub(super) fn count_rule_keeping(
        &mut self,
        ctx: &Ctx,
        household: PermanentId,
        building: PermanentId,
    ) {
        let Some(x) = self.household(household) else {
            return;
        };
        let rules = self.rules_in_force(ctx, x.settlement);
        if rules.is_empty() {
            return;
        }
        let Some(pi) = x.settlement.and_then(|s| self.polity_of(s)) else {
            return;
        };
        let Some(b) = ctx.land.buildings.iter().find(|b| b.id == building) else {
            return;
        };
        let Some(rect) = ctx
            .land
            .plots
            .iter()
            .find(|p| p.id == b.plot)
            .map(|p| p.rect)
        else {
            return;
        };
        let elder = self.elder_of(household, ctx.now, ctx.params);
        let unburnt = crate::rules::roof_unburnt(&b.spec);
        let plots = |o: &civ_land::Building| {
            ctx.land
                .plots
                .iter()
                .find(|p| p.id == o.plot)
                .map(|p| p.rect)
        };
        for (law, t) in rules {
            let RuleTemplate::RoofsUnburnt { within_m } = t;
            let bound = crate::rules::near_another(
                &rect,
                f64::from(within_m),
                &ctx.land.buildings,
                Some(building),
                &plots,
            );
            if !bound {
                continue;
            }
            let Some(l) = self.polities[pi].law_mut(law) else {
                continue;
            };
            if unburnt {
                l.compliance.complied += 1;
            } else if elder.is_some_and(|e| l.knows(e)) {
                l.compliance.broken += 1;
            } else {
                l.compliance.broken_unaware += 1;
            }
        }
    }

    /// The distance household `household`'s builders keep a roof that burns from others under
    /// the rules they keep, if any: for tests.
    #[doc(hidden)]
    pub fn roof_rule_kept_for_tests(&self, ctx: &Ctx, household: PermanentId) -> Option<f64> {
        self.roof_rule_kept(ctx, self.household(household)?)
    }

    /// The first home household `household` would lay out today, beside its settlement's
    /// hearth, as if it had none: for tests.
    #[doc(hidden)]
    pub fn first_home_for_tests(
        &self,
        ctx: &Ctx,
        household: PermanentId,
    ) -> Option<civ_grammar::BuildingSpec> {
        let hh = self.household(household)?.clone();
        let hearth = hh.settlement.and_then(|s| {
            ctx.land
                .settlements
                .iter()
                .find(|x| x.id == s)
                .map(|x| x.hearth_m)
        });
        let labour = self.labour_per_day(&hh.members, ctx.now, ctx.params);
        self.new_building(ctx, &hh, hearth, hh.id, labour, None)
            .map(|n| n.spec)
    }

    /// Counts whether building `building` household `household` began keeps the rules in force
    /// that bind it: for tests.
    #[doc(hidden)]
    pub fn count_rule_keeping_for_tests(
        &mut self,
        ctx: &Ctx,
        household: PermanentId,
        building: PermanentId,
    ) {
        self.count_rule_keeping(ctx, household, building);
    }
}
