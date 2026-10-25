//! Inoculation (M10c slice ED, second part; ADR-0035 §4; brief m10-services §2.3; research 05-05
//! §2.1): a disease that may be given on purpose names the technique that gives it, and how much
//! milder a course so given is (variolation: Jurin's 2.1% died after inoculation against 16.6% of
//! natural smallpox). While it runs anywhere, a household that believes it near (someone its
//! members have ties or kin with died of it within the sickness spell: the cause of a death is
//! known to those close to the dead) and weighs the inoculated course's deaths below the chance
//! it believes its people would take it, with members it does not protect, calls someone it knows
//! who knows the technique, kin asking nothing and others the call's fee. They come and inoculate
//! those members at a session at its home, each taking a mild course of it that sheds and can pass
//! as any.

use super::*;
use crate::decide::{CallFacts, TendFacts};
use crate::params::InoculationDef;
use crate::sickness::Acquired;

/// How long a call to inoculate stands unanswered, days (a design prior: a fortnight).
const INOCULATION_CALL_DAYS: i64 = 14;

impl Population {
    /// The diseases that may be given on purpose and run now, with how (by index).
    fn inoculable(&self, ctx: &Ctx) -> Vec<(u16, InoculationDef)> {
        let mut out: Vec<(u16, InoculationDef)> = Vec::new();
        for &i in self.sickness.running() {
            let Some(e) = self.sickness.get(i) else {
                continue;
            };
            let Some(inoc) = ctx
                .catalog
                .diseases
                .get(usize::from(e.disease))
                .and_then(|d| d.inoculation)
            else {
                continue;
            };
            if !out.iter().any(|o| o.0 == e.disease) {
                out.push((e.disease, inoc));
            }
        }
        out.sort_unstable_by_key(|o| o.0);
        out
    }

    /// Whether household `x` believes `disease` near on `day`: someone close kin of, or tied to,
    /// one of its members died of it within the sickness spell.
    fn believes_near(&self, ctx: &Ctx, x: &Household, disease: u16, day: i64) -> bool {
        let spell = i64::from(ctx.params.word.sickness_spell_days);
        // An infection that ended in the spell began within some months of it.
        let since = day - spell - 120;
        let dead: Vec<PermanentId> = self
            .sickness
            .episodes()
            .iter()
            .rev()
            .take_while(|e| e.infected >= since)
            .filter(|e| e.disease == disease)
            .filter(|e| {
                matches!(e.ended, Some((d, crate::sickness::Outcome::Died)) if day - d <= spell)
            })
            .map(|e| e.person)
            .collect();
        if dead.is_empty() {
            return false;
        }
        x.members.iter().any(|&m| {
            self.close_kin(m).iter().any(|k| dead.contains(k))
                || self.ties.of(m).iter().any(|t| dead.contains(&t.to))
        })
    }

    /// The members of household `x` to inoculate against `disease` on `day`: not protected by it
    /// (taking it now, or protected by one lived through) and not ill.
    fn to_inoculate(&self, x: &Household, disease: u16, day: i64) -> Vec<PermanentId> {
        x.members
            .iter()
            .copied()
            .filter(|&m| !self.sickness.protected(m, disease, day) && !self.sickness.ill(m, day))
            .collect()
    }

    /// The disease household `x` would have its people inoculated against on `day`, if any, and
    /// how many of them: one it believes near, whose inoculated course's deaths it weighs below
    /// the chance it believes they would take it, with members to inoculate.
    fn inoculation_wanted(
        &self,
        ctx: &Ctx,
        x: &Household,
        day: i64,
    ) -> Option<(u16, InoculationDef, usize)> {
        self.inoculable(ctx).into_iter().find_map(|(d, inoc)| {
            if inoc.deaths >= inoc.believed_take || !self.believes_near(ctx, x, d, day) {
                return None;
            }
            let n = self.to_inoculate(x, d, day).len();
            (n > 0).then_some((d, inoc, n))
        })
    }

    /// Whom person `who` of household `x` would call to inoculate its people (M10c slice ED,
    /// second part): where it wants them inoculated and has no such call standing, the household
    /// of kin, asking nothing, then of someone its members have ties with, for the call's fee,
    /// where someone able knows the technique; the nearest, as for a carer.
    pub(super) fn inoculation_call_facts(
        &self,
        ctx: &Ctx,
        who: PermanentId,
        x: &Household,
        field_key: PermanentId,
    ) -> Option<CallFacts> {
        if self
            .sickness
            .inoculation_calls
            .iter()
            .any(|c| c.caller == x.id)
        {
            return None;
        }
        let day = ctx.now.day_index();
        let (_, inoc, n) = self.inoculation_wanted(ctx, x, day)?;
        let knows = |h: &PermanentId| {
            self.household(*h).is_some_and(|y| {
                y.members.iter().any(|&m| {
                    !self.sickness.ill(m, day)
                        && self.person(m).is_some_and(|p| p.knows(inoc.technique))
                })
            })
        };
        let mut kin: Vec<PermanentId> = x
            .members
            .iter()
            .flat_map(|&m| self.close_kin(m))
            .filter_map(|k| self.person(k).map(|q| q.household))
            .filter(|h| *h != x.id && knows(h))
            .collect();
        kin.sort_unstable();
        kin.dedup();
        let mut tied = self.tie_households(who, x);
        tied.retain(|h| *h != x.id && knows(h));
        let any = |_: &Household| true;
        self.choose_carer(ctx, x, field_key, &kin, &tied, &any, n as f64, false)
    }

