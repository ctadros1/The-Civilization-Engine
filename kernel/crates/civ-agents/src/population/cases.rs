//! Cases before the gathering (M4b slice AB, ADR-0015 §4-§5): a household taken from brings an
//! accusation under a law against taking; the gathering hears it as it decides a law, those who
//! come standing by what they believe, what their household stands to gain or lose, and their
//! regard for each party; a finding imposes the law's bundle as obligations. What the gathering
//! knows is what was brought before it: the household's account of its loss and the witnesses
//! behind it. A case never reads the incident it is about, and a wrong decision is never
//! repaired.

use super::*;
use crate::crime::{
    Belief, Case, CaseStage, CaseStance, Obligation, Owed, Source, Standing, belief_points,
    case_strength, comply_points, logistic,
};
use crate::forum::{ForumDef, ForumSeat};
use crate::polity::{
    Gathering, LawStatus, Outcome as Decided, PolicyDef, PolicyKind, Sanction, Stance,
};

/// Purpose tag for the lot that draws a forum's panel for a sitting (M7b slice BR).
pub const PURPOSE_PANEL: u64 = 0x7061_6e65_6c6c_6f74; // "panellot"
/// Purpose tag for whether an accused's household offers to settle a case (M7b slice BR).
pub const PURPOSE_SETTLE: u64 = 0x7365_7474_6c65_3031; // "settle01"

/// `s` with its first letter in lower case: "the hearer Ada".
fn lower_first(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map(|f| f.to_lowercase().collect::<String>() + c.as_str())
        .unwrap_or_default()
}

/// Law `l` as the move that would make it now (M7b slice BT): what a household weighs keeping it
/// or ending it by.
fn law_move(l: &crate::polity::Law) -> crate::polity::MoveOption {
    crate::polity::MoveOption {
        policy: l.policy,
        levy_share: f64::from(l.levy_share),
        issue: l.issue,
        nominee: l.holder,
        sanction: l.sanction,
        hours: l.hours,
        body: None,
        ends: None,
        target: l.target,
        site: None,
        own_gain: 0.0,
        followers_gain: 0.0,
        support: 0.5,
    }
}

/// "Bram", "Bram and Ada", "Bram, Ada and 2 others".
fn names(list: &[String]) -> String {
    match list {
        [] => "nobody".to_owned(),
        [a] => a.clone(),
        [a, b] => format!("{a} and {b}"),
        [a, b, rest @ ..] => {
            let n = rest.len();
            let others = if n == 1 {
                "1 other".to_owned()
            } else {
                format!("{n} others")
            };
            format!("{a}, {b} and {others}")
        }
    }
}

impl Population {
    /// The law against taking in force where household `household` lives, if `by` knows of it:
    /// its id and bundle (research 09-07 §4.1: knowing of a forum is one of the barriers).
    pub(super) fn law_against_taking(
        &self,
        ctx: &Ctx,
        household: PermanentId,
        by: PermanentId,
    ) -> Option<(PermanentId, Sanction)> {
        let settlement = self.household(household)?.settlement?;
        let polity = &self.polities[self.polity_of(settlement)?];
        polity
            .in_force(&ctx.catalog.policies, PolicyKind::AgainstTaking)
            .find(|l| l.knows(by))
            .map(|l| (l.id, l.sanction))
    }

    /// The words for household `household`: "the household of Rilla".
    fn household_words(&self, ctx: &Ctx, household: PermanentId) -> String {
        self.elder_of(household, ctx.now, ctx.params).map_or_else(
            || "a household".to_owned(),
            |e| format!("the household of {}", self.name_of(e)),
        )
    }

