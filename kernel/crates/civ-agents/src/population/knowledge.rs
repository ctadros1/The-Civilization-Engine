//! What people know, in the running population (ADR-0008): the technique each piece of work
//! needs, who may do it, learning by working beside someone who knows, children brought up with
//! their household's work, each settlement's record, loss with the last who knew, and the god
//! tool that introduces a technique.

use civ_core::{PermanentId, Rng64, SimTime};

use super::{Ctx, Population, cell_of};
use crate::decide;
use crate::history::{
    ChronicleKind, FOUND_AGAIN, FOUND_FIRST_ANYWHERE, FOUND_FIRST_HERE, FOUND_KNOWN_HERE,
};
use crate::knowledge::{KnowledgeEvent, KnowledgeEventKind, could_find, find_chance};
use crate::params::{ActivityDef, Behavior, Catalog, PeopleParams};
use crate::person::{Know, KnowSource, Person, Step, Target};

/// Whether a person may do a piece of work that needs a technique (ADR-0008 §4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Gate {
    /// They know it.
    Open,
    /// They do not know it, but may work beside this person, who does, and learn.
    Learner(PermanentId),
    /// They do not know it, and nobody they could learn from is at the work.
    Closed,
}

/// Flags of a loss, in its chronicle entry's number (07-02 §5.4): someone there still knows of it
/// or is learning it...
pub const LOST_AWARE: f64 = 1.0;
/// ...and goods or buildings made with it remain there.
pub const LOST_MADE_REMAIN: f64 = 2.0;
/// ...and it is still known in another settlement where someone left there has kin or a tie (M5b
/// slice AR; 07-02 §5.4: no local practitioner, but an accessible one).
pub const LOST_KNOWN_NEAR: f64 = 4.0;

impl Population {
    /// The technique the work of activity `def` aimed at `target` by a member of `household`
    /// needs (ADR-0008 §1): the activity's own or its recipe's, the program's of the building it
    /// works on (a new home's: the one the household plans), or the recipe's of the workshop that
    /// hires it.
    pub(crate) fn technique_for(
        &self,
        ctx: &Ctx,
        def: &ActivityDef,
        target: Target,
        household: PermanentId,
    ) -> Option<usize> {
        let catalog = ctx.catalog;
        match def.behavior {
            Behavior::Build => match target {
                Target::Building(b) => ctx
                    .land
                    .buildings
                    .iter()
                    .find(|x| x.id == b)
                    .and_then(|x| catalog.building_index(&x.spec.program))
                    .and_then(|i| catalog.buildings[i].technique),
                // The home it plans, or else the first it may build.
                Target::NewBuilding => self
                    .planned_home(household)
                    .and_then(|spec| catalog.building_index(&spec.program))
                    .or_else(|| ctx.params.build.programs.first().copied())
                    .and_then(|p| catalog.buildings.get(p))
                    .and_then(|b| b.technique),
                _ => None,
            },
            // Work on a well needs its system's technique (M6a slice AY, step three).
            Behavior::Well => match target {
                Target::Well(w) => ctx
                    .land
                    .wells
                    .get(w)
                    .and_then(|x| catalog.wells.get(x.system))
                    .and_then(|d| d.technique),
                _ => None,
            },
            Behavior::Hire => match target {
                Target::Firm(f) => self
                    .firm(f)
                    .and_then(|f| f.wage)
                    .and_then(|w| catalog.activities.get(usize::from(w.activity)))
                    .and_then(|a| catalog.technique_of(a)),
                _ => None,
            },
            _ => catalog.technique_of(def),
        }
    }

    /// The most skilled member of household `household` who knows technique `t` (ties by id),
    /// other than `except`, and among those at activity `at_work` now if any is.
    fn knower_in(
        &self,
        ctx: &Ctx,
        household: PermanentId,
        t: usize,
        except: Option<PermanentId>,
        at_work: Option<(u16, Target)>,
    ) -> Option<PermanentId> {
        let hh = self.household(household)?;
        let skill = ctx.catalog.techniques.get(t).and_then(|d| d.domain);
        let level = |q: &Person| skill.map_or(0.0, |k| q.skill(k));
        hh.members
            .iter()
            .filter(|m| Some(**m) != except)
            .filter_map(|m| self.person(*m))
            .filter(|q| q.knows(t) && at_work.is_none_or(|(def, target)| working(q, def, target)))
            .max_by(|a, b| level(a).total_cmp(&level(b)).then(b.id.cmp(&a.id)))
            .map(|q| q.id)
    }

    /// Who brings person `who` up with technique `t` (ADR-0008 §4): someone in their household
    /// who knows it, if `t` is learnt in upbringing and `who` is of an age to learn it at home
    /// ([`crate::knowledge::upbringing_ages`]).
    fn upbringer(&self, ctx: &Ctx, who: &Person, t: usize) -> Option<PermanentId> {
        let (from, until) =
            crate::knowledge::upbringing_ages(ctx.catalog, t, ctx.params.family.independent_age)?;
        let age = who.age_years(ctx.now);
        if age < from || age >= until {
            return None;
        }
        self.knower_in(ctx, who.household, t, Some(who.id), None)
    }

    /// Whether person `who`, of household `household`, may learn technique `t` beside a member
    /// of it in a household-resolved day (M10c slice EB, first part; ADR-0035 §5): someone of
    /// it knows it, and fewer of it are learning it today than its knowers' room for learners.
    /// The most skilled knower is named; whether they work at it is settled when the day is
    /// lived.
    fn alongside_teacher(
        &self,
        ctx: &Ctx,
        who: PermanentId,
        household: PermanentId,
        t: usize,
    ) -> Option<PermanentId> {
        let hh = self.household(household)?;
        let knowers = hh
            .members
            .iter()
            .filter(|&&m| m != who)
            .filter_map(|&m| self.person(m))
            .filter(|q| q.knows(t))
            .count();
        let learning = self.alongside.learners(household, t, who);
        let room = knowers * ctx.params.knowledge.max_learners as usize;
        (learning < room)
            .then(|| self.knower_in(ctx, household, t, Some(who), None))
            .flatten()
    }

    /// Those who worked beside a knower of their household in the household-resolved day just
    /// lived (M10c slice EB, first part; ADR-0035 §5, amending ADR-0008 §4): each learner of a
    /// technique at a household is credited the hours they gave it, up to their share of what
    /// its knowers there gave it times the learners each may have; learnt so, they know it from
    /// that household's most skilled knower, whom they saw at the craft.
    pub(super) fn credit_alongside(&mut self, ctx: &mut Ctx) {
        let held = std::mem::take(&mut self.alongside);
        let max = f64::from(ctx.params.knowledge.max_learners);
        let now = ctx.now;
        for (household, t, learners) in held.groups() {
            let given = held.held(household, t);
            if given <= 0.0 || learners.is_empty() {
                continue;
            }
            let share = given * max / learners.len() as f64;
            for (who, hours) in learners {
                let credit = hours.min(share);
                let Some(teacher) = self.knower_in(ctx, household, t, Some(who), None) else {
                    continue;
                };
                let learn_h = ctx.catalog.techniques.get(t).map_or(1.0, |d| d.learn_h);
                let learnt = self
                    .person_mut_by_id(who)
                    .is_some_and(|p| !p.knows(t) && p.learn(t, credit, teacher, learn_h, now));
                self.note_tie(
                    ctx,
                    who,
                    teacher,
                    crate::ties::Act::LearnedFrom,
                    credit,
                    0.0,
                );
                if learnt {
                    self.on_known(ctx, who, t, KnowSource::Taught(teacher));
                }
            }
        }
    }

