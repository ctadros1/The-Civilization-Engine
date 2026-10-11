//! Sickness among people (M6a slice AZ; ADR-0021 §4, §5, §8; research 05-03 §7.2-§7.6). At each
//! day's turn every infection runs its course: someone severely ill may die of it, by a draw of
//! their own each day; one whose course has run recovers, protected for a while. Then what people
//! shed moves through the ground and water (`contagion`), and whoever is exposed today, by every
//! route (living with those who shed it, and what they drank of their household's water), takes
//! it or not by one draw (1 − e^(−ΣH)), recorded as coming by the route that brought the most of
//! it. The observer's plague tool brings a disease to one person, as if they took it elsewhere.
//! Nothing here is read by any choice: people know only who they see abed. A household's members
//! see someone of theirs abed, and its close kin learn of a death of it, as a claim that the
//! household had sickness from a day, told as any word is (M6a slice BA, ADR-0021 §6).

use super::*;
use crate::Cause;
use crate::decide::TendFacts;
use crate::influence::InfluenceKind;
use crate::params::DiseaseRoute;
use crate::sickness::{Acquired, Outcome, SickDraw, course, sickness_rng};

impl Population {
    /// The day's turn for infections (see the module documentation).
    pub(super) fn sickness_day(&mut self, ctx: &mut Ctx, day: i64) {
        if self.sickness.quiet() && self.contagion.is_empty() && !self.sickness.outbreak_open() {
            return;
        }
        let running = self.sickness.running().to_vec();
        for i in running {
            let Some(e) = self.sickness.get(i).copied() else {
                continue;
            };
            // Dead of something else, or gone from the world, while it ran.
            if self.person(e.person).is_none() {
                self.sickness.end(i, day, Outcome::Gone);
                continue;
            }
            // The day just lived, severely ill, may have killed them: less likely the more care
            // they were given that day, and less still if a carer knew a treatment (step three;
            // 05-05 §2.2, 05-03 §3.2).
            let lived = day - 1;
            if e.course.severe
                && e.ill_on(lived)
                && let Some(def) = ctx.catalog.diseases.get(usize::from(e.disease))
            {
                let h = (def.severe_death_per_day * e.care.rr(def, lived)).clamp(0.0, 1.0);
                let u =
                    sickness_rng(ctx.seed, e.person, lived, e.disease, SickDraw::Death).next_f64();
                // A blessing or a curse moves their own chance, as it moves illness's in the life
                // table (ADR-0016 §5); what it turned is noted.
                let luck = self.influences.luck(e.person, lived);
                let chance = luck.map_or(h, |l| (h * l.harm()).min(1.0));
                if let Some(l) = luck
                    && (u < chance) != (u < h)
                {
                    self.note_turned(ctx, e.person, l, None);
                }
                if u < chance {
                    let name = format!("disease:{}", def.name.to_lowercase());
                    // Their close kin learn of the death, and with it of the sickness (ADR-0021
                    // §6), wherever they live.
                    let home = self
                        .person(e.person)
                        .map(|p| p.household)
                        .and_then(|h| Some((h, self.household(h)?.settlement?)));
                    self.refresh_kin();
                    let kin = self.close_kin(e.person);
                    self.sickness.end(i, day, Outcome::Died);
                    self.die_named(ctx, e.person, Cause::Disease, name);
                    if let Some((household, settlement)) = home {
                        let claim = self.sickness_claim(household, settlement, lived, None);
                        for k in kin {
                            self.word.hear(k, claim, day, None, Some(k));
                        }
                    }
                    continue;
                }
            }
            if e.run_by(day) {
                self.sickness.end(i, day, Outcome::Recovered);
            }
        }
        // What people shed moves through the ground and water (step two).
        self.contagion_day(ctx, day);
        // Who is exposed today, by every route, before anyone takes it: nobody infected today
        // sheds before tomorrow, so no chain runs within a day.
        let mut shedding: BTreeMap<(PermanentId, u16), u32> = BTreeMap::new();
        for &i in self.sickness.running() {
            let Some(e) = self.sickness.get(i) else {
                continue;
            };
            let household_route = ctx
                .catalog
                .diseases
                .get(usize::from(e.disease))
                .is_some_and(|d| d.passes_by(DiseaseRoute::Household));
            if household_route
                && e.shedding(day)
                && let Some(p) = self.person(e.person)
            {
                *shedding.entry((p.household, e.disease)).or_default() += 1;
            }
        }
        // Each person's hazards of each disease, by route.
        let mut exposed: BTreeMap<(PermanentId, u16), Vec<(f64, Acquired)>> = BTreeMap::new();
        for ((household, disease), shedders) in shedding {
            let Some(def) = ctx.catalog.diseases.get(usize::from(disease)) else {
                continue;
            };
            let Some(x) = self.household(household) else {
                continue;
            };
            let hazard = def.household_hazard.max(0.0) * f64::from(shedders);
            for &m in &x.members {
                exposed
                    .entry((m, disease))
                    .or_default()
                    .push((hazard, Acquired::Household { household }));
            }
        }
        // A dose drunk is a hazard of one (12-02 §5.4's exponential, its rate in the dose).
        for (m, disease, dose, source) in self.water_doses(ctx) {
            exposed
                .entry((m, disease))
                .or_default()
                .push((dose, Acquired::Water { source }));
        }
        let mut taken = Vec::new();
        for ((m, disease), hazards) in exposed {
            if self.sickness.protected(m, disease, day) {
                continue;
            }
            let total: f64 = hazards.iter().map(|(h, _)| h.max(0.0)).sum();
            if total <= 0.0 {
                continue;
            }
            let u = sickness_rng(ctx.seed, m, day, disease, SickDraw::Infection).next_f64();
            if u < 1.0 - (-total).exp() {
                // Taken by the route that brought the most of it.
                let (_, acquired) = hazards
                    .iter()
                    .copied()
                    .reduce(|a, b| if b.0 > a.0 { b } else { a })
                    .expect("a hazard");
                taken.push((m, disease, acquired));
            }
        }
        for (m, disease, acquired) in taken {
            self.infect(ctx, m, disease, day, acquired);
        }
        self.end_outbreaks(ctx, day);
        self.sickness_seen(ctx, day);
    }

