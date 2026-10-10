//! Choosing what to do next (research 01-09 §4.3, 04-07 §5.1; ADR-0003).
//!
//! At each decision every authored activity the person could do gets its **one best target**
//! (the best known patch, the nearest water, home, the hearth). Each candidate is scored as a sum
//! of considerations in one unit (points); options that cannot be done are excluded with a
//! reason. A softmax over the totals, with a temperature proportional to their spread, picks one
//! with a keyed draw. The scores, the runner-up and the exclusions are kept as the receipt.

use civ_core::PermanentId;
use civ_land::FieldTask;

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
    /// The household's settlement and where its hearth is, if it has one.
    pub hearth: Option<(PermanentId, (f32, f32))>,
    /// The gathering sitting at the hearth that they may attend, if any (ADR-0013 §1).
    pub gathering: Option<GatheringFacts>,
    /// A petition sitting at the hearth they heard of and may join (M4c slice AH): what joining
    /// it is worth to them and the minutes it still sits.
    pub petition: Option<GatheringFacts>,
    /// The round of the watch they keep, if they keep one and may walk it now (M4b slice AC).
    pub watch: Option<WatchFacts>,
    /// A payment an agreement owes that they are to carry, set aside and waiting, if they can
    /// walk it there and back in the day (M5c slice AV).
    pub carry: Option<CarryFacts>,
    /// A crossing their household is building, if it is within their reach (M5c slice AW, step
    /// two).
    pub crossing: Option<CrossingFacts>,
    /// The well their household is digging or relining, if any (M6a slice AY, step three).
    pub well: Option<WellFacts>,
    /// Whether a blow keeps them from work today (M4c slice AI, step four): they eat, drink, rest,
    /// sleep and keep company, and do no work.
    pub hurt: bool,
    /// Whether illness keeps them abed today (M6a slice AZ, ADR-0021 §5): they sleep, eat and
    /// rest at home, and do nothing else.
    pub ill: bool,
}

/// A crossing someone's household is building, or their polity is and they know the law that
/// asks it (M5c slice AW, steps two and three): where on the bank its work is done, the walk
/// there from home, the hours of a capable adult's work it still takes, the hours of walking each
/// hour of that work saves the household over the crossing's life, and, while their household
/// has not given the share of a polity's crossing asked of it, what keeping to the gathering's
/// word is worth to them, points (0 otherwise).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CrossingFacts {
    pub crossing: PermanentId,
    pub at: (f32, f32),
    pub walk_min: f64,
    pub left_h: f64,
    pub saves_per_hour: f64,
    pub duty: f64,
    /// As many as there is room for have begun work on it today (M5c slice AX).
    pub full: bool,
}

/// A well someone's household is digging or relining beside its home (M6a slice AY, step three;
/// ADR-0021 §3): where its work is done, the walk there from home, the hours of a capable adult's
/// work it still takes, the hours of walking a year each hour of that work saves the household,
/// and whether as many as its shaft has room for have begun work on it today.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WellFacts {
    pub well: PermanentId,
    pub at: (f32, f32),
    pub walk_min: f64,
    pub left_h: f64,
    pub saves_per_hour: f64,
    pub full: bool,
}

/// A payment someone is to carry (M5c slice AV, ADR-0020 §7): where the store set it aside, the
/// hearth of the settlement it goes to, the walk there from home, and what carrying it is worth.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CarryFacts {
    /// Where the paying store keeps it, metres.
    pub store: (f32, f32),
    /// The settlement it goes to, and its hearth, metres.
    pub settlement: PermanentId,
    pub hearth: (f32, f32),
    /// One-way walk to that hearth, minutes.
    pub walk_min: f64,
    /// Points: the duty.
    pub points: f64,
}

/// A round of the watch someone keeps (M4b slice AC): the homes it passes, in order, and what
/// walking it is worth to them.
#[derive(Clone, Debug, PartialEq)]
pub struct WatchFacts {
    /// Where to stand watch, in order, metres.
    pub stops: Vec<(f32, f32)>,
    /// Points: the duty, less for each round already walked tonight.
    pub points: f64,
}

/// A gathering a person may attend now (ADR-0013 §1): what having a say there is worth to them,
/// and the minutes it still sits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GatheringFacts {
    /// Points: the custom's pull, and what their household and their regard for the law's
    /// sponsor have at stake.
    pub points: f64,
    /// Minutes it still sits.
    pub minutes: f64,
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
    /// For digging: the deposit dug at, in place of a patch (M3b slice Q).
    pub deposit: Option<PermanentId>,
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

/// Work on what the household builds, its home, a store or a workshop: which building (or a new
/// one), where, the stage under way and how much of it can be done with the materials at home.
#[derive(Clone, Copy, Debug)]
pub struct BuildOption {
    /// The building, or `None` for one not yet begun.
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

/// Another settlement's hearth someone could visit (M5a slice AM): which, where, the walk one
/// way and what those they would see there are worth to them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VisitOption {
    /// The settlement.
    pub settlement: PermanentId,
    /// Its hearth, metres.
    pub hearth: (f32, f32),
    /// One-way walk, minutes.
    pub walk_min: f64,
    /// Points: kin there, those they know there, and the hope of meeting someone.
    pub points: f64,
}

/// A household whose store someone could take from (M4b slice AA, ADR-0015 §2), with what
/// weighs against it for this person, in points: their objection, the chance they believe they
/// run of being seen times what that would cost them, and their household's regard for the one
/// taken from.
#[derive(Clone, Copy, Debug)]
pub struct TakeOption {
    /// The household.
    pub household: PermanentId,
    /// One-way walk to its home, minutes.
    pub walk_min: f64,
    /// Its home, metres.
    pub at: (f32, f32),
    /// Food energy they could carry off, kcal.
    pub kcal: f64,
    /// Points against, for their objection to taking.
    pub objection: f64,
    /// Points against, for the chance of being seen.
    pub risk: f64,
    /// Points against, for regard.
    pub regard: f64,
}

/// The best purchase a household knows of (slice I): the seller, the walk to its home, the good
/// and what it would bring the household.
#[derive(Clone, Copy, Debug)]
pub struct TradeOption {
    /// The selling household, or workshop.
    pub seller: PermanentId,
    /// The seller is a workshop (slice J).
    pub firm: bool,
    /// One-way walk to its home, minutes.
    pub walk_min: f64,
    /// Its home, metres.
    pub at: (f32, f32),
    /// The good bought, by index in the catalog's goods.
    pub good: usize,
    /// What it brings the household.
    pub worth: TradeWorth,
}

