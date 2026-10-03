//! Choosing what to do next (research 01-09 §4.3, 04-07 §5.1; ADR-0003).
//!
//! At each decision every authored activity the person could do gets its **one best target**
//! (the best known patch, the nearest water, home, the hearth). Each candidate is scored as a sum
//! of considerations in one unit (points); options that cannot be done are excluded with a
//! reason. A softmax over the totals, with a temperature proportional to their spread, picks one
//! with a keyed draw. The scores, the runner-up and the exclusions are kept as the receipt.

use crate::history::{Reason, Scored, Term};
use crate::params::{ActivityDef, Behavior, DecisionParams};
use crate::person::{Step, Target};

/// Everything a decision looks at, copied out so scoring borrows nothing.
#[derive(Clone, Debug)]
pub struct Facts {
    /// Age, years.
    pub age: f64,
    /// Work capacity, 0–1.
    pub capacity: f64,
    /// Hunger, 0 when full; 1 at full hunger, more with an energy deficit.
    pub hunger: f64,
    /// Sleep drive: pressure times the circadian weight.
    pub sleep_drive: f64,
    /// Sleep pressure alone.
    pub sleep_pressure: f64,
    /// Loneliness, 0–1.
    pub loneliness: f64,
    /// Whether it is dark now.
    pub dark: bool,
    /// Minutes of daylight left today (0 in the dark).
    pub daylight_left_min: f64,
    /// Evening: the hours after sunset when people sit together, 0–1.
    pub evening: f64,
    /// Minutes until the next sunrise.
    pub until_sunrise_min: f64,
    /// Household's food, days.
    pub food_days: f64,
    /// Days of food the household tries to keep.
    pub food_target_days: f64,
    /// Household's water, days.
    pub water_days: f64,
    /// Days of water the household tries to keep.
    pub water_target_days: f64,
    /// Household's daily food need, kcal.
    pub household_kcal_day: f64,
    /// Whether the household has food at all.
    pub has_food: bool,
    /// Whether the person stands at home.
    pub at_home: bool,
    /// Home position, metres.
    pub home: (f32, f32),
    /// The settlement hearth, if any.
    pub hearth: Option<(f32, f32)>,
}

/// A place a gathering trip could go: patch, walking minutes one way, expected return per
/// person-hour, and the destination point.
#[derive(Clone, Copy, Debug)]
pub struct PatchOption {
    /// The patch.
    pub patch: u32,
    /// One-way walk, minutes.
    pub walk_min: f64,
    /// Expected return, units per person-hour.
    pub rate: f64,
    /// Where to stand, metres.
    pub at: (f32, f32),
}

/// A nearby water point: walking minutes one way and the destination.
#[derive(Clone, Copy, Debug)]
pub struct WaterOption {
    /// The cell next to water.
    pub cell: u32,
    /// One-way walk, minutes.
    pub walk_min: f64,
    /// Where to stand, metres.
    pub at: (f32, f32),
}

/// A scored option with the steps it would take.
#[derive(Clone, Debug)]
pub struct Candidate {
    /// The score.
    pub scored: Scored,
    /// The steps.
    pub steps: Vec<Step>,
}

/// Inputs that are not facts about the person: sleep length, meal length, carry limits.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// Minutes of sleep needed to be rested.
    pub sleep_needed_min: f64,
    /// Shortest sleep, minutes.
    pub sleep_min: f64,
    /// Longest sleep, minutes.
    pub sleep_max: f64,
    /// Shortest and longest daytime nap, minutes.
    pub nap: (f64, f64),
    /// Sleep drive below which sleep is not an option (the wake threshold).
    pub sleep_threshold: f64,
    /// Minutes a meal takes.
    pub meal_min: u32,
    /// Food one person carries, kcal.
    pub carry_kcal: f64,
}

fn term(terms: &mut Vec<Term>, reason: Reason, points: f64) {
    if points != 0.0 && points.is_finite() {
        terms.push(Term {
            reason,
            points: points as f32,
        });
    }
}

fn finish(def: u16, target: Target, mut terms: Vec<Term>, steps: Vec<Step>) -> Candidate {
    terms.sort_by(|a, b| b.points.abs().total_cmp(&a.points.abs()));
    terms.truncate(8);
    let total = terms.iter().map(|t| t.points).sum();
    Candidate {
        scored: Scored {
            def,
            target,
            total,
            terms,
        },
        steps,
    }
}

fn walk_home_first(f: &Facts) -> Vec<Step> {
    if f.at_home {
        Vec::new()
    } else {
        vec![Step::Walk { to: f.home }]
    }
}

