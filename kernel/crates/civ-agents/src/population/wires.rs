//! Lines of current begun and laid (M11a, third part, fourth step; ADR-0036 §3): a line is routed
//! along ways whose keepers give leave, as a gas main is, marked out at its length with nothing
//! laid, and strung by its station's household a metre at a time from the conductor it holds. It
//! carries current only once whole. A building hooked up to a line is served each evening as far
//! as its station's current and its lines carry it (ADR-0036 §4), lit as a gas burner lights it.
//! A household whose dynamo turns beside its own installation hooks its home up to it (the fourth
//! part). One that also knows how a dynamo is made weighs a line to other homes by what they would
//! pay by the meter, as a gasworks weighs a main; homes along a whole line hook up where light by
//! wire costs them less than their lamp, pay its station each evening they are served, and are
//! cut off when they cannot (the fifth part; ADR-0036 §6).

use civ_core::time::DAYS_PER_YEAR;
use civ_core::{PermanentId, Rng64};
use civ_land::ways::Keeper;
use civ_land::wire::{Cause, Hookup, Line};

use super::*;
use crate::decide::WayFacts;
use crate::ledger::Channel;

/// The kilowatts a home hooked up to the wire asks of an evening (a design prior: a few
/// incandescent lamps of the 1880s at some 60 W each; research 07-13 §5.2 E04).
pub const HOME_LIGHT_KW: f64 = 0.2;

/// What a kilowatt-hour costs at the meter, in days of one person's food (a design prior, as a
/// carer's fee is: research 07-13 asks for prices set where they are paid rather than money
/// carried over; a home's evening, 0.8 kWh, comes to some 200 kcal at 2,500 a day).
pub const METER_FOOD_DAYS_KWH: f64 = 0.1;

/// Keyed-randomness purpose of a station's day of the year for weighing a line.
const PURPOSE_LINE_REVIEW: u64 = 0x7769_7265_6c69_6e65; // "wireline"

/// What a station reckons of a line to cell `to` at its review (M11a, fifth part): its cells and
/// length, metres, the homes along it that would hook up, what they would repay it a year and
/// what stringing it takes, hours.
#[derive(Clone, Debug, PartialEq)]
pub struct LineReckoning {
    pub to: u32,
    pub cells: Vec<u32>,
    pub length_m: f64,
    pub homes: u32,
    pub worth_h_year: f64,
    pub cost_h: f64,
}

/// The meter's price of a kilowatt-hour, kcal of food.
fn meter_kcal_kwh(ctx: &Ctx) -> f64 {
    METER_FOOD_DAYS_KWH * ctx.params.household.daily_kcal_per_person
}

/// The hours of its own work a kcal of food costs a household whose own costs are `costs`: its
/// cheapest food it puts a figure on. `None` where it puts one on none.
pub(super) fn hours_per_kcal(
    goods: &[crate::params::GoodDef],
    costs: &[Option<f64>],
) -> Option<f64> {
    goods
        .iter()
        .enumerate()
        .filter(|(_, d)| d.purpose == GoodUse::Food && d.kcal_per_kg > 0.0 && !d.kept_back())
        .filter_map(|(g, d)| Some(costs.get(g).copied().flatten()? / d.kcal_per_kg))
        .filter(|h| *h > 0.0)
        .min_by(f64::total_cmp)
}

impl Population {
    /// The dynamos of the catalogue, by good index.
    fn dynamo_goods(ctx: &Ctx) -> Vec<usize> {
        ctx.catalog
            .goods
            .iter()
            .enumerate()
            .filter(|(_, d)| {
                d.power
                    .is_some_and(|p| p.source == crate::params::PowerSource::Shaft)
            })
            .map(|(g, _)| g)
            .collect()
    }

    /// Whether household `x` holds a whole dynamo beside an installation of its own that is ready,
    /// or beside an engine it holds with the fuel it burns for an evening (M11b, first part).
    fn turns_dynamo(ctx: &Ctx, x: &Household, dynamos: &[usize]) -> bool {
        dynamos
            .iter()
            .any(|&g| x.stores.get(g).copied().unwrap_or(0.0) >= 1.0 - 1e-9)
            && (ctx
                .land
                .installations
                .iter()
                .enumerate()
                .any(|(i, s)| s.owner == x.id && ctx.land.installation_ready(i))
                || super::engine_kw(&ctx.catalog.goods, &x.stores, super::gas::GAS_EVENING_H).0
                    > 0.0)
    }