    /// Sickness seen (M6a slice BA, ADR-0021 §6): the members of each household with someone abed
    /// today see it, at first hand. A household's sickness first seen more than the content's
    /// spell ago is new sickness, a new claim.
    fn sickness_seen(&mut self, ctx: &Ctx, day: i64) {
        let spell = i64::from(ctx.params.word.sickness_spell_days);
        let mut abed: Vec<PermanentId> = self
            .sickness
            .running()
            .iter()
            .filter_map(|&i| self.sickness.get(i))
            .filter(|e| e.ill_on(day))
            .filter_map(|e| self.person(e.person).map(|p| p.household))
            .collect();
        abed.sort_unstable();
        abed.dedup();
        for household in abed {
            let Some((settlement, members)) = self
                .household(household)
                .and_then(|x| Some((x.settlement?, x.members.clone())))
            else {
                continue;
            };
            let claim = self.sickness_claim(household, settlement, day, Some(spell));
            for m in members {
                self.word.hear(m, claim, day, None, Some(m));
            }
        }
    }

    /// The claim that someone of `household` (of `settlement`) lay sick: the one held, if it was
    /// first seen no more than `spell` days before `day` (any held, without a spell), or else a new
    /// one from `day`.
    fn sickness_claim(
        &mut self,
        household: PermanentId,
        settlement: PermanentId,
        day: i64,
        spell: Option<i64>,
    ) -> u32 {
        match self.word.sickness(household) {
            Some(c) if spell.is_none_or(|s| day - c.day <= s) => c.id,
            _ => self.word.make(crate::word::Claim {
                id: 0,
                kind: crate::word::ClaimKind::Sickness,
                settlement,
                day,
                subject: Some(household),
                grievance: None,
                suspected: None,
            }),
        }
    }

