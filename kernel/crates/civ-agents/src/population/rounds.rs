//! The refuse round (M11f, third part; ADR-0038 §4; research 12-10 §1.3–§1.4, §5.3, §5.9): a
//! household that keeps a cart may post a round, as a road line's keeper posts runs (ADR-0036 §7):
//! the homes of its settlement it calls at, a run every so many days, and the open tip it carries
//! to. On a run it calls at the homes in turn, timed by its cart's way, empties each heap, its
//! decaying share and its ash, into the cart up to the cart's payload, and tips the load at the
//! tip as a household carrying its own heap does: mass and disease loads move whole, recorded as
//! carried. Homes beyond what a run carries wait for the next. Each home called at pays the keeper
//! a fee in food on the `Refuse` channel, its hours on a run over the homes it calls at, at what an
//! hour of its work brings in food; a home that cannot pay is not called at until it can. Nothing
//! loads without a tip a polity opened and keeps by law.
//!
//! Households weigh it in hours: a home joins where the fees a year cost it less, by its hours,
//! than the carrying a law asks of it a year; under no law it gains nothing and does not join. The
//! keeper posts a round where the homes that would join at the fee repay its hours, a run fitting
//! in a day's work. A polity may instead run one by law (`collect_refuse`): worked by a household
//! of its settlement that keeps a cart, calling at every home for nothing, its hours paid from the
//! store as common work; with nothing in the store to pay them it does not go.

use civ_core::time::DAYS_PER_YEAR;
use civ_core::{PermanentId, Rng64};

use super::*;
use crate::ledger::Channel;
use crate::polity::{LawStatus, PolicyKind, RefuseRound, RoundCall, RoundRun};

/// Days between a posted round's runs (a design prior: research 12-10 §1.3 and §2.4, a week
/// before refuse in a warm season breeds flies).
pub const ROUND_EVERY_DAYS: u16 = 7;

/// The most homes a keeper weighs calling at (a design prior: about what a cart's keeper could
/// know as its neighbours).
pub const MAX_ROUND_HOMES: usize = 40;

/// Keyed-randomness purpose of a cart keeper's day of the year for weighing a refuse round.
const PURPOSE_REFUSE_REVIEW: u64 = 0x7265_6675_7365_7276; // "refuserv"

/// The food a household holds to pay with, kcal: what a fee or a fare is paid from.
fn food_kcal(ctx: &Ctx, x: &Household) -> f64 {
    let goods = &ctx.catalog.goods;
    let stores = stores_now(x, ctx.now, ctx.params, goods);
    goods
        .iter()
        .enumerate()
        .filter(|(_, d)| d.purpose == GoodUse::Food && d.kcal_per_kg > 0.0 && !d.kept_back())
        .map(|(g, d)| stores.get(g).copied().unwrap_or(0.0).max(0.0) * d.kcal_per_kg)
        .sum()
}

/// The metres between two places.
fn apart(a: (f32, f32), b: (f32, f32)) -> f64 {
    f64::from(a.0 - b.0).hypot(f64::from(a.1 - b.1))
}

impl Population {
    /// The cart household `x` would work a round with: its made cart of a vehicle that goes as a
    /// cart, the greatest payload first, then the oldest; and the vehicle's index.
    fn round_cart(ctx: &Ctx, x: PermanentId) -> Option<(PermanentId, usize)> {
        let vehicles = &ctx.catalog.vehicles;
        ctx.land
            .carts
            .made_of(x)
            .filter_map(|c| {
                let v = usize::from(c.vehicle);
                vehicles
                    .get(v)
                    .filter(|d| d.mode == crate::params::VehicleMode::Cart && d.payload_kg > 0.0)
                    .map(|d| (c.id, v, d.payload_kg))
            })
            .fold(
                None,
                |best: Option<(PermanentId, usize, f64)>, c| match best {
                    Some(b) if b.2 >= c.2 => Some(b),
                    _ => Some(c),
                },
            )
            .map(|(c, v, _)| (c, v))
    }