    /// Whether cell `cell` lies on or beside a whole, unbroken line of station `station`, where a
    /// home may hook up without another line.
    fn by_a_line(ctx: &Ctx, station: PermanentId, cell: u32) -> bool {
        let w = i64::from(ctx.map.width.max(1));
        let (px, py) = (i64::from(cell) % w, i64::from(cell) / w);
        ctx.land
            .wires
            .lines
            .iter()
            .filter(|l| l.station == station && l.whole() && !l.broken)
            .any(|l| {
                l.cells.iter().any(|&c| {
                    ((i64::from(c) % w) - px).abs() <= 1 && ((i64::from(c) / w) - py).abs() <= 1
                })
            })
    }

    /// Household `h`'s finished, standing home, if it has one.
    fn finished_home(ctx: &Ctx, h: PermanentId) -> Option<PermanentId> {
        ctx.land
            .buildings
            .iter()
            .find(|b| {
                b.household == h
                    && b.finished()
                    && b.standing()
                    && ctx.catalog.is_dwelling(&b.spec.program)
            })
            .map(|b| b.id)
    }

    /// Each midnight, a household that holds a whole dynamo beside an installation of its own that
    /// is ready, and whose finished home is hooked up to nothing, hooks it up on a line from its
    /// own station of no length (M11a, fourth part; ADR-0036 §6: its own use first); then the
    /// stations whose day of the year it is weigh a line to other homes (the fifth part). Nothing
    /// where the land lays no line or no installation stands.
    pub(super) fn wire_reviews(&mut self, ctx: &mut Ctx, day: i64) {
        if ctx.land_params.wire.is_none() {
            return;
        }
        let dynamos = Self::dynamo_goods(ctx);
        if dynamos.is_empty() {
            return;
        }
        let mut due: Vec<(PermanentId, PermanentId, u32)> = Vec::new();
        let mut stations: Vec<PermanentId> = Vec::new();
        for (_, x) in self.households.iter() {
            if !Self::turns_dynamo(ctx, x, &dynamos) {
                continue;
            }
            if Rng64::from_key(&[ctx.seed, PURPOSE_LINE_REVIEW, x.id.get()]).next_u64()
                % DAYS_PER_YEAR as u64
                == day.rem_euclid(DAYS_PER_YEAR) as u64
            {
                stations.push(x.id);
            }
            let Some(home) = Self::finished_home(ctx, x.id) else {
                continue;
            };
            if ctx.land.wires.hookups.iter().any(|h| h.building == home) {
                continue;
            }
            due.push((x.id, home, cell_of(ctx.map, x.home) as u32));
        }
        for (household, building, cell) in due {
            if let Some(line) = self.begin_line(ctx, household, cell, cell, None) {
                self.begin_hookup(ctx, building, household, line, HOME_LIGHT_KW);
            }
        }
        for station in stations {
            self.review_wire_line(ctx, station);
        }
    }

    /// The midnight review of hooking homes up: for tests.
    #[doc(hidden)]
    pub fn wire_reviews_for_tests(&mut self, ctx: &mut Ctx) {
        let day = ctx.now.day_index();
        self.wire_reviews(ctx, day);
    }

    /// Whether household `x` would have its home lit by wire at the meter (M11a, fifth part): an
    /// hour of a home's current, in hours of its own food, costs it less than an hour of its work,
    /// or than an hour of a lamp it holds whole, as gas light is weighed.
    fn wants_wire(&self, ctx: &Ctx, x: &Household) -> bool {
        self.wire_spares_h(ctx, x) > 0.0
    }

    /// The hours of its own an hour of light by wire at the meter would spare household `x`: an
    /// hour of its work, or of a lamp it holds whole, the least, less what the hour's current
    /// costs it in its own food. Nothing (or less) where it puts no figure on its food.
    fn wire_spares_h(&self, ctx: &Ctx, x: &Household) -> f64 {
        let goods = &ctx.catalog.goods;
        let costs = self.own_costs_of(ctx, x);
        let Some(per_kcal) = hours_per_kcal(goods, &costs) else {
            return 0.0;
        };
        let lamp_hour = goods
            .iter()
            .enumerate()
            .filter(|&(g, d)| {
                d.light.is_some_and(|l| !l.piped)
                    && make::in_use(goods, g, x.stores.get(g).copied().unwrap_or(0.0))
                        >= crate::decide::MIN_TOOL
            })
            .filter_map(|(g, _)| super::light_hour_cost(goods, &costs, g))
            .fold(1.0f64, f64::min);
        lamp_hour - HOME_LIGHT_KW * meter_kcal_kwh(ctx) * per_kcal
    }

