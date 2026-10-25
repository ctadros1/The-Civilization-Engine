//! Water mills a household sets by its own choice (M10a slice DR, third part; ADR-0033 §5-6). A
//! household one of whose people knows a way of making by falling water weighs, on its day of the
//! year, what that way would save it: the goods it made in the last year that the way makes,
//! each kilogram at what it costs the household now less what it would cost by the wheel. A
//! technique's answer to a need is read from its recipes, not authored. It seeks a site along a
//! reach near home where a headrace dug at a ditch's least fall from an intake to a wheel further
//! down leaves the wheel a fall of its own, and sets one where the hours saved over the years
//! exceed the digging and the wheel. The headrace is dug as a ditch is, by the same work; the
//! installation turns once it is dug, and only then does the household want a wheel for it.
//! Limitation: the mill is minded from home, so only a site within a ditch's reach of home is
//! weighed.

use super::*;
use crate::decide::MillFacts;
use crate::ledger::{Channel, Leg};
use crate::params::{GoodPower, PowerSource};
use crate::person::MillSeen;
use civ_land::RectCm;
use civ_land::ditches::DitchSegment;
use civ_land::earth::{EARTH_VERSION, EarthKind, Earthwork};
use civ_land::power::Installation;

/// Purpose tag for the day of the year a household weighs a mill.
const PURPOSE_MILL_REVIEW: u64 = 0x6d69_6c6c_7276_7631; // "millrvv1"
/// The least fall a wheel is set at, metres (a design prior: below half a metre an undershot
/// wheel in a small race turns too slowly to drive stones).
const MIN_HEAD_M: f64 = 0.5;
/// Hours a day a mill grinds while its people are about it (a design prior).
const MILL_HOURS_A_DAY: f64 = 8.0;
/// The least distance between two wheels, metres (a design prior: one fall drives one mill;
/// a wheel below another takes the water the first has spent its fall on).
const MILL_SPACING_M: f64 = 50.0;
/// The share of what a neighbour's mill grinds that its owner keeps (M10a slice DR, fourth
/// part): a tuning value (research 08-03 names "customer-owned inputs processed for a fee or
/// toll" but gives no figure; a sixteenth is a design prior).
pub(crate) const TOLL_SHARE: f64 = 1.0 / 16.0;
/// Hours a year keeping a mill asks: dressing its stones and clearing its race (a design prior,
/// logged as a NUDGE in PROJECT_PLAN: reckoned in the choice, not yet worked).
const MILL_UPKEEP_H: f64 = 100.0;
/// The fewest days between two counts from which a year's making is read.
const MIN_SEEN_DAYS: i64 = 180;

/// A way of making by falling water: its recipe, the wheel it turns on, and that wheel's power.
#[derive(Clone, Copy)]
struct PoweredWay {
    recipe: usize,
    wheel: usize,
    power: GoodPower,
}

/// A mill a household would set: its intake and wheel on a reach, the headrace between them, and
/// what it would bring the household a year and over its years, hours of its work.
struct MillPlan {
    reach: u32,
    intake: [f64; 2],
    wheel: [f64; 2],
    drop_m: f64,
    head_m: f64,
    race_m3s: f64,
    dig_h: f64,
    gain_h: f64,
    net_h: f64,
}

impl Population {
    /// The households whose day of the year it is weigh a mill (M10a slice DR), where the content
    /// has a way of making by falling water at all and one of their people knows it.
    pub(super) fn mills_day(&mut self, ctx: &mut Ctx, day: i64) {
        let ways = powered_ways(ctx.catalog);
        if ways.is_empty() {
            return;
        }
        let due: Vec<PermanentId> = self
            .households
            .iter()
            .filter(|(_, x)| !x.members.is_empty())
            .filter(|(_, x)| {
                ways.iter().any(|w| {
                    ctx.catalog.recipes[w.recipe]
                        .technique
                        .is_some_and(|t| self.household_knows(&x.members, t))
                })
            })
            .map(|(_, x)| x.id)
            .filter(|h| {
                Rng64::from_key(&[ctx.seed, PURPOSE_MILL_REVIEW, h.get()]).next_u64()
                    % DAYS_PER_YEAR as u64
                    == day.rem_euclid(DAYS_PER_YEAR) as u64
            })
            .collect();
        for h in due {
            self.review_mill(ctx, h, &ways);
        }
    }

