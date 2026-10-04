//! Choosing what to do next (research 01-09 §4.3, 04-07 §5.1; ADR-0003).
//!
//! At each decision every authored activity the person could do gets its **one best target**
//! (the best known patch, the nearest water, home, the hearth). Each candidate is scored as a sum
//! of considerations in one unit (points); options that cannot be done are excluded with a
//! reason. A softmax over the totals, with a temperature proportional to their spread, picks one
//! with a keyed draw. The scores, the runner-up and the exclusions are kept as the receipt.

use civ_core::PermanentId;

use crate::history::{Reason, Scored, Term};
use crate::params::{ActivityDef, Behavior, DecisionParams, GoodUse};
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
    /// Days of food the household wants in store now to see it through to its next harvest,
    /// with a margin (0 when it expects none).
    pub food_outlook_days: f64,
    /// Household's water, days.
    pub water_days: f64,
    /// Days of water the household tries to keep.
    pub water_target_days: f64,
    /// Household's daily food need, kcal.
    pub household_kcal_day: f64,
    /// Whether there is food at home that can be eaten now.
    pub has_food: bool,
    /// Whether there is food at home that needs a fire, and no firewood.
    pub food_needs_fire: bool,
    /// Household's firewood, days.
    pub fuel_days: f64,
    /// Days of firewood the household tries to keep.
    pub fuel_target_days: f64,
    /// Household's daily firewood use, kilograms.
    pub household_fuel_day: f64,
    /// Whether the person stands at home.
    pub at_home: bool,
    /// Home position, metres.
    pub home: (f32, f32),
    /// The settlement hearth, if any.
    pub hearth: Option<(f32, f32)>,
}

/// A place a gathering trip could go: patch, walking minutes one way, expected return per
/// person-hour, what the good is for, and the destination point.
#[derive(Clone, Copy, Debug)]
pub struct PatchOption {
    /// The patch.
    pub patch: u32,
    /// One-way walk, minutes.
    pub walk_min: f64,
    /// Expected return, kilograms of the good per person-hour.
    pub kg_per_hour: f64,
    /// What the good is for.
    pub purpose: GoodUse,
    /// Its food energy, kcal per kilogram.
    pub kcal_per_kg: f64,
    /// Days of the household's need it already holds of this good.
    pub stored_days: f64,
    /// For a material: kilograms of it the household still needs for what it is building,
    /// beyond what it holds (0 for food and fuel).
    pub need_kg: f64,
    /// For a material: how pressing the building it is for is ([`BuildOption::urgency`]).
    pub urgency: f64,
    /// For a material: kilograms of it the tools the household lacks are made of, beyond what
    /// it holds.
    pub tool_need_kg: f64,
    /// For a material: whether work waits on the tool it is for.
    pub tool_blocked: bool,
    /// Where to stand, metres.
    pub at: (f32, f32),
}

/// Field work a person could do: on which field, where, and what it is worth.
#[derive(Clone, Copy, Debug)]
pub struct FieldOption {
    /// The field, or `None` for new ground to mark out.
    pub field: Option<PermanentId>,
    /// One-way walk, minutes (0 for work at home).
    pub walk_min: f64,
    /// Where to stand, metres.
    pub at: (f32, f32),
    /// Food for the year ahead an hour of a capable adult's work brings: the field's expected
    /// harvest over the work it still needs, kcal.
    pub kcal_per_hour: f64,
    /// Hours of a capable adult's work this task still needs on the field.
    pub hours_left: f64,
    /// How much more grain is worth to the household, 0–1: `target / (target + held)`.
    pub room: f64,
    /// Field work left over the work the household can still do before the task's season
    /// closes; 0 when it has no deadline.
    pub urgency: f64,
    /// The work is done at home (threshing).
    pub at_home: bool,
    /// The work brings food within days (reaping and threshing), so a shortage presses on it as
    /// on gathering.
    pub soon: bool,
}

