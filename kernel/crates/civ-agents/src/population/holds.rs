//! A polity over several settlements (M6c slice BM, ADR-0023 §9). A polity keeps its seat and the
//! settlements it took in, each with since, how and from whom; membership is residence in any of
//! them, and its body gathers at the seat's hearth, where people of a far village walk or are
//! absent. Taking another polity in closes that polity's deciding: its record is kept and names
//! the polity that took it, its laws lapse, its common store becomes the taker's, and its
//! agreements end. Nothing rises toward loyalty: afterwards each person's compliance is their own
//! choice, as before (research 13-03 §1.6).

use super::*;
use crate::agreements::AgreementState;
use crate::polity::{Body, HeldHow, HeldSettlement, LawStatus, Polity, PolityEnd};

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

    /// The settlement a band polity `pi`'s law sent holds and has held `bands.hold_days`, if any
    /// (M6c slice BM), with the polity still deciding that holds it, if any: one `pi` does not
    /// hold already. The first such band in the order called.
    pub(super) fn held_by(&self, pi: usize) -> Option<(PermanentId, Option<usize>)> {
        let laws = &self.polities[pi].laws;
        self.order.bands.iter().find_map(|b| {
            if b.stage != crate::war::BandStage::Holding || b.held.is_none() {
                return None;
            }
            let law = b.law?;
            laws.iter().find(|l| l.id == law)?;
            let at = self.household(b.target)?.settlement?;
            if self.polities[pi].holds(Some(at)) {
                return None;
            }
            Some((at, self.polity_of(at)))
        })
    }

    /// Law `law` of polity `pi`, to take in settlement `at`, passed (M6c slice BM): `pi` takes in
    /// the polity holding `at`, or `at` alone where none does; the bands holding it go home; the
    /// law has run its course.
    pub(super) fn annex_passed(
        &mut self,
        ctx: &mut Ctx,
        pi: usize,
        law: PermanentId,
        at: PermanentId,
    ) {
        if self.polities[pi].holds(Some(at)) {
            return;
        }
        match self.polity_of(at) {
            Some(qi) => self.take_in(ctx, pi, qi, HeldHow::Annexed),
            None => {
                let day = ctx.now.day_index();
                let from = self.polities[pi].id;
                self.polities[pi].held.push(HeldSettlement {
                    settlement: at,
                    since: day,
                    how: HeldHow::Annexed,
                    from,
                });
            }
        }
        let holders: Vec<usize> = (0..self.order.bands.len())
            .filter(|&bi| {
                let b = &self.order.bands[bi];
                b.stage == crate::war::BandStage::Holding
                    && self.household(b.target).and_then(|x| x.settlement) == Some(at)
            })
            .collect();
        for bi in holders {
            let b = &self.order.bands[bi];
            let there: Vec<PermanentId> = b
                .arrived
                .iter()
                .copied()
                .filter(|m| !b.home.contains(m))
                .collect();
            for m in there {
                self.leave_now(ctx, m);
            }
            // It is on its way home, as any band is once it has met what it met.
            let b = &mut self.order.bands[bi];
            b.stage = crate::war::BandStage::Out;
            b.since = ctx.now.minutes();
        }
        if let Some(l) = self.polities[pi].law_mut(law) {
            l.status = LawStatus::Lapsed;
        }
    }

    /// A revolt called by `by` held at settlement `at`, which polity `pi` took in (ADR-0023 §9:
    /// secession is a revolt scoped to one settlement): `at` is a polity of its own from now on,
    /// founded with the revolt's body `body` as a daughter's is founded with its parent's, its
    /// founding custom taken by those who stood with `by`. It starts with no store and no laws:
    /// the common store stays where it was kept, `pi`'s. Agreements `pi` made stay `pi`'s. The
    /// chronicle tells it.
    pub(super) fn secede(
        &mut self,
        ctx: &mut Ctx,
        pi: usize,
        at: PermanentId,
        body: Body,
        by: PermanentId,
    ) {
        let Some(hi) = self.polities[pi]
            .held
            .iter()
            .position(|h| h.settlement == at)
        else {
            return;
        };
        if self.polities[pi].ended.is_some() {
            return;
        }
        self.polities[pi].held.remove(hi);
        let now = ctx.now;
        let mut polity = Polity::found(ctx.ids.allocate(), at, now, &ctx.params.polity);
        polity.body = body;
        for v in &mut polity.versions {
            v.body = body;
            v.seized_by = Some(by);
        }
        self.polities.push(polity);
        let name = self.settlement_name(ctx, at);
        let theirs = self.settlement_name(ctx, self.polities[pi].settlement);
        let place = ctx
            .land
            .settlements
            .iter()
            .find(|s| s.id == at)
            .map(|s| s.hearth_m);
        let words = format!(
            "At {name}, those who stood with {} broke from {theirs}'s polity: from now on {name} \
             decides for itself, {}.",
            self.name_of(by),
            body.clause()
        );
        self.chronicle_push(
            now,
            ChronicleKind::Seceded,
            vec![by],
            Some(at),
            place,
            0.0,
            words,
        );
    }

    /// For tests: a revolt called by `by` held at settlement `at` of polity `pi`, with body
    /// `body`.
    pub fn secede_for_tests(
        &mut self,
        ctx: &mut Ctx,
        pi: usize,
        at: PermanentId,
        body: Body,
        by: PermanentId,
    ) {
        self.secede(ctx, pi, at, body, by);
    }

    /// For tests: law `law` of polity `pi`, to take in settlement `at`, passed.
    pub fn annex_passed_for_tests(
        &mut self,
        ctx: &mut Ctx,
        pi: usize,
        law: PermanentId,
        at: PermanentId,
    ) {
        self.annex_passed(ctx, pi, law, at);
    }

    /// For tests: the settlement a band of polity `pi`'s law holds, if any.
    pub fn held_by_for_tests(&self, pi: usize) -> Option<PermanentId> {
        self.held_by(pi).map(|h| h.0)
    }

    /// For tests: polity `pi` takes polity `qi` in, `how`.
    pub fn take_in_for_tests(&mut self, ctx: &Ctx, pi: usize, qi: usize, how: HeldHow) {
        self.take_in(ctx, pi, qi, how);
    }
}
