//! How fast a village of a given size lives (M3a slice L): founds a band of `--people` (beyond the
//! content's band limits if asked; the content itself is not changed), then lives `--days` days at
//! full detail and prints the wall time per simulated day, in windows, with the village's size.
//!
//! ```sh
//! cargo run --release -p civ-sim --example bench_village -- --people 200 --days 30
//! ```
//!
//! Runs differ by world, as every world draws its own identity; compare several runs, or profile
//! one (for example under `valgrind --tool=callgrind --toggle-collect='*advance_minutes*'`).
//! `--save-to DIR` saves the world when the days are lived, and `--load FILE` lives on from a
//! save instead of making a world, so a profile can start in the middle of a year. Days after
//! `--profile-after N` are lived through `profiled_day`, for
//! `valgrind --tool=callgrind --toggle-collect='*profiled_day*'` to measure them alone.
//! `--digest` prints a digest of every section a save would hold, to check that a change meant
//! to make the engine faster leaves what happens exactly as it was. `--families N` sends N
//! families to the village once it is founded, in groups as the observer's god tool sends them.
//! `--max` lives the days at Max, in Accelerated mode from the first midnight with the
//! approximations it declares (ADR-0011 §4); `--exact` with it switches them off, and `--only
//! leisure` or `--only view` keeps one. `--digest` also prints how people spent their time.
//! `--groups N` founds N groups of `--people` each, their sites chosen together (ADR-0018 §6);
//! `--known` has them know where each other camped. `--stop-trade` stops purchases between
//! settlements (the twin harness, ADR-0019 §8), and `--no-sights` has nobody note the buildings
//! they see in other settlements (M5b slice AR).

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

use civ_core::time::MINUTES_PER_DAY;
use civ_sim::{NewWorld, SPEED_MAX, Sim, persist};
use commons_persist::{SaveDir, SaveKind};

struct Args {
    people: u32,
    days: u32,
    seed: u64,
    size: u32,
    regime: String,
    window: u32,
    save_to: Option<String>,
    load: Option<String>,
    profile_after: u32,
    digest: bool,
    families: u32,
    max: bool,
    exact: bool,
    only: String,
    groups: u32,
    known: bool,
    stop_trade: bool,
    no_sights: bool,
}

fn args() -> Args {
    let mut a = Args {
        people: 200,
        days: 30,
        seed: 2,
        size: 768,
        regime: String::new(),
        window: 10,
        save_to: None,
        load: None,
        profile_after: 0,
        digest: false,
        families: 0,
        max: false,
        exact: false,
        only: String::new(),
        groups: 1,
        known: false,
        stop_trade: false,
        no_sights: false,
    };
    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        if flag == "--digest" {
            a.digest = true;
            continue;
        }
        if flag == "--max" {
            a.max = true;
            continue;
        }
        if flag == "--exact" {
            a.exact = true;
            continue;
        }
        if flag == "--known" {
            a.known = true;
            continue;
        }
        if flag == "--stop-trade" {
            a.stop_trade = true;
            continue;
        }
        // Nobody notes buildings seen elsewhere (M5b slice AR): the content's `seen_most` 0.
        if flag == "--no-sights" {
            a.no_sights = true;
            continue;
        }
        let value = it.next().unwrap_or_default();
        match flag.as_str() {
            "--people" => a.people = value.parse().expect("--people N"),
            "--days" => a.days = value.parse().expect("--days N"),
            "--seed" => a.seed = value.parse().expect("--seed N"),
            "--size" => a.size = value.parse().expect("--size CELLS"),
            "--regime" => a.regime = value,
            "--window" => a.window = value.parse().expect("--window DAYS"),
            "--save-to" => a.save_to = Some(value),
            "--load" => a.load = Some(value),
            "--profile-after" => a.profile_after = value.parse().expect("--profile-after DAYS"),
            "--families" => a.families = value.parse().expect("--families N"),
            "--only" => a.only = value,
            "--groups" => a.groups = value.parse().expect("--groups N"),
            other => panic!("unknown flag {other}"),
        }
    }
    a
}

/// One simulated day, where a profiler can see it alone.
#[inline(never)]
fn profiled_day(sim: &mut Sim) {
    sim.advance_minutes(MINUTES_PER_DAY)
        .expect("the world lives on");
}

