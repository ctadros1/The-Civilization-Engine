//! Road transit (M11b, third part; ADR-0036 §7): a household that keeps a motor cart may post its
//! runs from its door to another settlement's hearth and back, by a timetable, taking riders for a
//! fare, as a line's trains are posted (ADR-0034 §4–§5). On its day of the year it weighs the
//! market its settlement's people ran the most errands to in a year, at least a rider a month, and
//! posts a run a day there along its cart's way, the fare its fuel for a run at its own cost shared
//! over the riders it expects; a year on, it sets its runs a day by the rides taken, up to what a
//! run seats, and its fare by them, and stops where nobody rode. Someone on an errand to that
//! market may ride out and back where the ride's time and fare cost them less than the walk; they
//! ride the timetable as a train's riders do, paying the fare in food, and the cart burns a run's
//! fuel when the first rider of a departure boards.

use civ_core::time::DAYS_PER_YEAR;
use civ_core::{PermanentId, Rng64};
use civ_land::trains::RoadLine;

use super::trains::{MOST_RUNS_A_DAY, Ride, TIMETABLE_SPAN_MIN, departures};
use super::*;
use crate::ledger::Channel;

/// The fewest errands a year from the keeper's settlement to a market for which it posts runs
/// there (a design prior: a rider a month).
const MIN_ERRANDS_YEAR: u32 = 12;

/// The minutes a road line's cart stands at the far hearth before it sets off back.
const TURN_MIN: i64 = 15;

/// A rider's mass with what they carry, kilograms, for the seats a run has (a design prior).
const RIDER_KG: f64 = 100.0;

/// Keyed-randomness purpose of a cart keeper's day of the year for weighing its runs.
const PURPOSE_TRANSIT_REVIEW: u64 = 0x7472_616e_7369_7472; // "transitr"

/// A road line's ride for an errand (M11b, third part): the station to walk to (the cell its runs
/// leave from), the end cell at the far hearth and where it is, and the station's cell to ride
/// back to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RoadRide {
    pub station: (f32, f32),
    pub out_to: u32,
    pub end_at: (f32, f32),
    pub back_to: u32,
}

/// The minutes of the day road line `l`'s runs leave each end: out from its keeper's door by its
/// timetable, back from the hearth a run's minutes and a turn later.
pub fn road_departures(l: &RoadLine, back: bool) -> Vec<i64> {
    let out = departures(l.runs_a_day.clamp(1, MOST_RUNS_A_DAY));
    if !back {
        return out;
    }
    let later = f64::from(l.run_min()).ceil() as i64 + TURN_MIN;
    out.into_iter().map(|m| m + later).collect()
}

impl Population {
    /// Household `x`'s made motor cart, and its vehicle: a cart whose vehicle has an engine.
    fn motor_cart(ctx: &Ctx, x: PermanentId) -> Option<(PermanentId, usize)> {
        ctx.land.carts.made_of(x).find_map(|c| {
            let v = usize::from(c.vehicle);
            ctx.catalog
                .vehicles
                .get(v)
                .filter(|d| d.engine.is_some() && d.mode == crate::params::VehicleMode::Cart)
                .map(|_| (c.id, v))
        })
    }

    /// Each midnight, households keeping a motor cart whose day of the year it is weigh their
    /// runs (M11b, third part). Nothing where nobody keeps one.
    pub(super) fn transit_reviews(&mut self, ctx: &mut Ctx, day: i64) {
        if ctx.land.carts.list.is_empty() {
            return;
        }
        let due: Vec<PermanentId> = self
            .households
            .iter()
            .map(|(_, x)| x.id)
            .filter(|&id| {
                Rng64::from_key(&[ctx.seed, PURPOSE_TRANSIT_REVIEW, id.get()]).next_u64()
                    % DAYS_PER_YEAR as u64
                    == day.rem_euclid(DAYS_PER_YEAR) as u64
            })
            .filter(|&id| {
                Self::motor_cart(ctx, id).is_some()
                    || ctx.land.runs.roads.iter().any(|l| l.operator == id)
            })
            .collect();
        for operator in due {
            self.review_road_line(ctx, operator, day);
        }
    }

