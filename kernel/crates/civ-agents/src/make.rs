//! Recipes and tools as arithmetic on a household's stores (ADR-0006 §1–2): what a session of a
//! recipe can make, what it takes and gives, what tools a household wants and how use wears them.
//! Pure functions; the population applies them.

use crate::params::{GoodDef, RecipeDef};

/// Spare tool a household keeps beyond the ones its members use, in standard tools: it makes the
/// next one while the last is still worn down to this (a tuning value).
pub const SPARE_TOOL: f64 = 0.25;

/// Tools a household wants, by good index: `per_worker` of each tool for each member old enough
/// for the work that needs it (`workers(tool)`), rounded up, and at least one of any tool it
/// wants at all.
pub fn tool_wants(goods: &[GoodDef], workers: &dyn Fn(usize) -> usize) -> Vec<f64> {
    goods
        .iter()
        .enumerate()
        .map(|(i, g)| match &g.tool {
            Some(t) if t.per_worker > 0.0 => (t.per_worker * workers(i) as f64).ceil().max(1.0),
            _ => 0.0,
        })
        .collect()
}

/// Tools a household whose members are `ages` years old wants (see [`tool_wants`]): for each
/// tool, its members old enough for the work that needs it, or grown (`grown` years) for a tool
/// no work needs.
pub fn tool_wants_for(catalog: &crate::params::Catalog, ages: &[f64], grown: f64) -> Vec<f64> {
    let youngest = tool_ages(catalog);
    tool_wants(&catalog.goods, &|t| {
        let from = youngest.get(t).copied().flatten().unwrap_or(grown);
        ages.iter().filter(|&&a| a >= from).count()
    })
}

/// The youngest age at which anyone does work that needs each tool, by good index: the lowest
/// `min_age_years` of the activities that need it, their own or their recipe's (`None` for goods
/// no work needs).
pub fn tool_ages(catalog: &crate::params::Catalog) -> Vec<Option<f64>> {
    let mut ages = vec![None; catalog.goods.len()];
    for a in &catalog.activities {
        for t in catalog.tools_of(a) {
            if let Some(age) = ages.get_mut(t) {
                *age = Some(age.map_or(a.min_age_years, |x: f64| x.min(a.min_age_years)));
            }
        }
    }
    ages
}

/// How much of a tool a household still wants, 0–1, given what it wants and holds: a whole tool
/// when it is short of one, less as the spare it keeps is worn down.
pub fn tool_need(wanted: f64, held: f64) -> f64 {
    if wanted <= 0.0 {
        0.0
    } else {
        (wanted + SPARE_TOOL - held.max(0.0)).clamp(0.0, 1.0)
    }
}

/// What a household can put into a recipe of each good: what it holds, and in hunger the goods
/// kept back for it beyond `protected` (`(good, kilograms)`: the seed for the ground already
/// cropped).
pub fn available(
    stores: &[f64],
    goods: &[GoodDef],
    good: usize,
    reserve_ok: bool,
    protected: Option<(usize, f64)>,
) -> f64 {
    let mut have = stores.get(good).copied().unwrap_or(0.0).max(0.0);
    if reserve_ok {
        for (g, def) in goods.iter().enumerate() {
            if def.reserve_for == Some(good) {
                let keep = protected.filter(|(p, _)| *p == g).map_or(0.0, |(_, kg)| kg);
                have += (stores.get(g).copied().unwrap_or(0.0) - keep).max(0.0);
            }
        }
    }
    have
}

/// Units of `recipe` the inputs at hand allow: 0 if its session inputs are not all there, and
/// without limit if it takes nothing per unit (time and its most per session limit it then).
pub fn units_from_inputs(recipe: &RecipeDef, have: &dyn Fn(usize) -> f64) -> f64 {
    for &(g, amount) in &recipe.session_inputs {
        if have(g) + 1e-9 < amount {
            return 0.0;
        }
    }
    let mut units = f64::INFINITY;
    for &(g, amount) in &recipe.inputs {
        if amount > 0.0 {
            // What is left once the session's own share of the good is taken.
            let session: f64 = recipe
                .session_inputs
                .iter()
                .filter(|(s, _)| *s == g)
                .map(|(_, a)| a)
                .sum();
            units = units.min(((have(g) - session) / amount).max(0.0));
        }
    }
    units
}

/// Units a session of `minutes` makes at `speed` (work done per hour against the recipe's
/// rates): the work left after the session's own labour, at the recipe's labour per unit.
pub fn units_in(recipe: &RecipeDef, minutes: f64, speed: f64) -> f64 {
    let hours = minutes / 60.0 * speed.max(0.0);
    let units = if recipe.unit_h > 0.0 {
        ((hours - recipe.session_h) / recipe.unit_h).max(0.0)
    } else if hours >= recipe.session_h {
        f64::INFINITY
    } else {
        0.0
    };
    if recipe.max_units > 0.0 {
        units.min(recipe.max_units)
    } else {
        units
    }
}

