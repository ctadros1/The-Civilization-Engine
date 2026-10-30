//! Recipes and tools as arithmetic on a household's stores (ADR-0006 §1–2): what a session of a
//! recipe can make, what it takes and gives, what tools a household wants and how use wears them.
//! Pure functions; the population applies them.

use crate::params::{GoodDef, GoodUse, RecipeDef};

/// Spare tool a household keeps beyond the ones its members use, in standard tools: it makes the
/// next one while the last is still worn down to this (a tuning value).
pub const SPARE_TOOL: f64 = 0.25;

/// Tools a household wants, by good index: `per_worker` of each tool for each member old enough
/// for the work that needs it (`workers(tool)`), rounded up, and at least one of any tool it
/// wants at all. `None` workers: no work the household can do needs the tool, and it wants none.
pub fn tool_wants(goods: &[GoodDef], workers: &dyn Fn(usize) -> Option<usize>) -> Vec<f64> {
    goods
        .iter()
        .enumerate()
        .map(|(i, g)| match (&g.tool, workers(i)) {
            (Some(t), Some(n)) if t.per_worker > 0.0 => (t.per_worker * n as f64).ceil().max(1.0),
            _ => 0.0,
        })
        .collect()
}

/// Tools a household whose members are `ages` years old wants (see [`tool_wants`]): for each
/// tool, its members old enough for the work that needs it, or grown (`grown` years) for a tool
/// no work needs. Work needing a technique nobody in the household knows (`known`) does not
/// count (ADR-0008 §1): a household wants no tool for work it cannot do.
pub fn tool_wants_for(
    catalog: &crate::params::Catalog,
    ages: &[f64],
    grown: f64,
    known: &dyn Fn(usize) -> bool,
) -> Vec<f64> {
    let youngest = tool_ages(catalog, known);
    tool_wants(&catalog.goods, &|t| {
        // A tool that serves for another is wanted as that one is (M7d slice CE).
        if catalog.goods[t]
            .tool
            .as_ref()
            .is_some_and(|x| x.serves.is_some())
        {
            return None;
        }
        let from = match youngest.get(t).copied().unwrap_or(ToolAge::NoWork) {
            ToolAge::From(age) => age,
            ToolAge::NoWork => grown,
            ToolAge::Unknown => return None,
        };
        Some(ages.iter().filter(|&&a| a >= from).count())
    })
}

/// The youngest age at which anyone does work that needs a tool (see [`tool_ages`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ToolAge {
    /// Work from this age needs it.
    From(f64),
    /// No work needs it.
    NoWork,
    /// Only work the household does not know needs it.
    Unknown,
}

/// The youngest age at which anyone does work that needs each tool, by good index: the lowest
/// `min_age_years` of the activities that need it, their own or their recipe's, among those
/// whose technique is `known` or that need none.
pub fn tool_ages(catalog: &crate::params::Catalog, known: &dyn Fn(usize) -> bool) -> Vec<ToolAge> {
    let mut ages = vec![ToolAge::NoWork; catalog.goods.len()];
    // Each technique is asked about once: many activities share one.
    let mut asked: Vec<Option<bool>> = vec![None; catalog.techniques.len()];
    let mut knows = |t: usize| match asked.get_mut(t) {
        Some(slot) => *slot.get_or_insert_with(|| known(t)),
        None => known(t),
    };
    for a in &catalog.activities {
        // Work that needs no tool is not asked about: most do, and asking reads every member.
        if catalog.tools_of(a).next().is_none() {
            continue;
        }
        let usable = catalog.technique_of(a).is_none_or(&mut knows);
        for t in catalog.tools_of(a) {
            if let Some(age) = ages.get_mut(t) {
                *age = match (*age, usable) {
                    (ToolAge::From(x), true) => ToolAge::From(x.min(a.min_age_years)),
                    (_, true) => ToolAge::From(a.min_age_years),
                    (ToolAge::From(x), false) => ToolAge::From(x),
                    (_, false) => ToolAge::Unknown,
                };
            }
        }
    }
    ages
}