fn main() {
    let a = args();
    let root = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
        .expect("content/ is above the crate");
    let mut content = civ_content::load(&root)
        .registry
        .expect("the content loads");
    let band = &mut content.people.params.band;
    band.max_size = band.max_size.max(a.people);
    band.min_size = band.min_size.min(a.people);
    if a.no_sights {
        content.people.params.style.seen_most = 0;
    }
    let started = Instant::now();
    let mut sim = match &a.load {
        Some(file) => persist::load(Path::new(file), &content).expect("the save loads"),
        None => Sim::create(
            &NewWorld {
                name: "Bench".to_owned(),
                seed: a.seed,
                preset_id: content.default_preset().id.clone(),
                size_cells: a.size,
                band_size: a.people,
                neighbours: vec![a.people; a.groups.saturating_sub(1) as usize],
                neighbours_known: a.known,
                regime_id: a.regime.clone(),
            },
            &content,
            &mut |_| {},
            &AtomicBool::new(false),
        )
        .expect("the world generates"),
    };
    if let Some(problem) = sim.founding_problem() {
        panic!("the band could not settle: {problem}");
    }
    if a.max {
        sim.set_speed(SPEED_MAX).expect("Max is a speed");
    }
    if a.exact {
        sim.set_approximations(civ_sim::Approximations::NONE);
    }
    if a.stop_trade {
        sim.stop_trade_between(true);
    }
    match a.only.as_str() {
        "" => {}
        "leisure" => sim.set_approximations(civ_sim::Approximations {
            leisure_blocks: true,
            household_view: false,
        }),
        "view" => sim.set_approximations(civ_sim::Approximations {
            leisure_blocks: false,
            household_view: true,
        }),
        other => panic!("--only leisure or view, not {other}"),
    }
    if a.families > 0 {
        let people = sim.send_families_to_hearth(a.families);
        println!("{} families sent: {people} people came", a.families);
    }
    println!(
        "seed {} on {} cells: {} people in {} households, made in {:.1} s",
        a.seed,
        a.size,
        sim.people().living(),
        sim.people()
            .households
            .iter()
            .filter(|(_, h)| !h.members.is_empty())
            .count(),
        started.elapsed().as_secs_f64()
    );
    let lived = Instant::now();
    let mut window = Instant::now();
    let (mut slowest, mut slowest_day) = (0.0f64, 0);
    // Errands to fetch goods to sell at home (M5b slice AQ, step two), as planned: those still
    // waiting at a day's end, by household and day planned (a lower bound: one planned and run
    // within a day is not seen).
    let mut errands: BTreeMap<(u64, i64), String> = BTreeMap::new();
    // Buildings of other settlements people saw (M5b slice AR), as held at a day's end, by person
    // and building; and each settlement's mean taste at the start.
    let mut sights: std::collections::BTreeSet<(u64, u64)> = Default::default();
    let tastes_before = mean_tastes(&sim);
    let lived_from = sim.now().minutes();
    for day in 1..=a.days {
        let t = Instant::now();
        if day > a.profile_after && a.profile_after > 0 {
            profiled_day(&mut sim);
        } else {
            sim.advance_minutes(MINUTES_PER_DAY)
                .expect("the world lives on");
        }
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if a.digest {
            for (p, list) in &sim.people().seen_away {
                sights.extend(list.iter().map(|b| (p.get(), b.get())));
            }
            for (h, e) in &sim.people().reports.errands {
                errands
                    .entry((h.get(), e.day))
                    .or_insert_with(|| sim.rules().catalog.goods[usize::from(e.good)].id.clone());
            }
        }
        if ms > slowest {
            (slowest, slowest_day) = (ms, day);
        }
        if day % a.window == 0 || day == a.days {
            let n = (day - 1) % a.window + 1;
            println!(
                "days {:>4}-{:<4} {:>8.1} ms a day; {} people, {} fields, {} buildings",
                day + 1 - n,
                day,
                window.elapsed().as_secs_f64() * 1000.0 / f64::from(n),
                sim.people().living(),
                sim.land().fields.len(),
                sim.land().buildings.len(),
            );
            window = Instant::now();
        }
    }
    let total = lived.elapsed().as_secs_f64();
    if a.digest {
        use std::hash::{DefaultHasher, Hash, Hasher};
        for section in persist::encode_sections(&sim) {
            let mut h = DefaultHasher::new();
            section.bytes.hash(&mut h);
            println!(
                "digest {} {} {:016x}",
                section.tag,
                section.index,
                h.finish()
            );
        }
    }
    if a.digest {
        // Hours a person-day by what people did (every finished step's minutes).
        let t = &sim.people().time_use;
        let total: f64 = t.work.iter().sum::<f64>() + t.walking + t.waiting;
        let h = |m: f64| 24.0 * m / total.max(1.0);
        let mut line = String::from("time use, hours a person-day:");
        for (b, m) in civ_agents::Behavior::ALL.iter().zip(&t.work) {
            if *m > 0.0 {
                line.push_str(&format!(" {} {:.3}", b.name(), h(*m)));
            }
        }
        println!(
            "{line} walking {:.3} waiting {:.3}",
            h(t.walking),
            h(t.waiting)
        );
        // Visits between settlements (M5a slice AM).
        let (visits, minutes, marriages, moved) = sim
            .people()
            .contacts
            .years
            .values()
            .fold((0u32, 0u64, 0u32, 0u32), |(v, m, w, d), c| {
                (v + c.visits, m + c.minutes, w + c.marriages, d + c.moved)
            });
        let left = sim
            .people()
            .records
            .values()
            .filter(|r| r.left.is_some())
            .count();
        println!(
            "contacts: {visits} visits, {:.0} hours at others' hearths, {marriages} marriages, \
             {moved} people moved between settlements, {left} left the map",
            minutes as f64 / 60.0
        );
        // Buying between settlements by report (M5b slice AP).
        let pop = sim.people();
        let (bought, missed) = pop.contacts.years.values().fold(
            (0u32, [0u32; civ_agents::reports::Missed::COUNT]),
            |(b, mut m), c| {
                for (x, n) in m.iter_mut().zip(c.missed) {
                    *x += n;
                }
                (b + c.bought, m)
            },
        );
        println!(
            "reports: {} held by {} households; {bought} purchases between settlements, trips \
             that bought nothing: {} sold out, {} terms, {} payment, {} no longer needed",
            pop.reports.held.values().map(Vec::len).sum::<usize>(),
            pop.reports.held.len(),
            missed[0],
            missed[1],
            missed[2],
            missed[3]
        );
        // What the latest trades each market remembers sold to buyers from elsewhere.
        let mut from_elsewhere: BTreeMap<&str, usize> = BTreeMap::new();
        for t in pop
            .markets
            .iter()
            .flat_map(|m| m.recent.iter())
            .filter(|t| t.from.is_some())
        {
            let id = &sim.rules().catalog.goods[usize::from(t.good)].id;
            *from_elsewhere.entry(id.as_str()).or_default() += 1;
        }
        println!("bought by people of other settlements, latest trades: {from_elsewhere:?}");
        // What people carried home from other settlements' markets, all months (ADR-0019 §7).
        let (mut trips, mut walk_h) = (0u32, 0.0f64);
        let mut carried: BTreeMap<&str, f64> = BTreeMap::new();
        for c in pop.convergence.carried.values() {
            trips += c.trips;
            walk_h += f64::from(c.walk_h);
            for &(g, u) in &c.goods {
                let id = &sim.rules().catalog.goods[usize::from(g)].id;
                *carried.entry(id.as_str()).or_default() += f64::from(u);
            }
        }
        println!(
            "carried between settlements: {trips} trips, {walk_h:.0} hours walked, units {carried:?}"
        );
        let mut planned: BTreeMap<&str, usize> = BTreeMap::new();
        for g in errands.values() {
            *planned.entry(g.as_str()).or_default() += 1;
        }
        println!(
            "errands to sell at home seen planned: {} by {} households: {planned:?}",
            errands.len(),
            errands
                .keys()
                .map(|k| k.0)
                .collect::<std::collections::BTreeSet<_>>()
                .len()
        );
        // Diffusion through contact (M5b slice AR): what was seen elsewhere, the households whose
        // taste a building elsewhere moved most, and how far apart settlements' tastes are.
        let admired_away = pop
            .households
            .iter()
            .filter(|(_, h)| !h.members.is_empty())
            .filter(|(_, h)| {
                h.admired
                    .and_then(|b| sim.land().buildings.iter().find(|x| x.id == b))
                    .and_then(|b| pop.household(b.household))
                    .is_some_and(|o| o.settlement != h.settlement)
            })
            .count();
        println!(
            "buildings seen elsewhere: {} sightings by {} people of {} buildings; {admired_away} \
             households admire a building elsewhere",
            sights.len(),
            sights
                .iter()
                .map(|s| s.0)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            sights
                .iter()
                .map(|s| s.1)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
        );
        // Techniques known of by sight elsewhere, and the record's arrivals from and losses with
        // another settlement (M5b slice AR, step two).
        let mut seen_of: BTreeMap<&str, usize> = BTreeMap::new();
        for (_, p) in pop.people.iter() {
            for k in &p.knows {
                if matches!(k.source, civ_agents::person::KnowSource::Seen(_)) && !k.known {
                    let id = &sim.rules().catalog.techniques[usize::from(k.technique)].id;
                    *seen_of.entry(id.as_str()).or_default() += 1;
                }
            }
        }
        let (brought, lost_near) = pop.knowledge.iter().filter(|e| e.elsewhere.is_some()).fold(
            (0, 0),
            |(b, l), e| match e.kind {
                civ_agents::knowledge::KnowledgeEventKind::Lost => (b, l + 1),
                _ => (b + 1, l),
            },
        );
        println!(
            "techniques known of by sight elsewhere, people: {seen_of:?}; the record: {brought} \
             brought from another settlement, {lost_near} lost while known where kin or friends live"
        );
        // The record's entries made while the days were lived, by kind and technique.
        let mut made: BTreeMap<String, usize> = BTreeMap::new();
        for e in pop.knowledge.iter().filter(|e| e.at.minutes() > lived_from) {
            let what = match e.kind {
                civ_agents::knowledge::KnowledgeEventKind::Lost => "lost",
                _ => "known",
            };
            let t = &sim.rules().catalog.techniques[usize::from(e.technique)].id;
            let key = format!(
                "{what} {t}{}",
                if e.elsewhere.is_some() {
                    " (elsewhere)"
                } else {
                    ""
                }
            );
            *made.entry(key).or_default() += 1;
        }
        println!("knowledge record entries made: {made:?}");
        if std::env::var_os("BENCH_KNOWLEDGE").is_some() {
            for e in pop.knowledge.iter().filter(|e| e.at.minutes() > lived_from) {
                println!(
                    "  {} settlement {} technique {} person {} {:?} elsewhere {:?}",
                    e.at, e.settlement, e.technique, e.person, e.kind, e.elsewhere
                );
            }
        }
        let tastes_after = mean_tastes(&sim);
        let keys: Vec<u64> = tastes_after.keys().copied().collect();
        for (i, x) in keys.iter().enumerate() {
            for y in keys.iter().skip(i + 1) {
                let gap = |m: &BTreeMap<u64, [f64; 3]>| {
                    let (Some(a), Some(b)) = (m.get(x), m.get(y)) else {
                        return f64::NAN;
                    };
                    a.iter()
                        .zip(b)
                        .map(|(p, q)| (p - q).powi(2))
                        .sum::<f64>()
                        .sqrt()
                };
                println!(
                    "taste {x}-{y}: mean apart {:.1} at the start, {:.1} at the end \
                     (centidegrees and centimetres)",
                    gap(&tastes_before),
                    gap(&tastes_after)
                );
            }
        }
        // The convergence record (M5b slice AQ): per pair, the goods offered in both and the mean
        // of |log(ask there / ask here)| over them, in the first and last months recorded.
        // By pair: (month, goods offered in both, the mean gap) for each month on record.
        type Months = Vec<(u32, usize, f64)>;
        let mut pairs: BTreeMap<(u64, u64), Months> = BTreeMap::new();
        for (&(month, x, y), gaps) in &pop.convergence.gaps {
            let mean = gaps
                .iter()
                .map(|g| (f64::from(g.ask_h[0]) / f64::from(g.ask_h[1])).ln().abs())
                .sum::<f64>()
                / gaps.len().max(1) as f64;
            pairs
                .entry((x.get(), y.get()))
                .or_default()
                .push((month, gaps.len(), mean));
        }
        for ((x, y), months) in &pairs {
            let first: Vec<String> = months
                .iter()
                .take(3)
                .map(|(m, n, g)| format!("m{m} {n} goods {g:.3}"))
                .collect();
            let last: Vec<String> = months
                .iter()
                .rev()
                .take(3)
                .rev()
                .map(|(m, n, g)| format!("m{m} {n} goods {g:.3}"))
                .collect();
            println!(
                "convergence {x}-{y}: {} months; first {}; last {}",
                months.len(),
                first.join(", "),
                last.join(", ")
            );
        }
        // Ties (ADR-0014): how many, and what last moved them.
        let ties = &sim.people().ties;
        let holders = ties.holders();
        let most = holders.iter().map(|x| ties.of(*x).len()).max().unwrap_or(0);
        let mut by_act = [0usize; civ_agents::ties::ACTS];
        for x in &holders {
            for t in ties.of(*x) {
                if let Some(r) = t.reason {
                    by_act[r.act as usize] += 1;
                }
            }
        }
        let mut line = format!(
            "ties: {} held by {} of {} people ({:.1} each, most {most}), {} let go; reasons:",
            ties.len(),
            holders.len(),
            sim.people().living(),
            ties.len() as f64 / holders.len().max(1) as f64,
            ties.let_go,
        );
        for (act, n) in civ_agents::ties::Act::ALL.iter().zip(by_act) {
            if n > 0 {
                line.push_str(&format!(" {} {n}", act.name()));
            }
        }
        println!("{line}");
        // Standing (ADR-0014 §3-4), as last worked out.
        let table = &sim.people().standing;
        for s in &sim.land().settlements {
            let rows: Vec<_> = table.in_settlement(s.id).collect();
            let esteemed = rows.iter().filter(|r| r.total() > 0.05).count();
            let most = rows.iter().map(|r| r.influence).max().unwrap_or(0);
            let notables: Vec<String> = rows
                .iter()
                .filter(|r| r.notable)
                .map(|r| {
                    format!(
                        "{} (influence {}, esteem {:.1})",
                        sim.people().name_of(r.person),
                        r.influence,
                        r.total()
                    )
                })
                .collect();
            println!(
                "standing of {}: {} adults, {esteemed} esteemed by others, most influence {most}; notables: {}",
                s.name,
                rows.len(),
                notables.join(", ")
            );
        }
    }
    if let Some(dir) = &a.save_to {
        let saves = SaveDir::create(Path::new(dir), civ_schema::SAVE_EXTENSION).expect("save dir");
        let saved = persist::save(&mut sim, &saves, SaveKind::Manual, "bench").expect("saves");
        println!("saved {}", saved.path.display());
    }
    println!(
        "{} days in {:.1} s: {:.1} ms a day on average ({:.1} s a year); slowest day {} at {:.0} ms",
        a.days,
        total,
        total * 1000.0 / f64::from(a.days.max(1)),
        total * 365.0 / f64::from(a.days.max(1)),
        slowest_day,
        slowest
    );
}

/// Each settlement's mean taste over its households with anyone in them: pitch (centidegrees),
/// eaves (cm) and overhang (cm).
fn mean_tastes(sim: &Sim) -> BTreeMap<u64, [f64; 3]> {
    let mut sum: BTreeMap<u64, ([f64; 3], f64)> = BTreeMap::new();
    for (_, h) in sim.people().households.iter() {
        let Some(s) = h.settlement.filter(|_| !h.members.is_empty()) else {
            continue;
        };
        let e = sum.entry(s.get()).or_default();
        for (x, t) in e.0.iter_mut().zip(h.taste.traits()) {
            *x += f64::from(t);
        }
        e.1 += 1.0;
    }
    sum.into_iter()
        .map(|(s, (t, n))| (s, t.map(|x| x / n)))
        .collect()
}