/// What a purchase, or a wage, brings the household that gets it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TradeWorth {
    /// A tool it needs: how much of one it still wants, 0–1, as for making one.
    Tool { need: f64 },
    /// Food, kcal.
    Food { kcal: f64 },
    /// Other goods, worth so many hours of the household's own work to it (slice J: a wage).
    Goods { hours: f64 },
    /// Goods fetched from another settlement to sell at home (M5b slice AQ, ADR-0019 §6): the
    /// share of what they fetch there that the household keeps, 0–1, as for making to sell.
    Sale { share: f64 },
}

/// Paid work at a workshop of another household (slice J): where, for how long, and what the
/// pay brings the worker's household.
#[derive(Clone, Copy, Debug)]
pub struct JobOption {
    /// The workshop.
    pub firm: PermanentId,
    /// One-way walk to it, minutes.
    pub walk_min: f64,
    /// Where it is, metres.
    pub at: (f32, f32),
    /// Minutes of work it wants of the person this session.
    pub minutes: f64,
    /// What the pay brings the household.
    pub worth: TradeWorth,
}

/// A session of trying toward a technique (ADR-0008 §3): which, and what it is worth.
#[derive(Clone, Copy, Debug)]
pub struct TryOption {
    /// The technique, by index in the catalog's techniques.
    pub technique: u16,
    /// Utility points: the try weight times how much of the household's food the problem the
    /// technique answers would cost.
    pub points: f64,
}

/// Hours of a household's own work a wage must be worth to be worth half as much as can be (a
/// tuning value: a session's work).
pub const WAGE_HALF_WORTH_H: f64 = 3.0;

