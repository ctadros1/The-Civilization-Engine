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

/// Whether a rule forbids a building as it stands (M6b slice BH).
type Forbids<'a> = dyn Fn(&civ_land::Building) -> bool + 'a;

/// What the rules a household's builders keep ask of its next building (M6b slice BH).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Kept {
    /// A roof that burns keeps this far from every other building, metres.
    pub roof_within: Option<f64>,
    /// Its walls are of one of these kinds, a bit for each.
    pub walls: Option<u8>,
    /// It keeps this far from every other building, metres.
    pub gap: Option<f64>,
}

impl Kept {
    /// Whether it keeps any rule.
    pub fn any(&self) -> bool {
        self.roof_within.is_some() || self.walls.is_some() || self.gap.is_some()
    }
}

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

    /// The households among `households` that heard a building of their settlement gave way
    /// within memory (M6b slice BH): those for whom the issue `gave_way` holds.
    pub(super) fn gave_way_homes(&self, households: &[PermanentId]) -> Vec<PermanentId> {
        households
            .iter()
            .copied()
            .filter(|&h| self.household(h).is_some_and(|x| !x.gave_way.is_empty()))
            .collect()
    }

    /// What each rule for building in the catalog would bring each of `households` of polity
    /// `pi`, by template index and then household: the food a year the loss it believes averted
    /// could bring, and the food a year the work its next building would take more under the
    /// rule, spread over the years it weighs, could (kcal). Empty when no household remembers a
    /// fire or a building that gave way.
    ///
    /// The loss it believes averted is the rate at which it saw buildings of the kind the rule
    /// forbids lost (the losses it remembers over the settlement's standing buildings and the
    /// memory's years), times rebuilding its own buildings of that kind (and its next home, if it
    /// is about to build), times `rules.avoided_share`. A roof rule counts burnt thatch; a walls
    /// rule counts every fire it remembers and every building it heard gave way whose walls the
    /// rule would forbid, whatever truly brought it down (research 11-09 §1.4: remedies follow
    /// the cause people attribute); a gap rule counts burnt thatch, against its buildings that
    /// stand nearer others than the gap.
    pub(super) fn rule_forecasts(
        &self,
        ctx: &Ctx,
        pi: usize,
        households: &[PermanentId],
    ) -> RuleForecasts {
        if self.burned_homes(households).is_empty() && self.gave_way_homes(households).is_empty() {
            return Vec::new();
        }
        let settlement = self.polities[pi].settlement;
        let rp = &ctx.params.rules;
        // The buildings of the settlement that stand: what a household's remembered losses are
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
        let plots = |o: &civ_land::Building| {
            ctx.land
                .plots
                .iter()
                .find(|p| p.id == o.plot)
                .map(|p| p.rect)
        };
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
                    let burnt = x.fire_losses.iter().filter(|l| l.burnable).count() as f64;
                    // What the rule forbids, of a building of its own standing now; the losses
                    // it remembers of that kind; and what its next building would cost more.
                    let (forbidden, seen, extra): (&Forbids, f64, f64) = match rule.template {
                        RuleTemplate::RoofsUnburnt { .. } => (
                            &|b: &civ_land::Building| !crate::rules::roof_unburnt(&b.spec),
                            burnt,
                            self.redesign_hours(ctx, x, &|spec, _| self.tiled(ctx, x, spec)),
                        ),
                        RuleTemplate::WallsOfKinds { kinds } => (
                            &move |b: &civ_land::Building| {
                                kinds & (1 << self.wall_kind_of(ctx, b)) == 0
                            },
                            x.fire_losses.len() as f64
                                + x.gave_way
                                    .iter()
                                    .filter(|l| kinds & (1 << l.wall_kind.min(7)) == 0)
                                    .count() as f64,
                            self.redesign_hours(ctx, x, &|spec, rules| {
                                self.walled(ctx, x, spec, rules, kinds)
                            }),
                        ),
                        RuleTemplate::LeastGap { gap_m } => (
                            &move |b: &civ_land::Building| {
                                plots(b).is_some_and(|r| {
                                    crate::rules::near_another(
                                        &r,
                                        f64::from(gap_m),
                                        &ctx.land.buildings,
                                        Some(b.id),
                                        &plots,
                                    )
                                })
                            },
                            burnt,
                            0.0,
                        ),
                    };
                    let rate = seen / (standing * rp.memory_years.max(1e-9));
                    let at_risk: f64 = ctx
                        .land
                        .buildings
                        .iter()
                        .filter(|b| b.household == h && b.standing() && forbidden(b))
                        .filter_map(|b| self.rebuild_hours(ctx, b))
                        .sum();
                    // A household about to build reckons its next home at risk too, as its
                    // newest home took to build, and pays what keeping the rule adds to it.
                    let (next, extra) = if self.builds_soon(ctx, x) {
                        (self.newest_home_hours(ctx, x), extra)
                    } else {
                        (0.0, 0.0)
                    };
                    let averted_h = rate * (at_risk + next) * rp.avoided_share;
                    Some((
                        h,
                        averted_h * kcal,
                        extra / rp.horizon_years.max(1e-9) * kcal,
                    ))
                })
                .collect();
            out.push((k as u16, of));
        }
        out
    }

    /// The kind of building `b`'s walls (ADR-0022 §10).
    fn wall_kind_of(&self, ctx: &Ctx, b: &civ_land::Building) -> u8 {
        ctx.catalog
            .building_index(&b.spec.program)
            .and_then(|i| ctx.catalog.buildings.get(i))
            .map_or(0, |d| crate::rules::wall_kind(&b.spec, &d.rules))
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

    /// Building `spec` with a tile roof, if household `x` can fire tile (M6b slice BH); false
    /// when it cannot, or the roof is tiled already.
    fn tiled(&self, ctx: &Ctx, x: &Household, spec: &mut civ_grammar::BuildingSpec) -> bool {
        let firing = ctx.catalog.technique_index("core:technique/firing");
        if crate::rules::roof_unburnt(spec)
            || !firing.is_some_and(|t| self.household_knows(&x.members, t))
        {
            return false;
        }
        let covering = usize::from(civ_grammar::frame::frame_materials::COVERING);
        let Some(tile) = ctx
            .catalog
            .goods
            .iter()
            .find(|g| g.id == "core:good/roof_tile")
        else {
            return false;
        };
        let Some(slot) = spec.materials.get_mut(covering) else {
            return false;
        };
        *slot = tile.id.clone();
        true
    }

    /// Building `spec` (of program rules `rules`) with walls of the best of `kinds` household
    /// `x` knows how to build, on a stone footing (ADR-0022 §10: fired brick needs firing and lime
    /// burning, mud brick and rubble their techniques); false when it knows none of them, or its
    /// walls are of an allowed kind already.
    fn walled(
        &self,
        ctx: &Ctx,
        x: &Household,
        spec: &mut civ_grammar::BuildingSpec,
        rules: &civ_grammar::ProgramRules,
        kinds: u8,
    ) -> bool {
        if kinds & (1 << crate::rules::wall_kind(spec, rules)) != 0 {
            return false;
        }
        let knows = |id: &str| {
            ctx.catalog
                .technique_index(id)
                .is_some_and(|t| self.household_knows(&x.members, t))
        };
        let can = [
            (
                3u8,
                knows("core:technique/firing") && knows("core:technique/lime_burning"),
            ),
            (2, knows("core:technique/mud_brick")),
            (1, knows("core:technique/rubble_walling")),
        ];
        let Some(k) = can
            .iter()
            .find(|(k, known)| *known && kinds & (1 << k) != 0)
            .map(|(k, _)| *k)
        else {
            return false;
        };
        use civ_grammar::frame::frame_params as fp;
        spec.params[fp::WALL_KIND] = i32::from(k);
        spec.params[fp::FOOTING] = 1;
        true
    }

    /// The work its newest home, a frame building, built again as `change` makes it would take
    /// more than it took, hours: building and carrying, and making what it is built of at the
    /// household's own cost in hours (M6b slice BH). Nothing for a hut, or when `change` cannot
    /// make it so: such a household keeps a rule some other way or breaks it, which it reckons
    /// costs it nothing.
    fn redesign_hours(
        &self,
        ctx: &Ctx,
        x: &Household,
        change: &dyn Fn(&mut civ_grammar::BuildingSpec, &civ_grammar::ProgramRules) -> bool,
    ) -> f64 {
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
        if !matches!(def.rules, civ_grammar::ProgramRules::Frame(_)) {
            return 0.0;
        }
        let mut changed = b.spec.clone();
        if !change(&mut changed, &def.rules) {
            return 0.0;
        }
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
        match (cost(&changed), cost(&b.spec)) {
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

    /// What the rules in force where household `hh` lives that its eldest knows and chooses to
    /// keep ask of its next building (M6b slice BH), each weighed as a curfew's keeping is
    /// (research 09-06 §1.5: the norms they hold that the gathering binds, where they stood on it
    /// and their regard for its sponsor): the farthest reach of a roof rule, the wall kinds every
    /// walls rule allows, and the widest gap.
    pub(crate) fn rules_kept(&self, ctx: &Ctx, hh: &Household) -> Kept {
        let mut kept = Kept::default();
        let rules = self.rules_in_force(ctx, hh.settlement);
        if rules.is_empty() {
            return kept;
        }
        let Some(pi) = hh.settlement.and_then(|s| self.polity_of(s)) else {
            return kept;
        };
        let Some(person) = self.elder_of(hh.id, ctx.now, ctx.params) else {
            return kept;
        };
        for (law, t) in rules {
            let Some(l) = self.polities[pi].laws.iter().find(|l| l.id == law) else {
                continue;
            };
            if !self.keeps(ctx, person, l) {
                continue;
            }
            match t {
                RuleTemplate::RoofsUnburnt { within_m } => {
                    let m = f64::from(within_m);
                    kept.roof_within = Some(kept.roof_within.map_or(m, |k| k.max(m)));
                }
                RuleTemplate::WallsOfKinds { kinds } => {
                    kept.walls = Some(kept.walls.map_or(kinds, |k| k & kinds));
                }
                RuleTemplate::LeastGap { gap_m } => {
                    let m = f64::from(gap_m);
                    kept.gap = Some(kept.gap.map_or(m, |k| k.max(m)));
                }
            }
        }
        kept
    }

    /// The plans in force where household `hh` lives whose laws its eldest knows and chooses to
    /// keep (M8c slice CT, third part; ADR-0030 §2), weighed as a rule for building is: its
    /// builders keep its streets and public sites clear.
    pub(crate) fn plans_kept(&self, ctx: &Ctx, hh: &Household) -> Vec<&civ_land::plans::Layout> {
        let Some(pi) = hh.settlement.and_then(|s| self.polity_of(s)) else {
            return Vec::new();
        };
        let polity = &self.polities[pi];
        if polity.plans.is_empty() {
            return Vec::new();
        }
        let Some(person) = self.elder_of(hh.id, ctx.now, ctx.params) else {
            return Vec::new();
        };
        polity
            .plans_in_force()
            .filter(|(law, _)| {
                polity
                    .laws
                    .iter()
                    .find(|l| l.id == *law)
                    .is_some_and(|l| self.keeps(ctx, person, l))
            })
            .map(|(_, p)| &p.layout)
            .collect()
    }

    /// Household `household` has begun building `building` on plot `rect` (M8c slice CT, third
    /// part): each plan in force where it lives over whose ground it stands counts whether it
    /// was kept, and a building on ground the plan reserves is an encroachment, recorded.
    fn count_plan_keeping(
        &mut self,
        ctx: &Ctx,
        household: PermanentId,
        building: PermanentId,
        rect: &civ_land::RectCm,
    ) {
        let Some(pi) = self
            .household(household)
            .and_then(|x| x.settlement)
            .and_then(|s| self.polity_of(s))
        else {
            return;
        };
        if self.polities[pi].plans.is_empty() {
            return;
        }
        let found: Vec<(PermanentId, bool)> = self.polities[pi]
            .plans_in_force()
            .filter(|(_, p)| plan_bounds(&p.layout).is_some_and(|b| b.near(rect, 0)))
            .map(|(law, p)| (law, !p.layout.reserves(rect, 0)))
            .collect();
        if found.is_empty() {
            return;
        }
        let elder = self.elder_of(household, ctx.now, ctx.params);
        let day = ctx.now.day_index();
        let polity = &mut self.polities[pi];
        for (law, kept) in found {
            let Some(l) = polity.law_mut(law) else {
                continue;
            };
            if kept {
                l.compliance.complied += 1;
                continue;
            }
            if elder.is_some_and(|e| l.knows(e)) {
                l.compliance.broken += 1;
            } else {
                l.compliance.broken_unaware += 1;
            }
            polity.encroachments.push(crate::polity::Encroachment {
                day,
                law,
                household,
                building,
            });
        }
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
    /// whether whoever broke it knew of it. A roof rule binds a building within its reach of
    /// another; a walls or gap rule binds every building.
    pub(super) fn count_rule_keeping(
        &mut self,
        ctx: &Ctx,
        household: PermanentId,
        building: PermanentId,
    ) {
        let Some(x) = self.household(household) else {
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
        let settlement = x.settlement;
        self.count_plan_keeping(ctx, household, building, &rect);
        let rules = self.rules_in_force(ctx, settlement);
        if rules.is_empty() {
            return;
        }
        let Some(pi) = settlement.and_then(|s| self.polity_of(s)) else {
            return;
        };
        let elder = self.elder_of(household, ctx.now, ctx.params);
        let wall = self.wall_kind_of(ctx, b);
        let plots = |o: &civ_land::Building| {
            ctx.land
                .plots
                .iter()
                .find(|p| p.id == o.plot)
                .map(|p| p.rect)
        };
        let near = |m: f32| {
            crate::rules::near_another(
                &rect,
                f64::from(m),
                &ctx.land.buildings,
                Some(building),
                &plots,
            )
        };
        for (law, t) in rules {
            let (bound, kept) = match t {
                RuleTemplate::RoofsUnburnt { within_m } => {
                    (near(within_m), crate::rules::roof_unburnt(&b.spec))
                }
                RuleTemplate::WallsOfKinds { kinds } => (true, kinds & (1 << wall) != 0),
                RuleTemplate::LeastGap { gap_m } => (true, !near(gap_m)),
            };
            if !bound {
                continue;
            }
            let Some(l) = self.polities[pi].law_mut(law) else {
                continue;
            };
            if kept {
                l.compliance.complied += 1;
            } else if elder.is_some_and(|e| l.knows(e)) {
                l.compliance.broken += 1;
            } else {
                l.compliance.broken_unaware += 1;
            }
        }
    }

    /// The plans household `household`'s builders keep, as laid: for tests.
    #[doc(hidden)]
    pub fn plans_kept_for_tests(
        &self,
        ctx: &Ctx,
        household: PermanentId,
    ) -> Vec<civ_land::plans::Layout> {
        self.household(household).map_or_else(Vec::new, |x| {
            self.plans_kept(ctx, x).into_iter().cloned().collect()
        })
    }

    /// What the rules household `household`'s builders keep ask of its next building: for tests.
    #[doc(hidden)]
    pub fn rules_kept_for_tests(&self, ctx: &Ctx, household: PermanentId) -> Kept {
        self.household(household)
            .map_or_else(Kept::default, |x| self.rules_kept(ctx, x))
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
            .ok()
            .map(|n| n.spec)
    }

    /// The home household `household` would lay out today, beside its settlement's hearth, as if
    /// it had none, and the parcel of a plan it would stand on, if a keeper gives it one: for
    /// tests.
    #[doc(hidden)]
    pub fn first_home_parcel_for_tests(
        &self,
        ctx: &Ctx,
        household: PermanentId,
    ) -> Option<(civ_grammar::BuildingSpec, Option<civ_land::Allotment>)> {
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
            .ok()
            .map(|n| (n.spec, n.parcel))
    }

    /// The parcel a plan in force would give household `household` now: for tests.
    #[doc(hidden)]
    pub fn parcel_offer_for_tests(
        &self,
        ctx: &Ctx,
        household: PermanentId,
    ) -> Option<super::ParcelOffer> {
        self.household(household)
            .and_then(|x| self.parcel_offer(ctx, x))
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

/// The ground a plan's layout covers: the least rectangle about its streets, sites and parcels.
fn plan_bounds(layout: &civ_land::plans::Layout) -> Option<civ_land::RectCm> {
    let rects = layout
        .streets
        .iter()
        .chain(layout.sites.iter().map(|s| &s.rect))
        .chain(layout.parcels.iter().map(|p| &p.rect));
    rects.fold(None, |acc: Option<civ_land::RectCm>, r| {
        Some(acc.map_or(*r, |a| {
            let (x0, y0) = (a.x.min(r.x), a.y.min(r.y));
            let (x1, y1) = ((a.x + a.w).max(r.x + r.w), (a.y + a.h).max(r.y + r.h));
            civ_land::RectCm {
                x: x0,
                y: y0,
                w: x1 - x0,
                h: y1 - y0,
            }
        }))
    })
}
