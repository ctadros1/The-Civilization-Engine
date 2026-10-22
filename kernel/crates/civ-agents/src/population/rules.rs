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
    /// A home where fire is worked keeps this far from other households' homes, metres (M8c
    /// slice CW).
    pub fire_from: Option<f64>,
    /// A home, and the midden beside it, keeps this far from every open well, metres (M8c slice
    /// CW).
    pub midden_from: Option<f64>,
    /// The most storeys it rises with walls of each kind (M8d slice DB).
    pub storeys: Option<[u8; 4]>,
    /// Its mass walls above a storey are tied at each floor (M8d slice DB).
    pub tied: bool,
}

impl Kept {
    /// Whether it keeps any rule.
    pub fn any(&self) -> bool {
        self.roof_within.is_some()
            || self.walls.is_some()
            || self.gap.is_some()
            || self.fire_from.is_some()
            || self.midden_from.is_some()
            || self.storeys.is_some()
            || self.tied
    }

    /// Whether a building of `spec`, its walls of kind `wall_kind`, breaks a height or ties rule
    /// kept (M8d slice DB).
    pub fn too_tall(&self, spec: &civ_grammar::BuildingSpec, wall_kind: u8) -> bool {
        crate::rules::too_tall(
            spec.storeys,
            wall_kind,
            crate::rules::tied(spec),
            self.storeys,
            self.tied,
        )
    }

