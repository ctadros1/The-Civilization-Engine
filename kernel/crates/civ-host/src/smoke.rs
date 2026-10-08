//! Smoke seeds (plan §4.7): a few worlds per preset, checked against fixed thresholds. No
//! statistics; any failure blocks the milestone.
//!
//! The checks cover terrain and water, the save round trip, and a month of the founding band's
//! life: they settle, keep water and firewood at home, still have food, break ground and sow
//! their first field (unless the weather kept them off the ground), and nobody is ever stuck
//! between events.
//!
//! With `years`, each world then lives on for years (the M1 sanity run, plan §4.7), checked at
//! every year's end: nobody stuck, no population or land problems, every good held accounted for
//! by what came in and went out (ADR-0006 §3), no population explosion,
//! households under a roof from the second year, and a first trail worn out of the settlement
//! within the first. A band may fail and leave its valley, as foundings did (research 05-06 §5.2);
//! across the worlds at least half the bands must still live where they settled. Runs differ from
//! one to the next (determinism is not a goal), so the thresholds leave room for chance.
//!
//! Every year's end also checks that the techniques every founder brings and children learn in
//! upbringing are known by nearly everyone old enough for them (ADR-0008 §4); a technique lost
//! where people still live, and every find, is noted.
//!
//! The worlds take the content's property regimes in turn by seed, and a long run checks their
//! economies too ([`crate::economy`]): land claims, fields and wealth measures at each year's end,
//! and at the run's end, graded, whether food stocks rise with the harvest, whether grain is asked
//! more for before it than after, how unequal goods are and how workshop sizes spread.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use civ_content::ContentRegistry;
use civ_core::time::{MINUTES_PER_DAY, SimTime};
use civ_schema::SAVE_EXTENSION;
use civ_sim::{NewWorld, Sim, persist};
use civ_world::MapStats;
use commons_persist::{SaveDir, SaveKind};

use crate::economy::{Check, Economy, Grade};

/// Least elevation range, metres.
pub const MIN_RELIEF_M: f32 = 20.0;
/// Least share of the map that is not ocean.
pub const MIN_NOT_OCEAN: f32 = 0.5;
/// Least share of dry land with slopes under 5 %.
pub const MIN_GENTLE: f32 = 0.05;
/// Least total river length, km.
pub const MIN_RIVER_KM: f32 = 0.5;
/// In-game days the founding band lives.
pub const DAYS: i64 = 30;
/// Days of its first month, and of the sowing window within it, the ground must have been
/// workable on the valley floor for the band to be held to a sown field (ADR-0012 §5: ground
/// frozen, under snow or sodden most of a spring is weather, not a failure).
pub const WORKABLE_DAYS: (usize, usize) = (15, 3);
/// Most people a band may grow to, as a multiple of the founding band, in the long run.
pub const MAX_GROWTH: f64 = 3.0;
/// People a band must keep to count as still living where it settled, in the long run.
pub const MIN_ALIVE: usize = 10;
/// Share of the households, from the second year on, that must live under a roof.
pub const MIN_ROOFED: f64 = 0.8;
/// Least share of the living old enough for a technique every founder brings and children learn
/// in upbringing who know it, at each year's end (ADR-0008 §4).
pub const MIN_UPBRINGING: f64 = 0.95;

/// What to run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SmokeOptions {
    /// Cells per side of each world.
    pub size: u32,
    /// Seeds per preset, starting at 1.
    pub seeds: u64,
    /// Years each world lives on after its first month (0: the first month only).
    pub years: u32,
}

/// One world's result.
#[derive(Clone, Debug)]
pub struct SmokeResult {
    /// Preset id.
    pub preset: String,
    /// Seed.
    pub seed: u64,
    /// Time to generate, save, load and run.
    pub elapsed: Duration,
    /// The world's figures, when it was generated.
    pub stats: Option<MapStats>,
    /// Every threshold it missed; empty means it passed.
    pub failures: Vec<String>,
    /// People living at the end, when it lived on for years.
    pub living: Option<usize>,
    /// What else happened that a reader should know (a band that left its valley).
    pub notes: Vec<String>,
    /// The property regime it lived under.
    pub regime: String,
    /// How its economy was graded, when it lived on for years.
    pub economy: Vec<Check>,
}

fn check_world(stats: &MapStats) -> Vec<String> {
    let mut failures = Vec::new();
    let relief = stats.max_elevation_m - stats.min_elevation_m;
    if relief < MIN_RELIEF_M {
        failures.push(format!("relief {relief:.1} m < {MIN_RELIEF_M} m"));
    }
    let not_ocean = 1.0 - stats.ocean_fraction;
    if not_ocean < MIN_NOT_OCEAN {
        failures.push(format!(
            "land {:.0}% < {:.0}%",
            not_ocean * 100.0,
            MIN_NOT_OCEAN * 100.0
        ));
    }
    if stats.gentle_land_fraction < MIN_GENTLE {
        failures.push(format!(
            "gentle land {:.1}% < {:.0}%",
            stats.gentle_land_fraction * 100.0,
            MIN_GENTLE * 100.0
        ));
    }
    if stats.river_length_km < MIN_RIVER_KM {
        failures.push(format!(
            "rivers {:.2} km < {MIN_RIVER_KM} km",
            stats.river_length_km
        ));
    }
    failures
}

/// Saves, reloads and saves again; the reloaded map must equal the original and the second
/// save's raw section digests must equal the first's (ADR-0002 §6).
fn check_round_trip(sim: &mut Sim, content: &ContentRegistry) -> Result<(), String> {
    let temp = tempfile::tempdir().map_err(|e| format!("no temp folder: {e}"))?;
    let first_dir =
        SaveDir::create(temp.path().join("a"), SAVE_EXTENSION).map_err(|e| e.to_string())?;
    let second_dir =
        SaveDir::create(temp.path().join("b"), SAVE_EXTENSION).map_err(|e| e.to_string())?;
    let first = persist::save(sim, &first_dir, SaveKind::Manual, "smoke")
        .map_err(|e| format!("save failed: {e}"))?;
    let mut loaded =
        persist::load(&first.path, content).map_err(|e| format!("load failed: {e}"))?;
    if **loaded.map() != **sim.map() {
        return Err("the reloaded map differs".to_owned());
    }
    let second = persist::save(&mut loaded, &second_dir, SaveKind::Manual, "smoke")
        .map_err(|e| format!("second save failed: {e}"))?;
    let digests = |chunks: &[commons_persist::ChunkInfo]| {
        chunks
            .iter()
            .map(|c| (c.tag, c.index, c.raw_digest))
            .collect::<Vec<_>>()
    };
    if digests(&first.chunks) != digests(&second.chunks) {
        return Err("a section changed in the save → load → save round trip".to_owned());
    }
    Ok(())
}

