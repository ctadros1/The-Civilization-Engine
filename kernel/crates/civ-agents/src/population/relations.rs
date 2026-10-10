//! Relations between polities as people live them (M5c slice AT, ADR-0020 §3–§5): what those who
//! work a place their polity claims hold against people of another settlement they see working
//! it, and what they come to believe of that settlement's polity.

use super::*;
use crate::uses::Meeting;
use crate::views::ViewAct;
use crate::word::{Blamed, Grieved, Wrong};

impl Population {
    /// People of more than one settlement worked one place on one day (ADR-0020 §4–§5). Each who
    /// worked it for a polity that claims it, and knows the claim, holds a grievance against each
    /// household of another settlement seen there, the harm the food that household got there in
    /// days of their own household's need, and takes the day as evidence that the outsiders'
    /// polity harms theirs. Nobody of the outsiders' settlement is bound by the claim, and none of
    /// their choices reads it.
    pub(super) fn trespass(&mut self, ctx: &Ctx, meetings: &[Meeting]) {
        let day = ctx.now.day_index();
        let rp = &ctx.params.relations;
        for m in meetings {
            for pi in 0..self.polities.len() {
                let settlement = self.polities[pi].settlement;
                let Some(law) = self.polities[pi].claim_on(m.place) else {
                    continue;
                };
                // The outsiders there, by household, with the food each got there.
                let mut outsiders: Vec<(PermanentId, PermanentId, f64)> = Vec::new();
                for w in m.by.iter().filter(|w| w.settlement != settlement) {
                    match outsiders.iter_mut().find(|o| o.0 == w.household) {
                        Some(o) => o.2 += f64::from(w.kcal),
                        None => outsiders.push((w.household, w.settlement, f64::from(w.kcal))),
                    }
                }
                let mut theirs: Vec<PermanentId> = outsiders
                    .iter()
                    .filter_map(|o| self.polity_of(o.1).map(|i| self.polities[i].id))
                    .collect();
                theirs.sort_unstable();
                theirs.dedup();
                let mut witnesses: Vec<PermanentId> =
                    m.by.iter()
                        .filter(|w| w.settlement == settlement)
                        .filter_map(|w| w.person)
                        .collect();
                witnesses.sort_unstable();
                witnesses.dedup();
                for who in witnesses {
                    let knows = self.polities[pi]
                        .laws
                        .iter()
                        .any(|l| l.id == law && l.knows(who));
                    let Some(household) = self.person(who).map(|p| p.household) else {
                        continue;
                    };
                    if !knows {
                        continue;
                    }
                    let need = self.day_need(household, ctx.params).max(1.0);
                    for &(o, _, kcal) in &outsiders {
                        let blamed = Blamed::Household(o);
                        let issue = Grieved::Extraction;
                        self.grieve(ctx, who, issue, blamed, law, kcal / need, Wrong::Trespass);
                    }
                    for &polity in &theirs {
                        let seen = rp.seen_trespass;
                        self.polity_views
                            .record(who, polity, ViewAct::SawTrespass, seen, day, rp);
                    }
                }
            }
        }
    }

    /// `listener` was told for the first time of a grievance against household `blamed` raised by
    /// its people working a place the teller's polity claims (ADR-0020 §3): if that household
    /// lives in another settlement than the listener's, it is evidence that its polity harms
    /// theirs.
    pub(super) fn heard_of_trespass(&mut self, ctx: &Ctx, listener: PermanentId, blamed: Blamed) {
        let Blamed::Household(o) = blamed else {
            return;
        };
        let settlement_of =
            |p: &Population, h: PermanentId| p.household(h).and_then(|x| x.settlement);
        let mine = self
            .person(listener)
            .and_then(|p| settlement_of(self, p.household));
        let Some(theirs) = settlement_of(self, o) else {
            return;
        };
        if mine == Some(theirs) {
            return;
        }
        let Some(polity) = self.polity_of(theirs).map(|i| self.polities[i].id) else {
            return;
        };
        let rp = &ctx.params.relations;
        let day = ctx.now.day_index();
        self.polity_views.record(
            listener,
            polity,
            ViewAct::HeardTrespass,
            rp.heard_trespass,
            day,
            rp,
        );
    }
}