    /// The kcal of food household `operator` puts on the fuel of a run of `minutes` by its cart's
    /// vehicle `vehicle`, at its own cost of the fuel and of food; `None` where it puts no figure
    /// on either.
    fn run_cost_kcal(
        &self,
        ctx: &Ctx,
        operator: PermanentId,
        vehicle: usize,
        minutes: f64,
    ) -> Option<f64> {
        let goods = &ctx.catalog.goods;
        let x = self.household(operator)?;
        let (fuel, kg) = ctx
            .catalog
            .vehicles
            .get(vehicle)?
            .trip_fuel(goods, minutes)?;
        let costs = self.own_costs_of(ctx, x);
        let per_kcal = super::wires::hours_per_kcal(goods, &costs)?;
        let fuel_h = super::gas::fuel_hours(goods, &costs, fuel)?;
        Some(kg * fuel_h / per_kcal.max(1e-12))
    }

    /// Household `operator` weighs its runs (M11b, third part): with a line posted, it sets its
    /// runs a day by the rides taken since it was last weighed, as many as a run seats each, and
    /// its fare by them, and stops where nobody rode in a year or its cart is gone; without one, it
    /// posts a run a day to the market of the settlement its own settlement's people ran the most
    /// errands to in a year, at least [`MIN_ERRANDS_YEAR`], along its cart's way from its door to
    /// that hearth, the fare a run's fuel at its own cost over the riders it expects a run.
    fn review_road_line(&mut self, ctx: &mut Ctx, operator: PermanentId, day: i64) {
        let cart = Self::motor_cart(ctx, operator);
        if let Some(i) = ctx
            .land
            .runs
            .roads
            .iter()
            .position(|l| l.operator == operator)
        {
            let l = &ctx.land.runs.roads[i];
            let days = (day - l.since).max(1);
            let gone = cart.is_none_or(|(c, _)| c != l.cart);
            if gone || (l.riders == 0 && days >= DAYS_PER_YEAR) {
                ctx.land.runs.roads.remove(i);
                ctx.land.runs.changed();
                return;
            }
            let Some((_, vehicle)) = cart else {
                return;
            };
            let seats = ctx
                .catalog
                .vehicles
                .get(vehicle)
                .map_or(1.0, |v| (v.payload_kg / RIDER_KG).floor().max(1.0));
            // Each errand rides out and back: a direction's riders a day.
            let a_day = f64::from(l.riders) / (2.0 * days as f64);
            let runs = (a_day / seats)
                .ceil()
                .clamp(1.0, f64::from(MOST_RUNS_A_DAY)) as u8;
            let run_min = f64::from(l.run_min());
            let per_run = (a_day / f64::from(runs)).max(1.0);
            let fare = self
                .run_cost_kcal(ctx, operator, vehicle, run_min)
                .map(|k| (k / per_run) as f32);
            let l = &mut ctx.land.runs.roads[i];
            l.runs_a_day = runs;
            if let Some(f) = fare {
                l.fare_kcal = f;
            }
            l.riders = 0;
            l.since = day;
            ctx.land.runs.changed();
            return;
        }
        let Some((cart_id, vehicle)) = cart else {
            return;
        };
        let Some(x) = self.household(operator) else {
            return;
        };
        let (Some(home), door) = (x.settlement, x.home) else {
            return;
        };
        let Some(v) = ctx.catalog.vehicles.get(vehicle) else {
            return;
        };
        let walk = ctx.params.nav.tobler_ms(0.0);
        let mut best: Option<(u32, PermanentId, civ_land::carting::CartRoute)> = None;
        for s in &ctx.land.settlements {
            if s.id == home {
                continue;
            }
            let errands = self.errands_between(ctx, home, s.id);
            if errands < MIN_ERRANDS_YEAR || best.as_ref().is_some_and(|b| b.0 >= errands) {
                continue;
            }
            if let Some((route, _)) =
                ctx.land
                    .cart_way(ctx.map, &v.wheels(RIDER_KG, walk), door, s.hearth_m)
                && route.cells.len() >= 2
            {
                best = Some((errands, s.id, route));
            }
        }
        let Some((errands, market, route)) = best else {
            return;
        };
        let minutes: Vec<f32> = route.at_s.iter().map(|s| (s / 60.0) as f32).collect();
        let run_min = f64::from(minutes.last().copied().unwrap_or(0.0));
        let Some(cost) = self.run_cost_kcal(ctx, operator, vehicle, run_min) else {
            return;
        };
        // The riders it expects a run: each errand rides out once.
        let per_run = (f64::from(errands) / DAYS_PER_YEAR as f64).max(1.0);
        let id = ctx.ids.allocate();
        ctx.land.runs.roads.push(RoadLine {
            id,
            operator,
            cart: cart_id,
            market,
            cells: route.cells,
            minutes,
            runs_a_day: 1,
            fare_kcal: (cost / per_run) as f32,
            riders: 0,
            since: day,
            last_run: -1,
        });
        ctx.land.runs.changed();
    }