/// Days from the world's founding to now on which the ground could be worked on the valley floor,
/// and those of them in its crop's sowing window: the world's weather lived again from its seed.
fn workable_days(sim: &Sim, content: &ContentRegistry) -> (usize, usize) {
    let rules = sim.rules();
    let (Some(preset), Some(crop)) = (
        content.preset(&sim.meta().preset_id),
        rules.catalog.crops.get(rules.people.farm.crop),
    ) else {
        return (0, 0);
    };
    let founded = civ_core::time::DEFAULT_WORLD_START.day_index();
    let window = i64::from(crop.sow_from_day)..=i64::from(crop.sow_until_day);
    let (mut month, mut sowing) = (0, 0);
    crate::weather::replay(
        content,
        preset,
        sim.meta().seed,
        sim.now().day_index() - 1,
        |day, _, why| {
            if day >= founded && why.is_none() {
                month += 1;
                if window.contains(&civ_land::day_of_year(day)) {
                    sowing += 1;
                }
            }
        },
    );
    (month, sowing)
}

/// Lives the founding band's first month and checks what must always hold.
pub(crate) fn check_people(sim: &mut Sim, content: &ContentRegistry) -> Vec<String> {
    let mut failures = Vec::new();
    if let Some(why) = sim.founding_problem() {
        return vec![format!("the founding band could not settle: {why}")];
    }
    let band = content.people.params.band.default_size as usize;
    if sim.people().living() != band {
        failures.push(format!(
            "{} people arrived instead of {band}",
            sim.people().living()
        ));
    }
    match sim.advance_minutes(DAYS * MINUTES_PER_DAY) {
        Ok(advance) if i64::from(advance.days) == DAYS => {}
        Ok(advance) => failures.push(format!(
            "the clock crossed {} day boundaries in {DAYS} days",
            advance.days
        )),
        Err(e) => failures.push(format!("the clock stopped: {e}")),
    }
    let now = sim.now();
    let stuck = sim
        .people()
        .people
        .iter()
        .filter(|(_, p)| p.act.step_ends < now)
        .count();
    if stuck > 0 {
        failures.push(format!("{stuck} people are stuck between events"));
    }
    let problems = sim
        .people()
        .problems(sim.ids().peek_next(), sim.rules().catalog.activities.len());
    if let Some(first) = problems.first() {
        failures.push(format!(
            "{} population problems, first: {first}",
            problems.len()
        ));
    }
    let homeless = sim
        .people()
        .households
        .iter()
        .filter(|(_, h)| {
            let cell = civ_agents::population::cell_of(sim.map(), h.home);
            !sim.nav().walkable(cell)
        })
        .count();
    if homeless > 0 {
        failures.push(format!("{homeless} homes stand where nobody can walk"));
    }
    let per_person = content.people.params.household.water_l_per_person_day;
    let dry = sim
        .people()
        .households
        .iter()
        .filter(|(_, h)| h.water_at_time(now, h.members.len() as f64 * per_person) <= 0.0)
        .count();
    if dry > 0 {
        failures.push(format!("{dry} households have no water"));
    }
    let rules = sim.rules();
    let goods = &rules.catalog.goods;
    let cold = sim
        .people()
        .households
        .iter()
        .filter(|(_, h)| {
            let stores = civ_agents::population::stores_now(h, now, &rules.people, goods);
            civ_agents::person::fuel_kg(&stores, goods) <= 0.0
        })
        .count();
    if cold * 2 > sim.people().households.len() {
        failures.push(format!(
            "{cold} of {} households have no firewood",
            sim.people().households.len()
        ));
    }
    // The band arrives with stores for months: a month in, no settlement may be short.
    for s in &sim.land().settlements {
        if s.food_short {
            failures.push(format!("{} ran short of food within {DAYS} days", s.name));
        }
    }
    // Arriving on the first of March, they break ground and sow once the window opens and the
    // ground can be worked.
    let sown = sim
        .land()
        .fields
        .iter()
        .filter(|f| f.stage == civ_land::FieldStage::Sown)
        .count();
    if sown == 0 {
        let (month, window) = workable_days(sim, content);
        if month >= WORKABLE_DAYS.0 && window >= WORKABLE_DAYS.1 {
            failures.push(format!(
                "no field sown within {DAYS} days ({} marked out), though the ground could be \
                 worked on {month} of them, {window} in the sowing window",
                sim.land().fields.len()
            ));
        }
    }
    // In their first weeks most households claim ground for their homes and begin their huts,
    // on ground nobody else has claimed. A household short of hands may put its fields first and
    // build after sowing.
    let land = sim.land();
    let households = sim.people().households.len();
    let begun = sim
        .people()
        .households
        .iter()
        .filter(|(_, h)| land.buildings.iter().any(|b| b.household == h.id))
        .count();
    if begun * 2 < households {
        failures.push(format!(
            "only {begun} of {households} households began a home within {DAYS} days"
        ));
    }
    let crowded = land
        .plots
        .iter()
        .enumerate()
        .filter(|(i, p)| {
            land.plots[i + 1..].iter().any(|q| p.rect.near(&q.rect, 0))
                || land.fields.iter().any(|f| p.rect.near(&f.rect, 0))
        })
        .count();
    if crowded > 0 {
        failures.push(format!("{crowded} plots overlap another plot or a field"));
    }
    let problems = land.problems(
        sim.map(),
        sim.rules().land.habitats.len(),
        sim.ids().peek_next(),
    );
    if let Some(first) = problems.first() {
        failures.push(format!("{} land problems, first: {first}", problems.len()));
    }
    failures
}

/// Lives a world on for `years` after its first month, checking it at every year's end: one
/// year, two years and so on from the founding.
/// What a long run carries from year to year and checks at each year's end, whatever its length:
/// the smoke's years and the dashboard's fifty (plan §4.7).
#[derive(Debug)]
pub struct LongRun {
    /// The people when the run began.
    pub founders: usize,
    /// The books when the run began: every good held is accounted for from here on (ADR-0006 §3).
    books: (Vec<f64>, civ_agents::person::Flows),
    /// What the run has seen of the economy, sampled as each month begins.
    pub economy: Economy,
    /// Months lived-in buildings have stood, counted as each month begins: their standing
    /// buildings, finished, of households with someone living.
    pub lived_building_months: u64,
}

impl LongRun {
    /// Begins a long run of `sim` as it stands.
    pub fn begin(sim: &Sim) -> LongRun {
        LongRun {
            founders: sim.people().living().max(1),
            books: (sim.people().goods_held(), sim.people().flows()),
            economy: Economy::default(),
            lived_building_months: 0,
        }
    }

    /// Lives `sim` to `end` month by month, sampling the economy as each month begins.
    pub fn live_to(&mut self, sim: &mut Sim, end: i64) -> Result<(), String> {
        loop {
            let d = sim.now().date();
            let (ny, nm) = if d.month == 12 {
                (d.year + 1, 1)
            } else {
                (d.year, d.month + 1)
            };
            let next = SimTime::from_date(ny, nm, 1, 0, 0).map_or(end, |t| t.minutes());
            let to = next.min(end);
            sim.advance_minutes(to - sim.now().minutes())
                .map_err(|e| e.to_string())?;
            if to == end {
                return Ok(());
            }
            self.economy.sample(sim);
            let people = sim.people();
            self.lived_building_months += sim
                .land()
                .buildings
                .iter()
                .filter(|b| {
                    b.finished()
                        && b.standing()
                        && people
                            .household(b.household)
                            .is_some_and(|h| !h.members.is_empty())
                })
                .count() as u64;
        }
    }

