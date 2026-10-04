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

use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

use civ_core::time::MINUTES_PER_DAY;
use civ_sim::{NewWorld, Sim, persist};
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
    };
    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        if flag == "--digest" {
            a.digest = true;
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
    for day in 1..=a.days {
        let t = Instant::now();
        if day > a.profile_after && a.profile_after > 0 {
            profiled_day(&mut sim);
        } else {
            sim.advance_minutes(MINUTES_PER_DAY)
                .expect("the world lives on");
        }
        let ms = t.elapsed().as_secs_f64() * 1000.0;
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
