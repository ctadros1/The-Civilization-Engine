//! The ground shaken by the observer (M8d slice DA, ADR-0031 §3-4). Every standing building
//! reached is checked once, at the event's minute, in its lateral modes, against a share of its
//! weight drawn once for it: `C_s = strength · exp(0.30·z)`, `z` keyed by world, event and
//! building, never drawn again. What gives way goes the way a building gives way under load, so
//! those inside may die as they would; a mass wall that holds but shows the strain cracks, an
//! authored loss its upkeep mends, and keeps no cohesion until it is mended. Nothing here is
//! read by anyone's choice, and nothing is evaluated daily.

use super::loads::{Fell, gravity, loads_on};
use super::*;
use crate::influence::InfluenceKind;
use crate::quake::{Outcome, Quake, RESPONSE_SPREAD, STRENGTH_RANGE, Shaken};
use crate::structure::{self, STRAIN_SHOWS};
use civ_grammar::{Group, GroupKind};
use civ_land::GroupState;

/// Keyed-randomness purpose of a building's response to an event.
const PURPOSE_SHAKEN: u64 = 0x7368_616b_656e_3031; // "shaken01"

/// The loss a mass wall takes when it cracks: an authored share, as the reports give a crack's
/// extent only by eye (11-06 §3.2's damage grades; a tuning value). At least what shows, so its
/// upkeep sees and mends it.
pub const CRACK_LOSS: f32 = 0.2;

impl Population {
    /// The observer shakes the ground under settlement `settlement`, or the whole map, with a
    /// lateral share `strength` of every building's weight (ADR-0031 §4).
    pub fn shake(
        &mut self,
        ctx: &mut Ctx,
        strength: f64,
        settlement: Option<PermanentId>,
    ) -> Result<super::influence::Reached, String> {
        let (lo, hi) = STRENGTH_RANGE;
        // The observer's strength comes in single precision: 0.6 arrives a hair above 0.6.
        if !(lo as f32..=hi as f32).contains(&(strength as f32)) {
            return Err(format!("a shaking's strength is between {lo} and {hi}"));
        }
        if let Some(s) = settlement
            && !ctx.land.settlements.iter().any(|x| x.id == s)
        {
            return Err("no such settlement".to_string());
        }
        let id = (self.quakes.list.len() + 1) as u32;
        let target = settlement
            .or_else(|| ctx.land.settlements.first().map(|s| s.id))
            .unwrap_or_else(|| PermanentId::from_raw(1).expect("id"));
        let influence = self
            .influences
            .add(ctx.now, InfluenceKind::Earthquake, target, id);
        let reached: Vec<PermanentId> = ctx
            .land
            .buildings
            .iter()
            .filter(|b| !b.condition.is_empty() && b.standing())
            .filter(|b| {
                settlement.is_none()
                    || self.household(b.household).and_then(|x| x.settlement) == settlement
            })
            .map(|b| b.id)
            .collect();
        let mut shaken = Vec::with_capacity(reached.len());
        let mut changed: Vec<PermanentId> = Vec::new();
        for building in reached {
            let Some(i) = ctx.land.buildings.iter().position(|b| b.id == building) else {
                continue;
            };
            let household = ctx.land.buildings[i].household;
            let key = [
                ctx.seed,
                PURPOSE_SHAKEN,
                u64::from(influence),
                building.get(),
            ];
            let z = crate::demography::normal(&mut Rng64::from_key(&key));
            let cs = strength * (RESPONSE_SPREAD * z).exp();
            let kept = self.kept_by(ctx, household, i);
            let (fell, cracked) = self.shaken(ctx, i, &kept, cs);
            let deaths = if fell.is_empty() {
                0
            } else {
                self.give_way(ctx, i, &kept, &fell, 0.0, 0.0, true)
            };
            if !fell.is_empty() || !cracked.is_empty() {
                changed.push(household);
            }
            let outcome = if !fell.is_empty() {
                Outcome::Fell
            } else if !cracked.is_empty() {
                Outcome::Cracked
            } else {
                Outcome::Stood
            };
            shaken.push(Shaken {
                building,
                household,
                settlement: self.household(household).and_then(|x| x.settlement),
                z: z as f32,
                cs: cs as f32,
                outcome,
                cracked,
                deaths: deaths.min(usize::from(u16::MAX)) as u16,
            });
        }
        changed.sort_unstable();
        changed.dedup();
        for household in changed {
            self.buildings_changed(ctx.now, ctx.land, ctx.catalog, ctx.params, household);
        }
        let count = |o: Outcome| shaken.iter().filter(|s| s.outcome == o).count();
        let words = format!(
            " with {strength:.2} of a building's weight: {} fell, {} cracked, {} stood.",
            count(Outcome::Fell),
            count(Outcome::Cracked),
            count(Outcome::Stood)
        );
        self.quakes.list.push(Quake {
            id,
            influence,
            at: ctx.now,
            strength: strength as f32,
            settlement,
            origin: crate::floods::Origin::recorded(vec![influence]),
            shaken,
        });
        self.chronicle_push(
            ctx.now,
            crate::history::ChronicleKind::Influence,
            Vec::new(),
            settlement.or_else(|| ctx.land.settlements.first().map(|s| s.id)),
            None,
            f64::from(InfluenceKind::Earthquake.code()),
            words,
        );
        Ok(super::influence::Reached {
            id: influence,
            repeat: false,
            holds: false,
            fit: 0.0,
            chance: 0.0,
        })
    }