    /// Households whose day of the year it is weigh sending a child to learn a craft (M10c
    /// slice EB, second part); none while no market remembers a trade.
    pub(super) fn enrol_reviews(&mut self, ctx: &mut Ctx, day: i64) {
        self.prune_untaught(day);
        if self.markets.iter().all(|m| m.recent.is_empty()) {
            return;
        }
        let due: Vec<PermanentId> = self
            .households
            .iter()
            .map(|(_, x)| x.id)
            .filter(|h| {
                Rng64::from_key(&[ctx.seed, PURPOSE_ENROL, h.get()]).next_u64()
                    % civ_core::time::DAYS_PER_YEAR as u64
                    == day.rem_euclid(civ_core::time::DAYS_PER_YEAR) as u64
            })
            .collect();
        for h in due {
            self.review_enrolment(ctx, h);
        }
    }

    /// The technique working a recipe that makes good `g` needs, if any: the first such recipe's
    /// own, or the activity's that works it.
    fn technique_making(catalog: &Catalog, g: usize) -> Option<usize> {
        catalog
            .recipes
            .iter()
            .enumerate()
            .filter(|(_, r)| r.outputs.iter().any(|&(o, _)| o == g))
            .find_map(|(ri, r)| {
                r.technique.or_else(|| {
                    catalog
                        .activities
                        .iter()
                        .find(|a| a.recipe == Some(ri))
                        .and_then(|a| catalog.technique_of(a))
                })
            })
    }

    /// Household `h` weighs sending a child to learn a craft (M10c slice EB, second part;
    /// ADR-0035 §5; brief §3.2: a household sends a child for what it can see learning brings):
    /// its oldest child of the study activity's age and under the age of independence not
    /// enrolled, to learn the technique of a good it bought at least twice from one household of
    /// its market, in the trades the market remembers, where none of it knows that technique or is
    /// enrolled to learn it and someone grown of the seller's household knows it; with the seller's most skilled knower, kin
    /// asking no fee and others the fee for a day of care for each day of study, which it must
    /// hold. One enrolment a review.
    fn review_enrolment(&mut self, ctx: &mut Ctx, h: PermanentId) {
        let catalog = ctx.catalog;
        let Some(x) = self.household(h) else {
            return;
        };
        let Some(market) = x.settlement.and_then(|s| self.market(s)) else {
            return;
        };
        let now = ctx.now;
        let from = catalog
            .activities
            .iter()
            .find(|a| a.behavior == Behavior::Study)
            .map_or(f64::INFINITY, |a| a.min_age_years);
        let until = ctx.params.family.independent_age;
        let Some(child) = x
            .members
            .iter()
            .filter_map(|&m| self.person(m))
            .filter(|p| (from..until).contains(&p.age_years(now)))
            .filter(|p| !self.enrolments.iter().any(|e| e.learner == p.id))
            .min_by_key(|p| (p.born, p.id))
            .map(|p| p.id)
        else {
            return;
        };
        let mut bought: std::collections::BTreeMap<(PermanentId, u16), u32> = Default::default();
        for t in market
            .recent
            .iter()
            .filter(|t| t.buyer == h && t.seller != h)
        {
            *bought.entry((t.seller, t.good)).or_default() += 1;
        }
        let knows = |pop: &Self, members: &[PermanentId], t: usize| {
            members
                .iter()
                .any(|&m| pop.person(m).is_some_and(|p| p.knows(t)))
        };
        let members = x.members.clone();
        let grown = 16.0;
        let mut chosen = None;
        for ((seller, good), n) in bought {
            if n < 2 {
                continue;
            }
            let Some(t) = Self::technique_making(catalog, usize::from(good)) else {
                continue;
            };
            // Nor a technique someone of it is already enrolled to learn.
            if knows(self, &members, t)
                || self
                    .enrolments
                    .iter()
                    .any(|e| usize::from(e.technique) == t && members.contains(&e.learner))
            {
                continue;
            }
            // A school the polity keeps takes the child first, without a fee (M10c slice EB,
            // third part).
            if let Some(teacher) = self.school_for(h, t) {
                chosen = Some((teacher, seller, t, true));
                break;
            }
            let Some(teacher) = self.knower_in(ctx, seller, t, None, None) else {
                continue;
            };
            if self
                .person(teacher)
                .is_none_or(|p| p.age_years(now) < grown)
            {
                continue;
            }
            chosen = Some((teacher, seller, t, false));
            break;
        }
        // Else letters, for what they visibly bring (M10c slice EE, fourth part; brief
        // m10-services §3.2): where its polity pays the holder of an office that asks a
        // technique, the child is sent to learn it, or the first of what it requires the child
        // does not know, from a school, or else from the holder.
        if chosen.is_none()
            && let Some((holder, office_t)) = self.paid_office_technique(ctx, h)
        {
            let t = self.first_needed(catalog, child, office_t);
            let enrolled = self
                .enrolments
                .iter()
                .any(|e| usize::from(e.technique) == t && members.contains(&e.learner));
            let holder_home = self.person(holder).map(|p| p.household);
            if !knows(self, &members, t) && !enrolled {
                if let Some(teacher) = self.school_for(h, t) {
                    chosen = holder_home.map(|s| (teacher, s, t, true));
                } else if self.person(holder).is_some_and(|p| p.knows(t))
                    && holder_home.is_some_and(|s| s != h)
                {
                    chosen = holder_home.map(|s| (holder, s, t, false));
                }
            }
        }
        let Some((teacher, seller, t, school)) = chosen else {
            return;
        };
        let Ok(technique) = u16::try_from(t) else {
            return;
        };
        self.refresh_kin();
        let kin = school
            || members.iter().any(|&m| {
                self.close_kin(m)
                    .iter()
                    .any(|k| self.person(*k).is_some_and(|q| q.household == seller))
            });
        let Some(x) = self.household(h) else {
            return;
        };
        let fee = if kin { 0.0 } else { self.call_fee(ctx, x) };
        let (params, goods) = (ctx.params, &catalog.goods);
        if crate::person::stock_kcal(&super::stores_now(x, now, params, goods), goods) < fee {
            // Remembered a year, for a school (M10c slice EB, third part).
            self.note_untaught(h, technique, now.day_index());
            return;
        }
        self.untaught
            .retain(|u| !(u.household == h && u.technique == technique));
        self.enrolments.push(crate::knowledge::Enrolment {
            teacher,
            learner: child,
            technique,
            fee_kcal: fee as f32,
            payer: h,
            since: now.day_index(),
            paid: -1,
        });
    }

    /// The holder of an office of household `h`'s polity that asks a technique, and the
    /// technique, where a law in force pays common work (M10c slice EE, fourth part): the office
    /// visibly paid.
    fn paid_office_technique(&self, ctx: &Ctx, h: PermanentId) -> Option<(PermanentId, usize)> {
        let pi = self.polity_of(self.household(h)?.settlement?)?;
        self.work_pay_of(ctx, pi)?;
        self.polities[pi].laws.iter().find_map(|l| {
            if l.status != crate::polity::LawStatus::InForce {
                return None;
            }
            let t = ctx
                .catalog
                .policies
                .get(usize::from(l.policy))?
                .holder_needs?;
            Some((l.holder?, t))
        })
    }

