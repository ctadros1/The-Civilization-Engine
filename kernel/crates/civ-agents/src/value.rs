//! What goods are worth to a household, in hours of its own work: the time it would take it to
//! get one more unit itself (slice I). It anchors the terms a household posts and is the
//! yardstick by which it judges an offer (research 08-04 §1.2: prices from a smoothed normal
//! cost; 08-15 §2.1: keep work time per unit beside prices).

use civ_core::time::DAYS_PER_YEAR;
use civ_land::{CropParams, ResourceParams};

use crate::params::{Catalog, RecipeDef, interpolate};
use crate::person::KnownPatch;

/// Rounds of relaxation over the recipes: enough for the content's chains (an input of an input
/// of an input).
const ROUNDS: usize = 4;
/// Hours a gathering trip takes over the hours of work at the place: the walk there and back (a
/// tuning value).
const TRIP_OVERHEAD: f64 = 1.25;
/// What a household expects of a place it has not been to, a share of the best rate a resource
/// gives (a tuning value).
const UNSEEN_RATE_SHARE: f64 = 0.5;

/// Hours per kilogram of growing each crop's grain and seed: the crop's field work over a
/// cautious yield less the seed (`yield_share` of the average), and threshing; and kept from the
/// crop's harvest to `day` (a day index), what the store took of it meanwhile at
/// `half_life(good)` days (0 for none). A kilogram held in spring had to be grown as more than a
/// kilogram in the autumn before: storage links harvest prices to later ones (research 08-05
/// §1.6), so grain is cheapest as it ripens and dearest just before.
pub fn grown_costs(
    crops: &[CropParams],
    goods: usize,
    yield_share: f64,
    day: i64,
    half_life: &dyn Fn(usize) -> f64,
) -> Vec<Option<f64>> {
    let mut out = vec![None; goods];
    for c in crops {
        let net = c.yield_kg_per_ha * yield_share - c.seed_kg_per_ha;
        if net <= 0.0 {
            continue;
        }
        let field_h = c.prepare_h_per_ha + c.sow_h_per_ha + c.tend_h_per_ha + c.reap_h_per_ha;
        let h = field_h / net + c.thresh_h_per_kg;
        let ripe = i64::from(c.sow_from_day) + i64::from(c.grow_days);
        let kept = (civ_land::day_of_year(day) - ripe).rem_euclid(DAYS_PER_YEAR) as f64;
        for g in [c.good, c.seed_good] {
            let life = half_life(g);
            let carried = if life > 0.0 {
                h * (kept * std::f64::consts::LN_2 / life).exp()
            } else {
                h
            };
            if let Some(slot) = out.get_mut(g) {
                *slot = Some(slot.map_or(carried, |x: f64| x.min(carried)));
            }
        }
    }
    out
}

/// Hours per unit of gathering each good, from what a household has seen of the places it knows
/// (the best return it found, with the walk there and back), or for a good it has never gone out
/// for, half the best rate its resources give.
pub fn gathered_costs(
    resources: &[ResourceParams],
    known: &[KnownPatch],
    goods: usize,
) -> Vec<Option<f64>> {
    let mut out = vec![None; goods];
    let mut seen = vec![false; goods];
    for k in known {
        if let Some(r) = resources.get(usize::from(k.resource))
            && let Some(s) = seen.get_mut(r.good)
        {
            *s = true;
        }
    }
    for r in resources {
        let per_hour = r.max_rate_per_hour * r.unit_kg * UNSEEN_RATE_SHARE;
        if per_hour <= 0.0 || seen.get(r.good).copied().unwrap_or(true) {
            continue;
        }
        let h = TRIP_OVERHEAD / per_hour;
        if let Some(slot) = out.get_mut(r.good) {
            *slot = Some(slot.map_or(h, |x: f64| x.min(h)));
        }
    }
    for k in known {
        let Some(r) = resources.get(usize::from(k.resource)) else {
            continue;
        };
        let per_hour = f64::from(k.rate) * r.unit_kg;
        if per_hour <= 0.0 {
            continue;
        }
        let h = TRIP_OVERHEAD / per_hour;
        if let Some(slot) = out.get_mut(r.good) {
            *slot = Some(slot.map_or(h, |x: f64| x.min(h)));
        }
    }
    out
}

