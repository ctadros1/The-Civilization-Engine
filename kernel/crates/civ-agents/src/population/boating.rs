//! Boats people make and take (M8b slice CO; ADR-0029 §2, §3, §5): a household that found, in
//! planning an errand, that a boat would save it more walking over the boat's life than making
//! it takes, and could make one, begins one on the bank nearest home where the water carries it,
//! and its people work on it there by choice until its labour is done; paddling the household's
//! boat from the bank where it is drawn up to another by the quickest way by water today, the
//! boat drawn up where it lands; pulling the household's cart from its door to another and back
//! by the quickest way for its wheels with what it carries (M8b slice CP); unloading a load in
//! transit into the household's store once the boat or cart is home; and fetching home a boat or
//! cart of the household's that stands away, one an heir took where it stood in another
//! settlement among them (backlog D, ninth item). A trip afloat wears no ground and passes no
//! crossing or gate; a cart's wears the ground as a walk does.

use super::*;
use crate::decide::BoatFacts;
use crate::history::TakenUp;
use crate::traffic;
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
        self.count_trip(ctx, h, crate::travel::Mode::Boat, duration as f32);
        true
    }

    /// Sets out pulling the household's cart from where `h` stands to the door at `to`
    /// ([`Step::Haul`], M8b slice CP): false where no cart of the household stands here, or no way
    /// for its wheels, with what `h` carries in it, is open today.
    pub(super) fn start_haul(&mut self, ctx: &mut Ctx, h: Handle<Person>, to: (f32, f32)) -> bool {
        let (now, params) = (ctx.now, ctx.params);
        let Some(p) = self.people.get(h) else {
            return false;
        };
        let (household, pos, who) = (p.household, p.pos, p.id);
        let here = cell_of(ctx.map, pos) as u32;
        let Some(v) = ctx
            .land
            .carts
            .list
            .iter()
            .find(|c| c.owner == household && c.at == here && c.made())
            .and_then(|c| ctx.catalog.vehicles.get(usize::from(c.vehicle)))
        else {
            return false;
        };
        // What it carries: the load in transit its puller has in it.
        let load: f64 = self
            .consignments
            .iter()
            .filter(|c| c.carrier == who)
            .map(|c| c.stores.iter().sum::<f64>())
            .sum();
        let wheels = v.wheels(load, params.nav.tobler_ms(0.0));
        let Some((route, _)) = ctx.land.cart_way(ctx.map, &wheels, pos, to) else {
            return false;
        };
        let Some(leg_v) = wheels.speed(wheels.trail_c, 0.0) else {
            return false;
        };
        let centre = |c: u32| cell_centre(ctx.map, c as usize);
        let apart = |a: (f32, f32), b: (f32, f32)| f64::from((a.0 - b.0).hypot(a.1 - b.1));
        // Pushed from the door to the ground it goes on, along it, and on to the door.
        let mut points = vec![pos];
        let mut minutes = vec![0.0f32];
        let first = centre(route.cells[0]);
        let off_s = apart(pos, first) / leg_v;
        for (&c, s) in route.cells.iter().zip(&route.at_s) {
            points.push(centre(c));
            minutes.push(((off_s + s) / 60.0) as f32);
        }
        let last = *points.last().unwrap_or(&pos);
        let mut total_s = off_s + route.seconds + apart(last, to) / leg_v;
        points.push(to);
        minutes.push((total_s / 60.0) as f32);
        // Carts meet by clearance (M8b slice CP, fourth part).
        let door = self
            .household(household)
            .map(|x| cell_of(ctx.map, x.home) as u32);
        let Some(waited_s) = self.wait_at_narrows(
            ctx,
            h,
            &wheels,
            &route.cells,
            &mut points,
            &mut minutes,
            door == Some(here),
        ) else {
            return false;
        };
        total_s += waited_s;
        // What the climbs on it cost beyond the level is remembered where each begins (M8b slice
        // CQ, first part).
        let (day, half) = (now.day_index(), params.places.use_half_life_days);
        for (cell, s) in ctx.land.cart_climbs(ctx.map, &wheels, &route.cells) {
            if s >= traffic::LEAST_LOSS_S {
                self.traffic.lost(
                    household,
                    cell,
                    traffic::Cause::Grade,
                    (s / 3600.0) as f32,
                    day,
                    half,
                );
            }
        }
        // And what its worn trail cost beyond a way laid there, cell by cell, where the haul's
        // whole of it is worth remembering (M8b slice CQ, fourth part).
        let roughs = ctx.land.cart_roughs(ctx.map, &wheels, &route.cells);
        if roughs.iter().map(|r| r.1).sum::<f64>() >= traffic::LEAST_LOSS_S {
            for (cell, s) in roughs {
                self.traffic.lost(
                    household,
                    cell,
                    traffic::Cause::Rough,
                    (s / 3600.0) as f32,
                    day,
                    half,
                );
            }
        }
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
            afloat: false,
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
        self.count_trip(ctx, h, crate::travel::Mode::Cart, duration as f32);
        true
    }

    /// Carts meet by clearance (M8b slice CP, fourth part; ADR-0029 §4). Along `cells`, the cart
    /// route `h` is to pull its cart by (`points` and `minutes` the haul as planned free-flow, its
    /// door legs first and last), each run of cells too narrow for two carts is a single-cart
    /// stretch: a cart already pulled through it the other way, there when this one would be, is
    /// waited for at its near end until it is clear ([`traffic::clearance_wait_s`]), the wait put
    /// into the haul as a stop there. The household remembers each wait of a minute or more at that
    /// stretch, and a long one is told. The seconds waited in all; or none, the cart not taken, where
    /// it leaves its own door (`from_door`) and would wait longer than [`traffic::TURN_BACK_MIN`]
    /// at one stretch: its puller turns back.
    #[allow(clippy::too_many_arguments)]
    fn wait_at_narrows(
        &mut self,
        ctx: &mut Ctx,
        h: Handle<Person>,
        wheels: &civ_land::carting::Wheels,
        cells: &[u32],
        points: &mut Vec<(f32, f32)>,
        minutes: &mut Vec<f32>,
        from_door: bool,
    ) -> Option<f64> {
        let narrow: Vec<bool> = cells
            .iter()
            .map(|&c| {
                ctx.land
                    .cart_going(ctx.map, c, wheels)
                    .is_some_and(|(_, room)| wheels.single_file(room))
            })
            .collect();
        if !narrow.contains(&true) {
            return Some(0.0);
        }
        // The other hauls under way: each the cells of its points and their times, seconds.
        let hauls: Vec<(Vec<u32>, Vec<f64>)> = self
            .people
            .iter()
            .filter(|&(k, _)| k != h)
            .filter(|(_, p)| {
                matches!(
                    p.act.steps.get(p.act.step as usize),
                    Some(Step::Haul { .. })
                )
            })
            .filter_map(|(_, p)| p.trip.as_ref())
            .map(|t| {
                let at = t.depart.minutes() as f64 * 60.0;
                (
                    t.points
                        .iter()
                        .map(|&q| cell_of(ctx.map, q) as u32)
                        .collect(),
                    t.minutes
                        .iter()
                        .map(|&m| at + f64::from(m) * 60.0)
                        .collect(),
                )
            })
            .collect();
        let p = self.people.get(h)?;
        let (who, household) = (p.id, p.household);
        let now_s = ctx.now.minutes() as f64 * 60.0;
        let day = ctx.now.day_index();
        let half = ctx.params.places.use_half_life_days;
        // Each stretch: the point it is waited for at (the one before its first cell), the wait.
        let mut stops: Vec<(usize, f64, u32, u32, u32)> = Vec::new();
        let mut shift = 0.0;
        let mut i = 0;
        while i < cells.len() {
            if !narrow[i] {
                i += 1;
                continue;
            }
            let mut j = i;
            while j + 1 < cells.len() && narrow[j + 1] {
                j += 1;
            }
            let run = &cells[i..=j];
            // Points: 0 the start, 1..=n the cells, n + 1 the end.
            let enter_s = now_s + f64::from(minutes[i + 1]) * 60.0 + shift;
            let leave_s = now_s + f64::from(minutes[j + 2]) * 60.0 + shift;
            let after = cell_of(ctx.map, points[j + 2]) as u32;
            let place = |c: u32| run.iter().position(|&x| x == c);
            let others: Vec<traffic::OnStretch> = hauls
                .iter()
                .filter_map(|(pc, pt)| {
                    let first = pc.iter().position(|&c| place(c).is_some())?;
                    let last = pc.iter().rposition(|&c| place(c).is_some())?;
                    let (a, b) = (place(pc[first])?, place(pc[last])?);
                    let same_way = if a != b {
                        a < b
                    } else {
                        first == 0 || pc[first - 1] != after
                    };
                    Some(traffic::OnStretch {
                        enter_s: pt[first],
                        leave_s: pt[(last + 1).min(pt.len() - 1)],
                        same_way,
                    })
                })
                .collect();
            let wait = traffic::clearance_wait_s(enter_s, leave_s, &others);
            if wait >= 60.0 {
                if from_door && wait > traffic::TURN_BACK_MIN * 60.0 {
                    // Turned back at its own door: the wait it would have had is remembered,
                    // and as the way's loss there.
                    self.traffic.waited(
                        household,
                        traffic::Facility::Narrows(run[0]),
                        (wait / 60.0) as f32,
                        day,
                        half,
                    );
                    self.traffic.lost(
                        household,
                        run[run.len() / 2],
                        traffic::Cause::Width,
                        (wait / 3600.0) as f32,
                        day,
                        half,
                    );
                    return None;
                }
                let met = others
                    .iter()
                    .filter(|o| !o.same_way && o.leave_s > enter_s)
                    .count() as u32;
                stops.push((i, wait, run[0], run[run.len() / 2], met));
                shift += wait;
            }
            i = j + 1;
        }
        let settlement = self.household(household).and_then(|x| x.settlement);
        for (added, &(i, wait, cell, middle, met)) in stops.iter().enumerate() {
            let k = i + 1 + added;
            let (at, at_min) = (points[k - 1], minutes[k - 1]);
            let waited = (wait / 60.0) as f32;
            points.insert(k, at);
            minutes.insert(k, at_min + waited);
            for m in &mut minutes[k + 1..] {
                *m += waited;
            }
            let facility = traffic::Facility::Narrows(cell);
            self.traffic.waited(household, facility, waited, day, half);
            self.count_wait(ctx, household, who, waited, false);
            self.traffic.lost(
                household,
                middle,
                traffic::Cause::Width,
                waited / 60.0,
                day,
                half,
            );
            if waited >= traffic::LONG_WAIT_MIN {
                self.traffic.long_wait(traffic::LongWait {
                    id: 0,
                    person: who,
                    household,
                    settlement,
                    facility,
                    at,
                    time: ctx.now.plus_minutes(at_min.floor() as i64),
                    minutes: waited,
                    ahead: met,
                });
            }
        }
        Some(shift)
    }

    /// Someone of `household` pulled its cart from cell `from` to cell `to`: it stands there now.
    pub(super) fn hauled(ctx: &mut Ctx, household: PermanentId, (from, to): (u32, u32)) {
        if let Some(c) = ctx
            .land
            .carts
            .list
            .iter_mut()
            .find(|c| c.owner == household && c.at == from && c.made())
        {
            c.at = to;
            ctx.land.carts.changed();
        }
    }

    /// No way by water or for wheels from here today: the rest of what `h` meant to do is the
    /// walk home; the boat or cart stays where it is.
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
        let within = Self::bank_reach_m(ctx, work);
        let (rivers, hull) = (&ctx.land.rivers, def.hull());
        let (hyd, q) = (&ctx.land.floodplain.reaches, &ctx.land.reach_discharges);
        let bank =
            rivers.nearest_bank(hh.home, within, |c| rivers.floats(ctx.map, c, hyd, q, hull))?;
        Some(BoatItCouldMake { vehicle, bank })
    }

    /// How far from home a household draws its boat up, metres: the walk the work on one
    /// (`work`) allows, off the trail, and never beyond a landing's walk.
    fn bank_reach_m(ctx: &Ctx, work: &ActivityDef) -> f32 {
        let params = ctx.params;
        let pace = params.nav.tobler_ms(0.0) * params.nav.offtrail_factor;
        (f64::from(work.max_walk_minutes) * 60.0 * pace)
            .min(f64::from(super::market::LANDING_WALK_M)) as f32
    }

    /// The way boat or cart `v` could be brought today to a home at `home` (backlog D, ninth item):
    /// a boat paddled by the quickest way by water to the bank nearest home where the water
    /// carries it, within where a household draws one up, and walked up from there; a cart, not a
    /// motor cart, pulled empty to the door by the quickest way for its wheels. The bank it would
    /// be drawn up at (none for a cart) and the seconds; none where no way is open today.
    pub(super) fn way_home(ctx: &Ctx, home: (f32, f32), v: &Boat) -> Option<(Option<u32>, f64)> {
        let (map, land) = (ctx.map, &*ctx.land);
        let def = ctx.catalog.vehicles.get(usize::from(v.vehicle))?;
        let walk = ctx.params.nav.tobler_ms(0.0).max(1e-6);
        match def.mode {
            crate::params::VehicleMode::Boat => {
                let within = ctx
                    .catalog
                    .activities
                    .iter()
                    .find(|a| a.behavior == Behavior::Boat)
                    .map_or(super::market::LANDING_WALK_M, |w| {
                        Self::bank_reach_m(ctx, w)
                    });
                let (rivers, hull) = (&land.rivers, def.hull());
                let (hyd, q) = (&land.floodplain.reaches, &land.reach_discharges);
                let bank = rivers.nearest_bank(home, within, |c| {
                    c != v.at && rivers.floats(map, c, hyd, q, hull)
                })?;
                let route = rivers.route(
                    hyd,
                    q,
                    hull,
                    &rivers.boarding(map, v.at),
                    &rivers.boarding(map, bank),
                )?;
                let at = rivers.centre(bank);
                let up_s = f64::from((at.0 - home.0).hypot(at.1 - home.1)) / walk;
                Some((Some(bank), route.seconds + up_s))
            }
            crate::params::VehicleMode::Cart if def.engine.is_none() => {
                let at = cell_centre(map, v.at as usize);
                let (route, legs_s) = land.cart_way(map, &def.wheels(0.0, walk), at, home)?;
                Some((None, route.seconds + legs_s))
            }
            _ => None,
        }
    }

    /// The made boat or cart of household `hh` that `me` could fetch home today, if any (backlog
    /// D, ninth item): a boat drawn up beyond where the household draws one up, paddled to the
    /// bank nearest home where the water carries it, by today's quickest way by water; or a cart
    /// standing away from the door, pulled home empty by today's quickest way for its wheels. A
    /// motor cart burns its fuel and is not pulled. Either is walked to from home (`field_key`'s
    /// walk). None after dark, while another of the household is fetching or the vehicle is in
    /// use on a trip, or where no way is open; of two, the one worth more beyond its trip. Its
    /// worth is the walking it saves over the life it has left, at its household's reckoning,
    /// and never less than the making of another for that life (one given whole was reckoned to
    /// save nothing).
    pub(super) fn vehicle_fetch_facts(
        &self,
        ctx: &Ctx,
        hh: &Household,
        me: PermanentId,
        field_key: PermanentId,
        dark: bool,
    ) -> Option<crate::decide::VehicleFetch> {
        if dark {
            return None;
        }
        let catalog = ctx.catalog;
        catalog
            .activities
            .iter()
            .find(|a| a.behavior == Behavior::Fetch)?;
        let (map, land) = (ctx.map, &*ctx.land);
        let rivers = &land.rivers;
        let door = cell_of(map, hh.home) as u32;
        let within = catalog
            .activities
            .iter()
            .find(|a| a.behavior == Behavior::Boat)
            .map_or(super::market::LANDING_WALK_M, |w| {
                Self::bank_reach_m(ctx, w)
            });
        let apart = |a: (f32, f32), b: (f32, f32)| (a.0 - b.0).hypot(a.1 - b.1);
        let vehicle = |v: &Boat| catalog.vehicles.get(usize::from(v.vehicle));
        let boat = land
            .boats
            .made_of(hh.id)
            .find(|b| apart(hh.home, rivers.centre(b.at)) > within);
        let cart = land.carts.made_of(hh.id).find(|c| {
            c.at != door
                && vehicle(c).is_some_and(|v| {
                    v.mode == crate::params::VehicleMode::Cart && v.engine.is_none()
                })
        });
        if boat.is_none() && cart.is_none() {
            return None;
        }
        // One of a household fetches at a time, and not a boat or cart someone has out.
        let (mut paddling, mut hauling) = (false, false);
        for q in hh
            .members
            .iter()
            .filter(|&&m| m != me)
            .filter_map(|&m| self.person(m))
        {
            if catalog
                .activities
                .get(q.act.def as usize)
                .is_some_and(|a| a.behavior == Behavior::Fetch)
            {
                return None;
            }
            for s in q.act.steps.iter().skip(usize::from(q.act.step)) {
                paddling |= matches!(s, Step::Paddle { .. });
                hauling |= matches!(s, Step::Haul { .. });
            }
        }
        let reach = &self.homes.get(&field_key)?.reach;
        let worth = |v: &Boat, def: &crate::params::VehicleDef| {
            (f64::from(v.saves_h_year) * def.life_years).max(def.labour_h)
                * (1.0 - f64::from(v.wear)).max(0.0)
        };
        let fetch = |v: &Boat, at: (f32, f32)| {
            let (bank, back_s) = Self::way_home(ctx, hh.home, v)?;
            let there_s = f64::from(reach.seconds_to(v.at as usize)?);
            Some(crate::decide::VehicleFetch {
                vehicle: v.id,
                at,
                bank,
                door: hh.home,
                trip_min: (there_s + back_s) / 60.0,
                worth_h: worth(v, vehicle(v)?),
            })
        };
        let by_water = boat
            .filter(|_| !paddling)
            .and_then(|b| fetch(b, rivers.centre(b.at)));
        let by_wheels = cart
            .filter(|_| !hauling)
            .and_then(|c| fetch(c, cell_centre(map, c.at as usize)));
        let net = |v: &crate::decide::VehicleFetch| v.worth_h - v.trip_min / 60.0;
        match (by_water, by_wheels) {
            (Some(a), Some(b)) => Some(if net(&b) > net(&a) { b } else { a }),
            (a, b) => a.or(b),
        }
    }

    /// The walk a fetch allows from home, metres, at an off-trail pace (backlog D, ninth item):
    /// how far off a boat or cart may stand for someone to fetch it home; 0 without fetching in
    /// the content.
    fn fetch_range_m(ctx: &Ctx) -> f64 {
        let params = ctx.params;
        ctx.catalog
            .activities
            .iter()
            .find(|a| a.behavior == Behavior::Fetch)
            .map_or(0.0, |a| {
                f64::from(a.max_walk_minutes)
                    * 60.0
                    * params.nav.tobler_ms(0.0)
                    * params.nav.offtrail_factor
            })
    }

    /// The boat or cart of vehicle `vehicle` that nobody is left to take which household
    /// `household`, finding one would save it `saves_h_year` hours of walking a year, would take
    /// up rather than make its own, if any (backlog D, eleventh item; research 08-09 §1.2: what
    /// nobody holds any more is taken up, as ground is at a regime's review). Nobody is left to
    /// take it when the household it belongs to is no more: none was left to inherit it, or its
    /// heir lives elsewhere and could not bring it, or it was still being made. It must stand
    /// where the household could use it as its own: a boat within where it draws one up, a cart
    /// at its door or, being made, within the walk its work allows; or, made, where a fetch would
    /// bring it home today ([`Population::way_home`]) within a fetch's walk. Its worth is the
    /// walking it saves over the life it has left less the work left on it, at the household's
    /// reckoning, and it is taken where that is no less than making a new one would keep (whose
    /// material it also spares); of several, the one worth most.
    pub(super) fn nobodys_vehicle(
        &self,
        ctx: &Ctx,
        household: PermanentId,
        vehicle: usize,
        saves_h_year: f32,
    ) -> Option<PermanentId> {
        let def = ctx.catalog.vehicles.get(vehicle)?;
        let home = self.household(household)?.home;
        let (map, land) = (ctx.map, &*ctx.land);
        let cart = def.mode == crate::params::VehicleMode::Cart;
        let list = if cart { &land.carts } else { &land.boats };
        let pace = ctx.params.nav.tobler_ms(0.0).max(1e-6);
        let cart_walk_m = ctx
            .catalog
            .activities
            .iter()
            .find(|a| a.behavior == Behavior::Cart)
            .map_or(0.0, |a| f64::from(a.max_walk_minutes) * 60.0 * pace);
        let boat_reach_m = ctx
            .catalog
            .activities
            .iter()
            .find(|a| a.behavior == Behavior::Boat)
            .map_or(0.0, |w| f64::from(Self::bank_reach_m(ctx, w)));
        let range_m = Self::fetch_range_m(ctx);
        let door = cell_of(map, home) as u32;
        let life_h = f64::from(saves_h_year) * def.life_years;
        let new_net = life_h - def.labour_h;
        let mut best: Option<(f64, PermanentId)> = None;
        for v in list
            .list
            .iter()
            .filter(|v| usize::from(v.vehicle) == vehicle && self.household(v.owner).is_none())
        {
            let at = if cart {
                cell_centre(map, v.at as usize)
            } else {
                land.rivers.centre(v.at)
            };
            let off_m = f64::from((at.0 - home.0).hypot(at.1 - home.1));
            // Brought home today, within a fetch's walk.
            let brought = || off_m <= range_m && Self::way_home(ctx, home, v).is_some();
            let usable = if cart {
                // At its door, or brought there: made, or once made where it is worked on.
                v.at == door || ((v.made() || off_m <= cart_walk_m) && brought())
            } else {
                // Where it draws one up, or, made, brought there.
                off_m <= boat_reach_m || (v.made() && brought())
            };
            if !usable {
                continue;
            }
            let net = life_h * (1.0 - f64::from(v.wear)).max(0.0)
                - f64::from(v.labour_h - v.work_h).max(0.0);
            if net + 1e-9 >= new_net
                && best.is_none_or(|(b, id)| net > b || (net == b && v.id < id))
            {
                best = Some((net, v.id));
            }
        }
        best.map(|(_, id)| id)
    }

    /// The cart household `hh`, holding `stores`, could make, if any (M8b slice CP, third part):
    /// while it has none, made or being made, the first cart vehicle a member knows the technique
    /// of, whose tool it holds and whose material it has in store, made at its door. None without
    /// work on a cart in the content.
    pub(super) fn cart_it_could_make(
        &self,
        ctx: &Ctx,
        hh: &Household,
        stores: &[f64],
    ) -> Option<usize> {
        if ctx.land.carts.of(hh.id).next().is_some()
            || !ctx
                .catalog
                .activities
                .iter()
                .any(|a| a.behavior == Behavior::Cart)
        {
            return None;
        }
        let held = |g: usize| stores.get(g).copied().unwrap_or(0.0);
        ctx.catalog.vehicles.iter().position(|d| {
            d.mode == crate::params::VehicleMode::Cart
                && d.technique
                    .is_none_or(|t| self.household_knows(&hh.members, t))
                && d.tool.is_none_or(|t| held(t) >= crate::decide::MIN_TOOL)
                && held(d.good) >= d.material_kg
        })
    }

    /// The households that found, planning an errand, that a boat would save them more walking
    /// over its life than making it takes begin one where they meant to (M8b slice CO, third
    /// part), unless they have one by now, or take up one of that vehicle nobody is left to take
    /// ([`Population::nobodys_vehicle`], backlog D, eleventh item), with what work was done on it.
    pub(super) fn begin_boats(&mut self, ctx: &mut Ctx) {
        let day = ctx.now.day_index();
        for (household, vehicle, bank, saves_h_year) in std::mem::take(&mut self.boat_wishes) {
            let Some(def) = ctx.catalog.vehicles.get(vehicle) else {
                continue;
            };
            let cart = def.mode == crate::params::VehicleMode::Cart;
            let list = if cart {
                &ctx.land.carts
            } else {
                &ctx.land.boats
            };
            if list.of(household).next().is_some()
                || f64::from(saves_h_year) * def.life_years <= def.labour_h
            {
                continue;
            }
            // One nobody is left to take, standing where it could use it, is taken up instead
            // where that keeps it as much (backlog D, eleventh item).
            if let Some(v) = self.nobodys_vehicle(ctx, household, vehicle, saves_h_year) {
                let list = if cart {
                    &mut ctx.land.carts
                } else {
                    &mut ctx.land.boats
                };
                let taken = list.list.iter_mut().find(|b| b.id == v).map(|b| {
                    b.owner = household;
                    b.saves_h_year = saves_h_year;
                    b.clone()
                });
                list.changed();
                if let Some(b) = taken {
                    self.tell_taken_up(ctx, household, &b, cart);
                }
                continue;
            }
            // A cart's timber comes out of the household's store as it is begun (M8b slice CP,
            // third part): boards and an axle shaped at home, not a trunk felled where it is
            // worked.
            if cart {
                let (g, kg) = (def.good, def.material_kg);
                let Some(x) = self.household_mut_by_id(household) else {
                    continue;
                };
                if x.stores.get(g).copied().unwrap_or(0.0) < kg {
                    continue;
                }
                x.stores[g] -= kg;
                x.flows.add(Flow::Used, g, kg);
            }
            let id = ctx.ids.allocate();
            let list = if cart {
                &mut ctx.land.carts
            } else {
                &mut ctx.land.boats
            };
            list.list.push(Boat {
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
            list.changed();
        }
    }

    /// The chronicle tells that household `household` took up `b`, a boat or (`cart`) a cart
    /// nobody was left to take (backlog D, seventeenth item): its elder, what it took up, made or
    /// half made with the work done on it, and where it stands, metres from its home (a boat) or
    /// its door (a cart), placed there. Told once, at the taking up: a household holding one
    /// takes up no other.
    fn tell_taken_up(&mut self, ctx: &Ctx, household: PermanentId, b: &Boat, cart: bool) {
        let Some(def) = ctx.catalog.vehicles.get(usize::from(b.vehicle)) else {
            return;
        };
        let Some(home) = self.household(household).map(|x| x.home) else {
            return;
        };
        let settlement = self.household(household).and_then(|x| x.settlement);
        let elder = self.elder_of(household, ctx.now, ctx.params);
        let at = if cart {
            cell_centre(ctx.map, b.at as usize)
        } else {
            ctx.land.rivers.centre(b.at)
        };
        let made = b.made();
        let off_m = (at.0 - home.0).hypot(at.1 - home.1);
        let place = if cart {
            if b.at == cell_of(ctx.map, home) as u32 {
                "at its door".to_owned()
            } else {
                format!("{off_m:.0} m from its door")
            }
        } else {
            format!("on the bank {off_m:.0} m from its home")
        };
        let state = match (made, cart) {
            (true, false) => format!("made and drawn up {place}"),
            (true, true) => format!("made and standing {place}"),
            (false, _) => format!(
                "half made {place}: {:.0} of its {:.0} hours' work were done",
                b.work_h, b.labour_h
            ),
        };
        let what = match (cart, made) {
            (false, true) => TakenUp::Boat,
            (false, false) => TakenUp::BoatHalfMade,
            (true, true) => TakenUp::Cart,
            (true, false) => TakenUp::CartHalfMade,
        };
        let who = elder.map_or_else(
            || "A household".to_owned(),
            |e| format!("{}'s household", self.name_of(e)),
        );
        self.chronicle_push(
            ctx.now,
            ChronicleKind::VehicleTakenUp,
            elder.into_iter().collect(),
            settlement,
            Some(at),
            f64::from(what as u8),
            format!(
                "{who} took up a {} nobody was left to take, {state}.",
                def.name.to_lowercase()
            ),
        );
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

    /// The cart household `x` is making at its door, if it is light (M8b slice CP, third part):
    /// as [`Population::boat_facts`], the walk the one from home to where it stands.
    pub(super) fn cart_facts(&self, ctx: &Ctx, x: &Household, dark: bool) -> Option<BoatFacts> {
        if dark {
            return None;
        }
        // Or the wagon it is making at the home end of its line (M10b slice DV, fourth part).
        let c = ctx
            .land
            .carts
            .of(x.id)
            .find(|c| !c.made())
            .or_else(|| ctx.land.wagons.of(x.id).find(|c| !c.made()))?;
        let def = ctx.catalog.vehicles.get(usize::from(c.vehicle))?;
        let at = cell_centre(ctx.map, c.at as usize);
        let pace = ctx.params.nav.tobler_ms(0.0).max(1e-6);
        let metres = f64::from((at.0 - x.home.0).hypot(at.1 - x.home.1));
        Some(BoatFacts {
            boat: c.id,
            at,
            walk_min: metres / pace / 60.0,
            left_h: f64::from(c.labour_h - c.work_h).max(0.0),
            saves_per_hour: f64::from(c.saves_h_year) * def.life_years
                / f64::from(c.labour_h).max(1.0),
            full: c.crew.0 == ctx.now.day_index() && u32::from(c.crew.1) >= def.crew,
        })
    }

    /// Someone sets off to work on boat `boat`, or on the cart of that id: one of its crew today.
    pub(super) fn joined_boat_crew(ctx: &mut Ctx, boat: PermanentId) {
        let day = ctx.now.day_index();
        let land = &mut *ctx.land;
        if let Some(b) = land
            .boats
            .list
            .iter_mut()
            .chain(land.carts.list.iter_mut())
            .chain(land.wagons.list.iter_mut())
            .find(|b| b.id == boat)
        {
            if b.crew.0 != day {
                b.crew = (day, 0);
            }
            b.crew.1 = b.crew.1.saturating_add(1);
        }
    }

    /// `hours` of a capable adult's work by `who` (handle and id) of `household` on the boat or
    /// cart `boat` it is making (M8b slice CO, third part; slice CP, third part): they count
    /// toward its labour, and they practise its vehicle's skill. A boat's trunk is felled where
    /// it is worked; a cart's timber was taken from the store when it was begun. Once its labour
    /// is done it is made, on the day it is, and the chronicle says so.
    pub(super) fn boat_work(
        &mut self,
        ctx: &mut Ctx,
        (h, who): (Handle<Person>, PermanentId),
        household: PermanentId,
        boat: PermanentId,
        hours: f64,
    ) {
        let making = |b: &Boat| b.id == boat && b.owner == household && !b.made();
        // 0 a boat, 1 a cart, 2 a wagon (M10b slice DV, fourth part).
        let (kind, i) = match ctx.land.boats.list.iter().position(making) {
            Some(i) => (0, i),
            None => match ctx.land.carts.list.iter().position(making) {
                Some(i) => (1, i),
                None => match ctx.land.wagons.list.iter().position(making) {
                    Some(i) => (2, i),
                    None => return,
                },
            },
        };
        let cart = kind != 0;
        let list = match kind {
            0 => &mut ctx.land.boats,
            1 => &mut ctx.land.carts,
            _ => &mut ctx.land.wagons,
        };
        let Some(def) = ctx.catalog.vehicles.get(usize::from(list.list[i].vehicle)) else {
            return;
        };
        let skill = def
            .skill
            .and_then(|k| ctx.catalog.skills.get(k).map(|s| (k, s)));
        let b = &mut list.list[i];
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
        let cell = b.at;
        list.changed();
        let name = def.name.to_lowercase();
        let settlement = self.household(household).and_then(|x| x.settlement);
        let wagon = kind == 2;
        let (kind, at, sentence) = if wagon && def.engine.is_some() {
            // An engine stands where it was made, at the pit end (M10b slice DW, fourth part).
            (
                ChronicleKind::CartMade,
                cell_centre(ctx.map, cell as usize),
                format!(
                    "{}'s household finished a {name} and stood it at the pit end of its line, \
                     ready to draw its wagons.",
                    self.name_of(who)
                ),
            )
        } else if wagon {
            (
                ChronicleKind::CartMade,
                cell_centre(ctx.map, cell as usize),
                format!(
                    "{}'s household finished a {name} and stood it at the end of its line, ready \
                     for the pit.",
                    self.name_of(who)
                ),
            )
        } else if cart {
            (
                ChronicleKind::CartMade,
                cell_centre(ctx.map, cell as usize),
                format!(
                    "{}'s household finished a {name} and stood it at the door, ready to load.",
                    self.name_of(who)
                ),
            )
        } else {
            (
                ChronicleKind::BoatMade,
                ctx.land.rivers.centre(cell),
                format!(
                    "{}'s household finished a {name} and drew it up on the bank, ready for the water.",
                    self.name_of(who)
                ),
            )
        };
        self.chronicle_push(now, kind, vec![who], settlement, Some(at), 0.0, sentence);
    }

    /// Household `from` is no more and its people joined household `to` (backlog D, third item):
    /// what it had in transit passes to `to`, as its store does, and, where `to` lives in the
    /// settlement `from` lived in (`near`), so do its boats and carts, made or being made. A boat
    /// stays drawn up where it is; a cart standing at `from`'s door (`door`) is wheeled to `to`'s,
    /// where a household's cart is used and made, as its store is carried there. Where `to` lives
    /// in another settlement (the ninth item), its made boats and carts pass too, standing where
    /// they are until someone of `to` fetches them ([`Population::vehicle_fetch_facts`]), so long
    /// as they stand within the walk a fetch allows of `to`'s home and a way would bring them
    /// there today ([`Population::way_home`]): an heir does not make its own while it holds one,
    /// so one it could not bring, one being made and a motor cart stay where they stand,
    /// `from`'s, as before. A wagon passes with the line it runs on ([`Population::lines_pass`]).
    pub(super) fn vehicles_pass(
        &mut self,
        ctx: &mut Ctx,
        (from, to): (PermanentId, PermanentId),
        near: bool,
        door: u32,
    ) {
        for c in self.consignments.iter_mut().filter(|c| c.owner == from) {
            c.owner = to;
        }
        let Some(home) = self.household(to).map(|x| x.home) else {
            return;
        };
        let new_door = cell_of(ctx.map, home) as u32;
        // How far off a fetch may find it: the walk a fetch allows, at an off-trail pace.
        let range_m = Self::fetch_range_m(ctx);
        let within = |at: (f32, f32)| f64::from((at.0 - home.0).hypot(at.1 - home.1)) <= range_m;
        // A made one of another settlement passes where it stands within that walk and a way
        // brings it home today.
        let fetchable = |v: &Boat, at: (f32, f32)| {
            v.made() && within(at) && Self::way_home(&*ctx, home, v).is_some()
        };
        let boats: Vec<bool> = ctx
            .land
            .boats
            .list
            .iter()
            .map(|b| b.owner == from && (near || fetchable(b, ctx.land.rivers.centre(b.at))))
            .collect();
        let carts: Vec<bool> = ctx
            .land
            .carts
            .list
            .iter()
            .map(|c| c.owner == from && (near || fetchable(c, cell_centre(ctx.map, c.at as usize))))
            .collect();
        let land = &mut *ctx.land;
        if boats.contains(&true) {
            for (b, _) in land.boats.list.iter_mut().zip(&boats).filter(|x| *x.1) {
                b.owner = to;
            }
            land.boats.changed();
        }
        if carts.contains(&true) {
            for (c, _) in land.carts.list.iter_mut().zip(&carts).filter(|x| *x.1) {
                c.owner = to;
                if near && c.at == door {
                    c.at = new_door;
                }
            }
            land.carts.changed();
        }
    }

    /// Household `household` finds a boat or cart of vehicle `vehicle` would save it
    /// `saves_h_year` hours of walking a year, to be made at cell `at`, and begins it or takes one
    /// up (backlog D, eleventh item): for tests.
    #[doc(hidden)]
    pub fn begin_vehicle_for_tests(
        &mut self,
        ctx: &mut Ctx,
        household: PermanentId,
        vehicle: usize,
        at: u32,
        saves_h_year: f32,
    ) {
        self.boat_wishes
            .push((household, vehicle, at, saves_h_year));
        self.begin_boats(ctx);
    }

    /// `hours` of work by the first of household `household` on the boat, cart, wagon or engine
    /// `boat` it is making: for tests.
    #[doc(hidden)]
    pub fn vehicle_work_for_tests(
        &mut self,
        ctx: &mut Ctx,
        household: PermanentId,
        boat: PermanentId,
        hours: f64,
    ) {
        let Some(who) = self
            .household(household)
            .and_then(|x| x.members.first().copied())
        else {
            return;
        };
        let Some(h) = self.index.get(&who).copied() else {
            return;
        };
        self.boat_work(ctx, (h, who), household, boat, hours);
    }
}
