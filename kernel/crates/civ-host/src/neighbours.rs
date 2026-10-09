//! `civ-host neighbours`: the M5a demo (plan §7). Founding groups that know where one another
//! camped settle apart and live thirty years, each its first month by the minute and then a day at
//! a time at Max, as the dashboard's worlds do; at the end of a chosen year the observer sends a
//! migration wave to dry ground among them. Each year's end reports each settlement's people and
//! accounts, what passed between the settlements, the moves between them by reason, the
//! coalitions that began or ended, and where the wave's people live. The end sums them, with
//! every coalition from its first review to its fate, and says so if no splinter founding
//! happened.
//!
//! The long run's checks fail it (every settlement's accounts balance, the books, residence
//! histories that agree with the world: [`LongRun`]); nothing else is graded, and nothing here
//! steers the world: the observer's choices are the groups, whether they know of one another and
//! the wave, all stated in the report.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use civ_agents::places::{CoalitionFate, Lacking};
use civ_content::ContentRegistry;
use civ_core::time::{MINUTES_PER_DAY, MINUTES_PER_YEAR};
use civ_core::{PermanentId, SimTime};
use civ_land::Founding;
use civ_schema::SAVE_EXTENSION;
use civ_sim::{NewWorld, SPEED_MAX, Sim, persist};
use commons_persist::{SaveDir, SaveKind};

use crate::dashboard::{Between, PRESET};
use crate::smoke::{self, LongRun};

/// Years the demo lives after its first month.
pub const YEARS: u32 = 30;
/// Founding groups (M5a: three).
pub const GROUPS: u32 = 3;
/// The year at whose end the wave is sent.
pub const WAVE_YEAR: u32 = 10;
/// The wave's households (the observer's choice, within the god tool's range).
pub const WAVE_HOUSEHOLDS: u32 = 10;
/// The days over which the wave's households come.
pub const WAVE_DAYS: u32 = 3;
/// The months of food each of the wave's households carries.
pub const WAVE_MONTHS: u32 = 12;
/// How far from every settlement lived in the wave is sent, metres: beyond the 600 m within which
/// it would join one, so its households found their own and then choose where to go.
pub const WAVE_APART_M: f32 = 1500.0;

/// What to run.
#[derive(Clone, Debug)]
pub struct NeighboursOptions {
    /// The world-generation preset, or the dashboard's river valley.
    pub preset: Option<String>,
    /// The world seed.
    pub seed: u64,
    /// Map side, cells.
    pub size: u32,
    /// Founding groups, each a band of the content's default size.
    pub groups: u32,
    /// Whether the groups know where one another camped (the new-world dialog's choice).
    pub known: bool,
    /// Years to live after the first month.
    pub years: u32,
    /// The year at whose end the wave is sent (0: none).
    pub wave_year: u32,
    /// The wave's households, the days over which they come and their months of food.
    pub wave_households: u32,
    pub wave_days: u32,
    pub wave_months: u32,
    /// Keep the world's saves at every tenth year's end (and the last) in this folder.
    pub keep_saves: Option<PathBuf>,
}

impl Default for NeighboursOptions {
    fn default() -> Self {
        NeighboursOptions {
            preset: None,
            seed: 1,
            size: 1024,
            groups: GROUPS,
            known: true,
            years: YEARS,
            wave_year: WAVE_YEAR,
            wave_households: WAVE_HOUSEHOLDS,
            wave_days: WAVE_DAYS,
            wave_months: WAVE_MONTHS,
            keep_saves: None,
        }
    }
}

/// What the run found: the checks that failed, and the saves kept.
#[derive(Clone, Debug, Default)]
pub struct Outcome {
    /// The checks that failed.
    pub failures: Vec<String>,
    /// The saves kept.
    pub saves: Vec<PathBuf>,
}

/// Visits, hours, marriages and people moved from one settlement to another.
/// Per pair of settlements: visits, hours at the hearth, marriages, people moved, purchases and
/// trips to buy that bought nothing.
type Totals = BTreeMap<(PermanentId, PermanentId), (u32, f64, u32, u32, u32, u32)>;

