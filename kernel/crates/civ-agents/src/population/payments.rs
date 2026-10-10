//! Payments agreements owe, performed by people with real goods (M5c slice AV, ADR-0020 §7;
//! research 13-02 §1.2 C, F; 13-01 §6.2). A gift falls due when its agreement comes into force,
//! and a transfer then and every `every_days` after while it is in force. Each midnight the paying
//! polity's common store sets aside what it holds of what is owed, up to the whole; nothing is
//! taken from households for it. What is set aside the payment holds, a ledger holder of its own,
//! and it spoils in the open. The one to carry it, the keeper of the paying store or else the one
//! who agreed to the terms, may take it up and walk it to the other settlement's hearth, where it
//! goes into the other's store; nobody is made to. A payment not handed over within
//! `deliver_days` is missed, its cause kept (an empty store, nobody to carry it, nobody who did),
//! and what it still holds goes back to the paying store. Four accounts: what the paying store's
//! households paid in (its levy), what it set aside, what was owed, and what arrived.

use super::*;
use crate::agreements::{AgreementState, Clause, Due, DueState, Miss};
use crate::decide::CarryFacts;
use crate::history::AgreementStep;
use crate::ledger::{Channel, Leg};

/// Less than this of a good is nothing, kilograms.
const NOTHING_KG: f64 = 1e-6;

impl Population {
    /// The payments' midnight (M5c slice AV), after the agreements': those falling due are
    /// owed, each open one has what the paying store holds of it set aside and someone to carry
    /// it, and one past its day is missed.
    pub(super) fn dues_day(&mut self, ctx: &mut Ctx) {
        let day = ctx.now.day_index();
        let deliver = ctx.params.relations.deliver_days;
        // Payments falling due under agreements in force.
        let mut owed: Vec<(PermanentId, u8, PermanentId, PermanentId, u16, u32)> = Vec::new();
        for a in &self.agreements.list {
            let AgreementState::InForce { since } = a.state else {
                continue;
            };
            for (k, c) in a.clauses.iter().enumerate() {
                let (count, good, kg) = match *c {
                    Clause::Leave { .. } => continue,
                    Clause::Gift { good, kg, .. } => (1, good, kg),
                    Clause::Transfer {
                        good,
                        kg,
                        every_days,
                        ..
                    } => ((day - since) / i64::from(every_days.max(1)) + 1, good, kg),
                };
                let made = self
                    .agreements
                    .dues
                    .iter()
                    .filter(|d| d.agreement == a.id && usize::from(d.clause) == k)
                    .count() as i64;
                if made < count {
                    let s = usize::from(c.from());
                    owed.push((a.id, k as u8, a.polities[s], a.polities[1 - s], good, kg));
                }
            }
        }
        for (agreement, clause, from, to, good, kg) in owed {
            let id = ctx.ids.allocate();
            self.agreements.dues.push(Due {
                id,
                agreement,
                clause,
                from,
                to,
                good,
                owed_kg: f64::from(kg),
                made: day,
                due: day + deliver,
                set_aside_kg: 0.0,
                arrived_kg: 0.0,
                carrier: None,
                state: DueState::Open,
                stores: vec![0.0; ctx.catalog.goods.len()],
                stores_at: ctx.now,
                flows: Default::default(),
            });
        }
        for i in 0..self.agreements.dues.len() {
            if self.agreements.dues[i].open() {
                self.tend_due(ctx, i);
            }
        }
    }