    /// The tip of polity `pi` a round may carry to (ADR-0038 §4): the first it keeps whose law
    /// carrying refuse to it is in force; its index.
    pub(super) fn round_tip(&self, pi: usize) -> Option<usize> {
        let p = &self.polities[pi];
        p.tips.iter().position(|t| {
            p.laws.iter().any(|l| {
                l.id == t.law && l.status == LawStatus::InForce && l.kind == PolicyKind::RefuseToTip
            })
        })
    }

    /// The index of tip `tip` of polity `pi`, while the law that opened it is in force: a run
    /// whose tip is gone does not load.
    fn usable_tip(&self, pi: usize, tip: PermanentId) -> Option<usize> {
        let p = &self.polities[pi];
        let ti = p.tips.iter().position(|t| t.id == tip)?;
        p.laws
            .iter()
            .any(|l| {
                l.id == p.tips[ti].law
                    && l.status == LawStatus::InForce
                    && l.kind == PolicyKind::RefuseToTip
            })
            .then_some(ti)
    }

    /// The hours a year a law asks household `hh` to carry its own heap (ADR-0038 §4): under a
    /// law carrying refuse to an open tip, its carrying every so many days; else under a law
    /// clearing middens, where its home lies near the water, its carrying every so many days;
    /// else none.
    pub(super) fn carrying_asked_h_year(&self, ctx: &Ctx, hh: &Household) -> f64 {
        if let Some((_, _, _, def)) = self.tip_law(ctx, hh) {
            return DAYS_PER_YEAR as f64 / f64::from(def.every_days.max(1))
                * f64::from(def.carry_h);
        }
        let Some(pi) = hh.settlement.and_then(|s| self.polity_of(s)) else {
            return 0.0;
        };
        let middens = self.polities[pi]
            .laws
            .iter()
            .filter(|l| l.status == LawStatus::InForce && l.kind == PolicyKind::ClearMiddens)
            .find_map(|l| ctx.catalog.policies.get(usize::from(l.policy))?.middens);
        match middens {
            Some(def)
                if Self::nearest_source(ctx, hh.home, f64::from(def.distance_m)).is_some() =>
            {
                DAYS_PER_YEAR as f64 / f64::from(def.keep_days.max(1)) * f64::from(def.carry_h)
            }
            _ => 0.0,
        }
    }

    /// Whether household `hh` would join a round calling every `every_days` for a fee of
    /// `fee_kcal` a call (ADR-0038 §4): where the fees a year cost it less, at what an hour of its
    /// work brings in food, than the hours a law asks it to carry its heap a year. Under no law it
    /// gains nothing and does not.
    pub(super) fn would_join_round(
        &self,
        ctx: &Ctx,
        hh: &Household,
        fee_kcal: f64,
        every_days: u16,
    ) -> bool {
        let asked_h = self.carrying_asked_h_year(ctx, hh);
        if asked_h <= 0.0 {
            return false;
        }
        let fees = DAYS_PER_YEAR as f64 / f64::from(every_days.max(1)) * fee_kcal.max(0.0);
        fees < asked_h * self.hour_kcal(ctx, hh)
    }

    /// Households `homes` in calling order from `door`: each next the nearest to the last, the
    /// lower id of two alike.
    fn round_order(&self, door: (f32, f32), homes: &[PermanentId]) -> Vec<PermanentId> {
        let mut left: Vec<(PermanentId, (f32, f32))> = homes
            .iter()
            .filter_map(|&h| self.household(h).map(|x| (h, x.home)))
            .collect();
        left.sort_unstable_by_key(|e| e.0);
        let mut out = Vec::with_capacity(left.len());
        let mut at = door;
        while !left.is_empty() {
            let i = (0..left.len())
                .min_by(|&a, &b| apart(at, left[a].1).total_cmp(&apart(at, left[b].1)))
                .unwrap_or(0);
            let (h, p) = left.remove(i);
            out.push(h);
            at = p;
        }
        out
    }

