//! Engines at pits (M10a slice DS, third part; ADR-0033 §7; the M10 power brief §1.4): an engine
//! set at a pit burns fuel to pump the water that comes into it, so its diggers need not bail.
//! Each day an engine whose owner holds the engine whole and fuel for it lifts out what stands in
//! its pit, at most its rated power for the hours it is kept going, burning the owner's fuels by
//! their heat at the engine's efficiency (research 07-06 §2.3: 3.6 / (MJ η) kg a kWh), and the
//! engine wears by the hours it ran.
//!
//! Who sets one (the fourth part): a household one of whose people knows how to build an engine
//! weighs, on its day of the year, each of its pits that it has bailed by hand for half a year or
//! more: the hours a year it bails there, against the fuel an engine would burn lifting the same
//! water and the engine itself, over the years its life lasts at that work. Where the hours saved
//! exceed both, it sets one at the pit with the most to gain, and only then does it want the
//! engine, which it makes or buys as it would any tool.

use super::digging;
use super::*;
use crate::params::PowerSource;
use civ_land::earth::EarthKind;
use civ_land::power::{Engine, fuel_kg, lift_kwh};

/// Purpose tag for the day of the year a household weighs an engine.
const PURPOSE_ENGINE_REVIEW: u64 = 0x656e_6769_6e72_7631; // "enginrv1"
/// The fewest days of bailing from which a year's bailing is read.
const MIN_BAILED_DAYS: i64 = 180;
/// The most years an engine's gain is counted over, whatever its life (a design prior: a
/// household looks no further ahead than a mill's ditch lasts).
const MAX_ENGINE_YEARS: f64 = 20.0;

/// The share of an engine's work its pump turns into lifting water (research 07-06 §2.2's 50 %
/// for lifting by a wheel or a pump; a tuning value).
const PUMP_EFFICIENCY: f64 = 0.5;

/// The hours a day an engine is kept going: fed, stoked and watched in daylight (a design prior;
/// its tending is not yet counted in anyone's hours).
const ENGINE_HOURS_A_DAY: f64 = 12.0;

/// How far a water wheel's work is carried by rods to a pit's pump, metres (M10a slice DT, fifth
/// part; a design prior: research 07-06 §5 names pumping among a wheel's work but gives no reach).
const ROD_REACH_M: f64 = 300.0;

/// The hours a day a wheel pumps a pit, as long as an engine is kept going (a design prior: the
/// wheel's other work, a mill's or a hammer's, is taken in the same daylight and not netted off).
const WHEEL_PUMP_HOURS: f64 = 12.0;

impl Population {
    /// Every working water wheel pumps the flooded pits its holder dug within its rods' reach
    /// (M10a slice DT, fifth part; ADR-0033 §7's "by a wheel"): at the wheel's power today for
    /// its hours, nearest pit first, before any engine burns fuel at them. A wheel held by a
    /// workshop pumps its owners' pits.
    pub(super) fn wheels_day(&mut self, ctx: &mut Ctx) {
        let (now, params) = (ctx.now, ctx.params);
        for i in 0..ctx.land.installations.len() {
            let x = ctx.land.installations[i];
            let kw = ctx.land.installation_kw(ctx.land_params, i);
            if kw <= 0.0 {
                continue;
            }
            let holder = self.firm(x.owner).map_or(x.owner, |f| f.owner);
            let mut pits: Vec<(f64, PermanentId)> = ctx
                .land
                .earthworks
                .iter()
                .filter(|w| w.kind == EarthKind::Pit && w.household == holder)
                .filter_map(|w| {
                    let (cx, cy) = w.rect.centre_m();
                    let d = (f64::from(cx) - x.at[0]).hypot(f64::from(cy) - x.at[1]);
                    (d <= ROD_REACH_M).then_some((d, w.id))
                })
                .collect();
            pits.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
            let mut kwh = kw * WHEEL_PUMP_HOURS;
            for (_, pit) in pits {
                if kwh <= 1e-12 {
                    break;
                }
                let Some(state) = ctx.land.pit_water_now(
                    ctx.map,
                    ctx.land_params,
                    params.digging.influence_m,
                    pit,
                    now,
                ) else {
                    continue;
                };
                let need = lift_kwh(state.water_m3, state.lift_m(), PUMP_EFFICIENCY);
                if need <= 0.0 {
                    continue;
                }
                let done = need.min(kwh);
                kwh -= done;
                ctx.land
                    .set_pit_water(pit, state.water_m3 * (1.0 - done / need), now);
            }
        }
    }

    /// Every water wheel pumps its holder's pits now: for tests.
    #[doc(hidden)]
    pub fn wheels_day_for_tests(&mut self, ctx: &mut Ctx) {
        self.wheels_day(ctx);
    }

