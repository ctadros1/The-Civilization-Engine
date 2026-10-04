//! Land under the world's property regime (ADR-0007 §2): how a settlement gives its fields to work
//! by need, the share of its families' fields a new couple takes, what becomes of a household's
//! holdings when it is no more, and ground nobody holds any more taken up again. Who holds the
//! ground a household breaks is set where it is marked out. Holding or working a field changes
//! nothing about how it yields (research 08-09: a change of claim is not a change of fertility).

use std::collections::HashSet;

use civ_core::PermanentId;
use civ_land::{Field, FieldStage, Party};

use super::{Ctx, Population};
use crate::farm;
use crate::params::{LandUse, Succession};

/// Whether a field can change hands at a review: between crops, with nothing waiting to be
/// threshed and no work begun on its next crop, so a crop and the work put into it stay with the
/// household that did it.
fn between_crops(f: &Field) -> bool {
    f.stage == FieldStage::Fallow && f.sheaves_kg <= 0.0 && f.work_h <= 0.0
}

fn distance_m(a: (f32, f32), b: (f32, f32)) -> f64 {
    f64::from(a.0 - b.0).hypot(f64::from(a.1 - b.1))
}

/// Which of `fields` (`(area, distance)`, in the order to take them) make up `share` of `held`
/// hectares, to the nearest whole field: indexes into `fields`, taken in order until one more
/// would overshoot by more than half itself.
fn pick_share(fields: &[(f64, f64)], held: f64, share: f64) -> Vec<usize> {
    let target = held * share.clamp(0.0, 1.0);
    let mut taken = 0.0;
    let mut out = Vec::new();
    for (i, &(area, _)) in fields.iter().enumerate() {
        if taken + area / 2.0 > target {
            break;
        }
        taken += area;
        out.push(i);
    }
    out
}

/// Divides `areas` among `heirs` heirs, as near equal in area as whole fields allow: the largest
/// first, each to the heir with the least so far (the first such). Returns each field's heir, by
/// index; ties between fields of one size keep their order.
fn divide(areas: &[f64], heirs: usize) -> Vec<usize> {
    let mut order: Vec<usize> = (0..areas.len()).collect();
    order.sort_by(|&a, &b| areas[b].total_cmp(&areas[a]).then(a.cmp(&b)));
    let mut got = vec![0.0f64; heirs.max(1)];
    let mut out = vec![0; areas.len()];
    for i in order {
        let k = (0..got.len())
            .min_by(|&a, &b| got[a].total_cmp(&got[b]).then(a.cmp(&b)))
            .unwrap_or(0);
        got[k] += areas[i];
        out[i] = k;
    }
    out
}

impl Population {
    /// The area a household of `members` needs to crop, hectares, as households plan it
    /// themselves ([`farm::need_area_ha`]); 0 without a crop.
    pub(super) fn need_ha(&self, ctx: &Ctx, members: usize) -> f64 {
        let Some(crop) = ctx.catalog.crops.get(ctx.params.farm.crop) else {
            return 0.0;
        };
        let kcal = ctx
            .catalog
            .goods
            .get(crop.good)
            .map_or(0.0, |g| g.kcal_per_kg);
        farm::need_area_ha(members, ctx.params, crop, kcal)
    }

    /// Whether a field is vacant under the regime: nobody may work it until it changes hands.
    /// Under allocation by need, a field whose household is no more; under holders, ground whose
    /// holder is no household of the living (it left, died out with no heir, or the ground went
    /// back to the settlement).
    fn vacant(&self, regime_use: LandUse, f: &Field) -> bool {
        match regime_use {
            LandUse::Need => self.household(f.household).is_none(),
            LandUse::Holder => match f.holder {
                Party::Household(h) => {
                    self.household(h).is_none() && self.household(f.household).is_none()
                }
                Party::Settlement(_) => self.household(f.household).is_none(),
            },
        }
    }