    /// What household `operator` would post with its cart's vehicle `vehicle` to the tip at `tip`
    /// (ADR-0038 §4): the homes of its settlement, the nearest [`MAX_ROUND_HOMES`] in calling
    /// order, that would join at the fee, and the fee, kcal a call. A run calls at as many as the
    /// heaps of [`ROUND_EVERY_DAYS`] fill its cart with, each its people's share of a heap a day;
    /// its hours are the way door to homes to tip and back, at the seconds a metre of its cart's
    /// way from the door to the tip, and loading and tipping what it carries by hand; the fee is
    /// those hours at what an hour of the keeper's work brings in food, and a motor cart's fuel
    /// for the way at its own cost, over the homes a run calls at. The homes that would not join
    /// at that fee are let go and the fee weighed again until all that are left would. None where
    /// the cart has no way to the tip, the keeper puts no figure on its hours, a run does not fit
    /// in a day's work, or no home would join.
    fn plan_round(
        &self,
        ctx: &Ctx,
        operator: PermanentId,
        vehicle: usize,
        tip: (f32, f32),
    ) -> Option<(Vec<PermanentId>, f32)> {
        let x = self.household(operator)?;
        let (settlement, door) = (x.settlement?, x.home);
        let v = ctx.catalog.vehicles.get(vehicle)?;
        let walk = ctx.params.nav.tobler_ms(0.0);
        let (route, legs_s) =
            ctx.land
                .cart_way(ctx.map, &v.wheels(v.payload_kg / 2.0, walk), door, tip)?;
        let way_s = route.at_s.last().copied().unwrap_or(0.0) + legs_s;
        let s_per_m = way_s / apart(door, tip).max(f64::from(ctx.map.cell_size_m));
        let mp = &ctx.params.midden;
        let every = f64::from(ROUND_EVERY_DAYS);
        // A keeper that puts no figure on its hours has nothing for fees to repay.
        let hour_kcal = self.hour_kcal(ctx, x);
        if hour_kcal <= 0.0 {
            return None;
        }
        let day_h = ctx.params.farm.work_hours_per_day;
        let mut near: Vec<(PermanentId, f64)> = self
            .households
            .iter()
            .map(|(_, h)| h)
            .filter(|h| h.id != operator && h.settlement == Some(settlement))
            .filter(|h| !h.members.is_empty())
            .map(|h| (h.id, apart(door, h.home)))
            .collect();
        near.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
        near.truncate(MAX_ROUND_HOMES);
        let ids: Vec<PermanentId> = near.into_iter().map(|e| e.0).collect();
        let mut homes = self.round_order(door, &ids);
        for _ in 0..=MAX_ROUND_HOMES {
            if homes.is_empty() {
                return None;
            }
            // The homes a run calls at: as many as a run's heaps fill the cart with, at least one.
            let call_kg: Vec<f64> = homes
                .iter()
                .filter_map(|&h| self.household(h))
                .map(|h| h.members.len() as f64 * mp.kg_per_person_day * every)
                .collect();
            let mut m = 0;
            let mut kg = 0.0;
            for &c in &call_kg {
                if m > 0 && kg + c > v.payload_kg {
                    break;
                }
                kg += c;
                m += 1;
            }
            let run = |m: usize| -> Option<(f64, f64)> {
                let mut at = door;
                let mut metres = 0.0;
                for &h in &homes[..m] {
                    let p = self.household(h)?.home;
                    metres += apart(at, p);
                    at = p;
                }
                metres += apart(at, tip) + apart(tip, door);
                let travel_h = metres * s_per_m / 3600.0;
                let loaded: f64 = call_kg[..m].iter().sum::<f64>().min(v.payload_kg);
                let handling_h = 2.0 * loaded / v.handling_kg_min.max(1e-6) / 60.0;
                Some((travel_h + handling_h, travel_h))
            };
            let (mut hours, mut travel_h) = run(m)?;
            while hours > day_h && m > 1 {
                m -= 1;
                (hours, travel_h) = run(m)?;
            }
            if hours > day_h {
                return None;
            }
            let fuel_kcal = if v.engine.is_some() {
                self.run_cost_kcal(ctx, operator, vehicle, travel_h * 60.0)?
            } else {
                0.0
            };
            let fee = (hours * hour_kcal + fuel_kcal) / m as f64;
            let joining: Vec<PermanentId> = homes
                .iter()
                .copied()
                .filter(|&h| {
                    self.household(h)
                        .is_some_and(|x| self.would_join_round(ctx, x, fee, ROUND_EVERY_DAYS))
                })
                .collect();
            if joining.len() == homes.len() {
                return Some((homes, fee as f32));
            }
            homes = joining;
        }
        None
    }