/// Work on the household's home: which building (or a new one), where, the stage under way and
/// how much of it can be done with the materials at home.
#[derive(Clone, Copy, Debug)]
pub struct BuildOption {
    /// The building, or `None` for a home not yet begun.
    pub building: Option<PermanentId>,
    /// Where to work, metres.
    pub at: (f32, f32),
    /// One-way walk, minutes.
    pub walk_min: f64,
    /// Hours of a capable adult's work the materials at home allow on the stage under way.
    pub workable_h: f64,
    /// Hours of a capable adult's work the stage under way still needs, materials aside.
    pub left_h: f64,
    /// Building work left over the work the household can still do before it wants to be under
    /// a roof.
    pub urgency: f64,
}

/// A household of the settlement that could spare food: who, the walk to its home, and the food
/// it would give.
#[derive(Clone, Copy, Debug)]
pub struct GiverOption {
    /// The household.
    pub household: PermanentId,
    /// One-way walk to its home, minutes.
    pub walk_min: f64,
    /// Its home, metres.
    pub at: (f32, f32),
    /// Food energy it would give, kcal.
    pub kcal: f64,
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

/// What working a recipe would bring the household.
#[derive(Clone, Copy, Debug)]
pub enum MakeWorth {
    /// Food made ready to eat, or a step nearer it.
    Food {
        /// Food energy made, kcal.
        kcal: f64,
        /// How short the household is of what the recipe makes, 0–1: ready food against the
        /// days of it kept in hand, or the food a step from ready against its own days.
        short: f64,
        /// How much more of it is worth: `target / (target + held)`.
        room: f64,
        /// With nothing ready to eat, it is the step toward a meal that can be taken now.
        toward_meal: bool,
    },
    /// A tool the household lacks.
    Tool {
        /// The tool, by index in the goods.
        tool: usize,
        /// Units of the household's want it answers, 0–1.
        need: f64,
    },
}

/// A recipe a person could work at home: how much of it, for how long, and what it is worth.
#[derive(Clone, Copy, Debug)]
pub struct MakeOption {
    /// Units the session would make.
    pub units: f64,
    /// Minutes of work for them at the person's speed.
    pub minutes: f64,
    /// What they are worth.
    pub worth: MakeWorth,
}

/// Least of a tool, in standard tools, that still does the work (the last of a worn one).
pub const MIN_TOOL: f64 = 0.02;

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
    /// What one person carries, kilograms.
    pub carry_kg: f64,
}

/// Daylight left, minutes, below which a household whose water will not last until morning
/// fetches water before anything else of ordinary weight (a tuning value).
const LAST_WATER_BEFORE_DARK_MIN: f64 = 180.0;

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

/// A make activity's recipe option, or why there is none and the tool it lacks if that is why;
/// given the tools work is already waiting on.
pub type BestMake<'a> = &'a dyn Fn(usize, &[usize]) -> Result<MakeOption, (Reason, Option<usize>)>;

/// What the household has to work with: free tools, and the recipes it could work.
pub struct Workshop<'a> {
    /// Tools free for the person to use, in standard tools, by good index (0 for other goods).
    pub free_tools: &'a [f64],
    /// For a make activity: the recipe option, or why there is none (and the tool it lacks, if
    /// that is why). Given the tools work is already waiting on.
    pub best_make: BestMake<'a>,
    /// Whether a make activity makes a tool (scored last, once the tools work waits on are known).
    pub makes_tool: &'a dyn Fn(usize) -> bool,
}

impl std::fmt::Debug for Workshop<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Workshop")
            .field("free_tools", &self.free_tools)
            .finish_non_exhaustive()
    }
}

