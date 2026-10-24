//! Gas lit each evening (M10b slice DY, third part; ADR-0034 §8; research 12-09 §5.2, §5.5): at
//! dusk each main's district is checked, the gas its works' holder keeps and the main's capacity
//! against the burners connected to it, lit for the evening's hours, and the gas served is drawn
//! from the works' household. A connection lit is light at its home after dark, as a lamp is.

//!
//! By choice (the fourth part): households along a main laid whole connect where gas light costs
//! them less than their lamp or their work, and pay its works by the meter for the gas served.

use civ_core::time::DAYS_PER_YEAR;
use civ_core::{PermanentId, Rng64};
use civ_land::gas::{Cause, Connection, Main};
use civ_land::ways::Keeper;

use super::{AgentEvent, Ctx, Flow, Household, Population, cell_centre, cell_of, fuel_per_day};
use crate::decide::WayFacts;
use crate::ledger::{Channel, Leg};
use crate::params::{Catalog, GoodUse};
use crate::{decide, make};

/// The meter's price of gas `gas` (M10b slice DY, fourth part): the good it is paid in and how
/// much of it a kilogram of gas costs, the coal a retort charges for it (the first input of the
/// recipe that makes it, over the gas that recipe makes). `None` where no recipe makes it.
pub fn meter_price(catalog: &Catalog, gas: usize) -> Option<(usize, f64)> {
    catalog.recipes.iter().find_map(|r| {
        let made = r.outputs.iter().find(|&&(o, _)| o == gas)?.1;
        let &(coal, kg) = r.inputs.first()?;
        (made > 0.0 && kg > 0.0).then_some((coal, kg / made))
    })
}

/// What a unit of good `g` costs a household whose own costs are `costs`, hours (M10b slice DY,
/// fourth part): its own figure, or for a fuel it has none for (coal, dug rather than made), the
/// heat of it in the hearth's measure (firewood) at its own cost of that.
fn fuel_hours(goods: &[crate::params::GoodDef], costs: &[Option<f64>], g: usize) -> Option<f64> {
    costs.get(g).copied().flatten().or_else(|| {
        let measure = goods.iter().position(|d| {
            d.purpose == GoodUse::Fuel && d.fuel.is_none_or(|f| f.technique.is_none())
        })?;
        let per_kg = costs.get(measure).copied().flatten()?;
        goods
            .get(g)?
            .fuel
            .map(|_| per_kg * crate::params::fuel_ratio(goods, g))
    })
}

/// Whether `stores` hold a gasworks (M10b slice DY, fourth part): a whole holder of some gas and
/// the whole fixed tools of a recipe that makes it (a retort).
fn holds_works(catalog: &Catalog, stores: &[f64]) -> bool {
    let whole = |g: usize| stores.get(g).copied().unwrap_or(0.0) >= 1.0 - 1e-9;
    catalog.goods.iter().enumerate().any(|(h, d)| {
        let Some(gas) = d.holder.and_then(|x| x.good) else {
            return false;
        };
        whole(h)
            && catalog.recipes.iter().any(|r| {
                r.outputs.iter().any(|&(o, _)| o == gas)
                    && !r.tools.is_empty()
                    && r.tools.iter().all(|&t| whole(t))
            })
    })
}

/// Keyed-randomness purpose of a household's day of the year for weighing a gas main.
const PURPOSE_MAIN_REVIEW: u64 = 0x6761_736d_6169_6e72; // "gasmainr"

/// What a kilogram of gas `gas` sold by the meter gains the works that makes it, hours of its own
/// work (M10b slice DY, fourth part): the meter gives back the coal its retort charged, so the gain
/// is what else that charge leaves (coke), at the works' own cost of it or a fuel's heat in
/// firewood, less the hours of stoking it took.
fn gas_gain_h(catalog: &Catalog, costs: &[Option<f64>], gas: usize) -> f64 {
    let Some((r, made)) = catalog.recipes.iter().find_map(|r| {
        let made = r.outputs.iter().find(|&&(o, _)| o == gas)?.1;
        (made > 0.0).then_some((r, made))
    }) else {
        return 0.0;
    };
    let left: f64 = r
        .outputs
        .iter()
        .filter(|&&(o, _)| o != gas)
        .map(|&(o, a)| a * fuel_hours(&catalog.goods, costs, o).unwrap_or(0.0))
        .sum();
    (left - r.unit_h) / made
}

