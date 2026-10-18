//! Boats people take (M8b slice CO; ADR-0029 §3, §5): paddling the household's boat from the bank
//! where it is drawn up to another by the quickest way by water today, the boat drawn up where it
//! lands; and unloading a load in transit into the household's store once the boat is home. A
//! trip afloat wears no ground and passes no crossing or gate.

use super::*;

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
            .find(|b| b.owner == household && b.at == from)
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
            .find(|b| b.owner == household && b.at == from)
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
}
