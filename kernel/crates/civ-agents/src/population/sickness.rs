//! Sickness among people (M6a slice AZ; ADR-0021 §4, §5, §8; research 05-03 §7.2-§7.6). At each
//! day's turn every infection runs its course: someone severely ill may die of it, by a draw of
//! their own each day; one whose course has run recovers, protected for a while. Then what people
//! shed moves through the ground and water (`contagion`), and whoever is exposed today, by every
//! route (living with those who shed it, and what they drank of their household's water), takes
//! it or not by one draw (1 − e^(−ΣH)), recorded as coming by the route that brought the most of
//! it. The observer's plague tool brings a disease to one person, as if they took it elsewhere.
//! Nothing here is read by any choice: people know only who they see abed.

use super::*;
use crate::Cause;
use crate::influence::InfluenceKind;
use crate::params::DiseaseRoute;
use crate::sickness::{Acquired, Outcome, SickDraw, course, sickness_rng};

impl Population {
    /// The day's turn for infections (see the module documentation).
    pub(super) fn sickness_day(&mut self, ctx: &mut Ctx, day: i64) {
        if self.sickness.quiet() && self.contagion.is_empty() {
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
            if e.course.severe
                && e.ill_on(day)
                && let Some(def) = ctx.catalog.diseases.get(usize::from(e.disease))
            {
                let h = def.severe_death_per_day.clamp(0.0, 1.0);
                let u =
                    sickness_rng(ctx.seed, e.person, day, e.disease, SickDraw::Death).next_f64();
                // A blessing or a curse moves their own chance, as it moves illness's in the life
                // table (ADR-0016 §5); what it turned is noted.
                let luck = self.influences.luck(e.person, day);
                let chance = luck.map_or(h, |l| (h * l.harm()).min(1.0));
                if let Some(l) = luck
                    && (u < chance) != (u < h)
                {
                    self.note_turned(ctx, e.person, l, None);
                }
                if u < chance {
                    let name = format!("disease:{}", def.name.to_lowercase());
                    self.sickness.end(i, day, Outcome::Died);
                    self.die_named(ctx, e.person, Cause::Disease, name);
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
        let age = self.person(person)?.age_years(ctx.now);
        let mut rng = sickness_rng(ctx.seed, person, day, disease, SickDraw::Course);
        let c = course(def, age, day, &mut rng);
        Some(self.sickness.add(person, disease, day, acquired, c))
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
