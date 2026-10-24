//! Trains on a household's line (M10b slice DW, third part; ADR-0034 §3–§4; the rail brief
//! §1.2–§1.4): an engine on rails is a rail vehicle with an engine, a record kept beside the
//! wagons, standing at an end of its household's line. Where the household's whole line runs from
//! beside a pit to beside home with its engine and wagons standing at the pit end, a session of
//! digging there loads what the work digs onto the wagons rather than being carried home, up to
//! what the engine's driven wheels hold up the steepest rise homeward. Each day the household
//! brings its trains to account: a run that has arrived leaves its engine and wagons at that end,
//! and at the home end the wagons' goods go into the store; an engine standing at the pit end
//! with loaded wagons sets off home in the morning, and one at the home end with empty wagons goes
//! back out light, each burning coal by the work it does, `Σ M·g·(C + s)·run` over the steps that
//! need pull, at its efficiency, taken from the store when it sets off. A run is the line's own
//! (`civ_land::trains`): it holds block sections between loops and waits for them.

use super::*;
use crate::consignments::Consignment;
use civ_land::carting::{DOOR_LEG_M, pull_n};
use civ_land::rail;
use civ_land::trains::{self, Run, RunState, Train};
use civ_land::ways::{Keeper, RAIL};

/// The minute of the day an engine sets off along its line.
const DEPART_MIN: i64 = 6 * 60;

/// The share of another household's goods a line's keeper keeps for carrying them home on its
/// train (M10b slice DX, first part): a design prior, as the mill's toll is (research 07-08 §1.6
/// counts a carrier's fees but gives no share for so short a haul).
pub(crate) const FREIGHT_SHARE: f64 = 1.0 / 10.0;

/// What an engine on its line would spare a household a year and cost, as it reckons it (M10b
/// slice DW, fourth part): the line's pit end, where it would stand; the hours a year it spares;
/// the hours a year its coal costs at the household's own cost; and what making it takes, hours
/// (its iron at the household's own cost).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EngineReckoning {
    pub pit_end: u32,
    pub saves_h_year: f64,
    pub coal_h_year: f64,
    pub cost_h: f64,
}

/// A household's engine on its whole line between a pit and home.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct EngineLine {
    pub way: PermanentId,
    pub cells: Vec<u32>,
    pub track: rail::Track,
    /// The line's cells beside the pit and beside home.
    pub pit_end: u32,
    pub home_end: u32,
    /// Whether homeward runs from the line's last cell to its first.
    pub home_back: bool,
    /// The engine, by record and by vehicle index.
    pub engine: PermanentId,
    pub vehicle: usize,
}

impl Population {
    /// Household `household`'s engine line from a pit at `pit` (any pit, where `None`) to home at
    /// `home`, if any: a whole line kept by it alone, serving wagons, one end within
    /// [`DOOR_LEG_M`] of home and the other of the pit, with a made engine of its gauge standing
    /// at either end and not out on a run. The first such line, in the order lines were begun.
    pub(super) fn engine_line(
        ctx: &Ctx,
        household: PermanentId,
        home: (f32, f32),
        pit: Option<(f32, f32)>,
    ) -> Option<EngineLine> {
        if ctx.land.wagons.list.is_empty() {
            return None;
        }
        Self::own_lines(ctx, household, home, pit)
            .into_iter()
            .find_map(|mut line| {
                let (first, last) = (line.cells[0], *line.cells.last()?);
                let (engine, vehicle) = ctx.land.wagons.made_of(household).find_map(|w| {
                    let v = usize::from(w.vehicle);
                    let def = ctx.catalog.vehicles.get(v)?;
                    (def.engine.is_some()
                        && (w.at == first || w.at == last)
                        && (def.gauge_m - f64::from(line.track.gauge_m)).abs() < 1e-3
                        && !ctx.land.runs.aboard(w.id))
                    .then_some((w.id, v))
                })?;
                line.engine = engine;
                line.vehicle = vehicle;
                Some(line)
            })
    }

