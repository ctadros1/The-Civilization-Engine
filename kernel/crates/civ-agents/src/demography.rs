//! Births, deaths and couples as pure rules (research 05-01, 05-02 §1.6, 04-08, 06-01, 06-07).
//! The population applies them once a day (`population::life`); everything here can be tested
//! without a world.
//!
//! Hazards are daily chances `1 − e^(−h·Δt)` (ADR-0003 §1.6, research 05-01 §1.1). Every draw is
//! keyed by world seed, purpose, person, day and what it is for, so no counter is saved and
//! splitting a day into pieces changes nothing.

use std::collections::BTreeMap;

use civ_core::time::DAYS_PER_YEAR;
use civ_core::{PermanentId, Rng64, SimTime};

use crate::history::{Cause, PersonRecord};
use crate::params::{FamilyParams, FertilityParams, MortalityParams, step_at};
use crate::person::Traits;

/// Purpose tag for life draws (ADR-0003 keyed randomness).
pub const PURPOSE_LIFE: u64 = 0x6c69_6665_3030_3031; // "life0001"

/// Days in a month, for monthly chances.
pub const DAYS_PER_MONTH: f64 = DAYS_PER_YEAR as f64 / 12.0;

/// What a life draw is for: the last part of its key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Draw {
    /// Whether someone dies today.
    Death = 1,
    /// Whether a woman conceives today.
    Conception = 2,
    /// How a pregnancy goes.
    Pregnancy = 3,
    /// A birth: the child, and its mother's recovery and survival.
    Birth = 4,
    /// Whether someone looks for a partner today, and whom they find.
    Partner = 5,
    /// A woman's lasting fecundability.
    Fecundity = 6,
    /// Whether a household gives up and leaves today.
    Leave = 7,
    /// A person's objection to taking, at birth (M4b slice AA).
    Objection = 8,
}

/// The generator for `person`'s draw of `what` on `day`.
pub fn life_rng(seed: u64, person: PermanentId, day: i64, what: Draw) -> Rng64 {
    Rng64::from_key(&[seed, PURPOSE_LIFE, person.get(), day as u64, what as u64])
}

