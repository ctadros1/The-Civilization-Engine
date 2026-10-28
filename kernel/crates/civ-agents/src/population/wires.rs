//! Lines of current begun and laid (M11a, third part, fourth step; ADR-0036 §3): a line is routed
//! along ways whose keepers give leave, as a gas main is, marked out at its length with nothing
//! laid, and strung by its station's household a metre at a time from the conductor it holds. It
//! carries current only once whole. Nobody begins one in a lived day yet.

use civ_core::PermanentId;
use civ_land::ways::Keeper;
use civ_land::wire::Line;

use super::*;

impl Population {
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
