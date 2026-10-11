//! Infections, as truth (M6a slice AZ; ADR-0021 §5; research 05-03 §1.1, §7.1-§7.6). Each
//! infection is an episode: when it came and how (its acquisition record), when its bearer sheds
//! it, whether and when they are ill and how severely, how it ended, and how long it protects
//! them after. No choice reads an episode: people know only what they see (slice BA).

use std::collections::BTreeMap;

use civ_core::{PermanentId, Rng64};

use crate::demography::normal;
use crate::params::{DiseaseDef, interpolate};

/// The key purpose of sickness draws.
pub const PURPOSE_SICKNESS: u64 = 0x7369_636b_6e65_7331; // "sicknes1"

/// What a sickness draw is for. Codes are part of every world's history: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SickDraw {
    /// Whether someone exposed today is infected.
    Infection = 1,
    /// An infection's course, drawn when it comes.
    Course = 2,
    /// Whether someone severely ill dies today.
    Death = 3,
}

/// The generator for `person`'s draw of `what` about disease `disease` on `day`.
pub fn sickness_rng(
    seed: u64,
    person: PermanentId,
    day: i64,
    disease: u16,
    what: SickDraw,
) -> Rng64 {
    Rng64::from_key(&[
        seed,
        PURPOSE_SICKNESS,
        person.get(),
        day as u64,
        u64::from(disease),
        what as u64,
    ])
}

/// How an infection came: its acquisition record (ADR-0021 §5; 05-03 §7.4). Codes are part of
/// saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Acquired {
    /// Brought by the observer (ADR-0021 §8): the influence record that brought it.
    Observer { influence: u32 },
    /// From living with members who shed it: the household, as the source, never one of them
    /// (05-03 §7.4: a mixture is not one person).
    Household { household: PermanentId },
    /// From water drunk: where most of what they drank of it was drawn (step two), a well or a
    /// river reach.
    Water { source: crate::contagion::Node },
}

impl Acquired {
    /// Its code and number in saves.
    pub fn code(self) -> (u8, u64) {
        match self {
            Acquired::Observer { influence } => (0, u64::from(influence)),
            Acquired::Household { household } => (1, household.get()),
            Acquired::Water { source } => match source {
                crate::contagion::Node::Well(w) => (2, w.get()),
                crate::contagion::Node::Reach(r) => (3, u64::from(r)),
                other => (4 + other.code().0, other.code().1),
            },
        }
    }

    /// The record with code `code` and number `n`.
    pub fn from_code(code: u8, n: u64) -> Option<Acquired> {
        match code {
            0 => Some(Acquired::Observer {
                influence: u32::try_from(n).ok()?,
            }),
            1 => Some(Acquired::Household {
                household: PermanentId::from_raw(n)?,
            }),
            2 => Some(Acquired::Water {
                source: crate::contagion::Node::Well(PermanentId::from_raw(n)?),
            }),
            3 => Some(Acquired::Water {
                source: crate::contagion::Node::Reach(u32::try_from(n).ok()?),
            }),
            c @ 4.. => Some(Acquired::Water {
                source: crate::contagion::Node::from_code(c - 4, n)?,
            }),
        }
    }
}

/// How an episode ended. Codes are part of saves: append only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Its course ran and they lived.
    Recovered = 0,
    /// It killed them.
    Died = 1,
    /// They died of something else, or left the world, while it ran.
    Gone = 2,
}

impl Outcome {
    pub fn code(self) -> u8 {
        self as u8
    }

    pub fn from_code(code: u8) -> Option<Outcome> {
        [Outcome::Recovered, Outcome::Died, Outcome::Gone]
            .get(usize::from(code))
            .copied()
    }
}

/// The days an infection's course runs, drawn when it comes (05-03 §7.3: durations from
/// distributions, in an order the body allows). Days are half-open: `[from, until)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Course {
    /// When its bearer sheds it.
    pub shed_from: i64,
    pub shed_until: i64,
    /// When they are ill: the same day twice when it brings no symptoms.
    pub ill_from: i64,
    pub ill_until: i64,
    /// Whether they are severely ill while ill.
    pub severe: bool,
    /// The first day another infection of it can take, should they live.
    pub immune_until: i64,
}

