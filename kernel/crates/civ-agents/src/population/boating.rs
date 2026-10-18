//! Boats people make and take (M8b slice CO; ADR-0029 §2, §3, §5): a household that found, in
//! planning an errand, that a boat would save it more walking over the boat's life than making
//! it takes, and could make one, begins one on the bank nearest home where the water carries it,
//! and its people work on it there by choice until its labour is done; paddling the household's
//! boat from the bank where it is drawn up to another by the quickest way by water today, the
//! boat drawn up where it lands; and unloading a load in transit into the household's store once
//! the boat is home. A trip afloat wears no ground and passes no crossing or gate.

use super::*;
use crate::decide::BoatFacts;
use civ_land::boats::Boat;

/// A boat a household could make: its vehicle (catalog index) and the bank cell it would be
/// drawn up at.
pub(super) struct BoatItCouldMake {
    pub vehicle: usize,
    pub bank: u32,
}

impl Population {
    /// Sets out by boat from the bank `h` stands at to bank cell `to` ([`Step::Paddle`]): false
    /// where no boat of the household is drawn up here, or no way by water is open today.
    pub(super) fn start_paddle(&mut self, ctx: &mut Ctx, h: Handle<Person>, to: u32) -> bool {
        let (now, params) = (ctx.now, ctx.params);
        let Some(p) = self.people.get(h) else {
            return false;
        };
        let (household, pos) = (p.household, p.pos);
        let from = cell_of(ctx.map, pos) as u32;
        let Some(hull) = ctx
            .land
            .boats
            .list
            .iter()
            .find(|b| b.owner == household && b.at == from && b.made())
            .and_then(|b| ctx.catalog.vehicles.get(usize::from(b.vehicle)))
            .map(crate::params::VehicleDef::hull)
        else {
            return false;
        };
        let rivers = &ctx.land.rivers;
        let (board, land) = (rivers.boarding(ctx.map, from), rivers.boarding(ctx.map, to));
        let Some(route) = rivers.route(
            &ctx.land.floodplain.reaches,
            &ctx.land.reach_discharges,
            hull,
            &board,
            &land,
        ) else {
            return false;
        };
        // Pushing off and drawing up at the paddlers' speed through still water.
        let apart = |a: (f32, f32), b: (f32, f32)| f64::from((a.0 - b.0).hypot(a.1 - b.1));
        let mut points = vec![pos];
        let mut minutes = vec![0.0f32];
        let first = rivers.centre(route.cells[0]);
        let push_off_s = apart(pos, first) / hull.paddle_m_s;
        for (&c, s) in route.cells.iter().zip(&route.at_s) {
            points.push(rivers.centre(c));
            minutes.push(((push_off_s + s) / 60.0) as f32);
        }
        let ashore = rivers.centre(to);
        let last = *points.last().unwrap_or(&pos);
        let total_s = push_off_s + route.seconds + apart(last, ashore) / hull.paddle_m_s;
        points.push(ashore);
        minutes.push((total_s / 60.0) as f32);
        let duration = (total_s / 60.0).ceil().max(1.0);
        let Some(p) = self.people.get_mut(h) else {
            return false;
        };
        self.next_trip += 1;
        let trip = Trip {
            id: self.next_trip,
            rev: 0,
            depart: now,
            points,
            minutes,
            ahead: None,
            afloat: true,
        };
        settle(p, now, params);
        p.burn_kcal_min = burn_rate(p, now, params, params.energy.walk_par);
        p.asleep = false;
        p.company = 0.0;
        p.trip = Some(trip);
        p.act.step_started = now;
        p.act.step_ends = now.plus_minutes(duration as i64);
        let (id, version, ends) = (p.id, p.act.version, p.act.step_ends);
        ctx.schedule_step(ends, id, version);
        true
    }

    /// No way by water from here today: the rest of what `h` meant to do is the walk home; the
    /// boat stays where it is drawn up.
    pub(super) fn cannot_paddle(&mut self, h: Handle<Person>) {
        let home = self
            .people
            .get(h)
            .and_then(|p| self.household(p.household))
            .map(|x| x.home);
        if let (Some(p), Some(home)) = (self.people.get_mut(h), home) {
            let at = p.act.step as usize;
            p.act.steps.truncate(at);
            p.act.steps.push(Step::Walk { to: home });
        }
    }