    /// Household `household` weighs a mill now, whatever its day of the year: for tests.
    #[doc(hidden)]
    pub fn review_mill_for_tests(&mut self, ctx: &mut Ctx, household: PermanentId) {
        let ways = powered_ways(ctx.catalog);
        self.review_mill(ctx, household, &ways);
    }

    /// Household `household` counts what it has made that its known water ways make, reads a
    /// year's making from its last count, and sets the mill whose hours saved over its years
    /// most exceed its digging and its wheel, if any does: one to a household, and none while it
    /// digs a ditch or a headrace.
    fn review_mill(&mut self, ctx: &mut Ctx, household: PermanentId, ways: &[PoweredWay]) {
        let Some(x) = self.household(household) else {
            return;
        };
        let known: Vec<PoweredWay> = ways
            .iter()
            .copied()
            .filter(|w| {
                ctx.catalog.recipes[w.recipe]
                    .technique
                    .is_none_or(|t| self.household_knows(&x.members, t))
            })
            .collect();
        if x.members.is_empty() || known.is_empty() {
            return;
        }
        let mut outputs: Vec<usize> = known
            .iter()
            .filter_map(|w| ctx.catalog.recipes[w.recipe].outputs.first().map(|o| o.0))
            .collect();
        outputs.sort_unstable();
        outputs.dedup();
        let made: f64 = outputs.iter().map(|&g| x.flows.get(Flow::Made, g)).sum();
        let day = ctx.now.day_index();
        let seen = x.mill_seen;
        if let Some(x) = self.household_mut(household) {
            x.mill_seen = Some(MillSeen { day, kg: made });
        }
        let Some(seen) = seen.filter(|s| day - s.day >= MIN_SEEN_DAYS) else {
            return;
        };
        let x = self.household(household).expect("the household is there");
        let owns = ctx.land.installations.iter().any(|i| i.owner == household);
        let digging = ctx
            .land
            .earthworks
            .iter()
            .any(|e| e.household == household && e.kind == EarthKind::Ditch && e.done < 1.0);
        if owns || digging {
            return;
        }
        let kg_year = (made - seen.kg).max(0.0) * DAYS_PER_YEAR as f64 / (day - seen.day) as f64;
        if kg_year <= 0.0 {
            return;
        }
        // Each known way's saving a kilogram: what the good costs the household now (it has no
        // wheel, so by its other ways) less what the wheel's way would cost it.
        let costs = self.own_costs_of(ctx, x);
        let Some((way, saved)) = known
            .iter()
            .filter_map(|w| {
                let r = &ctx.catalog.recipes[w.recipe];
                let &(out, per) = r.outputs.first()?;
                let now = costs.get(out).copied().flatten()?;
                let mut by_wheel = r.unit_h;
                for &(g, amount) in &r.inputs {
                    by_wheel += costs.get(g).copied().flatten()? * amount;
                }
                Some((*w, now - by_wheel / per.max(1e-9)))
            })
            .filter(|(_, s)| *s > 0.0)
            .max_by(|a, b| a.1.total_cmp(&b.1))
        else {
            return;
        };
        // A mill of its own spares only what grinding at a neighbour's for its toll would not
        // (M10a slice DR, fourth part): the toll's flour and a round trip for each load.
        let key = x.settlement.unwrap_or(x.id);
        let saved = match self.nearest_mill(ctx, x, key, ways) {
            None => saved,
            Some((_, w, walk_min)) => {
                let r = &ctx.catalog.recipes[w.recipe];
                let by_toll = r.inputs.first().zip(r.outputs.first()).and_then(
                    |(&(_, per_in), &(out, per))| {
                        let now = costs.get(out).copied().flatten()?;
                        let mut by_mill = r.unit_h;
                        for &(g, amount) in &r.inputs {
                            by_mill += costs.get(g).copied().flatten()? * amount;
                        }
                        let loads = per_in / ctx.params.household.carry_kg.max(1e-9);
                        let walk_h = 2.0 * walk_min / 60.0 * loads;
                        let kept = per * (1.0 - TOLL_SHARE);
                        Some(now - (by_mill + walk_h) / kept.max(1e-9))
                    },
                );
                saved - by_toll.unwrap_or(0.0).max(0.0)
            }
        };
        if saved <= 0.0 {
            return;
        }
        let Some(wheel_h) = costs.get(way.wheel).copied().flatten() else {
            return;
        };
        let home = [f64::from(x.home.0), f64::from(x.home.1)];
        let Some(p) = mill_site(ctx, household, home, &way, kg_year, saved, wheel_h) else {
            return;
        };
        let dp = &ctx.params.ditches;
        let work = ctx.ids.allocate();
        // The headrace is a ditch from the intake to the wheel that waters no field: its field is
        // its own work, so it never takes water as a ditch does (the wheel's water returns to the
        // river at its tail, `civ_land::power`).
        ctx.land
            .ditches
            .push(race(work, p.reach, p.intake, p.wheel, dp, p.drop_m));
        let rect = RectCm {
            x: ((p.intake[0] - 1.0) * 100.0).max(0.0) as i32,
            y: ((p.intake[1] - 1.0) * 100.0).max(0.0) as i32,
            w: 200,
            h: 200,
        };
        ctx.land.earthworks.push(Earthwork {
            id: work,
            kind: EarthKind::Ditch,
            rect,
            level_cm: 0,
            side_run_cm: (dp.side_run * 100.0).round() as u16,
            plot: None,
            deposit: None,
            heap: None,
            household,
            cut_m3: 0.0,
            done: 0.0,
            version: EARTH_VERSION,
            begun: ctx.now,
        });
        ctx.land.installations.push(Installation {
            id: work,
            reach: p.reach,
            at: p.wheel,
            head_m: p.head_m,
            race_m3s: p.race_m3s,
            efficiency: way.power.efficiency,
            rated_kw: way.power.rated_kw,
            owner: household,
            building: None,
        });
        let worth = (p.gain_h / p.dig_h.max(1e-6)) as f32;
        if let Some(x) = self.household_mut(household) {
            x.ditch_worth = worth;
        }
    }