    /// Where the settlement's people could string a line of current together (M11a, eighth part;
    /// ADR-0036 §6), if anywhere, with the station it leaves: from the home of a household whose
    /// dynamo turns beside its own installation and that has current to spare, to a home of the
    /// settlement, along ways with leave, where two or more homes along it, not yet hooked up,
    /// would have light by wire, as many as the spare current serves, and none of the polity's
    /// lines of current is being strung. Each such household is reckoned to spare a year an
    /// evening's hours of its lamp or work less its current at the meter; the line chosen is the
    /// one whose pooled hours spared over a line's life most exceed its stringing and its copper
    /// at the cheapest any of the settlement's households can have it. As a passing place: the
    /// home cell it runs to, its stringing, a line's life, what it spares each of those
    /// households, and the settlement's households who would share its work.
    pub(crate) fn public_current(
        &self,
        ctx: &Ctx,
        settlement: PermanentId,
        households: &[PermanentId],
    ) -> Option<(PermanentId, super::way_works::PublicWay)> {
        let params = ctx.land_params.wire?;
        if !ctx
            .catalog
            .policies
            .iter()
            .any(|d| d.kind == crate::polity::PolicyKind::LayCurrent)
        {
            return None;
        }
        let dynamos = Self::dynamo_goods(ctx);
        let ours: Vec<&Household> = households
            .iter()
            .filter_map(|&h| self.household(h))
            .filter(|x| x.settlement == Some(settlement) && !x.members.is_empty())
            .collect();
        let stations: Vec<&Household> = ours
            .iter()
            .copied()
            .filter(|x| Self::turns_dynamo(ctx, x, &dynamos))
            .collect();
        if stations.is_empty() {
            return None;
        }
        // One line at a time: none of the polity's being strung.
        let wires = &ctx.land.wires;
        if let Some(pi) = self.polity_of(settlement)
            && self.polities[pi].ways.iter().any(|(_, s)| {
                s.work
                    .is_some_and(|w| wires.lines.iter().any(|l| l.id == w && !l.whole()))
            })
        {
            return None;
        }
        // Copper at the cheapest any of the settlement's households can have it (what they hold
        // counts as in hand where none can put a figure on it).
        let copper_h = ours
            .iter()
            .filter_map(|x| {
                self.own_costs_of(ctx, x)
                    .get(params.good)
                    .copied()
                    .flatten()
            })
            .reduce(f64::min);
        let copper_held: f64 = ours
            .iter()
            .map(|x| x.stores.get(params.good).copied().unwrap_or(0.0).max(0.0))
            .sum();
        let year = super::gas::GAS_EVENING_H * DAYS_PER_YEAR as f64;
        // Homes not yet hooked up, with what light by wire would spare each a year.
        let homes: Vec<(PermanentId, u32, f64)> = ours
            .iter()
            .filter(|x| {
                !wires.hookups.iter().any(|h| h.household == x.id)
                    && Self::finished_home(ctx, x.id).is_some()
            })
            .map(|x| {
                (
                    x.id,
                    cell_of(ctx.map, x.home) as u32,
                    self.wire_spares_h(ctx, x) * year,
                )
            })
            .filter(|h| h.2 > 0.0)
            .collect();
        let w = i64::from(ctx.map.width.max(1));
        let mut best: Option<(f64, PermanentId, super::way_works::PublicWay)> = None;
        for s in stations {
            let asked: f64 = wires
                .hookups
                .iter()
                .filter(|h| {
                    !h.cut_off
                        && wires
                            .lines
                            .iter()
                            .any(|l| l.id == h.line && l.station == s.id)
                })
                .map(|h| h.ask_kw)
                .sum();
            let slots = ((self.electric_kw(ctx, s.id) - asked) / HOME_LIGHT_KW)
                .floor()
                .max(0.0) as usize;
            if slots < 2 {
                continue;
            }
            let from = cell_of(ctx.map, s.home) as u32;
            // Homes by one of its lines already hook up without another.
            let beyond: Vec<(PermanentId, u32, f64)> = homes
                .iter()
                .copied()
                .filter(|h| h.0 != s.id && !Self::by_a_line(ctx, s.id, h.1))
                .collect();
            for &(_, to, _) in &beyond {
                let Some((cells, length_m)) =
                    self.wire_route(ctx, s.id, from, to).filter(|r| r.1 >= 1e-3)
                else {
                    continue;
                };
                let kg = length_m * params.good_kg_m;
                let copper = match copper_h {
                    Some(h) => kg * h,
                    None if copper_held + 1e-9 >= kg => 0.0,
                    None => continue,
                };
                let near = |cell: u32| {
                    let (px, py) = (i64::from(cell) % w, i64::from(cell) / w);
                    cells.iter().any(|&c| {
                        ((i64::from(c) % w) - px).abs() <= 1 && ((i64::from(c) / w) - py).abs() <= 1
                    })
                };
                let mut saves: Vec<(PermanentId, f64)> = beyond
                    .iter()
                    .filter(|h| near(h.1))
                    .map(|h| (h.0, h.2))
                    .collect();
                // As many as its spare current serves, those it would spare most first.
                saves.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
                saves.truncate(slots);
                if saves.len() < 2 {
                    continue;
                }
                saves.sort_unstable_by_key(|s| s.0);
                let line = super::way_works::PublicWay {
                    cell: to,
                    labour_h: length_m * params.lay_h_m,
                    life: params.life_years,
                    saves,
                    households: ours.len(),
                };
                let net = line.pooled_net_h() - copper;
                if net > 0.0 && best.as_ref().is_none_or(|(b, _, _)| net > *b) {
                    best = Some((net, s.id, line));
                }
            }
        }
        best.map(|b| (b.1, b.2))
    }

