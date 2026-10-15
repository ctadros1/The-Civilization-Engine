//! A polity over several settlements (M6c slice BM, ADR-0023 §9). A polity keeps its seat and the
//! settlements it took in, each with since, how and from whom; membership is residence in any of
//! them, and its body gathers at the seat's hearth, where people of a far village walk or are
//! absent. Taking another polity in closes that polity's deciding: its record is kept and names
//! the polity that took it, its laws lapse, its common store becomes the taker's, and its
//! agreements end. Nothing rises toward loyalty: afterwards each person's compliance is their own
//! choice, as before (research 13-03 §1.6).

use super::*;
use crate::agreements::AgreementState;
use crate::polity::{HeldHow, HeldSettlement, LawStatus, PolityEnd};

impl Population {
    /// Polity `pi` takes polity `qi` in, `how` (ADR-0023 §9): `qi`'s settlements become `pi`'s,
    /// their residents its members; `qi`'s deciding closes, its laws lapse and its gathering is
    /// called off; its store, brought up to date, is added to `pi`'s (the store stays where it was
    /// and is now `pi`'s; one store a polity is a simplification); every agreement `qi` was party
    /// to ends. The chronicle tells it. Nothing happens if either has already been taken in.
    pub(super) fn take_in(&mut self, ctx: &Ctx, pi: usize, qi: usize, how: HeldHow) {
        if pi == qi || self.polities[pi].ended.is_some() || self.polities[qi].ended.is_some() {
            return;
        }
        let (now, day) = (ctx.now, ctx.now.day_index());
        let goods = &ctx.catalog.goods;
        for i in [pi, qi] {
            let sheltered = self.polity_sheltered(i);
            self.polities[i].settle_stores(now, goods, sheltered);
        }
        let (by, from) = (self.polities[pi].id, self.polities[qi].id);
        let taken = self.polities[qi].settlements();
        let stores = std::mem::take(&mut self.polities[qi].stores);
        {
            let q = &mut self.polities[qi];
            for (g, &kg) in stores.iter().enumerate() {
                if kg > 0.0 {
                    q.flows.add(crate::person::Flow::Given, g, kg);
                }
            }
            q.stores = vec![0.0; stores.len()];
            for l in &mut q.laws {
                if l.status == LawStatus::InForce {
                    l.status = LawStatus::Lapsed;
                }
            }
            q.gathering = None;
            q.ended = Some(PolityEnd { day, by, how });
        }
        {
            let p = &mut self.polities[pi];
            if p.stores.len() < stores.len() {
                p.stores.resize(stores.len(), 0.0);
            }
            for (g, &kg) in stores.iter().enumerate() {
                if kg > 0.0 {
                    p.stores[g] += kg;
                    p.flows.add(crate::person::Flow::Received, g, kg);
                }
            }
            for s in &taken {
                p.held.push(HeldSettlement {
                    settlement: *s,
                    since: day,
                    how,
                    from,
                });
            }
        }
        // Every agreement it was party to ends: there is nobody left to keep it on its side.
        for a in &mut self.agreements.list {
            if (a.open() || a.in_force())
                && let Some(s) = a.side_of(from)
            {
                a.state = AgreementState::Ended { day, by: Some(s) };
            }
        }
        let (seat, name) = (taken[0], self.settlement_name(ctx, taken[0]));
        let theirs = self.settlement_name(ctx, self.polities[pi].settlement);
        let place = ctx
            .land
            .settlements
            .iter()
            .find(|s| s.id == seat)
            .map(|s| s.hearth_m);
        let sentence = match how {
            HeldHow::Annexed => format!(
                "The gathering at {theirs} took {name} in: its gathering decides no more, its laws \
                 lapsed, and its people are members of {theirs}'s polity."
            ),
            HeldHow::Merged => format!(
                "{name} and {theirs} became one body, as both their gatherings agreed: the custom \
                 at {theirs} governs, and {name}'s people are its members."
            ),
        };
        self.chronicle_push(
            now,
            ChronicleKind::PolityJoined,
            Vec::new(),
            Some(seat),
            place,
            f64::from(how.code()),
            sentence,
        );
    }

    /// For tests: polity `pi` takes polity `qi` in, `how`.
    pub fn take_in_for_tests(&mut self, ctx: &Ctx, pi: usize, qi: usize, how: HeldHow) {
        self.take_in(ctx, pi, qi, how);
    }
}
