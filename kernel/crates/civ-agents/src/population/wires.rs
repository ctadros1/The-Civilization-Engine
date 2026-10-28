//! Lines of current begun and laid (M11a, third part, fourth step; ADR-0036 §3): a line is routed
//! along ways whose keepers give leave, as a gas main is, marked out at its length with nothing
//! laid, and strung by its station's household a metre at a time from the conductor it holds. It
//! carries current only once whole. A building hooked up to a line is served each evening as far
//! as its station's current and its lines carry it (ADR-0036 §4), lit as a gas burner lights it.
//! A household whose dynamo turns beside its own installation hooks its home up to it (the fourth
//! part); lines to other homes, and who weighs them, come after.

use civ_core::PermanentId;
use civ_land::ways::Keeper;
use civ_land::wire::{Cause, Hookup, Line};

use super::*;

/// The kilowatts a home hooked up to the wire asks of an evening (a design prior: a few
/// incandescent lamps of the 1880s at some 60 W each; research 07-13 §5.2 E04).
pub const HOME_LIGHT_KW: f64 = 0.2;

impl Population {
    /// Each midnight, a household that holds a whole dynamo beside an installation of its own that
    /// is ready, and whose finished home is hooked up to nothing, hooks it up on a line from its
    /// own station of no length (M11a, fourth part; ADR-0036 §6: its own use first). Nothing where
    /// the land lays no line or no installation stands.
    pub(super) fn wire_reviews(&mut self, ctx: &mut Ctx) {
        if ctx.land_params.wire.is_none() || ctx.land.installations.is_empty() {
            return;
        }
        let dynamos: Vec<usize> = ctx
            .catalog
            .goods
            .iter()
            .enumerate()
            .filter(|(_, d)| {
                d.power
                    .is_some_and(|p| p.source == crate::params::PowerSource::Shaft)
            })
            .map(|(g, _)| g)
            .collect();
        if dynamos.is_empty() {
            return;
        }
        let mut due: Vec<(PermanentId, PermanentId, u32)> = Vec::new();
        for (_, x) in self.households.iter() {
            if !dynamos
                .iter()
                .any(|&g| x.stores.get(g).copied().unwrap_or(0.0) >= 1.0 - 1e-9)
            {
                continue;
            }
            let turning = ctx
                .land
                .installations
                .iter()
                .enumerate()
                .any(|(i, s)| s.owner == x.id && ctx.land.installation_ready(i));
            if !turning {
                continue;
            }
            let Some(home) = ctx.land.buildings.iter().find(|b| {
                b.household == x.id
                    && b.finished()
                    && b.standing()
                    && ctx.catalog.is_dwelling(&b.spec.program)
            }) else {
                continue;
            };
            if ctx.land.wires.hookups.iter().any(|h| h.building == home.id) {
                continue;
            }
            due.push((x.id, home.id, cell_of(ctx.map, x.home) as u32));
        }
        for (household, building, cell) in due {
            if let Some(line) = self.begin_line(ctx, household, cell, cell, None) {
                self.begin_hookup(ctx, building, household, line, HOME_LIGHT_KW);
            }
        }
    }

    /// The midnight review of hooking homes up: for tests.
    #[doc(hidden)]
    pub fn wire_reviews_for_tests(&mut self, ctx: &mut Ctx) {
        self.wire_reviews(ctx);
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
        });
        Some(id)
    }

    /// `hours` of work by someone of `household` on its station's line `line`: a metre strung
    /// each `lay_h_m` hours, each taking its conductor from the household's store, no more than
    /// the conductor held allows or the line has left.
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
        let Some(l) = ctx
            .land
            .wires
            .lines
            .iter_mut()
            .find(|l| l.id == line && l.station == household && !l.whole())
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
        });
    }

    /// The wire's evening (M11a; ADR-0036 §4): for each station with lines, in the order its
    /// first line was made, its dynamos' current from its shafts today
    /// ([`Population::electric_kw`]) is dispatched down its tree ([`civ_land::wire::dispatch`]);
    /// each hookup on it keeps its cause, and one served is lit for [`super::gas::GAS_EVENING_H`]
    /// from now, as a gas burner is. Nothing where no line is laid or the land lays none.
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
            let (causes, _) =
                civ_land::wire::dispatch(&wires.lines, &mine, station, output, &params);
            for (&i, cause) in on.iter().zip(causes) {
                let h = &mut wires.hookups[i];
                h.cause = cause;
                if cause == Cause::Served {
                    h.lit_until = until;
                }
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
