//! Agreements between polities as people make them (M5c slice AU, ADR-0020 §6). Someone whose
//! household heard that another polity claims places it works, and who may propose at home, seeks
//! terms with the one of that settlement they know best who may propose there. The two weigh a
//! bounded set of packages, each by their own household's forecast and the support they predict
//! at their own gathering, and agree on one both expect to pass, or part with none. Each sponsors
//! it at home as a law that polity's own custom decides; it is in force once both gatherings have
//! passed it and each side has heard of the other's decision from someone of the other
//! settlement. Failure is an outcome, never repaired. The meeting itself takes no time: nobody
//! walks to it (a simplification).

use super::*;
use crate::agreements::{Agreement, AgreementState, Clause, Failure};
use crate::history::AgreementStep;
use crate::polity::{IssueKind, LawStatus, MoveOption, Outcome, Outlook, PolicyKind};
use crate::uses::{Place, per_year};

/// A package two who met weigh: what each side grants, and for how long.
#[derive(Clone, Debug, PartialEq)]
struct Package {
    clauses: Vec<Clause>,
    term_days: u32,
}

/// What one side's negotiator makes of a package: their score, points, and the share of those
/// at their gathering they expect to back it.
#[derive(Clone, Copy, Debug)]
struct Weighed {
    score: f64,
    support: f64,
}

/// What a household of one side stands to get back, and to give up, a year under leave, kcal.
#[derive(Clone, Copy, Debug)]
struct Stakes {
    household: PermanentId,
    outlook: Outlook,
    /// What the other polity's claims it heard of cost it at places it works, that leave from the
    /// other would give back.
    recover: f64,
    /// What people of the other settlement take at places its own polity claims and it works,
    /// that its claim kept and leave to them would give up.
    owe: f64,
}

impl Stakes {
    /// Its forecast of a package by which its polity `gives` leave and `gets` it, in the units of
    /// [`crate::polity::against_gain`].
    fn gain(&self, gives: bool, gets: bool, pp: &crate::polity::PolityParams) -> f64 {
        let recover = if gets { self.recover } else { 0.0 };
        let owe = if gives { self.owe } else { 0.0 };
        crate::polity::against_gain(&self.outlook, recover, owe, pp)
    }
}

impl Population {
    /// Whether an agreement in force gives the people of polity `to` leave to use the places
    /// polity `from` claims, as one of `who` knows it: they know the law by which polity `pi`,
    /// one of the two, decided it.
    pub(super) fn leave_known(
        &self,
        from: PermanentId,
        to: PermanentId,
        pi: usize,
        who: &[PermanentId],
    ) -> bool {
        if self.agreements.list.is_empty() {
            return false;
        }
        let polity = &self.polities[pi];
        self.agreements.list.iter().any(|a| {
            a.leave(from, to)
                && a.side_of(polity.id)
                    .and_then(|s| a.laws[usize::from(s)])
                    .is_some_and(|law| {
                        polity
                            .laws
                            .iter()
                            .any(|l| l.id == law && who.iter().any(|&p| l.knows(p)))
                    })
        })
    }