    /// The checks at the end of year `y` that fail a world: nobody stuck, no population or land
    /// problems, every good accounted for, households under a roof from the second year, the
    /// techniques every founder brings known, a first trail in the first year, and the economy's
    /// claims, fields and wealth measures in order. Growth is the caller's to judge.
    pub fn year_end(&mut self, sim: &Sim, y: u32) -> Vec<String> {
        let mut failures = self.economy.year_end(sim, y);
        let now = sim.now();
        let living = sim.people().living();
        let stuck = sim
            .people()
            .people
            .iter()
            .filter(|(_, p)| p.act.step_ends < now)
            .count();
        if stuck > 0 {
            failures.push(format!("{stuck} people stuck between events in year {y}"));
        }
        let problems = sim
            .people()
            .problems(sim.ids().peek_next(), sim.rules().catalog.activities.len());
        if let Some(first) = problems.first() {
            failures.push(format!(
                "{} population problems in year {y}, first: {first}",
                problems.len()
            ));
        }
        let land = sim.land();
        let problems = land.problems(
            sim.map(),
            sim.rules().land.habitats.len(),
            sim.ids().peek_next(),
        );
        if let Some(first) = problems.first() {
            failures.push(format!(
                "{} land problems in year {y}, first: {first}",
                problems.len()
            ));
        }
        let goods = &sim.rules().catalog.goods;
        let gaps = civ_agents::population::unaccounted(
            goods.len(),
            (&self.books.0, &self.books.1),
            (&sim.people().goods_held(), &sim.people().flows()),
        );
        if let Some(&(g, kg)) = gaps.first() {
            failures.push(format!(
                "{} goods unaccounted for in year {y}, first: {} {kg:+.3}",
                gaps.len(),
                goods[g].name
            ));
        }
        let households: Vec<_> = sim
            .people()
            .households
            .iter()
            .filter(|(_, h)| !h.members.is_empty())
            .map(|(_, h)| h.id)
            .collect();
        // Under a roof of their own home: a store or a workshop is no home.
        let catalog = &sim.rules().catalog;
        let roofed = households
            .iter()
            .filter(|&&h| {
                land.buildings
                    .iter()
                    .any(|b| b.household == h && b.roofed() && catalog.is_dwelling(&b.spec.program))
            })
            .count();
        if y >= 2 && (roofed as f64) < MIN_ROOFED * households.len() as f64 {
            failures.push(format!(
                "only {roofed} of {} households under a roof in year {y}",
                households.len()
            ));
        }
        if let Some(failure) = check_knowledge(sim, y) {
            failures.push(failure);
        }
        if y == 1
            && living > 0
            && !sim
                .people()
                .chronicle
                .iter()
                .any(|e| e.kind == civ_agents::ChronicleKind::FirstTrail)
        {
            failures.push("no trail worn out of the settlement in its first year".to_owned());
        }
        failures
    }
}

fn check_years(sim: &mut Sim, years: u32, result: &mut SmokeResult) {
    let mut run = LongRun::begin(sim);
    let year = civ_core::time::MINUTES_PER_YEAR;
    // The band has lived its first month (`check_people`).
    let founded = sim.now().minutes() - DAYS * MINUTES_PER_DAY;
    for y in 1..=years {
        let end = founded + i64::from(y) * year;
        if let Err(e) = run.live_to(sim, end) {
            result
                .failures
                .push(format!("the clock stopped in year {y}: {e}"));
            return;
        }
        result.failures.extend(run.year_end(sim, y));
        let living = sim.people().living();
        if living as f64 > MAX_GROWTH * run.founders as f64 {
            result
                .failures
                .push(format!("{living} people in year {y}: the band exploded"));
        }
        if living < MIN_ALIVE && !result.notes.iter().any(|n| n.starts_with("the band")) {
            result
                .notes
                .push(format!("the band was down to {living} in year {y}"));
        }
        result.living = Some(living);
    }
    result.economy = run.economy.grade(sim);
    for c in result.economy.iter().filter(|c| c.grade == Grade::Red) {
        result.failures.push(format!("{}: {}", c.name, c.text));
    }
    result.notes.extend(lost_where_people_live(sim));
    result.notes.extend(finds(sim));
    result.notes.extend(condition(sim));
    result.notes.extend(deposits(sim));
    result.notes.extend(earthworks(sim));
    result.notes.extend(kept_and_fixed(sim));
    result.notes.extend(style(sim));
    result.failures.extend(polity_problems(sim));
    result.notes.extend(polity(sim));
    result.notes.extend(takings(sim));
}

/// What a polity's history must hold to (ADR-0013): one polity a settlement; every law decided by
/// the rule of the custom's version in force when it was decided (M4c slice AF), over the stances
/// recorded, no more present than eligible; a law in force passed (an amendment later replaced
/// stands superseded); at most one law before the gathering, and a gathering only for it.
fn polity_problems(sim: &Sim) -> Vec<String> {
    use civ_agents::polity::{LawStatus, Outcome};
    let mut out = Vec::new();
    let pop = sim.people();
    for s in &sim.land().settlements {
        let n = pop.polities.iter().filter(|p| p.settlement == s.id).count();
        if n != 1 {
            out.push(format!("{} has {n} polities", s.name));
        }
    }
    for p in &pop.polities {
        for l in &p.laws {
            let (present, support, oppose) = l.counts();
            // The custom as it stood when the law was decided: the last version begun before then
            // (an amendment is decided by the custom it replaces).
            let body_then = l.decided.map_or(p.body, |t| {
                p.versions
                    .iter()
                    .rfind(|v| v.since < t || (v.since == t && v.law != Some(l.id)))
                    .map_or(p.body, |v| v.body)
            });
            match l.outcome {
                Some(o) => {
                    if o != body_then.decide(l.eligible, present, support, oppose)
                        || present > l.eligible
                    {
                        out.push(format!("law {} was not decided by its body's rule", l.id));
                    }
                    let passed = o == Outcome::Passed;
                    let held = matches!(
                        l.status,
                        LawStatus::InForce | LawStatus::Lapsed | LawStatus::Superseded
                    );
                    if passed != held {
                        out.push(format!("law {} stands {:?} after {o:?}", l.id, l.status));
                    }
                }
                None if l.status != LawStatus::Proposed => {
                    out.push(format!("law {} is {:?} undecided", l.id, l.status));
                }
                None => {}
            }
        }
        let proposed = p
            .laws
            .iter()
            .filter(|l| l.status == LawStatus::Proposed)
            .count();
        // A gathering may sit on cases alone; one called on a law carries the proposal.
        let called = p.gathering.as_ref().and_then(|g| g.law);
        if proposed > 1 || called != p.agenda().map(|l| l.id) {
            out.push(format!("polity {} has {proposed} laws before it", p.id));
        }
    }
    out
}