    /// Household `operator` weighs its refuse round (ADR-0038 §4): with one posted, it withdraws
    /// it where its cart is gone, its tip is no longer kept or no home would join, and otherwise
    /// sets its homes and fee afresh; without one, where it keeps a cart and its polity keeps an
    /// open tip, it posts one to that tip where [`Population::plan_round`] finds homes that
    /// would join.
    fn review_refuse_round(&mut self, ctx: &mut Ctx, operator: PermanentId, day: i64) {
        let Some(pi) = self
            .household(operator)
            .and_then(|x| x.settlement)
            .and_then(|s| self.polity_of(s))
        else {
            return;
        };
        let cart = Self::round_cart(ctx, operator);
        let posted = self.polities[pi]
            .rounds
            .iter()
            .position(|r| r.operator == operator && r.law.is_none());
        let tip = match posted {
            Some(ri) => self.usable_tip(pi, self.polities[pi].rounds[ri].tip),
            None => self.round_tip(pi),
        };
        let plan = match (cart, tip) {
            (Some((_, vehicle)), Some(ti)) => {
                let place = self.polities[pi].tips[ti].place;
                self.plan_round(ctx, operator, vehicle, place)
            }
            _ => None,
        };
        match (posted, plan, cart, tip) {
            (Some(ri), Some((homes, fee)), Some((cart, _)), Some(_)) => {
                let r = &mut self.polities[pi].rounds[ri];
                if r.homes != homes {
                    r.next = 0;
                }
                r.homes = homes;
                r.fee_kcal = fee;
                r.cart = cart;
            }
            (Some(ri), _, _, _) => {
                self.polities[pi].rounds.remove(ri);
            }
            (None, Some((homes, fee)), Some((cart, _)), Some(ti)) => {
                let id = ctx.ids.allocate();
                let tip = self.polities[pi].tips[ti].id;
                self.polities[pi].rounds.push(RefuseRound {
                    id,
                    operator,
                    cart,
                    tip,
                    law: None,
                    homes,
                    every_days: ROUND_EVERY_DAYS,
                    fee_kcal: fee,
                    since: day,
                    last_run: -1,
                    next: 0,
                    runs: Vec::new(),
                });
            }
            _ => {}
        }
    }

    /// Each polity's laws collecting refuse keep their rounds (ADR-0038 §4): a round for each law
    /// in force while an open tip is kept, worked by the household of the settlement whose cart
    /// carries most (the lower id of two alike) unless the one working it still keeps its cart,
    /// calling at every other home of the settlement in calling order; a law's round is let go
    /// with the law, or with its keeper's cart where no one else keeps one.
    pub(super) fn law_rounds(&mut self, ctx: &mut Ctx, day: i64) {
        for pi in 0..self.polities.len() {
            let laws: Vec<(PermanentId, u16)> = self.polities[pi]
                .laws
                .iter()
                .filter(|l| l.status == LawStatus::InForce && l.kind == PolicyKind::CollectRefuse)
                .filter_map(|l| {
                    let def = ctx.catalog.policies.get(usize::from(l.policy))?.round?;
                    Some((l.id, def.every_days))
                })
                .collect();
            self.polities[pi]
                .rounds
                .retain(|r| r.law.is_none_or(|l| laws.iter().any(|x| x.0 == l)));
            if laws.is_empty() {
                continue;
            }
            let settlement = self.polities[pi].settlement;
            let Some(ti) = self.round_tip(pi) else {
                continue;
            };
            let tip = self.polities[pi].tips[ti].id;
            let mut ours: Vec<PermanentId> = self
                .households
                .iter()
                .map(|(_, x)| x)
                .filter(|x| x.settlement == Some(settlement) && !x.members.is_empty())
                .map(|x| x.id)
                .collect();
            ours.sort_unstable();
            let keeper = ours
                .iter()
                .filter_map(|&h| {
                    let (c, v) = Self::round_cart(ctx, h)?;
                    Some((h, c, ctx.catalog.vehicles.get(v)?.payload_kg))
                })
                .fold(
                    None,
                    |best: Option<(PermanentId, PermanentId, f64)>, c| match best {
                        Some(b) if b.2 >= c.2 => Some(b),
                        _ => Some(c),
                    },
                );
            for (law, every_days) in laws {
                let ri = self.polities[pi]
                    .rounds
                    .iter()
                    .position(|r| r.law == Some(law));
                // Its keeper stays while it keeps its cart.
                let kept = ri.and_then(|ri| {
                    let r = &self.polities[pi].rounds[ri];
                    ctx.land
                        .carts
                        .made_of(r.operator)
                        .any(|c| c.id == r.cart)
                        .then_some((r.operator, r.cart))
                });
                let Some((operator, cart)) = kept.or(keeper.map(|k| (k.0, k.1))) else {
                    if let Some(ri) = ri {
                        self.polities[pi].rounds.remove(ri);
                    }
                    continue;
                };
                let Some(door) = self.household(operator).map(|x| x.home) else {
                    continue;
                };
                let others: Vec<PermanentId> =
                    ours.iter().copied().filter(|&h| h != operator).collect();
                match ri {
                    Some(ri) => {
                        let same = {
                            let r = &self.polities[pi].rounds[ri];
                            let mut was = r.homes.clone();
                            was.sort_unstable();
                            r.operator == operator && was == others
                        };
                        let homes = (!same).then(|| self.round_order(door, &others));
                        let r = &mut self.polities[pi].rounds[ri];
                        if let Some(homes) = homes {
                            r.homes = homes;
                            r.next = 0;
                        }
                        r.operator = operator;
                        r.cart = cart;
                        r.tip = tip;
                        r.every_days = every_days;
                    }
                    None => {
                        let homes = self.round_order(door, &others);
                        let id = ctx.ids.allocate();
                        self.polities[pi].rounds.push(RefuseRound {
                            id,
                            operator,
                            cart,
                            tip,
                            law: Some(law),
                            homes,
                            every_days,
                            fee_kcal: 0.0,
                            since: day,
                            last_run: -1,
                            next: 0,
                            runs: Vec::new(),
                        });
                    }
                }
            }
        }
    }