/// Makes and lives the world, writing the report to `out`.
pub fn run(
    content: &ContentRegistry,
    options: &NeighboursOptions,
    out: &mut dyn Write,
) -> anyhow::Result<Outcome> {
    let preset = options.preset.clone().unwrap_or_else(|| PRESET.to_owned());
    let mut sim = Sim::create(
        &NewWorld {
            name: format!("Neighbours {}", options.seed),
            seed: options.seed,
            preset_id: preset.clone(),
            size_cells: options.size,
            band_size: 0,
            neighbours: vec![0; options.groups.saturating_sub(1) as usize],
            neighbours_known: options.known,
            regime_id: String::new(),
        },
        content,
        &mut |_| {},
        &AtomicBool::new(false),
    )
    .map_err(|e| anyhow::anyhow!("generation failed: {e}"))?;
    let mut outcome = Outcome::default();
    writeln!(
        out,
        "{preset} seed {} on a {}-cell map under {}: {} founding groups {} where the others \
         camped, {} people",
        options.seed,
        options.size,
        sim.regime().name.to_lowercase(),
        sim.land().settlements.len(),
        if options.known {
            "knowing"
        } else {
            "not knowing"
        },
        sim.people().living(),
    )?;
    for s in &sim.land().settlements {
        writeln!(
            out,
            "  {} at ({:.0}, {:.0}) m, {} people",
            s.name,
            s.hearth_m.0,
            s.hearth_m.1,
            sim.people().residents(s.id)
        )?;
    }
    outcome
        .failures
        .extend(smoke::check_people(&mut sim, content));
    if !outcome.failures.is_empty() {
        for f in &outcome.failures {
            writeln!(out, "  FAILED: {f}")?;
        }
        return Ok(outcome);
    }
    sim.set_speed(SPEED_MAX)
        .and_then(|()| sim.advance_to_midnight().map(|_| ()))
        .map_err(|e| anyhow::anyhow!("the clock stopped: {e}"))?;
    let mut run = LongRun::begin(&sim);
    let start = sim.now().minutes();
    let dir = options
        .keep_saves
        .as_ref()
        .map(|d| d.join(format!("neighbours-{}", options.seed)));
    let mut year_from = sim.now();
    let mut contact_before = contact_totals(&sim);
    for y in 1..=options.years {
        if let Err(e) = run.live_to(&mut sim, start + i64::from(y) * MINUTES_PER_YEAR) {
            outcome
                .failures
                .push(format!("the clock stopped in year {y}: {e}"));
            break;
        }
        // The wave comes at the year's last minute, so its first households are in the year's
        // accounts and its save.
        if y == options.wave_year {
            send_wave(&mut sim, options, out)?;
        }
        let failures = run.year_end(&sim, y);
        let to = sim.now().plus_minutes(1);
        report_year(&sim, y, year_from, to, &contact_before, out)?;
        for f in &failures {
            writeln!(out, "  FAILED: {f}")?;
        }
        outcome.failures.extend(failures);
        let problems = sim.people().residence_problems();
        for p in problems.iter().take(5) {
            writeln!(out, "  FAILED: {p}")?;
        }
        outcome.failures.extend(problems);
        year_from = to;
        contact_before = contact_totals(&sim);
        if (y % 10 == 0 || y == options.years)
            && let Some(dir) = &dir
        {
            match keep(&mut sim, dir, y) {
                Ok(path) => outcome.saves.push(path),
                Err(e) => outcome.failures.push(format!("the save of year {y}: {e}")),
            }
        }
    }
    report_end(&sim, &outcome, out)?;
    Ok(outcome)
}

/// Saves the world as "year `y`" in `dir`.
fn keep(sim: &mut Sim, dir: &Path, y: u32) -> Result<PathBuf, String> {
    let saves = SaveDir::create(dir, SAVE_EXTENSION).map_err(|e| e.to_string())?;
    persist::save(sim, &saves, SaveKind::Manual, &format!("year {y}"))
        .map(|saved| saved.path)
        .map_err(|e| e.to_string())
}

/// Sends the wave to the dry, gentle ground nearest the middle of the settlements lived in that
/// is [`WAVE_APART_M`] from each, and says where.
fn send_wave(
    sim: &mut Sim,
    options: &NeighboursOptions,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    let Some(at) = wave_point(sim, WAVE_APART_M) else {
        writeln!(
            out,
            "the wave: no dry ground {WAVE_APART_M} m from every settlement"
        )?;
        return Ok(());
    };
    match sim.send_wave(
        at,
        options.wave_households,
        options.wave_days,
        options.wave_months,
    ) {
        Ok((record, came)) => writeln!(
            out,
            "the observer sent a wave (influence {record}) of {} households over {} days with {} \
             months' food to ({:.0}, {:.0}) m on {}: {} households came at once",
            options.wave_households,
            options.wave_days,
            options.wave_months,
            at.0,
            at.1,
            sim.date(),
            came.len()
        )?,
        Err(e) => writeln!(out, "the wave could not be sent: {e}")?,
    }
    Ok(())
}