/// What the polities did, in words: "3 laws proposed, 3 decided, 2 passed; a common store at a
/// twentieth: 1114 kg levied, 80 kg kept back, 0 kg given in relief, 1278 kg held; 1 storekeeper
/// named, 0 gone, 0 replaced, one keeping it now". `None` when nothing was proposed.
fn polity(sim: &Sim) -> Option<String> {
    use civ_agents::polity::{Law, LawStatus, Outcome, PolicyKind};
    let pop = sim.people();
    let policies = &sim.rules().catalog.policies;
    let kind = |l: &Law| policies.get(usize::from(l.policy)).map(|d| d.kind);
    let laws: Vec<_> = pop.polities.iter().flat_map(|p| &p.laws).collect();
    if laws.is_empty() {
        return None;
    }
    let decided = laws.iter().filter(|l| l.outcome.is_some()).count();
    let passed = laws
        .iter()
        .filter(|l| l.outcome == Some(Outcome::Passed))
        .count();
    // Who came (M4c slice AE: only those who heard a gathering was called may come).
    let sat: Vec<_> = laws
        .iter()
        .filter(|l| l.outcome.is_some() && l.eligible > 0)
        .collect();
    let came = if sat.is_empty() {
        String::new()
    } else {
        let share = sat
            .iter()
            .map(|l| l.stances.len() as f64 / f64::from(l.eligible))
            .sum::<f64>()
            / sat.len() as f64;
        let short = sat
            .iter()
            .filter(|l| l.outcome == Some(Outcome::NoQuorum))
            .count();
        format!(
            " ({:.0}\u{a0}% of the members came on average; {short} without a quorum)",
            100.0 * share
        )
    };
    let mut parts = vec![format!(
        "{} laws proposed, {decided} decided{came}, {passed} passed",
        laws.len()
    )];
    for p in &pop.polities {
        let stores = p
            .laws
            .iter()
            .filter(|l| l.status == LawStatus::InForce && kind(l) == Some(PolicyKind::CommonStore));
        for l in stores {
            let c = &l.compliance;
            let at = if l.levy_share > 0.0 {
                format!(
                    "at {}",
                    civ_agents::polity::share_text(f64::from(l.levy_share))
                )
            } else {
                "levying nothing".to_owned()
            };
            parts.push(format!(
                "a common store {at}: {:.0} kg levied, {:.0} kg kept back, {:.0} kg given in \
                 relief, {:.0} kg held",
                c.levied_kg,
                c.withheld_kg,
                c.relief_kg,
                p.stores.iter().sum::<f64>()
            ));
        }
        let named: Vec<_> = p
            .laws
            .iter()
            .filter(|l| kind(l) == Some(PolicyKind::KeepStore))
            .filter(|l| {
                matches!(
                    l.status,
                    LawStatus::InForce | LawStatus::Lapsed | LawStatus::Superseded
                )
            })
            .collect();
        if !named.is_empty() {
            let count = |s: LawStatus| named.iter().filter(|l| l.status == s).count();
            parts.push(format!(
                "{} storekeeper{} named, {} gone, {} replaced, {} keeping it now",
                named.len(),
                if named.len() == 1 { "" } else { "s" },
                count(LawStatus::Lapsed),
                count(LawStatus::Superseded),
                if p.keeper().is_some() { "one" } else { "none" }
            ));
        }
        // The watch (M4b slice AC): what anyone could see of it, and what it did with takings it
        // saw (the truth).
        let watches: Vec<_> = p
            .laws
            .iter()
            .filter(|l| kind(l) == Some(PolicyKind::KeepWatch))
            .filter(|l| matches!(l.status, LawStatus::InForce | LawStatus::Lapsed))
            .collect();
        if !watches.is_empty() {
            let rounds: u32 = watches.iter().map(|l| l.watch.rounds).sum();
            let hours: f64 = watches.iter().map(|l| l.watch.minutes).sum::<f64>() / 60.0;
            let cases: u32 = watches.iter().map(|l| l.watch.cases).sum();
            let order = &pop.order;
            let kept =
                |k: civ_agents::crime::Kept| order.sightings.iter().filter(|s| s.kept == k).count();
            use civ_agents::crime::Kept;
            parts.push(format!(
                "{} watch{} named, {} keeping it now: {rounds} rounds, {hours:.0} hours, {cases} \
                 case{} brought; of {} takings it saw, {} reported or told, {} let go, {} paid to \
                 say nothing",
                watches.len(),
                if watches.len() == 1 { "" } else { "es" },
                if p.watcher().is_some() { "one" } else { "none" },
                if cases == 1 { "" } else { "s" },
                order.sightings.len(),
                kept(Kept::Reported) + kept(Kept::Told) + kept(Kept::Refused),
                kept(Kept::LookedAway),
                kept(Kept::Paid),
            ));
        }
        // Laws against taking (M4b slice AB): how many were put, and the one in force.
        let against: Vec<_> = p
            .laws
            .iter()
            .filter(|l| kind(l) == Some(PolicyKind::AgainstTaking))
            .collect();
        if !against.is_empty() {
            let n = against.len();
            let now = against
                .iter()
                .find(|l| l.status == LawStatus::InForce)
                .map_or_else(
                    || "none in force".to_owned(),
                    |l| format!("in force: what was taken given back{}", l.sanction.words()),
                );
            parts.push(format!(
                "{n} law{} against taking proposed, {now}",
                if n == 1 { "" } else { "s" }
            ));
        }
        // Amendments of the custom (M4c slice AF): how many were put and passed, and the custom
        // now when it changed.
        let amendments: Vec<_> = p
            .laws
            .iter()
            .filter(|l| kind(l) == Some(PolicyKind::AmendBody))
            .collect();
        if !amendments.is_empty() {
            let passed = amendments
                .iter()
                .filter(|l| l.outcome == Some(Outcome::Passed))
                .count();
            let now = if p.versions.len() > 1 {
                format!("; the custom now: {}", p.body.clause())
            } else {
                String::new()
            };
            parts.push(format!(
                "{} amendment{} of the custom proposed, {passed} passed{now}",
                amendments.len(),
                if amendments.len() == 1 { "" } else { "s" },
            ));
        }
        // Curfews (M4b slice AD): how many were put, the one in force, and how it was kept.
        let curfews: Vec<_> = p
            .laws
            .iter()
            .filter(|l| kind(l) == Some(PolicyKind::Curfew))
            .collect();
        if !curfews.is_empty() {
            let n = curfews.len();
            let now = curfews
                .iter()
                .find(|l| l.status == LawStatus::InForce)
                .map_or_else(
                    || "none in force".to_owned(),
                    |l| {
                        format!(
                            "in force from {}:00 to {}:00, broken {} times knowingly and {} not",
                            l.hours.0, l.hours.1, l.compliance.broken, l.compliance.broken_unaware
                        )
                    },
                );
            parts.push(format!(
                "{n} curfew{} proposed, {now}",
                if n == 1 { "" } else { "s" }
            ));
        }
    }
    // Opinion (M4c slice AG): positions held, how often a person took in what a companion said
    // (research 06-04 §3.2 gives 0.5 exposures a person a week as a prior), and how far talk
    // has moved positions from each household's lot on average.
    let op = &pop.opinion;
    if !op.positions.is_empty() {
        let weeks = (sim.now().minutes() as f64 / (7.0 * 1440.0)).max(1.0);
        let adults = {
            let mut h: Vec<_> = op.positions.iter().map(|p| p.holder).collect();
            h.dedup();
            h.len().max(1) as f64
        };
        let drift = op
            .positions
            .iter()
            .map(|p| f64::from((p.x - p.anchor).abs()))
            .sum::<f64>()
            / op.positions.len() as f64;
        parts.push(format!(
            "{} positions held; {:.2} told and {:.2} taken in a person a week; talk had moved \
             them {drift:.3} from their household's lot on average",
            op.positions.len(),
            op.told as f64 / adults / weeks,
            op.taken as f64 / adults / weeks,
        ));
    }
    // Values (M4c slice AG): how widely each is held one way or the other, and how often what
    // someone holds dear weighed in a stance at a gathering (more than the stance margin).
    let vs = &pop.values;
    if !vs.held.is_empty() {
        let defs = &sim.rules().catalog.values;
        let spread: Vec<String> = defs
            .iter()
            .enumerate()
            .map(|(k, d)| {
                let held: Vec<f64> = vs
                    .held
                    .iter()
                    .filter(|h| usize::from(h.value) == k)
                    .map(|h| f64::from(h.v))
                    .collect();
                let n = held.len().max(1) as f64;
                let mean = held.iter().sum::<f64>() / n;
                let strong = held.iter().filter(|v| v.abs() >= 0.5).count() as f64 / n;
                format!("{} {mean:+.2} ({:.0} % strongly)", d.name, 100.0 * strong)
            })
            .collect();
        let margin = sim.rules().people.polity.stance_margin as f32;
        let (weighed, stances) = pop
            .polities
            .iter()
            .flat_map(|p| &p.laws)
            .flat_map(|l| &l.stances)
            .fold((0usize, 0usize), |(w, n), r| {
                (w + usize::from(r.values.abs() > margin), n + 1)
            });
        parts.push(format!(
            "values held on average: {}; what they hold dear weighed in {weighed} of {stances} \
             stances",
            spread.join(", ")
        ));
    }
    // Ideologies (M4c slice AG): how many hold each now, how often one was spoken of and taken up
    // at the hearth, and the laws proposed under each and how many of them passed.
    let ideas = &pop.ideologies;
    let defs = &sim.rules().catalog.ideologies;
    if !defs.is_empty() {
        let weeks = (sim.now().minutes() as f64 / (7.0 * 1440.0)).max(1.0);
        let people = pop.people.len().max(1) as f64;
        let each: Vec<String> = defs
            .iter()
            .enumerate()
            .map(|(k, d)| {
                let holders = ideas
                    .held
                    .iter()
                    .filter(|h| usize::from(h.ideology) == k)
                    .count();
                let laws: Vec<_> = pop
                    .polities
                    .iter()
                    .flat_map(|p| &p.laws)
                    .filter(|l| ideas.creed_of(l.id) == Some(k as u16))
                    .collect();
                let passed = laws
                    .iter()
                    .filter(|l| l.outcome == Some(civ_agents::polity::Outcome::Passed))
                    .count();
                format!(
                    "{} held by {holders} ({} law{} proposed under it, {passed} passed)",
                    d.name,
                    laws.len(),
                    if laws.len() == 1 { "" } else { "s" }
                )
            })
            .collect();
        parts.push(format!(
            "ideologies: {}; {:.3} spoken of and {:.3} taken up a person a week",
            each.join(", "),
            ideas.told as f64 / people / weeks,
            ideas.taken as f64 / people / weeks,
        ));
    }
    // Factions (M4c slice AH): how many were founded and how many have members now, their
    // sizes, how often people joined and left, and the food members' households gave their
    // stores and the stores gave back.
    let fs = &pop.factions;
    let live: Vec<usize> = fs
        .list
        .iter()
        .filter(|f| f.is_live())
        .map(|f| fs.members_of(f.id).count())
        .collect();
    let food_kg = |flow: civ_agents::person::Flow| -> f64 {
        let goods = &sim.rules().catalog.goods;
        fs.list
            .iter()
            .map(|f| {
                (0..goods.len())
                    .filter(|&g| goods[g].kcal_per_kg > 0.0)
                    .map(|g| f.flows.get(flow, g))
                    .sum::<f64>()
            })
            .sum()
    };
    parts.push(format!(
        "factions: {} founded, {} with members now{}; {} joinings and {} leavings; {:.0} kg given \
         in dues, {:.0} kg given out",
        fs.list.len(),
        live.len(),
        if live.is_empty() {
            String::new()
        } else {
            format!(
                " ({} members)",
                live.iter()
                    .map(usize::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        },
        fs.joined,
        fs.left,
        food_kg(civ_agents::person::Flow::Received).max(0.0),
        food_kg(civ_agents::person::Flow::Given).max(0.0),
    ));
    // Petitions (M4c slice AH, step two): how many were called, how many came to each, and what
    // the gathering made of those put to it.
    if !fs.petitions.is_empty() {
        let laws: Vec<&civ_agents::polity::Law> =
            pop.polities.iter().flat_map(|p| &p.laws).collect();
        let outcome = |p: &civ_agents::faction::Petition| {
            p.law
                .and_then(|id| laws.iter().find(|l| l.id == id))
                .and_then(|l| l.outcome)
        };
        let count = |o: civ_agents::polity::Outcome| {
            fs.petitions
                .iter()
                .filter(|p| outcome(p) == Some(o))
                .count()
        };
        let came: Vec<String> = fs
            .petitions
            .iter()
            .map(|p| p.came.len().to_string())
            .collect();
        let offices = fs.petitions.iter().filter(|p| p.nominee.is_some()).count();
        parts.push(format!(
            "petitions: {} called ({} came; {} for another officeholder, {} for a store's share); \
             {} granted, {} turned down, {} split, {} without a quorum, {} never put",
            fs.petitions.len(),
            came.join(", "),
            offices,
            fs.petitions.len() - offices,
            count(civ_agents::polity::Outcome::Passed),
            count(civ_agents::polity::Outcome::Failed),
            count(civ_agents::polity::Outcome::Tied),
            count(civ_agents::polity::Outcome::NoQuorum),
            fs.petitions
                .iter()
                .filter(|p| p.answered && p.law.is_none())
                .count(),
        ));
    }
    // Refusals of a levy (M4c slice AH, step three): how many were called, and how many kept
    // back what under them.
    if !fs.refusals.is_empty() {
        let kept: Vec<String> = fs
            .refusals
            .iter()
            .map(|r| format!("{} kept back {:.0} kg", r.kept.len(), r.kept_kg))
            .collect();
        parts.push(format!(
            "refusals of a levy: {} called ({})",
            fs.refusals.len(),
            kept.join("; ")
        ));
    }
    // Norms (M4c slice AG): for each, how far it is held on average, what people believe of
    // others against what households did at their last levy, how many it moves, and how often an
    // account was told and taken in.
    let nm = &pop.norms;
    for (k, def) in sim.rules().catalog.norms.iter().enumerate() {
        let held: Vec<_> = nm
            .states
            .iter()
            .filter(|s| usize::from(s.norm) == k)
            .collect();
        if held.is_empty() {
            continue;
        }
        let n = held.len() as f64;
        let mean = |f: &dyn Fn(&civ_agents::norm::NormState) -> f64| {
            held.iter().map(|s| f(s)).sum::<f64>() / n
        };
        let endorse = mean(&|s| f64::from(s.endorse));
        let expect = mean(&|s| f64::from(s.expect));
        let moved = mean(&|s| {
            civ_agents::norm::activation(f64::from(s.expect), f64::from(s.threshold), def.width)
        });
        let day = sim.now().day_index();
        let recent: Vec<f64> = nm
            .acts
            .iter()
            .filter(|a| day - a.day <= def.tell_days)
            .filter_map(civ_agents::norm::LevyAct::share_paid)
            .collect();
        let did = if recent.is_empty() {
            "no household paid or kept back a levy within the year".to_owned()
        } else {
            format!(
                "{:.2} paid by the {} households at their last levy",
                recent.iter().sum::<f64>() / recent.len() as f64,
                recent.len()
            )
        };
        let weeks = (sim.now().minutes() as f64 / (7.0 * 1440.0)).max(1.0);
        parts.push(format!(
            "{}: held {endorse:.2} on average; they believe {expect:.2} of households abide \
             ({did}); {moved:.2} moved by others' doing it; {:.2} accounts told and {:.2} taken \
             in a person a week",
            def.name,
            nm.told as f64 / n / weeks,
            nm.taken as f64 / n / weeks,
        ));
    }
    // Grievances (M4c slice AE): what is held now, by issue, and how many are told and still news.
    let word = &pop.word;
    if !word.grievances.is_empty() {
        use civ_agents::word::{ClaimKind, Grieved};
        let of = |i: Grieved| word.grievances.iter().filter(|g| g.issue == i).count();
        let mut holders: Vec<_> = word.grievances.iter().map(|g| g.holder).collect();
        holders.dedup();
        let told = word
            .claims
            .iter()
            .filter(|c| c.kind == ClaimKind::Grievance)
            .count();
        parts.push(format!(
            "{} grievances held now by {} people ({} over food, {} over a levy, {} over how they \
             were treated, {} over a wrong unanswered), {told} told and still news",
            word.grievances.len(),
            holders.len(),
            of(Grieved::Subsistence),
            of(Grieved::Extraction),
            of(Grieved::Treatment),
            of(Grieved::Collective),
        ));
    }
    for p in &pop.polities {
        let label = civ_sim::labels::label_of(sim, p);
        parts.push(format!(
            "labelled {} ({:.2})",
            label.in_prose(),
            label.confidence
        ));
    }
    Some(parts.join("; "))
}

/// What was taken and what followed (M4b slice AA), in words: "7 attempts to take by 3 people
/// (of 214 adults now), 4 takings (190 kg), 2 seen; 2 known to the household taken from, 1 demand
/// to give back (1 met, 0 refused); 3 asks refused; 0.71 of the takers' neighbours held more food".
/// `None` when nobody went to take.
fn takings(sim: &Sim) -> Option<String> {
    use civ_agents::crime::{Outcome, Standing};
    let pop = sim.people();
    let order = &pop.order;
    if order.incidents.is_empty() {
        return None;
    }
    let taken: Vec<_> = order
        .incidents
        .iter()
        .filter(|i| i.outcome == Outcome::Taken)
        .collect();
    let kg = taken
        .iter()
        .flat_map(|i| &i.goods)
        .map(|&(_, kg)| f64::from(kg))
        .sum::<f64>()
        .max(0.0);
    let seen = order
        .incidents
        .iter()
        .filter(|i| !i.seen_by.is_empty())
        .count();
    let known: std::collections::BTreeSet<u32> =
        order.responses.iter().map(|r| r.incident).collect();
    let demands = order
        .responses
        .iter()
        .filter(|r| r.choice == civ_agents::crime::Choice::Demand)
        .count();
    let standing = |s: Standing| {
        order
            .obligations
            .iter()
            .filter(|o| o.kind == civ_agents::crime::Owed::Demanded && o.standing == s)
            .count()
    };
    let stage =
        |s: civ_agents::crime::CaseStage| order.cases.iter().filter(|c| c.stage == s).count();
    let takers: std::collections::BTreeSet<_> = order.incidents.iter().map(|i| i.actor).collect();
    let adult = sim.rules().people.family.independent_age;
    let adults = pop
        .people
        .iter()
        .filter(|(_, p)| p.age_years(sim.now()) >= adult)
        .count();
    let n = |count: usize, one: &str, many: &str| {
        format!("{count} {}", if count == 1 { one } else { many })
    };
    let cases = if order.cases.is_empty() {
        String::new()
    } else {
        use civ_agents::crime::CaseStage;
        let imposed = order.obligations.iter().filter(|o| o.case.is_some());
        let paid = imposed
            .clone()
            .filter(|o| o.standing == Standing::Met)
            .count();
        format!(
            "; {} before the gathering ({} found, {} not found, {} unheard; {paid} of {} \
             imposed obligations paid)",
            n(order.cases.len(), "case", "cases"),
            stage(CaseStage::Found),
            stage(CaseStage::NotFound),
            stage(CaseStage::Unheard),
            imposed.count(),
        )
    };
    let richer = crate::dashboard::CrimeSeen::of(order)
        .attempts
        .mean()
        .map_or_else(String::new, |m| {
            format!("; {m:.2} of the takers' neighbours held more food")
        });
    Some(format!(
        "{} to take by {} (of {adults} adults now), {} ({kg:.0} kg), {seen} seen; {} known to \
         the household taken from, {} to give back ({} met, {} refused); {} refused{cases}{richer}",
        n(order.incidents.len(), "attempt", "attempts"),
        n(takers.len(), "person", "people"),
        n(taken.len(), "taking", "takings"),
        known.len(),
        n(demands, "demand", "demands"),
        standing(Standing::Met),
        standing(Standing::Refused),
        n(order.refusals as usize, "ask", "asks"),
    ))
}

/// What households hold of goods that keep others or stay where they are made (M3b slice Q), in
/// words: "held: 41 × Pot, 2 × Oven". `None` when they hold none.
fn kept_and_fixed(sim: &Sim) -> Option<String> {
    let goods = &sim.rules().catalog.goods;
    let parts: Vec<String> = goods
        .iter()
        .enumerate()
        .filter(|(_, g)| g.store.is_some() || g.tool.as_ref().is_some_and(|t| t.fixed))
        .filter_map(|(i, g)| {
            let held: f64 = sim
                .people()
                .households
                .iter()
                .map(|(_, h)| h.stores.get(i).copied().unwrap_or(0.0))
                .sum();
            (held >= 0.5).then(|| format!("{held:.0} × {}", g.name))
        })
        .collect();
    (!parts.is_empty()).then(|| format!("held: {}", parts.join(", ")))
}

/// The ways the world's buildings were built at the end (M3b slice R), in words: "roofs pitched
/// 46-51°, eaves 182-207 cm; 3 of 12 buildings built after an admired one". `None` without
/// buildings.
fn style(sim: &Sim) -> Option<String> {
    let catalog = &sim.rules().catalog;
    let buildings = &sim.land().buildings;
    let traits: Vec<[f32; 3]> = buildings
        .iter()
        .filter_map(|b| {
            let def = catalog
                .buildings
                .get(catalog.building_index(&b.spec.program)?)?;
            Some(civ_agents::style::traits_of(&b.spec, def).traits())
        })
        .collect();
    if traits.is_empty() {
        return None;
    }
    let range = |k: usize| {
        traits.iter().fold((f32::MAX, f32::MIN), |(lo, hi), t| {
            (lo.min(t[k]), hi.max(t[k]))
        })
    };
    let ((p0, p1), (e0, e1)) = (range(0), range(1));
    let followed = buildings.iter().filter(|b| b.style_from.is_some()).count();
    Some(format!(
        "roofs pitched {:.0}-{:.0}°, eaves {e0:.0}-{e1:.0} cm; {followed} of {} buildings built \
         after an admired one",
        p0 / 100.0,
        p1 / 100.0,
        buildings.len()
    ))
}

/// The earthworks by the end (ADR-0010 §2), in words: "3 plots levelled, 19 m³ cut; 2 pits and
/// quarries at deposits, 4.5 m³; 14 daub pits, 61 m³". `None` without earthworks.
fn earthworks(sim: &Sim) -> Option<String> {
    use civ_land::earth::{EarthKind, Earthwork};
    let works = &sim.land().earthworks;
    let count = |of: &dyn Fn(&Earthwork) -> bool| -> (usize, f64) {
        works.iter().filter(|w| of(w)).fold((0, 0.0), |(n, m3), w| {
            (n + 1, m3 + f64::from(w.cut_m3) * f64::from(w.done))
        })
    };
    let plural = |n: usize, one: &str, many: &str| if n == 1 { one } else { many }.to_owned();
    let mut parts = Vec::new();
    let (plots, cut) = count(&|w| w.kind == EarthKind::Platform);
    if plots > 0 {
        let noun = plural(plots, "plot", "plots");
        parts.push(format!("{plots} {noun} levelled, {cut:.0} m³ cut"));
    }
    let (workings, dug) = count(&|w| w.kind == EarthKind::Pit && w.deposit.is_some());
    if workings > 0 {
        let noun = plural(workings, "pit or quarry", "pits and quarries");
        parts.push(format!("{workings} {noun} at deposits, {dug:.1} m³"));
    }
    let (daub, daubed) = count(&|w| w.kind == EarthKind::Pit && w.deposit.is_none());
    if daub > 0 {
        let noun = plural(daub, "daub pit", "daub pits");
        parts.push(format!("{daub} {noun}, {daubed:.0} m³"));
    }
    (!parts.is_empty()).then(|| parts.join("; "))
}

/// The world's deposits at the end (ADR-0010 §1), in words: "96 deposits, 31 showing, 4 found",
/// and what has been dug from them: "; 1240 kg taken". `None` without deposits.
fn deposits(sim: &Sim) -> Option<String> {
    let all = &sim.land().deposits;
    if all.is_empty() {
        return None;
    }
    let showing = all.iter().filter(|d| d.body.exposed).count();
    let found = all
        .iter()
        .filter(|d| {
            sim.people()
                .deposits_known
                .iter()
                .any(|k| k.deposit == d.id)
        })
        .count();
    let taken: f64 = all.iter().map(|d| d.taken_kg).sum();
    let dug = if taken > 0.0 {
        format!("; {taken:.0} kg taken")
    } else {
        String::new()
    };
    Some(format!(
        "{} deposits, {showing} showing, {found} found{dug}",
        all.len()
    ))
}

/// How the world's buildings stand at the end (ADR-0009 §4, §5), in words: "14 buildings, 3
/// showing wear, 1 damaged, no ruin; 6 groups mended; 2 gave way, killing 1". `None` without
/// buildings.
fn condition(sim: &Sim) -> Option<String> {
    use civ_land::{BuildingState, GroupState};
    let buildings = &sim.land().buildings;
    if buildings.is_empty() {
        return None;
    }
    let count = |state: BuildingState| buildings.iter().filter(|b| b.state == state).count();
    let showing = buildings
        .iter()
        .filter(|b| {
            b.state == BuildingState::Standing
                && b.condition.iter().any(|c| c.state != GroupState::Sound)
        })
        .count();
    let mended: usize = buildings
        .iter()
        .map(|b| {
            b.condition
                .iter()
                .filter(|c| c.repaired > c.installed)
                .count()
        })
        .sum();
    let ruins = match count(BuildingState::Ruin) {
        0 => "no ruin".to_owned(),
        1 => "1 ruin".to_owned(),
        n => format!("{n} ruins"),
    };
    let failures: Vec<f64> = sim
        .people()
        .chronicle
        .iter()
        .filter(|e| e.kind == civ_agents::ChronicleKind::BuildingFailed)
        .map(|e| e.number)
        .collect();
    let gave_way = match failures.len() {
        0 => String::new(),
        n => format!(
            "; {n} gave way, killing {}",
            failures.iter().sum::<f64>().round() as u64
        ),
    };
    Some(format!(
        "{} buildings, {showing} showing wear, {} damaged, {ruins}; {mended} groups \
         mended{gave_way}",
        buildings.len(),
        count(BuildingState::Damaged)
    ))
}

/// What people worked out (ADR-0008 §3), in words: "found drying and smoking in year 4".
fn finds(sim: &Sim) -> Vec<String> {
    sim.people()
        .chronicle
        .iter()
        .filter(|e| e.kind == civ_agents::ChronicleKind::TechniqueFound)
        .map(|e| {
            format!(
                "found {} in year {}",
                e.name.to_lowercase(),
                e.at.date().year
            )
        })
        .collect()
}

/// Whether the techniques every founder brings and children learn in upbringing are known by
/// at least [`MIN_UPBRINGING`] of the living old enough for them: the least known one if not.
fn check_knowledge(sim: &Sim, y: u32) -> Option<String> {
    let (catalog, params) = (&sim.rules().catalog, &sim.rules().people);
    let now = sim.now();
    let mut worst: Option<(f64, usize, usize, &str)> = None;
    for &(t, share) in &params.knowledge.founders {
        let Some(def) = catalog.techniques.get(t) else {
            continue;
        };
        let age = civ_agents::knowledge::knowing_age(catalog, t, params.family.independent_age);
        let (Some(age), true) = (age, share >= 1.0 && def.upbringing) else {
            continue;
        };
        let (old_enough, knowing) = sim
            .people()
            .people
            .iter()
            .filter(|(_, p)| p.age_years(now) >= age)
            .fold((0, 0), |(n, k), (_, p)| {
                (n + 1, k + usize::from(p.knows(t)))
            });
        if old_enough == 0 {
            continue;
        }
        let coverage = knowing as f64 / old_enough as f64;
        if worst.is_none_or(|(c, ..)| coverage < c) {
            worst = Some((coverage, knowing, old_enough, &def.name));
        }
    }
    let (coverage, knowing, old_enough, name) = worst?;
    (coverage < MIN_UPBRINGING).then(|| {
        format!(
            "only {knowing} of the {old_enough} old enough know {} in year {y}",
            name.to_lowercase()
        )
    })
}

/// Techniques lost in a settlement where people still live, in words.
fn lost_where_people_live(sim: &Sim) -> Vec<String> {
    let people = sim.people();
    let lived_in: std::collections::BTreeSet<_> = people
        .households
        .iter()
        .filter(|(_, h)| !h.members.is_empty())
        .filter_map(|(_, h)| h.settlement)
        .collect();
    let catalog = &sim.rules().catalog;
    people
        .knowledge
        .iter()
        .filter(|e| {
            e.kind == civ_agents::knowledge::KnowledgeEventKind::Lost
                && lived_in.contains(&e.settlement)
        })
        .map(|e| {
            let name = catalog
                .techniques
                .get(usize::from(e.technique))
                .map_or("a technique", |d| d.name.as_str());
            format!("{} lost in year {}", name.to_lowercase(), e.at.date().year)
        })
        .collect()
}

/// Runs every preset with seeds `1..=seeds`, a few worlds at a time, reporting each result as it
/// finishes. The results come back in preset and seed order.
pub fn run(
    content: &ContentRegistry,
    options: SmokeOptions,
    report: &(dyn Fn(&SmokeResult) + Sync),
) -> Vec<SmokeResult> {
    let jobs: Vec<(String, u64)> = content
        .presets
        .iter()
        .flat_map(|p| (1..=options.seeds).map(move |seed| (p.id.clone(), seed)))
        .collect();
    let next = AtomicUsize::new(0);
    let results = Mutex::new(Vec::new());
    let workers = std::thread::available_parallelism()
        .map_or(1, |n| n.get())
        .min(jobs.len())
        .max(1);
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    let Some((preset, seed)) = jobs.get(i) else {
                        break;
                    };
                    let result = run_one(content, options, preset, *seed);
                    report(&result);
                    results
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .push((i, result));
                }
            });
        }
    });
    let mut results = results
        .into_inner()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    results.sort_by_key(|(i, _)| *i);
    results.into_iter().map(|(_, r)| r).collect()
}