    /// The first technique person `child` would learn toward `t`: `t` where they know what it
    /// requires, else the first of its first requirement they do not know, and so on.
    fn first_needed(&self, catalog: &Catalog, child: PermanentId, t: usize) -> usize {
        let knows = |u: usize| self.person(child).is_some_and(|p| p.knows(u));
        let mut at = t;
        for _ in 0..8 {
            let Some(def) = catalog.techniques.get(at) else {
                break;
            };
            if def.requires.is_empty() || def.requires.iter().any(|r| r.iter().all(|&u| knows(u))) {
                break;
            }
            match def.requires[0].iter().copied().find(|&u| !knows(u)) {
                Some(u) => at = u,
                None => break,
            }
        }
        at
    }

    /// Household `h` weighs sending a child to learn a craft now: for tests.
    #[doc(hidden)]
    pub fn review_enrolment_for_tests(&mut self, ctx: &mut Ctx, h: PermanentId) {
        self.review_enrolment(ctx, h);
    }

    /// The technique person `who` teaches today, if learners are enrolled with them for one
    /// they know that the learner does not yet, and they have not held today's session (M10c
    /// slice EB, second part): the one with the most learners, counted up to its group, the
    /// first enrolled of two alike.
    pub(super) fn teach_facts(&self, ctx: &Ctx, who: PermanentId) -> Option<decide::TeachFacts> {
        if self.enrolments.is_empty() {
            return None;
        }
        let p = self.person(who)?;
        let session = SESSION_MIN as f32;
        let mut best: Option<decide::TeachFacts> = None;
        for e in self.enrolments.iter().filter(|e| e.teacher == who) {
            let t = e.technique;
            if !p.knows(usize::from(t))
                || crate::knowledge::tallied(&self.taught, who, t) >= session
                || best.is_some_and(|b| b.technique == t)
            {
                continue;
            }
            let group = ctx
                .catalog
                .techniques
                .get(usize::from(t))
                .map_or(1, |d| u32::from(d.group.max(1)));
            let learners = self
                .enrolments
                .iter()
                .filter(|o| o.teacher == who && o.technique == t)
                .filter(|o| {
                    self.person(o.learner)
                        .is_some_and(|q| !q.knows(usize::from(t)))
                })
                .count() as u32;
            let learners = learners.min(group);
            if learners > 0 && best.is_none_or(|b| learners > b.learners) {
                best = Some(decide::TeachFacts {
                    technique: t,
                    learners,
                });
            }
        }
        best
    }

    /// The technique person `who` studies today, if they are enrolled to learn one they do not
    /// yet know and have not studied today's session, with where its teacher lives and the walk
    /// there from `field_key`'s travel field (M10c slice EB, second part).
    pub(super) fn study_facts(
        &self,
        ctx: &Ctx,
        who: PermanentId,
        field_key: PermanentId,
    ) -> Option<decide::StudyFacts> {
        if self.enrolments.is_empty() {
            return None;
        }
        let p = self.person(who)?;
        let e = self.enrolments.iter().find(|e| {
            e.learner == who
                && !p.knows(usize::from(e.technique))
                && crate::knowledge::tallied(&self.studied, who, e.technique) < SESSION_MIN as f32
        })?;
        let home = self.household(self.person(e.teacher)?.household)?.home;
        let secs = self
            .homes
            .get(&field_key)?
            .reach
            .seconds_to(cell_of(ctx.map, home))?;
        Some(decide::StudyFacts {
            technique: e.technique,
            at: home,
            walk_min: f64::from(secs) / 60.0,
        })
    }

    /// Learner `who` studied technique `t` today (M10c slice EB, second part): the household
    /// paying for it gives the teacher's the day's fee, once a day; one that cannot pay it all
    /// pays what it holds, and the enrolment ends.
    pub(super) fn pay_study_fee(&mut self, ctx: &Ctx, who: PermanentId, t: u16) {
        let day = ctx.now.day_index();
        let Some(i) = self
            .enrolments
            .iter()
            .position(|e| e.learner == who && e.technique == t && e.paid < day)
        else {
            return;
        };
        self.enrolments[i].paid = day;
        let e = self.enrolments[i];
        let fee = f64::from(e.fee_kcal);
        let Some(to) = self.person(e.teacher).map(|q| q.household) else {
            return;
        };
        if fee > 0.0 && self.pay_food(ctx, e.payer, to, fee) + 1e-6 < fee {
            self.enrolments
                .retain(|o| !(o.learner == who && o.technique == t));
        }
    }

    /// Yesterday's sessions are credited (M10c slice EB, second part; ADR-0035 §5, amending
    /// ADR-0008 §4): each learner who studied a technique is credited the hours they studied it,
    /// up to the hours their teacher taught it, shared beyond its group among those of that
    /// teacher who studied it; learnt so, they know it from their teacher. Enrolments end once
    /// the learner knows it, or either is no more or the teacher no longer knows it.
    pub(super) fn credit_study(&mut self, ctx: &mut Ctx) {
        if self.enrolments.is_empty() {
            self.taught.clear();
            self.studied.clear();
            return;
        }
        let (taught, studied) = (
            std::mem::take(&mut self.taught),
            std::mem::take(&mut self.studied),
        );
        let now = ctx.now;
        let list = self.enrolments.clone();
        // Schoolteachers are paid for yesterday's hours, where work is paid (M10c slice EB, third
        // part).
        self.pay_school(ctx, &taught, &studied, &list);
        for e in &list {
            let t = e.technique;
            let attended = crate::knowledge::tallied(&studied, e.learner, t);
            let held = crate::knowledge::tallied(&taught, e.teacher, t);
            if attended <= 0.0 || held <= 0.0 {
                continue;
            }
            let Some(def) = ctx.catalog.techniques.get(usize::from(t)) else {
                continue;
            };
            let came = list
                .iter()
                .filter(|o| o.teacher == e.teacher && o.technique == t)
                .filter(|o| crate::knowledge::tallied(&studied, o.learner, t) > 0.0)
                .count()
                .max(1);
            let group = usize::from(def.group.max(1));
            let share = if came <= group {
                held
            } else {
                held * group as f32 / came as f32
            };
            let hours = f64::from(attended.min(share)) / 60.0;
            let learn_h = def.learn_h;
            let learnt = self.person_mut_by_id(e.learner).is_some_and(|p| {
                !p.knows(usize::from(t)) && p.learn(usize::from(t), hours, e.teacher, learn_h, now)
            });
            self.note_tie(
                ctx,
                e.learner,
                e.teacher,
                crate::ties::Act::LearnedFrom,
                hours,
                0.0,
            );
            if learnt {
                self.on_known(
                    ctx,
                    e.learner,
                    usize::from(t),
                    KnowSource::Taught(e.teacher),
                );
            }
        }
        let alive = |pop: &Self, id: PermanentId| pop.person(id).is_some();
        let kept: Vec<_> = self
            .enrolments
            .iter()
            .copied()
            .filter(|e| {
                let t = usize::from(e.technique);
                alive(self, e.learner)
                    && self.person(e.teacher).is_some_and(|q| q.knows(t))
                    && self.person(e.learner).is_some_and(|q| !q.knows(t))
            })
            .collect();
        self.enrolments = kept;
    }