/// The cell centre nearest the middle of the settlements lived in that is dry land people can
/// walk on, no steeper than a founding band's site may be, and `apart_m` from every hearth.
fn wave_point(sim: &Sim, apart_m: f32) -> Option<(f32, f32)> {
    let hearths: Vec<(f32, f32)> = sim
        .land()
        .settlements
        .iter()
        .filter(|s| s.abandoned.is_none())
        .map(|s| s.hearth_m)
        .collect();
    if hearths.is_empty() {
        return None;
    }
    let n = hearths.len() as f32;
    let middle = (
        hearths.iter().map(|h| h.0).sum::<f32>() / n,
        hearths.iter().map(|h| h.1).sum::<f32>() / n,
    );
    let map = sim.map();
    let (cell, w) = (map.cell_size_m, map.width as usize);
    let max_slope = sim.rules().people.band.site_max_slope;
    let d = |p: &(f32, f32)| (p.0 - middle.0).hypot(p.1 - middle.1);
    (0..map.cell_count())
        .filter(|&i| {
            map.water[i] == civ_world::WATER_LAND
                && sim.nav().walkable(i)
                && f64::from(civ_world::terrain::slope_at(map, i)) <= max_slope
        })
        .map(|i| (((i % w) as f32 + 0.5) * cell, ((i / w) as f32 + 0.5) * cell))
        .filter(|p| {
            hearths
                .iter()
                .all(|h| (p.0 - h.0).hypot(p.1 - h.1) >= apart_m)
        })
        .min_by(|a, b| d(a).total_cmp(&d(b)))
}

/// Visits, hours, marriages, people moved, purchases and trips that bought nothing between each
/// pair of settlements, all years.
fn contact_totals(sim: &Sim) -> Totals {
    let mut out = Totals::new();
    for (&(_, a, b), c) in &sim.people().contacts.years {
        let e = out.entry((a, b)).or_default();
        e.0 += c.visits;
        e.1 += c.minutes as f64 / 60.0;
        e.2 += c.marriages;
        e.3 += c.moved;
        e.4 += c.bought;
        e.5 += c.missed.iter().sum::<u32>();
    }
    out
}

/// A settlement's name, or "beyond the map".
fn place(sim: &Sim, s: Option<PermanentId>) -> String {
    s.and_then(|s| sim.land().settlements.iter().find(|x| x.id == s))
        .map_or_else(|| "beyond the map".to_owned(), |x| x.name.clone())
}

/// How a settlement was founded, in words.
fn founded_by(f: Founding) -> &'static str {
    match f {
        Founding::Setup => "a founding group",
        Founding::Sent => "a family the observer sent",
        Founding::Wave => "a migration wave",
        Founding::Coalition => "a coalition of households",
    }
}

/// Day `d`'s date.
fn day_date(d: i64) -> String {
    SimTime::from_minutes(d * MINUTES_PER_DAY)
        .date()
        .to_string()
}

/// "none", or the items joined.
fn listed(items: &[String]) -> String {
    if items.is_empty() {
        "none".to_owned()
    } else {
        items.join(", ")
    }
}

/// The person a coalition is named for, as its household.
fn coalition_who(sim: &Sim, named: Option<PermanentId>) -> String {
    named.map_or_else(
        || "a household".to_owned(),
        |p| format!("{}'s household", sim.people().name_of(p)),
    )
}

