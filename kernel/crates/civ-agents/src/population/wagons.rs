//! Wagons on a household's line (M10b slice DV, third part; ADR-0034 §3; the rail brief §1.1–§1.2):
//! a wagon is a record of the boat's shape standing at an end of a line of track. Someone digging
//! at a pit where their household's whole line runs from beside the pit to beside home, with a
//! made wagon of the line's gauge standing at either end, loads it rather than carrying what one
//! person carries: as much as the work digs, up to what one person pushes up the steepest rise
//! homeward ([`civ_land::rail::most_load`]). The loaded push home takes longer than the walk, and
//! that time comes out of the session's work; the wagon is left at the home end.
//!
//! The fourth part: lines and wagons by choice. On its day of the year a household where someone
//! knows how to make the wagon weighs a line from the pit it digs most to home: a session there
//! now brings home what one person carries, however long they dig, and with a wagon it would
//! bring what the work digs, so each dig day spares the share of its hours (the session and the
//! walk there and back) that carrying one person's load at a time costs. Where those hours over
//! the wagon's life come to more than laying the line and making the wagon take, its timber
//! reckoned at the household's own cost, it marks out the line and begins a wagon at its home
//! end; a household with a line and no wagon begins one once it holds the timber.

use super::*;
use civ_land::carting::DOOR_LEG_M;
use civ_land::rail;
use civ_land::ways::{Keeper, RAIL};

/// Keyed-randomness purpose of a household's day of the year for weighing a line.
const PURPOSE_LINE_REVIEW: u64 = 0x6c69_6e65_7265_7677; // "linerevw"

