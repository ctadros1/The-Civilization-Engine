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
};
use crate::polity::{Gathering, Outcome as Decided, PolicyKind, Sanction, Stance};

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
        let (now, params) = (ctx.now, ctx.params);
        let day = now.day_index();
        let Some(settlement) = self.household(household).and_then(|x| x.settlement) else {
            return;
        };
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
        });
        self.put_cases(pi, day, params.polity.notice_days);
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
            "brought a case before the gathering at {name}: that {} took food from {whose}; \
             {saw}.",
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

    /// Puts polity `pi`'s open cases that no gathering is to hear before the one called, or calls
    /// one `notice_days` on for them.
    pub(super) fn put_cases(&mut self, pi: usize, day: i64, notice_days: u32) {
        let settlement = self.polities[pi].settlement;
        let waiting: Vec<u32> = {
            let on = self.polities[pi]
                .gathering
                .as_ref()
                .map(|g| g.cases.as_slice())
                .unwrap_or_default();
            self.order
                .cases
                .iter()
                .filter(|c| c.settlement == settlement && c.stage == CaseStage::Open)
                .map(|c| c.id)
                .filter(|id| !on.contains(id))
                .collect()
        };
        if waiting.is_empty() {
            return;
        }
        let polity = &mut self.polities[pi];
        match polity.gathering.as_mut() {
            Some(g) if g.day >= day => g.cases.extend(waiting),
            // One already sat and waits to be decided: the cases go to the next.
            Some(_) => {}
            None => {
                polity.gathering = Some(Gathering {
                    law: None,
                    day: day + i64::from(notice_days.max(1)),
                    stakes: Vec::new(),
                    present: Vec::new(),
                    cases: waiting,
                });
            }
        }
    }

    /// The gathering `g` of polity `pi` hears case `id` (ADR-0015 §4): each member who came stands
    /// for a finding or against it, by what they believe (or make of the accounts told there),
    /// what their household stands to gain or lose by it, and their regard for the one who brought
    /// it against their regard for the accused; the body decides by its rule. A finding imposes
    /// the law's bundle as obligations on the accused's household, and those who came believe it.
    pub(super) fn hear_case(&mut self, ctx: &mut Ctx, pi: usize, g: &Gathering, id: u32) {
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
        let need_of = |h: PermanentId| {
            self.household(h).map_or(1, |x| x.members.len().max(1)) as f64
                * params.household.daily_kcal_per_person
        };
        let debtor_need = need_of(accused_hh);
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
        let members = self.members_of(settlement, now, params);
        let regard = |p: PermanentId, q: PermanentId| {
            if p == q {
                1.0
            } else {
                self.ties.regard(p, q, day, &params.ties).clamp(0.0, 1.0)
            }
        };
        let stances: Vec<CaseStance> = g
            .present
            .iter()
            .filter_map(|&p| {
                let i = members.binary_search_by_key(&p, |m| m.0).ok()?;
                let household = members[i].1;
                let stake = if household == c.accuser {
                    gain
                } else if household == accused_hh {
                    -loss
                } else {
                    0.0
                };
                let believes = self
                    .order
                    .belief(p, c.incident)
                    .is_some_and(|b| b.taker == Some(c.accused));
                let belief = belief_points(believes, c.leads.len(), cp);
                let regard_points = pp.w_regard * (regard(p, c.by) - regard(p, c.accused));
                let stance = if p == c.by {
                    Stance::Support
                } else if p == c.accused {
                    Stance::Oppose
                } else {
                    let s = stake + belief + regard_points;
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
                })
            })
            .collect();
        let eligible = members.len() as u32;
        let body = self.polities[pi].body;
        let case = &mut self.order.cases[k];
        case.stances = stances;
        case.eligible = eligible;
        case.heard = Some(now);
        let (present, support, oppose) = case.counts();
        let decided = body.decide(eligible, present, support, oppose);
        case.stage = match decided {
            Decided::Passed => CaseStage::Found,
            Decided::Failed | Decided::Tied => CaseStage::NotFound,
            Decided::NoQuorum => CaseStage::Unheard,
        };
        let tally = format!("{support} for, {oppose} against; {present} of {eligible} adults came");
        let words = match decided {
            Decided::Passed => {
                // The household owes; the one found is the one sent away.
                let owed = Sanction {
                    exile: false,
                    ..sanction
                };
                let sent = if sanction.exile {
                    format!(" {accused_name} is sent from the valley.")
                } else {
                    String::new()
                };
                format!(
                    "The gathering at {name} found that {accused_name} took food from \
                     {accuser_words}: {tally}. {accused_name}'s household owes it back{}.{sent}",
                    owed.words()
                )
            }
            Decided::Failed => format!(
                "The gathering at {name} did not find that {accused_name} took food from \
                 {accuser_words}: {tally}."
            ),
            Decided::Tied => format!(
                "The gathering at {name} was evenly split on whether {accused_name} took food \
                 from {accuser_words}, so it found nothing: {tally}."
            ),
            Decided::NoQuorum => format!(
                "Too few came to the gathering at {name} to hear the case {accuser_words} \
                 brought against {accused_name}: {present} of {eligible} adults, where {} were \
                 needed.",
                body.quorum(eligible)
            ),
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
        if sanction.exile {
            self.exile(ctx, c.accused);
            self.order.cases[k].exiled = true;
        }
        for &p in &g.present {
            if p == c.accused
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