    /// Household `household`'s whole lines from a pit at `pit` (any pit, where `None`) to home at
    /// `home`, in the order begun: kept by it alone, serving wagons, one end within
    /// [`DOOR_LEG_M`] of home and the other of the pit; each with no engine yet named.
    fn own_lines(
        ctx: &Ctx,
        household: PermanentId,
        home: (f32, f32),
        pit: Option<(f32, f32)>,
    ) -> Vec<EngineLine> {
        let mut out = Vec::new();
        let near = |cell: u32, at: (f32, f32)| {
            let c = cell_centre(ctx.map, cell as usize);
            f64::from((c.0 - at.0).hypot(c.1 - at.1)) <= DOOR_LEG_M
        };
        for way in &ctx.land.ways.list {
            let Some(track) = &way.track else {
                continue;
            };
            if way.modes & RAIL == 0
                || !way.segments.iter().all(|s| {
                    s.keeper == Keeper::Household(household) && s.built_m + 1e-3 >= s.corridor_m
                })
            {
                continue;
            }
            let Some((first, last)) = way.ends() else {
                continue;
            };
            let at_pit = |cell: u32| pit.is_none_or(|p| near(cell, p));
            // Homeward is last to first when the first end lies by home.
            let home_back = if near(first, home) && at_pit(last) {
                true
            } else if near(last, home) && at_pit(first) {
                false
            } else {
                continue;
            };
            let (home_end, pit_end) = if home_back {
                (first, last)
            } else {
                (last, first)
            };
            out.push(EngineLine {
                way: way.id,
                cells: way
                    .segments
                    .iter()
                    .flat_map(|s| s.cells.iter().copied())
                    .collect(),
                track: track.clone(),
                pit_end,
                home_end,
                home_back,
                engine: way.id,
                vehicle: usize::MAX,
            });
        }
        out
    }

    /// Household `household`'s made wagons of gauge `gauge_m` standing at cell `at` and not out
    /// on a run, oldest first.
    fn wagons_at(ctx: &Ctx, household: PermanentId, at: u32, gauge_m: f32) -> Vec<PermanentId> {
        ctx.land
            .wagons
            .made_of(household)
            .filter(|w| {
                w.at == at
                    && !ctx.land.runs.aboard(w.id)
                    && ctx
                        .catalog
                        .vehicles
                        .get(usize::from(w.vehicle))
                        .is_some_and(|d| {
                            d.is_wagon() && (d.gauge_m - f64::from(gauge_m)).abs() < 1e-3
                        })
            })
            .map(|w| w.id)
            .collect()
    }

    /// The most goods, kilograms, `line`'s engine pulls home with wagons `cars`: what its driven
    /// wheels hold up the steepest rise homeward, less its own and the wagons' mass, and no more
    /// than the wagons carry.
    fn train_room(ctx: &Ctx, line: &EngineLine, cars: &[PermanentId]) -> f64 {
        Self::room_of(ctx, line, line.vehicle, cars)
    }

    /// As [`Population::train_room`], for an engine of vehicle `vehicle`.
    fn room_of(ctx: &Ctx, line: &EngineLine, vehicle: usize, cars: &[PermanentId]) -> f64 {
        let Some(def) = ctx.catalog.vehicles.get(vehicle) else {
            return 0.0;
        };
        let runs = rail::runs(&line.cells, ctx.map.width, f64::from(ctx.map.cell_size_m));
        let rise = rail::steepest_rise(&line.track.bed, &runs, line.home_back);
        let (tare, payload) = cars
            .iter()
            .filter_map(|&c| ctx.land.wagons.list.iter().find(|w| w.id == c))
            .filter_map(|w| ctx.catalog.vehicles.get(usize::from(w.vehicle)))
            .fold((def.tare_kg, 0.0), |(t, p), d| {
                (t + d.tare_kg, p + d.payload_kg)
            });
        (Train::most_mass(def.pull_n, def.rail_resistance, rise) - tare).clamp(0.0, payload)
    }

