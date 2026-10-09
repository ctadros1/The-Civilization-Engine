//! A day of life, at each day's end (research 04-08 §5.2: the order of a demographic step; 05-01
//! §5.2: dated pregnancy outcomes and daily hazards):
//!
//! 1. pregnancies that are due end, in a birth or a loss; a birth can cost the mother her life;
//! 2. people die, of the baseline hazard of their age or of hunger;
//! 3. households left with no one go to their nearest kin, and children left without an older
//!    member go to live with kin (or, failing kin, neighbours);
//! 4. households out of food and worn down, with no crop of theirs ripening soon, may give up
//!    and leave the valley;
//! 5. women living with their partners may conceive;
//! 6. unpartnered adults may find a partner, and the couple settles in a household.
//!
//! Nothing here is scheduled ahead of its day: a birth date is a pregnancy's own course, and a
//! death is a hazard drawn each day, never a date fixed at birth (05-01 §5.2).

use super::*;
use crate::demography::{
    DAYS_PER_MONTH, Draw, conception_chance, couple_score, daily_from_monthly, death_today,
    fecundity, inherit_traits, life_rng, minutes_of_days, pick_name, pick_softmax,
    pregnancy_course, recovery_days, too_close,
};
use crate::history::{Cause, Moved, Origin, ResidenceWhy, Stay, Union};
use crate::ledger::{Channel, Leg};
use crate::needs::Sex;
use crate::params::{FamilyParams, Residence};
use crate::person::{Flow, Flows, Repro, Traits};

/// Steps through parents and children searched for kin to take children in or inherit a
/// household.
const KIN_SEARCH_STEPS: usize = 4;

/// How much farther from the hearth than the household it leaves a new household makes its home,
/// metres (a tuning value; its hut is then sited on clear ground within reach of that point).
const NEW_HOME_STEP_M: f64 = 20.0;

/// The share of their body's reserve a person has drawn now, 0–1.
pub fn depleted(p: &Person, now: SimTime, params: &PeopleParams) -> f64 {
    let reserve = reserve_kcal(p, now, params);
    if reserve <= 0.0 {
        return 0.0;
    }
    (-energy_now(p, now, params) / reserve).clamp(0.0, 1.0)
}

/// Extra energy a person's body spends a day beyond their activity: a pregnancy's, by trimester
/// (research 05-02 §2.4).
pub fn extra_kcal_day(p: &Person, now: SimTime, params: &PeopleParams) -> f64 {
    match p.repro {
        Repro::Pregnant { conceived, .. } => {
            let days = (now.minutes() - conceived.minutes()).max(0) as f64 / 1440.0;
            let trimester = ((days / 89.0) as usize).min(2);
            params.fertility.pregnancy_kcal_day[trimester]
        }
        _ => 0.0,
    }
}

impl Population {
    /// For a world saved before couples were kept (save schema 5 and older): the couple at the
    /// head of each household, a woman and a man who share a child or else the first woman and
    /// first man it lists who are of age and not close kin, become partners; every woman draws
    /// her lasting fecundability. Nobody is pregnant or nursing.
    pub fn infer_couples(&mut self, params: &PeopleParams, seed: u64, now: SimTime) {
        let fam = &params.family;
        let mut households: Vec<PermanentId> = self.households.iter().map(|(_, x)| x.id).collect();
        households.sort_unstable();
        for household in households {
            let members = self
                .household(household)
                .map(|x| x.members.clone())
                .unwrap_or_default();
            let of_age = |sex: Sex| -> Vec<PermanentId> {
                members
                    .iter()
                    .copied()
                    .filter(|m| {
                        self.person(*m).is_some_and(|p| {
                            p.sex == sex
                                && p.partner.is_none()
                                && p.age_years(now) >= fam.seek_min_age[FamilyParams::of(sex)]
                        })
                    })
                    .collect()
            };
            let (women, men) = (of_age(Sex::Female), of_age(Sex::Male));
            let parents = |w: PermanentId, m: PermanentId| {
                self.records
                    .values()
                    .filter(|r| r.mother == Some(w) && r.father == Some(m))
                    .map(|r| r.born)
                    .min()
            };
            let shared = women.iter().find_map(|&w| {
                men.iter()
                    .find_map(|&m| parents(w, m).map(|eldest| (w, m, eldest)))
            });
            let couple = shared.or_else(|| {
                women.iter().find_map(|&w| {
                    men.iter()
                        .find(|&&m| !too_close(&self.records, w, m, fam.kin_exclusion_generations))
                        .map(|&m| (w, m, now))
                })
            });
            if let Some((woman, man, since)) = couple {
                for (a, b) in [(woman, man), (man, woman)] {
                    if let Some(p) = self.person_mut(a) {
                        p.partner = Some(b);
                    }
                }
                self.unions.push(Union {
                    woman,
                    man,
                    since: since.min(now),
                    ended: None,
                });
            }
        }
        for (_, p) in self.people.iter_mut() {
            p.repro = Repro::Open;
            p.nursing = None;
            p.fecundity = match p.sex {
                Sex::Female => fecundity(
                    &params.fertility,
                    &mut life_rng(seed, p.id, 0, Draw::Fecundity),
                ),
                Sex::Male => 1.0,
            };
        }
    }

    /// A day of life ends (see the module documentation).
    pub(super) fn live_day(&mut self, ctx: &mut Ctx) {
        let day = ctx.now.day_index();
        self.emptied.clear();
        self.end_pregnancies(ctx, day);
        self.mortality(ctx, day);
        self.care_for_households(ctx);
        self.departures(ctx, day);
        self.residence_reviews(ctx, day);
        self.conceptions(ctx, day);
        self.partnering(ctx, day);
        self.emptied.clear();
        self.note_abandoned(ctx);
        // Price reports too old to act on are let go, and those of a household's own market
        // (it moved there) or of a household that is no more (M5b slice AP).
        if !self.reports.held.is_empty() {
            let homes: BTreeMap<PermanentId, Option<Option<PermanentId>>> = self
                .reports
                .held
                .keys()
                .map(|&h| (h, self.household(h).map(|x| x.settlement)))
                .collect();
            let max_age = ctx.params.reports.max_age_days;
            self.reports
                .prune(day, max_age, |h| homes.get(&h).copied().flatten());
        }
    }

    /// Living people by permanent id: a fixed order, however the table is laid out.
    fn living_ids(&self) -> Vec<PermanentId> {
        let mut ids: Vec<PermanentId> = self.people.iter().map(|(_, p)| p.id).collect();
        ids.sort_unstable();
        ids
    }

    pub(super) fn person_mut(&mut self, id: PermanentId) -> Option<&mut Person> {
        let h = *self.index.get(&id)?;
        self.people.get_mut(h)
    }

    pub(super) fn household_mut(&mut self, id: PermanentId) -> Option<&mut Household> {
        let h = *self.hh_index.get(&id)?;
        self.households.get_mut(h)
    }