    /// The claims of other polities household `h` of polity `pi` heard of on places it worked
    /// within memory, which its own polity does not claim and which give it no leave it knows of:
    /// by polity, then place. Their being there is the issue *claimed from us*.
    pub(super) fn claimed_from(
        &self,
        ctx: &Ctx,
        pi: usize,
        h: PermanentId,
    ) -> Vec<(PermanentId, Place)> {
        let heard = self.claims_heard.of(h);
        if heard.is_empty() {
            return Vec::new();
        }
        let (day, half) = (ctx.now.day_index(), ctx.params.places.use_half_life_days);
        let polity = &self.polities[pi];
        let members = self.household(h).map_or(&[][..], |x| x.members.as_slice());
        let mut out: Vec<(PermanentId, Place)> = heard
            .iter()
            .filter(|c| c.polity != polity.id && polity.claim_on(c.place).is_none())
            .filter(|c| self.uses.held(h, c.place, day, half).is_some())
            .filter(|c| !self.leave_known(c.polity, polity.id, pi, members))
            .map(|c| (c.polity, c.place))
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    /// The households of `settlement` with anyone in them, in id order.
    fn homes_in(&self, settlement: PermanentId) -> Vec<PermanentId> {
        let mut out: Vec<PermanentId> = self
            .households
            .iter()
            .filter(|(_, x)| x.settlement == Some(settlement) && !x.members.is_empty())
            .map(|(_, x)| x.id)
            .collect();
        out.sort_unstable();
        out
    }

    /// What each household of polity `pi` with an outlook stands to get back and give up under
    /// leave between it and polity `oi`, in household order: as much of what the claims cost or
    /// keep as a claim is believed to keep ([`crate::polity::PolityParams::claim_keeps`]).
    fn stakes_of(&self, ctx: &Ctx, pi: usize, oi: usize) -> Vec<Stakes> {
        let (day, half) = (ctx.now.day_index(), ctx.params.places.use_half_life_days);
        let keeps = ctx.params.polity.claim_keeps;
        let other = &self.polities[oi];
        let ours = self.polities[pi].claims_now();
        self.homes_in(self.polities[pi].settlement)
            .into_iter()
            .filter_map(|h| {
                let outlook = self.outlook(ctx, h)?;
                let recover: f64 = self
                    .claimed_from(ctx, pi, h)
                    .iter()
                    .filter(|c| c.0 == other.id)
                    .filter_map(|c| self.uses.held(h, c.1, day, half))
                    .map(|u| per_year(f64::from(u.kcal), half))
                    .sum();
                let owe: f64 = ours
                    .iter()
                    .filter_map(|&p| self.uses.held(h, p, day, half))
                    .map(|u| {
                        let all: f64 = u.outsiders.iter().map(|o| f64::from(o.days)).sum();
                        let theirs: f64 = u
                            .outsiders
                            .iter()
                            .filter(|o| o.settlement == other.settlement)
                            .map(|o| f64::from(o.days))
                            .sum();
                        per_year(f64::from(u.kcal), half) * theirs
                            / (f64::from(u.days) + all).max(1e-9)
                    })
                    .sum();
                Some(Stakes {
                    household: h,
                    outlook,
                    recover: recover * keeps,
                    owe: owe * keeps,
                })
            })
            .collect()
    }

    /// The adults of polity `pi`'s settlement its body admits, with their households.
    fn body_of(&self, ctx: &Ctx, pi: usize) -> Vec<(PermanentId, PermanentId)> {
        let (now, params) = (ctx.now, ctx.params);
        let adults = self.members_of(self.polities[pi].settlement, now, params);
        self.admitted(
            self.polities[pi].body.members,
            &adults,
            &ctx.land.fields,
            now,
            params,
        )
    }

    /// What negotiator `n` of polity `pi` makes of each package with polity `oi`, being on side
    /// `side`, as a review weighs a move (ADR-0013 §5): their household's forecast, what the
    /// template does to what they hold dear, and the forecasts of those who regard them, at the
    /// support they predict among the members of their body they know; the forecasts over the
    /// term, in years.
    #[allow(clippy::too_many_arguments)]
    fn weigh_packages(
        &self,
        ctx: &Ctx,
        pi: usize,
        oi: usize,
        side: u8,
        n: PermanentId,
        k: u16,
        packages: &[Package],
    ) -> Vec<Weighed> {
        let (pp, tp) = (&ctx.params.polity, &ctx.params.ties);
        let day = ctx.now.day_index();
        let stakes = self.stakes_of(ctx, pi, oi);
        let members = self.body_of(ctx, pi);
        let Some(own) = members.iter().find(|m| m.0 == n).map(|m| m.1) else {
            return packages
                .iter()
                .map(|_| Weighed {
                    score: f64::NEG_INFINITY,
                    support: 0.0,
                })
                .collect();
        };
        let creed = self.creed_for(ctx, n, k).map_or(0.0, |c| c.1);
        let dear = (self.value_points(ctx, n, k) + creed) / pp.w_gain.max(1e-9);
        packages
            .iter()
            .map(|p| {
                let gives = p.clauses.contains(&Clause::Leave { from: side });
                let gets = p.clauses.contains(&Clause::Leave { from: 1 - side });
                let gain = |h: PermanentId| {
                    stakes
                        .binary_search_by_key(&h, |s| s.household)
                        .map_or(0.0, |i| stakes[i].gain(gives, gets, pp))
                };
                let (mut weighed, mut weight, mut support, mut oppose) = (0.0, 0.0, 0, 0);
                for &(a, h) in &members {
                    if a == n {
                        continue;
                    }
                    let g = gain(h);
                    let regard = self.ties.regard(a, n, day, tp).clamp(0.0, 1.0);
                    if regard > 0.0 {
                        weighed += regard * g;
                        weight += regard;
                    }
                    if self.ties.known(n, a, day, tp) > 0.0 {
                        let to_n = self.ties.regard(a, n, day, tp);
                        match crate::polity::stance(pp.w_gain * g, to_n, pp).0 {
                            crate::polity::Stance::Support => support += 1,
                            crate::polity::Stance::Oppose => oppose += 1,
                            crate::polity::Stance::Abstain => {}
                        }
                    }
                }
                let followers = if weight > 0.0 { weighed / weight } else { 0.0 };
                let support = if support + oppose > 0 {
                    f64::from(support) / f64::from(support + oppose)
                } else {
                    0.5
                };
                let years = f64::from(p.term_days) / 365.0;
                let forecast = (gain(own) + pp.w_followers * followers) * years;
                Weighed {
                    score: support * pp.w_gain * (forecast + dear) - pp.propose_cost,
                    support,
                }
            })
            .collect()
    }

    /// Polity `pi` reviewed and put nothing to its gathering (M5c slice AU): the elder of a
    /// household that heard another polity claims places it works, who may propose at home,
    /// seeks terms with the one of that polity's body they know best, when some package is one
    /// they would sponsor, unless an agreement between the two polities is open or in force, or
    /// one failed within memory. One meeting a review, the seekers in id order.
    pub(super) fn seek_terms(&mut self, ctx: &mut Ctx, pi: usize) {
        if self.claims_heard.households.is_empty() {
            return;
        }
        let Some(k) = ctx
            .catalog
            .policies
            .iter()
            .position(|d| d.kind == PolicyKind::Agreement)
        else {
            return;
        };
        let polity = &self.polities[pi];
        if polity.agenda().is_some() || polity.gathering.is_some() {
            return;
        }
        let (now, params) = (ctx.now, ctx.params);
        let (day, tp) = (now.day_index(), &params.ties);
        let own = polity.id;
        let members = self.body_of(ctx, pi);
        let mut wants: Vec<(PermanentId, Vec<PermanentId>)> = Vec::new();
        for h in self.homes_in(polity.settlement) {
            let mut theirs: Vec<PermanentId> = self
                .claimed_from(ctx, pi, h)
                .into_iter()
                .map(|c| c.0)
                .collect();
            theirs.dedup();
            if theirs.is_empty() {
                continue;
            }
            let Some(e) = self.elder_of(h, now, params) else {
                continue;
            };
            if members.binary_search_by_key(&e, |m| m.0).is_ok() {
                wants.push((e, theirs));
            }
        }
        wants.sort_unstable_by_key(|w| w.0);
        let memory = i64::from(params.polity.vote_memory_days);
        for (seeker, theirs) in wants {
            for q in theirs {
                let failed_lately = self.agreements.list.iter().any(|a| {
                    a.side_of(own).is_some()
                        && a.side_of(q).is_some()
                        && matches!(a.state, AgreementState::Failed { day: d, .. } if day - d <= memory)
                });
                if self.agreements.between(own, q) || failed_lately {
                    continue;
                }
                let Some(qi) = self.polities.iter().position(|p| p.id == q) else {
                    continue;
                };
                let counterpart = self
                    .body_of(ctx, qi)
                    .into_iter()
                    .map(|m| (self.ties.known(seeker, m.0, day, tp), m.0))
                    .filter(|m| m.0 > 0.0)
                    .max_by(|a, b| a.0.total_cmp(&b.0).then(b.1.cmp(&a.1)));
                // Seeking terms is a move like any other (ADR-0020 §6): they go only when some
                // package is one they would sponsor, by their own forecast and the support they
                // expect at home; otherwise there is nothing to seek.
                let worth = || {
                    let packages = self.packages(ctx, pi, qi);
                    self.weigh_packages(ctx, pi, qi, 0, seeker, k as u16, &packages)
                        .iter()
                        .any(|w| w.score > 0.0 && w.support > 0.5)
                };
                if let Some((_, c)) = counterpart
                    && worth()
                {
                    self.meet(ctx, pi, qi, k as u16, seeker, c);
                    return;
                }
            }
        }
    }

    /// `seeker` of polity `pi` and `counterpart` of polity `qi` meet to seek terms (ADR-0020 §6)
    /// and weigh the packages between them. They agree on the package each
    /// expects to pass at home and to be worth sponsoring, the one worth most to the two
    /// together, or part with none; either is recorded. With terms agreed, the seeker sponsors it
    /// at home at once.
    fn meet(
        &mut self,
        ctx: &mut Ctx,
        pi: usize,
        qi: usize,
        k: u16,
        seeker: PermanentId,
        counterpart: PermanentId,
    ) {
        let sides = [pi, qi];
        let negotiators = [seeker, counterpart];
        let packages = self.packages(ctx, pi, qi);
        let weighed = [0u8, 1].map(|s| {
            let s_ = usize::from(s);
            self.weigh_packages(
                ctx,
                sides[s_],
                sides[1 - s_],
                s,
                negotiators[s_],
                k,
                &packages,
            )
        });
        let both = |i: usize| weighed[0][i].score + weighed[1][i].score;
        let pick = (0..packages.len())
            .filter(|&i| {
                weighed
                    .iter()
                    .all(|w| w[i].support > 0.5 && w[i].score > 0.0)
            })
            .max_by(|&a, &b| both(a).total_cmp(&both(b)).then(b.cmp(&a)));
        let day = ctx.now.day_index();
        let id = ctx.ids.allocate();
        let (clauses, term_days, state) = match pick {
            Some(i) => (
                packages[i].clauses.clone(),
                packages[i].term_days,
                AgreementState::Offered,
            ),
            None => (
                Vec::new(),
                0,
                AgreementState::Failed {
                    day,
                    why: Failure::NoTerms,
                },
            ),
        };
        let agreement = Agreement {
            id,
            polities: [self.polities[pi].id, self.polities[qi].id],
            negotiators,
            clauses,
            term_days,
            made: day,
            laws: [None, None],
            passed: [None, None],
            heard: [None, None],
            state,
        };
        let words = self.agreement_words(ctx, &agreement);
        let (a_name, b_name) = (self.name_of(seeker), self.name_of(counterpart));
        let (from, to) = (
            self.settlement_name(ctx, self.polities[pi].settlement),
            self.settlement_name(ctx, self.polities[qi].settlement),
        );
        let (step, sentence) = if pick.is_some() {
            (
                AgreementStep::Agreed,
                format!(
                    "{a_name} of {from} sought terms with {b_name} of {to}, and the two agreed \
                     to put {words} to their gatherings."
                ),
            )
        } else {
            (
                AgreementStep::NoTerms,
                format!(
                    "{a_name} of {from} sought terms with {b_name} of {to}, and the two parted \
                     with none: no package was one both expected to pass at home."
                ),
            )
        };
        self.agreements.list.push(agreement);
        self.chronicle_agreement(ctx, id, step, sentence);
        if pick.is_some() {
            self.sponsor(ctx, id, 0, k);
        }
    }

    /// The packages two of polities `pi` (side 0) and `qi` (side 1) may weigh: leave on the places
    /// one side claims, the other's, or both, if it claims any, for each term the content offers,
    /// up to the content's number of packages.
    fn packages(&self, ctx: &Ctx, pi: usize, qi: usize) -> Vec<Package> {
        let rp = &ctx.params.relations;
        let sides = [pi, qi];
        let can: Vec<Clause> = (0..2u8)
            .filter(|&s| !self.polities[sides[usize::from(s)]].claims_now().is_empty())
            .map(|s| Clause::Leave { from: s })
            .collect();
        let mut sets: Vec<Vec<Clause>> = can.iter().map(|&c| vec![c]).collect();
        if can.len() == 2 {
            sets.push(can.clone());
        }
        sets.iter()
            .flat_map(|c| {
                rp.terms_days.iter().map(move |&t| Package {
                    clauses: c.clone(),
                    term_days: t,
                })
            })
            .take(rp.packages as usize)
            .collect()
    }

    /// The name of settlement `s`, or "another settlement".
    fn settlement_name(&self, ctx: &Ctx, s: PermanentId) -> String {
        ctx.land
            .settlements
            .iter()
            .find(|x| x.id == s)
            .map_or_else(|| "another settlement".to_owned(), |x| x.name.clone())
    }

    /// An agreement's terms in words: "leave for Ashford's people to use the places Oakholt
    /// claims, for a year".
    pub(super) fn agreement_words(&self, ctx: &Ctx, a: &Agreement) -> String {
        let name = |s: u8| {
            self.polities
                .iter()
                .find(|p| p.id == a.polities[usize::from(s)])
                .map_or_else(
                    || "another settlement".to_owned(),
                    |p| self.settlement_name(ctx, p.settlement),
                )
        };
        let clauses: Vec<String> = a
            .clauses
            .iter()
            .map(|c| match *c {
                Clause::Leave { from } => format!(
                    "leave for {}'s people to use the places {} claims",
                    name(1 - from),
                    name(from)
                ),
            })
            .collect();
        let term = match a.term_days {
            0 => "until withdrawn".to_owned(),
            365 => "for a year".to_owned(),
            d if d % 365 == 0 => format!("for {} years", d / 365),
            d => format!("for {d} days"),
        };
        format!("{}, {term}", clauses.join(", and "))
    }

    /// Adds a chronicle entry for agreement `id` at side 0's hearth.
    fn chronicle_agreement(
        &mut self,
        ctx: &Ctx,
        id: PermanentId,
        step: AgreementStep,
        sentence: String,
    ) {
        let Some(a) = self.agreements.get(id) else {
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

    /// Side `side`'s negotiator puts agreement `id` to their polity's gathering as a law of
    /// template `k`, each household's stake its forecast of the agreed package now.
    fn sponsor(&mut self, ctx: &mut Ctx, id: PermanentId, side: u8, k: u16) {
        let Some(a) = self.agreements.get(id) else {
            return;
        };
        let s = usize::from(side);
        let (n, polity, other) = (a.negotiators[s], a.polities[s], a.polities[1 - s]);
        let gives = a.clauses.contains(&Clause::Leave { from: side });
        let gets = a.clauses.contains(&Clause::Leave { from: 1 - side });
        let (Some(pi), Some(oi)) = (
            self.polities.iter().position(|p| p.id == polity),
            self.polities.iter().position(|p| p.id == other),
        ) else {
            return;
        };
        let pp = &ctx.params.polity;
        let stakes: Vec<(PermanentId, f32)> = self
            .stakes_of(ctx, pi, oi)
            .iter()
            .map(|x| (x.household, (pp.w_gain * x.gain(gives, gets, pp)) as f32))
            .collect();
        let m = MoveOption {
            policy: k,
            levy_share: 0.0,
            issue: if side == 0 {
                IssueKind::ClaimedFromUs
            } else {
                IssueKind::TermsSought
            },
            nominee: None,
            sanction: Default::default(),
            hours: (0, 0),
            body: None,
            ends: None,
            own_gain: 0.0,
            followers_gain: 0.0,
            support: 0.5,
        };
        let creed = self.creed_for(ctx, n, k).map(|c| c.0);
        self.propose(ctx, pi, n, m, stakes, creed);
        let law = self.polities[pi].laws.last_mut().map(|l| {
            l.agreement = Some(id);
            l.id
        });
        if let Some(a) = self.agreements.get_mut(id) {
            a.laws[s] = law;
        }
    }

    /// The agreements' midnight (ADR-0020 §6), after yesterday's gatherings decided: a side whose
    /// gathering decided its law passed it, or failed it; a side not yet put to its gathering is,
    /// by its negotiator when the gathering is free and they may still propose, or fails it once
    /// the content's days to answer are past; an agreement both passed and each side heard of
    /// comes into force, or fails unanswered; one in force ends when its term runs out or a law
    /// of either side ends its own. The laws of one failed or ended lapse.
    pub(super) fn agreements_day(&mut self, ctx: &mut Ctx) {
        if self.agreements.list.is_empty() {
            return;
        }
        let day = ctx.now.day_index();
        let answer = ctx.params.relations.answer_days;
        let k = ctx
            .catalog
            .policies
            .iter()
            .position(|d| d.kind == PolicyKind::Agreement);
        for i in 0..self.agreements.list.len() {
            let a = self.agreements.list[i].clone();
            if !(a.open() || a.in_force()) {
                continue;
            }
            let ids = a.polities;
            let index = |p: &Population, s: usize| p.polities.iter().position(|x| x.id == ids[s]);
            let law_of = |s: usize| {
                let pi = index(self, s)?;
                let law = a.laws[s]?;
                self.polities[pi].laws.iter().find(|l| l.id == law)
            };
            // A law of either side that another law ended ends the agreement (or the offer).
            let superseded =
                (0..2).find(|&s| law_of(s).is_some_and(|l| l.status == LawStatus::Superseded));
            if let Some(s) = superseded {
                let by = Some(s as u8);
                self.end_agreement(ctx, i, AgreementState::Ended { day, by });
                continue;
            }
            if let AgreementState::InForce { since } = a.state {
                if a.term_days > 0 && day - since >= i64::from(a.term_days) {
                    self.end_agreement(ctx, i, AgreementState::Ended { day, by: None });
                }
                continue;
            }
            // Offered: what each side's gathering decided.
            let mut failed = None;
            let mut passed = a.passed;
            for (s, at) in passed.iter_mut().enumerate() {
                if at.is_some() {
                    continue;
                }
                let Some(l) = law_of(s) else {
                    continue;
                };
                match l.outcome {
                    Some(Outcome::Passed) => {
                        *at = Some(l.decided.map_or(day, |t| t.day_index()));
                    }
                    Some(Outcome::NoQuorum) => failed = Some(Failure::TooFew(s as u8)),
                    Some(Outcome::Failed | Outcome::Tied) => {
                        failed = Some(Failure::TurnedDown(s as u8));
                    }
                    None => {}
                }
            }
            self.agreements.list[i].passed = passed;
            if let Some(why) = failed {
                self.end_agreement(ctx, i, AgreementState::Failed { day, why });
                continue;
            }
            // A side not yet put to its gathering.
            for s in 0..2 {
                if a.laws[s].is_some() {
                    continue;
                }
                let free = index(self, s).is_some_and(|pi| {
                    let p = &self.polities[pi];
                    p.agenda().is_none()
                        && p.gathering.is_none()
                        && self
                            .body_of(ctx, pi)
                            .iter()
                            .any(|m| m.0 == a.negotiators[s])
                });
                if free && let Some(k) = k {
                    self.sponsor(ctx, a.id, s as u8, k as u16);
                } else if day - a.made > answer {
                    let why = Failure::NeverCalled(s as u8);
                    self.end_agreement(ctx, i, AgreementState::Failed { day, why });
                    break;
                }
            }
            let a = &self.agreements.list[i];
            if !a.open() {
                continue;
            }
            if let [Some(p0), Some(p1)] = a.passed {
                if a.heard.iter().all(Option::is_some) {
                    let id = a.id;
                    self.agreements.list[i].state = AgreementState::InForce { since: day };
                    let words = self.agreement_words(ctx, &self.agreements.list[i]);
                    let (a_name, b_name) = self.agreement_names(ctx, i);
                    let sentence = format!(
                        "The agreement between {a_name} and {b_name} came into force, each \
                         having heard that the other's gathering passed it: {words}."
                    );
                    self.chronicle_agreement(ctx, id, AgreementStep::InForce, sentence);
                } else if day - p0.max(p1) > answer
                    && let Some(s) = a.heard.iter().position(Option::is_none)
                {
                    let why = Failure::Unanswered(s as u8);
                    self.end_agreement(ctx, i, AgreementState::Failed { day, why });
                }
            }
        }
        // The laws of agreements failed or ended lapse, once decided.
        for a in &self.agreements.list {
            if a.open() || a.in_force() {
                continue;
            }
            for (s, law) in a.laws.iter().enumerate() {
                let Some(law) = *law else {
                    continue;
                };
                if let Some(p) = self.polities.iter_mut().find(|p| p.id == a.polities[s])
                    && let Some(l) = p.law_mut(law)
                    && l.status == LawStatus::InForce
                {
                    l.status = LawStatus::Lapsed;
                }
            }
        }
    }

    /// The names of agreement `i`'s two settlements, side 0's first.
    fn agreement_names(&self, ctx: &Ctx, i: usize) -> (String, String) {
        let a = &self.agreements.list[i];
        let name = |s: usize| {
            self.polities
                .iter()
                .find(|p| p.id == a.polities[s])
                .map_or_else(
                    || "another settlement".to_owned(),
                    |p| self.settlement_name(ctx, p.settlement),
                )
        };
        (name(0), name(1))
    }

    /// Agreement `i` failed or ended, as `state` says.
    fn end_agreement(&mut self, ctx: &Ctx, i: usize, state: AgreementState) {
        self.agreements.list[i].state = state;
        let (a_name, b_name) = self.agreement_names(ctx, i);
        let side = |s: u8| if s == 0 { &a_name } else { &b_name };
        let (step, sentence) = match state {
            AgreementState::Failed { why, .. } => {
                let why = match why {
                    Failure::NoTerms => "no terms were agreed".to_owned(),
                    Failure::TurnedDown(s) => {
                        format!("the gathering at {} turned it down", side(s))
                    }
                    Failure::TooFew(s) => {
                        format!("too few came to the gathering at {}", side(s))
                    }
                    Failure::NeverCalled(s) => {
                        format!("nobody put it to the gathering at {} in time", side(s))
                    }
                    Failure::Unanswered(s) => format!(
                        "{} never heard in time that the other's gathering passed it",
                        side(s)
                    ),
                };
                (
                    AgreementStep::Failed,
                    format!("The agreement {a_name} and {b_name} sought failed: {why}."),
                )
            }
            AgreementState::Ended { by, .. } => (
                AgreementStep::Ended,
                match by {
                    None => {
                        format!("The agreement between {a_name} and {b_name} ran its term.")
                    }
                    Some(s) => format!(
                        "The agreement between {a_name} and {b_name} ended: a law at {} \
                         ended it.",
                        side(s)
                    ),
                },
            ),
            _ => return,
        };
        let id = self.agreements.list[i].id;
        self.chronicle_agreement(ctx, id, step, sentence);
    }

    /// `a` and `b` keep company at the hearth (ADR-0020 §2): one who knows their polity's
    /// gathering passed an agreement still awaiting word, with the polity the other lives in,
    /// tells them, and that side has heard of it.
    pub(crate) fn share_agreement_word(&mut self, ctx: &Ctx, a: PermanentId, b: PermanentId) {
        if !self.agreements.list.iter().any(Agreement::open) {
            return;
        }
        let polity_of = |p: PermanentId| {
            self.person(p)
                .and_then(|x| self.household(x.household))
                .and_then(|x| x.settlement)
                .and_then(|s| self.polity_of(s))
        };
        let (Some(pa), Some(pb)) = (polity_of(a), polity_of(b)) else {
            return;
        };
        if pa == pb {
            return;
        }
        let day = ctx.now.day_index();
        for (teller, ti, li) in [(a, pa, pb), (b, pb, pa)] {
            let (tid, lid) = (self.polities[ti].id, self.polities[li].id);
            for k in 0..self.agreements.list.len() {
                let x = &self.agreements.list[k];
                let (Some(ts), Some(ls)) = (x.side_of(tid), x.side_of(lid)) else {
                    continue;
                };
                let (ts, ls) = (usize::from(ts), usize::from(ls));
                if !x.open() || ts == ls || x.passed[ts].is_none() || x.heard[ls].is_some() {
                    continue;
                }
                let knows = x.laws[ts].is_some_and(|law| {
                    self.polities[ti]
                        .laws
                        .iter()
                        .any(|l| l.id == law && l.knows(teller))
                });
                if knows {
                    self.agreements.list[k].heard[ls] = Some(day);
                }
            }
        }
    }
}
