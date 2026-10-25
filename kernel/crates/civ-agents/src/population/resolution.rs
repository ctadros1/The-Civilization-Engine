//! A resolution per settlement (M9a slice DF, ADR-0032 §1, §3): which settlements are
//! household-resolved is a run setting, never saved. Their people are parked: an activity with no
//! steps standing at [`PARKED_STEP`], so nothing is scheduled for them, which a save keeps as it is. Parking and taking up
//! the day again happen at a midnight, and change nobody, no holding and no good.

use super::*;

/// The physical activity level of a household-resolved day, the same day's need [`Population`]'s
/// meals reckon with (a design prior until slice DG resolves the day's work).
const NEEDS_DAY_PAR: f64 = 1.6;

/// The step a parked activity stands at: past the end of no steps, which nothing else sets (an
/// activity restarted on load, or a newborn's or founder's first, has no steps and stands at 0).
pub const PARKED_STEP: u8 = u8::MAX;

/// The minutes after sunset a household-resolved day's people turn in: the middle of the four-hour
/// evening a decision reads (a design prior, NUDGE in PROJECT_PLAN §9: a Detailed day's bedtimes
/// spread over the evening as its choices do).
const TURN_IN_AFTER_SUNSET_MIN: i64 = 120;

/// Minutes of sleep a day: from turning in after sunset ([`TURN_IN_AFTER_SUNSET_MIN`]) to
/// sunrise, within the content's sleep range.
fn night_minutes(sleep: &crate::params::SleepParams, sun: (i64, i64)) -> f64 {
    let turn_in = (sun.1 + TURN_IN_AFTER_SUNSET_MIN).min(MINUTES_PER_DAY);
    let night = (MINUTES_PER_DAY - turn_in + sun.0) as f64;
    night.clamp(sleep.min_hours * 60.0, sleep.max_hours * 60.0)
}

/// Most trips for water a household makes in a household-resolved day (a design bound).
const MAX_WATER_TRIPS: usize = 8;

/// What a household-resolved day's planning found for one person (ADR-0032 §2).
#[derive(Debug)]
pub(super) struct Planned {
    /// The options as scored, after the technique gate and a curfew's cost.
    pub cands: Vec<decide::Candidate>,
    /// New ground the household would mark out.
    pub site: Option<Site>,
    /// Minutes a meal takes.
    pub meal_min: u32,
}

/// Where a household-resolved person's day puts them, block by block (ADR-0032 §6; M9b slice
/// DK): what the observer is shown of a day lived at midnight, by the clock. Recorded as the day
/// is lived, never read by the simulation and never saved.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DayBlock {
    /// The minute of the day it begins.
    pub from: u16,
    /// The minute of the day it ends.
    pub until: u16,
    /// Its activity, an index into the catalog's activities.
    pub def: u16,
    /// Where it is drawn: where its work is done, or home.
    pub at: (f32, f32),
    /// Whether it is the night's sleep.
    pub asleep: bool,
}

/// Where option `c`'s work is done: the end of the last walk before its first work, or `home`.
fn work_place(c: &decide::Candidate, home: (f32, f32)) -> (f32, f32) {
    let mut at = home;
    for s in &c.steps {
        match *s {
            Step::Walk { to } => at = to,
            Step::Work { .. } | Step::Wait { .. } => return at,
            _ => {}
        }
    }
    at
}

/// Behaviours a household-resolved day shares hours among (slice DG): the rest wait for slice DH
/// or are not taken while household-resolved (ADR-0032 §2).
const SHARED: [Behavior; 29] = [
    Behavior::Gather,
    Behavior::Farm,
    Behavior::Build,
    Behavior::Make,
    Behavior::Dig,
    Behavior::Try,
    Behavior::Rest,
    Behavior::Play,
    Behavior::Socialize,
    Behavior::Ask,
    Behavior::Trade,
    // Slice DH: work for others and the polity, the watch, care, taking, and the gatherings.
    Behavior::Hire,
    Behavior::Bridge,
    Behavior::Well,
    Behavior::Levee,
    Behavior::Ditch,
    Behavior::Palisade,
    Behavior::Way,
    Behavior::Raise,
    Behavior::Boat,
    Behavior::Cart,
    Behavior::Watch,
    Behavior::Light,
    Behavior::Tend,
    Behavior::Call,
    Behavior::Take,
    Behavior::Attend,
    Behavior::Poll,
    Behavior::Petition,
];