/// Minutes a session making `units` takes at `speed`.
pub fn minutes_for(recipe: &RecipeDef, units: f64, speed: f64) -> f64 {
    (recipe.session_h + units.max(0.0) * recipe.unit_h) * 60.0 / speed.max(1e-6)
}

/// Works `units` of `recipe` on `stores`: takes its inputs (goods kept back for an input only
/// when `reserve_ok`, beyond `protected`) and gives its outputs, tools made worth `quality`
/// standard tools each. Returns the units actually made, no more than the inputs allowed.
pub fn apply(
    recipe: &RecipeDef,
    stores: &mut [f64],
    goods: &[GoodDef],
    units: f64,
    quality: f64,
    reserve_ok: bool,
    protected: Option<(usize, f64)>,
) -> f64 {
    let can = {
        let snapshot: &[f64] = stores;
        units_from_inputs(recipe, &|g| {
            available(snapshot, goods, g, reserve_ok, protected)
        })
    };
    let most = if recipe.max_units > 0.0 {
        recipe.max_units
    } else {
        f64::INFINITY
    };
    let units = units.min(can).min(most).max(0.0);
    if !(units > 0.0 && units.is_finite()) {
        return 0.0;
    }
    let mut take = |g: usize, mut amount: f64| {
        let own = stores.get(g).copied().unwrap_or(0.0).max(0.0).min(amount);
        if let Some(s) = stores.get_mut(g) {
            *s -= own;
        }
        amount -= own;
        if !reserve_ok || amount <= 0.0 {
            return;
        }
        // The rest from goods kept back for it.
        for (k, def) in goods.iter().enumerate() {
            if def.reserve_for == Some(g) && amount > 0.0 {
                let keep = protected.filter(|(p, _)| *p == k).map_or(0.0, |(_, kg)| kg);
                let free = (stores.get(k).copied().unwrap_or(0.0) - keep).max(0.0);
                let used = free.min(amount);
                if let Some(s) = stores.get_mut(k) {
                    *s -= used;
                }
                amount -= used;
            }
        }
    };
    for &(g, amount) in &recipe.session_inputs {
        take(g, amount);
    }
    for &(g, amount) in &recipe.inputs {
        take(g, amount * units);
    }
    for &(g, amount) in &recipe.outputs {
        let q = if goods.get(g).is_some_and(|d| d.tool.is_some()) {
            quality
        } else {
            1.0
        };
        if let Some(s) = stores.get_mut(g) {
            *s += amount * units * q;
        }
    }
    units
}

