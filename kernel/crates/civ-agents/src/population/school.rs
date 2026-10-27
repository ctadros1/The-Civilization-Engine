//! Schools by law (M10c slice EB, third part; ADR-0035 §5; brief m10-services §3.2): where a
//! household of a polity found someone to teach a child of its a technique it lacked but could not
//! hold the fee, within the year, and someone grown of the settlement knows that technique, anyone
//! may propose that the polity keep a school. Passed, it names a schoolteacher, an office, who
//! takes the children of its households to learn what they know without a fee; where work is
//! paid by law, the store pays their household for the hours they teach them.

use super::*;
use crate::knowledge::{Enrolment, Session, Untaught};

impl Population {
    /// Household `h` could not hold the fee to send a child to learn technique `t` today.
    pub(super) fn note_untaught(&mut self, h: PermanentId, t: u16, day: i64) {
        self.untaught.push(Untaught {
            household: h,
            technique: t,
            day,
        });
    }

    /// Records older than a year of `day` are forgotten.
    pub(super) fn prune_untaught(&mut self, day: i64) {
        if self.untaught.is_empty() {
            return;
        }
        let since = day - DAYS_PER_YEAR;
        self.untaught.retain(|u| u.day > since);
    }

    /// The technique the most of `households` could not send a child to learn
    /// within the year that someone of `adults` knows, if any: the lower index of two alike.
    pub(super) fn school_technique(
        &self,
        ctx: &Ctx,
        households: &[PermanentId],
        adults: &[(PermanentId, PermanentId)],
    ) -> Option<usize> {
        if self.untaught.is_empty() {
            return None;
        }
        let since = ctx.now.day_index() - DAYS_PER_YEAR;
        let mut wanted: BTreeMap<u16, Vec<PermanentId>> = BTreeMap::new();
        for u in self
            .untaught
            .iter()
            .filter(|u| u.day > since && households.contains(&u.household))
        {
            let hs = wanted.entry(u.technique).or_default();
            if !hs.contains(&u.household) {
                hs.push(u.household);
            }
        }
        wanted
            .into_iter()
            .filter(|(t, _)| {
                adults
                    .iter()
                    .any(|&(c, _)| self.person(c).is_some_and(|p| p.knows(usize::from(*t))))
            })
            .max_by(|a, b| a.1.len().cmp(&b.1.len()).then(b.0.cmp(&a.0)))
            .map(|(t, _)| usize::from(t))
    }

    /// What a school would spare each of `households`, kcal (M10c slice EB, third
    /// part): for each technique it could not send a child to learn within the year that someone
    /// of `adults` knows, the fee it was asked for a day of study, a day of care, over the days
    /// learning it takes in sessions.
    pub(super) fn schooling(
        &self,
        ctx: &Ctx,
        households: &[PermanentId],
        adults: &[(PermanentId, PermanentId)],
    ) -> Vec<(PermanentId, f64)> {
        if self.untaught.is_empty() {
            return Vec::new();
        }
        let since = ctx.now.day_index() - DAYS_PER_YEAR;
        let session_h = f64::from(super::knowledge::SESSION_MIN) / 60.0;
        let mut out: Vec<(PermanentId, f64)> = Vec::new();
        for &h in households {
            let Some(x) = self.household(h) else {
                continue;
            };
            let mut seen: Vec<u16> = Vec::new();
            for u in self
                .untaught
                .iter()
                .filter(|u| u.household == h && u.day > since)
            {
                if seen.contains(&u.technique) {
                    continue;
                }
                seen.push(u.technique);
            }
            let saved: f64 = seen
                .into_iter()
                .filter(|&t| {
                    adults
                        .iter()
                        .any(|&(c, _)| self.person(c).is_some_and(|p| p.knows(usize::from(t))))
                })
                .filter_map(|t| ctx.catalog.techniques.get(usize::from(t)))
                .map(|d| self.call_fee(ctx, x) * d.learn_h / session_h)
                .sum();
            if saved > 0.0 {
                out.push((h, saved));
            }
        }
        out
    }

    /// The schoolteacher who would take a child of household `h` to learn technique `t` without
    /// a fee: the one its settlement's polity names to keep the school, if they know it and are
    /// not of `h`.
    pub(super) fn school_for(&self, h: PermanentId, t: usize) -> Option<PermanentId> {
        let settlement = self.household(h)?.settlement?;
        let pi = self.polity_of(settlement)?;
        let (holder, _) = self.polities[pi].schoolteacher()?;
        self.person(holder)
            .is_some_and(|p| p.knows(t) && p.household != h)
            .then_some(holder)
    }

    /// The hours each schoolteacher taught yesterday to learners of their polity's households
    /// enrolled with them without a fee are paid for from its store, where a law pays common
    /// work (M10c slice EB, third part): `taught` and `studied` are yesterday's sessions, `list`
    /// the enrolments as they stood.
    pub(super) fn pay_school(
        &mut self,
        ctx: &Ctx,
        taught: &[Session],
        studied: &[Session],
        list: &[Enrolment],
    ) {
        let mut paid: Vec<(PermanentId, PermanentId, f64)> = Vec::new();
        for s in taught {
            let Some(household) = self.person(s.person).map(|p| p.household) else {
                continue;
            };
            let Some(polity) = self.polities.iter().find(|p| {
                p.schoolteacher()
                    .is_some_and(|(holder, _)| holder == s.person)
                    && list.iter().any(|e| {
                        e.teacher == s.person
                            && e.technique == s.technique
                            && e.fee_kcal <= 0.0
                            && self
                                .household(e.payer)
                                .is_some_and(|x| p.holds(x.settlement))
                            && crate::knowledge::tallied(studied, e.learner, s.technique) > 0.0
                    })
            }) else {
                continue;
            };
            paid.push((polity.id, household, f64::from(s.minutes) / 60.0));
        }
        for (polity, household, hours) in paid {
            self.pay_for_work(ctx, polity, household, hours);
        }
    }

    /// Whether polity `pi`'s settlement has the issue `untaught` now: for tests.
    #[doc(hidden)]
    pub fn untaught_for_tests(&self, ctx: &Ctx, pi: usize) -> bool {
        let settlement = self.polities[pi].settlement;
        let mut households: Vec<PermanentId> = self
            .households
            .iter()
            .filter(|(_, x)| x.settlement == Some(settlement))
            .map(|(_, x)| x.id)
            .collect();
        households.sort_unstable();
        let adults = self.members_of(settlement, ctx.now, ctx.params);
        self.school_technique(ctx, &households, &adults).is_some()
    }
}