/// Behaviours never taken while household-resolved (ADR-0032 §2): journeys to another place
/// (a visit, a fetch, carrying dues, a band, grain to a mill) and the fire behaviours, as no fire
/// starts there and a burning settlement is held Detailed. Sleep, meals and water are lived apart
/// from the shares.
pub const NOT_TAKEN: [Behavior; 9] = [
    Behavior::Visit,
    Behavior::Fetch,
    Behavior::Carry,
    Behavior::Band,
    Behavior::Douse,
    Behavior::Strip,
    Behavior::Salvage,
    Behavior::Pull,
    Behavior::Mill,
];

/// Behaviours taken at most once a day: a claim on someone else's goods or work, a round of the
/// watch, or coming to a gathering, poll or petition.
const ONCE: [Behavior; 10] = [
    Behavior::Ask,
    Behavior::Trade,
    Behavior::Hire,
    Behavior::Take,
    Behavior::Watch,
    Behavior::Light,
    Behavior::Call,
    Behavior::Attend,
    Behavior::Poll,
    Behavior::Petition,
];

fn behavior_of(ctx: &Ctx, c: &decide::Candidate) -> Option<Behavior> {
    ctx.catalog
        .activities
        .get(usize::from(c.scored.def))
        .map(|a| a.behavior)
}

/// Minutes of work an option's steps hold.
fn work_minutes(c: &decide::Candidate) -> u32 {
    c.steps
        .iter()
        .map(|s| match s {
            Step::Work { minutes } | Step::Wait { minutes } => *minutes,
            _ => 0,
        })
        .sum()
}

/// Minutes one repetition of option `c` takes from `home`: its work and its walks. Its walks are
/// reckoned straight off trail from where each begins (an approximation the day's length only
/// weighs; the travel field gives a walk's time when slice DI measures it).
fn option_minutes(ctx: &Ctx, c: &decide::Candidate, home: (f32, f32)) -> f64 {
    let speed = ctx.params.nav.tobler_ms(0.0) * ctx.params.nav.offtrail_factor * 60.0;
    let mut at = home;
    let mut walk = 0.0;
    for s in &c.steps {
        if let Step::Walk { to } = *s {
            walk += f64::from((to.0 - at.0).hypot(to.1 - at.1)) / speed.max(1e-6);
            at = to;
        }
    }
    (f64::from(work_minutes(c)) + walk).max(1.0)
}