    /// Where the settlement's people could string a line of current together, as its station,
    /// the home cell it runs to, its stringing and what it would spare each household a year:
    /// for tests.
    #[doc(hidden)]
    #[allow(clippy::type_complexity)]
    pub fn public_current_for_tests(
        &self,
        ctx: &Ctx,
        settlement: PermanentId,
    ) -> Option<(PermanentId, u32, f64, Vec<(PermanentId, f64)>)> {
        let mut households: Vec<PermanentId> = self.households.iter().map(|(_, x)| x.id).collect();
        households.sort_unstable();
        self.public_current(ctx, settlement, &households)
            .map(|(s, p)| (s, p.cell, p.labour_h, p.saves))
    }

    /// The line of current law `law` of polity `pi` passed names is marked out at once from its
    /// station's home to the home cell it names (M11a, eighth part), the station's to serve, its
    /// stringing the polity's households'.
    pub(super) fn begin_public_current(&mut self, ctx: &mut Ctx, pi: usize, law: PermanentId) {
        let polity = &self.polities[pi];
        let Some(site) = polity
            .ways
            .iter()
            .find(|s| s.0 == law)
            .map(|s| s.1)
            .filter(|s| s.work.is_none())
        else {
            return;
        };
        let Some(station) = polity
            .laws
            .iter()
            .find(|l| l.id == law)
            .and_then(|l| l.target)
        else {
            return;
        };
        let Some(from) = self
            .household(station)
            .map(|x| cell_of(ctx.map, x.home) as u32)
        else {
            return;
        };
        let Some(line) = self.begin_line(ctx, station, from, site.cell, None) else {
            return;
        };
        if let Some(s) = self.polities[pi].ways.iter_mut().find(|s| s.0 == law) {
            s.1.work = Some(line);
        }
    }

    /// The line of current law `law` of polity `pi` passed names is marked out: for tests.
    #[doc(hidden)]
    pub fn begin_public_current_for_tests(&mut self, ctx: &mut Ctx, pi: usize, law: PermanentId) {
        self.begin_public_current(ctx, pi, law);
    }