    /// Whether person `who` may do activity `def` aimed at `target`, which needs technique `t`
    /// (ADR-0008 §4). A child who has reached the age of a work learnt in upbringing does it as
    /// their household does, and comes to know it at once. Someone who does not know it may work
    /// beside a member of their household
    /// who does and is at that same work there now, while that teacher has fewer learners than
    /// the profile allows; a hired hand may learn at a workshop whose owners know it.
    pub(crate) fn gate(&self, ctx: &Ctx, who: &Person, t: usize, def: u16, target: Target) -> Gate {
        if who.knows(t) || self.upbringer(ctx, who, t).is_some() {
            return Gate::Open;
        }
        let hired = ctx
            .catalog
            .activities
            .get(usize::from(def))
            .is_some_and(|a| a.behavior == Behavior::Hire);
        if let (true, Target::Firm(f)) = (hired, target) {
            return self
                .firm(f)
                .and_then(|firm| self.knower_in(ctx, firm.owner, t, Some(who.id), None))
                .map_or(Gate::Closed, Gate::Learner);
        }
        // In a household-resolved day every other member is parked while one is planned: a
        // knower of the household stands in for one at the work (M10c slice EB, first part).
        if self.resolving {
            return self
                .alongside_teacher(ctx, who.id, who.household, t)
                .map_or(Gate::Closed, Gate::Learner);
        }
        let Some(hh) = self.household(who.household) else {
            return Gate::Closed;
        };
        let teachers = hh
            .members
            .iter()
            .filter(|m| **m != who.id)
            .filter_map(|m| self.person(*m))
            .filter(|q| q.knows(t) && working(q, def, target))
            .count();
        // Learners at that work or on their way to it.
        let learners = hh
            .members
            .iter()
            .filter(|m| **m != who.id)
            .filter_map(|m| self.person(*m))
            .filter(|q| !q.knows(t) && q.act.def == def && q.act.target == target)
            .count();
        let room = teachers * ctx.params.knowledge.max_learners as usize;
        match self.knower_in(ctx, who.household, t, Some(who.id), Some((def, target))) {
            Some(teacher) if learners < room => Gate::Learner(teacher),
            _ => Gate::Closed,
        }
    }

    /// After `hours` of person `who`'s work at activity `def` aimed at `target`, which needs
    /// technique `t` (ADR-0008 §4): someone who knows it has practised it; a learner counts the
    /// hours toward learning it, from the household member at that work (or failing them, any who
    /// knows it), or for hired work from the workshop's owners. A session begun beside a teacher
    /// counts whole even if the teacher finishes first: a simplification, as presence is checked
    /// only when the learner chooses the work.
    pub(crate) fn practise(
        &mut self,
        ctx: &mut Ctx,
        who: PermanentId,
        t: usize,
        def: u16,
        target: Target,
        hours: f64,
    ) {
        let now = ctx.now;
        let Some(p) = self.person(who) else {
            return;
        };
        if p.knows(t) {
            // In a household-resolved day, hours a knower gives it there are what those beside
            // them learn by (M10c slice EB, first part).
            if self.resolving {
                let household = p.household;
                self.alongside.give(household, t, hours);
            }
            if let Some(p) = self.person_mut_by_id(who) {
                p.practised(t, now);
            }
            return;
        }
        if let Some(from) = self.upbringer(ctx, p, t) {
            let source = KnowSource::Upbringing(from);
            if self
                .person_mut_by_id(who)
                .is_some_and(|p| p.come_to_know(t, source, now))
            {
                self.on_known(ctx, who, t, source);
            }
            return;
        }
        let household = p.household;
        let hired = ctx
            .catalog
            .activities
            .get(usize::from(def))
            .is_some_and(|a| a.behavior == Behavior::Hire);
        // Beside a knower of their household in a household-resolved day, the hours are held
        // until the day is lived (M10c slice EB, first part).
        if self.resolving && !hired {
            if self.alongside.is_learning(household, t, who)
                || self.alongside_teacher(ctx, who, household, t).is_some()
            {
                self.alongside.attend(household, t, who, hours);
            }
            return;
        }
        let teacher = match (hired, target) {
            (true, Target::Firm(f)) => self
                .firm(f)
                .and_then(|firm| self.knower_in(ctx, firm.owner, t, Some(who), None)),
            _ => self
                .knower_in(ctx, household, t, Some(who), Some((def, target)))
                .or_else(|| self.knower_in(ctx, household, t, Some(who), None)),
        };
        let Some(teacher) = teacher else {
            return;
        };
        let learn_h = ctx.catalog.techniques.get(t).map_or(1.0, |d| d.learn_h);
        let learnt = self
            .person_mut_by_id(who)
            .is_some_and(|p| p.learn(t, hours, teacher, learn_h, now));
        // The learner saw the teacher's craft at work (ADR-0014 §2).
        self.note_tie(ctx, who, teacher, crate::ties::Act::LearnedFrom, hours, 0.0);
        if learnt {
            self.on_known(ctx, who, t, KnowSource::Taught(teacher));
        }
    }

    /// After `hours` of person `who`'s work at activity `def` aimed at `target`, begun at
    /// `started` (ADR-0008 §3). A session of trying counts all its hours toward the technique it
    /// was aimed at, the more for someone already aware of it; routine work counts the share of
    /// its hours given to experiment toward each technique its practice can find. Each is a
    /// draw, keyed by person, technique (by its content id, so content added beside it moves no
    /// draw) and session, against the chance those hours find it,
    /// for a technique they could find: not known, a route of what it needs known, its goods at
    /// home. The finder knows it at once, and the chronicle says what kind of find it was.
    pub(crate) fn discover(
        &mut self,
        ctx: &mut Ctx,
        who: PermanentId,
        (def, target): (u16, Target),
        hours: f64,
        started: SimTime,
    ) {
        let (now, catalog, k) = (ctx.now, ctx.catalog, &ctx.params.knowledge);
        let trying = catalog
            .activities
            .get(usize::from(def))
            .is_some_and(|a| a.behavior == Behavior::Try);
        let Some(p) = self.person(who) else {
            return;
        };
        let mut found: Vec<usize> = Vec::new();
        let mut stores: Option<Vec<f64>> = None;
        // A blessing or a curse moves their own chance of finding (M4c slice AJ).
        let luck = self.influences.luck(who, now.day_index());
        let mut turned: Vec<usize> = Vec::new();
        for (t, d) in catalog.techniques.iter().enumerate() {
            let share = if trying {
                if target != Target::Technique(t as u16) {
                    continue;
                }
                if p.know(t).is_some() {
                    k.aware_try_factor
                } else {
                    1.0
                }
            } else if d.tried_in.contains(&usize::from(def)) {
                k.experiment_share
            } else {
                continue;
            };
            if p.knows(t) {
                continue;
            }
            let stores = stores.get_or_insert_with(|| {
                self.household(p.household).map_or_else(Vec::new, |x| {
                    super::stores_now(x, now, ctx.params, &catalog.goods)
                })
            });
            if !could_find(catalog, t, &|u| p.knows(u), stores) {
                continue;
            }
            // What it needs found in the ground, its settlement must know (M10a slice DS).
            let settlement = self.household(p.household).and_then(|x| x.settlement);
            if !d.needs_known.iter().all(|&g| {
                settlement.is_some_and(|s| {
                    ctx.land.deposits.iter().any(|x| {
                        usize::from(x.body.good) == g
                            && x.left_kg() > 0.0
                            && self.knows_deposit(s, x.id)
                    })
                })
            }) {
                continue;
            }
            let chance = find_chance(hours * share, d.e50_h);
            let key = [
                ctx.seed,
                crate::found::PURPOSE_FIND,
                who.get(),
                crate::found::content_key(&d.id),
                started.minutes() as u64,
            ];
            let u = Rng64::from_key(&key).next_f64();
            let mut hit = u < chance;
            if let Some(luck) = luck {
                let moved = u < (chance * luck.fortune()).min(1.0);
                if moved != hit {
                    turned.push(t);
                }
                hit = moved;
            }
            if hit {
                found.push(t);
            }
        }
        let settlement = self.household(p.household).and_then(|x| x.settlement);
        if let Some(luck) = luck {
            for &t in &turned {
                self.note_turned(ctx, who, luck, Some(t));
            }
        }
        for t in found {
            let kind = self.find_kind(settlement, t);
            if self
                .person_mut_by_id(who)
                .is_some_and(|p| p.come_to_know(t, KnowSource::Found, now))
            {
                self.on_known(ctx, who, t, KnowSource::Found);
                let name = catalog.techniques[t].name.clone();
                self.chronicle_push(
                    now,
                    ChronicleKind::TechniqueFound,
                    vec![who],
                    settlement,
                    None,
                    kind as f64,
                    name,
                );
            }
        }
    }