/// Scores every activity. Returns the candidates and the exclusions.
pub fn candidates(
    defs: &[ActivityDef],
    w: &DecisionParams,
    f: &Facts,
    limits: &Limits,
    best_patch: &dyn Fn(usize) -> Option<PatchOption>,
    water: Option<WaterOption>,
) -> (Vec<Candidate>, Vec<(u16, Reason)>) {
    let mut out = Vec::new();
    let mut excluded = Vec::new();
    for (i, def) in defs.iter().enumerate() {
        let id = i as u16;
        if f.age < def.min_age_years {
            excluded.push((id, Reason::TooYoung));
            continue;
        }
        if f.age > def.max_age_years {
            excluded.push((id, Reason::TooOld));
            continue;
        }
        if def.daylight_only && f.dark {
            excluded.push((id, Reason::NotInDark));
            continue;
        }
        let mut terms = Vec::new();
        match def.behavior {
            Behavior::Sleep => {
                // Two-process model: sleep comes when circadian-weighted pressure is above the
                // threshold at which a sleeper wakes.
                if f.sleep_drive < limits.sleep_threshold {
                    excluded.push((id, Reason::NotTired));
                    continue;
                }
                // A night's sleep runs to about dawn; by day it is a nap.
                let night_left = f.until_sunrise_min - 15.0;
                let minutes = if f.dark && night_left >= limits.sleep_min {
                    limits
                        .sleep_needed_min
                        .max(night_left)
                        .clamp(limits.sleep_min, limits.sleep_max)
                } else if f.dark {
                    limits
                        .sleep_needed_min
                        .max(night_left)
                        .clamp(limits.nap.0, limits.sleep_max)
                } else {
                    limits.sleep_needed_min.clamp(limits.nap.0, limits.nap.1)
                };
                term(&mut terms, Reason::Sleep, w.w_sleep * f.sleep_drive);
                let mut steps = walk_home_first(f);
                steps.push(Step::Work {
                    minutes: minutes.round().max(1.0) as u32,
                });
                out.push(finish(id, Target::Home, terms, steps));
            }
            Behavior::Eat => {
                if !f.has_food {
                    excluded.push((id, Reason::NoFood));
                    continue;
                }
                term(&mut terms, Reason::Hunger, w.w_hunger * f.hunger);
                let mut steps = walk_home_first(f);
                steps.push(Step::Work {
                    minutes: limits.meal_min.max(1),
                });
                out.push(finish(id, Target::Home, terms, steps));
            }
            Behavior::FetchWater => {
                let Some(water) = water else {
                    excluded.push((id, Reason::NoPlace));
                    continue;
                };
                if water.walk_min > f64::from(def.max_walk_minutes) {
                    excluded.push((id, Reason::Unreachable));
                    continue;
                }
                let short = (1.0 - f.water_days / f.water_target_days.max(1e-6)).clamp(0.0, 1.0);
                term(&mut terms, Reason::WaterShortage, w.w_water * short);
                term(
                    &mut terms,
                    Reason::Walking,
                    -w.w_walk_hour * 2.0 * water.walk_min / 60.0,
                );
                let steps = vec![
                    Step::Walk { to: water.at },
                    Step::Work {
                        minutes: def.min_minutes.max(1),
                    },
                    Step::Walk { to: f.home },
                    Step::Deposit,
                ];
                out.push(finish(id, Target::Water(water.cell), terms, steps));
            }
            Behavior::Gather => {
                let Some(patch) = best_patch(i) else {
                    excluded.push((id, Reason::NoPlace));
                    continue;
                };
                let walk = patch.walk_min;
                // Work as long as daylight allows, within the authored range.
                let room = if def.daylight_only {
                    f.daylight_left_min - 2.0 * walk - 20.0
                } else {
                    f64::from(def.max_minutes)
                };
                if room < f64::from(def.min_minutes) {
                    excluded.push((id, Reason::NotInDark));
                    continue;
                }
                let minutes = room.min(f64::from(def.max_minutes)).max(1.0);
                let hours = minutes / 60.0;
                let expected = (patch.rate * hours * f.capacity).min(limits.carry_kcal);
                // A trip's worth saturates with what it brings (a response curve, research
                // 01-09 §4.3): a haul of `trip_half_worth_days` of household food is worth half.
                let half = w.trip_half_worth_days * f.household_kcal_day.max(1.0);
                let worth = expected / (expected + half.max(1.0));
                let short = (1.0 - f.food_days / f.food_target_days.max(1e-6)).clamp(0.0, 1.0);
                term(&mut terms, Reason::FoodShortage, w.w_food * short * worth);
                term(&mut terms, Reason::UsefulWork, w.w_work * worth);
                if !f.has_food {
                    // Hungry with nothing to eat at home: food is the point of going out.
                    term(&mut terms, Reason::Hunger, w.w_hunger * f.hunger * worth);
                }
                term(
                    &mut terms,
                    Reason::Walking,
                    -w.w_walk_hour * 2.0 * walk / 60.0,
                );
                term(
                    &mut terms,
                    Reason::Effort,
                    -w.w_effort * (def.par - 1.0).max(0.0) * hours * f.sleep_pressure,
                );
                let steps = vec![
                    Step::Walk { to: patch.at },
                    Step::Work {
                        minutes: minutes.round() as u32,
                    },
                    Step::Walk { to: f.home },
                    Step::Deposit,
                ];
                out.push(finish(id, Target::Patch(patch.patch), terms, steps));
            }
            Behavior::Socialize => {
                let Some(hearth) = f.hearth else {
                    excluded.push((id, Reason::NoHearth));
                    continue;
                };
                term(
                    &mut terms,
                    Reason::Loneliness,
                    w.w_social * f.loneliness * f.evening.max(0.25),
                );
                let steps = vec![
                    Step::Walk { to: hearth },
                    Step::Work {
                        minutes: def.min_minutes.max(1),
                    },
                ];
                out.push(finish(id, Target::Hearth, terms, steps));
            }
            Behavior::Rest => {
                term(&mut terms, Reason::Rest, w.w_rest);
                let mut steps = walk_home_first(f);
                steps.push(Step::Work {
                    minutes: def.min_minutes.max(1),
                });
                out.push(finish(id, Target::Home, terms, steps));
            }
            Behavior::Play => {
                term(&mut terms, Reason::Play, w.w_play);
                let mut steps = walk_home_first(f);
                steps.push(Step::Work {
                    minutes: def.min_minutes.max(1),
                });
                out.push(finish(id, Target::Home, terms, steps));
            }
        }
    }
    (out, excluded)
}