/// The food a recipe keeps from spoiling, if it is one that preserves (drying, smoking): its one
/// food input, by good index, when what it makes keeps at least [`PRESERVES_FACTOR`] times
/// longer. Making other food ready (grinding, baking) is not preserving.
pub fn preserves(recipe: &RecipeDef, goods: &[GoodDef]) -> Option<usize> {
    let keeps = |g: &GoodDef| {
        if g.half_life_days > 0.0 {
            g.half_life_days
        } else {
            f64::INFINITY
        }
    };
    let &(out, _) = recipe.outputs.first()?;
    let out = goods.get(out).filter(|g| g.purpose == GoodUse::Food)?;
    let mut food = recipe
        .inputs
        .iter()
        .filter(|&&(g, _)| goods.get(g).is_some_and(|d| d.purpose == GoodUse::Food));
    let &(input, _) = food.next()?;
    if food.next().is_some() {
        return None;
    }
    let inp = goods.get(input)?;
    (inp.half_life_days > 0.0 && keeps(out) >= PRESERVES_FACTOR * inp.half_life_days)
        .then_some(input)
}

/// How many times longer a recipe's food must keep than its food input for the recipe to count
/// as preserving it (a tuning value).
pub const PRESERVES_FACTOR: f64 = 4.0;

/// How much of a tool a household still wants, 0–1, given what it wants and holds: a whole tool
/// when it is short of one, less as the spare it keeps is worn down.
pub fn tool_need(wanted: f64, held: f64) -> f64 {
    if wanted <= 0.0 {
        0.0
    } else {
        (wanted + SPARE_TOOL - held.max(0.0)).clamp(0.0, 1.0)
    }
}

/// Tool `t` and the tools that serve for it (a copper axe for an axe), by good index, `t` first.
pub fn serving(goods: &[GoodDef], t: usize) -> impl Iterator<Item = usize> + '_ {
    std::iter::once(t).chain(
        goods
            .iter()
            .enumerate()
            .filter(move |(g, d)| *g != t && d.tool.as_ref().is_some_and(|x| x.serves == Some(t)))
            .map(|(g, _)| g),
    )
}

/// The tool good `g` serves for, if it serves for one; else `g` itself.
pub fn served(goods: &[GoodDef], g: usize) -> usize {
    goods
        .get(g)
        .and_then(|d| d.tool.as_ref())
        .and_then(|t| t.serves)
        .unwrap_or(g)
}

/// The recipe, by index, a household makes tool `t` by (M7d slice CE): `t`'s first recipe, or
/// a recipe of a tool that serves for it which the household can work (`can_work`), whichever
/// costs it least of its own hours (`costs`, hours a unit, asked only when there is a choice)
/// for each hour of use. A way whose cost it cannot tell is never chosen over `t`'s own.
pub fn way_to_tool(
    catalog: &crate::params::Catalog,
    t: usize,
    can_work: &dyn Fn(&RecipeDef) -> bool,
    costs: &dyn Fn() -> Vec<Option<f64>>,
) -> Option<usize> {
    let makes = |r: &RecipeDef, g: usize| r.outputs.iter().any(|&(o, _)| o == g);
    let own = catalog.recipes.iter().position(|r| makes(r, t));
    let others: Vec<(usize, usize)> = serving(&catalog.goods, t)
        .skip(1)
        .filter_map(|g| {
            catalog
                .recipes
                .iter()
                .position(|r| makes(r, g) && can_work(r))
                .map(|r| (g, r))
        })
        .collect();
    if others.is_empty() {
        return own;
    }
    let costs = costs();
    let per_hour = |g: usize| {
        let life = life_of(&catalog.goods, g);
        costs
            .get(g)
            .copied()
            .flatten()
            .filter(|_| life > 0.0)
            .map(|c| c / life)
    };
    let mut best = own.map(|r| (r, per_hour(t)));
    for (g, r) in others {
        let Some(h) = per_hour(g) else {
            continue;
        };
        if best.is_none_or(|(_, b)| b.is_none_or(|b| h < b)) {
            best = Some((r, Some(h)));
        }
    }
    best.map(|(r, _)| r)
}