    /// What kind of find of technique `t` a find in settlement `s` would be: the first anyone
    /// made, the first there of what is known elsewhere, of what others there know already, or
    /// of what was lost there.
    fn find_kind(&self, s: Option<PermanentId>, t: usize) -> i64 {
        let ever = |s: Option<PermanentId>| {
            self.knowledge
                .iter()
                .any(|e| usize::from(e.technique) == t && s.is_none_or(|s| e.settlement == s))
        };
        if !ever(None) {
            FOUND_FIRST_ANYWHERE
        } else if s.is_none() || !ever(s) {
            FOUND_FIRST_HERE
        } else if s.is_some_and(|s| self.known_in(s, t)) {
            FOUND_KNOWN_HERE
        } else {
            FOUND_AGAIN
        }
    }

    /// A person came to know technique `t` from `source`: the settlement's record notes it if
    /// nobody there knew it, and the chronicle notes learning a craft (not everyday work brought
    /// up with) and the observer's doing.
    pub(super) fn on_known(&mut self, ctx: &Ctx, who: PermanentId, t: usize, source: KnowSource) {
        let now = ctx.now;
        let settlement = self
            .person(who)
            .and_then(|p| self.household(p.household))
            .and_then(|x| x.settlement);
        if let Some(s) = settlement {
            self.note_known(now, s, who, t, source, None);
        }
        let Some(def) = ctx.catalog.techniques.get(t) else {
            return;
        };
        if let (KnowSource::Taught(teacher), false) = (source, def.upbringing) {
            self.chronicle_push(
                now,
                ChronicleKind::TechniqueLearned,
                vec![who, teacher],
                settlement,
                None,
                0.0,
                def.name.clone(),
            );
        }
    }

    /// Notes in settlement `s`'s record that `who` came to know technique `t`, if it was not
    /// known there (never, or not since it was lost); `elsewhere`, the settlement they brought it
    /// from, if they came from one (M5b slice AR).
    fn note_known(
        &mut self,
        now: SimTime,
        s: PermanentId,
        who: PermanentId,
        t: usize,
        source: KnowSource,
        elsewhere: Option<PermanentId>,
    ) {
        if self.known_in(s, t) {
            return;
        }
        self.knowledge.push(KnowledgeEvent {
            at: now,
            settlement: s,
            technique: t as u16,
            person: who,
            kind: KnowledgeEventKind::Known(source),
            elsewhere,
        });
    }

    /// Whether settlement `s`'s record says technique `t` is known there now.
    pub fn known_in(&self, s: PermanentId, t: usize) -> bool {
        self.knowledge
            .iter()
            .rev()
            .find(|e| e.settlement == s && usize::from(e.technique) == t)
            .is_some_and(|e| matches!(e.kind, KnowledgeEventKind::Known(_)))
    }

    /// People who have just arrived in settlement `s` (a founding band or a family sent): each
    /// technique one of them knows that was not known there is noted, as brought by its eldest
    /// knower among them.
    pub fn note_arrivals(
        &mut self,
        catalog: &Catalog,
        now: SimTime,
        s: PermanentId,
        people: &[PermanentId],
    ) {
        for t in 0..catalog.techniques.len() {
            let eldest = people
                .iter()
                .filter_map(|id| self.person(*id))
                .filter(|p| p.knows(t))
                .min_by_key(|p| (p.born, p.id))
                .map(|p| (p.id, p.know(t).map_or(KnowSource::Founder, |k| k.source)));
            if let Some((who, source)) = eldest {
                self.note_known(now, s, who, t, source, None);
            }
        }
    }

    /// A founding band brings at least one knower of every technique the people profile gives
    /// founders any share of (ADR-0008 §1): where the draws gave none, the eldest who is old
    /// enough for it knows it.
    pub fn ensure_knowers(
        &mut self,
        catalog: &Catalog,
        params: &PeopleParams,
        now: SimTime,
        people: &[PermanentId],
    ) {
        for &(t, share) in &params.knowledge.founders {
            if share <= 0.0
                || people
                    .iter()
                    .any(|id| self.person(*id).is_some_and(|p| p.knows(t)))
            {
                continue;
            }
            let Some(age) =
                crate::knowledge::knowing_age(catalog, t, params.family.independent_age)
            else {
                continue;
            };
            let eldest = people
                .iter()
                .filter_map(|id| self.person(*id))
                .filter(|p| p.age_years(now) >= age)
                .min_by_key(|p| (p.born, p.id))
                .map(|p| p.id);
            if let Some(p) = eldest.and_then(|id| self.person_mut_by_id(id)) {
                p.come_to_know(t, KnowSource::Founder, now);
            }
        }
    }