fn run_one(
    content: &ContentRegistry,
    options: SmokeOptions,
    preset: &str,
    seed: u64,
) -> SmokeResult {
    let started = Instant::now();
    // The content's regimes in turn, by seed.
    let regimes = &content.catalog.regimes;
    let regime = regimes
        .get((seed.max(1) - 1) as usize % regimes.len().max(1))
        .map_or_else(
            || (String::new(), String::new()),
            |r| (r.id.clone(), r.name.clone()),
        );
    let created = Sim::create(
        &NewWorld {
            name: format!("Smoke {seed}"),
            seed,
            preset_id: preset.to_owned(),
            size_cells: options.size,
            band_size: 0,
            regime_id: regime.0,
        },
        content,
        &mut |_| {},
        &AtomicBool::new(false),
    );
    let mut result = SmokeResult {
        preset: preset.to_owned(),
        seed,
        elapsed: Duration::ZERO,
        stats: None,
        failures: Vec::new(),
        living: None,
        notes: Vec::new(),
        regime: regime.1,
        economy: Vec::new(),
    };
    match created {
        Err(e) => result.failures.push(format!("generation failed: {e}")),
        Ok(mut sim) => {
            result.stats = Some(*sim.stats());
            result.failures.extend(check_world(sim.stats()));
            if let Err(e) = check_round_trip(&mut sim, content) {
                result.failures.push(e);
            }
            result.failures.extend(check_people(&mut sim, content));
            if options.years > 0 && result.failures.is_empty() {
                check_years(&mut sim, options.years, &mut result);
            }
        }
    }
    result.elapsed = started.elapsed();
    result
}

