//! The kernel's advance is the same however it is cut (ADR-0011 §5, Gate A): a world lived in one
//! stretch, in days, or saved at a midnight, loaded and lived on, ends in the same bytes.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use civ_content::ContentRegistry;
use civ_core::time::MINUTES_PER_DAY;
use civ_sim::persist::{self, SECTION_META};
use civ_sim::{NewWorld, Sim};
use commons_persist::{SaveDir, SaveKind, SectionData};

fn content() -> &'static ContentRegistry {
    static CONTENT: OnceLock<ContentRegistry> = OnceLock::new();
    CONTENT.get_or_init(|| {
        let root = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
            .expect("content/ is above the crate");
        civ_content::load(&root)
            .registry
            .expect("the core content loads")
    })
}

/// World `seed` with a fixed identity, so it lives the same life every run (its identity orders
/// the events of an instant): what holds here holds for any.
fn new_world(seed: u64) -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Modes".to_owned(),
            seed,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 512,
            band_size: 0,
            regime_id: String::new(),
        },
        content(),
        [7; 16],
    )
    .expect("generates")
}

fn save(sim: &mut Sim, dir: &Path, label: &str) -> PathBuf {
    let saves = SaveDir::create(dir, civ_schema::SAVE_EXTENSION).expect("save dir");
    persist::save(sim, &saves, SaveKind::Manual, label)
        .expect("saves")
        .path
}

/// The sections whose bytes differ, but for the save's own metadata.
fn differing(a: &[SectionData], b: &[SectionData]) -> Vec<String> {
    if a.len() != b.len() {
        return vec![format!("{} sections against {}", a.len(), b.len())];
    }
    a.iter()
        .zip(b)
        .filter(|(x, y)| x.tag != SECTION_META && (x.tag != y.tag || x.bytes != y.bytes))
        .map(|(x, _)| format!("{} {}", x.tag, x.index))
        .collect()
}

/// Panics naming the sections in which `a` and `b` differ.
fn assert_same(what: &str, a: &Sim, b: &Sim) {
    let d = differing(&persist::encode_sections(a), &persist::encode_sections(b));
    assert!(d.is_empty(), "{what}: sections differ: {d:?}");
}

/// A river-valley world lived 40 days from its founding (across a month's survey of the paths)
/// and saved at midnight: where each test starts.
fn founded(dir: &Path) -> PathBuf {
    let mut sim = new_world(3);
    sim.advance_minutes(40 * MINUTES_PER_DAY).expect("advances");
    save(&mut sim, dir, "founded")
}

fn load(path: &Path) -> Sim {
    persist::load(path, content()).expect("loads")
}

#[test]
fn a_world_saved_at_midnight_lives_on_as_if_never_saved() {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let start = founded(dir.path());
    let mut straight = load(&start);
    straight
        .advance_minutes(40 * MINUTES_PER_DAY)
        .expect("advances");
    let mut saved = load(&start);
    saved
        .advance_minutes(11 * MINUTES_PER_DAY)
        .expect("advances");
    let mid = save(&mut saved, dir.path(), "midnight");
    let mut resumed = load(&mid);
    resumed
        .advance_minutes(29 * MINUTES_PER_DAY)
        .expect("advances");
    assert_same("saved and loaded at a midnight", &straight, &resumed);
}

#[test]
fn a_stretch_lived_at_once_by_the_day_or_cut_anywhere_ends_the_same() {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let start = founded(dir.path());
    let total = 30 * MINUTES_PER_DAY;
    let mut once = load(&start);
    once.advance_minutes(total).expect("advances");
    let mut daily = load(&start);
    for _ in 0..30 {
        daily.advance_minutes(MINUTES_PER_DAY).expect("advances");
    }
    assert_same("thirty days one at a time", &once, &daily);
    // Cut at odd minutes, as a paced clock cuts Detailed time.
    let mut cut = load(&start);
    let mut done = 0;
    while done < total {
        let step = 433.min(total - done);
        cut.advance_minutes(step).expect("advances");
        done += step;
    }
    assert_same("thirty days cut every 433 minutes", &once, &cut);
}