    /// Household `operator` weighs its runs now, whatever its day of the year: for tests.
    #[doc(hidden)]
    pub fn review_road_line_for_tests(&mut self, ctx: &mut Ctx, operator: PermanentId) {
        let day = ctx.now.day_index();
        self.review_road_line(ctx, operator, day);
    }

    /// A ride on a road line from cell `from` to cell `to` (M11b, third part), its ends, either
    /// way, with its cart still made and kept by its keeper, and a departure still to come today
    /// at `minute` of the day or later: as a train's [`Population::ride_from`] gives one, the fare
    /// in kcal of food.
    pub(super) fn road_ride(ctx: &Ctx, from: u32, to: u32, minute: i64) -> Option<Ride> {
        for l in &ctx.land.runs.roads {
            let (Some(&first), Some(&last)) = (l.cells.first(), l.cells.last()) else {
                continue;
            };
            let back = if (first, last) == (from, to) {
                false
            } else if (last, first) == (from, to) {
                true
            } else {
                continue;
            };
            let Some(c) = ctx
                .land
                .carts
                .list
                .iter()
                .find(|c| c.id == l.cart && c.made() && c.owner == l.operator)
            else {
                continue;
            };
            let Some(engine) = ctx
                .catalog
                .vehicles
                .get(usize::from(c.vehicle))
                .and_then(|v| v.engine)
            else {
                continue;
            };
            let depart = road_departures(l, back)
                .into_iter()
                .find(|&m| m >= minute && m < MINUTES_PER_DAY)?;
            let run = f64::from(l.run_min());
            let (cells, minutes): (Vec<u32>, Vec<f64>) = if back {
                l.cells
                    .iter()
                    .rev()
                    .zip(l.minutes.iter().rev())
                    .map(|(&c, &m)| (c, run - f64::from(m)))
                    .unzip()
            } else {
                l.cells
                    .iter()
                    .zip(&l.minutes)
                    .map(|(&c, &m)| (c, f64::from(m)))
                    .unzip()
            };
            return Some(Ride {
                way: l.id,
                operator: l.operator,
                wait_min: depart - minute,
                cells,
                minutes,
                fuel: engine.fuel,
                fare_kg: f64::from(l.fare_kcal),
                road: Some(depart),
            });
        }
        None
    }