/// How many times a day each option is lived (ADR-0032 §2): each option is chosen with the
/// probability [`decide::choose`] would give it, and lasts its work and its walk there and back,
/// so over `waking` minutes it is chosen `waking · p / Σ p·T` times. Only [`SHARED`] behaviours
/// count, [`ONCE`] at most once, and an option that lays claim to new ground, a home or a
/// workshop once. Counts are rounded by largest remainder, ties to the earlier option.
fn shares(ctx: &Ctx, plan: &Planned, waking: f64, home: (f32, f32)) -> Vec<(usize, u32)> {
    // Options moved by a boat or a cart wait for the journeys they make (slice DH lists them); a
    // ride on a line's train is lived in the block (M10b slice DX, fourth part).
    let moved = |c: &decide::Candidate| {
        c.steps.iter().any(|s| {
            matches!(
                s,
                Step::Paddle { .. } | Step::Haul { .. } | Step::Handle { .. }
            )
        })
    };
    let idx: Vec<usize> = (0..plan.cands.len())
        .filter(|&i| {
            behavior_of(ctx, &plan.cands[i])
                .is_some_and(|b| SHARED.contains(&b) && !NOT_TAKEN.contains(&b))
                && !moved(&plan.cands[i])
        })
        .collect();
    if idx.is_empty() || waking <= 0.0 {
        return Vec::new();
    }
    let totals: Vec<f32> = idx.iter().map(|&i| plan.cands[i].scored.total).collect();
    let probs = decide::probabilities(&totals, &ctx.params.decision);
    let length = |i: usize| option_minutes(ctx, &plan.cands[i], home);
    let per: f64 = idx.iter().zip(&probs).map(|(&i, p)| p * length(i)).sum();
    if per <= 0.0 {
        return Vec::new();
    }
    let cap = |i: usize| {
        let c = &plan.cands[i];
        let once = behavior_of(ctx, c).is_some_and(|b| ONCE.contains(&b))
            || matches!(
                c.scored.target,
                Target::NewField | Target::NewBuilding | Target::NewFirm
            );
        if once { 1.0 } else { f64::INFINITY }
    };
    let want: Vec<f64> = idx
        .iter()
        .zip(&probs)
        .map(|(&i, p)| (waking * p / per).min(cap(i)))
        .collect();
    let mut counts: Vec<u32> = want.iter().map(|w| w.floor() as u32).collect();
    let total = want.iter().sum::<f64>().round() as u32;
    let mut order: Vec<usize> = (0..want.len()).collect();
    order.sort_by(|&a, &b| {
        let (ra, rb) = (want[a] - want[a].floor(), want[b] - want[b].floor());
        rb.total_cmp(&ra).then(a.cmp(&b))
    });
    let mut left = total.saturating_sub(counts.iter().sum());
    for k in order {
        if left == 0 {
            break;
        }
        if f64::from(counts[k]) + 1.0 <= cap(idx[k]) {
            counts[k] += 1;
            left -= 1;
        }
    }
    idx.into_iter()
        .zip(counts)
        .filter(|&(_, n)| n > 0)
        .collect()
}

/// What a switch of resolution must leave as it found it (ADR-0032 §3): who lives, in which
/// household, which households live where with whom, and who holds every field, building and
/// workshop. Goods are checked by the books ([`unaccounted`]).
#[derive(Clone, Debug, PartialEq)]
pub struct SwitchInvariants {
    /// Every living person and their household, by person.
    pub people: Vec<(PermanentId, PermanentId)>,
    /// Every household, its settlement and its members, by household.
    pub households: Vec<(PermanentId, Option<PermanentId>, Vec<PermanentId>)>,
    /// Every field, the household working it and its holder.
    pub fields: Vec<(PermanentId, PermanentId, Party)>,
    /// Every building and its household.
    pub buildings: Vec<(PermanentId, PermanentId)>,
    /// Every workshop and its owner.
    pub firms: Vec<(PermanentId, PermanentId)>,
}

impl Population {
    /// The settlements now household-resolved.
    pub fn household_resolved(&self) -> &BTreeSet<PermanentId> {
        &self.resolved
    }

    /// Sets the settlements household-resolved; the caller reconciles at a midnight
    /// ([`Self::reconcile_resolutions`]).
    pub fn set_household_resolved(&mut self, settlements: BTreeSet<PermanentId>) {
        self.resolved = settlements;
    }

    /// The block person `id`'s household-resolved day puts them in at `at` (ADR-0032 §6), if
    /// they are parked and `at` falls in the day last lived.
    pub fn day_block(&self, id: PermanentId, at: SimTime) -> Option<&DayBlock> {
        if at.day_index() != self.day_blocks_day {
            return None;
        }
        let m = at.minutes() - at.day_index() * MINUTES_PER_DAY;
        let parked = self.index.get(&id).is_some_and(|&h| self.parked(h));
        self.day_blocks
            .get(&id)
            .filter(|_| parked)?
            .iter()
            .find(|b| i64::from(b.from) <= m && m < i64::from(b.until))
    }

    /// The household-resolved settlements held Detailed for now (ADR-0032 §1).
    pub fn held_detailed(&self) -> &BTreeSet<PermanentId> {
        &self.held
    }

    /// Whether settlement `s` lives household-resolved now: set so, and not held.
    pub(super) fn resolved_settlement(&self, s: PermanentId) -> bool {
        self.resolved.contains(&s) && !self.held.contains(&s)
    }