    /// The land of `settlement` changes hands as the regime says (ADR-0007 §2), at its yearly
    /// review and when a household forms or ends. Households short of the area they need take,
    /// the furthest short first and nearest their home first:
    ///
    /// - under allocation by need, fields of the settlement that are vacant or beyond what the
    ///   household working them needs (never leaving it short; research 08-09 §5.2);
    /// - under holders, ground nobody holds any more, which the taker then holds (first
    ///   occupation, 08-09 §1.2).
    ///
    /// Only fields between crops change hands. Returns how many did.
    pub(super) fn review_land(&mut self, ctx: &mut Ctx, settlement: PermanentId) -> usize {
        let regime_use = ctx.regime.land_use;
        // Households of the settlement: their home, need and the area they work.
        let mut households: Vec<(PermanentId, (f32, f32), f64, f64)> = self
            .households
            .iter()
            .map(|(_, x)| x)
            .filter(|x| x.settlement == Some(settlement) && !x.members.is_empty())
            .map(|x| (x.id, x.home, self.need_ha(ctx, x.members.len()), 0.0))
            .collect();
        if households.is_empty() {
            return 0;
        }
        households.sort_by_key(|x| x.0);
        for f in &ctx.land.fields {
            if let Some(x) = households.iter_mut().find(|x| x.0 == f.household) {
                x.3 += f.rect.area_ha();
            }
        }
        // Fields that could change hands: the settlement's (allocation by need), or any vacant
        // ground near it (holders).
        let mine: HashSet<PermanentId> = households.iter().map(|x| x.0).collect();
        let candidates: Vec<usize> = ctx
            .land
            .fields
            .iter()
            .enumerate()
            .filter(|(_, f)| between_crops(f))
            .filter(|(_, f)| match regime_use {
                LandUse::Need => f.holder == Party::Settlement(settlement),
                LandUse::Holder => self.vacant(regime_use, f),
            })
            .map(|(i, _)| i)
            .collect();
        let mut done: HashSet<PermanentId> = HashSet::new();
        let mut moved = 0;
        loop {
            // The household furthest short of its need, by share of it.
            let taker = households
                .iter()
                .filter(|x| !done.contains(&x.0) && x.2 > 0.0 && x.3 < x.2)
                .max_by(|a, b| {
                    ((a.2 - a.3) / a.2)
                        .total_cmp(&((b.2 - b.3) / b.2))
                        .then(b.0.cmp(&a.0))
                })
                .map(|x| (x.0, x.1));
            let Some((taker, home)) = taker else {
                break;
            };
            let pick = candidates
                .iter()
                .copied()
                .filter(|&i| {
                    let f = &ctx.land.fields[i];
                    if f.household == taker {
                        return false;
                    }
                    if self.vacant(regime_use, f) {
                        return true;
                    }
                    // A field beyond what its household needs, under allocation by need.
                    regime_use == LandUse::Need
                        && mine.contains(&f.household)
                        && households
                            .iter()
                            .find(|x| x.0 == f.household)
                            .is_some_and(|x| x.3 - f.rect.area_ha() >= x.2)
                })
                .min_by(|&a, &b| {
                    let (fa, fb) = (&ctx.land.fields[a], &ctx.land.fields[b]);
                    distance_m(fa.rect.centre_m(), home)
                        .total_cmp(&distance_m(fb.rect.centre_m(), home))
                        .then(fa.id.cmp(&fb.id))
                });
            let Some(i) = pick else {
                done.insert(taker);
                continue;
            };
            let f = &mut ctx.land.fields[i];
            let area = f.rect.area_ha();
            if let Some(giver) = households.iter_mut().find(|x| x.0 == f.household) {
                giver.3 -= area;
            }
            f.household = taker;
            if regime_use == LandUse::Holder {
                f.holder = Party::Household(taker);
            }
            if let Some(x) = households.iter_mut().find(|x| x.0 == taker) {
                x.3 += area;
            }
            moved += 1;
        }
        moved
    }

    /// A new household `to` takes `share` of the fields household `from` holds and works (a
    /// couple's share of its families' land, as of their stores; ADR-0007 §2), the fields nearest
    /// its home first, to the nearest whole field. Fields with sheaves waiting stay; a crop
    /// growing goes with its field.
    pub(super) fn share_fields(
        &mut self,
        ctx: &mut Ctx,
        from: PermanentId,
        to: PermanentId,
        share: f64,
    ) {
        if share.is_nan() || share <= 0.0 {
            return;
        }
        let Some(home) = self.household(to).map(|x| x.home) else {
            return;
        };
        let mut own: Vec<usize> = ctx
            .land
            .fields
            .iter()
            .enumerate()
            .filter(|(_, f)| f.holder == Party::Household(from) && f.household == from)
            .map(|(i, _)| i)
            .collect();
        let held: f64 = own.iter().map(|&i| ctx.land.fields[i].rect.area_ha()).sum();
        own.retain(|&i| ctx.land.fields[i].sheaves_kg <= 0.0);
        own.sort_by(|&a, &b| {
            let (fa, fb) = (&ctx.land.fields[a], &ctx.land.fields[b]);
            distance_m(fa.rect.centre_m(), home)
                .total_cmp(&distance_m(fb.rect.centre_m(), home))
                .then(fa.id.cmp(&fb.id))
        });
        let candidates: Vec<(f64, f64)> = own
            .iter()
            .map(|&i| {
                let f = &ctx.land.fields[i];
                (f.rect.area_ha(), distance_m(f.rect.centre_m(), home))
            })
            .collect();
        for k in pick_share(&candidates, held, share) {
            let f = &mut ctx.land.fields[own[k]];
            f.holder = Party::Household(to);
            f.household = to;
        }
    }