    /// Brings a household's stores and water up to now at its present number of members (before
    /// that number changes).
    pub(super) fn settle_household(&mut self, ctx: &Ctx, id: PermanentId) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        if let Some(x) = self.household_mut(id) {
            let members = x.members.len();
            x.settle_stores(now, goods, &|d| fuel_per_day(params, members, d));
            x.settle_water(
                now,
                members as f64 * params.household.water_l_per_person_day,
            );
            let len = goods.len().max(x.stores.len());
            x.stores.resize(len, 0.0);
        }
    }

    /// Keeps a household's members oldest first.
    pub(super) fn sort_members(&mut self, id: PermanentId) {
        let Some(&hd) = self.hh_index.get(&id) else {
            return;
        };
        let born: HashMap<PermanentId, i64> = self
            .households
            .get(hd)
            .map(|x| {
                x.members
                    .iter()
                    .map(|m| (*m, self.records.get(m).map_or(0, |r| r.born.minutes())))
                    .collect()
            })
            .unwrap_or_default();
        if let Some(x) = self.households.get_mut(hd) {
            x.members
                .sort_by_key(|m| (born.get(m).copied().unwrap_or(0), *m));
        }
    }

    fn end_pregnancies(&mut self, ctx: &mut Ctx, day: i64) {
        let (now, f) = (ctx.now, &ctx.params.fertility);
        for id in self.living_ids() {
            let Some(p) = self.person_mut(id) else {
                continue;
            };
            let Repro::Pregnant {
                due, father, loss, ..
            } = p.repro
            else {
                continue;
            };
            if due > now {
                continue;
            }
            if loss {
                let days = f.loss_recovery_months * DAYS_PER_MONTH;
                p.repro = Repro::Recovering {
                    until: now.plus_minutes(minutes_of_days(days)),
                };
                continue;
            }
            self.birth(ctx, id, father, day);
        }
    }

    /// A woman gives birth: the child joins her household, she nurses it and cannot conceive for
    /// a while, and childbirth may cost her life.
    fn birth(&mut self, ctx: &mut Ctx, mother: PermanentId, father: Option<PermanentId>, day: i64) {
        let (now, params) = (ctx.now, ctx.params);
        let f = &params.fertility;
        let Some(m) = self.person(mother) else {
            return;
        };
        let (household, mother_traits) = (m.household, m.traits);
        let Some(hh) = self.household(household) else {
            return;
        };
        let (home, settlement) = (hh.home, hh.settlement);
        let father_traits: Option<Traits> = father.and_then(|x| self.person(x)).map(|x| x.traits);
        let mut rng = life_rng(ctx.seed, mother, day, Draw::Birth);
        let boys = f.boys_per_100_girls.max(0.0);
        let sex = if rng.next_f64() < boys / (boys + 100.0) {
            Sex::Male
        } else {
            Sex::Female
        };
        // Not a name anyone living in the settlement bears (research 06-07 §4.1).
        let taken: Vec<String> = self
            .people
            .iter()
            .filter(|(_, q)| {
                self.household(q.household)
                    .is_some_and(|x| x.settlement == settlement)
            })
            .map(|(_, q)| q.given.clone())
            .collect();
        let taken: Vec<&str> = taken.iter().map(String::as_str).collect();
        let list = match sex {
            Sex::Male => &params.names.male,
            Sex::Female => &params.names.female,
        };
        let given = pick_name(list, &taken, &mut rng);
        let traits = inherit_traits(
            Some(&mother_traits),
            father_traits.as_ref(),
            params.family.trait_heritability,
            &mut rng,
        );
        // The child's objection to taking, pulled toward its parents' (M4b slice AA), by a draw
        // of its own so the birth's other draws are as they were.
        let parent_objection =
            |p: Option<PermanentId>| p.and_then(|p| self.person(p)).map(|q| q.objection);
        let objection_parents = (parent_objection(Some(mother)), parent_objection(father));
        let recovery = recovery_days(f, &mut rng);
        let dies = rng.next_f64() < params.mortality.maternal_death_per_birth;
        let id = ctx.ids.allocate();
        let own_fecundity = match sex {
            Sex::Female => fecundity(f, &mut life_rng(ctx.seed, id, day, Draw::Fecundity)),
            Sex::Male => 1.0,
        };
        self.settle_household(ctx, household);
        self.records.insert(
            id,
            PersonRecord {
                id,
                given: given.clone(),
                sex,
                born: now,
                died: None,
                left: None,
                mother: Some(mother),
                father,
                origin: Origin::Born,
                residence: vec![Stay {
                    settlement,
                    since: now,
                    why: crate::history::ResidenceWhy::Born,
                }],
            },
        );
        self.insert_person(Person {
            id,
            given,
            sex,
            born: now,
            mother: Some(mother),
            father,
            household,
            traits,
            pos: home,
            energy_kcal: 0.0,
            satiety_until: now,
            sleep_pressure: params.sleep.wake_pressure as f32,
            relatedness: 0.8,
            needs_at: now,
            burn_kcal_min: 0.0,
            asleep: false,
            company: params.social.household_quality as f32,
            act: Activity {
                def: 0,
                target: Target::None,
                steps: Vec::new(),
                step: 0,
                started: now,
                step_started: now,
                step_ends: now,
                version: 0,
            },
            trip: None,
            carrying: Load::default(),
            draws: 0,
            receipts: Default::default(),
            partner: None,
            repro: Repro::Open,
            fecundity: own_fecundity,
            nursing: None,
            skills: Vec::new(),
            knows: Vec::new(),
            tried: None,
            objection: crate::crime::draw_objection(
                objection_parents.0,
                objection_parents.1,
                &params.crime,
                &mut life_rng(ctx.seed, id, day, Draw::Objection),
            ),
            risk_seen: params.crime.risk_prior as f32,
            guarded: None,
        });
        if let Some(x) = self.household_mut(household) {
            x.members.push(id);
        }
        self.sort_members(household);
        if let Some(m) = self.person_mut(mother) {
            m.repro = Repro::Recovering {
                until: now.plus_minutes(minutes_of_days(recovery)),
            };
            m.nursing = Some(id);
        }
        let mut people = vec![id, mother];
        people.extend(father);
        self.chronicle_push(
            now,
            ChronicleKind::Born,
            people,
            settlement,
            Some(home),
            0.0,
            String::new(),
        );
        self.begin(ctx, id);
        if dies {
            self.die(ctx, mother, Cause::Childbirth);
        }
    }

    fn mortality(&mut self, ctx: &mut Ctx, day: i64) {
        let (now, params) = (ctx.now, ctx.params);
        for id in self.living_ids() {
            let Some(p) = self.person(id) else {
                continue;
            };
            let (age, d) = (p.age_years(now), depleted(p, now, params));
            let u = life_rng(ctx.seed, id, day, Draw::Death).next_f64();
            let natural = death_today(&params.mortality, age, d, u);
            // A blessing or a curse moves their own chance of illness or accident (M4c slice AJ):
            // the same draw against it, and what it turned is noted.
            let cause = match self.influences.luck(id, day) {
                None => natural,
                Some(luck) => {
                    let moved = crate::demography::death_today_scaled(
                        &params.mortality,
                        age,
                        d,
                        u,
                        luck.harm(),
                    );
                    if moved.is_some() != natural.is_some() {
                        self.note_turned(ctx, id, luck, None);
                    }
                    moved
                }
            };
            if let Some(cause) = cause {
                self.die(ctx, id, cause);
            }
        }
    }

    /// Someone dies now, and a household left with no one is cared for as at the day's end: for
    /// tests that need a death on a given day. Deaths in a world come from the life table.
    #[doc(hidden)]
    pub fn die_for_tests(&mut self, ctx: &mut Ctx, id: PermanentId) {
        self.die(ctx, id, Cause::Unspecified);
        self.care_for_households(ctx);
        self.emptied.clear();
    }

    /// Someone is exiled now, as a gathering's finding would send them: for tests that need an
    /// exile on a given day. Exiles in a world come from cases (M4b slice AB).
    #[doc(hidden)]
    pub fn exile_for_tests(&mut self, ctx: &mut Ctx, id: PermanentId) {
        self.exile(ctx, id);
        self.care_for_households(ctx);
        self.emptied.clear();
    }

    /// Someone dies: they leave their household (what they carried stays with it), their partner
    /// is widowed, a mother nursing them can conceive again soon, and the chronicle notes it.
    pub(crate) fn die(&mut self, ctx: &mut Ctx, id: PermanentId, cause: Cause) {
        let now = ctx.now;
        let Some(p) = self.person(id) else {
            return;
        };
        let household = p.household;
        self.settle_household(ctx, household);
        let Some(h) = self.index.remove(&id) else {
            return;
        };
        let Some(p) = self.people.remove(h) else {
            return;
        };
        let at = p.position_at(now.minutes() as f64);
        let age = p.age_years(now);
        let mut settlement = None;
        let mut emptied = false;
        if let Some(x) = self.household_mut(household) {
            x.members.retain(|m| *m != id);
            if let Some(g) = p.carrying.good.map(usize::from)
                && let Some(kg) = x.stores.get_mut(g)
            {
                *kg += f64::from(p.carrying.kg);
                x.flows.add(Flow::Got, g, f64::from(p.carrying.kg));
            }
            x.water_l += f64::from(p.carrying.water_l);
            settlement = x.settlement;
            emptied = x.members.is_empty();
        }
        if emptied {
            self.emptied.push((household, id));
        }
        if let Some(r) = self.records.get_mut(&id) {
            r.died = Some((now, cause));
        }
        if let Some(q) = p.partner {
            if let Some(partner) = self.person_mut(q) {
                partner.partner = None;
            }
            for u in self.unions.iter_mut().filter(|u| u.ended.is_none()) {
                if (u.woman == id && u.man == q) || (u.man == id && u.woman == q) {
                    u.ended = Some(now);
                }
            }
        }
        // Weaning ends with the nursing child's death (research 05-01 §1.5).
        if let Some(mother) = p.mother {
            let soon = now.plus_minutes(minutes_of_days(
                ctx.params.fertility.weaned_recovery_months * DAYS_PER_MONTH,
            ));
            if let Some(m) = self.person_mut(mother)
                && m.nursing == Some(id)
            {
                m.nursing = None;
                if let Repro::Recovering { until } = m.repro {
                    m.repro = Repro::Recovering {
                        until: until.min(soon),
                    };
                }
            }
        }
        self.chronicle_push(
            now,
            ChronicleKind::Died,
            vec![id],
            settlement,
            Some(at),
            age,
            cause.key().to_owned(),
        );
        // What only they knew there is lost with them (ADR-0008 §5).
        self.check_loss(ctx, settlement, &[(id, p.knows)], None);
    }

    /// `id` is sent from the valley by a gathering's finding (M4b slice AB, ADR-0015 §5): they
    /// leave their household as the dead do (what they carried stays with it; their partner is
    /// left without them), their record says when they left, and the chronicle notes it as a
    /// leaving. A household left with no one is cared for at the day's end.
    pub(crate) fn exile(&mut self, ctx: &mut Ctx, id: PermanentId) {
        let now = ctx.now;
        let Some(household) = self.person(id).map(|p| p.household) else {
            return;
        };
        self.settle_household(ctx, household);
        let Some(h) = self.index.remove(&id) else {
            return;
        };
        let Some(p) = self.people.remove(h) else {
            return;
        };
        let at = p.position_at(now.minutes() as f64);
        let mut settlement = None;
        let mut emptied = false;
        if let Some(x) = self.household_mut(household) {
            x.members.retain(|m| *m != id);
            if let Some(g) = p.carrying.good.map(usize::from)
                && let Some(kg) = x.stores.get_mut(g)
            {
                *kg += f64::from(p.carrying.kg);
                x.flows.add(Flow::Got, g, f64::from(p.carrying.kg));
            }
            x.water_l += f64::from(p.carrying.water_l);
            settlement = x.settlement;
            emptied = x.members.is_empty();
        }
        if emptied {
            self.emptied.push((household, id));
        }
        // Where they go is theirs to choose (M5a slice AN): a settlement their household knows
        // where kin or those they know live, or that a member saw fed, and else beyond the map
        // (ADR-0018 §3).
        self.refresh_kin();
        let refuge = settlement.and_then(|s| self.exile_refuge(ctx, id, household, s));
        match refuge {
            Some(to) => self.exile_to(ctx, id, &p, household, to),
            None => {
                if let Some(r) = self.records.get_mut(&id) {
                    r.left = Some(now);
                }
                self.note_residence(id, None, now, ResidenceWhy::Exiled);
            }
        }
        if let Some(q) = p.partner {
            if let Some(partner) = self.person_mut(q) {
                partner.partner = None;
            }
            for u in self.unions.iter_mut().filter(|u| u.ended.is_none()) {
                if (u.woman == id && u.man == q) || (u.man == id && u.woman == q) {
                    u.ended = Some(now);
                }
            }
        }
        if refuge.is_none() {
            let left = settlement
                .and_then(|s| ctx.land.settlements.iter().find(|x| x.id == s))
                .map(|x| x.name.clone())
                .unwrap_or_default();
            self.chronicle_push(
                now,
                ChronicleKind::Left,
                vec![id],
                settlement,
                Some(at),
                1.0,
                left,
            );
        }
        // What only they knew there leaves with them (ADR-0008 §5).
        self.check_loss(ctx, settlement, &[(id, p.knows)], None);
    }

    /// Households no one is left in pass to the nearest kin of whoever lived there last; children
    /// left without an older member go, with what their household held, to the household of
    /// their nearest adult kin, or failing kin to the neighbours best able to feed them.
    fn care_for_households(&mut self, ctx: &mut Ctx) {
        let emptied = std::mem::take(&mut self.emptied);
        for (household, last) in emptied {
            if self
                .household(household)
                .is_none_or(|x| !x.members.is_empty())
            {
                continue;
            }
            let heir = self.kin_household(ctx, last, household, false);
            let heirs = self.heir_households(last, household, heir);
            self.dissolve(ctx, household, heir, &heirs);
        }
        let grown = ctx.params.family.independent_age;
        let mut ids: Vec<PermanentId> = self.households.iter().map(|(_, x)| x.id).collect();
        ids.sort_unstable();
        for household in ids {
            let Some(x) = self.household(household) else {
                continue;
            };
            let members = x.members.clone();
            let keeper = members.iter().any(|m| {
                self.person(*m)
                    .is_some_and(|p| p.age_years(ctx.now) >= grown)
            });
            if keeper || members.is_empty() {
                continue;
            }
            let Some(to) = self.foster_household(ctx, household, &members) else {
                continue;
            };
            let taker = self.household(to).and_then(|x| x.members.first().copied());
            self.merge_household(ctx, household, to, true, ResidenceWhy::TakenIn);
            let settlement = self.household(to).and_then(|x| x.settlement);
            let place = self.household(to).map(|x| x.home);
            let mut people: Vec<PermanentId> = taker.into_iter().collect();
            people.extend(members);
            self.chronicle_push(
                ctx.now,
                ChronicleKind::TakenIn,
                people,
                settlement,
                place,
                0.0,
                String::new(),
            );
        }
    }

    /// Households out of food whose members are worn down, with no crop of theirs ripening or
    /// waiting to be threshed, weigh leaving the valley (research 05-06 §1.4: famine migration
    /// follows expected access to food, not only hunger; §5.2: a founding can fail, and its
    /// households withdraw). Each day such a household goes with a chance that weighs the wait to
    /// its next harvest that its food, what others could spare and any relief it may ask for would
    /// not cover, against what its fields should bring, which leaving gives up ([`leave_chance`];
    /// M4a slice Z). They go together and take what they carry; their fields and huts stand
    /// abandoned.
    fn departures(&mut self, ctx: &mut Ctx, day: i64) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let h = &params.household;
        let crop = ctx.catalog.crops.get(params.farm.crop);
        let mut ids: Vec<PermanentId> = self.households.iter().map(|(_, x)| x.id).collect();
        ids.sort_unstable();
        // Per settlement, what its households could spare and how many are out of food, worked
        // out when first needed today.
        let mut villages: Vec<(PermanentId, f64, usize)> = Vec::new();
        for household in ids {
            let Some(x) = self.household(household) else {
                continue;
            };
            if x.members.is_empty() {
                continue;
            }
            let need = x.members.len() as f64 * h.daily_kcal_per_person;
            let food = stock_kcal(&stores_now(x, now, params, goods), goods);
            if food >= need {
                continue;
            }
            let worn: f64 = x
                .members
                .iter()
                .filter_map(|m| self.person(*m))
                .map(|p| depleted(p, now, params))
                .sum::<f64>()
                / x.members.len() as f64;
            if worn < h.leave_at_depletion {
                continue;
            }
            let fields = || self.fields_of(ctx.land, household);
            let coming = crop.is_some_and(|c| {
                fields().any(|f| match f.stage {
                    FieldStage::Sown => {
                        (f.ripe_day(c) - day) as f64 <= h.leave_unless_ripe_within_days
                    }
                    FieldStage::Reaped => true,
                    _ => false,
                })
            });
            if coming {
                continue;
            }
            // What staying offers: its own food, its share of what others could spare those out of
            // food, and its share of the common store if a law its members know keeps one,
            // against the wait to its next harvest.
            let settlement = x.settlement;
            let (spare, short) = match settlement {
                Some(s) => match villages.iter().find(|v| v.0 == s) {
                    Some(v) => (v.1, v.2),
                    None => {
                        let (spare, short) = self.village_food(ctx, s);
                        villages.push((s, spare, short));
                        (spare, short)
                    }
                },
                None => (0.0, 1),
            };
            let short = short.max(1) as f64;
            let relief = settlement
                .and_then(|s| self.polity_of(s))
                .map_or(0.0, |pi| {
                    let p = &self.polities[pi];
                    let knows = p
                        .in_force(
                            &ctx.catalog.policies,
                            crate::polity::PolicyKind::CommonStore,
                        )
                        .any(|l| x.members.iter().any(|&m| l.knows(m)));
                    if knows {
                        stock_kcal(&p.stores, goods)
                    } else {
                        0.0
                    }
                });
            let wait = crop.map_or(365.0, |c| farm::days_to_harvest(c, fields(), day));
            let reach = (food + (spare + relief) / short) / need.max(1.0);
            let gap = 1.0 - reach / wait.max(1.0);
            let stake = self
                .outlook(ctx, household)
                .map_or(0.0, |o| o.harvest / o.year_need.max(1.0));
            let chance = leave_chance(gap, stake, h);
            if life_rng(ctx.seed, household, day, Draw::Leave).next_f64() >= chance {
                continue;
            }
            // They go to a settlement they know that draws them, if one does (M5a slice AN),
            // and else beyond the map.
            self.refresh_kin();
            match self.refuge(ctx, household) {
                Some(to) => self.relocate(ctx, household, to),
                None => self.leave(ctx, household),
            }
        }
    }

    /// What settlement `settlement`'s households could spare a household in need, kcal, and how
    /// many of its households are out of food.
    fn village_food(&self, ctx: &Ctx, settlement: PermanentId) -> (f64, usize) {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let mut out = (0.0, 0);
        let mut homes: Vec<&Household> = self
            .households
            .iter()
            .map(|(_, x)| x)
            .filter(|x| x.settlement == Some(settlement) && !x.members.is_empty())
            .collect();
        homes.sort_by_key(|x| x.id);
        for x in homes {
            out.0 += spare_food_kcal(x, now, params, goods);
            let need = x.members.len() as f64 * params.household.daily_kcal_per_person;
            if stock_kcal(&stores_now(x, now, params, goods), goods) < need {
                out.1 += 1;
            }
        }
        out
    }

    /// Household `household` leaves the world: its people are no longer simulated, their
    /// records say when they left, and its land stands abandoned.
    fn leave(&mut self, ctx: &mut Ctx, household: PermanentId) {
        let now = ctx.now;
        let Some(x) = self.household(household) else {
            return;
        };
        let (members, settlement, home) = (x.members.clone(), x.settlement, x.home);
        let mut gone = Vec::new();
        for m in &members {
            let Some(h) = self.index.remove(m) else {
                continue;
            };
            let Some(p) = self.people.remove(h) else {
                continue;
            };
            gone.push((*m, p.knows.clone()));
            if let Some(r) = self.records.get_mut(m) {
                r.left = Some(now);
            }
            self.note_residence(*m, None, now, ResidenceWhy::LeftMap);
            if let Some(q) = p.partner.filter(|q| !members.contains(q))
                && let Some(partner) = self.person_mut(q)
            {
                partner.partner = None;
            }
        }
        for u in self.unions.iter_mut().filter(|u| u.ended.is_none()) {
            if members.contains(&u.woman) || members.contains(&u.man) {
                u.ended = Some(now);
            }
        }
        self.settle_household(ctx, household);
        if let Some(hd) = self.hh_index.remove(&household)
            && let Some(mut gone) = self.households.remove(hd)
        {
            self.homes_moved += 1;
            self.known_places.forget(household);
            self.reports.forget(household);
            for (g, kg) in gone.stores.iter().enumerate() {
                gone.flows.add(Flow::Departed, g, kg.max(0.0));
            }
            self.flows_gone.absorb(&gone.flows);
        }
        self.hand_over_land(ctx, household, None, false);
        // Ground it leaves may be taken up, or given out again (ADR-0007 §2).
        if let Some(s) = settlement {
            self.review_land(ctx, s);
        }
        let count = members.len() as f64;
        // The settlement they left, by name.
        let left = settlement
            .and_then(|s| ctx.land.settlements.iter().find(|x| x.id == s))
            .map(|x| x.name.clone())
            .unwrap_or_default();
        self.chronicle_push(
            now,
            ChronicleKind::Left,
            members,
            settlement,
            Some(home),
            count,
            left,
        );
        // What only they knew there leaves with them (ADR-0008 §5); the buildings they leave
        // standing still count as made with it.
        self.check_loss(ctx, settlement, &gone, Some(household));
    }

    /// Parents and children of everyone in the records.
    fn family_links(&self) -> HashMap<PermanentId, Vec<PermanentId>> {
        let mut links: HashMap<PermanentId, Vec<PermanentId>> = HashMap::new();
        for r in self.records.values() {
            for parent in [r.mother, r.father].into_iter().flatten() {
                links.entry(r.id).or_default().push(parent);
                links.entry(parent).or_default().push(r.id);
            }
        }
        for v in links.values_mut() {
            v.sort_unstable();
            v.dedup();
        }
        links
    }

    /// The household of `from`'s nearest living relative (through parents and children, a few
    /// steps out) that is not `exclude`; with `adult`, only a relative old enough to keep a
    /// household counts.
    fn kin_household(
        &self,
        ctx: &Ctx,
        from: PermanentId,
        exclude: PermanentId,
        adult: bool,
    ) -> Option<PermanentId> {
        let links = self.family_links();
        let grown = ctx.params.family.independent_age;
        let mut seen = vec![from];
        let mut layer = vec![from];
        for _ in 0..KIN_SEARCH_STEPS {
            let mut next = Vec::new();
            for x in &layer {
                for n in links.get(x).into_iter().flatten() {
                    if !seen.contains(n) {
                        seen.push(*n);
                        next.push(*n);
                    }
                }
            }
            next.sort_unstable();
            let found = next
                .iter()
                .filter_map(|n| self.person(*n))
                .filter(|p| p.household != exclude)
                .find(|p| !adult || p.age_years(ctx.now) >= grown)
                .map(|p| p.household);
            if found.is_some() {
                return found;
            }
            if next.is_empty() {
                break;
            }
            layer = next;
        }
        None
    }

    /// The households of `from`'s nearest living relatives (at the first step out where any is
    /// found), other than `exclude`, with `first` (the household its goods go to) first: who
    /// inherits under a rule that divides (ADR-0007 §2). Relatives in `exclude`'s settlement come
    /// before those elsewhere: only they are counted when there are any, so land is not divided
    /// among households too far away to work it.
    fn heir_households(
        &self,
        from: PermanentId,
        exclude: PermanentId,
        first: Option<PermanentId>,
    ) -> Vec<PermanentId> {
        let Some(first) = first else {
            return Vec::new();
        };
        let settlement = self.household(exclude).and_then(|x| x.settlement);
        let links = self.family_links();
        let mut seen = vec![from];
        let mut layer = vec![from];
        for _ in 0..KIN_SEARCH_STEPS {
            let mut next = Vec::new();
            for x in &layer {
                for n in links.get(x).into_iter().flatten() {
                    if !seen.contains(n) {
                        seen.push(*n);
                        next.push(*n);
                    }
                }
            }
            next.sort_unstable();
            let mut found: Vec<PermanentId> = next
                .iter()
                .filter_map(|n| self.person(*n))
                .map(|p| p.household)
                .filter(|h| *h != exclude)
                .collect();
            if !found.is_empty() {
                found.sort_unstable();
                found.dedup();
                let near: Vec<PermanentId> = found
                    .iter()
                    .copied()
                    .filter(|h| {
                        settlement.is_some()
                            && self.household(*h).and_then(|x| x.settlement) == settlement
                    })
                    .collect();
                if !near.is_empty() {
                    found = near;
                }
                let mut out = vec![first];
                out.extend(found.into_iter().filter(|h| *h != first));
                return out;
            }
            if next.is_empty() {
                break;
            }
            layer = next;
        }
        vec![first]
    }

    /// Where children left alone in `household` go: their nearest adult kin's household, else
    /// the household of their settlement with the most food for each of its members.
    fn foster_household(
        &self,
        ctx: &Ctx,
        household: PermanentId,
        children: &[PermanentId],
    ) -> Option<PermanentId> {
        if let Some(to) = children
            .iter()
            .find_map(|c| self.kin_household(ctx, *c, household, true))
        {
            return Some(to);
        }
        let settlement = self.household(household)?.settlement?;
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let grown = params.family.independent_age;
        let mut best: Option<(f64, PermanentId)> = None;
        for (_, x) in self.households.iter() {
            if x.id == household || x.settlement != Some(settlement) || x.members.is_empty() {
                continue;
            }
            let has_adult = x
                .members
                .iter()
                .any(|m| self.person(*m).is_some_and(|p| p.age_years(now) >= grown));
            if !has_adult {
                continue;
            }
            let food = stock_kcal(&stores_now(x, now, params, goods), goods);
            let per = food / x.members.len() as f64;
            if best.is_none_or(|(b, id)| per > b || (per == b && x.id < id)) {
                best = Some((per, x.id));
            }
        }
        best.map(|(_, id)| id)
    }

    /// Household `from` joins household `to`: its members, stores, water, what it knows of the
    /// land, and its plots and buildings, and with `fields` its fields too (when its people move
    /// with them; the fields of a household that is no more go by the regime's succession rule
    /// instead). `from` is no more.
    fn merge_household(
        &mut self,
        ctx: &mut Ctx,
        from: PermanentId,
        to: PermanentId,
        fields: bool,
        why: ResidenceWhy,
    ) {
        if from == to || self.household(to).is_none() {
            return;
        }
        self.settle_household(ctx, from);
        self.settle_household(ctx, to);
        // Everything it holds passes to the household its people join.
        let legs: Vec<Leg> = self
            .household(from)
            .map(|x| {
                x.stores
                    .iter()
                    .enumerate()
                    .filter(|&(_, &kg)| kg > 0.0)
                    .map(|(g, &kg)| Leg {
                        from,
                        to,
                        good: g,
                        amount: kg,
                    })
                    .collect()
            })
            .unwrap_or_default();
        self.transfer(
            ctx.now,
            ctx.params,
            &ctx.catalog.goods,
            &legs,
            Channel::Inherit,
        );
        // What its people knew of other places goes with them (ADR-0018 §4).
        let home = self.household(to).and_then(|x| x.settlement);
        self.known_places.bring(from, to, home);
        self.reports.bring(from, to, home);
        self.known_places.forget(from);
        self.reports.forget(from);
        let Some(hd) = self.hh_index.remove(&from) else {
            return;
        };
        let Some(gone) = self.households.remove(hd) else {
            return;
        };
        self.homes_moved += 1;
        self.flows_gone.absorb(&gone.flows);
        let settlement = self.household(to).and_then(|x| x.settlement);
        for m in &gone.members {
            if let Some(p) = self.person_mut(*m) {
                p.household = to;
            }
            self.note_residence(*m, settlement, ctx.now, why);
        }
        self.known_pruned.retain(|&(h, _), _| h != to);
        if let Some(x) = self.household_mut(to) {
            x.members.extend(gone.members.iter().copied());
            x.water_l += gone.water_l;
            for k in gone.known {
                match x
                    .known
                    .iter_mut()
                    .find(|j| j.resource == k.resource && j.patch == k.patch)
                {
                    Some(j) if j.seen_day < k.seen_day => *j = k,
                    Some(_) => {}
                    None => x.known.push(k),
                }
            }
        }
        self.sort_members(to);
        self.hand_over_land(ctx, from, Some(to), fields);
    }

    /// The land of household `from` passes to `to`, or, with no one to take it, stays as it is
    /// (fields fall fallow, huts stand empty): its plots and buildings, and with `fields` the
    /// fields it holds and works.
    pub(super) fn hand_over_land(
        &mut self,
        ctx: &mut Ctx,
        from: PermanentId,
        to: Option<PermanentId>,
        fields: bool,
    ) {
        // Its workshops go with it (slice J).
        self.pass_firms(ctx, from, to);
        self.homes.remove(&from);
        self.sites.remove(&from);
        self.home_sites.remove(&from);
        let Some(to) = to else {
            return;
        };
        if fields {
            for f in &mut ctx.land.fields {
                if f.household == from {
                    f.household = to;
                    self.fields_moved();
                }
                if f.holder == Party::Household(from) {
                    f.holder = Party::Household(to);
                }
                // Holder and tenant under one roof: no lease between them.
                if f.holder == Party::Household(f.household) {
                    f.lease = None;
                }
            }
        }
        for p in ctx.land.plots.iter_mut().filter(|p| p.household == from) {
            p.household = to;
        }
        for b in ctx
            .land
            .buildings
            .iter_mut()
            .filter(|b| b.household == from)
        {
            b.household = to;
        }
        self.sites.remove(&to);
        self.home_sites.remove(&to);
        let shelter = super::shelter_of(ctx.land, ctx.catalog, ctx.params, to);
        if let Some(x) = self.household_mut(to) {
            (x.sheltered, x.keeping) = shelter;
        }
    }

    /// A household with no one left goes to `heir`, or is no more. Its fields go as the regime's
    /// succession rule says, to `heirs` or back to its settlement (ADR-0007 §2), and its
    /// settlement's land is reviewed.
    fn dissolve(
        &mut self,
        ctx: &mut Ctx,
        household: PermanentId,
        heir: Option<PermanentId>,
        heirs: &[PermanentId],
    ) {
        let settlement = self.household(household).and_then(|x| x.settlement);
        self.succeed(ctx, household, heirs);
        match heir {
            Some(to) => self.merge_household(ctx, household, to, false, ResidenceWhy::TakenIn),
            None => {
                self.settle_household(ctx, household);
                if let Some(hd) = self.hh_index.remove(&household)
                    && let Some(mut gone) = self.households.remove(hd)
                {
                    self.homes_moved += 1;
                    self.known_places.forget(household);
                    self.reports.forget(household);
                    // What nobody is left to keep is left behind.
                    for (g, kg) in gone.stores.iter().enumerate() {
                        gone.flows.add(Flow::Departed, g, kg.max(0.0));
                    }
                    self.flows_gone.absorb(&gone.flows);
                }
                self.hand_over_land(ctx, household, None, false);
            }
        }
        if let Some(s) = settlement {
            self.review_land(ctx, s);
        }
    }

    fn conceptions(&mut self, ctx: &mut Ctx, day: i64) {
        let (now, params) = (ctx.now, ctx.params);
        let f = &params.fertility;
        for id in self.living_ids() {
            let partner_home = self
                .person(id)
                .and_then(|p| p.partner)
                .and_then(|q| self.person(q))
                .map(|q| q.household);
            let Some(p) = self.person_mut(id) else {
                continue;
            };
            if p.sex != Sex::Female {
                continue;
            }
            if let Repro::Recovering { until } = p.repro
                && until <= now
            {
                p.repro = Repro::Open;
                p.nursing = None;
            }
            // Exposure: an open woman living with her partner (research 04-08 §1.5).
            if p.repro != Repro::Open || partner_home != Some(p.household) {
                continue;
            }
            let age = p.age_years(now);
            let chance =
                conception_chance(f, age, f64::from(p.fecundity), depleted(p, now, params));
            if chance <= 0.0 || life_rng(ctx.seed, id, day, Draw::Conception).next_f64() >= chance {
                continue;
            }
            let mut rng = life_rng(ctx.seed, id, day, Draw::Pregnancy);
            let (due, loss) = pregnancy_course(f, age, now, &mut rng);
            p.repro = Repro::Pregnant {
                conceived: now,
                due,
                father: p.partner,
                loss,
            };
        }
    }

    fn partnering(&mut self, ctx: &mut Ctx, day: i64) {
        let (now, fam) = (ctx.now, ctx.params.family);
        for id in self.living_ids() {
            let Some(p) = self.person(id) else {
                continue;
            };
            let (sex, age) = (p.sex, p.age_years(now));
            if p.partner.is_some() || !fam.seeks_at(sex, age) {
                continue;
            }
            let chance = daily_from_monthly(fam.seek_per_month[FamilyParams::of(sex)]);
            let mut rng = life_rng(ctx.seed, id, day, Draw::Partner);
            if rng.next_f64() >= chance {
                continue;
            }
            let Some(other) = self.find_partner(ctx, id, &mut rng) else {
                // They looked and found nobody (M5a slice AM): a reason to look elsewhere.
                self.unmatched.insert(id, day);
                continue;
            };
            self.unmatched.remove(&id);
            self.unmatched.remove(&other);
            let (woman, man) = if sex == Sex::Female {
                (id, other)
            } else {
                (other, id)
            };
            self.unite(ctx, woman, man);
        }
    }

    /// Whom `id` finds: an unpartnered adult of the other sex in their settlement, or in another
    /// whom they hold a tie with, of an age either would accept and not close kin (research 04-08
    /// §1.1: eligibility, then acceptance; 06-01 §2.4), weighed by the age gap people look for.
    fn find_partner(&self, ctx: &Ctx, id: PermanentId, rng: &mut Rng64) -> Option<PermanentId> {
        let (now, fam) = (ctx.now, &ctx.params.family);
        let me = self.person(id)?;
        let settlement = self.household(me.household)?.settlement;
        let my_age = me.age_years(now);
        let day = now.day_index();
        let mut candidates: Vec<(PermanentId, f64)> = Vec::new();
        for (_, q) in self.people.iter() {
            if q.sex == me.sex || q.partner.is_some() {
                continue;
            }
            let age = q.age_years(now);
            if !fam.seeks_at(q.sex, age) {
                continue;
            }
            // Those of their own settlement, and of another the one they hold a tie with (M5a
            // slice AM; research 04-08 §1.1: the pool is those one has met).
            let theirs = self.household(q.household).map(|x| x.settlement);
            if theirs != Some(settlement)
                && (theirs.flatten().is_none()
                    || self.ties.known(id, q.id, day, &ctx.params.ties) <= 0.0)
            {
                continue;
            }
            let (woman, man) = if me.sex == Sex::Female {
                (my_age, age)
            } else {
                (age, my_age)
            };
            let Some(score) = couple_score(fam, woman, man) else {
                continue;
            };
            if too_close(&self.records, id, q.id, fam.kin_exclusion_generations) {
                continue;
            }
            candidates.push((q.id, score));
        }
        candidates.sort_by_key(|c| c.0);
        let scores: Vec<f64> = candidates.iter().map(|c| c.1).collect();
        pick_softmax(&scores, rng.next_f64()).map(|i| candidates[i].0)
    }

    /// Whether `id` is the only member of their household old enough to keep it.
    fn keeps_household(&self, ctx: &Ctx, id: PermanentId) -> bool {
        let grown = ctx.params.family.independent_age;
        let Some(p) = self.person(id) else {
            return false;
        };
        self.household(p.household).is_some_and(|x| {
            x.members
                .iter()
                .all(|m| *m == id || self.person(*m).is_none_or(|q| q.age_years(ctx.now) < grown))
        })
    }

    /// `id` and the children of theirs too young to keep a household who live with them.
    fn with_dependants(&self, ctx: &Ctx, id: PermanentId) -> Vec<PermanentId> {
        let grown = ctx.params.family.independent_age;
        let mut out = vec![id];
        let Some(p) = self.person(id) else {
            return out;
        };
        if let Some(x) = self.household(p.household) {
            for m in &x.members {
                let child = self
                    .records
                    .get(m)
                    .is_some_and(|r| r.mother == Some(id) || r.father == Some(id));
                if child
                    && self
                        .person(*m)
                        .is_some_and(|c| c.age_years(ctx.now) < grown)
                {
                    out.push(*m);
                }
            }
        }
        out
    }

    /// How far a household's home is built: its best dwelling's stage, roofed ones first.
    fn home_built(&self, ctx: &Ctx, household: PermanentId) -> u8 {
        ctx.land
            .buildings
            .iter()
            .filter(|b| {
                b.household == household && b.standing() && ctx.catalog.is_dwelling(&b.spec.program)
            })
            .map(|b| b.stage.saturating_add(if b.roofed() { 10 } else { 0 }))
            .max()
            .unwrap_or(0)
    }

    /// Two people become partners and settle: with the one who keeps a household alone if
    /// either does, otherwise as the content's residence rule says (research 06-01 §1.3: the
    /// residence of a new couple is its own rule).
    fn unite(&mut self, ctx: &mut Ctx, woman: PermanentId, man: PermanentId) {
        let now = ctx.now;
        for (a, b) in [(woman, man), (man, woman)] {
            if let Some(p) = self.person_mut(a) {
                p.partner = Some(b);
            }
        }
        self.unions.push(Union {
            woman,
            man,
            since: now,
            ended: None,
        });
        let (Some(hw), Some(hm)) = (
            self.person(woman).map(|p| p.household),
            self.person(man).map(|p| p.household),
        ) else {
            return;
        };
        let rule = ctx.params.family.residence;
        // Partners of two settlements (M5a slice AM; ADR-0018 §5): they settle with or beside
        // the household with more land worked per member, then the better housed, then by lot,
        // never by who is the woman or the man.
        let settlement_of =
            |p: &Population, h: PermanentId| p.household(h).and_then(|x| x.settlement);
        let before = (settlement_of(self, hw), settlement_of(self, hm));
        let across = matches!(before, (Some(a), Some(b)) if a != b);
        let to_hers_across = across.then(|| {
            let side = |h: PermanentId| {
                let members = self.household(h).map_or(1, |x| x.members.len().max(1));
                let ha: f64 = self.fields_of(ctx.land, h).map(|f| f.area_ha()).sum();
                (ha / members as f64, self.home_built(ctx, h))
            };
            let (first, second) = if hw < hm { (hw, hm) } else { (hm, hw) };
            let draw = life_rng(ctx.seed, woman.min(man), now.day_index(), Draw::Settle).next_f64();
            let first_wins = crate::places::settles_with_first(side(first), side(second), draw);
            (first == hw) == first_wins
        });
        let moved = if hw == hm {
            Moved::Stayed
        } else if let Some(to_hers) = to_hers_across {
            match (
                self.keeps_household(ctx, woman),
                self.keeps_household(ctx, man),
            ) {
                (true, true) if to_hers => {
                    self.merge_household(ctx, hm, hw, true, ResidenceWhy::Married);
                    Moved::HisToHers
                }
                (true, true) => {
                    self.merge_household(ctx, hw, hm, true, ResidenceWhy::Married);
                    Moved::HerToHis
                }
                (true, false) => {
                    let group = self.with_dependants(ctx, man);
                    self.move_people(ctx, &group, hm, hw, ResidenceWhy::Married);
                    Moved::HisToHers
                }
                (false, true) => {
                    let group = self.with_dependants(ctx, woman);
                    self.move_people(ctx, &group, hw, hm, ResidenceWhy::Married);
                    Moved::HerToHis
                }
                (false, false) => match rule {
                    Residence::NewHousehold => {
                        self.new_household(ctx, woman, man, if to_hers { hw } else { hm });
                        Moved::NewHousehold
                    }
                    Residence::HisHousehold | Residence::HerHousehold if to_hers => {
                        let group = self.with_dependants(ctx, man);
                        self.move_people(ctx, &group, hm, hw, ResidenceWhy::Married);
                        Moved::HisToHers
                    }
                    Residence::HisHousehold | Residence::HerHousehold => {
                        let group = self.with_dependants(ctx, woman);
                        self.move_people(ctx, &group, hw, hm, ResidenceWhy::Married);
                        Moved::HerToHis
                    }
                },
            }
        } else {
            match (
                self.keeps_household(ctx, woman),
                self.keeps_household(ctx, man),
            ) {
                (true, true) => {
                    let to_his = match rule {
                        Residence::HisHousehold => true,
                        Residence::HerHousehold => false,
                        Residence::NewHousehold => {
                            self.home_built(ctx, hm) > self.home_built(ctx, hw)
                        }
                    };
                    if to_his {
                        self.merge_household(ctx, hw, hm, true, ResidenceWhy::Married);
                        Moved::HerToHis
                    } else {
                        self.merge_household(ctx, hm, hw, true, ResidenceWhy::Married);
                        Moved::HisToHers
                    }
                }
                (true, false) => {
                    let group = self.with_dependants(ctx, man);
                    self.move_people(ctx, &group, hm, hw, ResidenceWhy::Married);
                    Moved::HisToHers
                }
                (false, true) => {
                    let group = self.with_dependants(ctx, woman);
                    self.move_people(ctx, &group, hw, hm, ResidenceWhy::Married);
                    Moved::HerToHis
                }
                (false, false) => match rule {
                    Residence::NewHousehold => {
                        self.new_household(ctx, woman, man, hw);
                        Moved::NewHousehold
                    }
                    Residence::HisHousehold => {
                        let group = self.with_dependants(ctx, woman);
                        self.move_people(ctx, &group, hw, hm, ResidenceWhy::Married);
                        Moved::HerToHis
                    }
                    Residence::HerHousehold => {
                        let group = self.with_dependants(ctx, man);
                        self.move_people(ctx, &group, hm, hw, ResidenceWhy::Married);
                        Moved::HisToHers
                    }
                },
            }
        };
        let household = self.person(woman).map(|p| p.household);
        let place = household.and_then(|h| self.household(h)).map(|x| x.home);
        let settlement = household
            .and_then(|h| self.household(h))
            .and_then(|x| x.settlement);
        // A marriage between settlements (M5a slice AM): counted, and each household knows the
        // other's settlement as where kin live (ADR-0018 §4); the chronicle names where they
        // settled.
        let named = if across {
            settlement
                .and_then(|s| ctx.land.settlements.iter().find(|x| x.id == s))
                .map_or_else(String::new, |s| s.name.clone())
        } else {
            String::new()
        };
        if let (true, Some(couple), Some(after)) = (across, household, settlement) {
            let day = now.day_index();
            for (who, natal, from) in [(woman, hw, before.0), (man, hm, before.1)] {
                let Some(from) = from.filter(|&f| f != after) else {
                    continue;
                };
                self.contacts
                    .marriage(day.div_euclid(civ_core::time::DAYS_PER_YEAR), from, after);
                if self.household(natal).is_some() {
                    self.known_places.learn(
                        natal,
                        after,
                        day,
                        crate::places::PlaceHow::Kin,
                        Some(who),
                    );
                }
                self.known_places
                    .learn(couple, from, day, crate::places::PlaceHow::Kin, None);
            }
        }
        self.chronicle_push(
            now,
            ChronicleKind::Paired,
            vec![woman, man],
            settlement,
            place,
            f64::from(moved as u8),
            named,
        );
    }

    /// `people` (all of household `from`) move to household `to`, taking a member's share each
    /// of what it holds in store, but not the materials of its building. If they are all its
    /// members, the households merge.
    fn move_people(
        &mut self,
        ctx: &mut Ctx,
        people: &[PermanentId],
        from: PermanentId,
        to: PermanentId,
        why: ResidenceWhy,
    ) {
        if from == to || people.is_empty() || self.household(to).is_none() {
            return;
        }
        let Some(before) = self.household(from).map(|x| x.members.len()) else {
            return;
        };
        // What they knew of other places goes with them (ADR-0018 §4).
        let home = self.household(to).and_then(|x| x.settlement);
        self.known_places.bring(from, to, home);
        self.reports.bring(from, to, home);
        if people.len() >= before {
            self.merge_household(ctx, from, to, true, why);
            return;
        }
        self.settle_household(ctx, from);
        self.settle_household(ctx, to);
        let goods = &ctx.catalog.goods;
        let share = people.len() as f64 / before as f64;
        // They take a member's share of the household's goods (research 08-06 §1.1: household
        // allocation). Building materials stay with the home they are for, and an oven with the
        // hearth it was built at.
        let legs: Vec<Leg> = self
            .household(from)
            .map(|x| {
                x.stores
                    .iter()
                    .enumerate()
                    .filter(|&(g, &kg)| {
                        kg > 0.0
                            && goods.get(g).is_some_and(|d| {
                                d.purpose != GoodUse::Material
                                    && !d.tool.as_ref().is_some_and(|t| t.fixed)
                            })
                    })
                    .map(|(g, &kg)| Leg {
                        from,
                        to,
                        good: g,
                        amount: kg * share,
                    })
                    .collect()
            })
            .unwrap_or_default();
        self.transfer(ctx.now, ctx.params, goods, &legs, Channel::Allocation);
        let mut water = 0.0;
        let mut known = Vec::new();
        if let Some(x) = self.household_mut(from) {
            water = x.water_l * share;
            x.water_l -= water;
            x.members.retain(|m| !people.contains(m));
            known = x.known.clone();
        }
        self.known_pruned.retain(|&(h, _), _| h != to);
        if let Some(x) = self.household_mut(to) {
            x.water_l += water;
            x.members.extend(people.iter().copied());
            if x.known.is_empty() {
                x.known = known;
            }
        }
        let settlement = self.household(to).and_then(|x| x.settlement);
        for m in people {
            if let Some(p) = self.person_mut(*m) {
                p.household = to;
            }
            self.note_residence(*m, settlement, ctx.now, why);
        }
        self.sort_members(to);
    }

    /// A couple sets up a household of their own: near the woman's household, a little farther
    /// from the hearth, each bringing a member's share of what their household holds and any
    /// children too young to stay behind.
    /// A couple sets up a household of its own beside household `beside`, one of theirs (its
    /// settlement's, and its home's neighbour).
    fn new_household(
        &mut self,
        ctx: &mut Ctx,
        woman: PermanentId,
        man: PermanentId,
        beside: PermanentId,
    ) {
        let Some(natal) = self.household(beside).cloned() else {
            return;
        };
        let hearth = natal.settlement.and_then(|s| {
            ctx.land
                .settlements
                .iter()
                .find(|x| x.id == s)
                .map(|x| x.hearth_m)
        });
        let id = ctx.ids.allocate();
        let home = self.new_home_site(ctx, natal.home, hearth, id);
        // They build as the households they grew up in did, halfway between the two, and after
        // the building that had moved hers most, else his (M3b slice R).
        let of = |who: PermanentId| {
            self.person(who)
                .and_then(|p| self.household(p.household))
                .map_or((natal.taste, natal.admired), |h| (h.taste, h.admired))
        };
        let (hers, his) = (of(woman), of(man));
        let (taste, admired) = crate::style::couple_taste((&hers.0, hers.1), (&his.0, his.1));
        self.insert_household(Household {
            id,
            members: Vec::new(),
            home,
            settlement: natal.settlement,
            stores: vec![0.0; ctx.catalog.goods.len()],
            stores_at: ctx.now,
            water_l: 0.0,
            water_at: ctx.now,
            known: natal.known.clone(),
            sheltered: false,
            keeping: crate::person::Keeping::default(),
            flows: Flows::default(),
            offers: Vec::new(),
            taste,
            admired,
            midden: crate::person::Midden::begun(ctx.now),
        });
        for who in [woman, man] {
            let Some(from) = self.person(who).map(|p| p.household) else {
                continue;
            };
            let group = self.with_dependants(ctx, who);
            let before = self.household(from).map_or(0, |x| x.members.len());
            self.move_people(ctx, &group, from, id, ResidenceWhy::Married);
            // A share of the family's fields, as of its stores, where the regime says so
            // (ADR-0007 §2); a whole household that moves brings all of them.
            if ctx.regime.union_share && before > group.len() && self.household(from).is_some() {
                self.share_fields(ctx, from, id, group.len() as f64 / before as f64);
            }
        }
        if let Some(s) = natal.settlement {
            self.review_land(ctx, s);
        }
        // A household formed reviews where to live (M5a slice AN, ADR-0018 §5).
        self.review_due.insert(id);
    }

    /// Where a new household's home goes: `near`, moved a little away from the hearth, on dry
    /// ground a short walk from it (the hut is then sited on clear ground nearby, as every
    /// household's is).
    pub(super) fn new_home_site(
        &self,
        ctx: &Ctx,
        near: (f32, f32),
        hearth: Option<(f32, f32)>,
        key: PermanentId,
    ) -> (f32, f32) {
        let Some(hearth) = hearth else {
            return near;
        };
        let (dx, dy) = (near.0 - hearth.0, near.1 - hearth.1);
        let angle = if dx.abs() + dy.abs() > 0.5 {
            f64::from(dy).atan2(f64::from(dx))
        } else {
            (key.get() % 360) as f64 / 360.0 * std::f64::consts::TAU
        };
        let wanted = (
            near.0 + (NEW_HOME_STEP_M * angle.cos()) as f32,
            near.1 + (NEW_HOME_STEP_M * angle.sin()) as f32,
        );
        let cell = cell_of(ctx.map, hearth);
        let field = ctx.nav.travel_field(
            &ctx.map.elevation,
            cell,
            crate::found::HOME_REACH_SECONDS,
            &|_| 0.0,
        );
        crate::found::home_site(ctx.map, &field, wanted).unwrap_or(near)
    }
}