    /// What a line from station `station`'s home to each other home of its settlement would repay
    /// it, as [`Population::review_wire_line`] reckons it, the best first (M11a, fifth part): a
    /// household whose dynamo turns beside its own installation, that knows how a dynamo is made
    /// and has no line of its own still being strung, weighs each home it can reach along ways
    /// with leave. The homes along a route, its end's among them, not yet hooked up, that would
    /// have light by wire at the meter, as many as the current it has to spare serves, would each
    /// repay an evening's current a day at the meter's price, in hours of the station's own food;
    /// against that, a metre's stringing and its conductor at the station's own cost (conductor
    /// it holds counts as in hand where it can put no figure on it). Empty where none reaches or
    /// the station cannot weigh one.
    pub(super) fn wire_line_reckonings(
        &self,
        ctx: &Ctx,
        station: PermanentId,
    ) -> Vec<LineReckoning> {
        let mut out = Vec::new();
        let Some(params) = ctx.land_params.wire else {
            return out;
        };
        let catalog = ctx.catalog;
        let goods = &catalog.goods;
        let Some(x) = self.household(station) else {
            return out;
        };
        let dynamos = Self::dynamo_goods(ctx);
        let knows = catalog
            .recipes
            .iter()
            .filter(|r| r.outputs.iter().any(|&(o, _)| dynamos.contains(&o)))
            .all(|r| {
                r.technique
                    .is_none_or(|t| self.household_knows(&x.members, t))
            });
        let wires = &ctx.land.wires;
        if x.members.is_empty()
            || dynamos.is_empty()
            || !knows
            || !Self::turns_dynamo(ctx, x, &dynamos)
            || wires
                .lines
                .iter()
                .any(|l| l.station == station && !l.whole())
        {
            return out;
        }
        let costs = self.own_costs_of(ctx, x);
        let Some(per_kcal) = hours_per_kcal(goods, &costs) else {
            return out;
        };
        let year_h = HOME_LIGHT_KW
            * super::gas::GAS_EVENING_H
            * DAYS_PER_YEAR as f64
            * meter_kcal_kwh(ctx)
            * per_kcal;
        // Homes the current it has to spare serves.
        let asked: f64 = wires
            .hookups
            .iter()
            .filter(|h| {
                !h.cut_off
                    && wires
                        .lines
                        .iter()
                        .any(|l| l.id == h.line && l.station == station)
            })
            .map(|h| h.ask_kw)
            .sum();
        let spare = self.electric_kw(ctx, station) - asked;
        let slots = (spare / HOME_LIGHT_KW).floor().max(0.0) as u32;
        if slots == 0 || year_h <= 0.0 {
            return out;
        }
        let held = x.stores.get(params.good).copied().unwrap_or(0.0);
        let conductor_h = costs.get(params.good).copied().flatten();
        let from = cell_of(ctx.map, x.home) as u32;
        let w = i64::from(ctx.map.width.max(1));
        // Homes of its settlement not yet hooked up, with whether each would have light by wire.
        let homes: Vec<(u32, bool)> = self
            .households
            .iter()
            .filter(|(_, y)| {
                y.id != station
                    && y.settlement == x.settlement
                    && !wires.hookups.iter().any(|h| h.household == y.id)
                    && Self::finished_home(ctx, y.id).is_some()
            })
            .map(|(_, y)| (cell_of(ctx.map, y.home) as u32, self.wants_wire(ctx, y)))
            // Homes by one of its lines already hook up without another.
            .filter(|&(c, _)| !Self::by_a_line(ctx, station, c))
            .collect();
        for &(to, wants) in &homes {
            if !wants {
                continue;
            }
            let Some((cells, length_m)) = self
                .wire_route(ctx, station, from, to)
                .filter(|r| r.1 >= 1e-3)
            else {
                continue;
            };
            let near = |cell: u32| {
                let (px, py) = (i64::from(cell) % w, i64::from(cell) / w);
                cells.iter().any(|&c| {
                    ((i64::from(c) % w) - px).abs() <= 1 && ((i64::from(c) / w) - py).abs() <= 1
                })
            };
            let along =
                (homes.iter().filter(|&&(c, wants)| wants && near(c)).count() as u32).min(slots);
            let kg = length_m * params.good_kg_m;
            let conductor = match conductor_h {
                Some(h) => kg * h,
                None if held + 1e-9 >= kg => 0.0,
                None => continue,
            };
            out.push(LineReckoning {
                to,
                length_m,
                homes: along,
                worth_h_year: f64::from(along) * year_h,
                cost_h: length_m * params.lay_h_m + conductor,
                cells,
            });
        }
        let net = |r: &LineReckoning| r.worth_h_year * params.life_years - r.cost_h;
        out.sort_by(|a, b| net(b).total_cmp(&net(a)).then(a.to.cmp(&b.to)));
        out
    }

    /// What lines from station `station` would repay it, the best first: for tests.
    #[doc(hidden)]
    pub fn wire_line_reckonings_for_tests(
        &self,
        ctx: &Ctx,
        station: PermanentId,
    ) -> Vec<LineReckoning> {
        self.wire_line_reckonings(ctx, station)
    }

    /// Station `station` weighs a line to other homes (M11a, fifth part): it begins the one that
    /// repays it most ([`Population::wire_line_reckonings`]) where the homes along it would repay
    /// more over the line's life than stringing it takes, leaving the station, marked out with
    /// what it reckoned they repay.
    fn review_wire_line(&mut self, ctx: &mut Ctx, station: PermanentId) {
        let Some(params) = ctx.land_params.wire else {
            return;
        };
        let Some(best) = self.wire_line_reckonings(ctx, station).into_iter().next() else {
            return;
        };
        if best.worth_h_year * params.life_years <= best.cost_h {
            return;
        }
        let Some(from) = self
            .household(station)
            .map(|x| cell_of(ctx.map, x.home) as u32)
        else {
            return;
        };
        if let Some(line) = self.begin_line(ctx, station, from, best.to, None)
            && let Some(l) = ctx.land.wires.lines.iter_mut().find(|l| l.id == line)
        {
            l.worth_h = best.worth_h_year;
        }
    }

    /// Station `station` weighs a line now, whatever its day of the year: for tests.
    #[doc(hidden)]
    pub fn review_wire_line_for_tests(&mut self, ctx: &mut Ctx, station: PermanentId) {
        self.review_wire_line(ctx, station);
    }