    /// The households whose day of the year it is weigh an engine (M10a slice DS, fourth part),
    /// where the content has a way of making one at all and one of their people knows it.
    pub(super) fn engine_reviews(&mut self, ctx: &mut Ctx, day: i64) {
        if ctx.land.pit_bailing.is_empty() {
            return;
        }
        let ways = engine_ways(ctx.catalog);
        if ways.is_empty() {
            return;
        }
        let mut due: Vec<PermanentId> = ctx
            .land
            .pit_bailing
            .iter()
            .filter_map(|b| ctx.land.earthworks.iter().find(|e| e.id == b.work))
            .map(|e| e.household)
            .collect();
        due.sort_unstable();
        due.dedup();
        due.retain(|h| {
            Rng64::from_key(&[ctx.seed, PURPOSE_ENGINE_REVIEW, h.get()]).next_u64()
                % DAYS_PER_YEAR as u64
                == day.rem_euclid(DAYS_PER_YEAR) as u64
        });
        for h in due {
            self.review_engine(ctx, h, &ways);
        }
    }

    /// Household `household` weighs an engine now, whatever its day of the year: for tests.
    #[doc(hidden)]
    pub fn review_engine_for_tests(&mut self, ctx: &mut Ctx, household: PermanentId) {
        let ways = engine_ways(ctx.catalog);
        self.review_engine(ctx, household, &ways);
    }

    /// Household `household` sets an engine at the pit of its own where one would most repay
    /// it, if any would: the hours a year it bails there less the fuel's, over the engine's
    /// years, against the engine's own cost, all in hours of its work.
    fn review_engine(&mut self, ctx: &mut Ctx, household: PermanentId, ways: &[(usize, usize)]) {
        let Some(x) = self.household(household) else {
            return;
        };
        if x.members.is_empty() {
            return;
        }
        let known: Vec<usize> = ways
            .iter()
            .filter(|&&(r, _)| {
                ctx.catalog.recipes[r]
                    .technique
                    .is_none_or(|t| self.household_knows(&x.members, t))
            })
            .map(|&(_, g)| g)
            .collect();
        if known.is_empty() {
            return;
        }
        let goods = &ctx.catalog.goods;
        let costs = self.own_costs_of(ctx, x);
        // The engine it can have for the fewest hours, and the fuel it burns for the fewest.
        let Some((engine, engine_h)) = known
            .iter()
            .filter_map(|&g| costs.get(g).copied().flatten().map(|h| (g, h)))
            .min_by(|a, b| a.1.total_cmp(&b.1))
        else {
            return;
        };
        let Some((life_h, power)) = goods[engine]
            .tool
            .as_ref()
            .map(|t| t.life_h)
            .zip(goods[engine].power)
        else {
            return;
        };
        let per_kwh = goods
            .iter()
            .enumerate()
            .filter(|(_, d)| d.purpose == GoodUse::Fuel)
            .filter_map(|(g, d)| {
                let f = d.fuel?;
                if f.technique
                    .is_some_and(|t| !self.household_knows(&x.members, t))
                {
                    return None;
                }
                let h = costs.get(g).copied().flatten()?;
                Some(h * fuel_kg(1.0, f.mj_per_kg, power.efficiency))
            })
            .filter(|h| h.is_finite() && *h >= 0.0)
            .min_by(f64::total_cmp);
        let rules = &ctx.params.digging;
        let now = ctx.now;
        let mut best: Option<(PermanentId, f64)> = None;
        for b in &ctx.land.pit_bailing {
            let days = now.day_index() - b.since.day_index();
            let Some(pit) =
                ctx.land.earthworks.iter().find(|e| {
                    e.id == b.work && e.kind == EarthKind::Pit && e.household == household
                })
            else {
                continue;
            };
            if days < MIN_BAILED_DAYS || ctx.land.engines.iter().any(|e| e.pit == b.work) {
                continue;
            }
            // Its fuel: the cheapest of the household's by their heat, or what the pit itself
            // yields where that is a fuel it burns, at the hours it takes to dig (an engine at a
            // pit head burns its own fuel, 07-06 §2.4).
            let at_pit = pit
                .deposit
                .and_then(|d| ctx.land.deposits.iter().find(|x| x.id == d))
                .and_then(|d| {
                    let g = usize::from(d.body.good);
                    let def = goods.get(g)?;
                    let f = def.fuel.filter(|_| def.purpose == GoodUse::Fuel)?;
                    if f.technique
                        .is_some_and(|t| !self.household_knows(&x.members, t))
                    {
                        return None;
                    }
                    let rates = digging::rates(ctx.land_params, &d.body, rules.h_per_m3);
                    let kg_h = digging::dig_kg_per_hour(&d.body, rates);
                    (kg_h > 0.0).then(|| fuel_kg(1.0, f.mj_per_kg, power.efficiency) / kg_h)
                });
            let Some(fuel_h_per_kwh) = [per_kwh, at_pit].into_iter().flatten().reduce(f64::min)
            else {
                continue;
            };
            // A year's bailing, the work it lifted, and that work by the engine and its fuel.
            let bail_h = b.hours * DAYS_PER_YEAR as f64 / days as f64;
            let lifted_kwh = bail_h * rules.bail_w * rules.bail_efficiency / 1_000.0;
            let kwh = lifted_kwh / PUMP_EFFICIENCY;
            let gain_h = bail_h - kwh * fuel_h_per_kwh;
            let run_h = kwh / power.rated_kw.max(1e-9);
            let years = (life_h / run_h.max(1e-9)).min(MAX_ENGINE_YEARS);
            let net_h = gain_h * years - engine_h;
            if net_h > 0.0 && best.is_none_or(|(_, n)| net_h > n) {
                best = Some((b.work, net_h));
            }
        }
        let Some((pit, _)) = best else {
            return;
        };
        let id = ctx.ids.allocate();
        ctx.land.engines.push(Engine {
            id,
            pit,
            owner: household,
            rated_kw: power.rated_kw,
            efficiency: power.efficiency,
        });
    }