    /// Takes a seat for household `household`'s member on road ride `ride` leaving at `depart`
    /// minutes into the day (M11b, third part): the first rider of a departure has the cart burn
    /// the run's fuel (none goes where its keeper has too little), and each pays the fare in food;
    /// the ride is counted on the line. `None` where the run cannot go or the fare cannot be paid.
    pub(super) fn board_road(
        &mut self,
        ctx: &mut Ctx,
        household: PermanentId,
        ride: Ride,
        depart: i64,
    ) -> Option<Ride> {
        let goods = &ctx.catalog.goods;
        let day_start = ctx.now.minutes() - ctx.now.minutes().rem_euclid(MINUTES_PER_DAY);
        let at = day_start + depart;
        let i = ctx.land.runs.roads.iter().position(|l| l.id == ride.way)?;
        let l = &ctx.land.runs.roads[i];
        let vehicle = ctx
            .land
            .carts
            .list
            .iter()
            .find(|c| c.id == l.cart)
            .map(|c| usize::from(c.vehicle))?;
        let burn = (l.last_run != at)
            .then(|| {
                ctx.catalog
                    .vehicles
                    .get(vehicle)?
                    .trip_fuel(goods, f64::from(l.run_min()))
            })
            .flatten();
        if let Some((fuel, kg)) = burn
            && self
                .household(ride.operator)
                .and_then(|x| x.stores.get(fuel).copied())
                .unwrap_or(0.0)
                + 1e-9
                < kg
        {
            return None;
        }
        if household != ride.operator && ride.fare_kg > 0.0 {
            let x = self.household(household)?;
            let stores = stores_now(x, ctx.now, ctx.params, goods);
            let food: f64 = goods
                .iter()
                .enumerate()
                .filter(|(_, d)| {
                    d.purpose == GoodUse::Food && d.kcal_per_kg > 0.0 && !d.kept_back()
                })
                .map(|(g, d)| stores.get(g).copied().unwrap_or(0.0).max(0.0) * d.kcal_per_kg)
                .sum();
            if food + 1e-6 < ride.fare_kg {
                return None;
            }
            self.pay_food_by(ctx, household, ride.operator, ride.fare_kg, Channel::Fare);
        }
        if let Some((fuel, kg)) = burn
            && let Some(x) = self.household_mut(ride.operator)
        {
            let held = x.stores.get(fuel).copied().unwrap_or(0.0);
            x.stores[fuel] = (held - kg).max(0.0);
            x.flows.add(Flow::Used, fuel, kg);
        }
        let l = &mut ctx.land.runs.roads[i];
        l.last_run = at;
        l.riders += 1;
        ctx.land.runs.changed();
        Some(ride)
    }

    /// The road line household `hh` could ride on an errand to the market of settlement `market`
    /// at a seller's door at `at` (M11b, third part): one whose keeper is of its own settlement,
    /// its cart still made; the line, the ride's legs, the hours there and back (walks to its
    /// station and from its far end to the door at `pace`, half the time between its runs
    /// waiting, and its minutes, each way) and its fare there and back, kcal of food.
    pub(super) fn road_legs(
        &self,
        ctx: &Ctx,
        hh: &Household,
        market: PermanentId,
        at: (f32, f32),
        pace: f64,
    ) -> Option<(PermanentId, RoadRide, f64, f64)> {
        let home = hh.settlement?;
        ctx.land.runs.roads.iter().find_map(|l| {
            if l.market != market
                || self.household(l.operator).and_then(|x| x.settlement) != Some(home)
                || !ctx
                    .land
                    .carts
                    .list
                    .iter()
                    .any(|c| c.id == l.cart && c.made() && c.owner == l.operator)
            {
                return None;
            }
            let (&first, &last) = (l.cells.first()?, l.cells.last()?);
            let station = cell_centre(ctx.map, first as usize);
            let end_at = cell_centre(ctx.map, last as usize);
            let walk_h = |a: (f32, f32), b: (f32, f32)| {
                f64::from((a.0 - b.0).hypot(a.1 - b.1)) / pace.max(1e-6) / 3600.0
            };
            let wait_h = TIMETABLE_SPAN_MIN as f64
                / (2.0 * f64::from(l.runs_a_day.clamp(1, MOST_RUNS_A_DAY)))
                / 60.0;
            let one_way = walk_h(hh.home, station)
                + wait_h
                + f64::from(l.run_min()) / 60.0
                + walk_h(end_at, at);
            Some((
                l.id,
                RoadRide {
                    station,
                    out_to: last,
                    end_at,
                    back_to: first,
                },
                2.0 * one_way,
                2.0 * f64::from(l.fare_kcal),
            ))
        })
    }
}