    /// Children brought up with their household's work (ADR-0008 §4): once a day, each member of
    /// an age to learn an `upbringing` technique at home learns it, if someone in their household
    /// knows it.
    pub(crate) fn bring_up(&mut self, ctx: &mut Ctx) {
        let now = ctx.now;
        let catalog = ctx.catalog;
        let grown_at = ctx.params.family.independent_age;
        let ages: Vec<Option<(f64, f64)>> = (0..catalog.techniques.len())
            .map(|t| crate::knowledge::upbringing_ages(catalog, t, grown_at))
            .collect();
        // Nobody this old learns anything at home.
        let Some(oldest) = ages
            .iter()
            .flatten()
            .map(|&(_, until)| until)
            .reduce(f64::max)
        else {
            return;
        };
        let mut households: Vec<PermanentId> = self.households.iter().map(|(_, x)| x.id).collect();
        households.sort_unstable();
        let mut learnt: Vec<(PermanentId, usize, PermanentId)> = Vec::new();
        for hh in households {
            let Some(x) = self.household(hh) else {
                continue;
            };
            for &m in &x.members {
                let Some(p) = self.person(m) else {
                    continue;
                };
                let age = p.age_years(now);
                if age >= oldest {
                    continue;
                }
                for (t, window) in ages.iter().enumerate() {
                    if window.is_some_and(|(from, until)| age >= from && age < until)
                        && !p.knows(t)
                        && let Some(from) = self.upbringer(ctx, p, t)
                    {
                        learnt.push((m, t, from));
                    }
                }
            }
        }
        for (who, t, from) in learnt {
            let source = KnowSource::Upbringing(from);
            let new = self
                .person_mut_by_id(who)
                .is_some_and(|p| p.come_to_know(t, source, now));
            if new {
                self.on_known(ctx, who, t, source);
            }
        }
    }

    /// People who have died or left (`gone`: who and what they knew), from settlement `s`: each
    /// technique they knew that nobody there knows any more is lost there (ADR-0008 §5), noted
    /// in the record and the chronicle with whether anyone there still knows of it and whether
    /// what was made with it remains, the buildings of a household that left (`left_behind`)
    /// included.
    pub(crate) fn check_loss(
        &mut self,
        ctx: &mut Ctx,
        s: Option<PermanentId>,
        gone: &[(PermanentId, Vec<Know>)],
        left_behind: Option<PermanentId>,
    ) {
        let Some(s) = s else {
            return;
        };
        let now = ctx.now;
        let catalog = ctx.catalog;
        let residents: Vec<PermanentId> = {
            let mut ids: Vec<PermanentId> = self
                .households
                .iter()
                .filter(|(_, x)| x.settlement == Some(s))
                .flat_map(|(_, x)| x.members.iter().copied())
                .collect();
            ids.sort_unstable();
            ids
        };
        let mut lost: Vec<(usize, PermanentId)> = Vec::new();
        for (who, knows) in gone {
            for k in knows.iter().filter(|k| k.known) {
                let t = usize::from(k.technique);
                if lost.iter().any(|(l, _)| *l == t) {
                    continue;
                }
                let still = residents
                    .iter()
                    .filter_map(|id| self.person(*id))
                    .any(|p| p.knows(t));
                if !still && self.known_in(s, t) {
                    lost.push((t, *who));
                }
            }
        }
        for (t, who) in lost {
            let aware = residents
                .iter()
                .filter_map(|id| self.person(*id))
                .any(|p| p.know(t).is_some());
            let made = self.made_with_remains(ctx, s, t, left_behind);
            let near = self.known_near(s, &residents, t);
            let flags = if aware { LOST_AWARE } else { 0.0 }
                + if made { LOST_MADE_REMAIN } else { 0.0 }
                + if near.is_some() { LOST_KNOWN_NEAR } else { 0.0 };
            self.knowledge.push(KnowledgeEvent {
                at: now,
                settlement: s,
                technique: t as u16,
                person: who,
                kind: KnowledgeEventKind::Lost,
                elsewhere: near,
            });
            let name = catalog
                .techniques
                .get(t)
                .map_or_else(String::new, |d| d.name.clone());
            self.chronicle_push(
                now,
                ChronicleKind::TechniqueLost,
                vec![who],
                Some(s),
                None,
                flags,
                name,
            );
        }
    }

    /// Another settlement where technique `t`, just lost in `s`, is still known, among those
    /// where one of `residents` (who remain in `s`) has close kin or a tie (M5b slice AR;
    /// research 07-02 §5.4: an accessible practitioner, not only a local one): the one with the
    /// most such links, then the most knowers, then the least id. `None` if there is none.
    fn known_near(
        &self,
        s: PermanentId,
        residents: &[PermanentId],
        t: usize,
    ) -> Option<PermanentId> {
        let lives_in = |q: PermanentId| {
            self.person(q)
                .and_then(|p| self.household(p.household))
                .and_then(|h| h.settlement)
                .filter(|&x| x != s)
        };
        // Links from those left here into each other settlement.
        let mut links: std::collections::BTreeMap<PermanentId, u32> = Default::default();
        for &r in residents {
            let kin = self.close_kin(r);
            let tied = self.ties.of(r).iter().map(|x| x.to);
            for q in kin.into_iter().chain(tied) {
                if let Some(x) = lives_in(q) {
                    *links.entry(x).or_default() += 1;
                }
            }
        }
        if links.is_empty() {
            return None;
        }
        let mut knowers: std::collections::BTreeMap<PermanentId, u32> = Default::default();
        for (_, h) in self.households.iter() {
            let Some(x) = h.settlement.filter(|x| links.contains_key(x)) else {
                continue;
            };
            let n = h
                .members
                .iter()
                .filter(|&&m| self.person(m).is_some_and(|p| p.knows(t)))
                .count() as u32;
            if n > 0 {
                *knowers.entry(x).or_default() += n;
            }
        }
        knowers
            .into_iter()
            .max_by_key(|&(x, n)| (links[&x], n, std::cmp::Reverse(x)))
            .map(|(x, _)| x)
    }

    /// `people` have come to live in settlement `to` from settlement `left` (M5b slice AR): a move
    /// with their household, a marriage, an exile taken in. If `check`, each technique only they
    /// knew in `left` is lost there (ADR-0008 §5), the buildings of a household that left
    /// (`left_behind`) counted as what remains; and each technique one of them knows that was not
    /// known in `to` is noted there as brought from `left` by its eldest knower among them.
    /// Nothing happens unless both are settlements and they differ.
    pub(crate) fn knowledge_crossed(
        &mut self,
        ctx: &mut Ctx,
        people: &[PermanentId],
        left: Option<PermanentId>,
        to: Option<PermanentId>,
        left_behind: Option<PermanentId>,
        check: bool,
    ) {
        let (Some(left), Some(to)) = (left, to) else {
            return;
        };
        if left == to || people.is_empty() {
            return;
        }
        if check {
            let gone: Vec<(PermanentId, Vec<Know>)> = people
                .iter()
                .filter_map(|&id| self.person(id).map(|p| (id, p.knows.clone())))
                .collect();
            self.check_loss(ctx, Some(left), &gone, left_behind);
        }
        let now = ctx.now;
        for t in 0..ctx.catalog.techniques.len() {
            let eldest = people
                .iter()
                .filter_map(|id| self.person(*id))
                .filter(|p| p.knows(t))
                .min_by_key(|p| (p.born, p.id))
                .map(|p| (p.id, p.know(t).map_or(KnowSource::Founder, |k| k.source)));
            if let Some((who, source)) = eldest {
                self.note_known(now, to, who, t, source, Some(left));
            }
        }
    }

    /// The technique that work `def` aimed at `target` needs: its own or its recipe's, or for
    /// building work, the technique of the building it works on (ADR-0008 §1).
    fn technique_at_work(ctx: &Ctx, def: &ActivityDef, target: Target) -> Option<usize> {
        let catalog = ctx.catalog;
        match target {
            Target::Building(b) if def.behavior == Behavior::Build => ctx
                .land
                .buildings
                .iter()
                .find(|x| x.id == b)
                .and_then(|x| catalog.building_index(&x.spec.program))
                .and_then(|i| catalog.buildings.get(i))
                .and_then(|d| d.technique),
            _ => catalog.technique_of(def),
        }
    }