/// The course of an infection of `def` taken on day `day` by someone `age` years old.
pub fn course(def: &DiseaseDef, age: f64, day: i64, rng: &mut Rng64) -> Course {
    let days = |x: f64| x.round().max(0.0) as i64;
    let onset = day + days(def.incubation.at(normal(rng))).max(1);
    let symptomatic = rng.next_f64() < def.symptomatic;
    let ill_days = days(def.ill.at(normal(rng))).max(1);
    let (ill_from, ill_until) = if symptomatic {
        (onset, onset + ill_days)
    } else {
        (onset, onset)
    };
    let shed_from = (onset - days(def.shed_lead_days)).max(day + 1);
    let shed_len = days(def.shed.at(normal(rng))).max(1);
    let after = days(def.shed_after.at(normal(rng)));
    let shed_until = (shed_from + shed_len).max(if symptomatic { ill_until + after } else { 0 });
    let severe = symptomatic && rng.next_f64() < interpolate(&def.severe_by_age, age);
    let [lo, hi] = def.immunity_years;
    let years = lo + rng.next_f64() * (hi - lo).max(0.0);
    let immune_until = shed_until.max(ill_until) + (years * 365.0).round() as i64;
    Course {
        shed_from,
        shed_until,
        ill_from,
        ill_until,
        severe,
        immune_until,
    }
}

/// The care an infection's bearer was given while ill (M6a slice AZ, step three; ADR-0021 §5;
/// research 05-05 §1.1: care is someone else's labour, recorded where it was given).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Care {
    /// The first day someone tended them, and who: when help came.
    pub first: Option<(i64, PermanentId)>,
    /// The last day they were tended, the hours given them that day, and the least relative risk
    /// of dying any treatment a carer that day knew brings (1 for none).
    pub day: i64,
    pub hours: f32,
    pub rr: f32,
    /// Hours given them over the whole illness.
    pub total_h: f32,
}

impl Care {
    /// What the care given on day `day` does to the chance of dying that day of `def`, as a
    /// relative risk: the least relative risk a carer brought that day (supportive care's, or a
    /// treatment's), in proportion to the share of a day's care given (05-05 §2.2: an effect
    /// applies only to care actually delivered). 1 on a day nobody tended them.
    pub fn rr(&self, def: &DiseaseDef, day: i64) -> f64 {
        if self.day != day || self.hours <= 0.0 {
            return 1.0;
        }
        let best = def.care_rr.min(f64::from(self.rr)).clamp(0.0, 1.0);
        let share = (f64::from(self.hours) / def.care_h_per_day.max(1e-6)).min(1.0);
        1.0 - (1.0 - best) * share
    }
}

impl Default for Care {
    fn default() -> Care {
        Care {
            first: None,
            day: 0,
            hours: 0.0,
            rr: 1.0,
            total_h: 0.0,
        }
    }
}

/// One infection. Saved.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Episode {
    /// Its number, from 1, in the order infections came.
    pub id: u32,
    pub person: PermanentId,
    /// Its disease, by index in the catalog's diseases.
    pub disease: u16,
    /// The day it came, and how.
    pub infected: i64,
    pub acquired: Acquired,
    pub course: Course,
    /// The day it ended, and how; `None` while it runs.
    pub ended: Option<(i64, Outcome)>,
    /// The care its bearer was given (step three).
    pub care: Care,
    /// The outbreak it is part of, by number (step three; ADR-0015 §8's `part_of`).
    pub outbreak: Option<u32>,
}

/// An outbreak (M6a slice AZ, step three; ADR-0015 §8, ADR-0021 §5 as amended): the cases of one
/// disease among the people of one settlement, from the first until none of them has run for as
/// long as a new infection can take to show (its incubation's mean and three spreads). A case
/// taken while one is open in its settlement is part of it; one taken while none is begins one,
/// and its acquisition record is the outbreak's cause. Saved.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Outbreak {
    /// Its number, from 1, in the order they began.
    pub id: u32,
    pub disease: u16,
    /// Where its cases lived when they took it (`None`: no settlement).
    pub settlement: Option<PermanentId>,
    /// The day its first case took it, and that case's episode number.
    pub began: i64,
    pub first: u32,
    /// The day it ended; `None` while it is open.
    pub ended: Option<i64>,
}