    /// The goods aboard wagons `cars`, kilograms.
    fn aboard_kg(consignments: &[Consignment], cars: &[PermanentId]) -> f64 {
        consignments
            .iter()
            .filter(|c| cars.contains(&c.boat))
            .map(|c| c.stores.iter().sum::<f64>())
            .sum()
    }

    /// Where a session's digging by someone of household `household`, at home at `home`, at a
    /// pit at `pit` goes when its engine works the line from there: the first wagon at the pit
    /// end and the room left on the train, kilograms; `None` where no engine works such a line
    /// from the pit end, no wagon stands there, or the train is full.
    pub(super) fn engine_room(
        ctx: &Ctx,
        consignments: &[Consignment],
        household: PermanentId,
        home: (f32, f32),
        pit: (f32, f32),
    ) -> Option<(PermanentId, f64)> {
        let line = Self::engine_line(ctx, household, home, Some(pit))?;
        let engine_at = ctx
            .land
            .wagons
            .list
            .iter()
            .find(|w| w.id == line.engine)?
            .at;
        if engine_at != line.pit_end {
            return None;
        }
        let cars = Self::wagons_at(ctx, household, line.pit_end, line.track.gauge_m);
        let first = *cars.first()?;
        let room = Self::train_room(ctx, &line, &cars) - Self::aboard_kg(consignments, &cars);
        (room > 0.0).then_some((first, room))
    }

    /// Where a session's digging by someone of household `household`, at home at `home`, at a
    /// pit at `pit` goes when another household's engine works a line from there to beside its
    /// home (M10b slice DX, first part): that household, the line's keeper; the first of its
    /// wagons at the pit end; and the room left on its train. Keepers are tried in the order their
    /// lines were begun.
    pub(super) fn freight_room(
        ctx: &Ctx,
        consignments: &[Consignment],
        household: PermanentId,
        home: (f32, f32),
        pit: (f32, f32),
    ) -> Option<(PermanentId, PermanentId, f64)> {
        if ctx.land.wagons.list.is_empty() {
            return None;
        }
        let mut tried = Vec::new();
        for way in &ctx.land.ways.list {
            if way.modes & RAIL == 0 {
                continue;
            }
            let Some(Keeper::Household(keeper)) = way.segments.first().map(|s| s.keeper) else {
                continue;
            };
            if keeper == household || tried.contains(&keeper) {
                continue;
            }
            tried.push(keeper);
            if let Some((wagon, room)) = Self::engine_room(ctx, consignments, keeper, home, pit) {
                return Some((keeper, wagon, room));
            }
        }
        None
    }

    /// Loads `kg` of good `good`, dug by household `household`, onto wagon `wagon` (its own, or
    /// another household's carrying it for freight): into the goods in transit of its aboard,
    /// begun if there are none.
    pub(super) fn load_wagon(
        &mut self,
        ctx: &mut Ctx,
        household: PermanentId,
        wagon: PermanentId,
        good: usize,
        kg: f64,
    ) {
        if kg <= 0.0 {
            return;
        }
        let (now, goods) = (ctx.now, &ctx.catalog.goods);
        let i = match self
            .consignments
            .iter()
            .position(|c| c.boat == wagon && c.owner == household)
        {
            Some(i) => i,
            None => {
                let place = self
                    .household(household)
                    .and_then(|x| x.settlement)
                    .unwrap_or(household);
                // Its carrier is the wagon itself, so nobody unloads it but at the line's end.
                let c = Consignment::new(
                    ctx.ids.allocate(),
                    (household, wagon, wagon),
                    (place, place),
                    now,
                    goods.len(),
                );
                self.consignments.push(c);
                self.consignments.len() - 1
            }
        };
        let c = &mut self.consignments[i];
        c.settle_stores(now, goods);
        if let Some(s) = c.stores.get_mut(good) {
            *s += kg;
            c.flows.add(Flow::Got, good, kg);
        }
    }