    /// `me` stands at `at` in `settlement`, not their own, at its hearth on a visit or at a
    /// seller's door on a trip to buy (M5b slice AR; the M5 diffusion brief §1.2): they come to
    /// know of each technique that work they see its people doing within the content's `watch_m`
    /// needs. Seeing gives awareness only (research 07-02 §1.2), which counts toward trying
    /// (ADR-0008 §3); what they knew of already is unchanged.
    pub(crate) fn watch_work(
        &mut self,
        ctx: &Ctx,
        me: PermanentId,
        settlement: PermanentId,
        at: (f32, f32),
    ) {
        let reach = ctx.params.knowledge.watch_m;
        if reach <= 0.0 {
            return;
        }
        let home = self
            .person(me)
            .and_then(|p| self.household(p.household))
            .and_then(|h| h.settlement);
        if home.is_none() || home == Some(settlement) {
            return;
        }
        let now = ctx.now;
        let minute = now.minutes() as f64;
        let mut seen: Vec<usize> = Vec::new();
        for h in self.households_in(settlement) {
            for &m in &h.members {
                let Some(q) = self.person(m) else {
                    continue;
                };
                let working = matches!(
                    q.act.steps.get(usize::from(q.act.step)),
                    Some(Step::Work { .. })
                );
                if !working {
                    continue;
                }
                let pos = q.position_at(minute);
                if f64::from((pos.0 - at.0).hypot(pos.1 - at.1)) > reach {
                    continue;
                }
                let Some(def) = ctx.catalog.activities.get(usize::from(q.act.def)) else {
                    continue;
                };
                if let Some(t) = Self::technique_at_work(ctx, def, q.act.target) {
                    seen.push(t);
                }
            }
        }
        seen.sort_unstable();
        seen.dedup();
        if let Some(p) = self.person_mut(me) {
            for t in seen {
                p.hear_of(t, KnowSource::Seen(settlement), now);
            }
        }
    }

    /// `me`, of another settlement, bought `good` at a door in `settlement` (M5b slice AR): a good
    /// that only work needing one technique makes shows that technique, and they come to know of
    /// it. A good made in several ways shows none, and buying a tool never teaches making it
    /// (research 07-02 §1.2).
    pub(crate) fn see_made_with(
        &mut self,
        ctx: &Ctx,
        me: PermanentId,
        settlement: PermanentId,
        good: usize,
    ) {
        let mut makers = ctx
            .catalog
            .recipes
            .iter()
            .filter(|r| r.outputs.iter().any(|&(g, q)| g == good && q > 0.0))
            .map(|r| r.technique);
        let Some(Some(t)) = makers.next() else {
            return;
        };
        if makers.any(|x| x != Some(t)) {
            return;
        }
        let now = ctx.now;
        if let Some(p) = self.person_mut(me) {
            p.hear_of(t, KnowSource::Seen(settlement), now);
        }
    }

    /// [`Population::watch_work`], for tests that put someone at a hearth.
    #[doc(hidden)]
    pub fn watch_work_for_tests(
        &mut self,
        ctx: &Ctx,
        me: PermanentId,
        settlement: PermanentId,
        at: (f32, f32),
    ) {
        self.watch_work(ctx, me, settlement, at);
    }

    /// [`Population::see_made_with`], for tests of a purchase between settlements.
    #[doc(hidden)]
    pub fn bought_for_tests(
        &mut self,
        ctx: &Ctx,
        me: PermanentId,
        settlement: PermanentId,
        good: usize,
    ) {
        self.see_made_with(ctx, me, settlement, good);
    }

    /// Whether goods or buildings made with technique `t` remain in settlement `s`: a standing
    /// building of a program that needs it, or goods a recipe that needs it makes, held by a
    /// household or workshop there.
    fn made_with_remains(
        &self,
        ctx: &Ctx,
        s: PermanentId,
        t: usize,
        left_behind: Option<PermanentId>,
    ) -> bool {
        let catalog = ctx.catalog;
        let households: Vec<PermanentId> = self
            .households
            .iter()
            .filter(|(_, x)| x.settlement == Some(s))
            .map(|(_, x)| x.id)
            .chain(left_behind)
            .collect();
        let building = ctx.land.buildings.iter().any(|b| {
            households.contains(&b.household)
                && catalog
                    .building_index(&b.spec.program)
                    .is_some_and(|i| catalog.buildings[i].technique == Some(t))
        });
        if building {
            return true;
        }
        let outputs: Vec<usize> = catalog
            .recipes
            .iter()
            .filter(|r| r.technique == Some(t))
            .flat_map(|r| r.outputs.iter().map(|&(g, _)| g))
            .collect();
        let held = |stores: &[f64]| {
            outputs
                .iter()
                .any(|&g| stores.get(g).copied().unwrap_or(0.0) > 1e-6)
        };
        self.households_in(s).any(|x| held(&x.stores))
            || self
                .firms
                .iter()
                .any(|f| f.is_open() && f.settlement == Some(s) && held(&f.stores))
    }

    /// The observer's god tool (ADR-0008 §6): living person `person` comes to know technique `t`,
    /// or with `aware_only` only hears of it, and the chronicle records the observer's doing.
    /// Prerequisites are not needed; skill is unchanged.
    pub fn introduce_technique(
        &mut self,
        ctx: &mut Ctx,
        person: PermanentId,
        t: usize,
        aware_only: bool,
    ) -> Result<(), String> {
        let now = ctx.now;
        let def = ctx
            .catalog
            .techniques
            .get(t)
            .ok_or_else(|| format!("there is no technique {t}"))?;
        let name = def.name.clone();
        let p = self
            .person_mut_by_id(person)
            .ok_or_else(|| format!("{person} is not alive here"))?;
        if p.knows(t) {
            return Err(format!("{} already knows {}", p.given, name.to_lowercase()));
        }
        if aware_only {
            if p.know(t).is_some() {
                return Err(format!(
                    "{} already knows of {}",
                    p.given,
                    name.to_lowercase()
                ));
            }
            p.hear_of(t, KnowSource::Observer, now);
        } else {
            p.come_to_know(t, KnowSource::Observer, now);
        }
        let household = self.person(person).map(|p| p.household);
        let settlement = household
            .and_then(|h| self.household(h))
            .and_then(|x| x.settlement);
        if !aware_only && let Some(s) = settlement {
            self.note_known(now, s, person, t, KnowSource::Observer, None);
        }
        // What it can build changes with what its members know: it plans its home again.
        if let Some(h) = household {
            self.home_sites.remove(&h);
        }
        self.chronicle_push(
            now,
            ChronicleKind::TechniqueIntroduced,
            vec![person],
            settlement,
            None,
            if aware_only { 1.0 } else { 0.0 },
            name,
        );
        Ok(())
    }