    /// Payment `i`, still owed: the paying store sets aside what it holds of what is owed, up to
    /// the whole; one is named to carry what is set aside; and one past its day is given up.
    fn tend_due(&mut self, ctx: &mut Ctx, i: usize) {
        let day = ctx.now.day_index();
        let d = &self.agreements.dues[i];
        let (id, from, good) = (d.id, d.from, usize::from(d.good));
        let Some(pi) = self.polities.iter().position(|p| p.id == from) else {
            return;
        };
        let want = d.owed_kg - d.set_aside_kg;
        if want > NOTHING_KG {
            let sheltered = self.polity_sheltered(pi);
            self.polities[pi].settle_stores(ctx.now, &ctx.catalog.goods, sheltered);
            let held = self.polities[pi].stores.get(good).copied().unwrap_or(0.0);
            let amount = want.min(held);
            if amount > NOTHING_KG {
                let leg = Leg {
                    from,
                    to: id,
                    good,
                    amount,
                };
                if self.transfer(
                    ctx.now,
                    ctx.params,
                    &ctx.catalog.goods,
                    &[leg],
                    Channel::Agreement,
                ) {
                    self.agreements.dues[i].set_aside_kg += amount;
                }
            }
        }
        let holding = self.agreements.dues[i]
            .stores
            .get(good)
            .is_some_and(|&kg| kg > NOTHING_KG);
        let carrier = if holding {
            self.carrier_for(pi, i)
        } else {
            None
        };
        self.agreements.dues[i].carrier = carrier;
        let d = &self.agreements.dues[i];
        if day > d.due {
            let why = if d.set_aside_kg <= NOTHING_KG {
                Miss::EmptyStore
            } else if carrier.is_none() {
                Miss::NoCarrier
            } else {
                Miss::NotCarried
            };
            // What it still holds goes back to the paying store.
            let left = d.stores.get(good).copied().unwrap_or(0.0);
            if left > NOTHING_KG {
                let leg = Leg {
                    from: id,
                    to: from,
                    good,
                    amount: left,
                };
                self.transfer(
                    ctx.now,
                    ctx.params,
                    &ctx.catalog.goods,
                    &[leg],
                    Channel::Agreement,
                );
            }
            self.agreements.dues[i].state = DueState::Missed { day, why };
            let sentence = format!(
                "{}'s payment of {} to {} was missed: {}.",
                self.polity_name(ctx, from),
                self.due_goods_words(ctx, i),
                self.polity_name(ctx, self.agreements.dues[i].to),
                why.words()
            );
            let agreement = self.agreements.dues[i].agreement;
            self.chronicle_payment(ctx, agreement, AgreementStep::Missed, sentence);
        }
    }

    /// Who is to carry payment `i` from polity `pi`: the one it named, while they live there; else
    /// the keeper of its store; else the one of its side who agreed to the terms; else nobody.
    fn carrier_for(&self, pi: usize, i: usize) -> Option<PermanentId> {
        let settlement = self.polities[pi].settlement;
        let lives_here = |p: PermanentId| {
            self.person(p)
                .and_then(|x| self.household(x.household))
                .is_some_and(|x| x.settlement == Some(settlement))
        };
        let d = &self.agreements.dues[i];
        let negotiator = self
            .agreements
            .get(d.agreement)
            .and_then(|a| a.side_of(d.from).map(|s| a.negotiators[usize::from(s)]));
        d.carrier
            .into_iter()
            .chain(self.polities[pi].keeper().map(|(k, _)| k))
            .chain(negotiator)
            .find(|&p| lives_here(p))
    }

    /// A payment `person` is to carry, set aside and waiting, if they may walk it there and back
    /// within the day (M5c slice AV): its store, where its polity keeps it (its keeper's home, or
    /// the hearth), and the hearth it goes to.
    pub(super) fn carry_facts(
        &self,
        ctx: &Ctx,
        person: PermanentId,
        field_key: PermanentId,
        dark: bool,
    ) -> Option<CarryFacts> {
        if dark || self.agreements.dues.is_empty() {
            return None;
        }
        let d = self.agreements.dues.iter().find(|d| {
            d.open()
                && d.carrier == Some(person)
                && d.stores
                    .get(usize::from(d.good))
                    .is_some_and(|&kg| kg > NOTHING_KG)
        })?;
        let payer = &self.polities[self.polities.iter().position(|p| p.id == d.from)?];
        let receiver = self.polities.iter().find(|p| p.id == d.to)?;
        let hearth_of = |s: PermanentId| {
            ctx.land
                .settlements
                .iter()
                .find(|x| x.id == s)
                .map(|x| x.hearth_m)
        };
        let (home_hearth, hearth) = (
            hearth_of(payer.settlement)?,
            hearth_of(receiver.settlement)?,
        );
        let store = payer
            .keeper()
            .and_then(|(k, _)| self.person(k))
            .and_then(|p| self.household(p.household))
            .map_or(home_hearth, |x| x.home);
        let reach = &self.homes.get(&field_key)?.reach;
        let walk_min = f64::from(reach.seconds_to(cell_of(ctx.map, hearth))?) / 60.0;
        let max = ctx
            .catalog
            .activities
            .iter()
            .filter(|a| a.behavior == Behavior::Carry)
            .map(|a| f64::from(a.max_walk_minutes))
            .fold(0.0, f64::max);
        if walk_min > max {
            return None;
        }
        Some(CarryFacts {
            store,
            settlement: receiver.settlement,
            hearth,
            walk_min,
            points: ctx.params.relations.carry_points,
        })
    }