    /// The line household `x`'s station is stringing to other homes, as way work for its people
    /// (M11a, fifth part, as a gas main's): where its work stands, the walk there, the hours still
    /// to string, and each hour's worth, the hours a year it was reckoned to repay over its life
    /// spread over its stringing. `None` without one, or without a metre's conductor in hand.
    pub(super) fn wire_line_facts(&self, ctx: &Ctx, x: &Household) -> Option<WayFacts> {
        let params = ctx.land_params.wire?;
        let l = ctx
            .land
            .wires
            .lines
            .iter()
            .find(|l| l.station == x.id && !l.whole() && !l.broken)?;
        if x.stores.get(params.good).copied().unwrap_or(0.0) < params.good_kg_m {
            return None;
        }
        let n = l.cells.len();
        let share = l.laid_m / l.length_m.max(1e-3);
        let cell = *l
            .cells
            .get(((share * n as f64) as usize).min(n.checked_sub(1)?))?;
        let at = cell_centre(ctx.map, cell as usize);
        let pace = ctx.params.nav.tobler_ms(0.0).max(1e-6);
        let labour_h = l.length_m * params.lay_h_m;
        Some(WayFacts {
            way: l.id,
            at,
            walk_min: f64::from((at.0 - x.home.0).hypot(at.1 - x.home.1)) / pace / 60.0,
            left_h: (l.length_m - l.laid_m).max(0.0) * params.lay_h_m,
            saves_per_hour: l.worth_h * params.life_years / labour_h.max(1e-6),
            duty: 0.0,
            full: false,
        })
    }

    /// The route of a line for household `station` from cell `from` to cell `to` and its length,
    /// metres, as [`Population::begin_line`] lays it.
    pub(super) fn wire_route(
        &self,
        ctx: &Ctx,
        station: PermanentId,
        from: u32,
        to: u32,
    ) -> Option<(Vec<u32>, f64)> {
        let x = self.household(station)?;
        let polity = x
            .settlement
            .and_then(|s| self.polity_of(s))
            .map(|pi| self.polities[pi].id);
        let cells =
            civ_land::gas::plan_main(&ctx.land.ways, ctx.map.width, from, to, |k| match k {
                Keeper::Nobody => true,
                Keeper::Household(h) => h == station,
                Keeper::Polity(p) => Some(p) == polity,
            })?;
        let length_m: f64 =
            civ_land::rail::runs(&cells, ctx.map.width, f64::from(ctx.map.cell_size_m))
                .iter()
                .sum();
        Some((cells, length_m))
    }

    /// Begins a line for household `station` from cell `from` to cell `to`, hanging from line
    /// `parent` (`None` for one leaving the station): routed along ways whose stretches' keepers
    /// give leave, nobody's, the household's own or its polity's
    /// ([`civ_land::gas::plan_main`]), marked out at its length with nothing yet laid, rated as
    /// the land profile's `[wire]` says. Returns its id; `None` where the land lays no line or no
    /// route keeps to such ways.
    pub fn begin_line(
        &self,
        ctx: &mut Ctx,
        station: PermanentId,
        from: u32,
        to: u32,
        parent: Option<PermanentId>,
    ) -> Option<PermanentId> {
        let params = ctx.land_params.wire?;
        let (cells, length_m) = self.wire_route(ctx, station, from, to)?;
        let id = ctx.ids.allocate();
        ctx.land.wires.lines.push(Line {
            id,
            station,
            parent,
            cells,
            rating_kw: params.rating_kw,
            length_m,
            laid_m: 0.0,
            broken: false,
            worth_h: 0.0,
        });
        Some(id)
    }