    /// Checks building `i`, holding `kept` of its household's goods, once in its lateral modes
    /// against a share `cs` of its weight: what gave way, the gravest first, and the groups that
    /// cracked.
    fn shaken(
        &mut self,
        ctx: &mut Ctx,
        i: usize,
        kept: &[Vec<f64>; 3],
        cs: f64,
    ) -> (Vec<Fell>, Vec<u32>) {
        let catalog = ctx.catalog;
        let b = &ctx.land.buildings[i];
        let Some(def) = catalog
            .building_index(&b.spec.program)
            .and_then(|d| catalog.buildings.get(d))
        else {
            return (Vec::new(), Vec::new());
        };
        let Some(e) = self.expansion_of(b, def) else {
            return (Vec::new(), Vec::new());
        };
        let loads = loads_on(b, &e, kept);
        let timber = |g: &Group| {
            g.material
                .and_then(|slot| def.materials.get(usize::from(slot)))
                .and_then(|&good| catalog.goods.get(good))
                .and_then(|good| good.timber)
        };
        let shows = def.upkeep.infill.shows_at as f32;
        let quakes = &self.quakes;
        let b = &mut ctx.land.buildings[i];
        let (mut fell, mut cracked) = (Vec::new(), Vec::new());
        for c in b.condition.iter_mut() {
            if c.state == GroupState::Failed {
                continue;
            }
            let Some(g) = e.groups.iter().find(|g| g.id == c.group) else {
                continue;
            };
            let t = match g.kind {
                GroupKind::MassWall => {
                    let (strength, density) = crate::rules::masonry(&def.rules, g.spacing_cm);
                    structure::Timber {
                        bending_pa: 0.0,
                        compression_pa: strength,
                        stiffness_pa: 0.0,
                        creep: 0.0,
                        sustained: 1.0,
                        density_kg_m3: density,
                    }
                }
                GroupKind::Posts => match timber(g) {
                    Some(t) => t,
                    None => continue,
                },
                _ => continue,
            };
            let was_cracked =
                g.kind == GroupKind::MassWall && quakes.cracked(b.id, c.group, c.repaired);
            let m = structure::lateral_margin(g, c, &e, &loads, &t, cs, was_cracked);
            if m.strength < 1.0 {
                c.state = GroupState::Failed;
                fell.push(Fell {
                    kind: g.kind,
                    quality: c.quality,
                    loss: c.loss,
                });
            } else if g.kind == GroupKind::MassWall && m.strength < STRAIN_SHOWS {
                c.loss = (c.loss + CRACK_LOSS).max(shows).min(1.0);
                c.state = condition::worse(c.state, condition::wear_state(c, &def.upkeep));
                cracked.push(c.group);
            }
        }
        condition::settle_state(b);
        fell.sort_by_key(|f| gravity(f.kind));
        (fell, cracked)
    }
}