    /// The working mill of another household of household `x`'s settlement nearest it by its
    /// people's walk, if any (M10a slice DR, fourth part): its installation's index, the way it
    /// grinds by, and the walk there, minutes. A mill works whose headrace is dug, whose owner
    /// holds a wheel for one of the ways and whose river turns it today.
    fn nearest_mill(
        &self,
        ctx: &Ctx,
        x: &Household,
        field_key: PermanentId,
        ways: &[PoweredWay],
    ) -> Option<(usize, PoweredWay, f64)> {
        let reach = &self.homes.get(&field_key)?.reach;
        let mut best: Option<(usize, PoweredWay, f64)> = None;
        for (i, m) in ctx.land.installations.iter().enumerate() {
            if m.owner == x.id || !ctx.land.installation_ready(i) {
                continue;
            }
            let Some(owner) = self.household(m.owner) else {
                continue;
            };
            if owner.members.is_empty() || owner.settlement != x.settlement {
                continue;
            }
            let Some(way) = ways
                .iter()
                .copied()
                .find(|w| owner.stores.get(w.wheel).copied().unwrap_or(0.0) >= decide::MIN_TOOL)
            else {
                continue;
            };
            if ctx.land.installation_kw(ctx.land_params, i) <= 0.0 {
                continue;
            }
            let at = (m.at[0] as f32, m.at[1] as f32);
            let Some(secs) = reach.seconds_to(cell_of(ctx.map, at)) else {
                continue;
            };
            let walk = f64::from(secs) / 60.0;
            if best.as_ref().is_none_or(|b| walk < b.2) {
                best = Some((i, way, walk));
            }
        }
        best
    }