    /// What becomes of the fields of household `from`, which is no more, under the regime's
    /// succession rule (ADR-0007 §2; research 08-09 §1.7): to one heir, divided among its heirs'
    /// households (whole fields, as near equal in area as they can be, a crop or sheaves going
    /// with its field as part of the estate), or back to its settlement. `heirs` are the
    /// households of its nearest kin, the first of them the one its people's goods go to. Fields
    /// it worked that others hold are left vacant for their holders.
    pub(super) fn succeed(&mut self, ctx: &mut Ctx, from: PermanentId, heirs: &[PermanentId]) {
        let settlement = self.household(from).and_then(|x| x.settlement);
        let mut held: Vec<usize> = ctx
            .land
            .fields
            .iter()
            .enumerate()
            .filter(|(_, f)| f.holder == Party::Household(from))
            .map(|(i, _)| i)
            .collect();
        match ctx.regime.succession {
            Succession::Heir | Succession::Divided => {
                let heirs: Vec<PermanentId> = match ctx.regime.succession {
                    Succession::Heir => heirs.iter().copied().take(1).collect(),
                    _ => heirs.to_vec(),
                };
                if heirs.is_empty() {
                    return;
                }
                held.sort_by_key(|&i| ctx.land.fields[i].id);
                let areas: Vec<f64> = held
                    .iter()
                    .map(|&i| ctx.land.fields[i].rect.area_ha())
                    .collect();
                for (&i, k) in held.iter().zip(divide(&areas, heirs.len())) {
                    let f = &mut ctx.land.fields[i];
                    f.holder = Party::Household(heirs[k]);
                    if f.household == from {
                        f.household = heirs[k];
                    }
                }
            }
            Succession::Settlement => {
                if let Some(s) = settlement {
                    for i in held {
                        ctx.land.fields[i].holder = Party::Settlement(s);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_share_is_taken_in_whole_fields_nearest_first() {
        let quarter = 0.25;
        let fields = [
            (quarter, 10.0),
            (quarter, 40.0),
            (quarter, 90.0),
            (quarter, 300.0),
        ];
        // A third of a hectare is one field and a third: one field.
        assert_eq!(pick_share(&fields, 1.0, 1.0 / 3.0), vec![0]);
        // Two leavers of five take two fifths of it: 0.4 ha, nearer two fields than one.
        assert_eq!(pick_share(&fields, 1.0, 0.4), vec![0, 1]);
        assert!(
            pick_share(&fields, 1.0, 0.1).is_empty(),
            "less than half a field"
        );
        assert_eq!(pick_share(&fields, 1.0, 1.0).len(), 4);
    }

    #[test]
    fn an_estate_is_divided_as_evenly_as_whole_fields_allow() {
        // Five fields among two heirs: 0.5 + 0.25 against 0.25 + 0.25 + 0.2.
        let areas = [0.25, 0.5, 0.25, 0.2, 0.25];
        let heirs = divide(&areas, 2);
        let share = |k: usize| -> f64 {
            areas
                .iter()
                .zip(&heirs)
                .filter(|&(_, &h)| h == k)
                .map(|(a, _)| a)
                .sum()
        };
        assert_eq!(heirs[1], 0, "the largest field to the first heir");
        assert!((share(0) - 0.75).abs() < 1e-9 && (share(1) - 0.7).abs() < 1e-9);
        assert_eq!(divide(&areas, 1), vec![0; 5], "one heir takes all");
        assert!(divide(&[], 3).is_empty());
    }
}
