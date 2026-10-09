//! The kernel's advance is the same however it is cut (ADR-0011 §5, Gate A): a world lived in one
//! stretch, in days, or saved at a midnight, loaded and lived on, ends in the same bytes; an
//! Accelerated clock stops only at midnight, where a change of mode takes effect; and Accelerated
//! mode without its declared approximations lives as Detailed mode does.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use civ_content::ContentRegistry;
use civ_core::time::MINUTES_PER_DAY;
use civ_sim::persist::{self, SECTION_META};
use civ_sim::{Approximations, Mode, NewWorld, SPEED_1X, SPEED_MAX, Sim};
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
            neighbours: Vec::new(),
            neighbours_known: false,
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
/// to a midnight, and saved there: where each test starts.
fn founded(dir: &Path) -> PathBuf {
    let mut sim = new_world(3);
    sim.advance_minutes(40 * MINUTES_PER_DAY).expect("advances");
    sim.advance_to_midnight().expect("advances");
    assert_eq!(sim.now().minute_of_day(), 0);
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
fn two_settlements_live_the_same_however_saved_or_cut() {
    // Two founding groups (ADR-0018 §6): the second settlement is lived as exactly as the first.
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let mut sim = Sim::create_for_tests(
        &NewWorld {
            name: "Neighbours".to_owned(),
            seed: 3,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 768,
            band_size: 0,
            neighbours: vec![0],
            neighbours_known: true,
            regime_id: String::new(),
        },
        content(),
        [7; 16],
    )
    .expect("generates");
    assert_eq!(sim.land().settlements.len(), 2);
    assert!(
        !sim.people().known_places.known.is_empty(),
        "each knows the other"
    );
    sim.advance_minutes(10 * MINUTES_PER_DAY).expect("advances");
    sim.advance_to_midnight().expect("advances");
    let start = save(&mut sim, dir.path(), "founded");
    let mut straight = load(&start);
    straight
        .advance_minutes(20 * MINUTES_PER_DAY)
        .expect("advances");
    let mut saved = load(&start);
    saved
        .advance_minutes(7 * MINUTES_PER_DAY)
        .expect("advances");
    let mid = save(&mut saved, dir.path(), "midnight");
    let mut resumed = load(&mid);
    resumed
        .advance_minutes(13 * MINUTES_PER_DAY)
        .expect("advances");
    assert_same("saved and loaded at a midnight", &straight, &resumed);
    let mut cut = load(&start);
    let (total, mut done) = (20 * MINUTES_PER_DAY, 0);
    while done < total {
        let step = 433.min(total - done);
        cut.advance_minutes(step).expect("advances");
        done += step;
    }
    assert_same("twenty days cut every 433 minutes", &straight, &cut);
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

#[test]
fn an_accelerated_clock_lives_a_day_at_a_time_and_stops_at_midnight() {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let mut sim = load(&founded(dir.path()));
    sim.set_paused(false);
    // At a midnight a change of mode takes effect at once.
    sim.set_speed(60.0 * SPEED_1X).expect("60x");
    assert_eq!(sim.mode(), Mode::Accelerated);
    let start = sim.now();
    // A real second at 60x is 5,760 simulated seconds: not yet a day.
    assert_eq!(sim.advance_real(1.0).expect("ok").minutes, 0);
    // Fourteen more make 86,400: a day, to the next midnight.
    let day = sim.advance_real(14.0).expect("ok");
    assert_eq!((day.days, day.minutes), (1, MINUTES_PER_DAY));
    assert_eq!(sim.now().minutes() - start.minutes(), MINUTES_PER_DAY);
    // Max lives a day each call, however short.
    sim.set_speed(SPEED_MAX).expect("Max");
    assert_eq!(sim.advance_real(0.001).expect("ok").days, 1);
    assert_eq!(sim.now().minute_of_day(), 0);
    // Back to 1x at a midnight: Detailed at once, by the minute.
    sim.set_speed(SPEED_1X).expect("1x");
    assert_eq!(sim.mode(), Mode::Detailed);
    assert_eq!(sim.advance_real(10.0).expect("ok").minutes, 16);
    // Between midnights a change of mode is held for the next one: an Accelerated speed first
    // lives the rest of the day, and the mode changes at midnight.
    sim.set_speed(600.0 * SPEED_1X).expect("600x");
    assert_eq!(sim.mode(), Mode::Detailed);
    let rest = sim.advance_real(0.05).expect("ok");
    assert_eq!(rest.minutes, MINUTES_PER_DAY - 16);
    assert_eq!(sim.now().minute_of_day(), 0);
    assert_eq!(sim.mode(), Mode::Accelerated);
}

/// Lives `days` days at `speed`, a real second a call (whole minutes of each in Detailed mode).
fn live_at(sim: &mut Sim, speed: f32, days: i64) {
    sim.set_speed(speed).expect("a speed");
    let until = sim.now().minutes() + days * MINUTES_PER_DAY;
    while sim.now().minutes() < until {
        sim.advance_real(1.0).expect("advances");
    }
    assert_eq!(sim.now().minutes(), until, "a stop at {speed} x/s overshot");
}

#[test]
fn detailed_then_accelerated_then_detailed_reproduces_exactly() {
    // Three days at 10x, ten at Max, three at 10x. Two such runs end in the same bytes, with
    // Accelerated mode's approximations (ADR-0011 §4) and without them; without them they end
    // where sixteen days at 10x do, so Accelerated mode is otherwise Detailed mode a day at a
    // time; and with them they do not, so the approximations are made.
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let start = founded(dir.path());
    let mixed = |approximations: Approximations| {
        let mut sim = load(&start);
        assert_eq!(sim.approximations(), Approximations::ACCELERATED);
        sim.set_approximations(approximations);
        sim.set_paused(false);
        live_at(&mut sim, 10.0 * SPEED_1X, 3);
        live_at(&mut sim, SPEED_MAX, 10);
        assert_eq!(sim.mode(), Mode::Accelerated);
        live_at(&mut sim, 10.0 * SPEED_1X, 3);
        assert_eq!(sim.mode(), Mode::Detailed);
        sim
    };
    let (a, b) = (
        mixed(Approximations::ACCELERATED),
        mixed(Approximations::ACCELERATED),
    );
    assert_same("two runs of Detailed, Accelerated, Detailed", &a, &b);
    let exact = mixed(Approximations::NONE);
    let mut detailed = load(&start);
    detailed.set_paused(false);
    live_at(&mut detailed, 10.0 * SPEED_1X, 16);
    assert_same(
        "without approximations, with Detailed throughout",
        &exact,
        &detailed,
    );
    assert!(
        !differing(
            &persist::encode_sections(&a),
            &persist::encode_sections(&exact)
        )
        .is_empty(),
        "leisure blocks change how the ten days are lived"
    );
}

#[test]
fn a_save_at_an_accelerated_midnight_lives_on_as_if_never_saved() {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let start = founded(dir.path());
    let mut straight = load(&start);
    straight.set_paused(false);
    live_at(&mut straight, SPEED_MAX, 12);
    let mut first = load(&start);
    first.set_paused(false);
    live_at(&mut first, SPEED_MAX, 5);
    let mid = save(&mut first, dir.path(), "accelerated");
    let mut resumed = load(&mid);
    assert_eq!(
        resumed.mode(),
        Mode::Accelerated,
        "the speed is kept, and so the mode"
    );
    resumed.set_paused(false);
    live_at(&mut resumed, SPEED_MAX, 7);
    assert_same("saved at an Accelerated midnight", &straight, &resumed);
}