    /// Whether person `h` lives in a settlement household-resolved now.
    pub(super) fn resolved_person(&self, h: Handle<Person>) -> bool {
        if self.resolved.is_empty() {
            return false;
        }
        self.people
            .get(h)
            .and_then(|p| self.household(p.household))
            .and_then(|x| x.settlement)
            .is_some_and(|s| self.resolved_settlement(s))
    }

    /// Works out which household-resolved settlements are held Detailed (ADR-0032 §1): any where
    /// a fire burns, and any a band was called from, or set off for, and is not yet home or
    /// dispersed; a band camped by a line or holding a place keeps both held.
    fn refresh_holds(&mut self) {
        self.held.clear();
        if self.resolved.is_empty() {
            return;
        }
        for f in &self.fires.list {
            if f.burning()
                && let Some(s) = f.settlement
            {
                self.held.insert(s);
            }
        }
        let mut bands = Vec::new();
        for b in &self.order.bands {
            if matches!(
                b.stage,
                crate::war::BandStage::Called
                    | crate::war::BandStage::Out
                    | crate::war::BandStage::Camped
                    | crate::war::BandStage::Holding
            ) {
                bands.push((b.settlement, b.target));
            }
        }
        for (from, target) in bands {
            self.held.insert(from);
            if let Some(s) = self.household(target).and_then(|x| x.settlement) {
                self.held.insert(s);
            }
        }
        self.held.retain(|s| self.resolved.contains(s));
    }

    /// The parked residents of settlement `s`, in id order: whom a Detailed visitor to its hearth
    /// meets while it is household-resolved (ADR-0032 §4).
    pub(super) fn parked_residents(&self, s: PermanentId) -> Vec<PermanentId> {
        let mut out: Vec<PermanentId> = self
            .households_in(s)
            .flat_map(|x| x.members.iter().copied())
            .filter(|&m| self.index.get(&m).is_some_and(|&h| self.parked(h)))
            .collect();
        out.sort_unstable();
        out
    }

    /// Whether person `h` is parked: their activity has no steps and stands at
    /// [`PARKED_STEP`], so nothing is scheduled for them.
    fn parked(&self, h: Handle<Person>) -> bool {
        self.people
            .get(h)
            .is_some_and(|p| p.act.steps.is_empty() && p.act.step == PARKED_STEP)
    }

    /// Parks person `h` (ADR-0032 §3): what they carry is banked, they stand at home, and their
    /// activity is replaced by one of no steps, so any event of the old one does nothing. Parking
    /// someone already parked changes nothing.
    pub(super) fn park(&mut self, ctx: &mut Ctx, h: Handle<Person>) {
        if self.parked(h) {
            return;
        }
        let laden = self
            .people
            .get(h)
            .is_some_and(|p| p.carrying.kg > 0.0 || p.carrying.water_l > 0.0);
        if laden {
            self.deposit(ctx, h);
        }
        let Some(home) = self
            .people
            .get(h)
            .and_then(|p| self.household(p.household))
            .map(|x| x.home)
        else {
            return;
        };
        let Some(p) = self.people.get_mut(h) else {
            return;
        };
        settle(p, ctx.now, ctx.params);
        p.trip = None;
        p.pos = home;
        p.act = Activity {
            def: p.act.def,
            target: Target::None,
            steps: Vec::new(),
            step: PARKED_STEP,
            started: ctx.now,
            step_started: ctx.now,
            step_ends: ctx.now,
            version: p.act.version.wrapping_add(1),
        };
    }

    /// Brings everyone's execution in line with the resolutions in force (ADR-0032 §3): those
    /// of household-resolved settlements are parked ([`Self::park_resolved`]), then anyone parked
    /// elsewhere takes up their day ([`Self::resume_unresolved`]).
    pub fn reconcile_resolutions(&mut self, ctx: &mut Ctx) {
        self.park_resolved(ctx);
        self.resume_unresolved(ctx);
    }

    /// Parks everyone of a household-resolved settlement not yet parked, in person-table order.
    /// It changes nobody, no holding and no good the books do not account for.
    pub fn park_resolved(&mut self, ctx: &mut Ctx) {
        self.refresh_holds();
        if self.resolved.is_empty() {
            return;
        }
        let handles: Vec<Handle<Person>> = self.people.iter().map(|(h, _)| h).collect();
        for h in handles {
            if self.resolved_person(h) {
                self.park(ctx, h);
            }
        }
    }

