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
use crate::contagion::Node;
use crate::decide::{CallFacts, TendFacts};
use crate::influence::InfluenceKind;
use crate::params::{COMPLICATIONS, Complication, DiseaseRoute};
use crate::sickness::{Acquired, CareCall, Outcome, SickDraw, course_with, sickness_rng};
use crate::{Cause, Repro};

/// An outbreak is over only once less than this much of its disease is left with its
/// settlement's households, doses: drunk whole, less than a dose is a hazard under one (a design
/// prior).
pub const OUTBREAK_LEFT_DOSES: f64 = 1.0;

/// The food a household offers someone not of its kin for each day of tending its ill, in days of
/// its own food (M10c slice DZ, second part; a tuning value: research 05-05 gives a healer's fee
/// in money, not in a household's food).
pub const CALL_FEE_FOOD_DAYS: f64 = 0.5;

/// Minutes of walking a unit of repute for tending is worth to a household choosing whom to call
/// (M10c slice DZ, third part; a tuning value: a carer whose tended lived twice is worth a
/// twenty-minute longer walk than one nobody has seen tend).
pub const REPUTE_WALK_MIN: f64 = 10.0;

/// Days before a birth is due within which a household with nobody to attend it calls an
/// attendant (M10c slice EA, second part; a tuning value: a due date is known to within weeks).
pub const BIRTH_CALL_DAYS: i64 = 7;