    /// The most storeys a building with walls of kind `wall_kind` rises under the height and
    /// ties rules kept, if it is not tied at each floor (M8d slice DB).
    pub fn most_storeys(&self, wall_kind: u8, tied: bool) -> u8 {
        let most = self
            .storeys
            .map_or(u8::MAX, |m| m[usize::from(wall_kind.min(3))]);
        if self.tied && wall_kind > 0 && !tied {
            most.min(1)
        } else {
            most
        }
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

    /// The cases of sickness at household `household`'s home its members have heard of (M8c
    /// slice CW): claims of kind `Sickness` about it.
    pub(super) fn sick_cases(&self, household: PermanentId) -> usize {
        let Some(x) = self.household(household) else {
            return 0;
        };
        let mut claims: Vec<u32> = x
            .members
            .iter()
            .flat_map(|&m| self.word.heard_by(m).iter().map(|h| h.claim))
            .filter(|&c| {
                self.word.claim(c).is_some_and(|c| {
                    c.kind == crate::word::ClaimKind::Sickness && c.subject == Some(household)
                })
            })
            .collect();
        claims.sort_unstable();
        claims.dedup();
        claims.len()
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
        if self.burned_homes(households).is_empty()
            && self.gave_way_homes(households).is_empty()
            && !households.iter().any(|&h| self.sick_cases(h) > 0)
        {
            return Vec::new();
        }
        // What a case of a waterborne sickness takes from a household's work, hours: the mean
        // days the ill are ill, of those with symptoms, at a day's work (M8c slice CW).
        let case_h = {
            let water: Vec<_> = ctx
                .catalog
                .diseases
                .iter()
                .filter(|d| d.routes.contains(&crate::params::DiseaseRoute::Water))
                .collect();
            water.iter().map(|d| d.ill.mean).sum::<f64>() / water.len().max(1) as f64
                * ctx.params.farm.work_hours_per_day
        };
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
                    let works_fire = self.works_fire_at_home(ctx, x);
                    // A rule keeping homes and their middens from wells (M8c slice CW) averts
                    // sickness, not the loss of a building: the cases of sickness at home the
                    // household has heard of a year, at what a case takes from its work, if its
                    // home (or the next it builds) stands within reach of an open well.
                    if let RuleTemplate::MiddenFromWells { from_m } = rule.template {
                        let near_well = ctx
                            .land
                            .buildings
                            .iter()
                            .filter(|b| b.household == h && b.standing())
                            .filter(|b| ctx.catalog.is_dwelling(&b.spec.program))
                            .filter_map(&plots)
                            .any(|r| {
                                crate::rules::near_open_well(&r, f64::from(from_m), &ctx.land.wells)
                            });
                        let homes = f64::from(u8::from(near_well))
                            + f64::from(u8::from(self.builds_soon(ctx, x)));
                        let cases = self.sick_cases(h) as f64 / rp.memory_years.max(1e-9);
                        return Some((h, cases * case_h * homes * rp.avoided_share * kcal, 0.0));
                    }
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
                        // A home where fire is worked near others' homes, if this household
                        // works fire: what burnt it remembers (M8c slice CW).
                        RuleTemplate::FireFromHomes { from_m } => (
                            &move |b: &civ_land::Building| {
                                works_fire
                                    && ctx.catalog.is_dwelling(&b.spec.program)
                                    && plots(b).is_some_and(|r| {
                                        crate::rules::near_others_homes(
                                            &r,
                                            f64::from(from_m),
                                            &ctx.land.buildings,
                                            b.household,
                                            &plots,
                                            &|o| ctx.catalog.is_dwelling(&o.spec.program),
                                        )
                                    })
                            },
                            burnt,
                            0.0,
                        ),
                        RuleTemplate::MiddenFromWells { .. } => unreachable!("weighed above"),
                        // A building rising higher on walls of the kinds named (M8d slice DB): the
                        // buildings it heard gave way that rose so, against the floor the rule
                        // would take from its next home.
                        RuleTemplate::MostStoreys { storeys, kinds } => (
                            &move |b: &civ_land::Building| {
                                kinds & (1 << self.wall_kind_of(ctx, b)) != 0
                                    && b.spec.storeys > storeys
                            },
                            x.gave_way
                                .iter()
                                .filter(|l| {
                                    kinds & (1 << l.wall_kind.min(7)) != 0 && l.storeys > storeys
                                })
                                .count() as f64,
                            self.floor_lost_hours(ctx, x, &|wall, _| {
                                (kinds & (1 << wall) != 0).then_some(storeys)
                            }),
                        ),
                        // Mass walls above a storey untied (M8d slice DB): the buildings it heard
                        // gave way so, against tying its next home's walls if it knows how, else
                        // the floor building it a storey high would take.
                        RuleTemplate::TiedWalls => {
                            let ties = ctx
                                .catalog
                                .technique_index("core:technique/tied_masonry")
                                .is_some_and(|t| self.household_knows(&x.members, t));
                            (
                                &move |b: &civ_land::Building| {
                                    self.wall_kind_of(ctx, b) > 0
                                        && b.spec.storeys > 1
                                        && !crate::rules::tied(&b.spec)
                                },
                                x.gave_way
                                    .iter()
                                    .filter(|l| l.wall_kind > 0 && l.storeys > 1 && !l.tied)
                                    .count() as f64,
                                if ties {
                                    self.redesign_hours(ctx, x, &|spec, rules| {
                                        let tie = crate::rules::wall_kind(spec, rules) > 0
                                            && spec.storeys > 1
                                            && !crate::rules::tied(spec);
                                        if tie {
                                            spec.params[civ_grammar::frame::frame_params::TIES] = 1;
                                        }
                                        tie
                                    })
                                } else {
                                    self.floor_lost_hours(ctx, x, &|wall, tied| {
                                        (wall > 0 && !tied).then_some(1)
                                    })
                                },
                            )
                        }
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

    /// The work the floor a height rule would take from its newest home would take to build,
    /// hours (M8d slice DB, ADR-0031 §5: a household that would build tall forecasts the floor a
    /// height rule takes from it): the share of its storeys above the most `most` allows, given
    /// its walls' kind and whether they are tied, of what the home took to build. Nothing when it
    /// rises no higher than allowed.
    fn floor_lost_hours(
        &self,
        ctx: &Ctx,
        x: &Household,
        most: &dyn Fn(u8, bool) -> Option<u8>,
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
        let storeys = b.spec.storeys.max(1);
        match most(self.wall_kind_of(ctx, b), crate::rules::tied(&b.spec)) {
            Some(m) if m < storeys => {
                self.rebuild_hours(ctx, b).unwrap_or(0.0) * f64::from(storeys - m)
                    / f64::from(storeys)
            }
            _ => 0.0,
        }
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
                // Only a household that works fire is bound to keep its home apart.
                RuleTemplate::FireFromHomes { from_m } => {
                    if self.works_fire_at_home(ctx, hh) {
                        let m = f64::from(from_m);
                        kept.fire_from = Some(kept.fire_from.map_or(m, |k| k.max(m)));
                    }
                }
                RuleTemplate::MiddenFromWells { from_m } => {
                    let m = f64::from(from_m);
                    kept.midden_from = Some(kept.midden_from.map_or(m, |k| k.max(m)));
                }
                // The fewest storeys any height rule allows walls of each kind (M8d slice DB).
                RuleTemplate::MostStoreys { storeys, kinds } => {
                    let mut most = kept.storeys.unwrap_or([u8::MAX; 4]);
                    for (k, m) in most.iter_mut().enumerate() {
                        if kinds & (1 << k) != 0 {
                            *m = (*m).min(storeys);
                        }
                    }
                    kept.storeys = Some(most);
                }
                RuleTemplate::TiedWalls => kept.tied = true,
            }
        }
        kept
    }