/// What a works reckons of a main to cell `to` at its review (M10b slice DY, fourth part): its
/// cells and length, metres, the homes along it that would connect, what they would repay it a
/// year and what laying it and fitting them takes, hours.
#[derive(Clone, Debug, PartialEq)]
pub struct MainReckoning {
    pub to: u32,
    pub cells: Vec<u32>,
    pub length_m: f64,
    pub homes: u32,
    pub worth_h_year: f64,
    pub cost_h: f64,
}

/// The hours a connection's burners burn of an evening (research 12-09 §2.3: house lamps lit
/// some four hours a day).
pub const GAS_EVENING_H: f64 = 4.0;

impl Population {
    /// Schedules today's lighting of gas at dusk (M10b slice DY), while any main is laid.
    pub(super) fn schedule_gas(&mut self, ctx: &mut Ctx) {
        if ctx.land.gas.mains.is_empty() {
            return;
        }
        let doy = civ_land::day_of_year(ctx.now.day_index());
        let (rise, set) = crate::needs::daylight(ctx.params.latitude_deg, doy);
        let start = ctx.now.plus_minutes(-ctx.now.minute_of_day());
        ctx.schedule
            .push((start.plus_minutes(set.max(rise + 1)), AgentEvent::Gas));
    }

    /// The evening's gas (M10b slice DY): each main in turn, households along it weigh connecting
    /// (the fourth part), then its connections are lit for [`GAS_EVENING_H`] from now, as far as
    /// the main carries their burners and its works' gas lasts, the gas served is drawn from the
    /// works' household's stores, and each household served pays the works for it by the meter.
    pub(super) fn gas_evening(&mut self, ctx: &mut Ctx) {
        let goods = &ctx.catalog.goods;
        // The piped burner and the gas it burns.
        let Some((burner, gas, kg_h)) = goods.iter().enumerate().find_map(|(g, d)| {
            let l = d.light.filter(|l| l.piped)?;
            Some((g, l.fuel?, l.kg_per_hour))
        }) else {
            return;
        };
        let (now, params) = (ctx.now, ctx.params);
        let day = now.day_index();
        let price = meter_price(ctx.catalog, gas);
        let mains: Vec<_> = ctx.land.gas.mains.iter().map(|m| (m.id, m.works)).collect();
        for (main, works) in mains {
            if let Some(price) = price {
                self.gas_connect(ctx, main, burner, kg_h, price);
            }
            let goods = &ctx.catalog.goods;
            let held = match self
                .hh_index
                .get(&works)
                .and_then(|&hd| self.households.get_mut(hd))
            {
                Some(x) => {
                    let members = x.members.len().max(1);
                    x.settle_stores(now, goods, &|d| fuel_per_day(params, members, d));
                    x.stores.resize(goods.len(), 0.0);
                    x.stores[gas]
                }
                None => 0.0,
            };
            let drawn = ctx
                .land
                .gas
                .evening(main, day, now.minutes(), GAS_EVENING_H, held, kg_h);
            if drawn > 0.0
                && let Some(x) = self
                    .hh_index
                    .get(&works)
                    .and_then(|&hd| self.households.get_mut(hd))
            {
                x.stores[gas] = (x.stores[gas] - drawn).max(0.0);
                x.flows.add(Flow::Used, gas, drawn);
            }
            if let Some(price) = price {
                self.gas_pay(ctx, main, works, kg_h, price);
            }
        }
    }