    /// The fuel, by good index, and the kilograms of it engine `vehicle` burns pulling `mass_kg`
    /// along a bed (`runs` apart) that way: the work of every step that needs pull, at its
    /// efficiency, over its fuel's heat. `None` where it has no engine or its fuel gives no heat.
    fn fuel_kg(
        ctx: &Ctx,
        vehicle: usize,
        bed: &[f32],
        runs: &[f64],
        mass_kg: f64,
        back: bool,
    ) -> Option<(usize, f64)> {
        let def = ctx.catalog.vehicles.get(vehicle)?;
        let engine = def.engine?;
        let mj = ctx.catalog.goods.get(engine.fuel)?.fuel?.mj_per_kg;
        if mj <= 0.0 {
            return None;
        }
        let mut steps: Vec<f64> = runs.get(1..).unwrap_or(&[]).to_vec();
        if back {
            steps.reverse();
        }
        let work_j: f64 = rail::step_grades(bed, runs, back)
            .iter()
            .zip(&steps)
            .map(|(&g, &run)| pull_n(mass_kg, def.rail_resistance, g).max(0.0) * run)
            .sum();
        Some((
            engine.fuel,
            work_j / engine.efficiency.max(1e-9) / (mj * 1e6),
        ))
    }

    /// Each household's trains brought to account on day `day`: runs that have arrived leave
    /// their engine and wagons at that end and, at the home end, the wagons' goods in the store;
    /// then an engine at the pit end with loaded wagons sets off home, and one at the home end
    /// with empty wagons goes back out, in the morning, each with its coal.
    pub(super) fn engine_runs(&mut self, ctx: &mut Ctx, day: i64) {
        if ctx.land.wagons.list.is_empty() {
            return;
        }
        let mut owners: Vec<PermanentId> = ctx
            .land
            .wagons
            .list
            .iter()
            .filter(|w| {
                w.made()
                    && ctx
                        .catalog
                        .vehicles
                        .get(usize::from(w.vehicle))
                        .is_some_and(|d| d.engine.is_some())
            })
            .map(|w| w.owner)
            .collect();
        owners.sort_unstable();
        owners.dedup();
        for household in owners {
            self.settle_runs(ctx, household);
            self.set_off(ctx, household, day);
        }
    }

    /// Household `household`'s runs that have arrived, taken off their line: their engine and
    /// wagons left at the end they reached, and the wagons' goods unloaded into its store where
    /// that end is beside home.
    fn settle_runs(&mut self, ctx: &mut Ctx, household: PermanentId) {
        let arrived: Vec<PermanentId> = ctx
            .land
            .runs
            .list
            .iter()
            .filter(|r| r.owner == household && matches!(r.state, RunState::Arrived { .. }))
            .map(|r| r.id)
            .collect();
        let home = self.household(household).map(|x| x.home);
        for id in arrived {
            let Some(run) = ctx.land.runs.take_off(id) else {
                continue;
            };
            let Some((first, last)) = ctx
                .land
                .ways
                .list
                .iter()
                .find(|w| w.id == run.way)
                .and_then(|w| w.ends())
            else {
                continue;
            };
            let end = if run.back { first } else { last };
            for w in &mut ctx.land.wagons.list {
                if w.id == run.engine || run.cars.contains(&w.id) {
                    w.at = end;
                }
            }
            ctx.land.wagons.changed();
            let by_home = home.is_some_and(|h| {
                let c = cell_centre(ctx.map, end as usize);
                f64::from((c.0 - h.0).hypot(c.1 - h.1)) <= DOOR_LEG_M
            });
            if by_home {
                self.unload_wagons(ctx, household, &run.cars);
            }
        }
    }