    /// Whether household `hh` works fire at home (M8c slice CW): it holds a fixed tool, an oven
    /// or a kiln, that a recipe burning fuel needs (`fire.rs`'s worked fires burn where those who
    /// work them are, at home).
    pub(crate) fn works_fire_at_home(&self, ctx: &Ctx, hh: &Household) -> bool {
        let goods = &ctx.catalog.goods;
        let fuelled = |r: &&crate::params::RecipeDef| {
            r.inputs.iter().chain(&r.session_inputs).any(|&(g, _)| {
                goods
                    .get(g)
                    .is_some_and(|d| d.purpose == crate::params::GoodUse::Fuel)
            })
        };
        ctx.catalog
            .recipes
            .iter()
            .filter(fuelled)
            .flat_map(|r| r.tools.iter())
            .any(|&t| {
                goods
                    .get(t)
                    .and_then(|d| d.tool.as_ref())
                    .is_some_and(|d| d.fixed)
                    && hh.stores.get(t).is_some_and(|&kg| kg > 0.0)
            })
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

    /// The parcels of the plans household `hh`'s builders keep whose district allows only some
    /// uses (M8c slice CW, ADR-0030 §6), and the uses it allows, a bit for each: a building of
    /// another use is not put on them.
    pub(crate) fn zoned_kept(&self, ctx: &Ctx, hh: &Household) -> Vec<(civ_land::RectCm, u8)> {
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
        let mut out = Vec::new();
        for (law, laid) in polity.plans_in_force() {
            let Some(l) = polity.laws.iter().find(|l| l.id == law) else {
                continue;
            };
            let Some(def) = ctx
                .catalog
                .policies
                .get(usize::from(l.policy))
                .and_then(|d| d.plan.as_ref())
                .filter(|d| !d.districts.is_empty())
            else {
                continue;
            };
            if !self.keeps(ctx, person, l) {
                continue;
            }
            let phase = laid.layout.phases(def.phases);
            for (i, q) in laid.layout.parcels.iter().enumerate() {
                if let Some(d) = def.districts.iter().find(|d| d.0 == phase[i]) {
                    out.push((q.rect, d.1));
                }
            }
        }
        out
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
        // A building of a use its district does not allow on a plan's parcel breaks the plan too
        // (M8c slice CW), though it is no encroachment: it stands on no reserved ground.
        let use_ = ctx
            .land
            .buildings
            .iter()
            .find(|b| b.id == building)
            .and_then(|b| ctx.catalog.use_of(&b.spec.program));
        let zoned_out = |law: PermanentId, p: &crate::polity::PlanLaid| {
            let Some(def) = self.polities[pi]
                .laws
                .iter()
                .find(|l| l.id == law)
                .and_then(|l| ctx.catalog.policies.get(usize::from(l.policy)))
                .and_then(|d| d.plan.as_ref())
                .filter(|d| !d.districts.is_empty())
            else {
                return false;
            };
            let Some(use_) = use_ else {
                return false;
            };
            let phase = p.layout.phases(def.phases);
            p.layout
                .parcels
                .iter()
                .enumerate()
                .any(|(i, q)| q.rect.near(rect, 0) && !def.allows(phase[i], use_))
        };
        let zoned: Vec<PermanentId> = self.polities[pi]
            .plans_in_force()
            .filter(|&(law, p)| zoned_out(law, p))
            .map(|(law, _)| law)
            .collect();
        for &law in &zoned {
            let elder = self.elder_of(household, ctx.now, ctx.params);
            if let Some(l) = self.polities[pi].law_mut(law) {
                if elder.is_some_and(|e| l.knows(e)) {
                    l.compliance.broken += 1;
                } else {
                    l.compliance.broken_unaware += 1;
                }
            }
        }
        let found: Vec<(PermanentId, bool)> = self.polities[pi]
            .plans_in_force()
            .filter(|(_, p)| plan_bounds(&p.layout).is_some_and(|b| b.near(rect, 0)))
            .filter(|(law, _)| !zoned.contains(law))
            .map(|(law, p)| (law, !p.layout.reserves(rect, 0)))
            .collect();
        if found.is_empty() {
            return;
        }
        let elder = self.elder_of(household, ctx.now, ctx.params);
        let day = ctx.now.day_index();
        // Who sees it begun (M8c slice CV): the grown of the households whose homes lie within
        // sight of it, never its own; each plan's keeper living there is added below.
        let at = rect.centre_m();
        let sight = ctx.params.crime.sight_m as f32;
        let settlement = self.household(household).and_then(|x| x.settlement);
        let mut near: Vec<PermanentId> = Vec::new();
        for (_, x) in self.households.iter() {
            if x.id == household
                || x.settlement != settlement
                || (x.home.0 - at.0).hypot(x.home.1 - at.1) > sight
            {
                continue;
            }
            near.extend(self.grown_of(x.id, ctx.now, ctx.params));
        }
        let keepers: Vec<(PermanentId, Option<PermanentId>)> = found
            .iter()
            .map(|&(law, _)| {
                let keeper = self.polities[pi]
                    .laws
                    .iter()
                    .find(|l| l.id == law)
                    .and_then(|l| l.holder)
                    .filter(|&k| {
                        self.person(k)
                            .and_then(|p| self.household(p.household))
                            .is_some_and(|x| x.settlement == settlement && x.id != household)
                    });
                (law, keeper)
            })
            .collect();
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
            let mut seen_by = near.clone();
            if let Some(k) = keepers.iter().find(|k| k.0 == law).and_then(|k| k.1) {
                seen_by.push(k);
            }
            seen_by.sort_unstable();
            seen_by.dedup();
            polity.encroachments.push(crate::polity::Encroachment {
                day,
                law,
                household,
                building,
                seen_by,
                order: None,
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
        let deterred = if l.kind == PolicyKind::Plan {
            self.plan_sanction_points(ctx, person, l)
        } else {
            0.0
        };
        crate::polity::comply_points(norm, stance, regard, -deterred, &ctx.params.polity) > 0.0
    }

    /// What `person` expects building against plan `l` to cost their household, in the points a
    /// day of its food is worth in a dispute (M8c slice CV, third part; ADR-0030 §5): the chance
    /// they believe that someone sees, a hearing finds and the remedy follows, from the findings
    /// under the plan they have heard of against the buildings it has had begun on its ground,
    /// times the remedy's cost in days: its compensation and fine, a yearly payment over the
    /// years a household reckons a plot by, and for removal the days of work their own home took.
    fn plan_sanction_points(&self, ctx: &Ctx, person: PermanentId, l: &Law) -> f64 {
        use crate::crime::CaseStage;
        let (params, catalog) = (ctx.params, ctx.catalog);
        let s = l.sanction;
        let Some(polity) = self
            .polities
            .iter()
            .find(|p| p.laws.iter().any(|x| x.id == l.id))
        else {
            return 0.0;
        };
        let begun = polity
            .encroachments
            .iter()
            .filter(|e| e.law == l.id)
            .count();
        if begun == 0 {
            return 0.0;
        }
        let found = self
            .order
            .cases
            .iter()
            .filter(|c| c.law == l.id && c.building.is_some() && c.stage == CaseStage::Found)
            .filter(|c| {
                self.word.claims.iter().any(|k| {
                    k.kind == crate::word::ClaimKind::Finding
                        && k.event == Some(c.id)
                        && self.word.has_heard(person, k.id)
                })
            })
            .count();
        if found == 0 {
            return 0.0;
        }
        let chance = (found as f64 / begun as f64).min(1.0);
        let removal = if s.remove {
            let household = self.person(person).map(|p| p.household);
            ctx.land
                .buildings
                .iter()
                .filter(|b| Some(b.household) == household && b.standing())
                .filter(|b| catalog.is_dwelling(&b.spec.program))
                .filter_map(|b| {
                    let d = catalog.building_index(&b.spec.program)?;
                    build::building_hours(
                        &b.spec,
                        catalog.buildings.get(d)?,
                        params.household.carry_kg,
                    )
                })
                .fold(0.0, f64::max)
                / params.farm.work_hours_per_day.max(1.0)
        } else {
            0.0
        };
        let days = f64::from(s.compensation_days)
            + f64::from(s.fine_days)
            + f64::from(s.yearly_days) * params.build.site_years
            + removal;
        params.crime.w_demand_loss * chance * days
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
        let home_of = |o: &civ_land::Building| ctx.catalog.is_dwelling(&o.spec.program);
        let is_home = home_of(b);
        let works_fire = self
            .household(household)
            .is_some_and(|x| self.works_fire_at_home(ctx, x));
        for (law, t) in rules {
            let (bound, kept) = match t {
                RuleTemplate::RoofsUnburnt { within_m } => {
                    (near(within_m), crate::rules::roof_unburnt(&b.spec))
                }
                RuleTemplate::WallsOfKinds { kinds } => (true, kinds & (1 << wall) != 0),
                RuleTemplate::LeastGap { gap_m } => (true, !near(gap_m)),
                // A home where fire is worked, apart from others' homes (M8c slice CW).
                RuleTemplate::FireFromHomes { from_m } => (
                    is_home && works_fire,
                    !crate::rules::near_others_homes(
                        &rect,
                        f64::from(from_m),
                        &ctx.land.buildings,
                        household,
                        &plots,
                        &home_of,
                    ),
                ),
                // A home, and its midden, apart from every open well.
                RuleTemplate::MiddenFromWells { from_m } => (
                    is_home,
                    !crate::rules::near_open_well(&rect, f64::from(from_m), &ctx.land.wells),
                ),
                // A building on walls of the kinds named, no higher than allowed (M8d slice DB).
                RuleTemplate::MostStoreys { storeys, kinds } => {
                    (kinds & (1 << wall) != 0, b.spec.storeys <= storeys)
                }
                // Mass walls above a storey, tied at each floor.
                RuleTemplate::TiedWalls => {
                    (wall > 0 && b.spec.storeys > 1, crate::rules::tied(&b.spec))
                }
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

    /// Rule for building `policy`, sponsored by `sponsor`, put in force at polity `pi` today and
    /// known to everyone of it: for tests. Its id.
    #[doc(hidden)]
    pub fn enact_rule_for_tests(
        &mut self,
        ctx: &mut Ctx,
        pi: usize,
        policy: u16,
        sponsor: PermanentId,
    ) -> PermanentId {
        let now = ctx.now;
        let def = &ctx.catalog.policies[usize::from(policy)];
        let id = ctx.ids.allocate();
        let mut known: Vec<(PermanentId, i64)> = self
            .people
            .iter()
            .map(|(_, p)| (p.id, now.day_index()))
            .collect();
        known.sort_unstable_by_key(|k| k.0);
        self.polities[pi].laws.push(Law {
            id,
            policy,
            kind: def.kind,
            levy_share: 0.0,
            holder: None,
            relief_days: 0.0,
            sanction: Default::default(),
            hours: (0, 0),
            status: crate::polity::LawStatus::InForce,
            sponsor,
            proposed: now,
            issue: def
                .answers
                .first()
                .copied()
                .unwrap_or(crate::polity::IssueKind::Burned),
            meets_day: now.day_index(),
            decided: Some(now),
            outcome: Some(crate::polity::Outcome::Passed),
            eligible: 0,
            stances: Vec::new(),
            known,
            compliance: Default::default(),
            watch: Default::default(),
            body: None,
            ends: None,
            agreement: None,
            target: None,
        });
        id
    }

    /// What `person` expects building against plan `law` to cost, in points: for tests.
    #[doc(hidden)]
    pub fn plan_sanction_points_for_tests(
        &self,
        ctx: &Ctx,
        person: PermanentId,
        law: PermanentId,
    ) -> f64 {
        self.polities
            .iter()
            .find_map(|p| p.laws.iter().find(|l| l.id == law))
            .map_or(0.0, |l| self.plan_sanction_points(ctx, person, l))
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