    /// The neighbour's mill household `x`'s people could grind its grain at, as their decision
    /// sees it (M10a slice DR, fourth part): none in the dark, for a household with a working
    /// mill of its own, or one with no grain to grind or no want of flour; otherwise the nearest
    /// working mill of its settlement, the grain one of them would carry there (no more than a
    /// load, the grain held, or what brings its flour up to its days' store), the hours that
    /// takes at the mill's power, and the hours of its own grinding each of them spares, the
    /// mill's toll counted at what flour costs the household.
    pub(super) fn mill_facts(
        &self,
        ctx: &Ctx,
        x: &Household,
        field_key: PermanentId,
        dark: bool,
    ) -> Option<MillFacts> {
        if dark || ctx.land.installations.is_empty() {
            return None;
        }
        let owns = ctx
            .land
            .installations
            .iter()
            .enumerate()
            .any(|(i, m)| m.owner == x.id && ctx.land.installation_ready(i));
        if owns {
            return None;
        }
        let ways = powered_ways(ctx.catalog);
        let (i, way, walk_min) = self.nearest_mill(ctx, x, field_key, &ways)?;
        let r = &ctx.catalog.recipes[way.recipe];
        let (&(grain, per_in), &(flour, per)) = (r.inputs.first()?, r.outputs.first()?);
        let goods = &ctx.catalog.goods;
        let stores = stores_now(x, ctx.now, ctx.params, goods);
        let flour_kcal = goods.get(flour).map_or(0.0, |g| g.kcal_per_kg);
        if flour_kcal <= 0.0 || per_in <= 0.0 || per <= 0.0 {
            return None;
        }
        let day_kcal = x.members.len() as f64 * ctx.params.household.daily_kcal_per_person;
        let target = ctx.params.household.processed_food_days * day_kcal / flour_kcal;
        let short = target - stores.get(flour).copied().unwrap_or(0.0);
        let units = (ctx.params.household.carry_kg / per_in)
            .min(stores.get(grain).copied().unwrap_or(0.0) / per_in)
            .min(short / (per * (1.0 - TOLL_SHARE)));
        if units < 1.0 {
            return None;
        }
        let costs = self.own_costs_of(ctx, x);
        let now_h = costs.get(flour).copied().flatten()?;
        let mut by_mill = r.unit_h;
        for &(g, amount) in &r.inputs {
            by_mill += costs.get(g).copied().flatten()? * amount;
        }
        // A unit gives its flour less the toll: what each kilogram brought home spares.
        let kept = per * (1.0 - TOLL_SHARE);
        let saved = now_h * kept - by_mill;
        if saved <= 0.0 {
            return None;
        }
        let kw = ctx.land.installation_kw(ctx.land_params, i);
        let unit_hours = r.unit_h.max(r.kwh_per_unit / kw.max(1e-9));
        let m = &ctx.land.installations[i];
        Some(MillFacts {
            mill: m.id,
            at: (m.at[0] as f32, m.at[1] as f32),
            walk_min,
            left_h: units * unit_hours,
            saves_per_hour: saved / unit_hours,
        })
    }

    /// The mill person `who`'s household would grind at, as their decision sees it: for tests.
    #[doc(hidden)]
    pub fn mill_facts_for_tests(&self, ctx: &Ctx, who: PermanentId) -> Option<MillFacts> {
        let x = self.household(self.person(who)?.household)?;
        let key = x.settlement.unwrap_or(x.id);
        self.mill_facts(ctx, x, key, false)
    }

    /// `minutes` of person `who` grinding their household's grain at mill `mill`: for tests.
    #[doc(hidden)]
    pub fn mill_grind_for_tests(
        &mut self,
        ctx: &mut Ctx,
        who: PermanentId,
        mill: PermanentId,
        minutes: u32,
    ) {
        if let Some(&h) = self.index.get(&who) {
            self.mill_grind(ctx, h, mill, minutes);
        }
    }