/// Food a household cannot get itself (provisions carried in) is worth what the same energy
/// costs it in its cheapest food: fills in those costs, by energy.
pub fn food_by_energy(goods: &[crate::params::GoodDef], costs: &mut [Option<f64>]) {
    use crate::params::GoodUse;
    let per_kcal = goods
        .iter()
        .zip(costs.iter())
        .filter(|(d, _)| d.purpose == GoodUse::Food && d.kcal_per_kg > 0.0)
        .filter_map(|(d, c)| c.map(|c| c / d.kcal_per_kg))
        .fold(f64::INFINITY, f64::min);
    if !per_kcal.is_finite() {
        return;
    }
    for (d, c) in goods.iter().zip(costs.iter_mut()) {
        if d.purpose == GoodUse::Food && d.kcal_per_kg > 0.0 && c.is_none() {
            *c = Some(per_kcal * d.kcal_per_kg);
        }
    }
}

/// A household's own cost of each good, hours of a capable adult's work per unit: the cheapest of
/// gathering it (`gathered`), growing it (`grown`), or making it by a recipe from inputs at their
/// own cost, with the tools the recipe wears. A recipe is made at `level(recipe)`, the skill of
/// the best of its members who know the technique it needs, and counts only if one of them does
/// (`None`; ADR-0008 §1). `None` where it has no way to get the good.
pub fn own_costs(
    catalog: &Catalog,
    level: &dyn Fn(&RecipeDef) -> Option<f64>,
    gathered: &[Option<f64>],
    grown: &[Option<f64>],
) -> Vec<Option<f64>> {
    let n = catalog.goods.len();
    let pick = |a: Option<f64>, b: Option<f64>| match (a, b) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (a, b) => a.or(b),
    };
    let mut cost: Vec<Option<f64>> = (0..n)
        .map(|g| {
            pick(
                gathered.get(g).copied().flatten(),
                grown.get(g).copied().flatten(),
            )
        })
        .collect();
    for _ in 0..ROUNDS {
        let mut changed = false;
        for r in &catalog.recipes {
            let Some(&(out, per)) = r.outputs.first() else {
                continue;
            };
            // A recipe nobody in the household knows is no way for it to get a good
            // (ADR-0008 §1).
            let Some(level) = level(r) else {
                continue;
            };
            let skill = r.skill.and_then(|k| catalog.skills.get(k).map(|s| (k, s)));
            let speed = skill
                .map_or(1.0, |(_, s)| interpolate(&s.speed, level))
                .max(1e-6);
            let made_tool = catalog.goods.get(out).is_some_and(|g| g.tool.is_some());
            let quality = match skill {
                Some((_, s)) if made_tool => interpolate(&s.quality, level).max(1e-6),
                _ => 1.0,
            };
            let work = r.unit_h / speed;
            let mut hours = Some(work);
            for &(g, amount) in &r.inputs {
                hours = match (hours, cost.get(g).copied().flatten()) {
                    (Some(h), Some(c)) => Some(h + amount * c),
                    _ => None,
                };
            }
            for &t in &r.tools {
                let life = catalog
                    .goods
                    .get(t)
                    .and_then(|g| g.tool.as_ref())
                    .map(|d| d.life_h);
                hours = match (hours, cost.get(t).copied().flatten(), life) {
                    (Some(h), Some(c), Some(life)) if life > 0.0 => Some(h + work / life * c),
                    _ => None,
                };
            }
            let Some(h) = hours else {
                continue;
            };
            let unit = h / (per * quality).max(1e-9);
            if cost[out].is_none_or(|c| unit < c - 1e-12) {
                cost[out] = Some(unit);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    cost
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::{Eaten, GoodDef, GoodUse, RecipeDef, SkillDef, ToolDef};

    fn good(id: &str, purpose: GoodUse, tool: Option<f64>) -> GoodDef {
        GoodDef {
            id: id.into(),
            name: id.into(),
            purpose,
            kcal_per_kg: 0.0,
            half_life_days: 0.0,
            sheltered_half_life_days: 0.0,
            eaten: Eaten::Never,
            shared: false,
            reserve_for: None,
            tool: tool.map(|life_h| ToolDef {
                life_h,
                per_worker: 1.0,
                fixed: false,
            }),
            timber: None,
            store: None,
        }
    }

    /// Flint (0), wood (1), a sickle (2) made from them by knapping (skill 0), and an axe (3)
    /// that nobody can make.
    fn catalog() -> Catalog {
        Catalog {
            goods: vec![
                good("flint", GoodUse::Material, None),
                good("wood", GoodUse::Material, None),
                good("sickle", GoodUse::Tool, Some(100.0)),
                good("axe", GoodUse::Tool, Some(200.0)),
            ],
            recipes: vec![RecipeDef {
                id: "make_sickle".into(),
                name: "Make a sickle".into(),
                inputs: vec![(0, 0.3), (1, 0.5)],
                outputs: vec![(2, 1.0)],
                session_inputs: Vec::new(),
                unit_h: 3.0,
                session_h: 0.0,
                max_units: 1.0,
                tools: Vec::new(),
                skill: Some(0),
                technique: None,
            }],
            skills: vec![SkillDef {
                id: "knapping".into(),
                name: "Knapping".into(),
                t80_h: 1000.0,
                speed: vec![(0.0, 0.5), (1.0, 1.5)],
                quality: vec![(0.0, 0.5), (1.0, 1.5)],
                founder_level: [0.2, 0.6],
            }],
            ..Catalog::default()
        }
    }

    #[test]
    fn a_skilled_household_makes_a_tool_for_fewer_hours() {
        let c = catalog();
        let gathered = vec![Some(2.0), Some(0.5), None, None];
        let novice = own_costs(&c, &|_| Some(0.0), &gathered, &[]);
        let master = own_costs(&c, &|_| Some(1.0), &gathered, &[]);
        // Materials: 0.3 kg of flint at 2 h/kg and 0.5 kg of wood at 0.5 h/kg, 0.85 h.
        // A novice works 3 / 0.5 = 6 h and makes half a standard sickle: 13.7 h a sickle.
        let n = novice[2].expect("a novice can make one");
        assert!((n - (6.0 + 0.85) / 0.5).abs() < 1e-9, "{n}");
        // A master works 2 h and makes 1.5 standard sickles: 1.9 h a sickle.
        let m = master[2].expect("so can a master");
        assert!((m - (2.0 + 0.85) / 1.5).abs() < 1e-9, "{m}");
        // What nobody can make or find has no cost: it can only be had from someone else.
        assert_eq!(novice[3], None);
        // Without flint to be found, no sickle either.
        let none = own_costs(&c, &|_| Some(1.0), &[None, Some(0.5), None, None], &[]);
        assert_eq!(none[2], None);
        // Nor when nobody in the household knows how to make one (ADR-0008 §1).
        let unknown = own_costs(&c, &|_| None, &gathered, &[]);
        assert_eq!(unknown[2], None);
    }

    #[test]
    fn grain_costs_the_field_work_over_a_cautious_yield_less_the_seed() {
        let crop = CropParams {
            id: "emmer".into(),
            name: "Emmer".into(),
            good: 0,
            seed_good: 1,
            seed_kg_per_ha: 90.0,
            yield_kg_per_ha: 900.0,
            prepare_from_day: 59,
            sow_from_day: 80,
            sow_until_day: 125,
            late_sowing_loss_per_day: 0.005,
            grow_days: 120,
            standing_loss_per_day: 0.02,
            untended_loss: 0.4,
            break_h_per_ha: 1000.0,
            prepare_h_per_ha: 300.0,
            sow_h_per_ha: 120.0,
            tend_h_per_ha: 200.0,
            reap_h_per_ha: 280.0,
            thresh_h_per_kg: 0.1,
            straw: None,
            kc: [0.4, 1.15, 0.4],
            kc_days: [30, 30, 40, 20],
            ky: 1.15,
            n_kg_per_kg_grain: 0.02,
            n_kg_per_kg_straw: 0.005,
            roots_n_kg_per_ha: 15.0,
        };
        let ripe = i64::from(crop.sow_from_day) + i64::from(crop.grow_days);
        let costs = grown_costs(std::slice::from_ref(&crop), 3, 0.8, ripe, &|_| 0.0);
        let h = 900.0 / (900.0 * 0.8 - 90.0) + 0.1;
        assert!((costs[0].expect("grain") - h).abs() < 1e-9);
        assert_eq!(costs[0], costs[1], "seed is grain");
        assert_eq!(costs[2], None);
        // Kept in store, it costs what was lost meanwhile: half a half-life after the harvest,
        // √2 as much; a year on, as it ripens again, as little as ever.
        let half = |_: usize| 200.0;
        let kept = grown_costs(std::slice::from_ref(&crop), 3, 0.8, ripe + 100, &half);
        assert!((kept[0].expect("grain") - h * 2f64.sqrt()).abs() < 1e-9);
        let again = grown_costs(&[crop], 3, 0.8, ripe + DAYS_PER_YEAR, &half);
        assert!((again[0].expect("grain") - h).abs() < 1e-9);
    }
}