    /// Households along main `main` weigh connecting (M10b slice DY, fourth part; brief §2): one
    /// whose home lies on the main or beside it, not its works and not yet connected, connects
    /// where an hour of gas light at the meter's `price` costs it fewer of its own hours than an
    /// hour of its work, or than an hour of a lamp it holds whole, and it holds an evening's
    /// price; the works fits it a burner the works holds, at its own cost. One cut off is
    /// restored once it holds an evening's price again. Only a main laid whole and not broken is
    /// weighed.
    fn gas_connect(
        &mut self,
        ctx: &mut Ctx,
        main: PermanentId,
        burner: usize,
        kg_h: f64,
        (coal, coal_per_kg): (usize, f64),
    ) {
        let Some(m) = ctx
            .land
            .gas
            .mains
            .iter()
            .find(|m| m.id == main && m.whole() && !m.broken)
        else {
            return;
        };
        let works = m.works;
        let w = i64::from(ctx.map.width.max(1));
        let near = |cell: usize| {
            let (x, y) = (cell as i64 % w, cell as i64 / w);
            m.cells.iter().any(|&c| {
                let (cx, cy) = (i64::from(c) % w, i64::from(c) / w);
                (cx - x).abs() <= 1 && (cy - y).abs() <= 1
            })
        };
        let mut along: Vec<PermanentId> = self
            .households
            .iter()
            .filter(|(_, x)| x.id != works && near(cell_of(ctx.map, x.home)))
            .map(|(_, x)| x.id)
            .collect();
        along.sort_unstable();
        // An evening's gas for a burner at the meter, in coal.
        let evening = kg_h * GAS_EVENING_H * coal_per_kg;
        for h in along {
            let Some(x) = self.household(h) else {
                continue;
            };
            let coal_held = x.stores.get(coal).copied().unwrap_or(0.0);
            if let Some(c) = ctx
                .land
                .gas
                .connections
                .iter_mut()
                .find(|c| c.household == h && c.main == main && c.cut_off)
            {
                if coal_held >= evening * f64::from(c.burners) {
                    c.cut_off = false;
                }
                continue;
            }
            if coal_held < evening || ctx.land.gas.connections.iter().any(|c| c.household == h) {
                continue;
            }
            if !self.wants_gas(ctx, x, (coal, coal_per_kg), kg_h) {
                continue;
            }
            let Some(building) = ctx
                .land
                .buildings
                .iter()
                .find(|b| b.household == h)
                .map(|b| b.id)
            else {
                continue;
            };
            // The works fits a burner it holds whole.
            let Some(f) = self.household_mut(works) else {
                return;
            };
            if f.stores.get(burner).copied().unwrap_or(0.0) < 1.0 - 1e-9 {
                return;
            }
            f.stores[burner] -= 1.0;
            f.flows.add(Flow::Used, burner, 1.0);
            ctx.land
                .gas
                .connections
                .push(Connection::new(building, h, main, 1));
        }
    }

    /// Whether household `x` would have its home lit by gas at the meter's `price` (M10b slice
    /// DY, fourth part): an hour of a burner's gas costs it fewer of its own hours than an hour of
    /// its work, or than an hour of a lamp it holds whole.
    fn wants_gas(
        &self,
        ctx: &Ctx,
        x: &Household,
        (coal, coal_per_kg): (usize, f64),
        kg_h: f64,
    ) -> bool {
        let goods = &ctx.catalog.goods;
        let costs = self.own_costs_of(ctx, x);
        let Some(coal_h) = fuel_hours(goods, &costs, coal) else {
            return false;
        };
        // Its work's hour, or its lamp's where it holds one whole.
        let lamp_hour = goods
            .iter()
            .enumerate()
            .filter(|&(g, d)| {
                d.light.is_some_and(|l| !l.piped)
                    && make::in_use(goods, g, x.stores.get(g).copied().unwrap_or(0.0))
                        >= decide::MIN_TOOL
            })
            .filter_map(|(g, _)| super::light_hour_cost(goods, &costs, g))
            .fold(1.0f64, f64::min);
        kg_h * coal_per_kg * coal_h < lamp_hour
    }