impl Episode {
    /// Whether it runs (has not ended).
    pub fn runs(&self) -> bool {
        self.ended.is_none()
    }

    /// Whether its bearer sheds it on `day`.
    pub fn shedding(&self, day: i64) -> bool {
        self.runs() && self.course.shed_from <= day && day < self.course.shed_until
    }

    /// Whether its bearer is ill of it on `day`.
    pub fn ill_on(&self, day: i64) -> bool {
        self.runs() && self.course.ill_from <= day && day < self.course.ill_until
    }

    /// Whether it brought symptoms.
    pub fn symptomatic(&self) -> bool {
        self.course.ill_until > self.course.ill_from
    }

    /// Whether its course has run by `day`: nothing shed, no illness, from then on.
    pub fn run_by(&self, day: i64) -> bool {
        day >= self.course.shed_until && day >= self.course.ill_until
    }
}

/// Every infection there has been, in the order they came, and every outbreak. Saved; the
/// indexes are derived.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Sickness {
    episodes: Vec<Episode>,
    outbreaks: Vec<Outbreak>,
    /// Episodes that run, by index, in order.
    running: Vec<usize>,
    /// Each person's episodes, by index, in order.
    by_person: BTreeMap<PermanentId, Vec<usize>>,
}

impl Sickness {
    /// The store of `episodes`, in the order they came, and `outbreaks`, in the order they began.
    pub fn with(episodes: Vec<Episode>, outbreaks: Vec<Outbreak>) -> Sickness {
        let mut s = Sickness {
            outbreaks,
            ..Sickness::default()
        };
        for e in episodes {
            s.push(e);
        }
        s
    }

    /// Every outbreak, in the order they began.
    pub fn outbreaks(&self) -> &[Outbreak] {
        &self.outbreaks
    }

    /// Whether any outbreak is open.
    pub fn outbreak_open(&self) -> bool {
        self.outbreaks.iter().any(|o| o.ended.is_none())
    }

    /// The episode at `index` is part of the open outbreak of its disease in `settlement`, or
    /// begins one on its day; returns that outbreak's number and whether it began.
    pub fn join_outbreak(&mut self, index: usize, settlement: Option<PermanentId>) -> (u32, bool) {
        let Some(e) = self.episodes.get(index).copied() else {
            return (0, false);
        };
        let open = self
            .outbreaks
            .iter()
            .find(|o| o.ended.is_none() && o.disease == e.disease && o.settlement == settlement)
            .map(|o| o.id);
        let (id, began) = match open {
            Some(id) => (id, false),
            None => {
                let id = self.outbreaks.len() as u32 + 1;
                self.outbreaks.push(Outbreak {
                    id,
                    disease: e.disease,
                    settlement,
                    began: e.infected,
                    first: e.id,
                    ended: None,
                });
                (id, true)
            }
        };
        self.episodes[index].outbreak = Some(id);
        (id, began)
    }

    /// The episodes part of outbreak `id`.
    pub fn cases(&self, id: u32) -> impl Iterator<Item = &Episode> {
        self.episodes.iter().filter(move |e| e.outbreak == Some(id))
    }

    /// Ends outbreak `id` on `day`.
    pub fn end_outbreak(&mut self, id: u32, day: i64) {
        if let Some(o) = self.outbreaks.iter_mut().find(|o| o.id == id) {
            o.ended.get_or_insert(day);
        }
    }

    fn push(&mut self, e: Episode) {
        let i = self.episodes.len();
        if e.runs() {
            self.running.push(i);
        }
        self.by_person.entry(e.person).or_default().push(i);
        self.episodes.push(e);
    }

    /// Every episode, in the order they came.
    pub fn episodes(&self) -> &[Episode] {
        &self.episodes
    }

    /// Whether no infection runs.
    pub fn quiet(&self) -> bool {
        self.running.is_empty()
    }

    /// The episodes that run, by index, in the order they came.
    pub fn running(&self) -> &[usize] {
        &self.running
    }

    /// The episode at `index`.
    pub fn get(&self, index: usize) -> Option<&Episode> {
        self.episodes.get(index)
    }