    /// Round `ri` of polity `pi` runs today (ADR-0038 §4), where its tip is still kept, its cart
    /// still made and kept by its keeper and, under a law, the store would pay its hours: from
    /// the keeper's door it calls at the homes on it in turn from where the last run stopped, by
    /// its cart's way loaded as it is, at each that holds a heap (and, paying a fee, would still
    /// join and holds the fee) emptying the heap into the cart up to the cart's payload, then
    /// carries the load to the tip and back. A motor cart burns its way's fuel, and goes only
    /// with the fuel. Each home pays its fee on the `Refuse` channel as its heap is loaded, its
    /// heap's share going whole to the tip; under a law the store pays the keeper's hours as
    /// common work. Returns whether it ran.
    fn refuse_run(&mut self, ctx: &mut Ctx, pi: usize, ri: usize, day: i64) -> bool {
        let r = self.polities[pi].rounds[ri].clone();
        let Some(ti) = self.usable_tip(pi, r.tip) else {
            return false;
        };
        let tip = self.polities[pi].tips[ti].place;
        let catalog = ctx.catalog;
        let Some(v) = ctx
            .land
            .carts
            .made_of(r.operator)
            .find(|c| c.id == r.cart)
            .and_then(|c| catalog.vehicles.get(usize::from(c.vehicle)))
        else {
            return false;
        };
        let Some((settlement, door)) = self
            .household(r.operator)
            .and_then(|x| Some((x.settlement?, x.home)))
        else {
            return false;
        };
        if r.law.is_some()
            && (self.work_pays_of(ctx, pi).next().is_none() || self.work_unpaid(ctx, pi))
        {
            return false;
        }
        let walk = ctx.params.nav.tobler_ms(0.0);
        let fee = f64::from(r.fee_kcal);
        let n = r.homes.len();
        let start = if n == 0 { 0 } else { r.next as usize % n };
        let (mut at, mut load, mut secs) = (door, 0.0_f64, 0.0_f64);
        let mut calls: Vec<(PermanentId, f64)> = Vec::new();
        let mut next = start;
        // Each home's stores brought up to now, so its heap holds the ash of what it burned.
        let params = ctx.params;
        for &h in &r.homes {
            if let Some(x) = self.household_mut(h) {
                let members = x.members.len().max(1);
                x.settle_stores(ctx.now, &catalog.goods, &|d| {
                    fuel_per_day(params, members, d)
                });
            }
        }
        for i in 0..n {
            let k = (start + i) % n;
            let h = r.homes[k];
            let Some(x) = self.household(h) else {
                continue;
            };
            if x.members.is_empty() || x.settlement != Some(settlement) {
                continue;
            }
            if r.law.is_none()
                && (!self.would_join_round(ctx, x, fee, r.every_days) || food_kcal(ctx, x) < fee)
            {
                continue;
            }
            let heap = x
                .midden
                .at_time(ctx.now, x.members.len(), &ctx.params.midden)
                + x.midden.ash_kg;
            if heap <= 0.0 {
                continue;
            }
            let Some((route, legs_s)) =
                ctx.land
                    .cart_way(ctx.map, &v.wheels(load, walk), at, x.home)
            else {
                continue;
            };
            let share = ((v.payload_kg - load) / heap).min(1.0);
            secs += route.at_s.last().copied().unwrap_or(0.0) + legs_s;
            load += heap * share;
            at = x.home;
            calls.push((h, share));
            if share < 1.0 || load >= v.payload_kg - 1e-9 {
                next = if share < 1.0 { k } else { (k + 1) % n };
                break;
            }
        }
        if calls.is_empty() {
            self.polities[pi].rounds[ri].last_run = day;
            return false;
        }
        let Some((route, legs_s)) = ctx.land.cart_way(ctx.map, &v.wheels(load, walk), at, tip)
        else {
            return false;
        };
        secs += route.at_s.last().copied().unwrap_or(0.0) + legs_s;
        if let Some((route, legs_s)) = ctx.land.cart_way(ctx.map, &v.wheels(0.0, walk), tip, door) {
            secs += route.at_s.last().copied().unwrap_or(0.0) + legs_s;
        }
        let burn = v.trip_fuel(&catalog.goods, secs / 60.0);
        if let Some((fuel, kg)) = burn
            && self
                .household(r.operator)
                .and_then(|x| x.stores.get(fuel).copied())
                .unwrap_or(0.0)
                + 1e-9
                < kg
        {
            return false;
        }
        let hours = secs / 3600.0 + 2.0 * load / v.handling_kg_min.max(1e-6) / 60.0;
        let mut called = Vec::with_capacity(calls.len());
        for (h, share) in calls {
            if r.law.is_none() && fee > 0.0 {
                self.pay_food_by(ctx, h, r.operator, fee, Channel::Refuse);
            }
            let (kg, ash_kg) = self.tip_share(ctx, h, pi, ti, share);
            called.push(RoundCall {
                household: h,
                kg,
                ash_kg,
            });
        }
        if let Some((fuel, kg)) = burn
            && let Some(x) = self.household_mut(r.operator)
        {
            let held = x.stores.get(fuel).copied().unwrap_or(0.0);
            x.stores[fuel] = (held - kg).max(0.0);
            x.flows.add(Flow::Used, fuel, kg);
        }
        if r.law.is_some() {
            let polity = self.polities[pi].id;
            self.pay_for_work(ctx, polity, r.operator, hours);
        }
        self.changed(r.operator);
        let round = &mut self.polities[pi].rounds[ri];
        round.last_run = day;
        round.next = next as u32;
        round.runs.retain(|x| x.day > day - DAYS_PER_YEAR);
        round.runs.push(RoundRun {
            day,
            hours: hours as f32,
            calls: called,
        });
        true
    }

