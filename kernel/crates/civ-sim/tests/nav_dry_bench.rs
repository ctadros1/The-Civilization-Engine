//! Measurement harness (experimental, ignored): one saved world lived the same span with
//! landmarks built on the flooded walking grid (as now) and on its dry grid
//! (`TCE_DRY_LANDMARKS=1`), for the owner's 2026-11-20 question on a faster walking grid that
//! keeps water's restrictions. Run each mode in its own process:
//!
//! ```sh
//! NAV_BENCH_SAVE=/path/w.tcesave NAV_BENCH_OUT=/path/base.txt \
//!   cargo test --profile simcheck -p civ-sim --test nav_dry_bench -- --ignored --nocapture
//! TCE_DRY_LANDMARKS=1 NAV_BENCH_SAVE=/path/w.tcesave NAV_BENCH_OUT=/path/dry.txt \
//!   cargo test --profile simcheck -p civ-sim --test nav_dry_bench -- --ignored --nocapture
//! ```
//!
//! The save is made on the first run if it is missing (`NAV_BENCH_SEED`, default 3; the
//! dashboard's river valley at 1024² cells, lived `NAV_BENCH_PRE_DAYS`, default 30).

use std::fmt::Write as _;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::Instant;

use civ_content::ContentRegistry;
use civ_sim::persist::{self, SECTION_META};
use civ_sim::{NewWorld, Sim};
use commons_persist::{SaveDir, SaveKind};

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

#[test]
#[ignore = "a measurement, run by hand"]
fn live_one_save_and_report() {
    let root = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
        .expect("content/ is above the crate");
    let content: ContentRegistry = civ_content::load(&root).registry.expect("content loads");
    let save = PathBuf::from(std::env::var("NAV_BENCH_SAVE").expect("NAV_BENCH_SAVE"));
    let out = PathBuf::from(std::env::var("NAV_BENCH_OUT").expect("NAV_BENCH_OUT"));
    let days = env_u64("NAV_BENCH_DAYS", 360);
    if !save.exists() {
        let mut sim = Sim::create_for_tests(
            &NewWorld {
                name: "Nav bench".to_owned(),
                seed: env_u64("NAV_BENCH_SEED", 3),
                preset_id: "core:worldgen/river_valley".to_owned(),
                size_cells: 1024,
                band_size: 0,
                neighbours: Vec::new(),
                neighbours_known: false,
                regime_id: String::new(),
            },
            &content,
            [3; 16],
        )
        .expect("generates");
        sim.advance_minutes(env_u64("NAV_BENCH_PRE_DAYS", 30) as i64 * 24 * 60)
            .expect("advances");
        let dir = save.parent().expect("a folder");
        let saves = SaveDir::create(dir, civ_schema::SAVE_EXTENSION).expect("save dir");
        let saved = persist::save(&mut sim, &saves, SaveKind::Manual, "navbench").expect("saves");
        std::fs::rename(&saved.path, &save).expect("moved to NAV_BENCH_SAVE");
    }
    let mode = if std::env::var_os("TCE_DRY_LANDMARKS").is_some() {
        "dry"
    } else {
        "flooded"
    };
    let mut sim = persist::load(&save, &content).expect("loads");
    let mut report = String::new();
    writeln!(report, "mode {mode}, {days} days from {}", save.display()).ok();
    writeln!(
        report,
        "month  wall_s  living  flood_rev  landmark_builds  landmark_s  route_clears"
    )
    .ok();
    let started = Instant::now();
    let mut lived = 0;
    while lived < days {
        let step = 30.min(days - lived);
        sim.advance_minutes(step as i64 * 24 * 60).expect("advances");
        lived += step;
        let (builds, secs) = civ_world::nav::landmark_builds();
        let line = format!(
            "{:>5}  {:>6.1}  {:>6}  {:>9}  {:>15}  {:>10.1}  {:>12}",
            lived / 30,
            started.elapsed().as_secs_f64(),
            sim.people().living(),
            sim.land().flood_revision,
            builds,
            secs,
            sim.people().route_counts.0,
        );
        println!("{line}");
        writeln!(report, "{line}").ok();
    }
    writeln!(report, "sections (tag index digest):").ok();
    for s in persist::encode_sections(&sim) {
        if s.tag == SECTION_META {
            continue;
        }
        let mut h = std::collections::hash_map::DefaultHasher::new();
        s.bytes.hash(&mut h);
        writeln!(report, "{} {} {:016x}", s.tag, s.index, h.finish()).ok();
    }
    std::fs::write(&out, report).expect("writes NAV_BENCH_OUT");
}