/// Year `y`'s lines: people, each settlement's accounts, contact, moves by reason, coalitions
/// and the wave, over `from` to `to`.
fn report_year(
    sim: &Sim,
    y: u32,
    from: SimTime,
    to: SimTime,
    contact_before: &Totals,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    let pop = sim.people();
    writeln!(out, "year {y} ({}): {} people", sim.date(), pop.living())?;
    for s in &sim.land().settlements {
        let a = pop.accounts(s.id, from, to);
        if a.start == 0 && a.end == 0 && a.arrived() == 0 && a.departed() == 0 {
            continue;
        }
        let came: Vec<String> = a
            .arrivals
            .iter()
            .map(|(&from, n)| format!("{n} from {}", place(sim, from)))
            .collect();
        let went: Vec<String> = a
            .departures
            .iter()
            .map(|(&to, n)| format!("{n} to {}", place(sim, to)))
            .collect();
        writeln!(
            out,
            "  {}: {} → {} ({} born, {} died; came: {}; went: {}){}{}",
            s.name,
            a.start,
            a.end,
            a.births,
            a.deaths,
            listed(&came),
            listed(&went),
            if a.balance() { "" } else { " UNBALANCED" },
            if s.abandoned.is_some_and(|t| from <= t && t < to) {
                " — abandoned"
            } else {
                ""
            },
        )?;
    }
    for (&(a, b), now) in &contact_totals(sim) {
        let was = contact_before.get(&(a, b)).copied().unwrap_or_default();
        let (v, h, m, mv) = (now.0 - was.0, now.1 - was.1, now.2 - was.2, now.3 - was.3);
        let (bought, missed) = (now.4 - was.4, now.5 - was.5);
        if v + m + mv + bought + missed == 0 {
            continue;
        }
        writeln!(
            out,
            "  {} → {}: {v} visits ({h:.0} h), {m} marriages, {mv} moved, {bought} purchases, \
             {missed} trips that bought nothing",
            place(sim, Some(a)),
            place(sim, Some(b))
        )?;
    }
    // Moves from a settlement to another or beyond the map, by reason.
    let mut moves: BTreeMap<(String, String, &'static str), u32> = BTreeMap::new();
    for r in pop.records.values() {
        for pair in r.residence.windows(2) {
            let (before, stay) = (&pair[0], &pair[1]);
            if !(from <= stay.since && stay.since < to)
                || before.settlement.is_none()
                || before.settlement == stay.settlement
            {
                continue;
            }
            *moves
                .entry((
                    place(sim, before.settlement),
                    place(sim, stay.settlement),
                    stay.why.words(),
                ))
                .or_default() += 1;
        }
    }
    for ((a, b, why), n) in &moves {
        writeln!(out, "  moved: {n} from {a} to {b}, {why}")?;
    }
    let (lo, hi) = (from.day_index(), to.day_index());
    for c in &pop.coalitions {
        let who = coalition_who(sim, c.named);
        if (lo..=hi).contains(&c.formed) {
            writeln!(
                out,
                "  coalition {}: {who} of {} began gathering on {}",
                c.id,
                place(sim, Some(c.from)),
                day_date(c.formed)
            )?;
        }
        if let Some(d) = c.ended.filter(|d| (lo..=hi).contains(d)) {
            let fate = match c.fate {
                CoalitionFate::Founded => format!("founded {}", place(sim, c.settlement)),
                _ => "gave up".to_owned(),
            };
            writeln!(
                out,
                "  coalition {}: {who} {fate} on {} ({} households, {} people)",
                c.id,
                day_date(d),
                c.members.len(),
                c.people
            )?;
        }
    }
    for w in &pop.influences.waves {
        writeln!(out, "  the wave: {}", wave_words(sim, w))?;
    }
    Ok(())
}

/// Where a wave's people are now.
fn wave_words(sim: &Sim, w: &civ_agents::influence::Wave) -> String {
    let pop = sim.people();
    let mut by: BTreeMap<String, u32> = BTreeMap::new();
    let (mut died, mut left) = (0, 0);
    for &p in &w.people {
        if let Some(q) = pop.person(p) {
            let home = pop.household(q.household).and_then(|x| x.settlement);
            *by.entry(place(sim, home)).or_default() += 1;
        } else if pop.records.get(&p).is_some_and(|r| r.died.is_some()) {
            died += 1;
        } else {
            left += 1;
        }
    }
    let living: Vec<String> = by.iter().map(|(s, n)| format!("{n} in {s}")).collect();
    format!(
        "{} of {} households came, {} people; living: {}; {died} died, {left} left the map",
        w.came,
        w.households,
        w.people.len(),
        listed(&living)
    )
}

/// The end: settlements, moves between them by reason, contact, the wave, every coalition, and
/// whether a splinter founding happened; then the checks.
fn report_end(sim: &Sim, outcome: &Outcome, out: &mut dyn Write) -> anyhow::Result<()> {
    let pop = sim.people();
    writeln!(out, "after {}: {} people", sim.date(), pop.living())?;
    for s in &sim.land().settlements {
        writeln!(
            out,
            "  {}: founded by {}{}, {} living there{}",
            s.name,
            founded_by(s.founding),
            s.parent
                .map_or_else(String::new, |p| format!(" from {}", place(sim, Some(p)))),
            pop.residents(s.id),
            s.abandoned
                .map_or_else(String::new, |t| format!(", abandoned {}", t.date())),
        )?;
    }
    let b = Between::of(sim);
    writeln!(
        out,
        "moves between settlements: {} with their household, {} to found one, {} on marrying, {} \
         taken in by kin, {} exiled",
        b.moved, b.founded, b.married, b.taken_in, b.exiled
    )?;
    for ((a, c), t) in contact_totals(sim) {
        writeln!(
            out,
            "  {} → {}: {} visits ({:.0} h), {} marriages, {} moved, {} purchases, {} trips that \
             bought nothing",
            place(sim, Some(a)),
            place(sim, Some(c)),
            t.0,
            t.1,
            t.2,
            t.3,
            t.4,
            t.5
        )?;
    }
    // What households believe of other settlements' offers (M5b slice AP).
    let held = pop.reports.held.values().map(Vec::len).sum::<usize>();
    let none_left = pop
        .reports
        .held
        .values()
        .flatten()
        .filter(|r| r.units <= 0.0)
        .count();
    writeln!(
        out,
        "price reports: {held} held by {} households, {none_left} of them that a seller had none \
         left",
        pop.reports.held.len()
    )?;
    if pop.influences.waves.is_empty() {
        writeln!(out, "no wave was sent")?;
    }
    for w in &pop.influences.waves {
        writeln!(out, "the wave: {}", wave_words(sim, w))?;
    }
    writeln!(out, "coalitions: {}", pop.coalitions.len())?;
    for c in &pop.coalitions {
        let members: Vec<String> = c
            .members
            .iter()
            .map(|&h| {
                pop.household(h)
                    .and_then(|x| x.members.first())
                    .map_or_else(|| format!("household {h} (gone)"), |&m| pop.name_of(m))
            })
            .collect();
        let lacked = match c.lacking {
            Lacking::Nothing => "",
            Lacking::Food => ", lacking food when last its time came",
            Lacking::Seed => ", lacking seed when last its time came",
        };
        let fate = match (c.fate, c.ended) {
            (CoalitionFate::Founded, Some(d)) => {
                format!("founded {} on {}", place(sim, c.settlement), day_date(d))
            }
            (CoalitionFate::Dissolved, Some(d)) => format!("gave up on {}", day_date(d)),
            _ => "still gathering".to_owned(),
        };
        writeln!(
            out,
            "  {}: {} of {}, began {}; {} households ({}), {} people at its last review, {} \
             reviews won{lacked}; {fate}",
            c.id,
            coalition_who(sim, c.named),
            place(sim, Some(c.from)),
            day_date(c.formed),
            c.members.len(),
            members.join(", "),
            c.people,
            c.reviews,
        )?;
    }
    if !pop
        .coalitions
        .iter()
        .any(|c| c.fate == CoalitionFate::Founded)
    {
        writeln!(
            out,
            "no splinter founding happened: {} coalitions gathered and none went",
            pop.coalitions.len()
        )?;
    }
    for path in &outcome.saves {
        writeln!(out, "kept {}", path.display())?;
    }
    if outcome.failures.is_empty() {
        writeln!(out, "every year's checks passed")?;
    } else {
        writeln!(out, "{} checks FAILED", outcome.failures.len())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    fn content() -> ContentRegistry {
        let root = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
            .expect("content/ is above the crate");
        civ_content::load(&root).registry.expect("content loads")
    }

    #[test]
    fn the_wave_goes_to_dry_ground_apart_from_every_settlement_and_founds_its_own() {
        let content = content();
        let mut sim = Sim::create_for_tests(
            &NewWorld {
                name: "Neighbours".into(),
                seed: 1,
                preset_id: PRESET.to_owned(),
                size_cells: 1024,
                band_size: 30,
                neighbours: vec![30, 30],
                neighbours_known: true,
                regime_id: String::new(),
            },
            &content,
            [5; 16],
        )
        .expect("world");
        let hearths: Vec<(f32, f32)> = sim.land().settlements.iter().map(|s| s.hearth_m).collect();
        assert_eq!(hearths.len(), 3, "three groups settled");
        let at = wave_point(&sim, WAVE_APART_M).expect("dry ground among them");
        for h in &hearths {
            assert!(
                (at.0 - h.0).hypot(at.1 - h.1) >= WAVE_APART_M,
                "{at:?} {h:?}"
            );
        }
        let options = NeighboursOptions {
            wave_households: 5,
            ..NeighboursOptions::default()
        };
        let mut said = Vec::new();
        send_wave(&mut sim, &options, &mut said).expect("written");
        let said = String::from_utf8(said).expect("words");
        assert!(said.contains("the observer sent a wave"), "{said}");
        let founded = sim
            .land()
            .settlements
            .iter()
            .find(|s| s.founding == Founding::Wave)
            .expect("the wave founded a settlement of its own");
        let d = (founded.hearth_m.0 - at.0).hypot(founded.hearth_m.1 - at.1);
        assert!(d < WAVE_APART_M, "{d} m from where it was sent");
    }
}