    /// Each midnight (ADR-0038 §4): the laws collecting refuse keep their rounds; the households
    /// keeping a cart whose day of the year it is weigh a round; and each round due runs. Nothing
    /// where no polity keeps a tip or a round.
    pub(super) fn refuse_day(&mut self, ctx: &mut Ctx, day: i64) {
        if self
            .polities
            .iter()
            .all(|p| p.tips.is_empty() && p.rounds.is_empty())
        {
            return;
        }
        self.law_rounds(ctx, day);
        if !ctx.land.carts.list.is_empty() {
            let operators: Vec<PermanentId> = self
                .polities
                .iter()
                .flat_map(|p| p.rounds.iter())
                .filter(|r| r.law.is_none())
                .map(|r| r.operator)
                .collect();
            let due: Vec<PermanentId> = self
                .households
                .iter()
                .map(|(_, x)| x.id)
                .filter(|&id| {
                    Rng64::from_key(&[ctx.seed, PURPOSE_REFUSE_REVIEW, id.get()]).next_u64()
                        % DAYS_PER_YEAR as u64
                        == day.rem_euclid(DAYS_PER_YEAR) as u64
                })
                .filter(|&id| Self::round_cart(ctx, id).is_some() || operators.contains(&id))
                .collect();
            for operator in due {
                self.review_refuse_round(ctx, operator, day);
            }
        }
        self.refuse_runs(ctx, day);
    }