    /// The goods aboard household `household`'s wagons `cars` go into its store.
    fn unload_wagons(&mut self, ctx: &Ctx, household: PermanentId, cars: &[PermanentId]) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let aboard: Vec<usize> = (0..self.consignments.len())
            .filter(|&i| cars.contains(&self.consignments[i].boat))
            .collect();
        for i in aboard.into_iter().rev() {
            let c = &mut self.consignments[i];
            c.settle_stores(now, goods);
            let (id, owner) = (c.id, c.owner);
            // Another household's goods pay the keeper its freight first (M10b slice DX, first
            // part), the rest going into their owner's store.
            let share = if owner == household {
                0.0
            } else {
                FREIGHT_SHARE
            };
            let held: Vec<(usize, f64)> = c
                .stores
                .iter()
                .enumerate()
                .filter(|&(_, &kg)| kg > 0.0)
                .map(|(good, &kg)| (good, kg))
                .collect();
            let freight: Vec<Leg> = held
                .iter()
                .filter(|_| share > 0.0)
                .map(|&(good, kg)| Leg {
                    from: id,
                    to: household,
                    good,
                    amount: kg * share,
                })
                .collect();
            if !freight.is_empty() {
                self.transfer(now, params, goods, &freight, Channel::Freight);
            }
            let legs: Vec<Leg> = held
                .iter()
                .map(|&(good, kg)| Leg {
                    from: id,
                    to: owner,
                    good,
                    amount: kg * (1.0 - share),
                })
                .collect();
            let moved =
                legs.is_empty() || self.transfer(now, params, goods, &legs, Channel::Carried);
            if moved && self.consignments.get(i).is_none_or(|c| c.is_empty()) {
                let c = self.consignments.remove(i);
                self.consigned_flows.absorb(&c.flows);
            }
        }
    }

    /// Household `household`'s engine sets off along its line on day `day`, if it should: home
    /// from the pit end with its loaded wagons, or back out light from the home end, when the
    /// household holds the coal the run burns and someone of it is of an age to drive it.
    fn set_off(&mut self, ctx: &mut Ctx, household: PermanentId, day: i64) {
        let Some(home) = self.household(household).map(|x| x.home) else {
            return;
        };
        let Some(line) = Self::engine_line(ctx, household, home, None) else {
            return;
        };
        if self.elder_of(household, ctx.now, ctx.params).is_none() {
            return;
        }
        let Some(engine_at) = ctx
            .land
            .wagons
            .list
            .iter()
            .find(|w| w.id == line.engine)
            .map(|w| w.at)
        else {
            return;
        };
        let cars = Self::wagons_at(ctx, household, engine_at, line.track.gauge_m);
        let load = Self::aboard_kg(&self.consignments, &cars);
        // Home with a load, or out light; never home empty nor out loaded.
        let homeward = engine_at == line.pit_end;
        if homeward != (load > 0.0) {
            return;
        }
        let Some(def) = ctx.catalog.vehicles.get(line.vehicle) else {
            return;
        };
        let Some(engine) = def.engine else {
            return;
        };
        let cars_tare: f64 = cars
            .iter()
            .filter_map(|&c| ctx.land.wagons.list.iter().find(|w| w.id == c))
            .filter_map(|w| ctx.catalog.vehicles.get(usize::from(w.vehicle)))
            .map(|d| d.tare_kg)
            .sum();
        let mass = def.tare_kg + cars_tare + load;
        let train = Train {
            power_w: def.power_w,
            pull_n: def.pull_n,
            top_m_s: engine.top_speed_m_s,
            mass_kg: mass,
            c: def.rail_resistance,
        };
        // Homeward runs as the line was found; outward the other way.
        let back = if homeward {
            line.home_back
        } else {
            !line.home_back
        };
        let runs = rail::runs(&line.cells, ctx.map.width, f64::from(ctx.map.cell_size_m));
        let Some(secs) = trains::step_seconds(&line.track.bed, &runs, &train, back) else {
            return;
        };
        let Some((fuel, kg)) = Self::fuel_kg(ctx, line.vehicle, &line.track.bed, &runs, mass, back)
        else {
            return;
        };
        let Some(x) = self.household_mut(household) else {
            return;
        };
        if x.stores.get(fuel).copied().unwrap_or(0.0) < kg {
            return;
        }
        x.stores[fuel] -= kg;
        x.flows.add(Flow::Used, fuel, kg);
        let sections = trains::sections(ctx.land.runs.loops_of(line.way), line.cells.len());
        let run = Run {
            id: ctx.ids.allocate(),
            way: line.way,
            back,
            engine: line.engine,
            owner: household,
            cars,
            legs: trains::legs(&secs, &sections, back),
            leg: 0,
            state: RunState::Arrived { at: 0 },
            load_kg: load as f32,
        };
        ctx.land.runs.depart(run, day * 1_440 + DEPART_MIN);
    }

    /// The engine the content makes, if any: the first rail vehicle with an engine.
    fn engine_vehicle(ctx: &Ctx) -> Option<usize> {
        ctx.catalog.vehicles.iter().position(|v| v.engine.is_some())
    }

    /// What an engine of vehicle `vehicle` on household `household`'s line from the pit it digs
    /// most would spare it, if it has such a line with wagons on it (M10b slice DW, fourth part):
    /// a dig day there brings home what one person pushes (or carries, with no wagon to push) in
    /// a session's work and walks, and with an engine what the session digs, up to what the
    /// train holds; so to bring as much a year, the train spares the share of each dig day's
    /// hours that the smaller load costs. Against that, the coal its runs burn (one home loaded
    /// and one out light each dig day) a year, at the household's own cost of coal or, where the
    /// pit yields it, the hours it takes to dig; and what the engine takes: its labour and its
    /// iron at the household's own cost, nothing more for iron it holds and cannot make.
    fn engine_reckoning(
        &self,
        ctx: &Ctx,
        household: PermanentId,
        vehicle: usize,
    ) -> Option<EngineReckoning> {
        let x = self.household(household)?;
        let def = ctx.catalog.vehicles.get(vehicle)?;
        let p = self.dig_profile(ctx, household)?;
        let line = Self::own_lines(ctx, household, x.home, Some(p.pit))
            .into_iter()
            .find(|l| (def.gauge_m - f64::from(l.track.gauge_m)).abs() < 1e-3)?;
        let (first, last) = (line.cells[0], *line.cells.last()?);
        let mut cars = Self::wagons_at(ctx, household, first, line.track.gauge_m);
        cars.extend(Self::wagons_at(ctx, household, last, line.track.gauge_m));
        if cars.is_empty() {
            return None;
        }
        let carry = ctx.params.household.carry_kg;
        let dug = p.kg_h * p.session_h;
        let by_now = Self::wagon_haul(ctx, household, x.home, p.pit).map_or(carry, |h| {
            h.load_kg
                .min(p.kg_h * (p.session_h - h.extra_min / 60.0).max(0.0))
        });
        let by_train = dug.min(Self::room_of(ctx, &line, vehicle, &cars));
        if by_train <= by_now || by_now <= 0.0 {
            return None;
        }
        let saves_h_year = p.days_year * (p.session_h + 2.0 * p.walk_h) * (1.0 - by_now / by_train);
        // The coal a dig day's two runs burn: home with the load, out light.
        let runs = rail::runs(&line.cells, ctx.map.width, f64::from(ctx.map.cell_size_m));
        let cars_tare: f64 = cars
            .iter()
            .filter_map(|&c| ctx.land.wagons.list.iter().find(|w| w.id == c))
            .filter_map(|w| ctx.catalog.vehicles.get(usize::from(w.vehicle)))
            .map(|d| d.tare_kg)
            .sum();
        let light = def.tare_kg + cars_tare;
        let (fuel, home_kg) = Self::fuel_kg(
            ctx,
            vehicle,
            &line.track.bed,
            &runs,
            light + by_train,
            line.home_back,
        )?;
        let (_, out_kg) =
            Self::fuel_kg(ctx, vehicle, &line.track.bed, &runs, light, !line.home_back)?;
        let costs = self.own_costs_of(ctx, x);
        // Its coal at the household's own cost, or what the pit yields where that is the fuel,
        // at the hours it takes to dig (an engine at a pit head burns its own fuel, 07-06 §2.4).
        let at_pit = (usize::from(p.deposit.body.good) == fuel).then(|| 1.0 / p.kg_h);
        let coal_h = [costs.get(fuel).copied().flatten(), at_pit]
            .into_iter()
            .flatten()
            .reduce(f64::min)?;
        // Its iron at the household's own cost; iron it holds and cannot make costs it nothing
        // more.
        let iron_h = match costs.get(def.good).copied().flatten() {
            Some(h) => h,
            None if x.stores.get(def.good).copied().unwrap_or(0.0) >= def.material_kg => 0.0,
            None => return None,
        };
        // As many trains a year as it takes to bring what it brings now.
        let trains_year = p.days_year * by_now / by_train;
        Some(EngineReckoning {
            pit_end: line.pit_end,
            saves_h_year,
            coal_h_year: trains_year * (home_kg + out_kg) * coal_h,
            cost_h: def.labour_h + def.material_kg * iron_h,
        })
    }

    /// Household `household`, with its line and wagons, weighs an engine (M10b slice DW, fourth
    /// part): where one of it knows how to make one, it has none, and the hours an engine would
    /// spare it beyond its coal over the engine's life come to more than the engine takes, it
    /// begins one at the line's pit end, its iron from the store.
    pub(super) fn review_locomotive(&mut self, ctx: &mut Ctx, household: PermanentId) {
        let Some(vehicle) = Self::engine_vehicle(ctx) else {
            return;
        };
        let Some(x) = self.household(household) else {
            return;
        };
        let Some(def) = ctx.catalog.vehicles.get(vehicle) else {
            return;
        };
        if x.members.is_empty()
            || def
                .technique
                .is_some_and(|t| !self.household_knows(&x.members, t))
            || ctx.land.wagons.of(household).any(|w| {
                ctx.catalog
                    .vehicles
                    .get(usize::from(w.vehicle))
                    .is_some_and(|d| d.engine.is_some())
            })
        {
            return;
        }
        let Some(r) = self.engine_reckoning(ctx, household, vehicle) else {
            return;
        };
        if (r.saves_h_year - r.coal_h_year) * def.life_years <= r.cost_h {
            return;
        }
        self.begin_wagon(ctx, household, vehicle, r.pit_end, r.saves_h_year);
    }

    /// What an engine would spare household `household`, as its review reckons it: for tests.
    #[doc(hidden)]
    pub fn engine_reckoning_for_tests(
        &self,
        ctx: &Ctx,
        household: PermanentId,
    ) -> Option<EngineReckoning> {
        self.engine_reckoning(ctx, household, Self::engine_vehicle(ctx)?)
    }

    /// Household `household` weighs an engine now, whatever its day of the year: for tests.
    #[doc(hidden)]
    pub fn review_locomotive_for_tests(&mut self, ctx: &mut Ctx, household: PermanentId) {
        self.review_locomotive(ctx, household);
    }

    /// Household `household`'s trains brought to account as on day `day`: for tests.
    #[doc(hidden)]
    pub fn engine_runs_for_tests(&mut self, ctx: &mut Ctx, household: PermanentId, day: i64) {
        self.settle_runs(ctx, household);
        self.set_off(ctx, household, day);
    }
}