/// Scores every activity. Returns the candidates and the exclusions. `best_field` gives each
/// farm activity its best field, or the reason there is none; `giver` is the household that
/// could best spare food; `build` is the work on the household's home, or why there is none;
/// `shop` the tools at hand and the recipes. Work that needs a tool nobody in the household has
/// free is excluded, and the tool counts as one work waits on.
#[allow(clippy::too_many_arguments)]
pub fn candidates(
    defs: &[ActivityDef],
    w: &DecisionParams,
    f: &Facts,
    limits: &Limits,
    best_patch: &dyn Fn(usize) -> Option<PatchOption>,
    best_field: &dyn Fn(usize) -> Result<FieldOption, Reason>,
    water: Option<WaterOption>,
    giver: Option<GiverOption>,
    build: Result<BuildOption, Reason>,
    shop: &Workshop,
) -> (Vec<Candidate>, Vec<(u16, Reason)>) {
    let mut out: Vec<Candidate> = Vec::new();
    let mut excluded = Vec::new();
    let mut blocked: Vec<usize> = Vec::new();
    // Make activities wait until the tools work waits on are known: food recipes first, then the
    // ones that make tools.
    let mut order: Vec<usize> = (0..defs.len())
        .filter(|&i| defs[i].behavior != Behavior::Make)
        .collect();
    order.extend(
        (0..defs.len()).filter(|&i| defs[i].behavior == Behavior::Make && !(shop.makes_tool)(i)),
    );
    order.extend(
        (0..defs.len()).filter(|&i| defs[i].behavior == Behavior::Make && (shop.makes_tool)(i)),
    );
    for i in order {
        let def = &defs[i];
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
                    let why = if f.food_needs_fire {
                        Reason::NoFire
                    } else {
                        Reason::NoFood
                    };
                    excluded.push((id, why));
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
                let mut short =
                    (1.0 - f.water_days / f.water_target_days.max(1e-6)).clamp(0.0, 1.0);
                // No water is fetched in the dark: late in the day, what is at home must last
                // until morning.
                let lasts_night = f.water_days * 1440.0 >= f.until_sunrise_min;
                if !f.dark && !lasts_night && f.daylight_left_min < LAST_WATER_BEFORE_DARK_MIN {
                    short = 1.0;
                }
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
                // How long daylight allows, after the walk there and back.
                let room = if def.daylight_only {
                    f.daylight_left_min - 2.0 * walk - 20.0
                } else {
                    f64::from(def.max_minutes)
                };
                if room < f64::from(def.min_minutes) {
                    excluded.push((id, Reason::NotInDark));
                    continue;
                }
                // Work until a load is full, or as long as daylight and the authored range allow.
                let pace = f.capacity * def.rate;
                let fill = limits.carry_kg / (patch.kg_per_hour * pace).max(1e-9) * 60.0;
                let minutes = room
                    .min(f64::from(def.max_minutes))
                    .min(fill.max(f64::from(def.min_minutes)))
                    .max(1.0);
                let hours = minutes / 60.0;
                let kg = (patch.kg_per_hour * hours * pace).min(limits.carry_kg);
                // A trip's worth saturates with what it brings (a response curve, research
                // 01-09 §4.3): a haul of `trip_half_worth_days` of the household's need is worth
                // half. Beyond any shortage, more of a good is worth less the more of that good
                // is in store (diminishing marginal value, `target / (target + stored)`): a good
                // that keeps is not piled up without end, while fresh food, eaten before the
                // stores that keep, is still worth gathering.
                match patch.purpose {
                    GoodUse::Food => {
                        let kcal = kg * patch.kcal_per_kg;
                        let half = w.trip_half_worth_days * f.household_kcal_day.max(1.0);
                        let worth = kcal / (kcal + half.max(1.0));
                        let target = f.food_target_days.max(1e-6);
                        let short = (1.0 - f.food_days / target).clamp(0.0, 1.0);
                        let room = target / (target + patch.stored_days.max(0.0));
                        term(&mut terms, Reason::FoodShortage, w.w_food * short * worth);
                        // Stores that will not last to the harvest: wild food stretches them
                        // while there is time to go out for it (research 08-05 §1.5: a grain
                        // holder plans a stock path to the next harvest).
                        let lean = if f.food_outlook_days > 0.0 {
                            (1.0 - f.food_days / f.food_outlook_days).clamp(0.0, 1.0)
                        } else {
                            0.0
                        };
                        term(&mut terms, Reason::LeanSeason, w.w_lean * lean * worth);
                        term(&mut terms, Reason::UsefulWork, w.w_work * worth * room);
                        if !f.has_food {
                            // Hungry with nothing to eat at home: food is the point of going out.
                            term(&mut terms, Reason::Hunger, w.w_hunger * f.hunger * worth);
                        }
                    }
                    GoodUse::Fuel => {
                        let half = w.trip_half_worth_days * f.household_fuel_day.max(0.1);
                        let worth = kg / (kg + half);
                        let target = f.fuel_target_days.max(1e-6);
                        let short = (1.0 - f.fuel_days / target).clamp(0.0, 1.0);
                        let room = target / (target + patch.stored_days.max(0.0));
                        term(&mut terms, Reason::FuelShortage, w.w_fuel * short * worth);
                        term(&mut terms, Reason::UsefulWork, w.w_work * worth * room);
                        if f.food_needs_fire {
                            // Food that needs cooking, and nothing to cook it on.
                            term(&mut terms, Reason::Hunger, w.w_hunger * f.hunger * worth);
                        }
                    }
                    GoodUse::Material | GoodUse::Tool => {
                        if patch.need_kg <= 0.0 && patch.tool_need_kg <= 0.0 {
                            excluded.push((id, Reason::NotNeeded));
                            continue;
                        }
                        // A load is worth what it brings toward what is still needed: two thirds
                        // for a full one, or for one that brings all that is still missing.
                        let toward = |need: f64| {
                            if need <= 0.0 {
                                return 0.0;
                            }
                            let useful = kg.min(need);
                            let scale = need.min(limits.carry_kg);
                            useful / (useful + scale / 2.0).max(1e-6)
                        };
                        let (build, tools) = (toward(patch.need_kg), toward(patch.tool_need_kg));
                        term(&mut terms, Reason::Shelter, w.w_shelter * build);
                        term(
                            &mut terms,
                            Reason::Deadline,
                            w.w_deadline * patch.urgency.clamp(0.0, 2.0) * build,
                        );
                        term(&mut terms, Reason::Tools, w.w_tools * tools);
                        if patch.tool_blocked {
                            term(&mut terms, Reason::Deadline, w.w_deadline * tools);
                        }
                    }
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
            Behavior::Farm => {
                let field = match best_field(i) {
                    Ok(field) => field,
                    Err(why) => {
                        excluded.push((id, why));
                        continue;
                    }
                };
                let walk = field.walk_min;
                let room = if def.daylight_only {
                    f.daylight_left_min - 2.0 * walk - 20.0
                } else {
                    f64::from(def.max_minutes)
                };
                if room < f64::from(def.min_minutes) {
                    excluded.push((id, Reason::NotInDark));
                    continue;
                }
                // Work as long as daylight and the task allow, within the authored range. Work
                // done by hand, without the tools the task's rates assume, goes slower.
                let pace = f.capacity * def.rate;
                let needed = field.hours_left * 60.0 / pace.max(0.05);
                let minutes = room
                    .min(f64::from(def.max_minutes))
                    .min(needed)
                    .max(f64::from(def.min_minutes))
                    .max(1.0);
                let hours = minutes / 60.0;
                // Future food, valued like food brought home now (the same response curve), less
                // the more grain the household already holds.
                let kcal = field.kcal_per_hour * hours * pace;
                let half = w.trip_half_worth_days * f.household_kcal_day.max(1.0);
                let worth = kcal / (kcal + half.max(1.0));
                term(&mut terms, Reason::Harvest, w.w_farm * worth * field.room);
                if field.soon {
                    // The crop is food within days: a shortage presses on bringing it in.
                    let target = f.food_target_days.max(1e-6);
                    let short = (1.0 - f.food_days / target).clamp(0.0, 1.0);
                    term(&mut terms, Reason::FoodShortage, w.w_food * short * worth);
                    if !f.has_food {
                        term(&mut terms, Reason::Hunger, w.w_hunger * f.hunger * worth);
                    }
                }
                term(
                    &mut terms,
                    Reason::Deadline,
                    w.w_deadline * field.urgency.clamp(0.0, 2.0),
                );
                if !field.at_home {
                    term(
                        &mut terms,
                        Reason::Walking,
                        -w.w_walk_hour * 2.0 * walk / 60.0,
                    );
                }
                term(
                    &mut terms,
                    Reason::Effort,
                    -w.w_effort * (def.par - 1.0).max(0.0) * hours * f.sleep_pressure,
                );
                let work = Step::Work {
                    minutes: minutes.round().max(1.0) as u32,
                };
                let steps = if field.at_home {
                    let mut steps = walk_home_first(f);
                    steps.push(work);
                    steps
                } else {
                    vec![Step::Walk { to: field.at }, work, Step::Walk { to: f.home }]
                };
                let target = field.field.map_or(Target::NewField, Target::Field);
                out.push(finish(id, target, terms, steps));
            }
            Behavior::Build => {
                let option = match build {
                    Ok(option) => option,
                    Err(why) => {
                        excluded.push((id, why));
                        continue;
                    }
                };
                let room = f.daylight_left_min - 2.0 * option.walk_min - 20.0;
                let room = if def.daylight_only {
                    room
                } else {
                    f64::from(def.max_minutes)
                };
                if room < f64::from(def.min_minutes) {
                    excluded.push((id, Reason::NotInDark));
                    continue;
                }
                // As long as daylight and the materials at hand allow, within the authored range;
                // a shorter session only to finish the stage.
                let pace = (f.capacity * def.rate).max(0.05);
                let workable = option.workable_h * 60.0 / pace;
                let left = option.left_h * 60.0 / pace;
                if workable <= 0.0 || workable < f64::from(def.min_minutes).min(left) {
                    excluded.push((id, Reason::NoMaterials));
                    continue;
                }
                let minutes = room
                    .min(f64::from(def.max_minutes))
                    .min(workable)
                    .max(f64::from(def.min_minutes).min(workable))
                    .max(1.0);
                let hours = minutes / 60.0;
                term(&mut terms, Reason::Shelter, w.w_shelter);
                term(
                    &mut terms,
                    Reason::Deadline,
                    w.w_deadline * option.urgency.clamp(0.0, 2.0),
                );
                if option.walk_min > 0.5 {
                    term(
                        &mut terms,
                        Reason::Walking,
                        -w.w_walk_hour * 2.0 * option.walk_min / 60.0,
                    );
                }
                term(
                    &mut terms,
                    Reason::Effort,
                    -w.w_effort * (def.par - 1.0).max(0.0) * hours * f.sleep_pressure,
                );
                let work = Step::Work {
                    minutes: minutes.round().max(1.0) as u32,
                };
                let steps = if option.walk_min > 0.5 {
                    vec![
                        Step::Walk { to: option.at },
                        work,
                        Step::Walk { to: f.home },
                    ]
                } else {
                    let mut steps = walk_home_first(f);
                    steps.push(work);
                    steps
                };
                let target = option
                    .building
                    .map_or(Target::NewBuilding, Target::Building);
                out.push(finish(id, target, terms, steps));
            }
            Behavior::Ask => {
                // Ask when in need, from a household able to help (research 08-11 §1.1, §5.4:
                // need-based help between households, no debt kept).
                let target = f.food_target_days.max(1e-6);
                let short = (1.0 - f.food_days / target).clamp(0.0, 1.0);
                if short <= 0.0 {
                    excluded.push((id, Reason::NotShort));
                    continue;
                }
                let Some(giver) = giver else {
                    excluded.push((id, Reason::NoOneToAsk));
                    continue;
                };
                if giver.walk_min > f64::from(def.max_walk_minutes) {
                    excluded.push((id, Reason::Unreachable));
                    continue;
                }
                let half = w.trip_half_worth_days * f.household_kcal_day.max(1.0);
                let worth = giver.kcal / (giver.kcal + half.max(1.0));
                term(&mut terms, Reason::FoodShortage, w.w_food * short * worth);
                if !f.has_food {
                    term(&mut terms, Reason::Hunger, w.w_hunger * f.hunger * worth);
                }
                term(
                    &mut terms,
                    Reason::Walking,
                    -w.w_walk_hour * 2.0 * giver.walk_min / 60.0,
                );
                let steps = vec![
                    Step::Walk { to: giver.at },
                    Step::Work {
                        minutes: def.min_minutes.max(1),
                    },
                    Step::Walk { to: f.home },
                ];
                out.push(finish(id, Target::Household(giver.household), terms, steps));
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
            Behavior::Make => {
                let option = match (shop.best_make)(i, &blocked) {
                    Ok(option) => option,
                    Err((why, tool)) => {
                        if let Some(t) = tool
                            && !blocked.contains(&t)
                        {
                            blocked.push(t);
                        }
                        excluded.push((id, why));
                        continue;
                    }
                };
                let minutes = option.minutes.min(f64::from(def.max_minutes)).max(1.0);
                let hours = minutes / 60.0;
                match option.worth {
                    MakeWorth::Food {
                        kcal,
                        short,
                        room,
                        toward_meal,
                    } => {
                        let half = w.trip_half_worth_days * f.household_kcal_day.max(1.0);
                        let worth = kcal / (kcal + half.max(1.0));
                        term(&mut terms, Reason::ReadyFood, w.w_food * short * worth);
                        term(&mut terms, Reason::UsefulWork, w.w_work * worth * room);
                        if toward_meal && !f.has_food {
                            term(&mut terms, Reason::Hunger, w.w_hunger * f.hunger * worth);
                        }
                    }
                    MakeWorth::Tool { tool, need } => {
                        term(&mut terms, Reason::Tools, w.w_tools * need.clamp(0.0, 1.0));
                        if blocked.contains(&tool) {
                            term(
                                &mut terms,
                                Reason::Deadline,
                                w.w_deadline * need.clamp(0.0, 1.0),
                            );
                        }
                    }
                }
                term(
                    &mut terms,
                    Reason::Effort,
                    -w.w_effort * (def.par - 1.0).max(0.0) * hours * f.sleep_pressure,
                );
                let mut steps = walk_home_first(f);
                steps.push(Step::Work {
                    minutes: minutes.round().max(1.0) as u32,
                });
                out.push(finish(id, Target::Home, terms, steps));
            }
        }
        // Work that needs a tool the household has none of free is left out, and the tool is one
        // that work waits on.
        if def.behavior != Behavior::Make
            && let Some(last) = out.last()
            && last.scored.def == id
            && let Some(&missing) = def
                .tools
                .iter()
                .find(|&&t| shop.free_tools.get(t).copied().unwrap_or(0.0) < MIN_TOOL)
        {
            out.pop();
            excluded.push((id, Reason::NoTool));
            if !blocked.contains(&missing) {
                blocked.push(missing);
            }
        }
    }
    // Field work done at a slower rate (by hand) is left out while the same task can be done
    // faster (with the tool for it).
    let fastest = |task| {
        out.iter()
            .map(|c| &defs[usize::from(c.scored.def)])
            .filter(|d| d.behavior == Behavior::Farm && d.task == Some(task))
            .map(|d| d.rate)
            .fold(0.0, f64::max)
    };
    let slower: Vec<u16> = out
        .iter()
        .filter(|c| {
            let d = &defs[usize::from(c.scored.def)];
            d.behavior == Behavior::Farm && d.task.is_some_and(|t| d.rate < fastest(t))
        })
        .map(|c| c.scored.def)
        .collect();
    out.retain(|c| !slower.contains(&c.scored.def));
    excluded.extend(slower.into_iter().map(|d| (d, Reason::BetterWay)));
    (out, excluded)
}

/// Softmax choice among the acceptable options (research 01-09 §4.3): those worth more than doing
/// nothing (a positive total), or every option when none is. The temperature is proportional to
/// the spread of the acceptable totals. `u` is a uniform draw in [0, 1). Returns the chosen index,
/// its probability and the temperature. An option whose costs outweigh what it brings is never
/// taken while something worthwhile can be done, so random draws do not pile up useless work.
pub fn choose(totals: &[f32], w: &DecisionParams, u: f64) -> (usize, f64, f64) {
    if totals.is_empty() {
        return (0, 1.0, 0.0);
    }
    let acceptable: Vec<usize> = (0..totals.len()).filter(|&i| totals[i] > 0.0).collect();
    let pool: Vec<usize> = if acceptable.is_empty() {
        (0..totals.len()).collect()
    } else {
        acceptable
    };
    let values: Vec<f64> = pool.iter().map(|&i| f64::from(totals[i])).collect();
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let var = values.iter().map(|t| (t - mean).powi(2)).sum::<f64>() / n;
    let temperature = (w.temperature_sd_fraction * var.sqrt()).max(w.min_temperature);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let weights: Vec<f64> = values
        .iter()
        .map(|t| ((t - max) / temperature).exp())
        .collect();
    let sum: f64 = weights.iter().sum();
    let mut acc = 0.0;
    let target = u * sum;
    for (k, wt) in weights.iter().enumerate() {
        acc += wt;
        if target < acc {
            return (pool[k], wt / sum, temperature);
        }
    }
    let last = weights.len() - 1;
    (pool[last], weights[last] / sum, temperature)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::ActivityDef;

    fn weights() -> DecisionParams {
        DecisionParams {
            temperature_sd_fraction: 0.4,
            min_temperature: 0.5,
            w_hunger: 10.0,
            max_hunger_drive: 2.0,
            w_sleep: 12.0,
            w_social: 4.0,
            w_food: 8.0,
            w_lean: 5.0,
            w_work: 2.0,
            w_fuel: 6.0,
            w_farm: 8.0,
            w_deadline: 6.0,
            w_shelter: 4.0,
            w_tools: 4.0,
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
    fn options_worth_less_than_nothing_are_not_taken_while_one_is_worth_something() {
        let w = weights();
        // Gathering firewood nobody needs, when tired: its costs outweigh it.
        let totals = [1.0f32, -0.5, -2.0, 3.0];
        for k in 0..100 {
            let (i, _, _) = choose(&totals, &w, f64::from(k) / 100.0);
            assert!(i == 0 || i == 3, "chose {i}");
        }
        // When nothing is worth doing, every option is possible and the least bad most likely.
        let bad = [-1.0f32, -0.2, -3.0];
        let mut picks = [0usize; 3];
        for k in 0..1000 {
            picks[choose(&bad, &w, f64::from(k) / 1000.0).0] += 1;
        }
        assert!(
            picks[1] > picks[0] && picks[0] > picks[2] && picks[2] > 0,
            "{picks:?}"
        );
    }

    fn facts() -> Facts {
        Facts {
            age: 30.0,
            capacity: 1.0,
            hunger: 0.5,
            sleep_drive: 0.0,
            sleep_pressure: 0.2,
            loneliness: 0.2,
            dark: false,
            daylight_left_min: 600.0,
            evening: 0.0,
            until_sunrise_min: 900.0,
            food_days: 50.0,
            food_target_days: 5.0,
            food_outlook_days: 0.0,
            water_days: 2.0,
            water_target_days: 1.5,
            household_kcal_day: 10_000.0,
            has_food: false,
            food_needs_fire: false,
            fuel_days: 5.0,
            fuel_target_days: 3.0,
            household_fuel_day: 6.0,
            at_home: true,
            home: (0.0, 0.0),
            hearth: None,
        }
    }

    fn limits() -> Limits {
        Limits {
            sleep_needed_min: 480.0,
            sleep_min: 240.0,
            sleep_max: 630.0,
            nap: (20.0, 90.0),
            sleep_threshold: 0.12,
            meal_min: 25,
            carry_kg: 20.0,
        }
    }

    fn activity(id: &str, behavior: Behavior, tools: Vec<usize>, rate: f64) -> ActivityDef {
        ActivityDef {
            id: id.into(),
            name: id.into(),
            doing: id.into(),
            behavior,
            resource: None,
            task: (behavior == Behavior::Farm).then_some(civ_land::FieldTask::Reap),
            recipe: (behavior == Behavior::Make).then_some(0),
            tools,
            rate,
            par: 3.0,
            min_age_years: 10.0,
            max_age_years: 70.0,
            min_minutes: 30,
            max_minutes: 240,
            daylight_only: true,
            max_walk_minutes: 30,
        }
    }

    fn field() -> FieldOption {
        FieldOption {
            field: Some(PermanentId::from_raw(7).expect("id")),
            walk_min: 5.0,
            at: (10.0, 0.0),
            kcal_per_hour: 3000.0,
            hours_left: 50.0,
            room: 0.5,
            urgency: 1.0,
            at_home: false,
            soon: true,
        }
    }

    /// Scores `defs` with the household's tools (`free`, by good) and a recipe option.
    fn score(
        defs: &[ActivityDef],
        free: &[f64],
        make: BestMake,
        makes_tool: &dyn Fn(usize) -> bool,
    ) -> (Vec<Candidate>, Vec<(u16, Reason)>) {
        let shop = Workshop {
            free_tools: free,
            best_make: make,
            makes_tool,
        };
        candidates(
            defs,
            &weights(),
            &facts(),
            &limits(),
            &|_| None,
            &|_| Ok(field()),
            None,
            None,
            Err(Reason::Built),
            &shop,
        )
    }

    #[test]
    fn work_without_its_tool_is_left_out_and_the_tool_is_wanted_the_more() {
        // Good 0 is the sickle. With one free, people reap with it and not by hand.
        let defs = vec![
            activity("reap", Behavior::Farm, vec![0], 1.0),
            activity("reap_by_hand", Behavior::Farm, Vec::new(), 0.6),
            activity("make_sickle", Behavior::Make, Vec::new(), 1.0),
        ];
        let tool = |_: usize, _: &[usize]| {
            Ok(MakeOption {
                units: 1.0,
                minutes: 180.0,
                worth: MakeWorth::Tool { tool: 0, need: 0.5 },
            })
        };
        let makes = |d: usize| d == 2;
        let (cands, excluded) = score(&defs, &[1.0], &tool, &makes);
        let chosen: Vec<u16> = cands.iter().map(|c| c.scored.def).collect();
        assert!(chosen.contains(&0) && !chosen.contains(&1), "{chosen:?}");
        assert!(excluded.contains(&(1, Reason::BetterWay)));
        let making = cands.iter().find(|c| c.scored.def == 2).expect("make");
        assert!(
            !making
                .scored
                .terms
                .iter()
                .any(|t| t.reason == Reason::Deadline)
        );
        // Without a free sickle, reaping waits on it: people reap by hand, and making a sickle
        // has the season pressing on it.
        let (cands, excluded) = score(&defs, &[0.0], &tool, &makes);
        let chosen: Vec<u16> = cands.iter().map(|c| c.scored.def).collect();
        assert!(chosen.contains(&1) && !chosen.contains(&0), "{chosen:?}");
        assert!(excluded.contains(&(0, Reason::NoTool)));
        let making = cands.iter().find(|c| c.scored.def == 2).expect("make");
        assert!(
            making
                .scored
                .terms
                .iter()
                .any(|t| t.reason == Reason::Deadline)
        );
    }

    #[test]
    fn making_food_ready_is_worth_most_to_the_hungry_with_nothing_ready() {
        let defs = vec![activity("bake", Behavior::Make, Vec::new(), 1.0)];
        let bake = |short: f64| {
            move |_: usize, _: &[usize]| {
                Ok(MakeOption {
                    units: 5.0,
                    minutes: 120.0,
                    worth: MakeWorth::Food {
                        kcal: 15_000.0,
                        short,
                        room: 0.5,
                        toward_meal: true,
                    },
                })
            }
        };
        let none = |_: usize| false;
        let total = |short: f64| {
            let (cands, _) = score(&defs, &[], &bake(short), &none);
            cands[0].scored.total
        };
        assert!(total(1.0) > total(0.2), "a shortage presses on it");
        let (cands, _) = score(&defs, &[], &bake(1.0), &none);
        assert!(
            cands[0]
                .scored
                .terms
                .iter()
                .any(|t| t.reason == Reason::Hunger)
        );
        assert_eq!(cands[0].steps, vec![Step::Work { minutes: 120 }]);
        // A recipe that cannot be worked is left out with its reason, and a missing tool is
        // one work waits on.
        let lacking = |_: usize, _: &[usize]| Err((Reason::NoTool, Some(3)));
        let (cands, excluded) = score(&defs, &[], &lacking, &none);
        assert!(cands.is_empty());
        assert_eq!(excluded, vec![(0, Reason::NoTool)]);
    }

    #[test]
    fn equal_options_share_the_probability() {
        let w = weights();
        let (_, p, _) = choose(&[3.0, 3.0, 3.0, 3.0], &w, 0.3);
        assert!((p - 0.25).abs() < 1e-9);
    }
}
