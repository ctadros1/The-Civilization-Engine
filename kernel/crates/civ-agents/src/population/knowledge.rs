//! What people know, in the running population (ADR-0008): the technique each piece of work
//! needs, who may do it, learning by working beside someone who knows, children brought up with
//! their household's work, each settlement's record, loss with the last who knew, and the god
//! tool that introduces a technique.

use civ_core::{PermanentId, Rng64, SimTime};

use super::{Ctx, Population};
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
    /// draw, keyed by person, technique and session, against the chance those hours find it,
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
            let chance = find_chance(hours * share, d.e50_h);
            let key = [
                ctx.seed,
                crate::found::PURPOSE_FIND,
                who.get(),
                t as u64,
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
    fn on_known(&mut self, ctx: &Ctx, who: PermanentId, t: usize, source: KnowSource) {
        let now = ctx.now;
        let settlement = self
            .person(who)
            .and_then(|p| self.household(p.household))
            .and_then(|x| x.settlement);
        if let Some(s) = settlement {
            self.note_known(now, s, who, t, source);
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
    /// known there (never, or not since it was lost).
    fn note_known(
        &mut self,
        now: SimTime,
        s: PermanentId,
        who: PermanentId,
        t: usize,
        source: KnowSource,
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
                self.note_known(now, s, who, t, source);
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
            let flags =
                if aware { LOST_AWARE } else { 0.0 } + if made { LOST_MADE_REMAIN } else { 0.0 };
            self.knowledge.push(KnowledgeEvent {
                at: now,
                settlement: s,
                technique: t as u16,
                person: who,
                kind: KnowledgeEventKind::Lost,
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
        self.households
            .iter()
            .any(|(_, x)| x.settlement == Some(s) && held(&x.stores))
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
            self.note_known(now, s, person, t, KnowSource::Observer);
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
                    self.note_known(now, s, who, t, KnowSource::Founder);
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