/// A nearby water point: walking minutes one way and the destination.
#[derive(Clone, Copy, Debug)]
pub struct WaterOption {
    /// The cell next to water.
    pub cell: u32,
    /// One-way walk, minutes.
    pub walk_min: f64,
    /// Where to stand, metres.
    pub at: (f32, f32),
    /// The well it is, if it is one (M6a slice AY, step three): its water is drawn from the
    /// well's column, not the cell's.
    pub well: Option<PermanentId>,
    /// Minutes to haul a load up to the ground there, beyond filling the vessels (0 at a bank or
    /// spring).
    pub lift_min: f64,
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
    /// A good others want and nobody offers, made to sell (slice I): the share of what they would
    /// give that the household keeps over its own cost, times how much of it is wanted, 0–1.
    Sale {
        /// 0–1.
        share: f64,
    },
    /// Food kept that would otherwise spoil before it is eaten (dried, smoked): its energy, kcal.
    Preserve {
        /// Kcal.
        kcal: f64,
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
    /// Made to sell: the household's workshop for it, if it has one yet (slice J).
    pub firm: Option<PermanentId>,
    /// Made at the workshop's own building, once it has one with its roof on (slice O): where,
    /// metres, and the one-way walk there, minutes. `None` for work at home.
    pub site: Option<((f32, f32), f64)>,
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
pub(crate) const LAST_WATER_BEFORE_DARK_MIN: f64 = 180.0;

/// Work at a site the household is making something at, a crossing or a well, as weighed.
struct Site {
    at: (f32, f32),
    walk_min: f64,
    left_h: f64,
    saves_per_hour: f64,
    duty: f64,
    full: bool,
}

/// A session of work at `site` for activity `def` (M5c slice AW, M6a slice AY): as long as
/// daylight allows within the authored range, a shorter session only to finish it; each hour of a
/// capable adult's work worth the walking it saves the household (`reason`), and, while a share
/// asked of the household is not given, the duty. Its steps, or why it is left out.
fn session_at_site(
    def: &ActivityDef,
    w: &DecisionParams,
    f: &Facts,
    site: &Site,
    reason: Reason,
    terms: &mut Vec<Term>,
) -> Result<Vec<Step>, Reason> {
    if site.walk_min > f64::from(def.max_walk_minutes) {
        return Err(Reason::Unreachable);
    }
    if site.full {
        return Err(Reason::Crowded);
    }
    let room = if def.daylight_only {
        f.daylight_left_min - 2.0 * site.walk_min - 20.0
    } else {
        f64::from(def.max_minutes)
    };
    if room < f64::from(def.min_minutes) {
        return Err(Reason::NotInDark);
    }
    let pace = (f.capacity * def.rate).max(0.05);
    let left = site.left_h * 60.0 / pace;
    let minutes = room
        .min(f64::from(def.max_minutes))
        .min(left)
        .max(f64::from(def.min_minutes).min(left))
        .max(1.0);
    let hours = minutes / 60.0;
    term(
        terms,
        reason,
        w.w_walk_hour * hours * pace * site.saves_per_hour,
    );
    if site.duty > 0.0 {
        term(terms, Reason::PublicWork, site.duty);
    }
    if site.walk_min > 0.5 {
        term(
            terms,
            Reason::Walking,
            -w.w_walk_hour * 2.0 * site.walk_min / 60.0,
        );
    }
    term(
        terms,
        Reason::Effort,
        -w.w_effort * (def.par - 1.0).max(0.0) * hours * f.sleep_pressure,
    );
    Ok(vec![
        Step::Walk { to: site.at },
        Step::Work {
            minutes: minutes.round().max(1.0) as u32,
        },
        Step::Walk { to: f.home },
    ])
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

/// Adds a consideration to a scored candidate, keeping its terms in order and its total.
pub fn add_term(c: &mut Candidate, reason: Reason, points: f64) {
    let mut terms = std::mem::take(&mut c.scored.terms);
    term(&mut terms, reason, points);
    let steps = std::mem::take(&mut c.steps);
    *c = finish(c.scored.def, c.scored.target, terms, steps);
}

/// A leisure block (Accelerated mode, ADR-0011 §4): the minutes leisure chosen with probability
/// `p` lasts, sessions of `session` minutes, and whether it ends by the person choosing something
/// else, when their next decision passes it over. Detailed mode would decide again after each
/// session and choose the same again with probability about `p` while nothing consequential
/// changes: the block is that run of sessions, its length a geometric draw from `u` (0–1). It is
/// cut at `until`, the minutes to the next consequential boundary, or at `longest`, where the
/// next decision is made afresh: as the draw is memoryless, the run is the same in distribution.
/// Never shorter than a session.
pub fn leisure_block(session: u32, longest: u32, until: f64, p: f64, u: f64) -> (u32, bool) {
    let session = session.max(1);
    // Sessions more after the first, before something else is chosen.
    let more = if p.is_nan() || p <= 0.0 {
        0.0
    } else if p >= 1.0 {
        f64::INFINITY
    } else {
        (u.clamp(f64::MIN_POSITIVE, 1.0).ln() / p.ln()).floor()
    };
    let drawn = f64::from(session) * (1.0 + more);
    let cut = until.min(f64::from(longest.max(session)));
    if drawn < cut {
        (drawn as u32, true)
    } else {
        ((cut.floor().max(0.0) as u32).max(session), false)
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
    /// Tools the household holds, free or in another member's hands, in standard tools, by good
    /// index (0 for other goods).
    pub held_tools: &'a [f64],
    /// For a make activity: the recipe option, or why there is none (and the tool it lacks, if
    /// that is why). Given the tools work is already waiting on.
    pub best_make: BestMake<'a>,
    /// Whether a make activity makes a tool (scored last, once the tools work waits on are known).
    pub makes_tool: &'a dyn Fn(usize) -> bool,
    /// Whether the person knows a technique, by index (ADR-0008): a faster way they do not know
    /// does not rule out a slower one they do.
    pub knows: &'a dyn Fn(usize) -> bool,
}

impl std::fmt::Debug for Workshop<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Workshop")
            .field("free_tools", &self.free_tools)
            .finish_non_exhaustive()
    }
}

/// The work an activity does, where more than one activity can do it at different rates: reaping
/// with a sickle or by hand, cutting wood with an axe or by hand.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Way {
    /// A task in a field.
    Field(FieldTask),
    /// A resource gathered, by index.
    Gather(usize),
}

impl Way {
    fn of(def: &ActivityDef) -> Option<Way> {
        match def.behavior {
            Behavior::Farm => def.task.map(Way::Field),
            Behavior::Gather => def.resource.map(Way::Gather),
            _ => None,
        }
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
    giver: &dyn Fn() -> Option<GiverOption>,
    trade: &dyn Fn() -> Option<TradeOption>,
    fetch: &dyn Fn() -> Result<TradeOption, Reason>,
    job: &dyn Fn() -> Option<JobOption>,
    build: Result<BuildOption, Reason>,
    shop: &Workshop,
    trying: &dyn Fn() -> Result<TryOption, Reason>,
    take: &dyn Fn() -> Result<TakeOption, Reason>,
    visits: &dyn Fn() -> Vec<VisitOption>,
) -> (Vec<Candidate>, Vec<(u16, Reason)>) {
    let mut out: Vec<Candidate> = Vec::new();
    let mut excluded = Vec::new();
    let mut blocked: Vec<usize> = Vec::new();
    // Work left out only because another member has the tools for it.
    let mut busy: Vec<u16> = Vec::new();
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
        // Carrying goods owed is open only to the one who is to carry them (M5c slice AV), and
        // is no part of anyone else's choice: not even a reason it was left out.
        if def.behavior == Behavior::Carry && f.carry.is_none() {
            continue;
        }
        // So is work on a crossing to all but those whose household is building one (M5c slice
        // AW).
        if def.behavior == Behavior::Bridge && f.crossing.is_none() {
            continue;
        }
        // And work on a well to all but those whose household is digging or relining one (M6a
        // slice AY, step three).
        if def.behavior == Behavior::Well && f.well.is_none() {
            continue;
        }
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
        // Illness keeps them abed (M6a slice AZ).
        if f.ill
            && !matches!(
                def.behavior,
                Behavior::Sleep | Behavior::Eat | Behavior::Rest
            )
        {
            excluded.push((id, Reason::Ill));
            continue;
        }
        // A blow keeps them from work (M4c slice AI, step four).
        if f.hurt
            && !matches!(
                def.behavior,
                Behavior::Sleep
                    | Behavior::Eat
                    | Behavior::FetchWater
                    | Behavior::Socialize
                    | Behavior::Rest
                    | Behavior::Ask
                    | Behavior::Attend
                    | Behavior::Petition
            )
        {
            excluded.push((id, Reason::Hurt));
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
                // until morning, through the first meal and the trip to the water at first light.
                // In the last of the light it counts as a full shortage; once the light left would
                // not see a meal and then the trip, the trip comes before anything that can wait
                // until dark, the strongest hunger included.
                let draw = def.min_minutes.max(1) + water.lift_min.round().max(0.0) as u32;
                let trip = 2.0 * water.walk_min + f64::from(draw);
                let morning = f.until_sunrise_min + f64::from(limits.meal_min) + trip;
                let lasts_night = f.water_days * 1440.0 >= morning;
                if !f.dark && !lasts_night {
                    if f.daylight_left_min < trip + f64::from(limits.meal_min) {
                        short = (w.w_hunger * w.max_hunger_drive + w.w_water) / w.w_water.max(1e-6);
                    } else if f.daylight_left_min < LAST_WATER_BEFORE_DARK_MIN {
                        short = 1.0;
                    }
                }
                term(&mut terms, Reason::WaterShortage, w.w_water * short);
                term(
                    &mut terms,
                    Reason::Walking,
                    -w.w_walk_hour * 2.0 * water.walk_min / 60.0,
                );
                let steps = vec![
                    Step::Walk { to: water.at },
                    Step::Work { minutes: draw },
                    Step::Walk { to: f.home },
                    Step::Deposit,
                ];
                let target = water.well.map_or(Target::Water(water.cell), Target::Well);
                out.push(finish(id, target, terms, steps));
            }
            // Digging is gathering at a deposit (M3b slice Q).
            Behavior::Gather | Behavior::Dig => {
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
                    GoodUse::Material | GoodUse::Tool | GoodUse::Store => {
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
                let target = patch
                    .deposit
                    .map_or(Target::Patch(patch.patch), Target::Deposit);
                out.push(finish(id, target, terms, steps));
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
                let Some(giver) = giver() else {
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
            Behavior::Take => {
                // Take from another household's store when short (M4b slice AA, ADR-0015 §2;
                // research 04-09 §5.3: U = G − C − M − I − p̂L). The gain is asking's, the food
                // the household needs; against it are the walk, the person's objection, the
                // chance they believe they run of being seen times what that would cost them, and
                // their regard for those they would take from. The moral filter is `take`'s: above
                // it, taking is not a candidate at all.
                let target = f.food_target_days.max(1e-6);
                let short = (1.0 - f.food_days / target).clamp(0.0, 1.0);
                if short <= 0.0 {
                    excluded.push((id, Reason::NotShort));
                    continue;
                }
                let t = match take() {
                    Ok(t) => t,
                    Err(why) => {
                        excluded.push((id, why));
                        continue;
                    }
                };
                if t.walk_min > f64::from(def.max_walk_minutes) {
                    excluded.push((id, Reason::Unreachable));
                    continue;
                }
                let half = w.trip_half_worth_days * f.household_kcal_day.max(1.0);
                let worth = t.kcal / (t.kcal + half.max(1.0));
                term(&mut terms, Reason::FoodShortage, w.w_food * short * worth);
                if !f.has_food {
                    term(&mut terms, Reason::Hunger, w.w_hunger * f.hunger * worth);
                }
                term(
                    &mut terms,
                    Reason::Walking,
                    -w.w_walk_hour * 2.0 * t.walk_min / 60.0,
                );
                term(&mut terms, Reason::Objection, -t.objection);
                term(&mut terms, Reason::Risk, -t.risk);
                term(&mut terms, Reason::Regard, -t.regard);
                let steps = vec![
                    Step::Walk { to: t.at },
                    Step::Work {
                        minutes: def.min_minutes.max(1),
                    },
                    Step::Walk { to: f.home },
                ];
                out.push(finish(id, Target::Household(t.household), terms, steps));
            }
            Behavior::Trade | Behavior::Fetch => {
                // Buy what is cheaper to get from a neighbour than to make or gather (research
                // 08-05 §1.4: a buyer compares the few sellers it knows by payment and walk), or,
                // by a report, from a seller in another settlement (M5b slice AP, ADR-0019 §2):
                // reached in daylight, home within the day.
                let fetching = def.behavior == Behavior::Fetch;
                let found = if fetching {
                    fetch()
                } else {
                    trade().ok_or(Reason::NoOffer)
                };
                let t = match found {
                    Ok(t) => t,
                    Err(why) => {
                        excluded.push((id, why));
                        continue;
                    }
                };
                if t.walk_min > f64::from(def.max_walk_minutes) {
                    excluded.push((id, Reason::Unreachable));
                    continue;
                }
                if fetching && f.daylight_left_min < t.walk_min {
                    excluded.push((id, Reason::NotInDark));
                    continue;
                }
                match t.worth {
                    TradeWorth::Tool { need } => {
                        term(&mut terms, Reason::Tools, w.w_tools * need.clamp(0.0, 1.0));
                    }
                    TradeWorth::Food { kcal } => {
                        let half = w.trip_half_worth_days * f.household_kcal_day.max(1.0);
                        let worth = kcal / (kcal + half.max(1.0));
                        let target = f.food_target_days.max(1e-6);
                        let short = (1.0 - f.food_days / target).clamp(0.0, 1.0);
                        term(&mut terms, Reason::FoodShortage, w.w_food * short * worth);
                        let lean = if f.food_outlook_days > 0.0 {
                            (1.0 - f.food_days / f.food_outlook_days).clamp(0.0, 1.0)
                        } else {
                            0.0
                        };
                        term(&mut terms, Reason::LeanSeason, w.w_lean * lean * worth);
                        if !f.has_food {
                            term(&mut terms, Reason::Hunger, w.w_hunger * f.hunger * worth);
                        }
                    }
                    TradeWorth::Goods { hours } => {
                        let worth = hours / (hours + WAGE_HALF_WORTH_H);
                        term(&mut terms, Reason::UsefulWork, w.w_work * worth);
                    }
                    TradeWorth::Sale { share } => {
                        term(
                            &mut terms,
                            Reason::ForSale,
                            w.w_tools * share.clamp(0.0, 1.0),
                        );
                    }
                }
                term(
                    &mut terms,
                    Reason::Walking,
                    -w.w_walk_hour * 2.0 * t.walk_min / 60.0,
                );
                let steps = vec![
                    Step::Walk { to: t.at },
                    Step::Work {
                        minutes: def.min_minutes.max(1),
                    },
                    Step::Walk { to: f.home },
                ];
                let target = if t.firm {
                    Target::Firm(t.seller)
                } else {
                    Target::Household(t.seller)
                };
                out.push(finish(id, target, terms, steps));
            }
            Behavior::Hire => {
                // Paid work at another household's workshop, for what the pay brings the
                // household (research 08-10 §1.3: a wage is worth it set against what the time
                // would do otherwise, which the other options weigh).
                let Some(j) = job() else {
                    excluded.push((id, Reason::NoWork));
                    continue;
                };
                if j.walk_min > f64::from(def.max_walk_minutes) {
                    excluded.push((id, Reason::Unreachable));
                    continue;
                }
                let minutes = j.minutes.min(f64::from(def.max_minutes)).max(1.0);
                match j.worth {
                    TradeWorth::Tool { need } => {
                        term(&mut terms, Reason::Wages, w.w_tools * need.clamp(0.0, 1.0));
                    }
                    TradeWorth::Food { kcal } => {
                        let half = w.trip_half_worth_days * f.household_kcal_day.max(1.0);
                        let worth = kcal / (kcal + half.max(1.0));
                        let target = f.food_target_days.max(1e-6);
                        let short = (1.0 - f.food_days / target).clamp(0.0, 1.0);
                        term(&mut terms, Reason::FoodShortage, w.w_food * short * worth);
                        let lean = if f.food_outlook_days > 0.0 {
                            (1.0 - f.food_days / f.food_outlook_days).clamp(0.0, 1.0)
                        } else {
                            0.0
                        };
                        term(&mut terms, Reason::LeanSeason, w.w_lean * lean * worth);
                        if !f.has_food {
                            term(&mut terms, Reason::Hunger, w.w_hunger * f.hunger * worth);
                        }
                        term(&mut terms, Reason::Wages, w.w_work * worth);
                    }
                    TradeWorth::Goods { hours } => {
                        let worth = hours / (hours + WAGE_HALF_WORTH_H);
                        term(&mut terms, Reason::Wages, w.w_work * worth);
                    }
                    TradeWorth::Sale { share } => {
                        term(&mut terms, Reason::Wages, w.w_tools * share.clamp(0.0, 1.0));
                    }
                }
                term(
                    &mut terms,
                    Reason::Walking,
                    -w.w_walk_hour * 2.0 * j.walk_min / 60.0,
                );
                term(
                    &mut terms,
                    Reason::Effort,
                    -w.w_effort * (def.par - 1.0).max(0.0) * minutes / 60.0 * f.sleep_pressure,
                );
                let steps = vec![
                    Step::Walk { to: j.at },
                    Step::Work {
                        minutes: minutes.round().max(1.0) as u32,
                    },
                    Step::Walk { to: f.home },
                ];
                out.push(finish(id, Target::Firm(j.firm), terms, steps));
            }
            Behavior::Try => {
                // Spare hours at home trying toward a technique that would answer a problem
                // there (ADR-0008 §3).
                let t = match trying() {
                    Ok(t) => t,
                    Err(why) => {
                        excluded.push((id, why));
                        continue;
                    }
                };
                let minutes = f64::from(def.min_minutes.max(1));
                term(&mut terms, Reason::Problem, t.points);
                term(
                    &mut terms,
                    Reason::Effort,
                    -w.w_effort * (def.par - 1.0).max(0.0) * minutes / 60.0 * f.sleep_pressure,
                );
                let mut steps = walk_home_first(f);
                steps.push(Step::Work {
                    minutes: minutes.round() as u32,
                });
                out.push(finish(id, Target::Technique(t.technique), terms, steps));
            }
            Behavior::Socialize => {
                let Some((settlement, hearth)) = f.hearth else {
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
                out.push(finish(id, Target::Hearth(settlement), terms, steps));
            }
            Behavior::Attend => {
                let (Some((settlement, hearth)), Some(g)) = (f.hearth, f.gathering) else {
                    excluded.push((id, Reason::NoGathering));
                    continue;
                };
                // A say in what the gathering decides, and the company of those who come.
                term(&mut terms, Reason::Gathering, g.points);
                term(
                    &mut terms,
                    Reason::Loneliness,
                    w.w_social * f.loneliness * f.evening.max(0.25),
                );
                let steps = vec![
                    Step::Walk { to: hearth },
                    Step::Work {
                        minutes: g.minutes.round().max(1.0) as u32,
                    },
                ];
                out.push(finish(id, Target::Hearth(settlement), terms, steps));
            }
            Behavior::Petition => {
                let (Some((settlement, hearth)), Some(g)) = (f.hearth, f.petition) else {
                    excluded.push((id, Reason::NoPetition));
                    continue;
                };
                // What joining the petition is worth to them, and the company of those who come.
                term(&mut terms, Reason::Petition, g.points);
                term(
                    &mut terms,
                    Reason::Loneliness,
                    w.w_social * f.loneliness * f.evening.max(0.25),
                );
                let steps = vec![
                    Step::Walk { to: hearth },
                    Step::Work {
                        minutes: g.minutes.round().max(1.0) as u32,
                    },
                ];
                out.push(finish(id, Target::Hearth(settlement), terms, steps));
            }
            Behavior::Visit => {
                // Company at another settlement's hearth (M5a slice AM): reached before dark,
                // home again the same day. Its company is worth what being alone makes it, as
                // at home, and what those they would see there are to them; the walk there and
                // back is its cost.
                let mut any = false;
                for v in visits() {
                    if f.daylight_left_min < v.walk_min {
                        continue;
                    }
                    any = true;
                    let mut terms = Vec::new();
                    term(
                        &mut terms,
                        Reason::Loneliness,
                        w.w_social * f.loneliness * f.evening.max(0.25),
                    );
                    term(&mut terms, Reason::Company, v.points);
                    term(
                        &mut terms,
                        Reason::Walking,
                        -w.w_walk_hour * 2.0 * v.walk_min / 60.0,
                    );
                    let steps = vec![
                        Step::Walk { to: v.hearth },
                        Step::Work {
                            minutes: def.min_minutes.max(1),
                        },
                        Step::Walk { to: f.home },
                    ];
                    out.push(finish(id, Target::Hearth(v.settlement), terms, steps));
                }
                if !any {
                    excluded.push((id, Reason::NoPlaceToVisit));
                }
            }
            Behavior::Watch => {
                // A round of the watch (M4b slice AC, ADR-0015 §6): a stand at each home on it,
                // the activity's least minutes each.
                let Some(round) = f.watch.as_ref() else {
                    excluded.push((id, Reason::NoWatch));
                    continue;
                };
                term(&mut terms, Reason::Duty, round.points);
                let stand = def.min_minutes.max(1);
                let mut steps = Vec::with_capacity(round.stops.len() * 2 + 1);
                for &at in &round.stops {
                    steps.push(Step::Walk { to: at });
                    steps.push(Step::Work { minutes: stand });
                }
                steps.push(Step::Walk { to: f.home });
                out.push(finish(id, Target::None, terms, steps));
            }
            Behavior::Carry => {
                // A payment an agreement owes (M5c slice AV): taken up where the store keeps it,
                // walked to the other hearth and handed over, home the same day. Nothing is said
                // of it to one who carries nothing.
                let Some(c) = f.carry else {
                    continue;
                };
                if f.daylight_left_min < 2.0 * c.walk_min + 2.0 * f64::from(def.min_minutes) {
                    excluded.push((id, Reason::NotInDark));
                    continue;
                }
                term(&mut terms, Reason::Duty, c.points);
                term(
                    &mut terms,
                    Reason::Walking,
                    -w.w_walk_hour * 2.0 * c.walk_min / 60.0,
                );
                let stand = def.min_minutes.max(1);
                let steps = vec![
                    Step::Walk { to: c.store },
                    Step::Work { minutes: stand },
                    Step::Walk { to: c.hearth },
                    Step::Work { minutes: stand },
                    Step::Walk { to: f.home },
                ];
                out.push(finish(id, Target::Hearth(c.settlement), terms, steps));
            }
            Behavior::Bridge => {
                // Work on the household's crossing (M5c slice AW, step two): as long as daylight
                // allows within the authored range, a shorter session only to finish it; each
                // hour of a capable adult's work worth the walking it saves the household.
                let Some(c) = f.crossing else {
                    continue;
                };
                let site = Site {
                    at: c.at,
                    walk_min: c.walk_min,
                    left_h: c.left_h,
                    saves_per_hour: c.saves_per_hour,
                    duty: c.duty,
                    full: c.full,
                };
                match session_at_site(def, w, f, &site, Reason::Crossing, &mut terms) {
                    Ok(steps) => out.push(finish(id, Target::Crossing(c.crossing), terms, steps)),
                    Err(why) => excluded.push((id, why)),
                }
            }
            Behavior::Well => {
                // Work on the household's well (M6a slice AY, step three), as on its crossing.
                let Some(c) = f.well else {
                    continue;
                };
                let site = Site {
                    at: c.at,
                    walk_min: c.walk_min,
                    left_h: c.left_h,
                    saves_per_hour: c.saves_per_hour,
                    duty: 0.0,
                    full: c.full,
                };
                match session_at_site(def, w, f, &site, Reason::Well, &mut terms) {
                    Ok(steps) => out.push(finish(id, Target::Well(c.well), terms, steps)),
                    Err(why) => excluded.push((id, why)),
                }
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
                    MakeWorth::Sale { share } => {
                        term(
                            &mut terms,
                            Reason::ForSale,
                            w.w_tools * share.clamp(0.0, 1.0),
                        );
                    }
                    MakeWorth::Preserve { kcal } => {
                        let half = w.trip_half_worth_days * f.household_kcal_day.max(1.0);
                        let worth = kcal / (kcal + half.max(1.0));
                        term(&mut terms, Reason::Spoiling, w.w_food * worth);
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
                let work = Step::Work {
                    minutes: minutes.round().max(1.0) as u32,
                };
                // At home, or at the workshop's building once it has one.
                let steps = match option.site {
                    Some((at, walk_min)) => {
                        term(
                            &mut terms,
                            Reason::Walking,
                            -w.w_walk_hour * 2.0 * walk_min / 60.0,
                        );
                        vec![Step::Walk { to: at }, work, Step::Walk { to: f.home }]
                    }
                    None => {
                        let mut steps = walk_home_first(f);
                        steps.push(work);
                        steps
                    }
                };
                // What is made to sell is made in the household's workshop, set up for it if it
                // has none yet (slice J).
                let target = match option.worth {
                    MakeWorth::Sale { .. } => option.firm.map_or(Target::NewFirm, Target::Firm),
                    _ => Target::Home,
                };
                out.push(finish(id, target, terms, steps));
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
            if def
                .tools
                .iter()
                .all(|&t| shop.held_tools.get(t).copied().unwrap_or(0.0) >= MIN_TOOL)
            {
                busy.push(id);
            }
        }
    }
    // Work done at a slower rate (by hand) is left out while the same work can be done faster
    // (with the tool for it): the same task in a field while the tool is free, and the same
    // resource gathered while the household holds the tool, even in another member's hands. A
    // ripe crop cannot wait for the sickle; wood can wait for the axe.
    let fastest = |way: Way| {
        let waiting = busy
            .iter()
            .copied()
            .filter(|_| matches!(way, Way::Gather(_)));
        out.iter()
            .map(|c| c.scored.def)
            .chain(waiting)
            .map(|d| &defs[usize::from(d)])
            .filter(|d| Way::of(d) == Some(way))
            .filter(|d| d.technique.is_none_or(|t| (shop.knows)(t)))
            .map(|d| d.rate)
            .fold(0.0, f64::max)
    };
    let slower: Vec<u16> = out
        .iter()
        .filter(|c| {
            let d = &defs[usize::from(c.scored.def)];
            Way::of(d).is_some_and(|way| d.rate < fastest(way))
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
            petition: None,
            hurt: false,
            ill: false,
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
            gathering: None,
            watch: None,
            carry: None,
            crossing: None,
            well: None,
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
            digs: None,
            tools,
            rate,
            par: 3.0,
            min_age_years: 10.0,
            max_age_years: 70.0,
            min_minutes: 30,
            max_minutes: 240,
            daylight_only: true,
            max_walk_minutes: 30,
            technique: None,
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
            held_tools: free,
            best_make: make,
            makes_tool,
            knows: &|_| true,
        };
        candidates(
            defs,
            &weights(),
            &facts(),
            &limits(),
            &|_| None,
            &|_| Ok(field()),
            None,
            &|| None,
            &|| None,
            &|| Err(Reason::NoReport),
            &|| None,
            Err(Reason::Built),
            &shop,
            &|| Err(Reason::NoProblem),
            &|| Err(Reason::WouldNotTake),
            &Vec::new,
        )
    }

    #[test]
    fn a_visit_is_worth_those_there_less_the_walk_and_needs_daylight_to_get_there() {
        let mut def = activity("visit", Behavior::Visit, Vec::new(), 1.0);
        def.daylight_only = false;
        def.min_minutes = 90;
        let defs = vec![def];
        let w = weights();
        let shop = Workshop {
            free_tools: &[],
            held_tools: &[],
            best_make: &|_, _| Err((Reason::NoMaterials, None)),
            makes_tool: &|_| false,
            knows: &|_| true,
        };
        let there = |walk_min: f64, points: f64| VisitOption {
            settlement: PermanentId::from_raw(9).expect("id"),
            hearth: (3000.0, 0.0),
            walk_min,
            points,
        };
        let run = |f: &Facts, options: Vec<VisitOption>| {
            candidates(
                &defs,
                &w,
                f,
                &limits(),
                &|_| None,
                &|_| Ok(field()),
                None,
                &|| None,
                &|| None,
                &|| Err(Reason::NoReport),
                &|| None,
                Err(Reason::Built),
                &shop,
                &|| Err(Reason::NoProblem),
                &|| Err(Reason::WouldNotTake),
                &|| options.clone(),
            )
        };
        let f = facts();
        let (cands, _) = run(&f, vec![there(60.0, 6.0)]);
        let c = &cands[0];
        assert_eq!(c.scored.target, Target::Hearth(there(0.0, 0.0).settlement));
        let term = |r: Reason| {
            c.scored
                .terms
                .iter()
                .find(|t| t.reason == r)
                .map(|t| t.points)
        };
        assert_eq!(term(Reason::Company), Some(6.0));
        // The walk there and back, an hour each way.
        assert_eq!(term(Reason::Walking), Some((-2.0 * w.w_walk_hour) as f32));
        // There, a session at the hearth, and home the same day.
        assert!(matches!(c.steps.last(), Some(Step::Walk { to }) if *to == f.home));
        // Not set out on when dark would fall before they got there, nor with nowhere to go.
        let dusk = Facts {
            daylight_left_min: 50.0,
            ..facts()
        };
        let (cands, excluded) = run(&dusk, vec![there(60.0, 6.0)]);
        assert!(cands.is_empty());
        assert!(excluded.contains(&(0, Reason::NoPlaceToVisit)));
        let (cands, excluded) = run(&f, Vec::new());
        assert!(cands.is_empty() && excluded.contains(&(0, Reason::NoPlaceToVisit)));
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
                firm: None,
                site: None,
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
    fn wood_is_cut_by_hand_only_in_a_household_without_an_axe() {
        // Good 0 is the axe. Cutting poles with it and cutting rods by hand gather the same wood
        // (resource 0), the second at a fifth of the rate.
        let gather = |id: &str, tools: Vec<usize>, rate: f64| ActivityDef {
            resource: Some(0),
            ..activity(id, Behavior::Gather, tools, rate)
        };
        let defs = vec![
            gather("cut_poles", vec![0], 1.0),
            gather("cut_rods", Vec::new(), 0.2),
        ];
        let wood = |_: usize| {
            Some(PatchOption {
                patch: 3,
                walk_min: 10.0,
                kg_per_hour: 20.0,
                purpose: GoodUse::Material,
                kcal_per_kg: 0.0,
                stored_days: 0.0,
                need_kg: 200.0,
                urgency: 1.0,
                tool_need_kg: 0.0,
                tool_blocked: false,
                at: (50.0, 0.0),
                deposit: None,
            })
        };
        let none = |_: usize| false;
        let nothing = |_: usize, _: &[usize]| Err((Reason::NoMaterials, None));
        let cut = |free: &[f64], held: &[f64]| {
            let shop = Workshop {
                free_tools: free,
                held_tools: held,
                best_make: &nothing,
                makes_tool: &none,
                knows: &|_| true,
            };
            candidates(
                &defs,
                &weights(),
                &facts(),
                &limits(),
                &wood,
                &|_| Ok(field()),
                None,
                &|| None,
                &|| None,
                &|| Err(Reason::NoReport),
                &|| None,
                Err(Reason::Built),
                &shop,
                &|| Err(Reason::NoProblem),
                &|| Err(Reason::WouldNotTake),
                &Vec::new,
            )
        };
        // With an axe free, wood is cut with it and not by hand.
        let (cands, excluded) = cut(&[1.0], &[1.0]);
        let chosen: Vec<u16> = cands.iter().map(|c| c.scored.def).collect();
        assert!(chosen.contains(&0) && !chosen.contains(&1), "{chosen:?}");
        assert!(excluded.contains(&(1, Reason::BetterWay)));
        // With the household's axe in another member's hands, the wood waits for it.
        let (cands, excluded) = cut(&[0.0], &[1.0]);
        assert!(cands.is_empty(), "{cands:?}");
        assert!(excluded.contains(&(0, Reason::NoTool)));
        assert!(excluded.contains(&(1, Reason::BetterWay)));
        // A household without one cuts rods by hand, and the axe is what the work waits on.
        let (cands, excluded) = cut(&[0.0], &[0.0]);
        let chosen: Vec<u16> = cands.iter().map(|c| c.scored.def).collect();
        assert!(chosen.contains(&1) && !chosen.contains(&0), "{chosen:?}");
        assert!(excluded.contains(&(0, Reason::NoTool)));
    }

    #[test]
    fn work_for_a_workshop_with_a_building_is_done_there() {
        // ADR-0009 §7: once its building has its roof on, a workshop's owners walk there to make
        // what it sells, and home again.
        let defs = vec![activity("make_sickle", Behavior::Make, Vec::new(), 1.0)];
        let firm = PermanentId::from_raw(7).expect("nonzero");
        let sell = |site: Option<((f32, f32), f64)>| {
            move |_: usize, _: &[usize]| {
                Ok(MakeOption {
                    units: 1.0,
                    minutes: 180.0,
                    worth: MakeWorth::Sale { share: 0.5 },
                    firm: Some(firm),
                    site,
                })
            }
        };
        let none = |_: usize| false;
        let (cands, _) = score(&defs, &[], &sell(Some(((10.0, 20.0), 6.0))), &none);
        let c = &cands[0];
        assert_eq!(c.scored.target, Target::Firm(firm));
        assert_eq!(
            c.steps,
            vec![
                Step::Walk { to: (10.0, 20.0) },
                Step::Work { minutes: 180 },
                Step::Walk { to: facts().home },
            ]
        );
        assert!(c.scored.terms.iter().any(|t| t.reason == Reason::Walking));
        // Without one, at home.
        let (cands, _) = score(&defs, &[], &sell(None), &none);
        assert_eq!(cands[0].steps, vec![Step::Work { minutes: 180 }]);
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
                    firm: None,
                    site: None,
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
    fn the_last_water_trip_before_dark_comes_before_a_meal_that_can_wait() {
        let mut eat = activity("eat", Behavior::Eat, Vec::new(), 1.0);
        eat.daylight_only = false;
        let mut fetch = activity("fetch_water", Behavior::FetchWater, Vec::new(), 1.0);
        fetch.min_minutes = 5;
        fetch.max_walk_minutes = 45;
        let defs = vec![eat, fetch];
        let water = WaterOption {
            cell: 1,
            walk_min: 5.0,
            at: (5.0, 0.0),
            well: None,
            lift_min: 0.0,
        };
        let cannot = |_: usize, _: &[usize]| Err((Reason::NoTool, None));
        let shop = Workshop {
            free_tools: &[],
            held_tools: &[],
            best_make: &cannot,
            makes_tool: &|_| false,
            knows: &|_| true,
        };
        // (eat, fetch water) totals for the hungriest of people.
        let totals = |daylight_left_min: f64, water_days: f64| {
            let f = Facts {
                hunger: 2.0,
                has_food: true,
                dark: false,
                daylight_left_min,
                until_sunrise_min: 800.0,
                water_days,
                ..facts()
            };
            let (cands, _) = candidates(
                &defs,
                &weights(),
                &f,
                &limits(),
                &|_| None,
                &|_| Err(Reason::NoPlace),
                Some(water),
                &|| None,
                &|| None,
                &|| Err(Reason::NoReport),
                &|| None,
                Err(Reason::Built),
                &shop,
                &|| Err(Reason::NoProblem),
                &|| Err(Reason::WouldNotTake),
                &Vec::new,
            );
            let total = |d: u16| {
                cands
                    .iter()
                    .find(|c| c.scored.def == d)
                    .map_or(f32::MIN, |c| c.scored.total)
            };
            (total(0), total(1))
        };
        // Half an hour of light, and water for five hours of a night of thirteen: the trip
        // first, since the meal can wait until dark.
        let (eat, water) = totals(30.0, 5.0 / 24.0);
        assert!(water > eat, "water {water} vs eat {eat}");
        // Two hours of light: the shortage is full, but there is time to eat first.
        let (eat, water) = totals(120.0, 5.0 / 24.0);
        assert!(eat > water, "water {water} vs eat {eat}");
        // Water enough for the night: no hurry.
        let (eat, water) = totals(30.0, 1.0);
        assert!(eat > water, "water {water} vs eat {eat}");
        // Water that runs out at first light, before the morning's meal and trip: as short.
        let (eat, water) = totals(30.0, 800.0 / 1440.0);
        assert!(water > eat, "water {water} vs eat {eat}");
    }

    #[test]
    fn work_on_a_well_is_weighed_by_the_walking_it_saves_and_its_lift_is_part_of_a_draw() {
        let mut dig = activity("work_on_well", Behavior::Well, Vec::new(), 1.0);
        dig.min_minutes = 60;
        dig.max_minutes = 240;
        dig.max_walk_minutes = 30;
        dig.par = 5.0;
        let mut fetch = activity("fetch_water", Behavior::FetchWater, Vec::new(), 1.0);
        fetch.min_minutes = 5;
        fetch.max_walk_minutes = 45;
        let defs = vec![dig, fetch];
        let cannot = |_: usize, _: &[usize]| Err((Reason::NoTool, None));
        let shop = Workshop {
            free_tools: &[],
            held_tools: &[],
            best_make: &cannot,
            makes_tool: &|_| false,
            knows: &|_| true,
        };
        let well = PermanentId::from_raw(9).expect("an id");
        let run = |f: &Facts, water: Option<WaterOption>| {
            candidates(
                &defs,
                &weights(),
                f,
                &limits(),
                &|_| None,
                &|_| Err(Reason::NoPlace),
                water,
                &|| None,
                &|| None,
                &|| Err(Reason::NoReport),
                &|| None,
                Err(Reason::Built),
                &shop,
                &|| Err(Reason::NoProblem),
                &|| Err(Reason::WouldNotTake),
                &Vec::new,
            )
        };
        // No well being dug: the work is no part of anyone's choice.
        let (cands, excluded) = run(&facts(), None);
        assert!(cands.iter().all(|c| c.scored.def != 0));
        assert!(excluded.iter().all(|e| e.0 != 0));
        // A well being dug beside home, with 20 hours left, each hour saving six of walking: a
        // full session, worth the walking it saves, at the well.
        let wf = WellFacts {
            well,
            at: (3.0, 0.0),
            walk_min: 0.1,
            left_h: 20.0,
            saves_per_hour: 6.0,
            full: false,
        };
        let f = Facts {
            well: Some(wf),
            dark: false,
            daylight_left_min: 600.0,
            ..facts()
        };
        let (cands, _) = run(&f, None);
        let c = cands.iter().find(|c| c.scored.def == 0).expect("weighed");
        assert_eq!(c.scored.target, Target::Well(well));
        assert!(matches!(c.steps[1], Step::Work { minutes: 240 }));
        let saving = c
            .scored
            .terms
            .iter()
            .find(|t| t.reason == Reason::Well)
            .expect("the walking saved");
        assert!((f64::from(saving.points) - weights().w_walk_hour * 4.0 * 6.0).abs() < 1e-3);
        // As many as its shaft has room for began today: left out, and why.
        let full = Facts {
            well: Some(WellFacts { full: true, ..wf }),
            ..f.clone()
        };
        let (cands, excluded) = run(&full, None);
        assert!(cands.iter().all(|c| c.scored.def != 0));
        assert!(excluded.contains(&(0, Reason::Crowded)));
        // Water drawn at a well is aimed at the well, and its lift lengthens the draw.
        let at_well = WaterOption {
            cell: 1,
            walk_min: 0.1,
            at: (3.0, 0.0),
            well: Some(well),
            lift_min: 2.4,
        };
        let thirsty = Facts {
            water_days: 0.0,
            ..f.clone()
        };
        let (cands, _) = run(&thirsty, Some(at_well));
        let c = cands.iter().find(|c| c.scored.def == 1).expect("fetching");
        assert_eq!(c.scored.target, Target::Well(well));
        assert!(
            matches!(c.steps[1], Step::Work { minutes: 7 }),
            "{:?}",
            c.steps
        );
    }

    #[test]
    fn equal_options_share_the_probability() {
        let w = weights();
        let (_, p, _) = choose(&[3.0, 3.0, 3.0, 3.0], &w, 0.3);
        assert!((p - 0.25).abs() < 1e-9);
    }

    #[test]
    fn a_leisure_block_is_the_run_of_sessions_detailed_mode_would_live() {
        // 30-minute sessions, at most 180 minutes, the next turn of the day 400 minutes away.
        let block = |p: f64, u: f64| leisure_block(30, 180, 400.0, p, u);
        // Chosen with probability one half: another session with chance one half each time.
        assert_eq!(block(0.5, 0.9), (30, true), "the first draw ends it");
        assert_eq!(block(0.5, 0.4), (60, true), "one more");
        assert_eq!(block(0.5, 0.2), (90, true), "two more");
        // Cut at the longest, or at the next turn: then the next decision is made afresh.
        assert_eq!(block(0.5, 1e-6), (180, false));
        assert_eq!(leisure_block(30, 180, 75.0, 0.5, 1e-6), (75, false));
        // Never shorter than a session; never chosen again, never more than one.
        assert_eq!(leisure_block(30, 180, 10.0, 0.9, 0.01), (30, false));
        assert_eq!(block(0.0, 0.01), (30, true));
        assert_eq!(block(1.0, 0.99), (180, false));
        // The mean run is a session over the chance of choosing something else, as in Detailed
        // mode: 1 / (1 - 0.75) = 4 sessions.
        let n = 10_000;
        let mean = (0..n)
            .map(|k| {
                f64::from(
                    leisure_block(30, 100_000, 1e9, 0.75, (f64::from(k) + 0.5) / f64::from(n)).0,
                )
            })
            .sum::<f64>()
            / f64::from(n);
        assert!((mean / 30.0 - 4.0).abs() < 0.05, "{mean}");
    }
}
