//! Word of mouth (M4c slice AE, ADR-0016 §3): a gathering is heard of by contact, never
//! broadcast. Its sponsor, or whoever brought its case, knows first; each midnight a household's
//! members tell one another what they heard, and companions at the hearth tell each other, each
//! by a keyed chance; only those who heard may come (research 09-16 §1.3: attendance and timing
//! decide who hears). What anyone has heard is their own record; nothing here reads another's
//! except to pass on what they could tell.

use super::*;
use crate::polity::{LawStatus, PolicyKind};
use crate::word::{Blamed, Claim, ClaimKind, Grievance, Grieved, Wrong};

/// Purpose tag for a household member telling another at midnight.
pub const PURPOSE_WORD_HOME: u64 = 0x776f_7264_686f_6d65; // "wordhome"
/// Purpose tag for a companion at the hearth telling another.
pub const PURPOSE_WORD_HEARTH: u64 = 0x776f_7264_6865_6172; // "wordhear"

/// What a draw about word of gathering or petition `c` is keyed by: its settlement, the day it
/// meets and its kind, so that other claims made or let go never move it (ADR-0016 §6).
fn gathering_key(c: &Claim) -> u64 {
    c.settlement.get() ^ (c.day as u64).rotate_left(40) ^ (u64::from(c.kind.code()) << 56)
}

/// Whether claim `c` is a call still ahead on `day`: a gathering, a petition or a refusal.
fn call_ahead(c: &Claim, day: i64) -> bool {
    matches!(
        c.kind,
        ClaimKind::Gathering | ClaimKind::Petition | ClaimKind::Refusal | ClaimKind::Revolt
    ) && c.day >= day
}

impl Population {
    /// Word that a gathering meets at `settlement` on `meets` (about law `law`, if called on one):
    /// the claim, made if it is new, and each of `first` hears it on `today` at first hand.
    pub(super) fn call_word(
        &mut self,
        settlement: PermanentId,
        meets: i64,
        law: Option<PermanentId>,
        first: &[PermanentId],
        today: i64,
    ) {
        let claim = match self.word.gathering(settlement, meets) {
            Some(c) => c,
            None => self.word.make(Claim {
                id: 0,
                kind: ClaimKind::Gathering,
                settlement,
                day: meets,
                subject: law,
                grievance: None,
            }),
        };
        for &p in first {
            self.word.hear(p, claim, today, None, Some(p));
        }
    }

    /// A save from before word of mouth (schema 34 and older): every adult had heard of a gathering
    /// called, so each gathering still to sit is heard of by every adult of its settlement, as of
    /// `now`.
    pub fn hear_of_gatherings_called(&mut self, now: SimTime, params: &PeopleParams) {
        let day = now.day_index();
        let called: Vec<(PermanentId, i64, Option<PermanentId>)> = self
            .polities
            .iter()
            .filter_map(|p| p.gathering.as_ref().map(|g| (p.settlement, g.day, g.law)))
            .collect();
        for (settlement, meets, law) in called {
            let adults: Vec<PermanentId> = self
                .members_of(settlement, now, params)
                .into_iter()
                .map(|m| m.0)
                .collect();
            self.call_word(settlement, meets, law, &adults, day);
        }
    }

    /// Word that a petition `petition` sits at `settlement` on `day`: the claim, made if it is
    /// new, and each of `first` hears it on `today` from `from`, who began it.
    pub(super) fn petition_word(
        &mut self,
        settlement: PermanentId,
        day: i64,
        petition: PermanentId,
        from: PermanentId,
        first: &[PermanentId],
        today: i64,
    ) {
        let claim = match self.word.petition(settlement, day) {
            Some(c) => c,
            None => self.word.make(Claim {
                id: 0,
                kind: ClaimKind::Petition,
                settlement,
                day,
                subject: Some(petition),
                grievance: None,
            }),
        };
        for &p in first {
            let told = (p != from).then_some(from);
            self.word.hear(p, claim, today, told, Some(from));
        }
    }