    /// Anyone parked outside a household-resolved settlement takes up their day with a first
    /// decision, as founders do, in person-table order.
    pub fn resume_unresolved(&mut self, ctx: &mut Ctx) {
        let handles: Vec<Handle<Person>> = self
            .people
            .iter()
            .filter(|(_, p)| p.act.steps.is_empty() && p.act.step == PARKED_STEP)
            .map(|(h, _)| h)
            .collect();
        for h in handles {
            if !self.resolved_person(h) {
                self.decide(ctx, h, 0);
            }
        }
    }

    /// The day of each household-resolved settlement, lived at the midnight that begins it
    /// (ADR-0032 §2). Each parked member lives the night asleep to sunrise and is scored there
    /// once, as at their first decision ([`Self::plan_day`]); they eat what the night and the day
    /// before used; the household fetches the water a day uses by the trip its members would
    /// take; and each member's waking hours are shared among what they would choose, each share a
    /// number of the chosen activity's repetitions, taken up and lived at once through its own
    /// steps' effects ([`Self::enact`]). Company at the hearth is drawn once a member-day from
    /// the settlement's pool of those who keep it. Effects land now, at midnight.
    pub fn live_household_days(&mut self, ctx: &mut Ctx) {
        self.day_blocks.clear();
        self.day_blocks_day = ctx.now.day_index();
        if self.resolved.is_empty() {
            return;
        }
        let (midnight, params) = (ctx.now, ctx.params);
        let doy = civ_land::day_of_year(midnight.day_index());
        let sun = needs::daylight(params.latitude_deg, doy);
        let sunrise = sun.0;
        let wake = midnight.plus_minutes(sunrise.clamp(0, MINUTES_PER_DAY - 1));
        let sleep_par = ctx
            .catalog
            .activities
            .iter()
            .find(|a| a.behavior == Behavior::Sleep)
            .map_or(1.0, |a| a.par);
        // The night runs from turning in after sunset to sunrise.
        let sleep_min = night_minutes(&params.sleep, sun);
        let turn_in = (sun.1 + TURN_IN_AFTER_SUNSET_MIN).min(MINUTES_PER_DAY);
        let def_of = |b: Behavior| {
            ctx.catalog
                .activities
                .iter()
                .position(|a| a.behavior == b)
                .map_or(0, |i| i as u16)
        };
        let (sleep_def, eat_def) = (def_of(Behavior::Sleep), def_of(Behavior::Eat));
        let minute = |m: i64| m.clamp(0, MINUTES_PER_DAY) as u16;
        let settlements: Vec<PermanentId> = self
            .resolved
            .iter()
            .copied()
            .filter(|s| !self.held.contains(s))
            .collect();
        self.resolving = true;
        for s in settlements {
            let mut company: Vec<(PermanentId, u32, u32)> = Vec::new();
            let households: Vec<PermanentId> = self.households_in(s).map(|x| x.id).collect();
            for id in households {
                let Some(hh) = self.household(id).cloned() else {
                    continue;
                };
                let mut water_cand: Option<(Handle<Person>, decide::Candidate)> = None;
                for &m in &hh.members {
                    let Some(&h) = self.index.get(&m) else {
                        continue;
                    };
                    if !self.parked(h) {
                        continue;
                    }
                    // The night, asleep to sunrise.
                    if let Some(p) = self.people.get_mut(h) {
                        settle(p, midnight, params);
                        p.asleep = true;
                        p.burn_kcal_min = burn_rate(p, midnight, params, sleep_par);
                        settle(p, wake, params);
                        p.asleep = false;
                        p.sleep_pressure = 0.0;
                    }
                    let plan = self.plan_day(ctx, h, wake);
                    // The afternoon is weighed as it stands at midday, the evening an hour after
                    // sunset, in the dark (ADR-0032 §2, amended): what they weigh changes nobody's
                    // needs.
                    let noon = (sun.0 + sun.1) / 2;
                    let afternoon = self.plan_evening(ctx, h, midnight.plus_minutes(noon));
                    let evening = self.plan_evening(ctx, h, midnight.plus_minutes(sun.1 + 60));
                    self.eat(ctx, h, id);
                    let Some(plan) = plan else {
                        continue;
                    };
                    let meals = 3.0 * f64::from(plan.meal_min);
                    // Daylight less the meals is the morning's to share; turning in less sunset the
                    // evening's.
                    let daylight = ((sun.1 - sun.0) as f64 - meals).max(0.0);
                    let evening_min =
                        (MINUTES_PER_DAY as f64 - sleep_min - meals - daylight).max(0.0);
                    if water_cand.is_none()
                        && let Some(c) = plan
                            .cands
                            .iter()
                            .find(|c| behavior_of(ctx, c) == Some(Behavior::FetchWater))
                    {
                        water_cand = Some((h, c.clone()));
                    }
                    // The night and the meals, as time use counts them (the shares fill the rest).
                    self.count_time(Behavior::Sleep, sleep_min);
                    self.count_time(Behavior::Eat, meals);
                    // The day's energy: each block at its activity's level, rest for the rest.
                    let rest_par = ctx
                        .catalog
                        .activities
                        .iter()
                        .find(|a| a.behavior == Behavior::Rest)
                        .map_or(NEEDS_DAY_PAR, |a| a.par);
                    let (mut par_minutes, mut active) = (0.0, 0.0);
                    // Each half of the daylight is weighed at its start, the evening in the dark.
                    let parts = [
                        Some((&plan, daylight / 2.0, sunrise, noon)),
                        afternoon.as_ref().map(|a| (a, daylight / 2.0, noon, sun.1)),
                        evening.as_ref().map(|e| (e, evening_min, sun.1, turn_in)),
                    ];
                    // What the observer is shown: the night to sunrise, each part's blocks laid
                    // end to end from its start, a meal at home in what is left of it, and the
                    // night from turning in.
                    let night = |from: i64, until: i64| DayBlock {
                        from: minute(from),
                        until: minute(until),
                        def: sleep_def,
                        at: hh.home,
                        asleep: true,
                    };
                    let mut blocks = vec![night(0, sunrise)];
                    for (part, minutes_to_share, from, to) in parts.into_iter().flatten() {
                        let mut t = from;
                        for (i, reps) in shares(ctx, part, minutes_to_share, hh.home) {
                            let cand = &part.cands[i];
                            let lasts = option_minutes(ctx, cand, hh.home) * f64::from(reps);
                            let start = t;
                            let end = (t + lasts.round() as i64).min(to);
                            if end > t {
                                blocks.push(DayBlock {
                                    from: minute(t),
                                    until: minute(end),
                                    def: cand.scored.def,
                                    at: work_place(cand, hh.home),
                                    asleep: false,
                                });
                                t = end;
                            }
                            let minutes = f64::from(work_minutes(cand) * reps);
                            let par = ctx
                                .catalog
                                .activities
                                .get(usize::from(cand.scored.def))
                                .map_or(rest_par, |a| a.par);
                            par_minutes += par * minutes;
                            active += minutes;
                            if behavior_of(ctx, cand) == Some(Behavior::Socialize) {
                                let minutes = work_minutes(cand) * reps;
                                let session = ctx
                                    .catalog
                                    .activities
                                    .get(usize::from(cand.scored.def))
                                    .map_or(minutes, |a| a.min_minutes.max(1));
                                company.push((m, minutes, (minutes / session.max(1)).max(1)));
                            }
                            // Each repetition is enacted now, its making's dark minutes
                            // counted to when it would have ended in the day.
                            let each = lasts / f64::from(reps.max(1));
                            for k in 0..reps {
                                let ends = start + (each * f64::from(k + 1)).round() as i64;
                                self.enact_end = Some(midnight.plus_minutes(ends));
                                self.enact(ctx, h, cand, part.site, &hh);
                            }
                            self.enact_end = None;
                        }
                        if t < to {
                            blocks.push(DayBlock {
                                from: minute(t),
                                until: minute(to),
                                def: eat_def,
                                at: hh.home,
                                asleep: false,
                            });
                        }
                    }
                    blocks.push(night(turn_in, MINUTES_PER_DAY));
                    self.day_blocks.insert(m, blocks);
                    let awake = (MINUTES_PER_DAY - sunrise).max(1) as f64;
                    let day_par =
                        (par_minutes + (awake - active).max(0.0) * rest_par) / awake.max(active);
                    if let Some(p) = self.people.get_mut(h) {
                        p.burn_kcal_min = burn_rate(p, wake, params, day_par);
                    }
                }
                // Water the household uses in a day, by the trip a member would make for it.
                if let Some((h, cand)) = water_cand {
                    for _ in 0..MAX_WATER_TRIPS {
                        let short = self.household(id).map_or(0.0, |x| {
                            let day =
                                x.members.len().max(1) as f64 * x.water_use(&params.household);
                            day - x.water_at_time(midnight, day)
                        });
                        if short <= 0.0 {
                            break;
                        }
                        self.enact(ctx, h, &cand, None, &hh);
                    }
                }
            }
            // The works' machines whose crews came today run (M10a slice DT).
            self.run_held_crews(ctx);
            // Company at the hearth: each who kept it meets those of the pool (ADR-0032 §2).
            let mut pool: Vec<PermanentId> = company.iter().map(|c| c.0).collect();
            pool.sort_unstable();
            pool.dedup();
            for (me, minutes, sessions) in company {
                self.keep_company(ctx, me, &pool, minutes, sessions);
            }
        }
        self.resolving = false;
    }