    /// `minutes` of person `h` grinding their household's grain at mill `mill` (M10a slice DR,
    /// fourth part): as much as the session, the mill's power today and a load of the grain held
    /// allow, the flour theirs but the toll, which goes to the mill's owner; the owner's wheel
    /// wears by the hours it turned.
    pub(super) fn mill_grind(
        &mut self,
        ctx: &mut Ctx,
        h: Handle<Person>,
        mill: PermanentId,
        minutes: u32,
    ) {
        let Some(i) = ctx.land.installations.iter().position(|m| m.id == mill) else {
            return;
        };
        let owner = ctx.land.installations[i].owner;
        let Some(p) = self.people.get(h) else {
            return;
        };
        let household = p.household;
        if owner == household || !ctx.land.installation_ready(i) {
            return;
        }
        let ways = powered_ways(ctx.catalog);
        let Some(way) = self.household(owner).and_then(|o| {
            ways.iter()
                .copied()
                .find(|w| o.stores.get(w.wheel).copied().unwrap_or(0.0) >= decide::MIN_TOOL)
        }) else {
            return;
        };
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let r = &ctx.catalog.recipes[way.recipe];
        let (Some(&(grain, per_in)), Some(&(flour, per))) = (r.inputs.first(), r.outputs.first())
        else {
            return;
        };
        let hours = f64::from(minutes) / 60.0;
        let kw = ctx.land.installation_kw(ctx.land_params, i);
        let mut units = make::units_in(r, f64::from(minutes), 1.0)
            .min(kw * hours / r.kwh_per_unit.max(1e-9))
            .min(params.household.carry_kg / per_in.max(1e-9));
        let mut toll = 0.0;
        if let Some(x) = self
            .hh_index
            .get(&household)
            .and_then(|&hd| self.households.get_mut(hd))
        {
            let members = x.members.len().max(1);
            x.settle_stores(now, goods, &|d| fuel_per_day(params, members, d));
            x.stores.resize(goods.len(), 0.0);
            units = units.min(x.stores[grain] / per_in.max(1e-9));
            if units <= 0.0 {
                return;
            }
            let before = x.stores.clone();
            x.stores[grain] -= units * per_in;
            x.stores[flour] += units * per;
            toll = units * per * TOLL_SHARE;
            x.flows
                .add_changes(&before, &x.stores, Flow::Made, Flow::Used);
        }
        if units <= 0.0 {
            return;
        }
        // The toll is the owner's, by the ledger (a transfer the customer's store covers).
        let legs = [Leg {
            from: household,
            to: owner,
            good: flour,
            amount: toll,
        }];
        self.transfer(now, params, goods, &legs, Channel::Toll);
        // The wheel turned for the hours the grinding took.
        let turned = units * r.kwh_per_unit / kw.max(1e-9);
        if let Some(o) = self
            .hh_index
            .get(&owner)
            .and_then(|&hd| self.households.get_mut(hd))
        {
            let members = o.members.len().max(1);
            o.settle_stores(now, goods, &|d| fuel_per_day(params, members, d));
            o.stores.resize(goods.len(), 0.0);
            let before = o.stores.clone();
            make::wear(&mut o.stores, goods, &r.tools, turned.min(hours));
            o.flows
                .add_changes(&before, &o.stores, Flow::Made, Flow::Worn);
        }
    }
}

/// The best site for a mill of way `way` for household `holder` at `home` that makes
/// `kg_year` of its good a year and would save `saved` hours on each kilogram, its wheel
/// costing it `wheel_h` (M10a slice DR): an intake on any reach and a wheel further down it,
/// both within a ditch's longest of home, the wheel on the bank beside the river cell its
/// water falls back to and no nearer another wheel than `MILL_SPACING_M`, joined by a
/// headrace falling at a ditch's least gradient that leaves the wheel at least `MIN_HEAD_M`
/// of fall and crosses no field, plot or well. The wheel's power is reckoned at its reach's
/// mean flow; a mill that cannot grind all the household makes saves on what it can, less
/// `MILL_UPKEEP_H` a year. Of those, the one whose gain over a ditch's years most exceeds its
/// digging and its wheel.
fn mill_site(
    ctx: &Ctx,
    holder: PermanentId,
    home: [f64; 2],
    way: &PoweredWay,
    kg_year: f64,
    saved: f64,
    wheel_h: f64,
) -> Option<MillPlan> {
    let dp = &ctx.params.ditches;
    let lp = &ctx.land_params.ditches;
    let map = ctx.map;
    let r = &ctx.catalog.recipes[way.recipe];
    let per = r.outputs.first().map_or(0.0, |o| o.1);
    if r.kwh_per_unit <= 0.0 || per <= 0.0 {
        return None;
    }
    let cell = f64::from(map.cell_size_m);
    let centre = |c: u32| {
        [
            (f64::from(c % map.width) + 0.5) * cell,
            (f64::from(c / map.width) + 0.5) * cell,
        ]
    };
    let near = |p: [f64; 2]| (p[0] - home[0]).hypot(p[1] - home[1]) <= dp.max_length_m;
    let ground = |p: [f64; 2]| civ_land::height_at(map, p[0] as f32, p[1] as f32);
    let taken = |p: [f64; 2]| {
        ctx.land
            .installations
            .iter()
            .any(|x| (x.at[0] - p[0]).hypot(x.at[1] - p[1]) < MILL_SPACING_M)
    };
    let mut best: Option<MillPlan> = None;
    for (ri, reach) in map.reaches.iter().enumerate() {
        let flow = f64::from(reach.discharge_m3s);
        for (i, &a) in reach.cells.iter().enumerate() {
            let intake = centre(a);
            if !near(intake) {
                continue;
            }
            for &b in &reach.cells[i + 1..] {
                let tail = centre(b);
                if (intake[0] - tail[0]).hypot(intake[1] - tail[1]) > dp.max_length_m {
                    break;
                }
                // The wheel stands on the bank beside the river where its water falls back.
                let Some(wheel) = bank(map, b, home).map(centre) else {
                    continue;
                };
                let length = (intake[0] - wheel[0]).hypot(intake[1] - wheel[1]);
                if length <= 0.0 || length > dp.max_length_m || !near(wheel) || taken(wheel) {
                    continue;
                }
                let drop = lp.min_gradient * length;
                let head = ground(intake) - ground(tail) - drop;
                if head < MIN_HEAD_M {
                    continue;
                }
                let probe = race(holder, ri as u32, intake, wheel, dp, drop);
                let race_m3s = probe.capacity_m3s(lp);
                let kw = Installation {
                    id: holder,
                    reach: ri as u32,
                    at: wheel,
                    head_m: head,
                    race_m3s,
                    efficiency: way.power.efficiency,
                    rated_kw: way.power.rated_kw,
                    owner: holder,
                    building: None,
                }
                .power_kw(flow, lp.max_share);
                let can_kg = kw * MILL_HOURS_A_DAY * DAYS_PER_YEAR as f64 / r.kwh_per_unit * per;
                let gain_h = saved * kg_year.min(can_kg);
                let dig_h = dp.dig_hours(length);
                let net_h = (gain_h - MILL_UPKEEP_H) * dp.life_years - dig_h - wheel_h;
                if net_h <= 0.0 || best.as_ref().is_some_and(|p| p.net_h >= net_h) {
                    continue;
                }
                if !super::ditches::line_clear(ctx, holder, intake, wheel) {
                    continue;
                }
                best = Some(MillPlan {
                    reach: ri as u32,
                    intake,
                    wheel,
                    drop_m: drop,
                    head_m: head,
                    race_m3s,
                    dig_h,
                    gain_h,
                    net_h,
                });
            }
        }
    }
    best
}