    /// Each household served this evening on main `main` pays its works by the meter (M10b
    /// slice DY, fourth part): its burners' gas at the meter's `price`, by the `Gas` channel. One
    /// that cannot pay it all pays what it holds and is cut off.
    fn gas_pay(
        &mut self,
        ctx: &mut Ctx,
        main: PermanentId,
        works: PermanentId,
        kg_h: f64,
        (coal, coal_per_kg): (usize, f64),
    ) {
        let (now, params) = (ctx.now, ctx.params);
        let (goods, day) = (&ctx.catalog.goods, now.day_index());
        let served: Vec<(usize, PermanentId, f64)> = ctx
            .land
            .gas
            .connections
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                c.main == main
                    && c.household != works
                    && c.evening == day
                    && c.cause == Cause::Served
            })
            .map(|(i, c)| {
                let owed = f64::from(c.burners) * kg_h * GAS_EVENING_H * coal_per_kg;
                (i, c.household, owed)
            })
            .collect();
        for (i, household, owed) in served {
            let held = match self.household_mut(household) {
                Some(x) => {
                    let members = x.members.len().max(1);
                    x.settle_stores(now, goods, &|d| fuel_per_day(params, members, d));
                    x.stores.get(coal).copied().unwrap_or(0.0).max(0.0)
                }
                None => 0.0,
            };
            let pay = owed.min(held);
            let leg = Leg {
                from: household,
                to: works,
                good: coal,
                amount: pay,
            };
            let paid = pay > 0.0 && self.transfer(now, params, goods, &[leg], Channel::Gas);
            if !paid || pay + 1e-9 < owed {
                ctx.land.gas.connections[i].cut_off = true;
            }
        }
    }

    /// Begins a main from household `works`' home to cell `to` (M10b slice DY, third part, second
    /// step): routed along ways whose stretches' keepers give leave, nobody's, the works' own or
    /// its polity's ([`civ_land::gas::plan_main`]), marked out at its length with nothing yet
    /// laid, and carrying what the land profile's bore does. Returns its id; `None` where the land
    /// profile lays no main or no route keeps to such ways.
    pub fn begin_main(&self, ctx: &mut Ctx, works: PermanentId, to: u32) -> Option<PermanentId> {
        let params = ctx.land_params.gas_main?;
        let (cells, length_m) = self.main_route(ctx, works, to)?;
        let id = ctx.ids.allocate();
        ctx.land.gas.mains.push(Main {
            id,
            works,
            cells,
            capacity_kg_h: params.capacity_kg_h,
            broken: false,
            length_m: length_m as f32,
            laid_m: 0.0,
            worth_h: 0.0,
        });
        Some(id)
    }

    /// The route of a main from household `works`' home to cell `to` and its length, metres, as
    /// [`Population::begin_main`] lays it.
    fn main_route(&self, ctx: &Ctx, works: PermanentId, to: u32) -> Option<(Vec<u32>, f64)> {
        let x = self.household(works)?;
        let from = cell_of(ctx.map, x.home) as u32;
        let polity = x
            .settlement
            .and_then(|s| self.polity_of(s))
            .map(|pi| self.polities[pi].id);
        let cells =
            civ_land::gas::plan_main(&ctx.land.ways, ctx.map.width, from, to, |k| match k {
                Keeper::Nobody => true,
                Keeper::Household(h) => h == works,
                Keeper::Polity(p) => Some(p) == polity,
            })?;
        let length_m: f64 =
            civ_land::rail::runs(&cells, ctx.map.width, f64::from(ctx.map.cell_size_m))
                .iter()
                .sum();
        Some((cells, length_m))
    }

    /// Households whose day of the year it is, holding a gasworks, weigh a main from it (M10b
    /// slice DY, fourth part); none where the land lays no main.
    pub(super) fn main_reviews(&mut self, ctx: &mut Ctx, day: i64) {
        if ctx.land_params.gas_main.is_none() {
            return;
        }
        let due: Vec<PermanentId> = self
            .households
            .iter()
            .filter(|(_, x)| holds_works(ctx.catalog, &x.stores))
            .map(|(_, x)| x.id)
            .filter(|h| {
                Rng64::from_key(&[ctx.seed, PURPOSE_MAIN_REVIEW, h.get()]).next_u64()
                    % DAYS_PER_YEAR as u64
                    == day.rem_euclid(DAYS_PER_YEAR) as u64
            })
            .collect();
        for h in due {
            self.review_main(ctx, h);
        }
    }

    /// What a main from household `works`' gasworks to each other home of its settlement would
    /// repay it, as [`Population::review_main`] reckons it, the best first (M10b slice DY, fourth
    /// part): a works with gas making known, a whole retort and holder, and no main of its own
    /// still being laid weighs each home it can reach along ways with leave. The homes along a
    /// route, its end's among them, not yet connected, that would have gas light at the meter's
    /// price would each repay a burner's evenings a year at the works' gain on a kilogram
    /// ([`gas_gain_h`]); against that, a metre's digging and laying and its pipe at the works' own
    /// cost (pipe the works holds counts as in hand where it can put no figure on it), and a
    /// burner each at its own cost. Empty where none reaches or the works cannot weigh one.
    pub(super) fn main_reckonings(&self, ctx: &Ctx, works: PermanentId) -> Vec<MainReckoning> {
        let mut out = Vec::new();
        let Some(params) = ctx.land_params.gas_main else {
            return out;
        };
        let catalog = ctx.catalog;
        let goods = &catalog.goods;
        let Some((burner, gas, kg_h)) = goods.iter().enumerate().find_map(|(g, d)| {
            let l = d.light.filter(|l| l.piped)?;
            Some((g, l.fuel?, l.kg_per_hour))
        }) else {
            return out;
        };
        let Some(price) = meter_price(catalog, gas) else {
            return out;
        };
        let Some(x) = self.household(works) else {
            return out;
        };
        let knows = catalog
            .recipes
            .iter()
            .filter(|r| r.outputs.iter().any(|&(o, _)| o == gas))
            .all(|r| {
                r.technique
                    .is_none_or(|t| self.household_knows(&x.members, t))
            });
        if x.members.is_empty()
            || !knows
            || !holds_works(catalog, &x.stores)
            || ctx
                .land
                .gas
                .mains
                .iter()
                .any(|m| m.works == works && !m.whole())
        {
            return out;
        }
        let costs = self.own_costs_of(ctx, x);
        let year_h = kg_h * GAS_EVENING_H * DAYS_PER_YEAR as f64 * gas_gain_h(catalog, &costs, gas);
        if year_h <= 0.0 {
            return out;
        }
        let pipe_held = x.stores.get(params.good).copied().unwrap_or(0.0);
        let pipe_h = costs.get(params.good).copied().flatten();
        let burner_h = costs.get(burner).copied().flatten().unwrap_or(0.0);
        let w = i64::from(ctx.map.width.max(1));
        // Homes of its settlement not yet connected, by household, with whether each would have
        // gas light.
        let homes: Vec<(PermanentId, u32, bool)> = self
            .households
            .iter()
            .filter(|(_, y)| {
                y.id != works
                    && y.settlement == x.settlement
                    && !ctx.land.gas.connections.iter().any(|c| c.household == y.id)
                    && ctx.land.buildings.iter().any(|b| b.household == y.id)
            })
            .map(|(_, y)| {
                (
                    y.id,
                    cell_of(ctx.map, y.home) as u32,
                    self.wants_gas(ctx, y, price, kg_h),
                )
            })
            .collect();
        for &(_, to, wants) in &homes {
            if !wants {
                continue;
            }
            let Some((cells, length_m)) = self.main_route(ctx, works, to) else {
                continue;
            };
            let near = |cell: u32| {
                let (px, py) = (i64::from(cell) % w, i64::from(cell) / w);
                cells.iter().any(|&c| {
                    ((i64::from(c) % w) - px).abs() <= 1 && ((i64::from(c) / w) - py).abs() <= 1
                })
            };
            let along = homes
                .iter()
                .filter(|&&(_, c, wants)| wants && near(c))
                .count() as u32;
            let pipe_kg = length_m * params.good_kg_m;
            let pipe_cost = match pipe_h {
                Some(h) => pipe_kg * h,
                None if pipe_held + 1e-9 >= pipe_kg => 0.0,
                None => continue,
            };
            out.push(MainReckoning {
                to,
                length_m,
                homes: along,
                worth_h_year: f64::from(along) * year_h,
                cost_h: length_m * params.lay_h_m + pipe_cost + f64::from(along) * burner_h,
                cells,
            });
        }
        let net = |r: &MainReckoning| r.worth_h_year * params.life_years - r.cost_h;
        out.sort_by(|a, b| net(b).total_cmp(&net(a)).then(a.to.cmp(&b.to)));
        out
    }

    /// Household `works` weighs a main from its gasworks (M10b slice DY, fourth part): it begins
    /// the one that repays it most ([`Population::main_reckonings`]) where the homes along it
    /// would repay more over the main's life than laying it and fitting them takes, marked out
    /// with what it reckoned they repay.
    fn review_main(&mut self, ctx: &mut Ctx, works: PermanentId) {
        let Some(params) = ctx.land_params.gas_main else {
            return;
        };
        let Some(best) = self.main_reckonings(ctx, works).into_iter().next() else {
            return;
        };
        if best.worth_h_year * params.life_years <= best.cost_h {
            return;
        }
        if let Some(main) = self.begin_main(ctx, works, best.to)
            && let Some(m) = ctx.land.gas.mains.iter_mut().find(|m| m.id == main)
        {
            m.worth_h = best.worth_h_year as f32;
        }
    }

    /// The main household `x`'s gasworks is laying, as way work for its people (M10b slice DY,
    /// fourth part): where its work stands, the walk there, the hours still to lay, and each hour's
    /// worth, the hours a year it was reckoned to repay over its life spread over its laying.
    /// `None` without one, or without a metre's pipe in hand.
    pub(super) fn main_facts(&self, ctx: &Ctx, x: &Household) -> Option<WayFacts> {
        let params = ctx.land_params.gas_main?;
        let m = ctx
            .land
            .gas
            .mains
            .iter()
            .find(|m| m.works == x.id && !m.whole())?;
        if x.stores.get(params.good).copied().unwrap_or(0.0) < params.good_kg_m {
            return None;
        }
        let n = m.cells.len();
        let share = f64::from(m.laid_m) / f64::from(m.length_m).max(1e-3);
        let cell = *m
            .cells
            .get(((share * n as f64) as usize).min(n.checked_sub(1)?))?;
        let at = cell_centre(ctx.map, cell as usize);
        let pace = ctx.params.nav.tobler_ms(0.0).max(1e-6);
        let labour_h = f64::from(m.length_m) * params.lay_h_m;
        Some(WayFacts {
            way: m.id,
            at,
            walk_min: f64::from((at.0 - x.home.0).hypot(at.1 - x.home.1)) / pace / 60.0,
            left_h: f64::from(m.length_m - m.laid_m).max(0.0) * params.lay_h_m,
            saves_per_hour: f64::from(m.worth_h) * params.life_years / labour_h.max(1e-6),
            duty: 0.0,
            full: false,
        })
    }

    /// What mains from household `works`' gasworks would repay it, the best first: for tests.
    #[doc(hidden)]
    pub fn main_reckonings_for_tests(&self, ctx: &Ctx, works: PermanentId) -> Vec<MainReckoning> {
        self.main_reckonings(ctx, works)
    }

    /// Household `works` weighs a main now, whatever its day of the year: for tests.
    #[doc(hidden)]
    pub fn review_main_for_tests(&mut self, ctx: &mut Ctx, works: PermanentId) {
        self.review_main(ctx, works);
    }

    /// `hours` of work by someone of `household` on main `main` (M10b slice DY, third part,
    /// second step): its works' own household digs, lays and fills in its pipe, a metre each
    /// `lay_h_m` hours, each taking its pipe from the household's store, no more hours than the
    /// pipe held allows. The main carries gas only once whole.
    pub(super) fn main_work(
        &mut self,
        ctx: &mut Ctx,
        household: PermanentId,
        main: PermanentId,
        hours: f64,
    ) {
        let Some(params) = ctx.land_params.gas_main else {
            return;
        };
        let Some(m) = ctx
            .land
            .gas
            .mains
            .iter_mut()
            .find(|m| m.id == main && m.works == household && !m.whole())
        else {
            return;
        };
        let Some(x) = self.household_mut(household) else {
            return;
        };
        let held = x.stores.get(params.good).copied().unwrap_or(0.0).max(0.0);
        // Metres the hours lay, no more than the pipe held makes or the main has left.
        let metres = (hours.max(0.0) / params.lay_h_m.max(1e-9))
            .min(held / params.good_kg_m.max(1e-9))
            .min(f64::from(m.length_m - m.laid_m).max(0.0));
        m.laid_m = (m.laid_m + metres as f32).min(m.length_m);
        if m.length_m - m.laid_m < 1e-3 {
            m.laid_m = m.length_m;
        }
        let pipe = (metres * params.good_kg_m).min(held);
        if pipe > 0.0 {
            x.stores[params.good] = held - pipe;
            x.flows.add(Flow::Used, params.good, pipe);
        }
    }

    /// Begins a main from household `works`' home to cell `to`: for tests.
    #[doc(hidden)]
    pub fn begin_main_for_tests(
        &self,
        ctx: &mut Ctx,
        works: PermanentId,
        to: u32,
    ) -> Option<PermanentId> {
        self.begin_main(ctx, works, to)
    }

    /// `hours` of work by someone of `household` on main `main`: for tests.
    #[doc(hidden)]
    pub fn main_work_for_tests(
        &mut self,
        ctx: &mut Ctx,
        household: PermanentId,
        main: PermanentId,
        hours: f64,
    ) {
        self.main_work(ctx, household, main, hours);
    }

    /// The evening's gas lit now: for tests.
    #[doc(hidden)]
    pub fn gas_evening_for_tests(&mut self, ctx: &mut Ctx) {
        self.gas_evening(ctx);
    }
}