    /// Person `h` scored as they would stand at `at`, in the evening, with their needs then put
    /// back as they were: the evening's weighing changes nothing it only weighs.
    fn plan_evening(&mut self, ctx: &mut Ctx, h: Handle<Person>, at: SimTime) -> Option<Planned> {
        let kept = self.people.get(h).map(|p| {
            (
                p.energy_kcal,
                p.sleep_pressure,
                p.relatedness,
                p.needs_at,
                p.asleep,
            )
        })?;
        let plan = self.plan_day(ctx, h, at);
        if let Some(p) = self.people.get_mut(h) {
            (
                p.energy_kcal,
                p.sleep_pressure,
                p.relatedness,
                p.needs_at,
                p.asleep,
            ) = kept;
        }
        plan
    }

    /// Minutes of `b` counted in time use, as an event-driven day counts them at a step's end.
    fn count_time(&mut self, b: Behavior, minutes: f64) {
        if let Some(i) = Behavior::ALL.iter().position(|&x| x == b)
            && let Some(m) = self.time_use.work.get_mut(i)
        {
            *m += minutes;
        }
    }

    /// Person `h` scored as at their first decision of the day at `wake` (ADR-0032 §2): what
    /// [`Self::decide`] would weigh, with nothing chosen, drawn or begun.
    fn plan_day(&mut self, ctx: &mut Ctx, h: Handle<Person>, wake: SimTime) -> Option<Planned> {
        let at = ctx.now;
        ctx.now = wake;
        self.planning = true;
        self.decide(ctx, h, 0);
        self.planning = false;
        ctx.now = at;
        self.planned.take()
    }