/// The kilowatts of current the whole dynamos in `stores` give from `shaft_kw` of a wheel's or an
/// engine's work (M11a, second part; ADR-0036 §2): each kind, in the catalogue's order, takes as
/// much of the shaft as its whole machines' ratings allow and turns its `efficiency` of it into
/// current. Nothing without a whole dynamo or a turning shaft.
pub fn dynamo_kw(goods: &[GoodDef], stores: &[f64], shaft_kw: f64) -> f64 {
    let mut left = shaft_kw.max(0.0);
    let mut out = 0.0;
    for (g, d) in goods.iter().enumerate() {
        let Some(p) = d
            .power
            .filter(|p| p.source == crate::params::PowerSource::Shaft)
        else {
            continue;
        };
        let whole = (stores.get(g).copied().unwrap_or(0.0) + 1e-9)
            .floor()
            .max(0.0);
        if whole <= 0.0 || p.efficiency <= 0.0 || left <= 0.0 {
            continue;
        }
        let take = left.min(whole * p.rated_kw / p.efficiency);
        out += take * p.efficiency;
        left -= take;
    }
    out
}

/// The kilowatts of shaft work the whole dynamos in `stores` take to give `current_kw` of current
/// (M11b, first part): [`dynamo_kw`] turned about, each kind in the catalogue's order giving as
/// much as its whole machines' ratings allow at its `efficiency`. What they cannot give takes no
/// shaft.
pub fn dynamo_shaft_kw(goods: &[GoodDef], stores: &[f64], current_kw: f64) -> f64 {
    let mut left = current_kw.max(0.0);
    let mut shaft = 0.0;
    for (g, d) in goods.iter().enumerate() {
        let Some(p) = d
            .power
            .filter(|p| p.source == crate::params::PowerSource::Shaft)
        else {
            continue;
        };
        let whole = (stores.get(g).copied().unwrap_or(0.0) + 1e-9)
            .floor()
            .max(0.0);
        if whole <= 0.0 || p.efficiency <= 0.0 || left <= 0.0 {
            continue;
        }
        let give = left.min(whole * p.rated_kw);
        shaft += give / p.efficiency;
        left -= give;
    }
    shaft
}

/// Hours of use a standard tool of `g` lasts (0 for no tool).
pub fn life_of(goods: &[GoodDef], g: usize) -> f64 {
    goods
        .get(g)
        .and_then(|d| d.tool.as_ref())
        .map_or(0.0, |t| t.life_h)
}

/// Standard tools at hand for work that needs tool `t`: in use of its own and of every tool that
/// serves for it, from what is held of each (`held`).
pub fn tools_for(goods: &[GoodDef], t: usize, held: &dyn Fn(usize) -> f64) -> f64 {
    serving(goods, t).map(|g| in_use(goods, g, held(g))).sum()
}