    /// `hours` of work by someone of `household` on its station's line `line`, or on a line its
    /// polity strings by law (M11a, eighth part): a metre strung each `lay_h_m` hours, each taking
    /// its conductor from the household's store, no more than the conductor held allows or the
    /// line has left. On a polity's, the hours it took are the household's given to the law.
    pub(super) fn line_work(
        &mut self,
        ctx: &mut Ctx,
        household: PermanentId,
        line: PermanentId,
        hours: f64,
    ) {
        let Some(params) = ctx.land_params.wire else {
            return;
        };
        // The polity whose law strings it, if one does, and the law, where `household` is of it.
        let public = self
            .household(household)
            .and_then(|x| x.settlement)
            .and_then(|s| self.polity_of(s))
            .and_then(|pi| {
                let law = self.polities[pi]
                    .ways
                    .iter()
                    .find(|s| s.1.work == Some(line))?
                    .0;
                Some((pi, law))
            });
        let Some(l) =
            ctx.land.wires.lines.iter_mut().find(|l| {
                l.id == line && (l.station == household || public.is_some()) && !l.whole()
            })
        else {
            return;
        };
        let Some(x) = self.household_mut(household) else {
            return;
        };
        let held = x.stores.get(params.good).copied().unwrap_or(0.0).max(0.0);
        let metres = (hours.max(0.0) / params.lay_h_m.max(1e-9))
            .min(held / params.good_kg_m.max(1e-9))
            .min((l.length_m - l.laid_m).max(0.0));
        l.laid_m = (l.laid_m + metres).min(l.length_m);
        if l.length_m - l.laid_m < 1e-3 {
            l.laid_m = l.length_m;
        }
        let used = (metres * params.good_kg_m).min(held);
        if used > 0.0 {
            x.stores[params.good] = held - used;
            x.flows.add(Flow::Used, params.good, used);
        }
        if let Some((pi, law)) = public
            && metres > 0.0
        {
            self.polities[pi].give_to_way(law, household, (metres * params.lay_h_m) as f32);
        }
    }

    /// Hooks building `building` of household `household` up to line `line`, asking `ask_kw` an
    /// hour; never yet lit.
    pub fn begin_hookup(
        &self,
        ctx: &mut Ctx,
        building: PermanentId,
        household: PermanentId,
        line: PermanentId,
        ask_kw: f64,
    ) {
        ctx.land.wires.hookups.push(Hookup {
            building,
            household,
            line,
            ask_kw,
            cut_off: false,
            lit_until: -1,
            cause: Cause::Served,
            asked: 0,
            served: 0,
        });
    }

    /// The wire's evening (M11a; ADR-0036 §4): for each station with lines, in the order its
    /// first line was made, homes along its lines weigh hooking up (the fifth part), then its
    /// dynamos' current from its shafts today ([`Population::electric_kw`]) is dispatched down its
    /// tree ([`civ_land::wire::dispatch`]); each hookup on it keeps its cause and writes the evening
    /// into its service record, one served is lit
    /// for [`super::gas::GAS_EVENING_H`] from now, as a gas burner is, and each household served
    /// pays the station by the meter. Nothing where no line is laid or the land lays none.
    pub(super) fn wire_evening(&mut self, ctx: &mut Ctx) {
        let Some(params) = ctx.land_params.wire else {
            return;
        };
        if ctx.land.wires.lines.is_empty() {
            return;
        }
        let mut stations: Vec<PermanentId> = Vec::new();
        for l in &ctx.land.wires.lines {
            if !stations.contains(&l.station) {
                stations.push(l.station);
            }
        }
        let until = ctx.now.minutes() + (super::gas::GAS_EVENING_H * 60.0).round() as i64;
        for station in stations {
            self.wire_connect(ctx, station);
            let output = self.electric_kw(ctx, station);
            let wires = &mut ctx.land.wires;
            let on: Vec<usize> = (0..wires.hookups.len())
                .filter(|&i| {
                    wires
                        .lines
                        .iter()
                        .any(|l| l.id == wires.hookups[i].line && l.station == station)
                })
                .collect();
            let mine: Vec<Hookup> = on.iter().map(|&i| wires.hookups[i].clone()).collect();
            let (causes, sent) =
                civ_land::wire::dispatch(&wires.lines, &mine, station, output, &params);
            for (&i, cause) in on.iter().zip(causes) {
                let h = &mut wires.hookups[i];
                h.record(cause);
                h.cause = cause;
                if cause == Cause::Served {
                    h.lit_until = until;
                }
            }
            self.wire_pay(ctx, station, until);
            self.burn_for_current(ctx, station, sent);
        }
    }

    /// Station `station` sent `sent_kw` of current down its tree, losses included, for the
    /// evening's hours: its engines that burn a named fuel give what its installations' shafts do
    /// not, burning that fuel from its stores (M11b, first part).
    fn burn_for_current(&mut self, ctx: &Ctx, station: PermanentId, sent_kw: f64) {
        if sent_kw <= 0.0 {
            return;
        }
        let goods = &ctx.catalog.goods;
        let water = super::power_kw_of(ctx, station);
        let hours = super::gas::GAS_EVENING_H;
        let Some(x) = self.household_mut(station) else {
            return;
        };
        let shaft = make::dynamo_shaft_kw(goods, &x.stores, sent_kw);
        let from_engines = (shaft - water).max(0.0) * hours;
        if from_engines <= 0.0 {
            return;
        }
        let (_, burned) = super::engine_work(goods, &x.stores, from_engines, hours);
        for (f, kg) in burned {
            let held = x.stores.get(f).copied().unwrap_or(0.0);
            let kg = kg.min(held.max(0.0));
            if kg > 0.0 {
                x.stores[f] = held - kg;
                x.flows.add(Flow::Used, f, kg);
            }
        }
    }

