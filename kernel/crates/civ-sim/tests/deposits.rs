//! Deposits in the ground (ADR-0010 §1; M3b slice Q): found by walking near them, laid down by the
//! observer, and kept across a save.

use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

use civ_agents::history::ChronicleKind;
use civ_content::ContentRegistry;
use civ_sim::persist;
use civ_sim::{NewWorld, Sim};
use commons_persist::{SaveDir, SaveKind};

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

fn world(seed: u64) -> Sim {
    Sim::create(
        &NewWorld {
            name: "Deposits".to_owned(),
            seed,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 512,
            band_size: 0,
            regime_id: String::new(),
        },
        content(),
        &mut |_| {},
        &AtomicBool::new(false),
    )
    .expect("generates")
}

/// Chronicle entries of `kind`, as (where, words).
fn entries(sim: &Sim, kind: ChronicleKind) -> Vec<(Option<(f32, f32)>, String)> {
    sim.people()
        .chronicle
        .iter()
        .filter(|e| e.kind == kind)
        .map(|e| (e.place, e.name.clone()))
        .collect()
}

#[test]
fn clay_showing_by_the_hearth_is_found_and_clay_under_the_ground_is_not() {
    let mut sim = world(3);
    let (settlement, hearth) = {
        let s = sim.land().settlements.first().expect("a settlement");
        (s.id, s.hearth_m)
    };
    let beside = (hearth.0 + 20.0, hearth.1);
    let below = (hearth.0 - 20.0, hearth.1);
    let shown = sim
        .place_deposit(beside, "core:good/clay", 5.0, true)
        .expect("laid down");
    let buried = sim
        .place_deposit(below, "core:good/clay", 5.0, false)
        .expect("laid down");
    // The observer's acts are in the chronicle.
    let placed = entries(&sim, ChronicleKind::DepositPlaced);
    assert_eq!(
        placed,
        vec![
            (Some(beside), "clay showing at the surface".to_owned()),
            (Some(below), "clay under the ground".to_owned()),
        ]
    );
    // Within a few days someone walking by the hearth sees the clay that shows, once.
    sim.advance_minutes(3 * 24 * 60).expect("advances");
    assert!(sim.people().knows_deposit(settlement, shown));
    assert!(!sim.people().knows_deposit(settlement, buried));
    let found = entries(&sim, ChronicleKind::DepositFound);
    let here: Vec<_> = found.iter().filter(|(at, _)| *at == Some(beside)).collect();
    assert_eq!(here.len(), 1, "{found:?}");
    assert_eq!(here[0].1, "clay showing at the surface");
    let k = sim
        .people()
        .deposits_known
        .iter()
        .find(|k| k.deposit == shown)
        .expect("known");
    let finder_home = sim
        .people()
        .households
        .iter()
        .find(|(_, h)| h.members.contains(&k.finder))
        .and_then(|(_, h)| h.settlement);
    assert_eq!(finder_home, Some(settlement), "a member found it");
    // What it knows and what lies in the ground survive a save and load.
    let dir = tempfile::tempdir().expect("tempdir");
    let saves = SaveDir::create(dir.path().join("saves"), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = persist::save(&mut sim, &saves, SaveKind::Manual, "deposits").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loaded.people().deposits_known, sim.people().deposits_known);
    assert_eq!(loaded.land().deposits, sim.land().deposits);
}

#[test]
fn the_observer_lays_down_deposits_only_where_they_can_lie() {
    let mut sim = world(3);
    let hearth = sim.land().settlements[0].hearth_m;
    assert!(
        sim.place_deposit(hearth, "core:good/nothing", 5.0, true)
            .is_err()
    );
    assert!(
        sim.place_deposit(hearth, "core:good/clay", 0.5, true)
            .is_err()
    );
    assert!(
        sim.place_deposit((-10.0, 5.0), "core:good/clay", 5.0, true)
            .is_err()
    );
    let map = sim.map();
    let wet = map
        .water
        .iter()
        .position(|&w| w != civ_world::WATER_LAND)
        .expect("the valley has water");
    let at = (
        ((wet % map.width as usize) as f32 + 0.5) * map.cell_size_m,
        ((wet / map.width as usize) as f32 + 0.5) * map.cell_size_m,
    );
    assert!(sim.place_deposit(at, "core:good/clay", 5.0, true).is_err());
    // A body's inventory is its volume times the density the land profile gives its good.
    let id = sim
        .place_deposit(hearth, "core:good/stone", 10.0, true)
        .expect("laid down");
    let d = sim
        .land()
        .deposits
        .iter()
        .find(|d| d.id == id)
        .expect("there");
    let t = f64::from(d.body.thickness_cm) / 100.0;
    let v = std::f64::consts::PI * 10.0 * 10.0 * t * 2_500.0;
    assert!((d.body.initial_kg - v).abs() < 0.01 * v, "{d:?}");
    assert_eq!(d.left_kg(), d.body.initial_kg);
}