/// Whether enough bands still live where they settled after a long run: at least half.
pub fn enough_alive(results: &[SmokeResult]) -> bool {
    let long: Vec<_> = results.iter().filter_map(|r| r.living).collect();
    long.is_empty() || 2 * long.iter().filter(|&&n| n >= MIN_ALIVE).count() >= long.len()
}

/// One line of the report table.
pub fn format_result(result: &SmokeResult) -> String {
    let figures = result.stats.map_or_else(String::new, |s| {
        format!(
            "{:>6.0} m {:>5.0}% {:>6.0}% {:>7} {:>9.1} {:>5}",
            s.max_elevation_m - s.min_elevation_m,
            (1.0 - s.ocean_fraction) * 100.0,
            s.gentle_land_fraction * 100.0,
            s.reaches,
            s.river_length_km,
            s.lakes
        )
    });
    let mut verdict = if result.failures.is_empty() {
        "pass".to_owned()
    } else {
        format!("FAIL: {}", result.failures.join("; "))
    };
    if let Some(living) = result.living {
        verdict = format!("{living:>3} people  {verdict}");
    }
    if !result.notes.is_empty() {
        verdict = format!("{verdict} ({})", result.notes.join("; "));
    }
    let mut line = format!(
        "{:<28} {:>4} {:>6.1} s {figures}  {verdict}",
        result.preset,
        result.seed,
        result.elapsed.as_secs_f64()
    );
    if !result.economy.is_empty() {
        let checks: Vec<String> = result
            .economy
            .iter()
            .map(|c| format!("{} ({})", c.text, c.grade.word()))
            .collect();
        line.push_str(&format!(
            "\n    {}: {}",
            result.regime.to_lowercase(),
            checks.join("; ")
        ));
    }
    line
}