/// How much of tool `good`, of which `held` standard tools are at hand, does its work: all of it,
/// but none of a tool that stays where it is made (an oven) unless a whole one stands. One partly
/// built does nothing, and one worn below whole is out of use until it is mended.
pub fn in_use(goods: &[GoodDef], good: usize, held: f64) -> f64 {
    let fixed = goods
        .get(good)
        .and_then(|g| g.tool.as_ref())
        .is_some_and(|t| t.fixed);
    if fixed && held < 1.0 - 1e-9 {
        0.0
    } else {
        held.max(0.0)
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
    let units = start(recipe, stores, goods, units, reserve_ok, protected);
    give(recipe, stores, goods, units, quality);
    units
}

/// Starts `units` of `recipe` on `stores`: takes its inputs as [`apply`] does, but gives nothing
/// yet (a batch that runs for its `elapsed_days`, M10a slice DP). Returns the units started, no
/// more than the inputs allowed.
pub fn start(
    recipe: &RecipeDef,
    stores: &mut [f64],
    goods: &[GoodDef],
    units: f64,
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
    units
}

/// The content id of the good a works' fuel leaves its ash as (ADR-0038 §2). A catalogue
/// without it keeps no ash from making.
pub const ASH: &str = "core:good/ash";

/// Kilograms of ash a session making `units` of `recipe` leaves (ADR-0038 §2): each fuel it takes,
/// by unit and by session, times its `ash_share`, less the ash the fuels it gives keep (coke keeps
/// its coal's ash, charcoal its wood's), never below none.
pub fn ash_left(recipe: &RecipeDef, goods: &[GoodDef], units: f64) -> f64 {
    let share = |g: usize| {
        goods
            .get(g)
            .and_then(|d| d.fuel)
            .map_or(0.0, |f| f.ash_share.max(0.0))
    };
    let burned: f64 = recipe
        .session_inputs
        .iter()
        .map(|&(g, a)| a * share(g))
        .chain(recipe.inputs.iter().map(|&(g, a)| a * units * share(g)))
        .sum();
    let kept: f64 = recipe
        .outputs
        .iter()
        .map(|&(g, a)| a * units * share(g))
        .sum();
    (burned - kept).max(0.0)
}

/// Leaves in `stores`, as `ash`, the ash of `kg` of fuel good `fuel` burned in an engine
/// (ADR-0038 §5): `kg` times its `ash_share`. Returns the ash good's index and the kilograms left;
/// `None` where it leaves none or the catalogue has no ash.
pub fn leave_ash(
    goods: &[GoodDef],
    stores: &mut [f64],
    fuel: usize,
    kg: f64,
) -> Option<(usize, f64)> {
    let share = goods
        .get(fuel)
        .and_then(|d| d.fuel)
        .map_or(0.0, |f| f.ash_share.max(0.0));
    let ash = kg.max(0.0) * share;
    if !(ash > 0.0 && ash.is_finite()) {
        return None;
    }
    let a = goods.iter().position(|d| d.id == ASH)?;
    *stores.get_mut(a)? += ash;
    Some((a, ash))
}

/// Gives the outputs of `units` of `recipe` to `stores`, tools made worth `quality` standard
/// tools each, and the ash its fuel leaves ([`ash_left`], ADR-0038 §2): the end of a session, or
/// the opening of a batch (M10a slice DP).
pub fn give(recipe: &RecipeDef, stores: &mut [f64], goods: &[GoodDef], units: f64, quality: f64) {
    if !(units > 0.0 && units.is_finite()) {
        return;
    }
    let ash = ash_left(recipe, goods, units);
    if ash > 0.0
        && let Some(s) = goods
            .iter()
            .position(|d| d.id == ASH)
            .and_then(|a| stores.get_mut(a))
    {
        *s += ash;
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
}

/// Wears the tools `tools` by `hours` of use: each loses `hours / life_h` of a standard tool,
/// and a tool worn out is gone.
/// The room for good `good` in the whole holders `stores` holds, kilograms (M10b slice DY;
/// ADR-0034 §8), or `None` for a good no holder keeps.
pub fn holder_room(goods: &[GoodDef], stores: &[f64], good: usize) -> Option<f64> {
    let mut room = None;
    for (h, d) in goods.iter().enumerate() {
        let Some(x) = d.holder.filter(|x| x.good == Some(good)) else {
            continue;
        };
        let whole = (stores.get(h).copied().unwrap_or(0.0) + 1e-9)
            .floor()
            .max(0.0);
        *room.get_or_insert(0.0) += whole * x.kg;
    }
    room
}

/// Keeps each good that only a holder keeps within the room of the whole holders held for it
/// (M10b slice DY; ADR-0034 §8: gas is stored, not made on demand): what lies beyond, or all of
/// it with no holder, is lost to the air. Returns what was lost, by good; nothing in a world
/// without holders.
pub fn keep_to_holders(goods: &[GoodDef], stores: &mut [f64]) -> Vec<(usize, f64)> {
    let mut lost = Vec::new();
    for d in goods {
        let Some(g) = d.holder.and_then(|x| x.good) else {
            continue;
        };
        if lost.iter().any(|&(l, _)| l == g) {
            continue;
        }
        let held = stores.get(g).copied().unwrap_or(0.0);
        if held <= 0.0 {
            continue;
        }
        let room = holder_room(goods, stores, g).unwrap_or(0.0);
        if held > room + 1e-12 {
            stores[g] = room;
            lost.push((g, held - room));
        }
    }
    lost
}

pub fn wear(stores: &mut [f64], goods: &[GoodDef], tools: &[usize], hours: f64) {
    for &t in tools {
        // The work is done with the longest-lived of the tools at hand that do it (M7d slice CE).
        let t = serving(goods, t)
            .filter(|&g| in_use(goods, g, stores.get(g).copied().unwrap_or(0.0)) > 1e-9)
            .max_by(|&a, &b| {
                life_of(goods, a)
                    .total_cmp(&life_of(goods, b))
                    .then(b.cmp(&a))
            })
            .unwrap_or(t);
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

    #[test]
    fn a_dynamo_turns_its_share_of_the_shaft_into_current_up_to_its_rating() {
        let mut dynamo = good("dynamo", GoodUse::Tool, 0.0, Eaten::Never);
        dynamo.power = Some(crate::params::GoodPower {
            source: crate::params::PowerSource::Shaft,
            rated_kw: 10.0,
            efficiency: 0.8,
            fuel: None,
        });
        let mut wheel = good("wheel", GoodUse::Tool, 0.0, Eaten::Never);
        wheel.power = Some(crate::params::GoodPower {
            source: crate::params::PowerSource::Water,
            rated_kw: 3.0,
            efficiency: 0.3,
            fuel: None,
        });
        let goods = vec![wheel, dynamo];
        // 3 kW of shaft through one dynamo: 2.4 kW of current.
        assert!((dynamo_kw(&goods, &[1.0, 1.0], 3.0) - 2.4).abs() < 1e-9);
        // 20 kW of shaft: one dynamo is full at 10 kW, two give 16.
        assert!((dynamo_kw(&goods, &[0.0, 1.0], 20.0) - 10.0).abs() < 1e-9);
        assert!((dynamo_kw(&goods, &[0.0, 2.0], 20.0) - 16.0).abs() < 1e-9);
        // A worn one is no whole machine, and a still shaft gives nothing; a wheel is no dynamo.
        assert_eq!(dynamo_kw(&goods, &[0.0, 0.9], 3.0), 0.0);
        assert_eq!(dynamo_kw(&goods, &[0.0, 1.0], 0.0), 0.0);
        assert_eq!(dynamo_kw(&goods, &[5.0, 0.0], 3.0), 0.0);
        // Turned about (M11b, first part): 2.4 kW of current took 3 kW of shaft; what the dynamos
        // cannot give takes none.
        assert!((dynamo_shaft_kw(&goods, &[1.0, 1.0], 2.4) - 3.0).abs() < 1e-9);
        assert!((dynamo_shaft_kw(&goods, &[0.0, 1.0], 25.0) - 12.5).abs() < 1e-9);
        assert_eq!(dynamo_shaft_kw(&goods, &[0.0, 0.0], 1.0), 0.0);
        for kw in [0.5, 3.0, 9.0, 20.0] {
            let c = dynamo_kw(&goods, &[0.0, 2.0], kw);
            assert!((dynamo_shaft_kw(&goods, &[0.0, 2.0], c) - kw).abs() < 1e-9);
        }
    }

    fn good(id: &str, purpose: GoodUse, kcal: f64, eaten: Eaten) -> GoodDef {
        GoodDef {
            fuel: None,
            power: None,
            light: None,
            holder: None,
            telegraph: None,
            money: None,
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
            timber: None,
            store: None,
            fire: None,
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
            serves: None,
            crew: 1,
        });
        let mut oven = good("oven", GoodUse::Tool, 0.0, Eaten::Never);
        oven.tool = Some(ToolDef {
            life_h: 1000.0,
            per_worker: 0.0,
            fixed: true,
            serves: None,
            crew: 1,
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

    #[test]
    fn gas_keeps_to_the_room_of_its_whole_holders_and_the_rest_goes_to_the_air() {
        let mut holder = good("holder", GoodUse::Tool, 0.0, Eaten::Never);
        holder.tool = Some(ToolDef {
            life_h: 50_000.0,
            per_worker: 0.0,
            fixed: true,
            serves: None,
            crew: 1,
        });
        holder.holder = Some(crate::params::GoodHolder {
            good: Some(0),
            kg: 150.0,
        });
        let goods = vec![
            good("gas", GoodUse::Material, 0.0, Eaten::Never),
            holder,
            good("grain", GoodUse::Food, 3340.0, Eaten::Never),
        ];
        // Two whole holders hold 300 kg; grain no holder keeps.
        let mut stores = vec![400.0, 2.0, 50.0];
        assert_eq!(holder_room(&goods, &stores, 0), Some(300.0));
        assert_eq!(holder_room(&goods, &stores, 2), None);
        assert_eq!(keep_to_holders(&goods, &mut stores), vec![(0, 100.0)]);
        assert_eq!(stores, vec![300.0, 2.0, 50.0]);
        // Within the room nothing is lost; a holder short of whole holds nothing.
        assert!(keep_to_holders(&goods, &mut stores).is_empty());
        stores[1] = 0.9;
        assert_eq!(keep_to_holders(&goods, &mut stores), vec![(0, 300.0)]);
        assert_eq!(stores, vec![0.0, 0.9, 50.0]);
    }

    #[test]
    fn a_tool_that_stays_where_it_is_made_works_only_whole() {
        let goods = goods();
        let (quern, oven) = (5, 6);
        assert_eq!(
            in_use(&goods, oven, 0.5),
            0.0,
            "a half-built oven bakes nothing"
        );
        assert_eq!(in_use(&goods, oven, 0.999), 0.0, "nor one worn below whole");
        assert_eq!(in_use(&goods, oven, 1.2), 1.2);
        assert_eq!(
            in_use(&goods, quern, 0.05),
            0.05,
            "the last of a worn quern grinds"
        );
        assert_eq!(in_use(&goods, 0, 3.0), 3.0, "nor are other goods held back");
    }

    fn grind() -> RecipeDef {
        RecipeDef {
            elapsed_days: 0.0,
            kwh_per_unit: 0.0,
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
            elapsed_days: 0.0,
            kwh_per_unit: 0.0,
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
    fn drying_preserves_food_and_grinding_does_not() {
        let mut goods = goods();
        let meat = goods.len();
        goods.push(GoodDef {
            fuel: None,
            power: None,
            light: None,
            holder: None,
            telegraph: None,
            half_life_days: 3.0,
            ..good("meat", GoodUse::Food, 1500.0, Eaten::Cooked)
        });
        let dried = goods.len();
        goods.push(GoodDef {
            fuel: None,
            power: None,
            light: None,
            holder: None,
            telegraph: None,
            half_life_days: 180.0,
            ..good("dried", GoodUse::Food, 3750.0, Eaten::Raw)
        });
        let dry = RecipeDef {
            elapsed_days: 0.0,
            kwh_per_unit: 0.0,
            inputs: vec![(meat, 1.0)],
            outputs: vec![(dried, 0.4)],
            ..grind()
        };
        assert_eq!(preserves(&dry, &goods), Some(meat));
        assert_eq!(
            preserves(&grind(), &goods),
            None,
            "grinding keeps nothing from spoiling"
        );
        let barely = RecipeDef {
            elapsed_days: 0.0,
            kwh_per_unit: 0.0,
            outputs: vec![(meat, 1.0)],
            ..dry.clone()
        };
        assert_eq!(preserves(&barely, &goods), None);
    }

    #[test]
    fn a_household_wants_tools_by_its_adults_and_keeps_a_spare_in_hand() {
        let goods = goods();
        let wants = tool_wants(&goods, &|_| Some(3));
        assert_eq!(wants[5], 2.0, "half a quern a worker, rounded up");
        assert_eq!(wants[6], 0.0, "nobody wants an oven of their own");
        assert_eq!(wants[0], 0.0);
        // A tool only work the household does not know needs is not wanted.
        assert_eq!(tool_wants(&goods, &|_| None)[5], 0.0);
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
            elapsed_days: 0.0,
            kwh_per_unit: 0.0,
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

    fn recipe(id: &str, out: usize, inputs: Vec<(usize, f64)>, unit_h: f64) -> RecipeDef {
        RecipeDef {
            elapsed_days: 0.0,
            kwh_per_unit: 0.0,
            id: id.to_owned(),
            name: id.to_owned(),
            inputs,
            outputs: vec![(out, 1.0)],
            session_inputs: Vec::new(),
            unit_h,
            session_h: 0.0,
            max_units: 1.0,
            tools: Vec::new(),
            skill: None,
            technique: None,
        }
    }

    /// Stone (0), an axe (1, 200 hours) and a copper axe (2, 600 hours) serving for it.
    fn axes() -> crate::params::Catalog {
        let mut stone = good("stone", GoodUse::Material, 0.0, Eaten::Never);
        stone.tool = None;
        let mut axe = good("axe", GoodUse::Tool, 0.0, Eaten::Never);
        axe.tool = Some(ToolDef {
            life_h: 200.0,
            per_worker: 0.5,
            fixed: false,
            serves: None,
            crew: 1,
        });
        let mut copper = good("copper_axe", GoodUse::Tool, 0.0, Eaten::Never);
        copper.tool = Some(ToolDef {
            life_h: 600.0,
            per_worker: 0.5,
            fixed: false,
            serves: Some(1),
            crew: 1,
        });
        crate::params::Catalog {
            goods: vec![stone, axe, copper],
            recipes: vec![
                recipe("make_axe", 1, vec![(0, 1.5)], 15.0),
                recipe("cast_axe", 2, Vec::new(), 20.0),
            ],
            ..crate::params::Catalog::default()
        }
    }

    #[test]
    fn a_tool_serving_for_another_does_its_work_and_is_worn_first() {
        let c = axes();
        assert_eq!(serving(&c.goods, 1).collect::<Vec<_>>(), vec![1, 2]);
        assert_eq!(served(&c.goods, 2), 1);
        let held = [0.0, 0.5, 1.0];
        assert!((tools_for(&c.goods, 1, &|g| held[g]) - 1.5).abs() < 1e-12);
        // Work needing an axe wears the copper axe, the longer-lived, while one is at hand.
        let mut stores = held.to_vec();
        wear(&mut stores, &c.goods, &[1], 60.0);
        assert!((stores[1] - 0.5).abs() < 1e-12, "the stone axe is not used");
        assert!(
            (stores[2] - 0.9).abs() < 1e-12,
            "the copper axe wears by 60/600"
        );
        // And the stone axe once no copper axe is left.
        let mut stores = vec![0.0, 0.5, 0.0];
        wear(&mut stores, &c.goods, &[1], 20.0);
        assert!((stores[1] - 0.4).abs() < 1e-12);
        // A tool serving for another is not wanted for itself.
        let wants = tool_wants_for(&c, &[30.0, 30.0], 16.0, &|_| true);
        assert_eq!(wants[2], 0.0);
    }

    #[test]
    fn a_household_makes_a_tool_the_way_that_costs_it_least_for_each_hour_of_use() {
        let c = axes();
        let yes = |_: &RecipeDef| true;
        // A stone axe at 16 hours for 200 hours' use; a copper axe at 30 hours for 600.
        let costs = || vec![Some(1.0 / 1.5), Some(16.0), Some(30.0)];
        assert_eq!(way_to_tool(&c, 1, &yes, &costs), Some(1), "the copper axe");
        // Dearer for each hour of use, or of a cost it cannot tell: its own way.
        let dear = || vec![Some(1.0 / 1.5), Some(16.0), Some(60.0)];
        assert_eq!(way_to_tool(&c, 1, &yes, &dear), Some(0));
        let unknown = || vec![Some(1.0 / 1.5), Some(16.0), None];
        assert_eq!(way_to_tool(&c, 1, &yes, &unknown), Some(0));
        // A way it cannot work is never weighed, and its costs never asked.
        let no = |r: &RecipeDef| r.id != "cast_axe";
        let never = || -> Vec<Option<f64>> { panic!("costs asked with no choice") };
        assert_eq!(way_to_tool(&c, 1, &no, &never), Some(0));
    }
}