impl Population {
    /// The day's turn for infections (see the module documentation).
    pub(super) fn sickness_day(&mut self, ctx: &mut Ctx, day: i64) {
        self.lapse_calls(ctx, day);
        // Each law clearing middens counts the day's breaches (M10c slice EE, third part).
        self.middens_day(ctx, day);
        // And each law carrying refuse to a tip (M11f, second part; ADR-0038 §3).
        self.tips_day(ctx, day);
        // Each sewer carries its connections' day (ADR-0041 §2).
        self.sewers_day(ctx, day);
        // Yesterday's company at the hearth, kept while a disease passing by company ran, and
        // yesterday's carers at a sickhouse (M10c slice EE, fourth part).
        let met = std::mem::take(&mut self.met);
        let staffed = std::mem::take(&mut self.staffed);
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
            // A severe day nobody tended them is measured against their polity: what a sickhouse
            // answers (M10c slice EE, fourth part).
            if e.course.severe && e.ill_on(lived) && e.care.day != lived {
                self.measure_untended(e.household, lived);
            }
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
                    self.note_tended(ctx, &e, false);
                    if let Some((household, settlement)) = home {
                        let claim = self.sickness_claim(household, settlement, lived, None);
                        for k in kin {
                            self.word.hear(k, claim, day, None, Some(k));
                        }
                        // Those left reconsider where to live (ADR-0021 §7, ADR-0018 §5).
                        if self.household(household).is_some() {
                            self.review_due.insert(household);
                        }
                    }
                    continue;
                }
            }
            if e.run_by(day) {
                self.sickness.end(i, day, Outcome::Recovered);
                self.note_tended(ctx, &e, true);
            }
        }
        // The sickhouses' admissions and discharges (M10c slice EE, fourth part).
        self.admit_day(ctx, day);
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
            // One lying in a sickhouse sheds there, not at home (M10c slice EE, fourth part).
            if household_route
                && e.shedding(day)
                && self.sickness.admitted_to(e.person).is_none()
                && let Some(p) = self.person(e.person)
            {
                *shedding.entry((p.household, e.disease)).or_default() += 1;
            }
        }
        // Each person's hazards of each disease, by route.
        let mut exposed: BTreeMap<(PermanentId, u16), Vec<(f64, Acquired)>> = BTreeMap::new();
        // Company at the hearth yesterday with those who shed then (M10c slice ED): each hour
        // with one puts the disease's hazard on the other.
        if !met.is_empty() {
            let mut shed: BTreeMap<PermanentId, Vec<u16>> = BTreeMap::new();
            for &i in self.sickness.running() {
                let Some(e) = self.sickness.get(i) else {
                    continue;
                };
                if e.shedding(day - 1)
                    && ctx
                        .catalog
                        .diseases
                        .get(usize::from(e.disease))
                        .is_some_and(|d| d.passes_by(DiseaseRoute::Company))
                {
                    shed.entry(e.person).or_default().push(e.disease);
                }
            }
            for (a, b, hours) in met {
                for (from, to) in [(a, b), (b, a)] {
                    for &disease in shed.get(&from).into_iter().flatten() {
                        let Some(def) = ctx.catalog.diseases.get(usize::from(disease)) else {
                            continue;
                        };
                        exposed.entry((to, disease)).or_default().push((
                            def.company_hazard.max(0.0) * f64::from(hours),
                            Acquired::Company { person: from },
                        ));
                    }
                }
            }
        }
        for ((household, disease), shedders) in shedding {
            let Some(def) = ctx.catalog.diseases.get(usize::from(disease)) else {
                continue;
            };
            let Some(x) = self.household(household) else {
                continue;
            };
            let hazard = def.household_hazard.max(0.0) * f64::from(shedders);
            for &m in x
                .members
                .iter()
                .filter(|&&m| self.sickness.admitted_to(m).is_none())
            {
                exposed
                    .entry((m, disease))
                    .or_default()
                    .push((hazard, Acquired::Household { household }));
            }
        }
        // At each sickhouse, from those lying there who shed it (M10c slice EE, fourth part).
        for (m, disease, hazard, from) in self.sickhouse_hazards(ctx, day, &staffed) {
            exposed
                .entry((m, disease))
                .or_default()
                .push((hazard, Acquired::Sickhouse { person: from }));
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
            let mut total: f64 = hazards.iter().map(|(h, _)| h.max(0.0)).sum();
            // Vaccination turns away its share (M10c slice ED, third part).
            if !self.sickness.vaccinated.is_empty()
                && let Some(v) = ctx
                    .catalog
                    .diseases
                    .get(usize::from(disease))
                    .and_then(|d| d.vaccination)
            {
                total *= self.sickness.unvaccinated_share(m, disease, day, &v);
            }
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
        // Each register of the sick kept takes the day's entries (M10c slice EE, second part).
        self.register_day(ctx, day);
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
    pub(super) fn sickness_claim(
        &mut self,
        household: PermanentId,
        settlement: PermanentId,
        day: i64,
        spell: Option<i64>,
    ) -> u32 {
        match self.word.sickness(household) {
            Some(c) if spell.is_none_or(|s| day - c.day <= s) => c.id,
            _ => self.word.make(crate::word::Claim {
                fire: None,
                event: None,
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

    /// What is left of `disease` with settlement `settlement`'s households (anyone's for `None`),
    /// doses: in their heaps and stored water, in their wells and their polity's and its tips, and on its way
    /// through the ground to any of those. What reaches a river is carried off by the next day.
    fn disease_left(&self, ctx: &Ctx, disease: u16, settlement: Option<PermanentId>) -> f64 {
        let ours = |h: PermanentId| {
            settlement.is_none_or(|s| self.household(h).is_some_and(|x| x.settlement == Some(s)))
        };
        let node_ours = |n: Node| match n {
            Node::Midden(h) | Node::Store(h) | Node::Refuse(h) => ours(h),
            // A tip, by the settlement of the polity that keeps it (ADR-0038 §3).
            Node::Tip(t) => self
                .tip_settlement(t)
                .is_some_and(|s| settlement.is_none_or(|x| x == s)),
            // A sewer's settling tank, likewise (ADR-0041 §4).
            Node::Sewer(t) => self
                .sewer_settlement(t)
                .is_some_and(|s| settlement.is_none_or(|x| x == s)),
            Node::Well(w) => ctx.land.wells.get(w).is_some_and(|w| {
                ours(w.household)
                    || w.polity.is_some_and(|p| {
                        self.polities
                            .iter()
                            .any(|q| q.id == p && settlement.is_none_or(|s| q.settlement == s))
                    })
            }),
            _ => false,
        };
        let c = &self.contagion;
        let heaps_and_wells: f64 = c
            .loads
            .iter()
            .filter(|&(&(n, d), _)| d == disease && node_ours(n))
            .map(|(_, &a)| a)
            .sum();
        let stored: f64 = c
            .stores
            .iter()
            .filter(|&(&(h, d), _)| d == disease && ours(h))
            .flat_map(|(_, from)| from.values())
            .sum();
        let underway: f64 = c
            .transit
            .iter()
            .filter(|t| t.disease == disease && node_ours(t.to))
            .map(|t| t.amount)
            .sum();
        heaps_and_wells + stored + underway
    }

    /// Ends each open outbreak none of whose cases has run for as long as a new infection of its
    /// disease can take to show (its incubation's mean and three spreads, at least a day), and of
    /// whose disease less than [`OUTBREAK_LEFT_DOSES`] is left with its settlement's households,
    /// and tells it with its counts. While its disease lies in their heaps, wells or water, a new
    /// case of it there is the same outbreak (M6a slice BB: one the observer began was otherwise
    /// closed with its wells still fouled, and the cases after counted as nobody's doing).
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
            if runs
                || day - last < quiet
                || self.disease_left(ctx, o.disease, o.settlement) >= OUTBREAK_LEFT_DOSES
            {
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
    /// three), or failing any, to those of the home of close kin elsewhere whose sickness they
    /// heard of within the word's spell (M10c slice DZ; ADR-0035 §3), the walk there counted from
    /// `field_key`'s travel field: the kin home owed most, the nearer of two alike. See
    /// [`TendFacts`].
    pub(super) fn tend_facts(
        &self,
        ctx: &Ctx,
        who: PermanentId,
        x: &Household,
        field_key: PermanentId,
    ) -> Option<TendFacts> {
        if self.sickness.quiet() {
            return None;
        }
        let day = ctx.now.day_index();
        let owed_by = |members: &[PermanentId]| self.care_owed(ctx, members, Some(who));
        let owed = owed_by(&x.members);
        if owed > 1e-9 {
            return Some(TendFacts {
                owed,
                household: None,
                at: x.home,
                walk_min: 0.0,
                sickhouse: None,
            });
        }
        // Close kin elsewhere, whose sickness they heard of lately, and the households that
        // called theirs to tend them (the second part).
        let spell = i64::from(ctx.params.word.sickness_spell_days);
        let heard = self.word.heard_by(who);
        let reach = &self.homes.get(&field_key)?.reach;
        let mut homes: Vec<PermanentId> = self
            .close_kin(who)
            .into_iter()
            .filter_map(|k| self.person(k).map(|q| q.household))
            .filter(|&h| {
                h != x.id
                    && self.word.sickness(h).is_some_and(|c| {
                        day - c.day <= spell && heard.iter().any(|h| h.claim == c.id)
                    })
            })
            .collect();
        homes.extend(
            self.sickness
                .calls
                .iter()
                .filter(|c| c.carer == x.id && c.caller != x.id)
                .map(|c| c.caller),
        );
        homes.sort_unstable();
        homes.dedup();
        let mut best: Option<TendFacts> = None;
        for h in homes {
            let Some(y) = self.household(h) else {
                continue;
            };
            let owed = owed_by(&y.members);
            if owed <= 1e-9 {
                continue;
            }
            let Some(secs) = reach.seconds_to(cell_of(ctx.map, y.home)) else {
                continue;
            };
            let walk_min = f64::from(secs) / 60.0;
            if best.is_none_or(|b| {
                owed > b.owed + 1e-9 || ((owed - b.owed).abs() <= 1e-9 && walk_min < b.walk_min)
            }) {
                best = Some(TendFacts {
                    owed,
                    household: Some(y.id),
                    at: y.home,
                    walk_min,
                    sickhouse: None,
                });
            }
        }
        // Else a household that called theirs to inoculate its people (M10c slice ED), or the
        // sickhouse (M10c slice EE, fourth part).
        best.or_else(|| self.inoculation_tend(ctx, who, x, field_key))
            .or_else(|| self.sickhouse_tend(ctx, who, x, field_key))
    }

    /// Person `who` sat `minutes` with the members of household `household` ill at home (their
    /// own, or close kin's, M10c slice DZ), at activity `def` (step three): the hours are shared
    /// among them, each episode recording who came and the least relative risk of dying a
    /// treatment the carer knows brings. The carer practises each treatment they know, and one
    /// who does not know it learns it from a member of their own household who does (ADR-0008
    /// §4). One come from another home is exposed there as its members are, by a draw of their
    /// own for the day.
    pub(super) fn tended(
        &mut self,
        ctx: &mut Ctx,
        who: PermanentId,
        household: PermanentId,
        def: u16,
        minutes: u32,
    ) {
        // Come to inoculate (M10c slice ED, second part).
        if self.inoculate_at(ctx, who, household, def, minutes) {
            return;
        }
        let day = ctx.now.day_index();
        let (Some(x), Some(carer)) = (self.household(household), self.person(who)) else {
            return;
        };
        let visiting = carer.household != household;
        // Those lying in a sickhouse are tended there (M10c slice EE, fourth part).
        let ill: Vec<usize> = x
            .members
            .iter()
            .filter(|&&m| m != who && self.sickness.admitted_to(m).is_none())
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
        // Their hands were on the ill today (M10c slice EA).
        self.sickness.tended.insert(who, day);
        treatments.sort_unstable();
        treatments.dedup();
        for t in treatments {
            self.practise(ctx, who, t, def, Target::Home, hours);
        }
        if visiting {
            self.visit_exposure(ctx, who, household, day);
            let carer_home = self.person(who).map(|p| p.household);
            if let Some(home) = carer_home {
                self.pay_care_fee(ctx, household, home, day, false);
            }
        }
    }

    /// The care owed today to those of `members` ill, but `except` (step three): the share of a
    /// day's care still owed each, the severely ill whole and the others half.
    fn care_owed(&self, ctx: &Ctx, members: &[PermanentId], except: Option<PermanentId>) -> f64 {
        // Those lying in a sickhouse are owed care there, not at home (M10c slice EE, fourth part).
        let home: Vec<PermanentId> = members
            .iter()
            .copied()
            .filter(|&m| Some(m) != except && self.sickness.admitted_to(m).is_none())
            .collect();
        self.care_owed_at(ctx, &home)
    }

    /// The care owed today to those of `members` ill: the share of a day's care still owed each,
    /// the severely ill whole and the others half.
    pub(super) fn care_owed_at(&self, ctx: &Ctx, members: &[PermanentId]) -> f64 {
        let day = ctx.now.day_index();
        let mut owed = 0.0;
        for &m in members {
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
        owed
    }

    /// An episode `e` ended, its bearer having `lived` or not (M10c slice DZ, third part): where
    /// the first to tend them was of another household, the living members of the bearer's
    /// household see what came of that carer's care, a tie act each ([`Act::TendedLived`] or
    /// [`Act::TendedDied`]): the carer's repute, as their clients saw it.
    fn note_tended(&mut self, ctx: &Ctx, e: &crate::sickness::Episode, lived: bool) {
        let Some((_, carer)) = e.care.first else {
            return;
        };
        let Some(home) = e
            .household
            .or_else(|| self.person(e.person).map(|p| p.household))
        else {
            return;
        };
        if self.person(carer).is_none_or(|c| c.household == home) {
            return;
        }
        let Some(members) = self.household(home).map(|x| x.members.clone()) else {
            return;
        };
        let act = if lived {
            crate::ties::Act::TendedLived
        } else {
            crate::ties::Act::TendedDied
        };
        // A death in the care of one paid, not kin, who knew no treatment of it, is remembered a
        // year (M10c slice EC, third part).
        if !lived {
            self.note_unknowing(ctx, e, carer, home, &members);
        }
        for m in members.into_iter().filter(|&m| m != e.person) {
            self.note_tie(ctx, m, carer, act, 1.0, 0.0);
        }
    }

    /// The repute for tending the sick that the members of `members` hold of anyone of
    /// `household` on `day` (M10c slice DZ, third part): the most net good evidence in craft any
    /// of them holds of any of its people, beyond the prior.
    fn care_repute(&self, ctx: &Ctx, members: &[PermanentId], household: &Household) -> f64 {
        let (day, params) = (ctx.now.day_index(), &ctx.params.ties);
        members
            .iter()
            .flat_map(|&m| self.ties.of(m))
            .filter(|t| household.members.contains(&t.to))
            .map(|t| t.at(day, params).esteem(crate::ties::Domain::Craft, params))
            .fold(0.0, f64::max)
    }

    /// Who of `members` could tend today: not ill, and of the tending activity's age.
    fn able_carers(&self, ctx: &Ctx, members: &[PermanentId]) -> usize {
        let day = ctx.now.day_index();
        let Some(min_age) = ctx
            .catalog
            .activities
            .iter()
            .find(|a| a.behavior == Behavior::Tend)
            .map(|a| a.min_age_years)
        else {
            return 0;
        };
        members
            .iter()
            .filter(|&&m| {
                !self.sickness.ill(m, day)
                    && self
                        .person(m)
                        .is_some_and(|q| q.age_years(ctx.now) >= min_age)
            })
            .count()
    }

    /// Whom person `who` of household `x` would call (M10c slice DZ, second part; ADR-0035 §2):
    /// someone to tend its ill, else someone to attend a birth due (slice EA, second part).
    pub(super) fn call_facts(
        &self,
        ctx: &Ctx,
        who: PermanentId,
        x: &Household,
        field_key: PermanentId,
    ) -> Option<CallFacts> {
        self.care_call_facts(ctx, who, x, field_key)
            .or_else(|| self.birth_call_facts(ctx, who, x, field_key))
            .or_else(|| self.inoculation_call_facts(ctx, who, x, field_key))
    }

    /// The care owed household `x`'s ill today beyond what its well members of tending age can
    /// give, each at most one.
    fn care_short(&self, ctx: &Ctx, x: &Household) -> f64 {
        if self.sickness.quiet() {
            return 0.0;
        }
        self.care_owed(ctx, &x.members, None) - self.able_carers(ctx, &x.members) as f64
    }

    /// Whom person `who` of household `x` would call to tend its ill (M10c slice DZ, second part;
    /// ADR-0035 §2): only where the care owed at home is more than its well members of tending
    /// age can give, each at most one, and it has no call standing. Kin of any of its members
    /// first, asking no fee, the nearest; then the households of those `who` or any of its
    /// household has ties with, for [`CALL_FEE_FOOD_DAYS`] of the household's food a day, which
    /// it must hold, the nearest less [`REPUTE_WALK_MIN`] for each unit of repute for tending its
    /// members hold of someone there (the third part); of each, a household with someone able to
    /// tend, the walk counted from `field_key`'s travel field.
    fn care_call_facts(
        &self,
        ctx: &Ctx,
        who: PermanentId,
        x: &Household,
        field_key: PermanentId,
    ) -> Option<CallFacts> {
        if self.sickness.call_of(x.id).is_some() {
            return None;
        }
        let short = self.care_short(ctx, x);
        if short <= 1e-9 {
            return None;
        }
        let mut kin: Vec<PermanentId> = x
            .members
            .iter()
            .flat_map(|&m| self.close_kin(m))
            .filter_map(|k| self.person(k).map(|q| q.household))
            .collect();
        kin.sort_unstable();
        kin.dedup();
        let mut tied = self.tie_households(who, x);
        // Under a licence, only those who know its technique are paid to tend (M10c slice EC).
        tied.retain(|&h| self.household(h).is_none_or(|y| self.licensed(ctx, x, y)));
        let able = |y: &Household| self.able_carers(ctx, &y.members) > 0;
        self.choose_carer(ctx, x, field_key, &kin, &tied, &able, short, false)
    }

    /// The households of those person `who` or any of household `x` has ties with.
    pub(super) fn tie_households(&self, who: PermanentId, x: &Household) -> Vec<PermanentId> {
        let mut tied: Vec<PermanentId> = std::iter::once(who)
            .chain(x.members.iter().copied())
            .flat_map(|m| self.ties.of(m))
            .filter_map(|t| self.person(t.to).map(|q| q.household))
            .collect();
        tied.sort_unstable();
        tied.dedup();
        tied
    }

    /// Of the households `kin` (asking no fee) and then `tied` (asking [`Population::call_fee`],
    /// which household `x` must hold), other than `x` and those `able` refuses, the one to call:
    /// kin first, the nearest; then the nearest less [`REPUTE_WALK_MIN`] for each unit of repute
    /// for tending `x`'s members hold of someone there. The walk is counted from `field_key`'s
    /// travel field; `short` and `birth` are what the call is for.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn choose_carer(
        &self,
        ctx: &Ctx,
        x: &Household,
        field_key: PermanentId,
        kin: &[PermanentId],
        tied: &[PermanentId],
        able: &dyn Fn(&Household) -> bool,
        short: f64,
        birth: bool,
    ) -> Option<CallFacts> {
        let reach = &self.homes.get(&field_key)?.reach;
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let pays = stock_kcal(&stores_now(x, now, params, goods), goods) >= self.call_fee(ctx, x);
        let tied = tied
            .iter()
            .filter(|h| pays && kin.binary_search(h).is_err());
        let mut best: Option<(bool, f64, CallFacts)> = None;
        for (asks_fee, &h) in kin
            .iter()
            .map(|h| (false, h))
            .chain(tied.map(|h| (true, h)))
        {
            let Some(y) = self.household(h).filter(|y| y.id != x.id) else {
                continue;
            };
            if !able(y) {
                continue;
            }
            let Some(secs) = reach.seconds_to(cell_of(ctx.map, y.home)) else {
                continue;
            };
            let walk_min = f64::from(secs) / 60.0;
            // Those not kin weighed by what the household saw of their care against the walk.
            let key = if asks_fee {
                walk_min - REPUTE_WALK_MIN * self.care_repute(ctx, &x.members, y)
            } else {
                walk_min
            };
            if best.is_none_or(|(f, k, _)| (asks_fee, key) < (f, k)) {
                best = Some((
                    asks_fee,
                    key,
                    CallFacts {
                        carer: y.id,
                        at: y.home,
                        walk_min,
                        short,
                        birth,
                    },
                ));
            }
        }
        best.map(|(_, _, c)| c)
    }

    /// Whether a woman of household `x` is due to give birth within [`BIRTH_CALL_DAYS`] and
    /// nobody of her household or her close kin of her settlement knows a technique of birth
    /// care to attend her (M10c slice EA, second part).
    fn birth_due_unattended(&self, ctx: &Ctx, x: &Household) -> bool {
        let soon = ctx
            .now
            .plus_minutes(BIRTH_CALL_DAYS * civ_core::time::MINUTES_PER_DAY);
        let knows = |q: PermanentId| {
            self.person(q).is_some_and(|p| {
                ctx.catalog
                    .techniques
                    .iter()
                    .enumerate()
                    .any(|(t, d)| p.knows(t) && d.birth.iter().any(|&e| e > 0.0))
            })
        };
        x.members.iter().any(|&m| {
            self.person(m).is_some_and(
                |p| matches!(p.repro, Repro::Pregnant { due, loss: false, .. } if due <= soon),
            ) && !x.members.iter().any(|&q| q != m && knows(q))
                && !self.close_kin(m).into_iter().any(|k| {
                    knows(k)
                        && self
                            .person(k)
                            .and_then(|q| self.household(q.household))
                            .is_some_and(|y| y.settlement == x.settlement)
                })
        })
    }

    /// Whom person `who` of household `x` would call to attend a birth due (M10c slice EA,
    /// second part; ADR-0035 §2-§3): where a woman of it is due within [`BIRTH_CALL_DAYS`] with
    /// nobody of her own or her close kin nearby who knows birth care, and it has no such call
    /// standing, the household of someone the household knows attended a birth, in its own home
    /// or the home of someone its members have ties with: kin asking no fee, others the fee for
    /// a day of care, paid at the birth.
    fn birth_call_facts(
        &self,
        ctx: &Ctx,
        who: PermanentId,
        x: &Household,
        field_key: PermanentId,
    ) -> Option<CallFacts> {
        if self.sickness.attended.is_empty()
            || self.sickness.birth_call_of(x.id).is_some()
            || !self.birth_due_unattended(ctx, x)
        {
            return None;
        }
        let tied = self.tie_households(who, x);
        let day = ctx.now.day_index();
        // Attendants it has heard of, living and well, by their households now.
        let mut known: Vec<PermanentId> = self
            .sickness
            .attended
            .iter()
            .filter(|a| a.household == x.id || tied.binary_search(&a.household).is_ok())
            .filter(|a| !self.sickness.ill(a.attendant, day))
            .filter_map(|a| self.person(a.attendant).map(|q| q.household))
            .collect();
        known.sort_unstable();
        known.dedup();
        let mut kin: Vec<PermanentId> = x
            .members
            .iter()
            .flat_map(|&m| self.close_kin(m))
            .filter_map(|k| self.person(k).map(|q| q.household))
            .filter(|h| known.binary_search(h).is_ok())
            .collect();
        kin.sort_unstable();
        kin.dedup();
        let any = |_: &Household| true;
        self.choose_carer(ctx, x, field_key, &kin, &known, &any, 1.0, true)
    }

    /// The food household `x` offers for a day of care by someone not of its kin, kcal.
    pub(super) fn call_fee(&self, ctx: &Ctx, x: &Household) -> f64 {
        CALL_FEE_FOOD_DAYS * x.members.len() as f64 * ctx.params.household.daily_kcal_per_person
    }

    /// Someone of household `caller` left a call at household `carer`'s door (M10c slice DZ,
    /// second part): it stands from today, asking no fee where someone of the caller is close
    /// kin of someone of the carer, else [`Population::call_fee`]. A household has one call
    /// standing at most.
    pub(super) fn call_carer(&mut self, ctx: &Ctx, caller: PermanentId, carer: PermanentId) {
        self.refresh_kin();
        let (Some(x), Some(y)) = (self.household(caller), self.household(carer)) else {
            return;
        };
        // For its ill where they are short of care, else for a birth due (M10c slice EA).
        let birth = if self.sickness.call_of(caller).is_none() && self.care_short(ctx, x) > 1e-9 {
            false
        } else if self.sickness.birth_call_of(caller).is_none() && self.birth_due_unattended(ctx, x)
        {
            true
        } else {
            // Else to inoculate its people (M10c slice ED, second part).
            self.call_inoculator(ctx, caller, carer);
            return;
        };
        let kin = x
            .members
            .iter()
            .any(|&m| self.close_kin(m).iter().any(|k| y.members.contains(k)));
        let fee_kcal = if kin {
            0.0
        } else {
            self.call_fee(ctx, x) as f32
        };
        self.sickness.calls.push(CareCall {
            caller,
            carer,
            fee_kcal,
            day: ctx.now.day_index(),
            paid: -1,
            birth,
        });
    }

    /// Household `caller` pays household `carer` the day's fee of its call for care on `day`
    /// (M10c slice DZ, second part), once a day, in food through the [`Channel::Fee`] channel,
    /// the least keeping first. One that cannot pay it all pays what it holds, and its call
    /// lapses.
    fn pay_care_fee(
        &mut self,
        ctx: &Ctx,
        caller: PermanentId,
        carer: PermanentId,
        day: i64,
        birth: bool,
    ) {
        let Some(i) = self.sickness.calls.iter().position(|c| {
            c.caller == caller && c.carer == carer && c.paid < day && c.birth == birth
        }) else {
            return;
        };
        self.sickness.calls[i].paid = day;
        let fee = f64::from(self.sickness.calls[i].fee_kcal);
        if fee <= 0.0 {
            return;
        }
        let want = fee - self.pay_food(ctx, caller, carer, fee);
        if want > 1e-6 {
            self.sickness
                .calls
                .retain(|c| c.caller != caller || c.birth != birth);
        }
    }

    /// Household `from` pays household `to` `kcal` of food through the [`Channel::Fee`] channel,
    /// the least keeping first, as far as it holds it (M10c slices DZ and EB); returns the kcal
    /// paid.
    pub(super) fn pay_food(
        &mut self,
        ctx: &Ctx,
        from: PermanentId,
        to: PermanentId,
        kcal: f64,
    ) -> f64 {
        self.pay_food_by(ctx, from, to, kcal, Channel::Fee)
    }

    /// [`Population::pay_food`], through `channel` (M11a, fifth part: the wire's meter).
    pub(super) fn pay_food_by(
        &mut self,
        ctx: &Ctx,
        from: PermanentId,
        to: PermanentId,
        kcal: f64,
        channel: Channel,
    ) -> f64 {
        let (now, params, goods) = (ctx.now, ctx.params, &ctx.catalog.goods);
        let Some(x) = self.household(from) else {
            return 0.0;
        };
        let stores = stores_now(x, now, params, goods);
        let mut order: Vec<usize> = (0..goods.len())
            .filter(|&g| {
                goods[g].purpose == GoodUse::Food
                    && goods[g].kcal_per_kg > 0.0
                    && !goods[g].kept_back()
            })
            .collect();
        let keeps = |g: usize| match goods[g].half_life_days {
            h if h > 0.0 => h,
            _ => f64::INFINITY,
        };
        order.sort_by(|&a, &b| keeps(a).total_cmp(&keeps(b)).then(a.cmp(&b)));
        let (mut want, mut legs) = (kcal, Vec::new());
        for g in order {
            if want <= 1e-9 {
                break;
            }
            let kg =
                (want / goods[g].kcal_per_kg).min(stores.get(g).copied().unwrap_or(0.0).max(0.0));
            if kg <= 0.0 {
                continue;
            }
            legs.push(Leg {
                from,
                to,
                good: g,
                amount: kg,
            });
            want -= kg * goods[g].kcal_per_kg;
        }
        if legs.is_empty() || !self.transfer(now, params, goods, &legs, channel) {
            return 0.0;
        }
        self.changed(from);
        self.changed(to);
        kcal - want.max(0.0)
    }

    /// Who would attend mother `mother`'s birth on `day` (M10c slice EA; ADR-0035 §3): of those
    /// of her household and her close kin living in her settlement, well and of tending age, one
    /// who knows a technique of birth care, the one whose techniques prevent the most of the
    /// deaths, by their shares (the lower id of two alike); with the share of each
    /// complication's deaths they prevent, and whether they come fresh from the ill without a
    /// technique against infection. `None` where nobody there knows any.
    pub(super) fn birth_attendant(
        &mut self,
        ctx: &Ctx,
        mother: PermanentId,
        day: i64,
    ) -> Option<(PermanentId, [f64; COMPLICATIONS], bool)> {
        if !ctx
            .catalog
            .techniques
            .iter()
            .any(|t| t.birth.iter().any(|&e| e > 0.0))
        {
            return None;
        }
        let m = self.person(mother)?;
        let home = m.household;
        let settlement = self.household(home)?.settlement;
        self.refresh_kin();
        let mut near: Vec<PermanentId> = self.household(home)?.members.clone();
        // And the household it called to attend (the second part).
        let called: Vec<PermanentId> = self
            .sickness
            .calls
            .iter()
            .filter(|c| c.birth && c.caller == home)
            .filter_map(|c| self.household(c.carer))
            .flat_map(|y| y.members.iter().copied())
            .collect();
        near.extend(called);
        near.extend(self.close_kin(mother).into_iter().filter(|&k| {
            self.person(k)
                .and_then(|q| self.household(q.household))
                .is_some_and(|x| x.settlement == settlement)
        }));
        near.sort_unstable();
        near.dedup();
        let min_age = ctx
            .catalog
            .activities
            .iter()
            .find(|a| a.behavior == Behavior::Tend)
            .map_or(10.0, |a| a.min_age_years);
        let birth = &ctx.params.mortality.birth;
        let mut best: Option<(f64, PermanentId, [f64; COMPLICATIONS])> = None;
        for q in near.into_iter().filter(|&q| q != mother) {
            let Some(p) = self.person(q) else {
                continue;
            };
            if p.age_years(ctx.now) < min_age || self.sickness.ill(q, day) {
                continue;
            }
            let mut e = [0.0; COMPLICATIONS];
            for (t, def) in ctx.catalog.techniques.iter().enumerate() {
                if p.knows(t) {
                    for (x, &b) in e.iter_mut().zip(&def.birth) {
                        *x = f64::max(*x, b);
                    }
                }
            }
            if e.iter().all(|&x| x <= 0.0) {
                continue;
            }
            let prevents: f64 = (0..COMPLICATIONS).map(|c| birth.shares[c] * e[c]).sum();
            if best.is_none_or(|(b, _, _)| prevents > b) {
                best = Some((prevents, q, e));
            }
        }
        let (_, who, e) = best?;
        let harmed = e[Complication::Infection as usize] <= 0.0
            && self
                .sickness
                .tended
                .get(&who)
                .is_some_and(|&d| day - d <= birth.fever_harm_days);
        Some((who, e, harmed))
    }

    /// Person `attendant` attended a birth in household `home` on `day`, the mother having `lived`
    /// or not (M10c slice EA, second part): it is kept as their record; the household's call for
    /// an attendant is answered, its fee paid where the attendant came on it; and where the
    /// attendant is of another household, its members see what came of their care, as of a
    /// carer's ([`Population::note_tended`]).
    pub(super) fn birth_attended(
        &mut self,
        ctx: &Ctx,
        attendant: PermanentId,
        home: PermanentId,
        day: i64,
        lived: bool,
    ) {
        self.sickness.attended.push(crate::sickness::BirthAttended {
            attendant,
            household: home,
            day,
            lived,
        });
        let Some(theirs) = self.person(attendant).map(|p| p.household) else {
            return;
        };
        if theirs == home {
            self.sickness
                .calls
                .retain(|c| !(c.birth && c.caller == home));
            return;
        }
        self.pay_care_fee(ctx, home, theirs, day, true);
        self.sickness
            .calls
            .retain(|c| !(c.birth && c.caller == home));
        let act = if lived {
            crate::ties::Act::TendedLived
        } else {
            crate::ties::Act::TendedDied
        };
        let members = self
            .household(home)
            .map(|x| x.members.clone())
            .unwrap_or_default();
        for m in members {
            self.note_tie(ctx, m, attendant, act, 1.0, 0.0);
        }
    }

    /// Calls for carers lapse at the day's turn (M10c slice DZ, second part): once nobody of the
    /// household that called is ill (for a birth, once nobody of it is with child), once it or
    /// the household it called is no more, or once the word's spell has passed since it called
    /// (for a birth, twice the days it calls before one is due). Who tended the ill is forgotten
    /// once it no longer matters to a birth (M10c slice EA).
    fn lapse_calls(&mut self, ctx: &Ctx, day: i64) {
        let days = ctx.params.mortality.birth.fever_harm_days;
        self.sickness.tended.retain(|_, d| day - *d <= days);
        self.lapse_inoculation_calls(ctx, day);
        if self.sickness.calls.is_empty() {
            return;
        }
        let spell = i64::from(ctx.params.word.sickness_spell_days);
        let calls = std::mem::take(&mut self.sickness.calls);
        let kept = calls
            .into_iter()
            .filter(|c| {
                let wanted = |m: PermanentId| {
                    if c.birth {
                        self.person(m)
                            .is_some_and(|p| matches!(p.repro, Repro::Pregnant { .. }))
                    } else {
                        self.sickness.ill(m, day)
                    }
                };
                let lasts = if c.birth { 2 * BIRTH_CALL_DAYS } else { spell };
                day - c.day <= lasts
                    && self.household(c.carer).is_some()
                    && self
                        .household(c.caller)
                        .is_some_and(|x| x.members.iter().any(|&m| wanted(m)))
            })
            .collect();
        self.sickness.calls = kept;
    }

    /// Whether an episode of a disease passing by company runs (M10c slice ED): only then is who
    /// met whom at the hearth kept.
    pub(super) fn company_running(&self, ctx: &Ctx) -> bool {
        self.sickness.running().iter().any(|&i| {
            self.sickness.get(i).is_some_and(|e| {
                ctx.catalog
                    .diseases
                    .get(usize::from(e.disease))
                    .is_some_and(|d| d.passes_by(DiseaseRoute::Company))
            })
        })
    }

    /// Person `who`, come to tend household `household` on `day` (M10c slice DZ), is exposed to
    /// each disease passing by the household route that its members shed today, at a member's
    /// hazard, by one draw a disease a day of their own: a carer coming twice in a day draws
    /// alike. Taken, it is recorded as coming from that household.
    fn visit_exposure(&mut self, ctx: &Ctx, who: PermanentId, household: PermanentId, day: i64) {
        let Some(x) = self.household(household) else {
            return;
        };
        let mut shedders: BTreeMap<u16, u32> = BTreeMap::new();
        for &m in &x.members {
            for e in self.sickness.of(m).filter(|e| e.runs() && e.shedding(day)) {
                *shedders.entry(e.disease).or_default() += 1;
            }
        }
        for (disease, n) in shedders {
            let Some(def) = ctx.catalog.diseases.get(usize::from(disease)) else {
                continue;
            };
            if !def.passes_by(DiseaseRoute::Household) || self.sickness.protected(who, disease, day)
            {
                continue;
            }
            let hazard = def.household_hazard.max(0.0) * f64::from(n);
            let u = sickness_rng(ctx.seed, who, day, disease, SickDraw::Visit).next_f64();
            if u < 1.0 - (-hazard).exp() {
                self.infect(ctx, who, disease, day, Acquired::Household { household });
            }
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
    pub(super) fn infect(
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
        // An inoculated course is milder by the disease's inoculation (M10c slice ED).
        let factor = match (acquired, def.inoculation) {
            (Acquired::Inoculated { .. }, Some(i)) => i.deaths,
            _ => 1.0,
        };
        let c = course_with(def, age, day, &mut rng, factor);
        let home = p.household;
        let i = self
            .sickness
            .add(person, Some(home), disease, day, acquired, c);
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