    /// `person` has come to settlement `settlement`'s hearth, at `pos`, with a payment they carry
    /// for its polity (M5c slice AV): what it holds goes into that polity's store, and once all
    /// that was owed has been set aside and handed over, it is met. What spoiled on the way is
    /// lost.
    pub(super) fn hand_over(
        &mut self,
        ctx: &Ctx,
        person: PermanentId,
        settlement: PermanentId,
        pos: (f32, f32),
    ) {
        let at_hearth = ctx
            .land
            .settlements
            .iter()
            .find(|x| x.id == settlement)
            .is_some_and(|x| {
                (x.hearth_m.0 - pos.0).abs() < 1.0 && (x.hearth_m.1 - pos.1).abs() < 1.0
            });
        let Some(to) = self.polity_of(settlement).map(|i| self.polities[i].id) else {
            return;
        };
        if !at_hearth {
            return;
        }
        let Some(i) = self.agreements.dues.iter().position(|d| {
            // The one the carry was chosen for: owed to them, and holding something.
            d.open()
                && d.carrier == Some(person)
                && d.to == to
                && d.stores
                    .get(usize::from(d.good))
                    .is_some_and(|&kg| kg > NOTHING_KG)
        }) else {
            return;
        };
        let (id, good) = (self.agreements.dues[i].id, self.agreements.dues[i].good);
        // What it holds now, after the walk.
        self.agreements.dues[i].settle_stores(ctx.now, &ctx.catalog.goods);
        let held = self.agreements.dues[i]
            .stores
            .get(usize::from(good))
            .copied()
            .unwrap_or(0.0);
        if held <= NOTHING_KG {
            return;
        }
        let leg = Leg {
            from: id,
            to,
            good: usize::from(good),
            amount: held,
        };
        if !self.transfer(
            ctx.now,
            ctx.params,
            &ctx.catalog.goods,
            &[leg],
            Channel::Agreement,
        ) {
            return;
        }
        let day = ctx.now.day_index();
        let d = &mut self.agreements.dues[i];
        d.arrived_kg += held;
        if d.set_aside_kg >= d.owed_kg - NOTHING_KG {
            d.state = DueState::Met { day };
        }
        let (from, agreement) = (d.from, d.agreement);
        let sentence = format!(
            "{} carried {:.0} kg of {} from {}'s store to {}'s, as their agreement asks.",
            self.name_of(person),
            held,
            self.good_words(ctx, good),
            self.polity_name(ctx, from),
            self.polity_name(ctx, to)
        );
        self.chronicle_payment(ctx, agreement, AgreementStep::Delivered, sentence);
    }

    /// The name of polity `p`'s settlement.
    fn polity_name(&self, ctx: &Ctx, p: PermanentId) -> String {
        self.polities
            .iter()
            .find(|x| x.id == p)
            .and_then(|x| ctx.land.settlements.iter().find(|s| s.id == x.settlement))
            .map_or_else(|| "another settlement".to_owned(), |s| s.name.clone())
    }

    /// A good's name in lower case.
    fn good_words(&self, ctx: &Ctx, good: u16) -> String {
        ctx.catalog
            .goods
            .get(usize::from(good))
            .map_or_else(|| "goods".to_owned(), |d| d.name.to_lowercase())
    }

    /// What payment `i` owes in words: "100 kg of grain".
    fn due_goods_words(&self, ctx: &Ctx, i: usize) -> String {
        let d = &self.agreements.dues[i];
        format!("{:.0} kg of {}", d.owed_kg, self.good_words(ctx, d.good))
    }

    /// Adds a chronicle entry for a payment under agreement `agreement`.
    fn chronicle_payment(
        &mut self,
        ctx: &Ctx,
        agreement: PermanentId,
        step: AgreementStep,
        sentence: String,
    ) {
        let Some(a) = self.agreements.get(agreement) else {
            return;
        };
        let negotiators = a.negotiators.to_vec();
        let settlement = self
            .polities
            .iter()
            .find(|p| p.id == a.polities[0])
            .map(|p| p.settlement);
        let place = settlement.and_then(|s| {
            ctx.land
                .settlements
                .iter()
                .find(|x| x.id == s)
                .map(|x| x.hearth_m)
        });
        self.chronicle_push(
            ctx.now,
            ChronicleKind::Agreement,
            negotiators,
            settlement,
            place,
            f64::from(step as u8),
            sentence,
        );
    }
}