    /// Someone of `household` drew up at bank `to` after paddling from bank `from`: their boat is
    /// drawn up there now.
    pub(super) fn paddled(ctx: &mut Ctx, household: PermanentId, (from, to): (u32, u32)) {
        if let Some(b) = ctx
            .land
            .boats
            .list
            .iter_mut()
            .find(|b| b.owner == household && b.at == from && b.made())
        {
            b.at = to;
            ctx.land.boats.changed();
        }
    }

    /// `who` unloaded the boat and carried its load up ([`Step::Handle`] with `unload`): what
    /// they carried in transit goes into its owner's store.
    pub(super) fn unloaded(&mut self, ctx: &Ctx, who: PermanentId) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let carried: Vec<usize> = (0..self.consignments.len())
            .filter(|&i| self.consignments[i].carrier == who)
            .collect();
        for i in carried.into_iter().rev() {
            let c = &mut self.consignments[i];
            c.settle_stores(now, goods);
            let (id, owner) = (c.id, c.owner);
            let legs: Vec<Leg> = c
                .stores
                .iter()
                .enumerate()
                .filter(|&(_, &kg)| kg > 0.0)
                .map(|(good, &amount)| Leg {
                    from: id,
                    to: owner,
                    good,
                    amount,
                })
                .collect();
            let moved =
                legs.is_empty() || self.transfer(now, params, goods, &legs, Channel::Carried);
            if moved && self.consignments.get(i).is_none_or(|c| c.is_empty()) {
                let c = self.consignments.remove(i);
                // What its counters saw stays in the world's flows.
                self.consigned_flows.absorb(&c.flows);
            }
        }
    }

    /// The boat household `hh`, holding `stores`, could make, if any (M8b slice CO, third part):
    /// while it has none, made or being made, of the first boat vehicle a member knows the
    /// technique of and whose tool it holds, drawn up at the bank nearest home where the water
    /// carries one today, within the walk the work allows. None without work on a boat in the
    /// content.
    pub(super) fn boat_it_could_make(
        &self,
        ctx: &Ctx,
        hh: &Household,
        stores: &[f64],
    ) -> Option<BoatItCouldMake> {
        if ctx.land.boats.of(hh.id).next().is_some() {
            return None;
        }
        let work = ctx
            .catalog
            .activities
            .iter()
            .find(|a| a.behavior == Behavior::Boat)?;
        let (vehicle, def) = ctx.catalog.vehicles.iter().enumerate().find(|(_, d)| {
            d.mode == crate::params::VehicleMode::Boat
                && d.technique
                    .is_none_or(|t| self.household_knows(&hh.members, t))
                && d.tool.is_none_or(|t| {
                    stores.get(t).copied().unwrap_or(0.0) >= crate::decide::MIN_TOOL
                })
        })?;
        let params = ctx.params;
        let pace = params.nav.tobler_ms(0.0) * params.nav.offtrail_factor;
        let within = (f64::from(work.max_walk_minutes) * 60.0 * pace)
            .min(f64::from(super::market::LANDING_WALK_M)) as f32;
        let (rivers, hull) = (&ctx.land.rivers, def.hull());
        let (hyd, q) = (&ctx.land.floodplain.reaches, &ctx.land.reach_discharges);
        let bank =
            rivers.nearest_bank(hh.home, within, |c| rivers.floats(ctx.map, c, hyd, q, hull))?;
        Some(BoatItCouldMake { vehicle, bank })
    }

    /// The households that found, planning an errand, that a boat would save them more walking
    /// over its life than making it takes begin one where they meant to (M8b slice CO, third
    /// part), unless they have one by now.
    pub(super) fn begin_boats(&mut self, ctx: &mut Ctx) {
        let day = ctx.now.day_index();
        for (household, vehicle, bank, saves_h_year) in std::mem::take(&mut self.boat_wishes) {
            if ctx.land.boats.of(household).next().is_some() {
                continue;
            }
            let Some(def) = ctx.catalog.vehicles.get(vehicle) else {
                continue;
            };
            if f64::from(saves_h_year) * def.life_years <= def.labour_h {
                continue;
            }
            let id = ctx.ids.allocate();
            ctx.land.boats.list.push(Boat {
                id,
                vehicle: vehicle as u16,
                owner: household,
                at: bank,
                made: day,
                wear: 0.0,
                labour_h: def.labour_h as f32,
                work_h: 0.0,
                saves_h_year,
                crew: (0, 0),
            });
            ctx.land.boats.changed();
        }
    }

    /// The boat household `x` is making, if its bank is within reach of home (`field_key`'s
    /// walk) and it is light (M8b slice CO, third part): where, the walk there, the work left,
    /// the walking each hour of its making saves the household over its life, and whether as
    /// many as may work on it at once have begun today.
    pub(super) fn boat_facts(
        &self,
        ctx: &Ctx,
        x: &Household,
        field_key: PermanentId,
        dark: bool,
    ) -> Option<BoatFacts> {
        if dark {
            return None;
        }
        let b = ctx.land.boats.of(x.id).find(|b| !b.made())?;
        let def = ctx.catalog.vehicles.get(usize::from(b.vehicle))?;
        let secs = self
            .homes
            .get(&field_key)?
            .reach
            .seconds_to(b.at as usize)?;
        Some(BoatFacts {
            boat: b.id,
            at: ctx.land.rivers.centre(b.at),
            walk_min: f64::from(secs) / 60.0,
            left_h: f64::from(b.labour_h - b.work_h).max(0.0),
            saves_per_hour: f64::from(b.saves_h_year) * def.life_years
                / f64::from(b.labour_h).max(1.0),
            full: b.crew.0 == ctx.now.day_index() && u32::from(b.crew.1) >= def.crew,
        })
    }

    /// Someone sets off to work on boat `boat`: one of its crew today.
    pub(super) fn joined_boat_crew(ctx: &mut Ctx, boat: PermanentId) {
        let day = ctx.now.day_index();
        if let Some(b) = ctx.land.boats.list.iter_mut().find(|b| b.id == boat) {
            if b.crew.0 != day {
                b.crew = (day, 0);
            }
            b.crew.1 = b.crew.1.saturating_add(1);
        }
    }

    /// `hours` of a capable adult's work by `who` (handle and id) of `household` on the boat
    /// `boat` it is making (M8b slice CO, third part): they count toward its labour, and they
    /// practise its vehicle's skill. The trunk is felled where it is worked, so nothing is taken
    /// from the store. Once its labour is done it is made, on the day it is, and the chronicle
    /// says so.
    pub(super) fn boat_work(
        &mut self,
        ctx: &mut Ctx,
        (h, who): (Handle<Person>, PermanentId),
        household: PermanentId,
        boat: PermanentId,
        hours: f64,
    ) {
        let Some(i) = ctx
            .land
            .boats
            .list
            .iter()
            .position(|b| b.id == boat && b.owner == household && !b.made())
        else {
            return;
        };
        let Some(def) = ctx
            .catalog
            .vehicles
            .get(usize::from(ctx.land.boats.list[i].vehicle))
        else {
            return;
        };
        let skill = def
            .skill
            .and_then(|k| ctx.catalog.skills.get(k).map(|s| (k, s)));
        let b = &mut ctx.land.boats.list[i];
        let left = f64::from(b.labour_h - b.work_h).max(0.0);
        let done = hours.clamp(0.0, left);
        b.work_h += done as f32;
        if let (Some((k, s)), Some(p)) = (skill, self.people.get_mut(h)) {
            p.set_skill(k, s.practised(p.skill(k), done));
        }
        if !b.made() {
            return;
        }
        let now = ctx.now;
        b.work_h = b.labour_h;
        b.made = now.day_index();
        let at = ctx.land.rivers.centre(b.at);
        ctx.land.boats.changed();
        let name = def.name.to_lowercase();
        let settlement = self.household(household).and_then(|x| x.settlement);
        let sentence = format!(
            "{}'s household finished a {name} and drew it up on the bank, ready for the water.",
            self.name_of(who)
        );
        self.chronicle_push(
            now,
            ChronicleKind::BoatMade,
            vec![who],
            settlement,
            Some(at),
            0.0,
            sentence,
        );
    }
}
