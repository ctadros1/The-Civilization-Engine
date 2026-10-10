//! Relations between polities as people live them (M5c slice AT, ADR-0020 §3–§5): what those who
//! work a place their polity claims hold against people of another settlement they see working
//! it, and what they come to believe of that settlement's polity.

use super::*;
use crate::polity::LawStatus;
use crate::uses::{HeardClaim, Meeting, Place};
use crate::views::ViewAct;
use crate::word::{Blamed, Grieved, Wrong};

/// The purpose of the draws on whether word of a claim is passed on at the hearth.
pub const PURPOSE_CLAIM_TOLD: u64 = 0x636c_6169_6d74_6f6c; // "claimtol"

impl Population {
    /// `a` and `b` keep company at the hearth (M5c slice AU, ADR-0020 §2): each tells the other,
    /// with the content's chance, of a claim their polity's law in force makes that they know of,
    /// when the other lives elsewhere, and of another polity's claim their household heard of,
    /// unless it is the other's own polity's; the listener's household then holds every place the
    /// law claims. Word of a claim crosses no other way.
    pub(crate) fn share_claims(&mut self, ctx: &Ctx, a: PermanentId, b: PermanentId) {
        let home_of = |p: PermanentId| {
            let q = self.person(p)?;
            Some((q.household, self.household(q.household)?.settlement))
        };
        let (Some(ha), Some(hb)) = (home_of(a), home_of(b)) else {
            return;
        };
        let polity_id = |s: Option<PermanentId>| {
            s.and_then(|s| self.polity_of(s))
                .map(|i| self.polities[i].id)
        };
        let day = ctx.now.day_index();
        let mut learnt: Vec<(PermanentId, HeardClaim)> = Vec::new();
        for ((teller, (th, ts)), (listener, (lh, ls))) in [((a, ha), (b, hb)), ((b, hb), (a, ha))] {
            let theirs = polity_id(ls);
            // What the teller can tell of: their own polity's claims in force that they know,
            // when the listener lives elsewhere, and the claims their household heard of.
            let mut told: Vec<(PermanentId, PermanentId, Place)> = Vec::new();
            if ts.is_some()
                && ts != ls
                && let Some(p) = ts
                    .and_then(|s| self.polity_of(s))
                    .map(|i| &self.polities[i])
            {
                for &(law, place) in &p.claimed {
                    if p.laws
                        .iter()
                        .any(|l| l.id == law && l.status == LawStatus::InForce && l.knows(teller))
                    {
                        told.push((law, p.id, place));
                    }
                }
            }
            for c in self.claims_heard.of(th) {
                if Some(c.polity) != theirs {
                    told.push((c.law, c.polity, c.place));
                }
            }
            let mut laws: Vec<PermanentId> = told.iter().map(|t| t.0).collect();
            laws.sort_unstable();
            laws.dedup();
            for law in laws {
                if self.claims_heard.knows_law(lh, law) {
                    continue;
                }
                let key = [
                    ctx.seed,
                    PURPOSE_CLAIM_TOLD,
                    teller.get(),
                    listener.get(),
                    law.get(),
                    ctx.now.minutes() as u64,
                ];
                if Rng64::from_key(&key).next_f64() >= ctx.params.relations.share_claims {
                    continue;
                }
                for &(_, polity, place) in told.iter().filter(|t| t.0 == law) {
                    let claim = HeardClaim {
                        place,
                        law,
                        polity,
                        day,
                        from: teller,
                    };
                    learnt.push((lh, claim));
                }
            }
        }
        for (household, claim) in learnt {
            self.claims_heard.learn(household, claim);
        }
    }

    /// Lets go of claims heard of that no law in force makes now, and of households no more.
    pub(super) fn forget_ended_claims(&mut self) {
        let mut heard = std::mem::take(&mut self.claims_heard);
        heard.retain(|h, c| {
            self.hh_index.contains_key(&h)
                && self.polities.iter().any(|p| {
                    p.id == c.polity
                        && p.laws
                            .iter()
                            .any(|l| l.id == c.law && l.status == LawStatus::InForce)
                })
        });
        self.claims_heard = heard;
    }

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
                let claimant = self.polities[pi].id;
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
                    // Those of a polity they know an agreement gives leave are not trespassing
                    // (M5c slice AU).
                    let by_leave: Vec<bool> = outsiders
                        .iter()
                        .map(|o| {
                            self.polity_of(o.1).is_some_and(|i| {
                                self.leave_known(claimant, self.polities[i].id, pi, &[who])
                            })
                        })
                        .collect();
                    let need = self.day_need(household, ctx.params).max(1.0);
                    for (k, &(o, os, kcal)) in outsiders.iter().enumerate() {
                        if by_leave[k] {
                            continue;
                        }
                        // One grievance for each other settlement's people: one already held
                        // against a household of theirs is raised again, so the trespass of
                        // many households does not crowd out what else one holds.
                        let (blamed, under) = self
                            .word
                            .grievances_of(who)
                            .find(|g| {
                                g.wrong == Wrong::Trespass
                                    && matches!(g.blamed, Blamed::Household(h)
                                        if self.household(h).and_then(|x| x.settlement)
                                            == Some(os))
                            })
                            .map_or((Blamed::Household(o), law), |g| (g.blamed, g.law));
                        let issue = Grieved::Extraction;
                        self.grieve(ctx, who, issue, blamed, under, kcal / need, Wrong::Trespass);
                    }
                    for &polity in &theirs {
                        if self.leave_known(claimant, polity, pi, &[who]) {
                            continue;
                        }
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