    /// `by` brings a case for household `household` that `accused` took `kcal` of food from it in
    /// incident `incident`, under law `law` (ADR-0015 §4). The case holds the witnesses the
    /// household's accounts come from; it goes before the gathering already called, or one called
    /// for it.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn bring_case(
        &mut self,
        ctx: &Ctx,
        incident: u32,
        household: PermanentId,
        by: PermanentId,
        accused: PermanentId,
        kcal: f64,
        law: PermanentId,
    ) {
        let Some(settlement) = self.household(household).and_then(|x| x.settlement) else {
            return;
        };
        self.bring_case_at(ctx, incident, household, by, accused, kcal, law, settlement);
    }

    /// As [`Population::bring_case`], before the gathering of settlement `settlement`, which may
    /// be another than the household's (a raid its own people made, M6c slice BJ).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn bring_case_at(
        &mut self,
        ctx: &Ctx,
        incident: u32,
        household: PermanentId,
        by: PermanentId,
        accused: PermanentId,
        kcal: f64,
        law: PermanentId,
        settlement: PermanentId,
    ) {
        let (now, params) = (ctx.now, ctx.params);
        let day = now.day_index();
        let Some(pi) = self.polity_of(settlement) else {
            return;
        };
        let Some(accused_household) = self.person(accused).map(|p| p.household) else {
            return;
        };
        // The witnesses behind it: what the household knows, and what the one who brings it saw
        // (a watcher who brings what they saw, M4b slice AC).
        let mut members = self
            .household(household)
            .map(|x| x.members.clone())
            .unwrap_or_default();
        let member = members.contains(&by);
        if !member {
            members.push(by);
        }
        let leads = self.order.leads(&members, incident, accused);
        let id = self.order.cases.last().map_or(1, |c| c.id + 1);
        self.order.cases.push(Case {
            id,
            incident,
            settlement,
            law,
            accuser: household,
            by,
            accused,
            accused_household,
            kcal: kcal as f32,
            leads: leads.clone(),
            opened: day,
            stage: CaseStage::Open,
            heard: None,
            eligible: 0,
            stances: Vec::new(),
            exiled: false,
            forum: None,
            challenge: None,
            tags: 0,
            departed: None,
        });
        self.put_cases(&ctx.catalog.policies, pi, day, params.polity.notice_days);
        // Before a forum, if one hears it (M7b slice BR), else before the gathering.
        let before = self
            .order
            .cases
            .last()
            .and_then(|c| c.forum)
            .and_then(|_| self.polities[pi].forum(&ctx.catalog.policies))
            .map_or_else(
                || "the gathering".to_owned(),
                |(l, f)| {
                    let holder = l.holder.map(|h| self.name_of(h));
                    let w = crate::forum::hearers_words(f, holder.as_deref());
                    let mut c = w.chars();
                    c.next()
                        .map(|f| f.to_lowercase().collect::<String>() + c.as_str())
                        .unwrap_or_default()
                },
            );
        let (place, name) = ctx
            .land
            .settlements
            .iter()
            .find(|s| s.id == settlement)
            .map_or((None, String::new()), |s| {
                (Some(s.hearth_m), s.name.clone())
            });
        let others: Vec<String> = leads
            .iter()
            .filter(|&&w| w != by)
            .map(|&w| self.name_of(w))
            .collect();
        let saw = match (leads.contains(&by), others.is_empty()) {
            (true, true) => "they saw it themselves".to_owned(),
            (true, false) => format!("they saw it themselves, as did {}", names(&others)),
            (false, _) => format!("{} saw it", names(&others)),
        };
        let whose = if member {
            "their household".to_owned()
        } else {
            self.household_words(ctx, household)
        };
        let words = format!(
            "brought a case before {before} at {name}: that {} took food from {whose}; {saw}.",
            self.name_of(accused)
        );
        self.chronicle_push(
            now,
            ChronicleKind::CaseBrought,
            vec![by, accused],
            Some(settlement),
            place,
            leads.len() as f64,
            words,
        );
    }

    /// Puts polity `pi`'s open cases that no gathering is to hear before its forum, if one hears
    /// them (M7b slice BR, ADR-0025 §1: a holder never hears a case their household is party
    /// to), else before the gathering called, or calls one `notice_days` on for them.
    pub(super) fn put_cases(
        &mut self,
        policies: &[PolicyDef],
        pi: usize,
        day: i64,
        notice_days: u32,
    ) {
        let settlement = self.polities[pi].settlement;
        if let Some((law, def)) = self.polities[pi].forum(policies) {
            let holder = law.holder.map(|h| (h, self.person(h).map(|p| p.household)));
            let on = self.polities[pi]
                .gathering
                .as_ref()
                .map(|g| g.cases.clone())
                .unwrap_or_default();
            let laws = &self.polities[pi].laws;
            for c in self.order.cases.iter_mut().filter(|c| {
                c.settlement == settlement
                    && c.stage == CaseStage::Open
                    && c.forum.is_none()
                    && !on.contains(&c.id)
            }) {
                let heard = laws
                    .iter()
                    .find(|l| l.id == c.law)
                    .is_some_and(|l| def.hears(l.kind));
                let party = holder.is_some_and(|(h, hh)| {
                    h == c.by
                        || h == c.accused
                        || hh.is_some_and(|hh| hh == c.accuser || hh == c.accused_household)
                });
                if heard && !party {
                    c.forum = Some(law.id);
                }
            }
        }
        let waiting: Vec<u32> = {
            let on = self.polities[pi]
                .gathering
                .as_ref()
                .map(|g| g.cases.as_slice())
                .unwrap_or_default();
            self.order
                .cases
                .iter()
                .filter(|c| {
                    c.settlement == settlement
                        && c.stage == CaseStage::Open
                        && c.forum.is_none()
                        && c.challenge.is_none()
                })
                .map(|c| c.id)
                .filter(|id| !on.contains(id))
                .collect()
        };
        if waiting.is_empty() {
            return;
        }
        // Those who brought them know the gathering that is to hear them (M4c slice AE).
        let bringers: Vec<PermanentId> = self
            .order
            .cases
            .iter()
            .filter(|c| waiting.contains(&c.id))
            .map(|c| c.by)
            .collect();
        let polity = &mut self.polities[pi];
        let meets = match polity.gathering.as_mut() {
            Some(g) if g.day >= day => {
                g.cases.extend(waiting);
                g.day
            }
            // One already sat and waits to be decided: the cases go to the next.
            Some(_) => return,
            None => {
                let meets = day + i64::from(notice_days.max(1));
                polity.gathering = Some(Gathering {
                    law: None,
                    day: meets,
                    stakes: Vec::new(),
                    present: Vec::new(),
                    cases: waiting,
                });
                meets
            }
        };
        let (settlement, law) = (
            polity.settlement,
            polity.gathering.as_ref().and_then(|g| g.law),
        );
        self.call_word(settlement, meets, law, &bringers, day);
    }

    /// Before a forum sits on a case brought before today (M7b slice BR, ADR-0025 §1): the one
    /// who brought it withdraws it if their household has had back, by demand, all it said it
    /// lost. Otherwise the accused's elder may offer back what is still owed of it, weighed as
    /// paying a demand is (how many households believe the accused took, their regard for the one
    /// who brought it, the offer in days of their household's food) and the weight of a finding
    /// by the case's accounts ([`case_strength`]); and the one who brought it accepts an offer at
    /// least what they believe a finding would bring back, the case's strength times what is owed
    /// and the bundle's compensation. An accepted offer is owed as restitution, and the case is
    /// settled unheard. The gathering's cases keep its own procedure.
    pub(super) fn settle_cases(&mut self, ctx: &mut Ctx) {
        let (now, params) = (ctx.now, ctx.params);
        let (cp, day) = (&params.crime, now.day_index());
        let due: Vec<u32> = self
            .order
            .cases
            .iter()
            .filter(|c| c.stage == CaseStage::Open && c.forum.is_some() && c.opened < day)
            // A challenge or a breach of a privilege has nothing taken to give back.
            .filter(|c| c.challenge.is_none() && c.incident != 0)
            .map(|c| c.id)
            .collect();
        for id in due {
            let Ok(k) = self.order.cases.binary_search_by_key(&id, |c| c.id) else {
                continue;
            };
            let c = self.order.cases[k].clone();
            let Some(pi) = self.polity_of(c.settlement) else {
                continue;
            };
            let paid: f64 = self
                .order
                .obligations
                .iter()
                .filter(|o| o.incident == c.incident && o.kind == Owed::Demanded)
                .map(|o| f64::from(o.paid_kcal))
                .sum();
            let owed = (f64::from(c.kcal) - paid).max(0.0);
            let (place, name) = ctx
                .land
                .settlements
                .iter()
                .find(|s| s.id == c.settlement)
                .map_or((None, String::new()), |s| {
                    (Some(s.hearth_m), s.name.clone())
                });
            let accuser_words = self.household_words(ctx, c.accuser);
            let accused_name = self.name_of(c.accused);
            if owed <= 1.0 {
                let case = &mut self.order.cases[k];
                case.stage = CaseStage::Withdrawn;
                case.heard = Some(now);
                let words = format!(
                    "{} withdrew the case {accuser_words} brought against {accused_name} at \
                     {name}: what was taken had been given back.",
                    self.name_of(c.by)
                );
                self.chronicle_push(
                    now,
                    ChronicleKind::CaseHeard,
                    vec![c.by, c.accused],
                    Some(c.settlement),
                    place,
                    f64::from(CaseStage::Withdrawn.code()),
                    words,
                );
                continue;
            }
            let (Some(elder), Some(hh)) = (
                self.elder_of(c.accused_household, now, params),
                self.household(c.accused_household),
            ) else {
                continue;
            };
            let need = hh.members.len().max(1) as f64 * params.household.daily_kcal_per_person;
            let Some(outlook) = self.outlook(ctx, c.accused_household) else {
                continue;
            };
            if crate::polity::cannot_pay(outlook.held, owed, outlook.year_need, &params.polity) {
                continue;
            }
            let since = day - i64::from(cp.remember_days);
            let households: Vec<&Household> = self
                .households
                .iter()
                .map(|(_, x)| x)
                .filter(|x| x.settlement == Some(c.settlement) && !x.members.is_empty())
                .collect();
            let knowing = households
                .iter()
                .filter(|x| {
                    x.members
                        .iter()
                        .any(|&m| self.order.believes_took(m, c.accused, since, &|_| true))
                })
                .count();
            let known = knowing as f64 / households.len().max(1) as f64;
            let regard = self.ties.regard(elder, c.by, day, &params.ties);
            let strength = case_strength(c.leads.len());
            let points =
                comply_points(known, regard, owed / need, cp) + cp.w_comply_found * strength;
            let key = [ctx.seed, PURPOSE_SETTLE, u64::from(id)];
            if Rng64::from_key(&key).next_f64() >= logistic(points) {
                continue;
            }
            let sanction = self.polities[pi]
                .laws
                .iter()
                .find(|l| l.id == c.law)
                .map(|l| l.sanction)
                .unwrap_or_default();
            let accuser_need = self
                .household(c.accuser)
                .map_or(1, |x| x.members.len().max(1)) as f64
                * params.household.daily_kcal_per_person;
            let worth = strength * (owed + f64::from(sanction.compensation_days) * accuser_need);
            if owed < worth {
                continue;
            }
            let oid = self.order.obligations.last().map_or(1, |o| o.id + 1);
            self.order.obligations.push(Obligation {
                id: oid,
                incident: c.incident,
                kind: Owed::Restitution,
                case: Some(id),
                debtor: c.accused_household,
                beneficiary: c.accuser,
                kcal: owed as f32,
                paid_kcal: 0.0,
                made: day,
                due: day + i64::from(cp.due_days),
                standing: Standing::Open,
                answer: None,
            });
            let case = &mut self.order.cases[k];
            case.stage = CaseStage::Settled;
            case.heard = Some(now);
            let words = format!(
                "{accused_name}'s household offered back what was taken from {accuser_words}, and \
                 {} accepted: the case at {name} was settled without a hearing.",
                self.name_of(c.by)
            );
            self.chronicle_push(
                now,
                ChronicleKind::CaseHeard,
                vec![c.by, c.accused],
                Some(c.settlement),
                place,
                f64::from(CaseStage::Settled.code()),
                words,
            );
        }
    }

    /// Case `c`'s fact tags (M7b slice BS, ADR-0025 §2), from what it holds as it is heard at
    /// polity `pi`: see [`crate::crime::CaseTags`]. `outsider`: the accused lives in another
    /// settlement.
    fn case_tags(&self, ctx: &Ctx, pi: usize, c: &Case, outsider: bool) -> u8 {
        use crate::crime::CaseTags as T;
        let mut t = 0;
        if self.polities[pi]
            .laws
            .iter()
            .find(|l| l.id == c.law)
            .is_some_and(|l| l.kind == PolicyKind::AgainstRaiding)
        {
            t |= T::RAIDING;
        }
        if outsider {
            t |= T::OUTSIDER;
        }
        let theirs = self
            .household(c.accuser)
            .map(|x| x.members.clone())
            .unwrap_or_default();
        if self.close_kin(c.accused).iter().any(|k| theirs.contains(k)) {
            t |= T::KIN;
        }
        if c.leads.len() >= 2 {
            t |= T::ACCOUNTS;
        }
        let need = theirs.len().max(1) as f64 * ctx.params.household.daily_kcal_per_person;
        if f64::from(c.kcal) >= 5.0 * need {
            t |= T::LARGE;
        }
        if self
            .order
            .cases
            .iter()
            .any(|o| o.id < c.id && o.accused == c.accused && o.stage == CaseStage::Found)
        {
            t |= T::AGAIN;
        }
        t
    }

    /// The findings `p` remembers that they recall at a hearing of case `own` tagged `tags` (M7b
    /// slice BS, ADR-0025 §2): up to `k`, those whose tags match most, newest first; each (case,
    /// direction: 1 found, -1 not, share of tags matched).
    fn recall(&self, p: PermanentId, own: u32, tags: u8, k: usize) -> Vec<(u32, f64, f64)> {
        let mut out: Vec<(u32, f64, f64)> = self
            .word
            .heard_by(p)
            .iter()
            .filter_map(|h| {
                let claim = self.word.claim(h.claim)?;
                (claim.kind == crate::word::ClaimKind::Finding).then_some(claim.event?)
            })
            .filter(|&id| id != own)
            .filter_map(|id| {
                let c = self.order.case(id)?;
                // Challenges are recalled at challenges, takings at takings (M7b slice BT).
                if (c.tags ^ tags) & crate::crime::CaseTags::CHALLENGE != 0 {
                    return None;
                }
                let dir = match c.stage {
                    CaseStage::Found => 1.0,
                    CaseStage::NotFound => -1.0,
                    _ => return None,
                };
                Some((id, dir, crate::crime::CaseTags::matched(c.tags, tags)))
            })
            .collect();
        out.sort_by(|a, b| b.2.total_cmp(&a.2).then(b.0.cmp(&a.0)));
        out.truncate(k);
        out
    }

    /// Whether anyone of polity `pi` challenges a law in force that a limit in force forbids, at
    /// its review (M7b slice BT, ADR-0025 §4): only where its forum may review, and for a law no
    /// open challenge names. Standing comes from bearing the law: each adult whose household it
    /// costs weighs U = p * V - C (09-07 §4.1), V what their household gains if it no longer binds
    /// (as ending it is weighed), p from the law's reading against the limit (an even chance at
    /// the bound, certain at twice it), C the cost of bringing a case and their regard for the
    /// law's sponsor. The one with the most to gain, if anyone, brings it.
    pub(super) fn weigh_challenges(&mut self, ctx: &mut Ctx, pi: usize) {
        let (now, params) = (ctx.now, ctx.params);
        let (pp, cp, day) = (&params.polity, &params.crime, now.day_index());
        let policies = &ctx.catalog.policies;
        let Some((forum, hearers)) = self.polities[pi]
            .forum(policies)
            .filter(|(_, f)| f.review)
            .map(|(l, f)| (l.id, f.clone()))
        else {
            return;
        };
        let limits: Vec<(PermanentId, crate::forum::LimitDef)> = self.polities[pi]
            .laws
            .iter()
            .filter(|l| l.status == LawStatus::InForce && l.kind == PolicyKind::Limit)
            .filter_map(|l| Some((l.id, policies.get(usize::from(l.policy))?.limit?)))
            .collect();
        if limits.is_empty() {
            return;
        }
        let latitude = params.latitude_deg;
        let open: Vec<PermanentId> = self
            .order
            .cases
            .iter()
            .filter(|c| c.stage == CaseStage::Open && c.challenge.is_some())
            .map(|c| c.law)
            .collect();
        let mut bound: Vec<(PermanentId, PermanentId, f64)> = Vec::new();
        for l in &self.polities[pi].laws {
            if l.status != LawStatus::InForce || open.contains(&l.id) {
                continue;
            }
            for (limit, lim) in &limits {
                if let Some(r) = lim.reading(l.kind, l.hours, f64::from(l.levy_share), latitude) {
                    bound.push((l.id, *limit, r));
                }
            }
        }
        if bound.is_empty() {
            return;
        }
        let adults = self.members_of_polity(pi, now, params);
        let mut households: Vec<PermanentId> = adults.iter().map(|a| a.1).collect();
        households.sort_unstable();
        households.dedup();
        let fc = self.forecasts(ctx, pi, &households, &adults);
        for (law_id, limit, reading) in bound {
            let Some(law) = self.polities[pi].laws.iter().find(|l| l.id == law_id) else {
                continue;
            };
            let m = law_move(law);
            let sponsor = law.sponsor;
            let Some(sponsor_hh) = self.person(sponsor).map(|p| p.household) else {
                continue;
            };
            let p = (0.5 + 0.5 * reading).clamp(0.0, 1.0);
            let mut best: Option<(f64, PermanentId, PermanentId)> = None;
            for &(a, h) in &adults {
                if h == sponsor_hh {
                    continue;
                }
                let v = -pp.w_gain * fc.gain(&m, h, policies, params);
                if v <= 0.0 {
                    continue;
                }
                let regard = self
                    .ties
                    .regard(a, sponsor, day, &params.ties)
                    .clamp(0.0, 1.0);
                let u = p * v - cp.report_cost - pp.w_regard * regard;
                if u > 0.0 && best.is_none_or(|b| u > b.0) {
                    best = Some((u, a, h));
                }
            }
            let Some((_, by, household)) = best else {
                continue;
            };
            let settlement = self.polities[pi].settlement;
            let id = self.order.cases.last().map_or(1, |c| c.id + 1);
            self.order.cases.push(Case {
                id,
                incident: 0,
                settlement,
                law: law_id,
                accuser: household,
                by,
                accused: sponsor,
                accused_household: sponsor_hh,
                kcal: 0.0,
                leads: Vec::new(),
                opened: day,
                stage: CaseStage::Open,
                heard: None,
                eligible: 0,
                stances: Vec::new(),
                exiled: false,
                forum: Some(forum),
                challenge: Some(limit),
                tags: 0,
                departed: None,
            });
            let (place, name) = ctx
                .land
                .settlements
                .iter()
                .find(|s| s.id == settlement)
                .map_or((None, String::new()), |s| {
                    (Some(s.hearth_m), s.name.clone())
                });
            let law_words = self.law_words_of(ctx, pi, law_id);
            let limit_words = limits
                .iter()
                .find(|l| l.0 == limit)
                .map(|l| l.1.words())
                .unwrap_or_default();
            let holder = self.polities[pi]
                .laws
                .iter()
                .find(|l| l.id == forum)
                .and_then(|l| l.holder)
                .map(|h| self.name_of(h));
            let before = crate::forum::hearers_words(&hearers, holder.as_deref());
            let words = format!(
                "challenged {law_words} before {} at {name}, under the limit that {limit_words}.",
                lower_first(&before)
            );
            self.chronicle_push(
                now,
                ChronicleKind::CaseBrought,
                vec![by, sponsor],
                Some(settlement),
                place,
                0.0,
                words,
            );
        }
    }

    /// Law `law_id` of polity `pi`, in words; a privilege names its organization (M7c slice BX).
    pub(super) fn law_words_of(&self, ctx: &Ctx, pi: usize, law_id: PermanentId) -> String {
        let law = self.polities[pi].laws.iter().find(|l| l.id == law_id);
        if let Some((org, def)) = law.and_then(|l| {
            let def = ctx.catalog.policies.get(usize::from(l.policy))?.privilege?;
            Some((self.factions.get(l.target?)?, def))
        }) {
            return format!(
                "{}'s organization's privilege (it alone {})",
                self.name_of(org.founder),
                def.words()
            );
        }
        self.polities[pi]
            .laws
            .iter()
            .find(|l| l.id == law_id)
            .map(|l| self.polities[pi].words_of(l, &ctx.catalog.policies, &|p| self.name_of(p)))
            .unwrap_or_else(|| "a law".to_owned())
    }

    /// A forum with review hears challenge `id` at polity `pi` (M7b slice BT, ADR-0025 §4): each
    /// who sits weighs how they read the law against the limit (`crime.w_reading` points times the
    /// reading, -1 clearly none to 1 clearly a breach), what their household gains if the law no
    /// longer binds, their regard for the one who brought it against the law's sponsor, and the
    /// findings on challenges they recall. A finding strikes the law down: it binds no more.
    fn hear_challenge(
        &mut self,
        ctx: &mut Ctx,
        pi: usize,
        heard_by: &[PermanentId],
        def: &ForumDef,
        holder: Option<PermanentId>,
        id: u32,
    ) {
        let (now, params) = (ctx.now, ctx.params);
        let (pp, cp, day) = (&params.polity, &params.crime, now.day_index());
        let Ok(k) = self.order.cases.binary_search_by_key(&id, |c| c.id) else {
            return;
        };
        let c = self.order.cases[k].clone();
        let settlement = self.polities[pi].settlement;
        let (place, name) = ctx
            .land
            .settlements
            .iter()
            .find(|s| s.id == settlement)
            .map_or((None, String::new()), |s| {
                (Some(s.hearth_m), s.name.clone())
            });
        let policies = &ctx.catalog.policies;
        let law = self.polities[pi]
            .laws
            .iter()
            .find(|l| l.id == c.law && l.status == LawStatus::InForce)
            .cloned();
        let lim = c
            .challenge
            .and_then(|lid| self.polities[pi].laws.iter().find(|l| l.id == lid))
            .filter(|l| l.status == LawStatus::InForce)
            .and_then(|l| policies.get(usize::from(l.policy))?.limit);
        let reading = law.as_ref().zip(lim).and_then(|(l, lim)| {
            lim.reading(
                l.kind,
                l.hours,
                f64::from(l.levy_share),
                params.latitude_deg,
            )
        });
        let law_words = self.law_words_of(ctx, pi, c.law);
        let (Some(law), Some(lim), Some(reading)) = (law, lim, reading) else {
            let case = &mut self.order.cases[k];
            case.stage = CaseStage::Lapsed;
            case.heard = Some(now);
            let words = format!(
                "The challenge of {law_words} at {name} was not heard: the law or the limit no \
                 longer stood."
            );
            self.chronicle_push(
                now,
                ChronicleKind::CaseHeard,
                vec![c.by],
                Some(settlement),
                place,
                f64::from(CaseStage::Lapsed.code()),
                words,
            );
            return;
        };
        let tags = crate::crime::CaseTags::CHALLENGE
            | u8::from(lim.predicate == crate::forum::Predicate::LevyShare);
        let m = law_move(&law);
        let adults = self.members_of_polity(pi, now, params);
        let mut households: Vec<PermanentId> = heard_by
            .iter()
            .filter_map(|&p| self.person(p).map(|x| x.household))
            .collect();
        households.sort_unstable();
        households.dedup();
        let fc = self.forecasts(ctx, pi, &households, &adults);
        let regard = |p: PermanentId, q: PermanentId| {
            if p == q {
                1.0
            } else {
                self.ties.regard(p, q, day, &params.ties).clamp(0.0, 1.0)
            }
        };
        let mut recalled_all: Vec<(u32, f64, f64)> = Vec::new();
        let stances: Vec<CaseStance> = heard_by
            .iter()
            .filter_map(|&p| {
                let household = self.person(p)?.household;
                let stake = -pp.w_gain * fc.gain(&m, household, policies, params);
                let read = cp.w_reading * reading;
                let regard_points = pp.w_regard * (regard(p, c.by) - regard(p, c.accused));
                let recalled = self.recall(p, c.id, tags, cp.recall_k as usize);
                let precedent = crate::forum::precedent_points(
                    def.authority,
                    &recalled.iter().map(|r| (r.1, r.2)).collect::<Vec<_>>(),
                    cp,
                );
                recalled_all.extend(recalled);
                let s = stake + read + regard_points + precedent;
                let stance = if s > pp.stance_margin {
                    Stance::Support
                } else if s < -pp.stance_margin {
                    Stance::Oppose
                } else {
                    Stance::Abstain
                };
                Some(CaseStance {
                    person: p,
                    household,
                    stance,
                    stake: stake as f32,
                    belief: read as f32,
                    regard: regard_points as f32,
                    precedent: precedent as f32,
                })
            })
            .collect();
        let pairs: Vec<(PermanentId, Stance)> =
            stances.iter().map(|r| (r.person, r.stance)).collect();
        let decided = crate::forum::forum_decides(def, heard_by, &pairs);
        let who = format!(
            "{} at {name}",
            crate::forum::hearers_words(def, holder.map(|h| self.name_of(h)).as_deref())
        );
        let case = &mut self.order.cases[k];
        case.stances = stances;
        case.eligible = heard_by.len() as u32;
        case.heard = Some(now);
        case.tags = tags;
        let (_, support, oppose) = case.counts();
        case.stage = match decided {
            Decided::Passed => CaseStage::Found,
            Decided::Failed | Decided::Tied => CaseStage::NotFound,
            Decided::NoQuorum => CaseStage::Unheard,
        };
        let against = match case.stage {
            CaseStage::Found => -1.0,
            CaseStage::NotFound => 1.0,
            _ => 0.0,
        };
        case.departed = recalled_all
            .iter()
            .filter(|r| r.1 == against && r.2 >= 1.0)
            .map(|r| r.0)
            .max();
        let departed = case.departed;
        let tally = if def.seat == ForumSeat::Holder {
            "heard alone".to_owned()
        } else {
            format!(
                "{support} for, {oppose} against, of {} who sat",
                heard_by.len()
            )
        };
        let limit_words = lim.words();
        let words = match decided {
            Decided::Passed => {
                if let Some(l) = self.polities[pi].law_mut(c.law) {
                    l.status = LawStatus::StruckDown;
                }
                format!(
                    "{who} found that {law_words} breaks the limit that {limit_words}: {tally}. \
                     It binds no more."
                )
            }
            Decided::NoQuorum => {
                format!("Nobody sat to hear the challenge of {law_words} at {name}.")
            }
            _ => format!(
                "{who} did not find that {law_words} breaks the limit that {limit_words}: \
                 {tally}. It stands."
            ),
        };
        let words = match departed {
            Some(n) => format!("{words} It departed from case {n}, decided the other way."),
            None => words,
        };
        self.chronicle_push(
            now,
            ChronicleKind::CaseHeard,
            vec![c.by, c.accused],
            Some(settlement),
            place,
            f64::from(self.order.cases[k].stage.code()),
            words,
        );
        if matches!(decided, Decided::Passed | Decided::Failed | Decided::Tied) {
            let claim = self.word.make(crate::word::Claim {
                id: 0,
                kind: crate::word::ClaimKind::Finding,
                settlement,
                day,
                subject: None,
                grievance: None,
                suspected: None,
                fire: None,
                event: Some(c.id),
            });
            let mut holders: Vec<PermanentId> = heard_by.to_vec();
            holders.extend([c.by, c.accused]);
            holders.sort_unstable();
            holders.dedup();
            for h in holders {
                if self.person(h).is_some() {
                    self.word.hear(h, claim, day, None, None);
                }
            }
        }
    }

    /// Forums sit (M7b slice BR, ADR-0025 §1): each case a forum is to hear, brought before
    /// today, is heard by those who sit in it. One whose forum has ended since goes back to wait
    /// for the gathering.
    pub(super) fn forums_day(&mut self, ctx: &mut Ctx) {
        let day = ctx.now.day_index();
        let due: Vec<(u32, PermanentId, PermanentId)> = self
            .order
            .cases
            .iter()
            .filter(|c| c.stage == CaseStage::Open && c.opened < day)
            .filter_map(|c| Some((c.id, c.forum?, c.settlement)))
            .collect();
        for (id, law, settlement) in due {
            let Some(pi) = self.polity_of(settlement) else {
                continue;
            };
            let sitting = self.polities[pi]
                .forum(&ctx.catalog.policies)
                .filter(|(l, _)| l.id == law)
                .map(|(l, f)| (l.holder, f.clone()));
            let challenge = self.order.case(id).is_some_and(|c| c.challenge.is_some());
            let Some((holder, def)) = sitting.filter(|s| !challenge || s.1.review) else {
                if let Ok(k) = self.order.cases.binary_search_by_key(&id, |c| c.id) {
                    // A challenge needs a forum with review: without one it is never heard
                    // (M7b slice BT).
                    if challenge {
                        self.order.cases[k].stage = CaseStage::Lapsed;
                        self.order.cases[k].heard = Some(ctx.now);
                    } else {
                        self.order.cases[k].forum = None;
                    }
                }
                continue;
            };
            let sitters = self.forum_sitters(ctx, pi, &def, holder, id);
            if challenge {
                self.hear_challenge(ctx, pi, &sitters, &def, holder, id);
            } else {
                self.hear_case(ctx, pi, &sitters, Some((&def, holder)), id);
            }
        }
    }

    /// Who sits in forum `def` of polity `pi` to hear case `id` (ADR-0025 §1), in id order: its
    /// holder; a panel of `def.panel` drawn by a lot keyed by the world, the polity and the case
    /// from the body's members; or the elders of its households. Nobody of the two households
    /// sits on a panel or among the elders.
    fn forum_sitters(
        &self,
        ctx: &Ctx,
        pi: usize,
        def: &ForumDef,
        holder: Option<PermanentId>,
        id: u32,
    ) -> Vec<PermanentId> {
        let Ok(k) = self.order.cases.binary_search_by_key(&id, |c| c.id) else {
            return Vec::new();
        };
        let c = &self.order.cases[k];
        let parties = [c.accuser, c.accused_household];
        let (now, params) = (ctx.now, ctx.params);
        let free = |list: Vec<(PermanentId, PermanentId)>| -> Vec<PermanentId> {
            list.into_iter()
                .filter(|&(p, h)| !parties.contains(&h) && p != c.by && p != c.accused)
                .map(|(p, _)| p)
                .collect()
        };
        let mut out = match def.seat {
            ForumSeat::Holder => holder.into_iter().collect(),
            ForumSeat::Panel => {
                let pool = free(self.body_members(&ctx.land.fields, pi, now, params));
                let key = Rng64::from_key(&[
                    ctx.seed,
                    self.polities[pi].id.get(),
                    u64::from(id),
                    PURPOSE_PANEL,
                ])
                .next_u64();
                crate::count::draw(&pool, u32::from(def.panel), key)
            }
            ForumSeat::Elders => {
                let adults = self.members_of_polity(pi, now, params);
                free(self.admitted(
                    crate::polity::Membership::Elders,
                    &adults,
                    &ctx.land.fields,
                    now,
                    params,
                ))
            }
        };
        out.sort_unstable();
        out
    }

    /// Case `id` is heard at polity `pi` by those `heard_by` (ADR-0015 §4): at a gathering, who
    /// came of its body's members; before a forum (M7b slice BR), those who sit in it. Each
    /// stands for a finding or against it, by what they believe (or make of the accounts told
    /// there), what their household stands to gain or lose by it, and their regard for the one
    /// who brought it against their regard for the accused; the body decides by its rule, a forum
    /// by its own. A finding imposes the law's bundle as obligations on the accused's household,
    /// and those who heard it believe it.
    pub(super) fn hear_case(
        &mut self,
        ctx: &mut Ctx,
        pi: usize,
        heard_by: &[PermanentId],
        forum: Option<(&ForumDef, Option<PermanentId>)>,
        id: u32,
    ) {
        let (now, params) = (ctx.now, ctx.params);
        let (pp, cp) = (&params.polity, &params.crime);
        let day = now.day_index();
        let Ok(k) = self.order.cases.binary_search_by_key(&id, |c| c.id) else {
            return;
        };
        let c = self.order.cases[k].clone();
        if c.stage != CaseStage::Open {
            return;
        }
        let settlement = self.polities[pi].settlement;
        let (place, name) = ctx
            .land
            .settlements
            .iter()
            .find(|s| s.id == settlement)
            .map_or((None, String::new()), |s| {
                (Some(s.hearth_m), s.name.clone())
            });
        let accuser_words = self.household_words(ctx, c.accuser);
        let accused_name = self.name_of(c.accused);
        // The household that brought it, or the accused, is gone: nothing is heard.
        let accused_hh = self.person(c.accused).map(|p| p.household).filter(|&h| {
            self.household(h)
                .is_some_and(|x| x.settlement == Some(settlement))
        });
        let (Some(accused_hh), true) = (accused_hh, self.household(c.accuser).is_some()) else {
            let why = match self.records.get(&c.accused) {
                Some(r) if r.died.is_some() => format!("{accused_name} had died"),
                Some(r) if r.left.is_some() => format!("{accused_name} had left the valley"),
                _ if self.household(c.accuser).is_none() => {
                    "the household that brought it was no more".to_owned()
                }
                _ => format!("{accused_name} no longer lived there"),
            };
            let case = &mut self.order.cases[k];
            case.stage = CaseStage::Lapsed;
            case.heard = Some(now);
            let words = format!(
                "The case {accuser_words} brought against {accused_name} at {name} was not \
                 heard: {why}."
            );
            self.chronicle_push(
                now,
                ChronicleKind::CaseHeard,
                vec![c.accused],
                Some(settlement),
                place,
                f64::from(CaseStage::Lapsed.code()),
                words,
            );
            return;
        };
        // What a finding would move: what is still owed of what was taken, and the bundle, in
        // days of the accused's household's food.
        let sanction = self.polities[pi]
            .laws
            .iter()
            .find(|l| l.id == c.law)
            .map(|l| l.sanction)
            .unwrap_or_default();
        // A breach of a privilege (M7c slice BX): an offer at a door, which everyone saw.
        let breach = self.polities[pi]
            .laws
            .iter()
            .find(|l| l.id == c.law)
            .is_some_and(|l| l.kind == PolicyKind::Privilege);
        let breach_words = if breach {
            self.law_words_of(ctx, pi, c.law)
        } else {
            String::new()
        };
        let need_of = |h: PermanentId| {
            self.household(h).map_or(1, |x| x.members.len().max(1)) as f64
                * params.household.daily_kcal_per_person
        };
        let debtor_need = need_of(accused_hh);
        // One of another settlement the gathering may find against, but cannot send from a
        // valley they do not live in, and its finding binds nobody there (M6c slice BI).
        let outsider = self.household(accused_hh).and_then(|x| x.settlement) != Some(settlement);
        let paid: f64 = self
            .order
            .obligations
            .iter()
            .filter(|o| o.incident == c.incident && o.kind == Owed::Demanded)
            .map(|o| f64::from(o.paid_kcal))
            .sum();
        let restitution = (f64::from(c.kcal) - paid).max(0.0);
        let compensation = f64::from(sanction.compensation_days) * debtor_need;
        let fine = f64::from(sanction.fine_days) * debtor_need;
        let exile = if sanction.exile {
            cp.exile_days * debtor_need
        } else {
            0.0
        };
        // What the two households stand to gain and lose, at the points a day of its food is
        // worth to a household in a dispute (the scale of a demand), however lean the year: a
        // few days' food matters most to those with least.
        let gain = cp.w_demand_loss * (restitution + compensation) / need_of(c.accuser);
        let loss = cp.w_demand_loss * (restitution + compensation + fine + exile) / debtor_need;
        // Its body's members now (M4c slice AF).
        let members = self.body_members(&ctx.land.fields, pi, now, params);
        let regard = |p: PermanentId, q: PermanentId| {
            if p == q {
                1.0
            } else {
                self.ties.regard(p, q, day, &params.ties).clamp(0.0, 1.0)
            }
        };
        // Its fact tags, and how far those hearing it follow the findings they remember (M7b
        // slice BS, ADR-0025 §2): the gathering's are informative.
        let tags = self.case_tags(ctx, pi, &c, outsider);
        let authority = forum.map_or(crate::forum::Authority::Informative, |(d, _)| d.authority);
        // Every finding recalled, (case, direction, share of tags matched).
        let mut recalled_all: Vec<(u32, f64, f64)> = Vec::new();
        let stances: Vec<CaseStance> = heard_by
            .iter()
            .filter_map(|&p| {
                // A forum's sitters need not be the body's members (an elder of a council's
                // polity, a hearer named from the adults).
                let household = if forum.is_some() {
                    self.person(p)?.household
                } else {
                    let i = members.binary_search_by_key(&p, |m| m.0).ok()?;
                    members[i].1
                };
                let stake = if household == c.accuser {
                    gain
                } else if household == accused_hh {
                    -loss
                } else {
                    0.0
                };
                let believes = breach
                    || self
                        .order
                        .belief(p, c.incident)
                        .is_some_and(|b| b.taker == Some(c.accused));
                let belief = belief_points(believes, c.leads.len(), cp);
                let regard_points = pp.w_regard * (regard(p, c.by) - regard(p, c.accused));
                let recalled = self.recall(p, c.id, tags, cp.recall_k as usize);
                let precedent = crate::forum::precedent_points(
                    authority,
                    &recalled.iter().map(|r| (r.1, r.2)).collect::<Vec<_>>(),
                    cp,
                );
                recalled_all.extend(recalled);
                let stance = if p == c.by {
                    Stance::Support
                } else if p == c.accused {
                    Stance::Oppose
                } else {
                    let s = stake + belief + regard_points + precedent;
                    if s > pp.stance_margin {
                        Stance::Support
                    } else if s < -pp.stance_margin {
                        Stance::Oppose
                    } else {
                        Stance::Abstain
                    }
                };
                Some(CaseStance {
                    person: p,
                    household,
                    stance,
                    stake: stake as f32,
                    belief: belief as f32,
                    regard: regard_points as f32,
                    precedent: precedent as f32,
                })
            })
            .collect();
        let body = self.polities[pi].body;
        // A council counts its seats, filled or empty (M7a slice BO); a forum, those who sit.
        let eligible = match forum {
            Some(_) => heard_by.len() as u32,
            None => body.eligible(members.len()),
        };
        // Who heard it, to begin a sentence (M7b slice BR).
        let who = match forum {
            Some((def, holder)) => {
                let holder = holder.map(|h| self.name_of(h));
                format!(
                    "{} at {name}",
                    crate::forum::hearers_words(def, holder.as_deref())
                )
            }
            None => format!("The gathering at {name}"),
        };
        let pairs: Vec<(PermanentId, Stance)> =
            stances.iter().map(|r| (r.person, r.stance)).collect();
        let case = &mut self.order.cases[k];
        case.stances = stances;
        case.eligible = eligible;
        case.heard = Some(now);
        let (came, support, oppose) = case.counts();
        let decided = match forum {
            Some((def, _)) => crate::forum::forum_decides(def, heard_by, &pairs),
            None => body.decide(eligible, came, support, oppose),
        };
        let present = came;
        case.stage = match decided {
            Decided::Passed => CaseStage::Found,
            Decided::Failed | Decided::Tied => CaseStage::NotFound,
            Decided::NoQuorum => CaseStage::Unheard,
        };
        case.tags = tags;
        // A finding against a remembered one of the same tags departs from it (ADR-0025 §2):
        // the newest such recalled.
        let against = match case.stage {
            CaseStage::Found => -1.0,
            CaseStage::NotFound => 1.0,
            _ => 0.0,
        };
        case.departed = recalled_all
            .iter()
            .filter(|r| r.1 == against && r.2 >= 1.0)
            .map(|r| r.0)
            .max();
        let departed = case.departed;
        let tally = match forum {
            Some((def, _)) if def.seat == ForumSeat::Holder => "heard alone".to_owned(),
            Some(_) => format!("{support} for, {oppose} against, of {eligible} who sat"),
            None => format!("{support} for, {oppose} against; {present} of {eligible} adults came"),
        };
        let words = match decided {
            Decided::Passed if breach => {
                let mut owed = Vec::new();
                if compensation > 1.0 {
                    owed.push(format!(
                        "{:.0} days' food to the household that brought it",
                        f64::from(sanction.compensation_days)
                    ));
                }
                if fine > 1.0 {
                    owed.push(format!(
                        "{:.0} days' food to the common store",
                        f64::from(sanction.fine_days)
                    ));
                }
                let owes = if owed.is_empty() {
                    String::new()
                } else {
                    format!(" {accused_name}'s household owes {}.", owed.join(" and "))
                };
                format!(
                    "{who} found that {accused_name}'s household broke {breach_words}: \
                     {tally}.{owes}"
                )
            }
            Decided::Failed if breach => format!(
                "{who} did not find that {accused_name}'s household broke {breach_words}: \
                 {tally}."
            ),
            Decided::Tied if breach => format!(
                "{who} was evenly split on whether {accused_name}'s household broke \
                 {breach_words}, so it found nothing: {tally}."
            ),
            Decided::Passed => {
                // The household owes; the one found is the one sent away.
                let owed = Sanction {
                    exile: false,
                    ..sanction
                };
                let sent = if sanction.exile && !outsider {
                    format!(" {accused_name} is sent from the valley.")
                } else {
                    String::new()
                };
                format!(
                    "{who} found that {accused_name} took food from {accuser_words}: {tally}. \
                     {accused_name}'s household owes it back{}.{sent}",
                    owed.words()
                )
            }
            Decided::Failed => format!(
                "{who} did not find that {accused_name} took food from {accuser_words}: {tally}."
            ),
            Decided::Tied => format!(
                "{who} was evenly split on whether {accused_name} took food from \
                 {accuser_words}, so it found nothing: {tally}."
            ),
            Decided::NoQuorum if forum.is_some() => format!(
                "Nobody sat to hear the case {accuser_words} brought against {accused_name} at \
                 {name}."
            ),
            Decided::NoQuorum => format!(
                "Too few came to the gathering at {name} to hear the case {accuser_words} \
                 brought against {accused_name}: {present} of {eligible} adults, where {} were \
                 needed.",
                body.quorum(eligible)
            ),
        };
        let words = match departed {
            Some(n) => format!(
                "{words} It departed from case {n}, of the same facts, which was decided the \
                 other way."
            ),
            None => words,
        };
        self.chronicle_push(
            now,
            ChronicleKind::CaseHeard,
            vec![c.by, c.accused],
            Some(settlement),
            place,
            f64::from(self.order.cases[k].stage.code()),
            words,
        );
        // Those who heard it, and the two parties, hold the finding (M7b slice BS, ADR-0025 §2).
        if matches!(decided, Decided::Passed | Decided::Failed | Decided::Tied) {
            let claim = self.word.make(crate::word::Claim {
                id: 0,
                kind: crate::word::ClaimKind::Finding,
                settlement,
                day,
                subject: None,
                grievance: None,
                suspected: None,
                fire: None,
                event: Some(c.id),
            });
            let mut holders: Vec<PermanentId> = heard_by.to_vec();
            holders.extend([c.by, c.accused]);
            holders.sort_unstable();
            holders.dedup();
            for h in holders {
                if self.person(h).is_some() {
                    self.word.hear(h, claim, day, None, None);
                }
            }
        }
        // What the decision leaves people holding against the gathering (M4c slice AE).
        let stage = self.order.cases[k].stage;
        self.grieve_case(
            ctx,
            pi,
            &c,
            stage,
            restitution + compensation + fine,
            decided == Decided::Passed && sanction.exile && !outsider,
        );
        if decided != Decided::Passed {
            return;
        }
        // The finding: the law's bundle, as obligations (ADR-0015 §5), and word of it.
        let polity = self.polities[pi].id;
        for (kind, kcal, beneficiary) in [
            (Owed::Restitution, restitution, c.accuser),
            (Owed::Compensation, compensation, c.accuser),
            (Owed::Fine, fine, polity),
        ] {
            if kcal <= 1.0 {
                continue;
            }
            let id = self.order.obligations.last().map_or(1, |o| o.id + 1);
            self.order.obligations.push(Obligation {
                id,
                incident: c.incident,
                kind,
                case: Some(c.id),
                debtor: accused_hh,
                beneficiary,
                kcal: kcal as f32,
                paid_kcal: 0.0,
                made: day,
                due: day + i64::from(cp.due_days),
                standing: Standing::Open,
                answer: None,
            });
        }
        let origin = c.leads.first().copied().unwrap_or(c.by);
        // Exile: those who found against them see them go (09-07 §1.1: enforcement needs an
        // actual coalition; here, the gathering that found). They leave the valley, recorded as
        // a migration (ADR-0015 §5); their household stays, and owes what was imposed.
        if sanction.exile && !outsider {
            self.exile(ctx, c.accused);
            self.order.cases[k].exiled = true;
        }
        for &p in heard_by {
            if breach
                || p == c.accused
                || self
                    .order
                    .belief(p, c.incident)
                    .is_some_and(|b| b.taker.is_some())
            {
                continue;
            }
            self.come_to_believe(
                ctx,
                Belief {
                    holder: p,
                    incident: c.incident,
                    taker: Some(c.accused),
                    source: Source::Told,
                    from: Some(c.by),
                    origin: Some(origin),
                    day,
                },
            );
        }
    }
}
