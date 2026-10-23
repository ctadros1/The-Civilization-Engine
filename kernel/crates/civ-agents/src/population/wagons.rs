//! Wagons on a household's line (M10b slice DV, third part; ADR-0034 §3; the rail brief §1.1–§1.2):
//! a wagon is a record of the boat's shape standing at an end of a line of track. Someone digging
//! at a pit where their household's whole line runs from beside the pit to beside home, with a
//! made wagon of the line's gauge standing at either end, loads it rather than carrying what one
//! person carries: as much as the work digs, up to what one person pushes up the steepest rise
//! homeward ([`civ_land::rail::most_load`]). The loaded push home takes longer than the walk, and
//! that time comes out of the session's work; the wagon is left at the home end.

use super::*;
use civ_land::carting::DOOR_LEG_M;
use civ_land::rail;
use civ_land::ways::{Keeper, RAIL};

/// A haul by wagon from a pit home: the wagon, the line's cell beside home, the most it carries
/// home, and the minutes the loaded push home takes beyond the walk.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Haul {
    pub wagon: PermanentId,
    pub home_end: u32,
    pub load_kg: f64,
    pub extra_min: f64,
}

impl Population {
    /// The haul household `household`, at home at `home` (metres), could make by wagon from a
    /// pit at `pit` home, if any: along a whole line it keeps alone, serving wagons, one end
    /// within [`DOOR_LEG_M`] of the pit and the other of home, with a made wagon of the line's
    /// gauge standing at either end. The first such line, in the order lines were begun.
    pub(super) fn wagon_haul(
        ctx: &Ctx,
        household: PermanentId,
        home: (f32, f32),
        pit: (f32, f32),
    ) -> Option<Haul> {
        if ctx.land.wagons.list.is_empty() {
            return None;
        }
        let near = |cell: u32, at: (f32, f32)| {
            let c = cell_centre(ctx.map, cell as usize);
            f64::from((c.0 - at.0).hypot(c.1 - at.1)) <= DOOR_LEG_M
        };
        let walk = ctx.params.nav.tobler_ms(0.0);
        let cell_m = f64::from(ctx.map.cell_size_m);
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
            // Homeward is first to last when the first end lies by the pit.
            let back = if near(first, pit) && near(last, home) {
                false
            } else if near(last, pit) && near(first, home) {
                true
            } else {
                continue;
            };
            let home_end = if back { first } else { last };
            let Some((wagon, def)) = ctx.land.wagons.made_of(household).find_map(|w| {
                let def = ctx.catalog.vehicles.get(usize::from(w.vehicle))?;
                (def.mode == crate::params::VehicleMode::Rail
                    && (w.at == first || w.at == last)
                    && (def.gauge_m - f64::from(track.gauge_m)).abs() < 1e-3)
                    .then_some((w.id, def))
            }) else {
                continue;
            };
            let cells: Vec<u32> = way
                .segments
                .iter()
                .flat_map(|s| s.cells.iter().copied())
                .collect();
            let runs = rail::runs(&cells, ctx.map.width, cell_m);
            let load_kg = rail::most_load(
                &track.bed,
                &runs,
                back,
                def.tare_kg,
                def.payload_kg,
                def.rail_resistance,
                def.pull_n,
            );
            let Some(push_s) = rail::push_seconds(
                &track.bed,
                &runs,
                &def.rail_wheels(load_kg, walk),
                def.rail_resistance,
                back,
            ) else {
                continue;
            };
            let walk_s = runs.iter().sum::<f64>() / walk.max(1e-6);
            return Some(Haul {
                wagon,
                home_end,
                load_kg,
                extra_min: ((push_s - walk_s) / 60.0).max(0.0),
            });
        }
        None
    }

    /// Someone of household `household`, at home at `home`, their settlement's hearth at
    /// `toward`, digs `minutes` at deposit `deposit` at `eff` of a capable adult's rate: carrying
    /// home what one person carries, or by the household's wagon on its line from the pit
    /// ([`Population::wagon_haul`]) as much as the work digs up to the wagon's load, its loaded
    /// push's extra minutes taken from the work, the wagon then standing at the line's home end.
    /// Returns the kilograms brought.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn dig_and_bring(
        ctx: &mut Ctx,
        household: PermanentId,
        home: (f32, f32),
        toward: (f64, f64),
        deposit: PermanentId,
        minutes: f64,
        eff: f64,
    ) -> f64 {
        let carry = ctx.params.household.carry_kg;
        let haul = ctx
            .land
            .deposits
            .iter()
            .find(|d| d.id == deposit)
            .copied()
            .and_then(|d| {
                let side = ctx.params.digging.pit_side_m;
                super::digging::dig_place(ctx.land, ctx.map, &d, toward, side)
            })
            .and_then(|pit| Self::wagon_haul(ctx, household, home, pit))
            .filter(|h| h.load_kg > carry);
        let Some(h) = haul else {
            return super::digging::dig_at(
                ctx,
                household,
                toward,
                deposit,
                minutes / 60.0 * eff,
                carry,
            );
        };
        let effort_h = (minutes - h.extra_min).max(0.0) / 60.0 * eff;
        let kg = super::digging::dig_at(ctx, household, toward, deposit, effort_h, h.load_kg);
        if kg > carry
            && let Some(w) = ctx.land.wagons.list.iter_mut().find(|w| w.id == h.wagon)
        {
            w.at = h.home_end;
            ctx.land.wagons.changed();
        }
        kg
    }

    /// Someone of household `household` digs `minutes` at deposit `deposit` at a capable adult's
    /// rate, as a session of digging would: for tests.
    #[doc(hidden)]
    pub fn dig_and_bring_for_tests(
        &mut self,
        ctx: &mut Ctx,
        household: PermanentId,
        deposit: PermanentId,
        minutes: f64,
    ) -> f64 {
        let Some(x) = self.household(household) else {
            return 0.0;
        };
        let home = x.home;
        let toward = x
            .settlement
            .and_then(|s| ctx.land.settlements.iter().find(|y| y.id == s))
            .map_or((0.0, 0.0), |y| {
                (f64::from(y.hearth_m.0), f64::from(y.hearth_m.1))
            });
        Self::dig_and_bring(ctx, household, home, toward, deposit, minutes, 1.0)
    }

    /// Gives household `household` a made wagon of vehicle `vehicle` standing at cell `at`: for
    /// tests (wagons are made by choice in a later part).
    #[doc(hidden)]
    pub fn add_wagon_for_tests(
        ctx: &mut Ctx,
        household: PermanentId,
        vehicle: usize,
        at: u32,
    ) -> PermanentId {
        let id = ctx.ids.allocate();
        let day = ctx.now.day_index();
        ctx.land.wagons.list.push(civ_land::boats::Boat {
            id,
            vehicle: vehicle as u16,
            owner: household,
            at,
            made: day,
            wear: 0.0,
            labour_h: 0.0,
            work_h: 0.0,
            saves_h_year: 0.0,
            crew: (0, 0),
        });
        ctx.land.wagons.changed();
        id
    }
}