    /// Word of episode `subject` of kind `kind` (a refusal or a revolt) at `settlement`, standing
    /// until `until`: the claim, made if it is new, and each of `first` hears it on `today` from
    /// `from`, who began it.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn episode_word(
        &mut self,
        kind: ClaimKind,
        settlement: PermanentId,
        until: i64,
        subject: PermanentId,
        from: PermanentId,
        first: &[PermanentId],
        today: i64,
    ) {
        let claim = match self.word.episode(kind, subject) {
            Some(c) => c,
            None => self.word.make(Claim {
                id: 0,
                kind,
                settlement,
                day: until,
                subject: Some(subject),
                grievance: None,
            }),
        };
        for &p in first {
            let told = (p != from).then_some(from);
            self.word.hear(p, claim, today, told, Some(from));
        }
    }

    /// Whether `person` has heard of episode `subject` of kind `kind`.
    pub(crate) fn heard_of_episode(
        &self,
        person: PermanentId,
        kind: ClaimKind,
        subject: PermanentId,
    ) -> bool {
        self.word
            .episode(kind, subject)
            .is_some_and(|c| self.word.has_heard(person, c))
    }

    /// Whether `person` has heard that a petition sits at `settlement` on `day`.
    pub(crate) fn heard_of_petition(
        &self,
        person: PermanentId,
        settlement: PermanentId,
        day: i64,
    ) -> bool {
        self.word
            .petition(settlement, day)
            .is_some_and(|c| self.word.has_heard(person, c))
    }

    /// Whether `person` has heard that a gathering meets at `settlement` on `day`.
    pub(crate) fn heard_of_gathering(
        &self,
        person: PermanentId,
        settlement: PermanentId,
        day: i64,
    ) -> bool {
        self.word
            .gathering(settlement, day)
            .is_some_and(|c| self.word.has_heard(person, c))
    }

    /// Midnight (ADR-0016 §3): each member of a household tells each other member of a gathering
    /// ahead that they heard of, by the content's chance; then what is no longer news is let go,
    /// and so is what those who died or left held. Who tells is fixed at the start, so the order
    /// households are seen in changes nothing.
    pub(super) fn word_day(&mut self, ctx: &Ctx) {
        let wp = &ctx.params.word;
        let day = ctx.now.day_index();
        let index = &self.index;
        self.word.let_go_of_gone(|p| index.contains_key(&p));
        self.word.prune(day, i64::from(wp.news_days));
        // Each with the settlement and day it is keyed by: never the claim's number, which
        // unrelated claims move (ADR-0016 §6).
        let ahead: Vec<(u32, u64)> = self
            .word
            .claims
            .iter()
            .filter(|c| call_ahead(c, day))
            .map(|c| (c.id, gathering_key(c)))
            .collect();
        if ahead.is_empty() {
            return;
        }
        let mut homes: Vec<Vec<PermanentId>> = self
            .households
            .iter()
            .filter(|(_, x)| x.members.len() > 1)
            .map(|(_, x)| {
                let mut m = x.members.clone();
                m.sort_unstable();
                m
            })
            .collect();
        homes.sort_unstable();
        let mut told: Vec<(PermanentId, u32, PermanentId, Option<PermanentId>)> = Vec::new();
        for members in &homes {
            for &(claim, which) in &ahead {
                for &teller in members {
                    let Some(origin) = self
                        .word
                        .heard_by(teller)
                        .iter()
                        .find(|h| h.claim == claim)
                        .map(|h| h.origin)
                    else {
                        continue;
                    };
                    for &listener in members {
                        if listener == teller || self.word.has_heard(listener, claim) {
                            continue;
                        }
                        let key = [
                            ctx.seed,
                            PURPOSE_WORD_HOME,
                            teller.get(),
                            listener.get(),
                            which,
                            day as u64,
                        ];
                        if Rng64::from_key(&key).next_f64() < wp.share_home {
                            told.push((listener, claim, teller, origin));
                        }
                    }
                }
            }
        }
        for (listener, claim, teller, origin) in told {
            self.word.hear(listener, claim, day, Some(teller), origin);
        }
    }

    /// `a` and `b` keep company at the hearth: each tells the other of a gathering ahead that
    /// they heard of and the other has not, and of the grievances they hold keenly, each by the
    /// content's chance (ADR-0016 §3).
    pub(crate) fn share_word(&mut self, ctx: &Ctx, a: PermanentId, b: PermanentId) {
        let wp = &ctx.params.word;
        let day = ctx.now.day_index();
        let mut told: Vec<(PermanentId, u32, PermanentId, Option<PermanentId>)> = Vec::new();
        for (teller, listener) in [(a, b), (b, a)] {
            for h in self.word.heard_by(teller) {
                let Some(which) = self
                    .word
                    .claim(h.claim)
                    .filter(|c| call_ahead(c, day))
                    .map(gathering_key)
                else {
                    continue;
                };
                if self.word.has_heard(listener, h.claim) {
                    continue;
                }
                let key = [
                    ctx.seed,
                    PURPOSE_WORD_HEARTH,
                    teller.get(),
                    listener.get(),
                    which,
                    ctx.now.minutes() as u64,
                ];
                if Rng64::from_key(&key).next_f64() < wp.share_urgent {
                    told.push((listener, h.claim, teller, h.origin));
                }
            }
        }
        for (listener, claim, teller, origin) in told {
            self.word.hear(listener, claim, day, Some(teller), origin);
        }
        // And of the grievances each holds keenly.
        self.tell_grievances(ctx, a, b);
        self.tell_grievances(ctx, b, a);
    }

    /// `holder` holds a grievance (ADR-0016 §2): `harm_days` of their household's food, of `issue`,
    /// against `blamed`, breaking the terms of law `law`, raised by `wrong`. One they already hold
    /// against the same party under the same law is raised again (a reminder; for a harm that goes
    /// on, at most once in `remind_days`); otherwise a new one is held, the least keenly felt giving
    /// way beyond the content's number.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn grieve(
        &mut self,
        ctx: &Ctx,
        holder: PermanentId,
        issue: Grieved,
        blamed: Blamed,
        law: PermanentId,
        harm_days: f64,
        wrong: Wrong,
    ) {
        let wp = &ctx.params.word;
        let day = ctx.now.day_index();
        let felt = (harm_days / wp.full_harm_days.max(1e-6)).clamp(0.0, 1.0);
        let half = wp.half_life(issue);
        if let Some(g) =
            self.word.grievances.iter_mut().find(|g| {
                g.holder == holder && g.issue == issue && g.blamed == blamed && g.law == law
            })
        {
            if wrong == Wrong::StoreEmpty && day - g.raised < i64::from(wp.remind_days) {
                return;
            }
            let now = g.activation_on(day, half);
            g.activation = (now + felt).min(1.0) as f32;
            g.raised = day;
            g.harm_days += harm_days as f32;
            g.unresolved_days += harm_days as f32;
            g.wrong = wrong;
            return;
        }
        let at = self.word.grievances.partition_point(|g| g.holder <= holder);
        self.word.grievances.insert(
            at,
            Grievance {
                holder,
                issue,
                blamed,
                law,
                harm_days: harm_days as f32,
                unresolved_days: harm_days as f32,
                activation: felt as f32,
                raised: day,
                made: day,
                wrong,
            },
        );
        // Beyond the content's number, the least keenly felt gives way.
        let held: Vec<usize> = (0..self.word.grievances.len())
            .filter(|&i| self.word.grievances[i].holder == holder)
            .collect();
        if held.len() > wp.max_grievances as usize {
            let felt_now = |i: usize| {
                let g = &self.word.grievances[i];
                g.activation_on(day, wp.half_life(g.issue))
            };
            if let Some(&least) = held
                .iter()
                .min_by(|&&a, &&b| felt_now(a).total_cmp(&felt_now(b)).then(a.cmp(&b)))
            {
                self.word.grievances.remove(least);
            }
        }
    }

    /// The members of household `household` old enough to sit at a gathering.
    fn grown_of(
        &self,
        household: PermanentId,
        now: SimTime,
        params: &PeopleParams,
    ) -> Vec<PermanentId> {
        self.household(household).map_or_else(Vec::new, |x| {
            x.members
                .iter()
                .copied()
                .filter(|&m| {
                    self.person(m)
                        .is_some_and(|p| p.age_years(now) >= params.family.independent_age)
                })
                .collect()
        })
    }

    /// A household's food need a day, kcal.
    fn day_need(&self, household: PermanentId, params: &PeopleParams) -> f64 {
        self.household(household)
            .map_or(1, |x| x.members.len().max(1)) as f64
            * params.household.daily_kcal_per_person
    }

    /// Midnight (ADR-0016 §2): a household short of food that knows of a common store and finds it
    /// empty holds it against the store's keeper, or with none (or for the keeper themselves)
    /// against the gathering, under the store's law (its terms promise relief to those short).
    pub(super) fn grieve_empty_stores(&mut self, ctx: &Ctx) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let short = params.household.short_food_days;
        for pi in 0..self.polities.len() {
            let polity = &self.polities[pi];
            let Some(store) = polity
                .laws
                .iter()
                .find(|l| l.status == LawStatus::InForce && l.kind == PolicyKind::CommonStore)
            else {
                continue;
            };
            if stock_kcal(&polity.stores, goods) > 1.0 {
                continue;
            }
            let (store_law, relief_days) = (store.id, f64::from(store.relief_days));
            let body = Blamed::Body(polity.id);
            let (blamed, keeper) = polity
                .laws
                .iter()
                .find(|l| {
                    l.status == LawStatus::InForce
                        && l.kind == PolicyKind::KeepStore
                        && l.holder.is_some()
                })
                .map_or((body, None), |l| (Blamed::Office(l.id), l.holder));
            let settlement = polity.settlement;
            let mut homes: Vec<(PermanentId, f64)> = self
                .households
                .iter()
                .filter(|(_, x)| x.settlement == Some(settlement) && !x.members.is_empty())
                .map(|(_, x)| {
                    let need = x.members.len() as f64 * params.household.daily_kcal_per_person;
                    let days = stock_kcal(&stores_now(x, now, params, goods), goods) / need;
                    (x.id, days)
                })
                .filter(|&(_, days)| days < short)
                .collect();
            homes.sort_unstable_by_key(|h| h.0);
            for (hh, days) in homes {
                let harm = relief_days
                    .min(params.household.food_target_days - days)
                    .max(0.0);
                if harm <= 0.0 {
                    continue;
                }
                for m in self.grown_of(hh, now, params) {
                    if self.polities[pi]
                        .laws
                        .iter()
                        .any(|l| l.id == store_law && l.knows(m))
                    {
                        // The keeper does not blame their own keeping (M4c slice AH).
                        let blamed = if keeper == Some(m) { body } else { blamed };
                        self.grieve(
                            ctx,
                            m,
                            Grieved::Subsistence,
                            blamed,
                            store_law,
                            harm,
                            Wrong::StoreEmpty,
                        );
                    }
                }
            }
        }
    }

    /// Relief of `days` of food came to household `household` from the store of law `store_law`:
    /// its members' grievances over that store, its emptiness or its levy, are redressed by as
    /// much (the store kept its promise to relieve them), and settled when nothing is left
    /// unresolved.
    pub(super) fn redress_store(
        &mut self,
        household: PermanentId,
        store_law: PermanentId,
        days: f64,
    ) {
        let members = self
            .household(household)
            .map(|x| x.members.clone())
            .unwrap_or_default();
        for g in &mut self.word.grievances {
            if matches!(g.issue, Grieved::Subsistence | Grieved::Extraction)
                && g.law == store_law
                && members.contains(&g.holder)
            {
                g.unresolved_days = (f64::from(g.unresolved_days) - days).max(0.0) as f32;
            }
        }
        self.word
            .grievances
            .retain(|g| !(g.law == store_law && g.unresolved_days <= 0.0));
    }

    /// A gathering of polity `pi` decided case `c` (ADR-0016 §2): one that found against a member
    /// of a household is held against the gathering by its grown members who know the law and do
    /// not believe they took, for what it imposed; one that did not find, or was not heard, by the
    /// grown members of the household taken from who know the law and believe the accused took,
    /// for what they lost.
    pub(super) fn grieve_case(
        &mut self,
        ctx: &Ctx,
        pi: usize,
        c: &crate::crime::Case,
        stage: crate::crime::CaseStage,
        imposed_kcal: f64,
        exiled: bool,
    ) {
        use crate::crime::CaseStage;
        let (now, params) = (ctx.now, ctx.params);
        let body = Blamed::Body(self.polities[pi].id);
        let knows = |pop: &Population, p: PermanentId| {
            pop.polities[pi]
                .laws
                .iter()
                .any(|l| l.id == c.law && l.knows(p))
        };
        let believes = |pop: &Population, p: PermanentId| {
            pop.order
                .belief(p, c.incident)
                .is_some_and(|b| b.taker == Some(c.accused))
        };
        let (household, issue, wrong) = match stage {
            CaseStage::Found => (c.accused_household, Grieved::Treatment, Wrong::FoundAgainst),
            CaseStage::NotFound => (c.accuser, Grieved::Collective, Wrong::NotFound),
            CaseStage::Unheard => (c.accuser, Grieved::Collective, Wrong::Unheard),
            _ => return,
        };
        let need = self.day_need(household, params);
        let harm = if stage == CaseStage::Found {
            imposed_kcal / need + if exiled { params.crime.exile_days } else { 0.0 }
        } else {
            f64::from(c.kcal) / need
        };
        if harm <= 0.0 {
            return;
        }
        for m in self.grown_of(household, now, params) {
            if m == c.accused || !knows(self, m) {
                continue;
            }
            let wronged = if stage == CaseStage::Found {
                !believes(self, m)
            } else {
                believes(self, m)
            };
            if wronged {
                self.grieve(ctx, m, issue, body, c.law, harm, wrong);
            }
        }
    }

    /// A levy under law `law` of polity `pi`, taken in a lean year, took food that household
    /// `household` needs: `short_kcal` of what it paid leaves it below a year of its members' food
    /// (ADR-0016 §2; research 04-10 §1.1: a levy breaks the store's protective promise when it
    /// takes from those it should relieve in a hungry year, as a burden alone does not). Its grown
    /// members who know the law hold it against the gathering.
    pub(super) fn grieve_levy(
        &mut self,
        ctx: &Ctx,
        pi: usize,
        law: PermanentId,
        household: PermanentId,
        short_kcal: f64,
    ) {
        let (now, params) = (ctx.now, ctx.params);
        let harm = short_kcal / self.day_need(household, params);
        if harm <= 0.0 {
            return;
        }
        let body = Blamed::Body(self.polities[pi].id);
        for m in self.grown_of(household, now, params) {
            if self.polities[pi]
                .laws
                .iter()
                .any(|l| l.id == law && l.knows(m))
            {
                self.grieve(
                    ctx,
                    m,
                    Grieved::Extraction,
                    body,
                    law,
                    harm,
                    Wrong::LeanLevy,
                );
            }
        }
    }

    /// Household `debtor` refused what a finding under law `law` imposed for household
    /// `beneficiary`, `left_kcal` of it unpaid (ADR-0016 §2): the beneficiary's grown members who
    /// know the law hold it against the debtor's household. One that could not pay is not blamed.
    pub(super) fn grieve_refusal(
        &mut self,
        ctx: &Ctx,
        law: PermanentId,
        debtor: PermanentId,
        beneficiary: PermanentId,
        left_kcal: f64,
    ) {
        let (now, params) = (ctx.now, ctx.params);
        let harm = left_kcal / self.day_need(beneficiary, params);
        if harm <= 0.0 {
            return;
        }
        for m in self.grown_of(beneficiary, now, params) {
            let knows = self
                .polities
                .iter()
                .flat_map(|p| &p.laws)
                .any(|l| l.id == law && l.knows(m));
            if knows {
                self.grieve(
                    ctx,
                    m,
                    Grieved::Collective,
                    Blamed::Household(debtor),
                    law,
                    harm,
                    Wrong::Unpaid,
                );
            }
        }
    }

    /// `teller` tells `listener` at the hearth of a grievance they hold keenly enough, by the
    /// content's chance (ADR-0016 §3): the claim that they hold it, made if it is new, is heard,
    /// and a listener who holds one against the same party over the same issue is reminded of it.
    fn tell_grievances(&mut self, ctx: &Ctx, teller: PermanentId, listener: PermanentId) {
        let wp = &ctx.params.word;
        let day = ctx.now.day_index();
        let Some(settlement) = self
            .person(teller)
            .and_then(|p| self.household(p.household))
            .and_then(|x| x.settlement)
        else {
            return;
        };
        let keen: Vec<(Blamed, Grieved)> = self
            .word
            .grievances_of(teller)
            .filter(|g| g.activation_on(day, wp.half_life(g.issue)) >= wp.tell_floor)
            .map(|g| (g.blamed, g.issue))
            .collect();
        for (blamed, issue) in keen {
            let key = [
                ctx.seed,
                PURPOSE_WORD_HEARTH,
                teller.get(),
                listener.get(),
                u64::from(blamed.to_raw().0) << 8 | u64::from(issue.code()),
                blamed.to_raw().1,
                ctx.now.minutes() as u64,
            ];
            if Rng64::from_key(&key).next_f64() >= wp.share_routine {
                continue;
            }
            let claim = self
                .word
                .claims
                .iter()
                .find(|c| {
                    c.kind == ClaimKind::Grievance
                        && c.subject == Some(teller)
                        && c.grievance == Some((blamed, issue))
                })
                .map(|c| c.id);
            let claim = claim.unwrap_or_else(|| {
                self.word.make(Claim {
                    id: 0,
                    kind: ClaimKind::Grievance,
                    settlement,
                    day,
                    subject: Some(teller),
                    grievance: Some((blamed, issue)),
                })
            });
            self.word.hear(teller, claim, day, None, Some(teller));
            self.word
                .hear(listener, claim, day, Some(teller), Some(teller));
            // Hearing one's own grievance told reminds one of it.
            if let Some(g) = self
                .word
                .grievances
                .iter_mut()
                .find(|g| g.holder == listener && g.blamed == blamed && g.issue == issue)
            {
                let now = g.activation_on(day, wp.half_life(issue));
                g.activation = (now + wp.reminder).min(1.0) as f32;
                g.raised = day;
            }
        }
    }
}