    /// Takes up `cand` for person `h` and lives it at once through its steps: walks are walked
    /// along their routes and end where they lead, work has its effect as at a step's end, loads
    /// are banked; the person is parked again after.
    fn enact(
        &mut self,
        ctx: &mut Ctx,
        h: Handle<Person>,
        cand: &decide::Candidate,
        site: Option<Site>,
        hh: &Household,
    ) {
        self.take_up(ctx, h, cand, site, hh, None, None);
        let Some(version) = self.people.get(h).map(|p| p.act.version) else {
            return;
        };
        let mut i = 0usize;
        loop {
            let Some(p) = self.people.get_mut(h) else {
                return;
            };
            if p.act.version != version {
                break;
            }
            let Some(step) = p.act.steps.get(i).copied() else {
                break;
            };
            p.act.step = i as u8;
            match step {
                // A walk is planned and walked as an event-driven one would be: its route
                // found (cached), the ground and ways along it worn, deposits looked for and the
                // trip counted; what it schedules is spent when the person is parked again.
                Step::Walk { to } => {
                    if !self.start_walk(ctx, h, to) {
                        break;
                    }
                    // The walk took what it was planned to: its step is dated so.
                    if let Some(p) = self.people.get_mut(h) {
                        let took = (p.act.step_ends.minutes() - ctx.now.minutes()).max(0);
                        p.act.step_started = ctx.now.plus_minutes(-took);
                    }
                    self.end_step(ctx, h);
                }
                Step::Work { minutes } => {
                    // The work took its minutes: its step is dated so, as time use and practice
                    // read it.
                    p.act.step_started = ctx.now.plus_minutes(-i64::from(minutes));
                    // Coming to a gathering, a poll or a petition is counted as an arrival there
                    // is (ADR-0032 §2).
                    let me = p.id;
                    match behavior_of(ctx, cand) {
                        Some(Behavior::Attend) => self.attend(ctx, me),
                        Some(Behavior::Poll) => self.cast(ctx, me),
                        Some(Behavior::Petition) => self.join_petition(ctx, me),
                        _ => {}
                    }
                    self.end_step(ctx, h);
                }
                Step::Deposit => self.deposit(ctx, h),
                // A ride (M10b slice DX, fourth part) takes its seat and fare on a run of the day's
                // timetable and ends at the line's end; with no train to take, it is left out and
                // the walk after it goes on from where the person stands.
                Step::Ride { to } => {
                    if !self.ride_in_block(ctx, h, to) {
                        if let Some(p) = self.people.get_mut(h) {
                            p.act.steps.remove(i);
                        }
                        continue;
                    }
                }
                Step::Wait { .. }
                | Step::Paddle { .. }
                | Step::Haul { .. }
                | Step::Handle { .. } => {}
            }
            i += 1;
        }
        self.park(ctx, h);
    }
    /// What a switch of resolution must leave unchanged (ADR-0032 §3).
    pub fn switch_invariants(&self, land: &Land) -> SwitchInvariants {
        let mut people: Vec<_> = self
            .people
            .iter()
            .map(|(_, p)| (p.id, p.household))
            .collect();
        people.sort_unstable();
        let mut households: Vec<_> = self
            .households
            .iter()
            .map(|(_, x)| {
                let mut m = x.members.clone();
                m.sort_unstable();
                (x.id, x.settlement, m)
            })
            .collect();
        households.sort_unstable_by_key(|x| x.0);
        SwitchInvariants {
            people,
            households,
            fields: land
                .fields
                .iter()
                .map(|f| (f.id, f.household, f.holder))
                .collect(),
            buildings: land.buildings.iter().map(|b| (b.id, b.household)).collect(),
            firms: self.firms.iter().map(|f| (f.id, f.owner)).collect(),
        }
    }