/// A standard normal draw (Box–Muller).
pub fn normal(rng: &mut Rng64) -> f64 {
    let u1 = 1.0 - rng.next_f64();
    let u2 = rng.next_f64();
    (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
}

/// Chance of an event within a day at a hazard of `per_year`.
pub fn chance_per_day(per_year: f64) -> f64 {
    1.0 - (-per_year.max(0.0) / DAYS_PER_YEAR as f64).exp()
}

/// The daily chance of an event whose chance in a month is `per_month`.
pub fn daily_from_monthly(per_month: f64) -> f64 {
    let p = per_month.clamp(0.0, 0.999_999);
    1.0 - (1.0 - p).powf(1.0 / DAYS_PER_MONTH)
}

/// Whether someone of `age` who has drawn the share `depleted` of their body's reserve dies today
/// on the draw `u`, and of what: the day's chance is split between the baseline and hunger in
/// proportion to their hazards (competing risks, research 05-01 §1.3).
pub fn death_today(params: &MortalityParams, age: f64, depleted: f64, u: f64) -> Option<Cause> {
    death_today_scaled(params, age, depleted, u, 1.0)
}

/// [`death_today`] with the baseline hazard, illness or accident, multiplied by `base_factor`
/// (an observer's blessing or curse, M4c slice AJ): hunger's part is the body's, never luck's.
pub fn death_today_scaled(
    params: &MortalityParams,
    age: f64,
    depleted: f64,
    u: f64,
    base_factor: f64,
) -> Option<Cause> {
    let (base, hunger) = params.hazards(age, depleted);
    let base = base * base_factor;
    let total = base + hunger;
    let p = chance_per_day(total);
    if total <= 0.0 || u >= p {
        return None;
    }
    Some(if u < p * base / total {
        Cause::Unspecified
    } else {
        Cause::Starvation
    })
}

/// A woman's chance to conceive today: her age, her lasting fecundability and the share of her
/// body's reserve she has drawn (research 05-02 §4.3: smooth, without a threshold).
pub fn conception_chance(f: &FertilityParams, age: f64, fecundity: f64, depleted: f64) -> f64 {
    let hunger = 0.5f64.powf(depleted.max(0.0) / f.hunger_halving.max(1e-6));
    daily_from_monthly(
        f.conception_per_month * step_at(&f.age_factor, age) * fecundity.max(0.0) * hunger,
    )
}

/// How a pregnancy conceived at `conceived` by a mother of `age` will go: when it ends and
/// whether in a loss.
pub fn pregnancy_course(
    f: &FertilityParams,
    age: f64,
    conceived: SimTime,
    rng: &mut Rng64,
) -> (SimTime, bool) {
    let loss = rng.next_f64() < step_at(&f.loss_by_age, age);
    let days = if loss {
        let (a, b) = (f.loss_days[0], f.loss_days[1].max(f.loss_days[0]));
        a + (b - a) * rng.next_f64()
    } else {
        let sd = f.pregnancy_sd_days.max(0.0);
        (f.pregnancy_days + sd * normal(rng)).clamp(
            (f.pregnancy_days - 3.0 * sd).max(1.0),
            f.pregnancy_days + 3.0 * sd,
        )
    };
    (conceived.plus_minutes(minutes_of_days(days)), loss)
}

/// Days a mother cannot conceive after a live birth (research 05-01 §1.5).
pub fn recovery_days(f: &FertilityParams, rng: &mut Rng64) -> f64 {
    (f.recovery_months + f.recovery_sd_months * normal(rng)).max(f.recovery_min_months)
        * DAYS_PER_MONTH
}

/// A woman's lasting fecundability factor: lognormal with mean 1.
pub fn fecundity(f: &FertilityParams, rng: &mut Rng64) -> f32 {
    let sd = f.fecundity_sd.max(0.0);
    (sd * normal(rng) - 0.5 * sd * sd).exp() as f32
}

/// Whole minutes in `days` days, at least one.
pub fn minutes_of_days(days: f64) -> i64 {
    ((days * civ_core::time::MINUTES_PER_DAY as f64).round() as i64).max(1)
}

/// The ancestors of `id` within `generations` (parents are the first), as the records know them.
pub fn ancestors(
    records: &BTreeMap<PermanentId, PersonRecord>,
    id: PermanentId,
    generations: u32,
) -> Vec<PermanentId> {
    let mut out = Vec::new();
    let mut layer = vec![id];
    for _ in 0..generations {
        let mut next = Vec::new();
        for p in &layer {
            if let Some(r) = records.get(p) {
                for parent in [r.mother, r.father].into_iter().flatten() {
                    if !out.contains(&parent) {
                        out.push(parent);
                        next.push(parent);
                    }
                }
            }
        }
        if next.is_empty() {
            break;
        }
        layer = next;
    }
    out
}

/// Whether two people are too closely related to be partners: one descends from the other, or
/// they share an ancestor, within `generations` (research 06-01 §1.4).
pub fn too_close(
    records: &BTreeMap<PermanentId, PersonRecord>,
    a: PermanentId,
    b: PermanentId,
    generations: u32,
) -> bool {
    if a == b {
        return true;
    }
    let up_a = ancestors(records, a, generations);
    let up_b = ancestors(records, b, generations);
    up_a.contains(&b) || up_b.contains(&a) || up_a.iter().any(|x| up_b.contains(x))
}

/// What a candidate couple is worth to the one looking, points (higher is better), or `None` if
/// the age gap rules it out: the man older than the woman by the gap people look for is best
/// (research 04-08 §1.1: eligibility, then acceptance).
pub fn couple_score(fam: &FamilyParams, woman_age: f64, man_age: f64) -> Option<f64> {
    let gap = man_age - woman_age;
    if gap < fam.age_gap_years[0] || gap > fam.age_gap_years[1] {
        return None;
    }
    Some(-fam.w_gap_per_year * (gap - fam.preferred_gap_years).abs())
}

/// Picks one of `scores` with probability proportional to `e^score` (a softmax at one point of
/// temperature) on the draw `u`.
pub fn pick_softmax(scores: &[f64], u: f64) -> Option<usize> {
    let top = scores.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if !top.is_finite() {
        return None;
    }
    let weights: Vec<f64> = scores.iter().map(|s| (s - top).exp()).collect();
    let total: f64 = weights.iter().sum();
    let mut at = u.clamp(0.0, 1.0) * total;
    for (i, w) in weights.iter().enumerate() {
        if at < *w {
            return Some(i);
        }
        at -= w;
    }
    Some(weights.len() - 1)
}

/// A child's personality: `heritability` of the mean of its parents' (a parent unknown counts as
/// the population mean of 0), and the rest its own, so the spread stays that of the parents'
/// generation (research 04-03 §1.2).
pub fn inherit_traits(
    mother: Option<&Traits>,
    father: Option<&Traits>,
    heritability: f64,
    rng: &mut Rng64,
) -> Traits {
    let h = heritability.clamp(0.0, 1.0);
    let own = (1.0 - h * h / 2.0).max(0.0).sqrt();
    let mut one = |m: f32, f: f32| (h * f64::from(m + f) / 2.0 + own * normal(rng)) as f32;
    let (m, f) = (
        mother.copied().unwrap_or_default(),
        father.copied().unwrap_or_default(),
    );
    Traits {
        openness: one(m.openness, f.openness),
        conscientiousness: one(m.conscientiousness, f.conscientiousness),
        extraversion: one(m.extraversion, f.extraversion),
        agreeableness: one(m.agreeableness, f.agreeableness),
        neuroticism: one(m.neuroticism, f.neuroticism),
        risk: one(m.risk, f.risk),
    }
}

/// A given name from `list` that nobody in `taken` bears, if any is left, else any (research
/// 06-07 §4.1: repetition is normal, but not among the living of one small community).
pub fn pick_name(list: &[String], taken: &[&str], rng: &mut Rng64) -> String {
    let free: Vec<&String> = list
        .iter()
        .filter(|n| !taken.contains(&n.as_str()))
        .collect();
    let pool: Vec<&String> = if free.is_empty() {
        list.iter().collect()
    } else {
        free
    };
    if pool.is_empty() {
        return "Unnamed".to_owned();
    }
    pool[rng.below(pool.len() as u64) as usize].clone()
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::needs::Sex;
    use crate::params::{Residence, Siler};

    pub(crate) fn mortality() -> MortalityParams {
        MortalityParams {
            siler: Siler {
                a: 0.351,
                b: 0.895,
                c: 0.011,
                d: 6.70e-6,
                e: 0.125,
            },
            hunger_ratio_at_half: 2.63,
            hunger_ratio_max: 11.6,
            exhaustion_per_day: 0.05,
            exhaustion_power: 20.0,
            maternal_death_per_birth: 0.0075,
        }
    }

    pub(crate) fn fertility() -> FertilityParams {
        FertilityParams {
            conception_per_month: 0.10,
            age_factor: vec![
                (15.0, 0.30),
                (20.0, 1.0),
                (30.0, 0.85),
                (35.0, 0.6),
                (40.0, 0.25),
                (45.0, 0.02),
                (50.0, 0.0),
            ],
            fecundity_sd: 0.5,
            hunger_halving: 0.33,
            pregnancy_days: 266.0,
            pregnancy_sd_days: 10.0,
            loss_by_age: vec![
                (0.0, 0.09),
                (25.0, 0.098),
                (30.0, 0.12),
                (35.0, 0.18),
                (40.0, 0.33),
                (45.0, 0.536),
            ],
            loss_days: [42.0, 140.0],
            recovery_months: 20.0,
            recovery_sd_months: 7.0,
            recovery_min_months: 6.0,
            loss_recovery_months: 1.0,
            weaned_recovery_months: 2.0,
            boys_per_100_girls: 105.0,
            pregnancy_kcal_day: [85.0, 285.0, 475.0],
        }
    }

    pub(crate) fn family() -> FamilyParams {
        FamilyParams {
            seek_min_age: [16.0, 18.0],
            seek_max_age: [45.0, 60.0],
            seek_per_month: [0.04, 0.03],
            age_gap_years: [-2.0, 14.0],
            preferred_gap_years: 3.0,
            w_gap_per_year: 0.5,
            kin_exclusion_generations: 2,
            residence: Residence::NewHousehold,
            independent_age: 15.0,
            trait_heritability: 0.4,
        }
    }

    fn id(n: u64) -> PermanentId {
        PermanentId::from_raw(n).expect("non-zero")
    }

    #[test]
    fn daily_draws_reproduce_the_life_table() {
        // Living every day of the first five years on daily draws gives the Siler table's 1q0 and
        // 5q0 (research 05-01 §1.2: 216 and 358 per thousand), not a once-a-year approximation.
        let m = mortality();
        let (mut dead1, mut dead5) = (0u32, 0u32);
        let n = 20_000u64;
        for k in 1..=n {
            for day in 0..5 * 365 {
                let age = (day as f64 + 0.5) / 365.0;
                let u = life_rng(7, id(k), day, Draw::Death).next_f64();
                if death_today(&m, age, 0.0, u).is_some() {
                    if day < 365 {
                        dead1 += 1;
                    }
                    dead5 += 1;
                    break;
                }
            }
        }
        let (q1, q5) = (f64::from(dead1) / n as f64, f64::from(dead5) / n as f64);
        let (e1, e5) = (1.0 - m.siler.survival(1.0), 1.0 - m.siler.survival(5.0));
        assert!((q1 - e1).abs() < 0.015, "1q0 {q1} against {e1}");
        assert!((q5 - e5).abs() < 0.015, "5q0 {q5} against {e5}");
    }

    #[test]
    fn hunger_raises_the_hazard_smoothly_and_names_its_cause() {
        let m = mortality();
        let (base, extra) = m.hazards(30.0, 0.0);
        assert!(base > 0.0 && extra == 0.0);
        // Half the reserve drawn: 2.63 times the baseline (research 05-02 §2.7), and a little of
        // the exhaustion hazard.
        let (b, e) = m.hazards(30.0, 0.5);
        let ratio = (b + e) / b;
        assert!((2.6..3.3).contains(&ratio), "ratio at half {ratio}");
        // It never falls as more is drawn, a severely wasted child is about as much at risk as
        // the research's severe wasting, and a body at the end of its reserve seldom lasts a
        // month.
        let mut last = 0.0;
        for k in 0..=20 {
            let (b, e) = m.hazards(30.0, f64::from(k) / 20.0);
            assert!(b + e >= last);
            last = b + e;
        }
        let (b, e) = m.hazards(2.0, 0.75);
        assert!((8.0..12.0).contains(&((b + e) / b)), "{}", (b + e) / b);
        assert!((1.0 - chance_per_day(last)).powi(30) < 0.25);
        // A death of someone starving is mostly put down to hunger.
        let u = chance_per_day(last) * 0.99;
        assert_eq!(death_today(&m, 30.0, 1.0, u), Some(Cause::Starvation));
        assert_eq!(death_today(&m, 30.0, 0.0, 0.999), None);
    }

    #[test]
    fn conception_follows_age_fecundity_and_hunger() {
        let f = fertility();
        assert_eq!(conception_chance(&f, 12.0, 1.0, 0.0), 0.0);
        assert_eq!(conception_chance(&f, 50.0, 1.0, 0.0), 0.0);
        let peak = conception_chance(&f, 25.0, 1.0, 0.0);
        // A month of daily chances is the authored monthly chance.
        let month = 1.0 - (1.0 - peak).powf(DAYS_PER_MONTH);
        assert!((month - 0.10).abs() < 1e-9, "{month}");
        assert!(conception_chance(&f, 42.0, 1.0, 0.0) < peak / 3.0);
        assert!(conception_chance(&f, 25.0, 2.0, 0.0) > peak * 1.9);
        // A third of the reserve drawn halves it, without a threshold.
        let hungry = conception_chance(&f, 25.0, 1.0, 0.33);
        assert!((hungry / peak - 0.5).abs() < 0.02, "{}", hungry / peak);
        // Fecundability averages 1 across women.
        let mean: f64 = (0..20_000u64)
            .map(|k| {
                f64::from(fecundity(
                    &f,
                    &mut life_rng(1, id(k + 1), 0, Draw::Fecundity),
                ))
            })
            .sum::<f64>()
            / 20_000.0;
        assert!((mean - 1.0).abs() < 0.03, "{mean}");
    }

    #[test]
    fn pregnancies_end_at_term_or_early_in_a_loss() {
        let f = fertility();
        let start = SimTime::from_minutes(1_000_000);
        let (mut losses, mut n) = (0u32, 0u32);
        for k in 0..4000u64 {
            let mut rng = life_rng(3, id(k + 1), 0, Draw::Pregnancy);
            let (due, loss) = pregnancy_course(&f, 27.0, start, &mut rng);
            let days = (due.minutes() - start.minutes()) as f64 / 1440.0;
            if loss {
                losses += 1;
                assert!((42.0..=140.0).contains(&days), "{days}");
            } else {
                assert!((236.0..=296.0).contains(&days), "{days}");
            }
            n += 1;
        }
        // 9.8 % at 25-29 (research 05-01 §2.2).
        let share = f64::from(losses) / f64::from(n);
        assert!((share - 0.098).abs() < 0.02, "{share}");
        let rec: f64 = (0..4000u64)
            .map(|k| recovery_days(&f, &mut life_rng(3, id(k + 1), 0, Draw::Birth)))
            .sum::<f64>()
            / 4000.0
            / DAYS_PER_MONTH;
        assert!((19.0..22.0).contains(&rec), "{rec}");
    }

    fn record(n: u64, sex: Sex, mother: Option<u64>, father: Option<u64>) -> PersonRecord {
        PersonRecord {
            id: id(n),
            given: format!("p{n}"),
            sex,
            born: SimTime::ZERO,
            died: None,
            left: None,
            mother: mother.map(id),
            father: father.map(id),
            origin: crate::history::Origin::Founder,
            residence: Vec::new(),
        }
    }

    #[test]
    fn close_kin_are_never_partners_and_second_cousins_may_be() {
        // 1 + 2 have 3 and 4; 3 has 5 with 10, 4 has 6 with 11; 5 has 7 with 12, 6 has 8 with
        // 13; 1 also has 9 with 14 (a half-sibling of 3 and 4).
        let mut records = BTreeMap::new();
        for r in [
            record(1, Sex::Female, None, None),
            record(2, Sex::Male, None, None),
            record(3, Sex::Female, Some(1), Some(2)),
            record(4, Sex::Male, Some(1), Some(2)),
            record(10, Sex::Male, None, None),
            record(11, Sex::Female, None, None),
            record(5, Sex::Female, Some(3), Some(10)),
            record(6, Sex::Male, Some(11), Some(4)),
            record(12, Sex::Male, None, None),
            record(13, Sex::Female, None, None),
            record(7, Sex::Female, Some(5), Some(12)),
            record(8, Sex::Male, Some(13), Some(6)),
            record(14, Sex::Male, None, None),
            record(9, Sex::Male, Some(1), Some(14)),
        ] {
            records.insert(r.id, r);
        }
        let close = |a: u64, b: u64| too_close(&records, id(a), id(b), 2);
        assert!(close(3, 4), "siblings");
        assert!(close(3, 9), "half-siblings");
        assert!(close(1, 4), "mother and son");
        assert!(close(5, 6), "first cousins");
        assert!(close(5, 4), "niece and uncle");
        assert!(close(1, 6), "grandmother and grandson");
        assert!(!close(7, 8), "second cousins");
        assert!(!close(10, 11), "strangers");
        // With no rule only the same person is excluded.
        assert!(!too_close(&records, id(3), id(4), 0));
    }

    #[test]
    fn couples_prefer_the_usual_age_gap() {
        let fam = family();
        assert_eq!(couple_score(&fam, 20.0, 40.0), None);
        assert_eq!(couple_score(&fam, 30.0, 27.0), None);
        let best = couple_score(&fam, 20.0, 23.0).expect("in range");
        assert!(best > couple_score(&fam, 20.0, 30.0).expect("in range"));
        assert_eq!(pick_softmax(&[], 0.5), None);
        assert_eq!(pick_softmax(&[0.0, -50.0], 0.99), Some(0));
        assert_eq!(pick_softmax(&[-50.0, 0.0], 0.01), Some(1));
    }

    #[test]
    fn children_resemble_their_parents_a_little() {
        let tall = Traits {
            openness: 2.0,
            conscientiousness: 2.0,
            extraversion: 2.0,
            agreeableness: 2.0,
            neuroticism: 2.0,
            risk: 2.0,
        };
        let n = 20_000u64;
        let (mut sum, mut sq) = (0.0, 0.0);
        let mut free = 0.0;
        for k in 0..n {
            let mut rng = life_rng(5, id(k + 1), 0, Draw::Birth);
            let t = inherit_traits(Some(&tall), Some(&tall), 0.4, &mut rng);
            sum += f64::from(t.openness);
            let u = inherit_traits(None, None, 0.4, &mut rng);
            sq += f64::from(u.openness).powi(2);
            free += f64::from(u.openness);
        }
        // Mid-parent 2 regresses to 0.8 at a heritability of 0.4; without parents the spread is
        // the generation's own share.
        assert!((sum / n as f64 - 0.8).abs() < 0.03);
        let var = sq / n as f64 - (free / n as f64).powi(2);
        assert!((var - 0.92).abs() < 0.05, "{var}");
    }

    #[test]
    fn names_are_not_repeated_among_the_living_while_others_are_free() {
        let list: Vec<String> = ["Ada", "Bryn", "Cora"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        for k in 0..50u64 {
            let mut rng = life_rng(1, id(k + 1), 0, Draw::Birth);
            assert_eq!(pick_name(&list, &["Ada", "Cora"], &mut rng), "Bryn");
            let any = pick_name(&list, &["Ada", "Bryn", "Cora"], &mut rng);
            assert!(list.contains(&any));
        }
        let mut rng = life_rng(1, id(1), 0, Draw::Birth);
        assert_eq!(pick_name(&[], &[], &mut rng), "Unnamed");
    }
}