/// How the worlds' economies were graded, check by check: how many came out at each grade.
pub fn economy_summary(results: &[SmokeResult]) -> Option<String> {
    let names: Vec<&str> = results
        .iter()
        .flat_map(|r| r.economy.iter().map(|c| c.name))
        .fold(Vec::new(), |mut v, n| {
            if !v.contains(&n) {
                v.push(n);
            }
            v
        });
    if names.is_empty() {
        return None;
    }
    let parts: Vec<String> = names
        .iter()
        .map(|&name| {
            let grades: Vec<String> = [Grade::Green, Grade::Amber, Grade::Gray, Grade::Red]
                .iter()
                .filter_map(|&g| {
                    let n = results
                        .iter()
                        .flat_map(|r| &r.economy)
                        .filter(|c| c.name == name && c.grade == g)
                        .count();
                    (n > 0).then(|| format!("{n} {}", g.word()))
                })
                .collect();
            format!("{name} {}", grades.join(", "))
        })
        .collect();
    Some(format!("economy: {}", parts.join("; ")))
}

/// The report table's header.
pub fn header() -> String {
    format!(
        "{:<28} {:>4} {:>8} {:>8} {:>6} {:>7} {:>7} {:>9} {:>5}  result",
        "preset", "seed", "time", "relief", "land", "gentle", "reaches", "river km", "lakes"
    )
}