    /// Anyone whose execution does not match their settlement's resolution: someone of a
    /// household-resolved settlement who is not parked, or someone parked elsewhere. Empty after
    /// every midnight (ADR-0032 §3).
    pub fn resolution_problems(&self) -> Vec<String> {
        self.people
            .iter()
            .filter_map(|(h, p)| match (self.resolved_person(h), self.parked(h)) {
                (true, false) => Some(format!(
                    "{} lives in a household-resolved settlement but is not parked",
                    p.id.get()
                )),
                (false, true) => Some(format!(
                    "{} is parked outside a household-resolved settlement",
                    p.id.get()
                )),
                _ => None,
            })
            .collect()
    }
}

/// Behaviours a household-resolved day lives apart from the shares: the night, meals and water.
#[cfg(test)]
const APART: [Behavior; 3] = [Behavior::Sleep, Behavior::Eat, Behavior::FetchWater];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_behaviour_is_shared_not_taken_or_lived_apart_and_only_one_of_them() {
        // ADR-0032 §2: each behaviour added names how a household-resolved day takes it.
        for b in Behavior::ALL {
            let n = usize::from(SHARED.contains(&b))
                + usize::from(NOT_TAKEN.contains(&b))
                + usize::from(APART.contains(&b));
            assert_eq!(n, 1, "{b:?} is in {n} of shared, not taken and lived apart");
        }
        for b in ONCE {
            assert!(SHARED.contains(&b), "{b:?} is once a day but not shared");
        }
    }
}