    /// Someone of household `caller` left a call to inoculate its people at household `carer`'s
    /// door, where it still wants them inoculated (M10c slice ED, second part): kin asking
    /// nothing, others the call's fee. Returns whether it was left.
    pub(super) fn call_inoculator(
        &mut self,
        ctx: &Ctx,
        caller: PermanentId,
        carer: PermanentId,
    ) -> bool {
        let day = ctx.now.day_index();
        let (Some(x), Some(y)) = (self.household(caller), self.household(carer)) else {
            return false;
        };
        if self
            .sickness
            .inoculation_calls
            .iter()
            .any(|c| c.caller == caller)
            || self.inoculation_wanted(ctx, x, day).is_none()
        {
            return false;
        }
        let kin = x
            .members
            .iter()
            .any(|&m| self.close_kin(m).iter().any(|k| y.members.contains(k)));
        let fee_kcal = if kin {
            0.0
        } else {
            self.call_fee(ctx, x) as f32
        };
        self.sickness
            .inoculation_calls
            .push(crate::sickness::CareCall {
                caller,
                carer,
                fee_kcal,
                day,
                paid: -1,
                birth: false,
            });
        true
    }

    /// The household person `who` of household `x` would go to inoculate (M10c slice ED, second
    /// part): one that called theirs, where they know the technique of a disease its people want
    /// inoculation against, with as many to inoculate as `owed`; the nearest of the most.
    pub(super) fn inoculation_tend(
        &self,
        ctx: &Ctx,
        who: PermanentId,
        x: &Household,
        field_key: PermanentId,
    ) -> Option<TendFacts> {
        if self.sickness.inoculation_calls.is_empty() {
            return None;
        }
        let day = ctx.now.day_index();
        let p = self.person(who)?;
        let reach = &self.homes.get(&field_key)?.reach;
        let mut best: Option<TendFacts> = None;
        for c in self
            .sickness
            .inoculation_calls
            .iter()
            .filter(|c| c.carer == x.id)
        {
            let Some(y) = self.household(c.caller) else {
                continue;
            };
            let Some((d, inoc, _)) = self.inoculation_wanted(ctx, y, day) else {
                continue;
            };
            if !p.knows(inoc.technique) {
                continue;
            }
            let owed = self.to_inoculate(y, d, day).len() as f64;
            let Some(secs) = reach.seconds_to(cell_of(ctx.map, y.home)) else {
                continue;
            };
            let walk_min = f64::from(secs) / 60.0;
            if best.is_none_or(|b| owed > b.owed || (owed == b.owed && walk_min < b.walk_min)) {
                best = Some(TendFacts {
                    owed,
                    household: Some(y.id),
                    at: y.home,
                    walk_min,
                });
            }
        }
        best
    }

    /// Person `who` came to household `household` at activity `def` for `minutes` (M10c slice
    /// ED, second part): where it called theirs to inoculate its people and they know the
    /// technique, each of its members to inoculate takes a mild course of the disease from them,
    /// the call's fee is paid and the call answered. Returns whether they inoculated.
    pub(super) fn inoculate_at(
        &mut self,
        ctx: &mut Ctx,
        who: PermanentId,
        household: PermanentId,
        def: u16,
        minutes: u32,
    ) -> bool {
        let day = ctx.now.day_index();
        let Some(own) = self.person(who).map(|p| p.household) else {
            return false;
        };
        let Some(i) = self
            .sickness
            .inoculation_calls
            .iter()
            .position(|c| c.caller == household && c.carer == own)
        else {
            return false;
        };
        let Some(x) = self.household(household) else {
            return false;
        };
        let Some((d, inoc, _)) = self.inoculation_wanted(ctx, x, day) else {
            return false;
        };
        if !self.person(who).is_some_and(|p| p.knows(inoc.technique)) {
            return false;
        }
        let members = self.to_inoculate(x, d, day);
        let call = self.sickness.inoculation_calls.remove(i);
        for m in members {
            if m != who {
                self.infect(ctx, m, d, day, Acquired::Inoculated { by: who });
            }
        }
        let hours = f64::from(minutes) / 60.0;
        self.practise(ctx, who, inoc.technique, def, Target::Home, hours);
        let fee = f64::from(call.fee_kcal);
        if fee > 0.0 {
            self.pay_food(ctx, household, own, fee);
        }
        true
    }

    /// Calls to inoculate that are past their days, or whose household no longer wants it or is
    /// gone, lapse.
    pub(super) fn lapse_inoculation_calls(&mut self, ctx: &Ctx, day: i64) {
        if self.sickness.inoculation_calls.is_empty() {
            return;
        }
        let calls = std::mem::take(&mut self.sickness.inoculation_calls);
        let kept = calls
            .into_iter()
            .filter(|c| {
                day - c.day <= INOCULATION_CALL_DAYS
                    && self.household(c.carer).is_some()
                    && self
                        .household(c.caller)
                        .is_some_and(|x| self.inoculation_wanted(ctx, x, day).is_some())
            })
            .collect();
        self.sickness.inoculation_calls = kept;
    }

    /// Whom person `who` of household `h` would call to inoculate its people now: for tests.
    #[doc(hidden)]
    pub fn inoculation_call_for_tests(
        &mut self,
        ctx: &Ctx,
        who: PermanentId,
        h: PermanentId,
    ) -> Option<PermanentId> {
        self.refresh_kin();
        let x = self.household(h)?;
        let key = x
            .settlement
            .filter(|s| self.homes.contains_key(s))
            .unwrap_or(x.id);
        self.inoculation_call_facts(ctx, who, x, key)
            .map(|c| c.carer)
    }
}