    /// Homes along station `station`'s lines weigh hooking up (M11a, fifth part; ADR-0036 §6): a
    /// household whose home lies on a whole, unbroken line or beside it, not the station's and
    /// not yet hooked up, whose finished home would have light by wire
    /// ([`Population::wants_wire`]) and that holds an evening's price in food, hooks its home up
    /// asking [`HOME_LIGHT_KW`]. One cut off is restored once it holds an evening's price again.
    fn wire_connect(&mut self, ctx: &mut Ctx, station: PermanentId) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let evening = HOME_LIGHT_KW * super::gas::GAS_EVENING_H * meter_kcal_kwh(ctx);
        let w = i64::from(ctx.map.width.max(1));
        let lines: Vec<(PermanentId, Vec<u32>)> = ctx
            .land
            .wires
            .lines
            .iter()
            .filter(|l| l.station == station && l.whole() && !l.broken)
            .map(|l| (l.id, l.cells.clone()))
            .collect();
        let holds = |x: &Household| {
            let stores = stores_now(x, now, params, goods);
            crate::person::stock_kcal(&stores, goods) >= evening
        };
        // Cut off, and able to pay again.
        let restored: Vec<usize> = ctx
            .land
            .wires
            .hookups
            .iter()
            .enumerate()
            .filter(|(_, h)| h.cut_off && lines.iter().any(|l| l.0 == h.line))
            .filter(|(_, h)| self.household(h.household).is_some_and(holds))
            .map(|(i, _)| i)
            .collect();
        for i in restored {
            ctx.land.wires.hookups[i].cut_off = false;
        }
        for (line, cells) in lines {
            let near = |cell: usize| {
                let (x, y) = (cell as i64 % w, cell as i64 / w);
                cells.iter().any(|&c| {
                    let (cx, cy) = (i64::from(c) % w, i64::from(c) / w);
                    (cx - x).abs() <= 1 && (cy - y).abs() <= 1
                })
            };
            let mut along: Vec<PermanentId> = self
                .households
                .iter()
                .filter(|(_, x)| x.id != station && near(cell_of(ctx.map, x.home)))
                .map(|(_, x)| x.id)
                .collect();
            along.sort_unstable();
            for h in along {
                if ctx.land.wires.hookups.iter().any(|k| k.household == h) {
                    continue;
                }
                let Some(x) = self.household(h) else {
                    continue;
                };
                if !holds(x) || !self.wants_wire(ctx, x) {
                    continue;
                }
                let Some(home) = Self::finished_home(ctx, h) else {
                    continue;
                };
                self.begin_hookup(ctx, home, h, line, HOME_LIGHT_KW);
            }
        }
    }

    /// Each household served this evening (lit until `until`) on station `station`'s lines,
    /// other than the station's own, pays it by the meter (M11a, fifth part; ADR-0036 §6): what
    /// its hookup asked over the evening's hours at the meter's price, in food, by the `Wire`
    /// channel. One that cannot pay it all pays what it holds and is cut off.
    fn wire_pay(&mut self, ctx: &mut Ctx, station: PermanentId, until: i64) {
        let price = meter_kcal_kwh(ctx);
        let served: Vec<(usize, PermanentId, f64)> = {
            let wires = &ctx.land.wires;
            wires
                .hookups
                .iter()
                .enumerate()
                .filter(|(_, h)| {
                    h.household != station
                        && h.cause == Cause::Served
                        && h.lit_until == until
                        && wires
                            .lines
                            .iter()
                            .any(|l| l.id == h.line && l.station == station)
                })
                .map(|(i, h)| (i, h.household, h.ask_kw * super::gas::GAS_EVENING_H * price))
                .collect()
        };
        for (i, household, owed) in served {
            let paid = self.pay_food_by(ctx, household, station, owed, Channel::Wire);
            if paid + 1e-6 < owed {
                ctx.land.wires.hookups[i].cut_off = true;
            }
        }
    }

    /// The wire's evening now: for tests.
    #[doc(hidden)]
    pub fn wire_evening_for_tests(&mut self, ctx: &mut Ctx) {
        self.wire_evening(ctx);
    }

    /// `hours` of work by someone of `household` on line `line`: for tests.
    #[doc(hidden)]
    pub fn line_work_for_tests(
        &mut self,
        ctx: &mut Ctx,
        household: PermanentId,
        line: PermanentId,
        hours: f64,
    ) {
        self.line_work(ctx, household, line, hours);
    }
}