/// What a line from a pit home would spare a household, as it reckons it at its review: the pit's
/// deposit, the line planned, the hours a year it would spare, and what laying it and making its
/// wagon take, hours (its timber at the household's own cost).
#[derive(Clone, Debug, PartialEq)]
pub struct LineReckoning {
    pub deposit: PermanentId,
    pub cells: Vec<u32>,
    pub track: civ_land::rail::Track,
    pub saves_h_year: f64,
    pub cost_h: f64,
}

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
                (def.is_wagon()
                    && !ctx.land.runs.aboard(w.id)
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

    /// The wagon the content makes, if any: the first rail vehicle with no engine.
    fn wagon_vehicle(ctx: &Ctx) -> Option<usize> {
        ctx.catalog
            .vehicles
            .iter()
            .position(crate::params::VehicleDef::is_wagon)
    }

    /// Households whose day of the year it is weigh a line from a pit home (M10b slice DV, fourth
    /// part); none where the land lays no track or the content has no wagon.
    pub(super) fn line_reviews(&mut self, ctx: &mut Ctx, day: i64) {
        if ctx.land_params.track.is_none() || ctx.land.deposits.is_empty() {
            return;
        }
        let Some(vehicle) = Self::wagon_vehicle(ctx) else {
            return;
        };
        let due: Vec<PermanentId> = self
            .households
            .iter()
            .map(|(_, x)| x.id)
            .filter(|h| {
                Rng64::from_key(&[ctx.seed, PURPOSE_LINE_REVIEW, h.get()]).next_u64()
                    % DAYS_PER_YEAR as u64
                    == day.rem_euclid(DAYS_PER_YEAR) as u64
            })
            .collect();
        for h in due {
            self.review_line(ctx, h, vehicle);
        }
    }

    /// Household `household` weighs a line now, whatever its day of the year: for tests.
    #[doc(hidden)]
    pub fn review_line_for_tests(&mut self, ctx: &mut Ctx, household: PermanentId) {
        if let Some(vehicle) = Self::wagon_vehicle(ctx) {
            self.review_line(ctx, household, vehicle);
        }
    }

    /// What a line home from the pit household `household` digs most would spare it, as
    /// [`Population::review_line`] reckons it: for tests.
    #[doc(hidden)]
    pub fn line_reckoning_for_tests(
        &self,
        ctx: &Ctx,
        household: PermanentId,
    ) -> Option<LineReckoning> {
        self.line_reckoning(ctx, household, Self::wagon_vehicle(ctx)?)
    }

    /// Household `household` weighs a line of vehicle `vehicle`'s gauge from the pit it digs most
    /// to home, if one of it knows how to make the wagon: with a line it keeps and no wagon, it
    /// begins one; with none, it marks out the line and begins its wagon where the hours the line
    /// would spare over the wagon's life are more than it and the wagon take.
    fn review_line(&mut self, ctx: &mut Ctx, household: PermanentId, vehicle: usize) {
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
        {
            return;
        }
        let mine = |w: &civ_land::ways::Way| {
            w.modes & RAIL != 0
                && w.segments
                    .iter()
                    .all(|s| s.keeper == Keeper::Household(household))
        };
        if let Some(line) = ctx.land.ways.list.iter().find(|w| mine(w)) {
            let home_end = line.ends().map(|e| e.1);
            if let Some(home_end) = home_end
                && ctx.land.wagons.of(household).next().is_none()
            {
                let saves = self
                    .line_reckoning(ctx, household, vehicle)
                    .map_or(0.0, |r| r.saves_h_year);
                self.begin_wagon(ctx, household, vehicle, home_end, saves);
            }
            return;
        }
        let Some(r) = self.line_reckoning(ctx, household, vehicle) else {
            return;
        };
        if r.saves_h_year * def.life_years <= r.cost_h {
            return;
        }
        let Some(params) = ctx.land_params.track else {
            return;
        };
        let home_end = *r.cells.last().expect("a line has cells");
        Self::lay_line(
            ctx,
            Keeper::Household(household),
            r.cells,
            r.track,
            params.shoulder_m,
        );
        self.begin_wagon(ctx, household, vehicle, home_end, r.saves_h_year);
    }

    /// What a line of vehicle `vehicle`'s gauge from the pit household `household` digs most to
    /// its home would spare it a year and take, if there is such a pit and line: the dig days a
    /// year it holds there, each spared the share of the session and its walks that carrying one
    /// person's load costs, at what a wagon would bring (what a session digs, no more than the
    /// line's steepest rise homeward lets one person push), against the line's and the wagon's
    /// labour and timber at the household's own cost.
    fn line_reckoning(
        &self,
        ctx: &Ctx,
        household: PermanentId,
        vehicle: usize,
    ) -> Option<LineReckoning> {
        let x = self.household(household)?;
        let def = ctx.catalog.vehicles.get(vehicle)?;
        let params = ctx.land_params.track?;
        let day = ctx.now.day_index();
        let half = ctx.params.places.use_half_life_days;
        // The deposit it digs most, by its dig days a year.
        let (d, days_year) = ctx
            .land
            .deposits
            .iter()
            .filter_map(|d| {
                let u = self
                    .uses
                    .held(household, crate::uses::Place::Deposit(d.id), day, half)?;
                Some((*d, crate::uses::per_year(f64::from(u.days), half)))
            })
            .filter(|(_, n)| *n > 0.0)
            .max_by(|a, b| a.1.total_cmp(&b.1).then(b.0.id.cmp(&a.0.id)))?;
        let good = usize::from(d.body.good);
        let act = ctx
            .catalog
            .activities
            .iter()
            .find(|a| a.behavior == Behavior::Dig && a.digs == Some(good))?;
        let toward = x
            .settlement
            .and_then(|s| ctx.land.settlements.iter().find(|y| y.id == s))
            .map_or((f64::from(x.home.0), f64::from(x.home.1)), |y| {
                (f64::from(y.hearth_m.0), f64::from(y.hearth_m.1))
            });
        let side = ctx.params.digging.pit_side_m;
        let pit = super::digging::dig_place(ctx.land, ctx.map, &d, toward, side)?;
        let rates = super::digging::rates(ctx.land_params, &d.body, ctx.params.digging.h_per_m3);
        let kg_h = super::digging::dig_kg_per_hour(&d.body, rates);
        let session_h = f64::from(act.max_minutes) / 60.0;
        let pace = ctx.params.nav.tobler_ms(0.0) * ctx.params.nav.offtrail_factor;
        let walk_h =
            f64::from((pit.0 - x.home.0).hypot(pit.1 - x.home.1)) / pace.max(1e-6) / 3600.0;
        // The steepest grade a full wagon is pushed up, and no gentler than a pusher holds back.
        let ruling = (def.pull_n / ((def.tare_kg + def.payload_kg) * civ_land::carting::G)
            - def.rail_resistance)
            .clamp(0.0005, def.brake_grade.max(0.0005));
        let (pit_cell, home_cell) = (
            cell_of(ctx.map, pit) as u32,
            cell_of(ctx.map, x.home) as u32,
        );
        let (cells, track) = Self::plan_line(ctx, pit_cell, home_cell, def.gauge_m as f32, ruling)?;
        let runs = rail::runs(&cells, ctx.map.width, f64::from(ctx.map.cell_size_m));
        let load = rail::most_load(
            &track.bed,
            &runs,
            false,
            def.tare_kg,
            def.payload_kg,
            def.rail_resistance,
            def.pull_n,
        );
        let carry = ctx.params.household.carry_kg;
        let by_wagon = load.min(kg_h * session_h);
        if by_wagon <= carry {
            return None;
        }
        let saves_h_year = days_year * (session_h + 2.0 * walk_h) * (1.0 - carry / by_wagon);
        let timber_h = self
            .own_costs_of(ctx, x)
            .get(params.good)
            .copied()
            .flatten()?;
        let cost_h = f64::from(track.labour_h)
            + def.labour_h
            + (f64::from(track.good_kg) + def.material_kg) * timber_h;
        Some(LineReckoning {
            deposit: d.id,
            cells,
            track,
            saves_h_year,
            cost_h,
        })
    }

    /// Household `household` begins a wagon of vehicle `vehicle` at cell `at`, the home end of its
    /// line, if it holds the wagon's timber: taken from its store as it is begun, as a cart's is,
    /// the wagon then made by the household's work on it.
    fn begin_wagon(
        &mut self,
        ctx: &mut Ctx,
        household: PermanentId,
        vehicle: usize,
        at: u32,
        saves_h_year: f64,
    ) {
        let Some(def) = ctx.catalog.vehicles.get(vehicle) else {
            return;
        };
        let (g, kg, labour_h) = (def.good, def.material_kg, def.labour_h);
        let Some(x) = self.household_mut(household) else {
            return;
        };
        if x.stores.get(g).copied().unwrap_or(0.0) < kg {
            return;
        }
        x.stores[g] -= kg;
        x.flows.add(Flow::Used, g, kg);
        let id = ctx.ids.allocate();
        let day = ctx.now.day_index();
        ctx.land.wagons.list.push(civ_land::boats::Boat {
            id,
            vehicle: vehicle as u16,
            owner: household,
            at,
            made: day,
            wear: 0.0,
            labour_h: labour_h as f32,
            work_h: 0.0,
            saves_h_year: saves_h_year as f32,
            crew: (0, 0),
        });
        ctx.land.wagons.changed();
    }

    /// Someone of household `household`, at home at `home`, their settlement's hearth at
    /// `toward`, digs `minutes` at deposit `deposit` at `eff` of a capable adult's rate: carrying
    /// home what one person carries, or by the household's wagon on its line from the pit
    /// ([`Population::wagon_haul`]) as much as the work digs up to the wagon's load, its loaded
    /// push's extra minutes taken from the work, the wagon then standing at the line's home end.
    /// Where the household's engine works the line from the pit (M10b slice DW, third part), what
    /// the work digs goes onto its wagons there instead, up to the room left on the train
    /// (`consignments` being what is aboard). Returns the kilograms brought, and the wagon loaded
    /// and the kilograms put on it.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn dig_and_bring(
        ctx: &mut Ctx,
        consignments: &[crate::consignments::Consignment],
        household: PermanentId,
        home: (f32, f32),
        toward: (f64, f64),
        deposit: PermanentId,
        minutes: f64,
        eff: f64,
    ) -> (f64, Option<(PermanentId, f64)>) {
        let carry = ctx.params.household.carry_kg;
        let pit = ctx
            .land
            .deposits
            .iter()
            .find(|d| d.id == deposit)
            .copied()
            .and_then(|d| {
                let side = ctx.params.digging.pit_side_m;
                super::digging::dig_place(ctx.land, ctx.map, &d, toward, side)
            });
        if let Some((wagon, room)) =
            pit.and_then(|pit| Self::engine_room(ctx, consignments, household, home, pit))
        {
            let effort_h = minutes / 60.0 * eff;
            let kg = super::digging::dig_at(ctx, household, toward, deposit, effort_h, room);
            return (0.0, Some((wagon, kg)));
        }
        let haul = pit
            .and_then(|pit| Self::wagon_haul(ctx, household, home, pit))
            .filter(|h| h.load_kg > carry);
        let Some(h) = haul else {
            let kg = super::digging::dig_at(
                ctx,
                household,
                toward,
                deposit,
                minutes / 60.0 * eff,
                carry,
            );
            return (kg, None);
        };
        let effort_h = (minutes - h.extra_min).max(0.0) / 60.0 * eff;
        let kg = super::digging::dig_at(ctx, household, toward, deposit, effort_h, h.load_kg);
        if kg > carry
            && let Some(w) = ctx.land.wagons.list.iter_mut().find(|w| w.id == h.wagon)
        {
            w.at = h.home_end;
            ctx.land.wagons.changed();
        }
        (kg, None)
    }

    /// Someone of household `household` digs `minutes` at deposit `deposit` at a capable adult's
    /// rate, as a session of digging would, what they dig onto a train loaded as the session
    /// would load it: for tests. Returns the kilograms brought home by hand or by wagon.
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
        let (kg, loaded) = Self::dig_and_bring(
            ctx,
            &self.consignments,
            household,
            home,
            toward,
            deposit,
            minutes,
            1.0,
        );
        let good = ctx
            .land
            .deposits
            .iter()
            .find(|d| d.id == deposit)
            .map(|d| usize::from(d.body.good));
        if let (Some((wagon, kg)), Some(good)) = (loaded, good) {
            self.load_wagon(ctx, household, wagon, good, kg);
        }
        kg
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