    /// Every engine pumps its pit for the day (M10a slice DS, third part).
    pub(super) fn engines_day(&mut self, ctx: &mut Ctx) {
        for i in 0..ctx.land.engines.len() {
            let e = ctx.land.engines[i];
            self.pump(ctx, &e);
        }
    }

    /// Every engine pumps its pit now, whatever the hour: for tests.
    #[doc(hidden)]
    pub fn engines_day_for_tests(&mut self, ctx: &mut Ctx) {
        self.engines_day(ctx);
    }

    /// Sets an engine of good `good` that household `owner` holds at pit `pit`, for tests: the
    /// engine's record, with the good's rated power and efficiency.
    #[doc(hidden)]
    pub fn set_engine_for_tests(
        &mut self,
        ctx: &mut Ctx,
        owner: PermanentId,
        pit: PermanentId,
        good: usize,
    ) -> Option<PermanentId> {
        let power = ctx.catalog.goods.get(good)?.power?;
        if power.source != PowerSource::Fuel {
            return None;
        }
        let id = ctx.ids.allocate();
        ctx.land.engines.push(Engine {
            id,
            pit,
            owner,
            rated_kw: power.rated_kw,
            efficiency: power.efficiency,
        });
        Some(id)
    }

    /// Engine `e` lifts out what stands in its pit today, as far as its hours and its owner's
    /// fuel allow.
    fn pump(&mut self, ctx: &mut Ctx, e: &Engine) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let Some(state) = ctx.land.pit_water_now(
            ctx.map,
            ctx.land_params,
            params.digging.influence_m,
            e.pit,
            now,
        ) else {
            return;
        };
        if state.water_m3 <= 0.0 {
            return;
        }
        let Some(&hd) = self.hh_index.get(&e.owner) else {
            return;
        };
        let Some(o) = self.households.get_mut(hd) else {
            return;
        };
        let members = o.members.len().max(1);
        o.settle_stores(now, goods, &|d| fuel_per_day(params, members, d));
        o.stores.resize(goods.len(), 0.0);
        // The engine itself, whole, among the owner's things.
        // An engine that burns one named fuel is not set at a pit (M11b, first part).
        let Some(engine) = (0..goods.len()).find(|&g| {
            goods[g]
                .power
                .is_some_and(|p| p.source == PowerSource::Fuel && p.fuel.is_none())
                && o.stores[g] >= decide::MIN_TOOL
        }) else {
            return;
        };
        // The work it would take to lift it all out, and as much as its hours give.
        let lift = state.lift_m();
        let need_kwh = lift_kwh(state.water_m3, lift, PUMP_EFFICIENCY);
        let mut kwh = need_kwh.min(e.rated_kw * ENGINE_HOURS_A_DAY);
        // Its fuel: the owner's fuels, in the catalogue's order, by their heat.
        let before = o.stores.clone();
        let mut done = 0.0;
        for (g, d) in goods.iter().enumerate() {
            if kwh <= 1e-12 {
                break;
            }
            let Some(f) = d.fuel.filter(|_| d.purpose == GoodUse::Fuel) else {
                continue;
            };
            let per_kwh = fuel_kg(1.0, f.mj_per_kg, e.efficiency);
            if per_kwh <= 0.0 {
                continue;
            }
            let burn = (kwh * per_kwh).min(o.stores[g].max(0.0));
            o.stores[g] -= burn;
            // Its ash stays in the owner's store (ADR-0038 §5), counted made below.
            make::leave_ash(goods, &mut o.stores, g, burn);
            done += burn / per_kwh;
            kwh -= burn / per_kwh;
        }
        if done <= 0.0 {
            return;
        }
        let hours = done / e.rated_kw.max(1e-9);
        make::wear(&mut o.stores, goods, &[engine], hours);
        o.flows
            .add_changes(&before, &o.stores, Flow::Made, Flow::Used);
        let left = state.water_m3 * (1.0 - (done / need_kwh.max(1e-12)).min(1.0));
        ctx.land.set_pit_water(e.pit, left, now);
    }
}

/// The ways of making an engine: each recipe whose first output is a tool powered by any fuel by
/// its heat (one that burns a named fuel is not set at a pit, M11b), with that tool, by index.
fn engine_ways(catalog: &crate::params::Catalog) -> Vec<(usize, usize)> {
    catalog
        .recipes
        .iter()
        .enumerate()
        .filter_map(|(r, d)| {
            let &(g, _) = d.outputs.first()?;
            let p = catalog.goods.get(g)?.power?;
            let fuel = p.source == PowerSource::Fuel && p.fuel.is_none();
            fuel.then_some((r, g))
        })
        .collect()
}