/// The land cell beside river cell `c` nearest `home`, if any: where a wheel taking its fall
/// there stands.
fn bank(map: &civ_world::WorldMap, c: u32, home: [f64; 2]) -> Option<u32> {
    let (w, h) = (i64::from(map.width), i64::from(map.height));
    let (x, y) = (i64::from(c) % w, i64::from(c) / w);
    let cell = f64::from(map.cell_size_m);
    let mut best: Option<(f64, u32)> = None;
    for (dx, dy) in [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ] {
        let (nx, ny) = (x + dx, y + dy);
        if nx < 0 || ny < 0 || nx >= w || ny >= h {
            continue;
        }
        let n = (ny * w + nx) as u32;
        if map.water.get(n as usize) != Some(&civ_world::WATER_LAND) {
            continue;
        }
        let far = ((nx as f64 + 0.5) * cell - home[0]).hypot((ny as f64 + 0.5) * cell - home[1]);
        if best.is_none_or(|(d, _)| far < d) {
            best = Some((far, n));
        }
    }
    best.map(|(_, n)| n)
}

/// The ways of making by falling water in the catalog that a household works at its own mill:
/// recipes that ask power and name a fixed tool that turns on water, with that tool and its
/// power, leaving out those whose machine wants a crew, which only a works works (M10a slice DT).
fn powered_ways(catalog: &crate::params::Catalog) -> Vec<PoweredWay> {
    catalog
        .recipes
        .iter()
        .enumerate()
        .filter(|(_, r)| r.kwh_per_unit > 0.0 && super::crew_of(&catalog.goods, r) <= 1)
        .filter_map(|(i, r)| {
            r.tools.iter().find_map(|&t| {
                let power = catalog.goods.get(t)?.power?;
                (power.source == PowerSource::Water).then_some(PoweredWay {
                    recipe: i,
                    wheel: t,
                    power,
                })
            })
        })
        .collect()
}

/// A headrace from `intake` to `wheel` on reach `reach`, of a ditch's section, its bed falling
/// `drop_m`: a ditch whose field is its own work.
fn race(
    work: PermanentId,
    reach: u32,
    intake: [f64; 2],
    wheel: [f64; 2],
    dp: &crate::irrigation::DitchPlanParams,
    drop_m: f64,
) -> DitchSegment {
    DitchSegment {
        work,
        reach,
        field: work,
        a: intake,
        b: wheel,
        bottom_m: dp.bottom_m,
        depth_m: dp.depth_m,
        side_run: dp.side_run,
        drop_m,
        weeds: 0.0,
        silt_m: 0.0,
        open: false,
        taken_m3: 0.0,
        kept_m3: 0.0,
        seeped_m3: 0.0,
        returned_m3: 0.0,
    }
}