    /// `person`'s episodes, in the order they came.
    pub fn of(&self, person: PermanentId) -> impl Iterator<Item = &Episode> {
        self.by_person
            .get(&person)
            .into_iter()
            .flatten()
            .filter_map(|&i| self.episodes.get(i))
    }

    /// Whether `person` cannot take `disease` on `day`: an infection of it runs in them, or one
    /// they lived through still protects them.
    pub fn protected(&self, person: PermanentId, disease: u16, day: i64) -> bool {
        self.of(person).any(|e| {
            e.disease == disease
                && (e.runs()
                    || (e.ended.is_some_and(|(_, o)| o == Outcome::Recovered)
                        && day < e.course.immune_until))
        })
    }

    /// Whether `person` is ill of anything on `day`.
    pub fn ill(&self, person: PermanentId, day: i64) -> bool {
        !self.running.is_empty() && self.of(person).any(|e| e.ill_on(day))
    }

    /// Records a new infection, numbered here; returns its index.
    pub fn add(
        &mut self,
        person: PermanentId,
        disease: u16,
        day: i64,
        acquired: Acquired,
        course: Course,
    ) -> usize {
        let id = self.episodes.len() as u32 + 1;
        self.push(Episode {
            id,
            person,
            disease,
            infected: day,
            acquired,
            course,
            ended: None,
            care: Care::default(),
            outbreak: None,
        });
        self.episodes.len() - 1
    }

    /// The episodes running in `person` that make them ill on `day`, by index.
    pub fn ill_with(&self, person: PermanentId, day: i64) -> Vec<usize> {
        self.by_person.get(&person).map_or_else(Vec::new, |v| {
            v.iter()
                .copied()
                .filter(|&i| self.episodes.get(i).is_some_and(|e| e.ill_on(day)))
                .collect()
        })
    }

    /// The hours of care the bearer of the episode at `index` has been given on `day`.
    pub fn cared(&self, index: usize, day: i64) -> f64 {
        self.episodes
            .get(index)
            .filter(|e| e.care.day == day)
            .map_or(0.0, |e| f64::from(e.care.hours))
    }

    /// `who` gave the bearer of the episode at `index` `hours` of care on `day`, knowing a
    /// treatment that brings its risk of dying to `rr` (1 for none).
    pub fn tend(&mut self, index: usize, day: i64, who: PermanentId, hours: f64, rr: f64) {
        let Some(e) = self.episodes.get_mut(index) else {
            return;
        };
        let c = &mut e.care;
        if c.first.is_none() {
            c.first = Some((day, who));
        }
        if c.day != day || c.hours <= 0.0 {
            c.day = day;
            c.hours = 0.0;
            c.rr = 1.0;
        }
        c.hours += hours as f32;
        c.rr = c.rr.min(rr as f32);
        c.total_h += hours as f32;
    }