    /// Ends each open outbreak none of whose cases has run for as long as a new infection of its
    /// disease can take to show (its incubation's mean and three spreads, at least a day), and
    /// tells it with its counts.
    fn end_outbreaks(&mut self, ctx: &Ctx, day: i64) {
        let open: Vec<_> = self
            .sickness
            .outbreaks()
            .iter()
            .filter(|o| o.ended.is_none())
            .copied()
            .collect();
        for o in open {
            let Some(def) = ctx.catalog.diseases.get(usize::from(o.disease)) else {
                continue;
            };
            let (mut took, mut ill, mut died, mut last) = (0u32, 0u32, 0u32, o.began);
            let mut runs = false;
            for e in self.sickness.cases(o.id) {
                took += 1;
                ill += u32::from(e.symptomatic());
                match e.ended {
                    Some((d, outcome)) => {
                        last = last.max(d);
                        died += u32::from(outcome == Outcome::Died);
                    }
                    None => runs = true,
                }
            }
            let quiet = (def.incubation.mean + 3.0 * def.incubation.sd)
                .ceil()
                .max(1.0) as i64;
            if runs || day - last < quiet {
                continue;
            }
            self.sickness.end_outbreak(o.id, day);
            let place = o.settlement.map_or_else(
                || "the people abroad".to_owned(),
                |s| self.settlement_name(ctx, s),
            );
            let count = |n: u32, one: &str, many: &str| match n {
                0 => format!("nobody {many}"),
                1 => format!("one {one}"),
                n => format!("{n} {many}"),
            };
            let sentence = format!(
                "{} has left {place}: {} over {} days; {} and {}.",
                def.name,
                count(took, "took it", "took it"),
                (last - o.began).max(1),
                count(ill, "fell ill", "fell ill"),
                count(died, "died", "died"),
            );
            self.chronicle_push(
                ctx.now,
                ChronicleKind::Outbreak,
                Vec::new(),
                o.settlement,
                None,
                f64::from(crate::OutbreakStep::Ended as u8),
                sentence,
            );
        }
    }

    /// What person `who` of household `x` sees owed to its members ill at home today (step
    /// three): see [`TendFacts`].
    pub(super) fn tend_facts(
        &self,
        ctx: &Ctx,
        who: PermanentId,
        x: &Household,
    ) -> Option<TendFacts> {
        if self.sickness.quiet() {
            return None;
        }
        let day = ctx.now.day_index();
        let mut owed = 0.0;
        for &m in x.members.iter().filter(|&&m| m != who) {
            for i in self.sickness.ill_with(m, day) {
                let Some(e) = self.sickness.get(i) else {
                    continue;
                };
                let Some(def) = ctx.catalog.diseases.get(usize::from(e.disease)) else {
                    continue;
                };
                let need = def.care_h_per_day.max(1e-6);
                let left = ((need - self.sickness.cared(i, day)) / need).clamp(0.0, 1.0);
                owed += left * if e.course.severe { 1.0 } else { 0.5 };
            }
        }
        (owed > 1e-9).then_some(TendFacts { owed })
    }

    /// Person `who` of household `household` sat `minutes` with its members ill at home, at
    /// activity `def` (step three): the hours are shared among them, each episode recording who
    /// came and the least relative risk of dying a treatment the carer knows brings. The carer
    /// practises each treatment they know, and one who does not know it learns it from a member
    /// who does (ADR-0008 §4).
    pub(super) fn tended(
        &mut self,
        ctx: &mut Ctx,
        who: PermanentId,
        household: PermanentId,
        def: u16,
        minutes: u32,
    ) {
        let day = ctx.now.day_index();
        let (Some(x), Some(carer)) = (self.household(household), self.person(who)) else {
            return;
        };
        let ill: Vec<usize> = x
            .members
            .iter()
            .filter(|&&m| m != who)
            .flat_map(|&m| self.sickness.ill_with(m, day))
            .collect();
        if ill.is_empty() {
            return;
        }
        let hours = f64::from(minutes) / 60.0;
        let mut given = Vec::new();
        let mut treatments: Vec<usize> = Vec::new();
        for &i in &ill {
            let Some(d) = self
                .sickness
                .get(i)
                .and_then(|e| ctx.catalog.diseases.get(usize::from(e.disease)))
            else {
                continue;
            };
            let rr = d
                .treatments
                .iter()
                .filter(|(t, _)| carer.knows(*t))
                .map(|(_, rr)| *rr)
                .fold(1.0, f64::min);
            given.push((i, rr));
            treatments.extend(d.treatments.iter().map(|(t, _)| *t));
        }
        for (i, rr) in given {
            self.sickness
                .tend(i, day, who, hours / ill.len() as f64, rr);
        }
        treatments.sort_unstable();
        treatments.dedup();
        for t in treatments {
            self.practise(ctx, who, t, def, Target::Home, hours);
        }
    }