    /// Every round whose days have passed since its last run runs (ADR-0038 §4).
    fn refuse_runs(&mut self, ctx: &mut Ctx, day: i64) {
        for pi in 0..self.polities.len() {
            for ri in 0..self.polities[pi].rounds.len() {
                let r = &self.polities[pi].rounds[ri];
                if r.last_run < 0 || day - r.last_run >= i64::from(r.every_days.max(1)) {
                    self.refuse_run(ctx, pi, ri, day);
                }
            }
        }
    }

    /// What a refuse round run by law would bring each of `households` a year and cost it, kcal
    /// (ADR-0038 §4): the hours a law asks it to carry its heap a year, which the round spares
    /// it, and its even share of the template's hours a run every `every_days` days paid from
    /// the store, each at what an hour of its work brings in food.
    pub(super) fn rounds_forecasts(
        &self,
        ctx: &Ctx,
        households: &[PermanentId],
    ) -> Vec<(PermanentId, f64, f64)> {
        let Some(def) = ctx.catalog.policies.iter().find_map(|d| d.round) else {
            return Vec::new();
        };
        let share = DAYS_PER_YEAR as f64 / f64::from(def.every_days.max(1))
            * f64::from(def.round_h)
            / households.len().max(1) as f64;
        households
            .iter()
            .filter_map(|&h| {
                let x = self.household(h)?;
                let kcal = self.hour_kcal(ctx, x);
                Some((h, self.carrying_asked_h_year(ctx, x) * kcal, share * kcal))
            })
            .collect()
    }

    /// Household `operator` weighs its refuse round now, whatever its day of the year: for
    /// tests.
    #[doc(hidden)]
    pub fn review_refuse_round_for_tests(&mut self, ctx: &mut Ctx, operator: PermanentId) {
        let day = ctx.now.day_index();
        self.review_refuse_round(ctx, operator, day);
    }

    /// Round `round` runs now, whatever its days: for tests. Whether it ran.
    #[doc(hidden)]
    pub fn refuse_run_for_tests(&mut self, ctx: &mut Ctx, round: PermanentId) -> bool {
        let day = ctx.now.day_index();
        let at = self.polities.iter().enumerate().find_map(|(pi, p)| {
            p.rounds
                .iter()
                .position(|r| r.id == round)
                .map(|ri| (pi, ri))
        });
        at.is_some_and(|(pi, ri)| self.refuse_run(ctx, pi, ri, day))
    }

    /// The laws collecting refuse keep their rounds now: for tests.
    #[doc(hidden)]
    pub fn law_rounds_for_tests(&mut self, ctx: &mut Ctx) {
        let day = ctx.now.day_index();
        self.law_rounds(ctx, day);
    }

    /// Whether household `household` would join a round calling every `every_days` days for
    /// `fee_kcal` a call: for tests.
    #[doc(hidden)]
    pub fn would_join_round_for_tests(
        &self,
        ctx: &Ctx,
        household: PermanentId,
        fee_kcal: f64,
        every_days: u16,
    ) -> bool {
        self.household(household)
            .is_some_and(|x| self.would_join_round(ctx, x, fee_kcal, every_days))
    }

    /// The hours a year a law asks household `household` to carry its heap: for tests.
    #[doc(hidden)]
    pub fn carrying_asked_h_year_for_tests(&self, ctx: &Ctx, household: PermanentId) -> f64 {
        self.household(household)
            .map_or(0.0, |x| self.carrying_asked_h_year(ctx, x))
    }
}