    /// Ends the episode at `index` on `day`, as `outcome`.
    pub fn end(&mut self, index: usize, day: i64, outcome: Outcome) {
        if let Some(e) = self.episodes.get_mut(index)
            && e.ended.is_none()
        {
            e.ended = Some((day, outcome));
            self.running.retain(|&i| i != index);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::{Days, DiseaseRoute};

    fn cholera() -> DiseaseDef {
        DiseaseDef {
            id: "test:disease/cholera".to_owned(),
            name: "Cholera".to_owned(),
            routes: vec![DiseaseRoute::Water, DiseaseRoute::Household],
            incubation: Days { mean: 1.5, sd: 1.0 },
            shed_lead_days: 0.0,
            shed: Days { mean: 5.0, sd: 2.5 },
            shed_after: Days { mean: 0.0, sd: 0.0 },
            symptomatic: 0.25,
            ill: Days { mean: 4.0, sd: 1.5 },
            severe_by_age: vec![(0.0, 0.4)],
            severe_death_per_day: 0.2,
            immunity_years: [3.0, 10.0],
            household_hazard: 0.03,
            shed_ill_per_day: 1000.0,
            shed_silent_per_day: 10.0,
            decay_per_day: 0.2,
            care_h_per_day: 2.0,
            care_rr: 0.95,
            treatments: Vec::new(),
        }
    }

    #[test]
    fn care_lowers_the_day_s_risk_by_the_share_of_a_day_s_care_given() {
        let mut d = cholera();
        d.care_rr = 0.9;
        let mut c = Care::default();
        assert_eq!(c.rr(&d, 5), 1.0, "nobody came");
        c.day = 5;
        c.hours = 1.0;
        c.rr = 1.0;
        assert!(
            (c.rr(&d, 5) - 0.95).abs() < 1e-12,
            "half a day's care, half the effect"
        );
        assert_eq!(c.rr(&d, 6), 1.0, "care given another day");
        c.hours = 3.0;
        assert!(
            (c.rr(&d, 5) - 0.9).abs() < 1e-12,
            "a full day's care at most"
        );
        c.rr = 0.05;
        assert!(
            (c.rr(&d, 5) - 0.05).abs() < 1e-6,
            "a treatment a carer knew"
        );
    }

    #[test]
    fn a_lognormal_duration_keeps_its_mean_and_spread() {
        let d = Days { mean: 5.0, sd: 2.5 };
        let mut rng = Rng64::from_key(&[1, 2, 3]);
        let n = 200_000;
        let xs: Vec<f64> = (0..n).map(|_| d.at(normal(&mut rng))).collect();
        let mean = xs.iter().sum::<f64>() / n as f64;
        let sd = (xs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n as f64).sqrt();
        assert!((mean - 5.0).abs() < 0.05, "{mean}");
        assert!((sd - 2.5).abs() < 0.05, "{sd}");
        assert!(xs.iter().all(|&x| x > 0.0));
        assert_eq!(Days { mean: 3.0, sd: 0.0 }.at(2.0), 3.0);
    }

    #[test]
    fn a_course_runs_in_an_order_the_body_allows() {
        let def = cholera();
        let mut ill = 0;
        let mut severe = 0;
        for k in 0..20_000u64 {
            let mut rng = Rng64::from_key(&[k]);
            let c = course(&def, 30.0, 100, &mut rng);
            // Nothing before the day after it came; shedding and illness as long as drawn.
            assert!(c.shed_from > 100 && c.ill_from > 100);
            assert!(c.shed_until > c.shed_from);
            assert!(c.ill_until >= c.ill_from);
            assert!(c.immune_until >= c.shed_until.max(c.ill_until) + 3 * 365);
            assert!(c.immune_until <= c.shed_until.max(c.ill_until) + 10 * 365);
            if c.ill_until > c.ill_from {
                ill += 1;
                // Shedding starts with symptoms for cholera and lasts at least through them.
                assert_eq!(c.shed_from, c.ill_from);
                assert!(c.shed_until >= c.ill_until);
            }
            if c.severe {
                severe += 1;
                assert!(c.ill_until > c.ill_from, "only the ill are severely ill");
            }
        }
        let ill_share = f64::from(ill) / 20_000.0;
        let severe_share = f64::from(severe) / f64::from(ill);
        assert!((ill_share - 0.25).abs() < 0.015, "{ill_share}");
        assert!((severe_share - 0.4).abs() < 0.03, "{severe_share}");
    }

    #[test]
    fn protection_holds_while_it_runs_and_after_it_until_its_day() {
        let id = PermanentId::from_raw(7).expect("non-zero");
        let mut s = Sickness::default();
        let course = Course {
            shed_from: 11,
            shed_until: 15,
            ill_from: 11,
            ill_until: 14,
            severe: false,
            immune_until: 1_000,
        };
        assert!(!s.protected(id, 0, 10));
        let i = s.add(id, 0, 10, Acquired::Observer { influence: 1 }, course);
        assert!(s.protected(id, 0, 10) && !s.protected(id, 1, 10));
        assert!(!s.ill(id, 10) && s.ill(id, 11) && s.ill(id, 13) && !s.ill(id, 14));
        assert!(s.get(i).is_some_and(|e| e.shedding(14) && !e.shedding(15)));
        s.end(i, 15, Outcome::Recovered);
        assert!(s.quiet());
        assert!(s.protected(id, 0, 999) && !s.protected(id, 0, 1_000));
        // Saved and loaded, the indexes come back.
        let again = Sickness::with(s.episodes().to_vec(), s.outbreaks().to_vec());
        assert_eq!(again, s);
    }
}