/// Softmax choice with a temperature proportional to the spread of the totals. `u` is a uniform
/// draw in [0, 1). Returns the chosen index, its probability and the temperature.
pub fn choose(totals: &[f32], w: &DecisionParams, u: f64) -> (usize, f64, f64) {
    if totals.is_empty() {
        return (0, 1.0, 0.0);
    }
    let n = totals.len() as f64;
    let mean = totals.iter().map(|&t| f64::from(t)).sum::<f64>() / n;
    let var = totals
        .iter()
        .map(|&t| (f64::from(t) - mean).powi(2))
        .sum::<f64>()
        / n;
    let temperature = (w.temperature_sd_fraction * var.sqrt()).max(w.min_temperature);
    let max = totals
        .iter()
        .map(|&t| f64::from(t))
        .fold(f64::NEG_INFINITY, f64::max);
    let weights: Vec<f64> = totals
        .iter()
        .map(|&t| ((f64::from(t) - max) / temperature).exp())
        .collect();
    let sum: f64 = weights.iter().sum();
    let mut acc = 0.0;
    let target = u * sum;
    for (i, wt) in weights.iter().enumerate() {
        acc += wt;
        if target < acc {
            return (i, wt / sum, temperature);
        }
    }
    let last = weights.len() - 1;
    (last, weights[last] / sum, temperature)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn weights() -> DecisionParams {
        DecisionParams {
            temperature_sd_fraction: 0.4,
            min_temperature: 0.5,
            w_hunger: 10.0,
            w_sleep: 12.0,
            w_social: 4.0,
            w_food: 8.0,
            w_work: 2.0,
            trip_half_worth_days: 0.25,
            w_water: 6.0,
            w_walk_hour: 1.0,
            w_effort: 1.0,
            w_dark: 5.0,
            w_rest: 1.0,
            w_play: 2.0,
        }
    }

    #[test]
    fn a_clear_favourite_usually_wins_but_not_always() {
        let w = weights();
        let totals = [10.0f32, 2.0, 1.0];
        let (i, p, t) = choose(&totals, &w, 0.0);
        assert_eq!(i, 0);
        assert!(p > 0.95, "p = {p}");
        assert!(t >= w.min_temperature);
        // The last sliver of the distribution goes to the weakest option.
        let (i, _, _) = choose(&totals, &w, 0.999_999_9);
        assert_eq!(i, 2);
    }

    #[test]
    fn equal_options_share_the_probability() {
        let w = weights();
        let (_, p, _) = choose(&[3.0, 3.0, 3.0, 3.0], &w, 0.3);
        assert!((p - 0.25).abs() < 1e-9);
    }
}