/// The chance a household out of food and worn down leaves today (M4a slice Z): `leave_per_day`
/// times the logistic of the points for going, `leave_w_gap` for the whole of the wait to its next
/// harvest uncovered (`gap`, 0-1), less those for staying, `leave_w_stake` for a whole year's food
/// its fields should bring (`stake`, 0-1) and `leave_stay`. A household that others or a store
/// could carry to its harvest seldom goes; one with nothing to wait for goes at nearly the full
/// rate.
pub(crate) fn leave_chance(gap: f64, stake: f64, h: &HouseholdParams) -> f64 {
    let points = h.leave_w_gap * gap.clamp(0.0, 1.0)
        - h.leave_w_stake * stake.clamp(0.0, 1.0)
        - h.leave_stay;
    h.leave_per_day / (1.0 + (-points).exp())
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_household_others_could_carry_seldom_leaves_and_one_with_nothing_goes() {
        let h = crate::found::tests::params().household;
        let nothing = super::leave_chance(1.0, 0.0, &h);
        let carried = super::leave_chance(0.0, 0.0, &h);
        let fields = super::leave_chance(1.0, 1.0, &h);
        assert!(nothing > 0.9 * h.leave_per_day, "{nothing}");
        assert!(carried < 0.2 * h.leave_per_day, "{carried}");
        assert!(fields < nothing, "fields to lose hold some back");
        assert!(super::leave_chance(5.0, -1.0, &h) <= h.leave_per_day);
    }
}