    /// For a world saved before people knew anything (schema 12 and earlier): everyone gets what a
    /// founder of their age would bring, each settlement brings at least one knower of each
    /// technique founders have, and the settlements' records begin with it (ADR-0008 §7).
    pub fn give_founders_knowledge(
        &mut self,
        catalog: &Catalog,
        params: &PeopleParams,
        seed: u64,
        now: SimTime,
    ) {
        for (_, p) in self.people.iter_mut() {
            let mut rng = Rng64::from_key(&[seed, crate::found::PURPOSE_KNOW, p.id.get()]);
            p.knows = crate::knowledge::founder_knowledge(
                catalog,
                &params.knowledge,
                params.family.independent_age,
                p.age_years(now),
                &mut rng,
                now,
            );
        }
        let mut by_settlement: std::collections::BTreeMap<PermanentId, Vec<PermanentId>> =
            std::collections::BTreeMap::new();
        for (_, x) in self.households.iter() {
            if let Some(s) = x.settlement {
                by_settlement
                    .entry(s)
                    .or_default()
                    .extend(x.members.iter().copied());
            }
        }
        for (s, mut people) in by_settlement {
            people.sort_unstable();
            self.ensure_knowers(catalog, params, now, &people);
            self.note_arrivals(catalog, now, s, &people);
        }
    }

    /// For a world saved before the content had techniques `new` (indexes into
    /// `catalog.techniques` the save never named): everyone gets those of them a founder of their
    /// age would bring, drawn as [`Population::give_founders_knowledge`] draws them, each
    /// settlement brings at least one knower of each, and the settlements' records note them now.
    /// Techniques the save named, known or lost, are left as they were.
    pub fn give_new_founder_knowledge(
        &mut self,
        catalog: &Catalog,
        params: &PeopleParams,
        seed: u64,
        now: SimTime,
        new: &[usize],
    ) {
        let brought: Vec<usize> = params
            .knowledge
            .founders
            .iter()
            .filter(|&&(t, share)| share > 0.0 && new.contains(&t))
            .map(|&(t, _)| t)
            .collect();
        if brought.is_empty() {
            return;
        }
        for (_, p) in self.people.iter_mut() {
            let mut rng = Rng64::from_key(&[seed, crate::found::PURPOSE_KNOW, p.id.get()]);
            let drawn = crate::knowledge::founder_knowledge(
                catalog,
                &params.knowledge,
                params.family.independent_age,
                p.age_years(now),
                &mut rng,
                now,
            );
            for k in drawn {
                if brought.contains(&usize::from(k.technique)) {
                    p.come_to_know(usize::from(k.technique), KnowSource::Founder, now);
                }
            }
        }
        let mut by_settlement: std::collections::BTreeMap<PermanentId, Vec<PermanentId>> =
            std::collections::BTreeMap::new();
        for (_, x) in self.households.iter() {
            if let Some(s) = x.settlement {
                by_settlement
                    .entry(s)
                    .or_default()
                    .extend(x.members.iter().copied());
            }
        }
        for (s, mut people) in by_settlement {
            people.sort_unstable();
            for &t in &brought {
                let Some(age) =
                    crate::knowledge::knowing_age(catalog, t, params.family.independent_age)
                else {
                    continue;
                };
                if !self.household_knows(&people, t) {
                    let eldest = people
                        .iter()
                        .filter_map(|id| self.person(*id))
                        .filter(|p| p.age_years(now) >= age)
                        .min_by_key(|p| (p.born, p.id))
                        .map(|p| p.id);
                    if let Some(p) = eldest.and_then(|id| self.person_mut_by_id(id)) {
                        p.come_to_know(t, KnowSource::Founder, now);
                    }
                }
                let eldest = people
                    .iter()
                    .filter_map(|id| self.person(*id))
                    .filter(|p| p.knows(t))
                    .min_by_key(|p| (p.born, p.id))
                    .map(|p| p.id);
                if let Some(who) = eldest {
                    self.note_known(now, s, who, t, KnowSource::Founder, None);
                }
            }
        }
    }

    /// Whether anyone among `members` knows technique `t`.
    pub(crate) fn household_knows(&self, members: &[PermanentId], t: usize) -> bool {
        members
            .iter()
            .any(|m| self.person(*m).is_some_and(|p| p.knows(t)))
    }

    /// A living person, mutably, by permanent id.
    pub(crate) fn person_mut_by_id(&mut self, id: PermanentId) -> Option<&mut Person> {
        let h = *self.index.get(&id)?;
        self.people.get_mut(h)
    }
}

/// Whether person `q` is at activity `def` aimed at `target` now: working there, not on the way.
fn working(q: &Person, def: u16, target: Target) -> bool {
    q.act.def == def
        && q.act.target == target
        && matches!(
            q.act.steps.get(q.act.step as usize),
            Some(Step::Work { .. })
        )
}

/// The minutes of a day's session of teaching or study (M10c slice EB, second part; the session
/// activities' length).
pub const SESSION_MIN: u32 = 120;

/// Keyed-randomness purpose of a household's day of the year for weighing a child's enrolment.
const PURPOSE_ENROL: u64 = 0x656e_726f_6c6d_656e; // "enrolmen"

/// Those learning a technique at a household, each with the hours they gave it.
type Learners = Vec<(PermanentId, f64)>;

/// Hours given a technique in a household-resolved day, by household (M10c slice EB, first part):
/// by those learning it there, each, and by those knowing it there, together.
#[derive(Debug, Default)]
pub(crate) struct Alongside {
    learning: std::collections::BTreeMap<(PermanentId, usize), Learners>,
    held: std::collections::BTreeMap<(PermanentId, usize), f64>,
}

impl Alongside {
    /// How many of household `household` other than `who` are learning technique `t` today.
    pub(crate) fn learners(&self, household: PermanentId, t: usize, who: PermanentId) -> usize {
        self.learning
            .get(&(household, t))
            .map_or(0, |l| l.iter().filter(|(q, _)| *q != who).count())
    }

    /// Whether `who` of household `household` is learning technique `t` today.
    pub(crate) fn is_learning(&self, household: PermanentId, t: usize, who: PermanentId) -> bool {
        self.learning
            .get(&(household, t))
            .is_some_and(|l| l.iter().any(|(q, _)| *q == who))
    }

    /// `who` of household `household` gave `hours` to learning technique `t`.
    pub(crate) fn attend(
        &mut self,
        household: PermanentId,
        t: usize,
        who: PermanentId,
        hours: f64,
    ) {
        let list = self.learning.entry((household, t)).or_default();
        match list.iter_mut().find(|(q, _)| *q == who) {
            Some((_, h)) => *h += hours,
            None => list.push((who, hours)),
        }
    }

    /// A knower of household `household` gave `hours` to technique `t`.
    pub(crate) fn give(&mut self, household: PermanentId, t: usize, hours: f64) {
        *self.held.entry((household, t)).or_default() += hours;
    }

    /// The hours household `household`'s knowers gave technique `t`.
    pub(crate) fn held(&self, household: PermanentId, t: usize) -> f64 {
        self.held.get(&(household, t)).copied().unwrap_or(0.0)
    }

    /// Each household and technique learnt there today, with its learners and their hours, in
    /// order.
    pub(crate) fn groups(&self) -> Vec<(PermanentId, usize, Learners)> {
        self.learning
            .iter()
            .map(|(&(h, t), l)| (h, t, l.clone()))
            .collect()
    }
}
