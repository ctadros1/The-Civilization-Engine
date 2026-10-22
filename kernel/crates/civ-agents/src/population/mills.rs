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
}

/// The best site for a mill of way `way` for household `holder` at `home` that makes
/// `kg_year` of its good a year and would save `saved` hours on each kilogram, its wheel
/// costing it `wheel_h` (M10a slice DR): an intake on any reach and a wheel further down it,
/// both within a ditch's longest of home, the wheel on the bank beside the river cell its
/// water falls back to and no nearer another wheel than `MILL_SPACING_M`, joined by a
/// headrace falling at a ditch's least gradient that leaves the wheel at least `MIN_HEAD_M`
/// of fall and crosses no field, plot or well. The wheel's power is reckoned at its reach's
/// mean flow; a mill that cannot grind all the household makes saves on what it can. Of
/// those, the one whose gain over a ditch's years most exceeds its digging and its wheel.
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
                let net_h = gain_h * dp.life_years - dig_h - wheel_h;
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

/// The ways of making by falling water in the catalog: recipes that ask power and name a fixed
/// tool that turns on water, with that tool and its power.
fn powered_ways(catalog: &crate::params::Catalog) -> Vec<PoweredWay> {
    catalog
        .recipes
        .iter()
        .enumerate()
        .filter(|(_, r)| r.kwh_per_unit > 0.0)
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