/// Wears the tools `tools` by `hours` of use: each loses `hours / life_h` of a standard tool,
/// and a tool worn out is gone.
pub fn wear(stores: &mut [f64], goods: &[GoodDef], tools: &[usize], hours: f64) {
    for &t in tools {
        if let (Some(def), Some(s)) = (
            goods.get(t).and_then(|g| g.tool.as_ref()),
            stores.get_mut(t),
        ) && def.life_h > 0.0
        {
            *s = (*s - hours.max(0.0) / def.life_h).max(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::{Eaten, GoodUse, ToolDef};

    fn good(id: &str, purpose: GoodUse, kcal: f64, eaten: Eaten) -> GoodDef {
        GoodDef {
            id: id.into(),
            name: id.into(),
            purpose,
            kcal_per_kg: kcal,
            half_life_days: 0.0,
            sheltered_half_life_days: 0.0,
            eaten,
            shared: false,
            reserve_for: None,
            tool: None,
        }
    }

    /// grain, seed, flour, bread, firewood, quern, oven.
    fn goods() -> Vec<GoodDef> {
        let mut seed = good("seed", GoodUse::Food, 3340.0, Eaten::Never);
        seed.reserve_for = Some(0);
        let mut quern = good("quern", GoodUse::Tool, 0.0, Eaten::Never);
        quern.tool = Some(ToolDef {
            life_h: 100.0,
            per_worker: 0.5,
            fixed: false,
        });
        let mut oven = good("oven", GoodUse::Tool, 0.0, Eaten::Never);
        oven.tool = Some(ToolDef {
            life_h: 1000.0,
            per_worker: 0.0,
            fixed: true,
        });
        vec![
            good("grain", GoodUse::Food, 3340.0, Eaten::Never),
            seed,
            good("flour", GoodUse::Food, 3340.0, Eaten::Never),
            good("bread", GoodUse::Food, 2570.0, Eaten::Raw),
            good("wood", GoodUse::Fuel, 0.0, Eaten::Never),
            quern,
            oven,
        ]
    }

    fn grind() -> RecipeDef {
        RecipeDef {
            id: "grind".into(),
            name: "Grind".into(),
            inputs: vec![(0, 1.0)],
            outputs: vec![(2, 0.95)],
            session_inputs: Vec::new(),
            unit_h: 0.5,
            session_h: 0.0,
            max_units: 0.0,
            tools: vec![5],
            skill: None,
            technique: None,
        }
    }

    fn oven_bake() -> RecipeDef {
        RecipeDef {
            id: "bake".into(),
            name: "Bake".into(),
            inputs: vec![(2, 1.0), (4, 0.1)],
            outputs: vec![(3, 1.3)],
            session_inputs: vec![(4, 20.0)],
            unit_h: 0.1,
            session_h: 2.0,
            max_units: 40.0,
            tools: vec![6],
            skill: None,
            technique: None,
        }
    }

    #[test]
    fn a_household_wants_tools_by_its_adults_and_keeps_a_spare_in_hand() {
        let goods = goods();
        let wants = tool_wants(&goods, &|_| 3);
        assert_eq!(wants[5], 2.0, "half a quern a worker, rounded up");
        assert_eq!(wants[6], 0.0, "nobody wants an oven of their own");
        assert_eq!(wants[0], 0.0);
        assert_eq!(tool_need(2.0, 0.6), 1.0);
        assert!((tool_need(2.0, 2.0) - SPARE_TOOL).abs() < 1e-12);
        assert_eq!(tool_need(2.0, 2.5), 0.0);
        assert_eq!(tool_need(0.0, 0.0), 0.0);
    }

    #[test]
    fn seed_goes_into_the_quern_only_in_hunger_and_never_the_seed_for_the_fields() {
        let goods = goods();
        let r = grind();
        let mut stores = vec![2.0, 30.0, 0.0, 0.0, 0.0, 1.0, 0.0];
        let have = |s: &[f64], ok| available(s, &goods, 0, ok, Some((1, 20.0)));
        assert_eq!(have(&stores, false), 2.0);
        assert_eq!(
            have(&stores, true),
            12.0,
            "10 kg of seed beyond the 20 kept"
        );
        let mut fed = stores.clone();
        assert_eq!(apply(&r, &mut fed, &goods, 50.0, 1.0, false, None), 2.0);
        assert_eq!(fed[1], 30.0, "no seed without hunger");
        let made = apply(&r, &mut stores, &goods, 50.0, 1.0, true, Some((1, 20.0)));
        assert!((made - 12.0).abs() < 1e-12);
        assert!(stores[0].abs() < 1e-12 && (stores[1] - 20.0).abs() < 1e-12);
        assert!((stores[2] - 12.0 * 0.95).abs() < 1e-12);
    }

    #[test]
    fn an_oven_needs_its_firing_whatever_the_batch_and_bakes_no_more_than_it_holds() {
        let goods = goods();
        let r = oven_bake();
        let have = |s: &[f64]| {
            let s = s.to_vec();
            units_from_inputs(&r, &move |g| s[g])
        };
        assert_eq!(
            have(&[0.0, 0.0, 50.0, 0.0, 10.0, 0.0, 1.0]),
            0.0,
            "no fuel to heat it"
        );
        // 25 kg of fuel: 20 to heat the oven, 5 left for 50 units' 0.1 each.
        assert!((have(&[0.0, 0.0, 60.0, 0.0, 25.0, 0.0, 1.0]) - 50.0).abs() < 1e-9);
        // Time: two hours of firing, then 0.1 h a unit, at most 40 units.
        assert_eq!(units_in(&r, 60.0, 1.0), 0.0);
        assert!((units_in(&r, 180.0, 1.0) - 10.0).abs() < 1e-9);
        assert_eq!(units_in(&r, 6000.0, 1.0), 40.0);
        assert!((minutes_for(&r, 10.0, 1.0) - 180.0).abs() < 1e-9);
        let mut stores = vec![0.0, 0.0, 60.0, 0.0, 25.0, 0.0, 1.0];
        let made = apply(&r, &mut stores, &goods, 40.0, 1.0, false, None);
        assert_eq!(made, 40.0);
        assert!((stores[3] - 52.0).abs() < 1e-9 && (stores[4] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn use_wears_a_tool_down_to_nothing_and_skill_makes_tools_that_last() {
        let goods = goods();
        let mut stores = vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.5, 0.0];
        wear(&mut stores, &goods, &[5], 30.0);
        assert!((stores[5] - 1.2).abs() < 1e-12);
        wear(&mut stores, &goods, &[5], 500.0);
        assert_eq!(stores[5], 0.0);
        let make_quern = RecipeDef {
            id: "q".into(),
            name: "q".into(),
            inputs: Vec::new(),
            outputs: vec![(5, 1.0)],
            session_inputs: Vec::new(),
            unit_h: 20.0,
            session_h: 0.0,
            max_units: 1.0,
            tools: Vec::new(),
            skill: None,
            technique: None,
        };
        apply(&make_quern, &mut stores, &goods, 1.0, 1.4, false, None);
        assert!(
            (stores[5] - 1.4).abs() < 1e-12,
            "a master's quern lasts longer"
        );
    }
}