    /// Person `who` of household `household` tends its ill at home for `minutes` now, at the
    /// tending activity: for tests.
    #[doc(hidden)]
    pub fn tended_for_tests(
        &mut self,
        ctx: &mut Ctx,
        who: PermanentId,
        household: PermanentId,
        minutes: u32,
    ) {
        let def = ctx
            .catalog
            .activities
            .iter()
            .position(|a| a.behavior == Behavior::Tend)
            .map_or(0, |i| i as u16);
        self.tended(ctx, who, household, def, minutes);
    }

    /// `person` takes `disease` on `day`, as `acquired` says: their course is drawn now.
    fn infect(
        &mut self,
        ctx: &Ctx,
        person: PermanentId,
        disease: u16,
        day: i64,
        acquired: Acquired,
    ) -> Option<usize> {
        let def = ctx.catalog.diseases.get(usize::from(disease))?;
        let p = self.person(person)?;
        let (age, given) = (p.age_years(ctx.now), p.given.clone());
        let settlement = self.household(p.household).and_then(|x| x.settlement);
        let mut rng = sickness_rng(ctx.seed, person, day, disease, SickDraw::Course);
        let c = course(def, age, day, &mut rng);
        let i = self.sickness.add(person, disease, day, acquired, c);
        // Part of the outbreak open where they live, or the first case of a new one.
        let (_, began) = self.sickness.join_outbreak(i, settlement);
        if began {
            let place = settlement.map_or_else(
                || "the people abroad".to_owned(),
                |s| self.settlement_name(ctx, s),
            );
            self.chronicle_push(
                ctx.now,
                ChronicleKind::Outbreak,
                vec![person],
                settlement,
                None,
                f64::from(crate::OutbreakStep::Began as u8),
                format!("{} came to {place}: {given} took it.", def.name),
            );
        }
        Some(i)
    }

    /// The observer brings disease `disease` (by index in the catalog) to `person`, as if they
    /// took it elsewhere (M6a slice AZ, ADR-0021 §8): their infection's course is drawn as any
    /// other's and its record names this intervention. Whether it goes further is the disease's
    /// and people's. Refused while an infection of it runs in them or one they lived through
    /// still protects them.
    pub fn plague(
        &mut self,
        ctx: &mut Ctx,
        person: PermanentId,
        disease: usize,
    ) -> Result<super::influence::Reached, String> {
        let p = self
            .person(person)
            .ok_or_else(|| format!("{person} is not alive here"))?;
        let settlement = self.household(p.household).and_then(|x| x.settlement);
        let given = p.given.clone();
        let def = ctx
            .catalog
            .diseases
            .get(disease)
            .ok_or_else(|| format!("no disease numbered {disease} in the content"))?;
        let name = def.name.to_lowercase();
        let d = u16::try_from(disease).map_err(|_| format!("no disease numbered {disease}"))?;
        let (now, day) = (ctx.now, ctx.now.day_index());
        if self.sickness.protected(person, d, day) {
            return Err(format!(
                "{given} cannot take {name} now: it runs in them, or one they lived through still \
                 protects them"
            ));
        }
        let id = self
            .influences
            .add(now, InfluenceKind::Plague, person, disease as u32);
        self.infect(ctx, person, d, day, Acquired::Observer { influence: id });
        self.chronicle_push(
            now,
            ChronicleKind::Influence,
            vec![person],
            settlement,
            None,
            f64::from(InfluenceKind::Plague.code()),
            format!(": {name}, as if they took it elsewhere."),
        );
        Ok(super::influence::Reached {
            id,
            repeat: false,
            holds: false,
            fit: 0.0,
            chance: 0.0,
        })
    }
}
