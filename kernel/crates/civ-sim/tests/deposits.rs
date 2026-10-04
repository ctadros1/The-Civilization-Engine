//! Deposits in the ground (ADR-0010 §1-2; M3b slice Q): found by walking near them, laid down by
//! the observer, dug at a pit for clay that is made into pots, and kept across a save.

use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

use civ_agents::history::ChronicleKind;
use civ_agents::person::{Step, Target};
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

#[test]
fn a_household_short_of_good_room_digs_clay_it_knows_and_makes_pots() {
    let mut sim = world(3);
    let (settlement, hearth) = {
        let s = sim.land().settlements.first().expect("a settlement");
        (s.id, s.hearth_m)
    };
    // Clay showing beside the hearth, which the village finds as its people pass.
    let clay = sim
        .place_deposit((hearth.0 + 30.0, hearth.1), "core:good/clay", 8.0, true)
        .expect("laid down");
    let goods = &content().catalog.goods;
    let index = |id: &str| goods.iter().position(|g| g.id == id).expect(id);
    let (grain, pot) = (index("core:good/grain"), index("core:good/pot"));
    // A household with more grain than its roofs keep well, which pots would keep.
    let household = sim
        .people()
        .households
        .iter()
        .min_by_key(|(_, h)| h.id)
        .map(|(_, h)| h.id)
        .expect("a household");
    for (_, h) in sim.people_mut_for_tests().households.iter_mut() {
        if h.id == household {
            h.stores[grain] += 2_000.0;
        }
    }
    let pots = |sim: &Sim| {
        sim.people()
            .households
            .iter()
            .map(|(_, h)| h.stores.get(pot).copied().unwrap_or(0.0))
            .sum::<f64>()
    };
    // Hour by hour, until pots are made and someone has been seen at work in the pit.
    let mut digging = None;
    for _ in 0..120 * 24 {
        sim.advance_minutes(60).expect("advances");
        if digging.is_none() {
            digging = sim
                .people()
                .people
                .iter()
                .map(|(_, p)| p)
                .find(|p| {
                    matches!(p.act.target, Target::Deposit(d) if d == clay)
                        && matches!(
                            p.act.steps.get(usize::from(p.act.step)),
                            Some(Step::Work { .. })
                        )
                })
                .map(|p| civ_sim::frames::people::doing(&sim, p));
        }
        if pots(&sim) > 0.0 && digging.is_some() {
            break;
        }
    }
    // The inspector says where they dig: "digging clay at the clay pit east of home".
    let digging = digging.expect("someone was seen digging");
    assert!(
        digging.starts_with("digging clay at the clay pit ") && digging.ends_with(" of home"),
        "{digging}"
    );
    assert!(
        sim.people()
            .deposits_known
            .iter()
            .any(|k| k.settlement == settlement && k.deposit == clay),
        "the clay was found"
    );
    assert!(pots(&sim) > 0.0, "pots were made");
    // The pit on the clay and its heap: what was dug is what was heaped and carried home.
    let deposit = *sim
        .land()
        .deposits
        .iter()
        .find(|d| d.id == clay)
        .expect("the clay");
    assert!(deposit.taken_kg > 0.0);
    let earth = &sim.land().earthworks;
    let pit = earth
        .iter()
        .find(|w| w.kind == civ_land::earth::EarthKind::Pit && w.deposit == Some(clay))
        .expect("a pit on the clay");
    let heap = earth
        .iter()
        .find(|w| Some(w.id) == pit.heap)
        .expect("its heap");
    assert_eq!(heap.kind, civ_land::earth::EarthKind::Spoil);
    let r = f64::from(deposit.body.radius_cm) / 100.0;
    let density = deposit.body.initial_kg
        / (std::f64::consts::PI * r * r * f64::from(deposit.body.thickness_cm) / 100.0);
    let carried_m3 = deposit.taken_kg * f64::from(deposit.body.quality) / density;
    let moved = f64::from(pit.cut_m3) - f64::from(heap.cut_m3);
    assert!(
        (moved - carried_m3).abs() < 1e-3 * (1.0 + carried_m3),
        "{moved} vs {carried_m3}"
    );
    // The records reproduce the ground, and a save keeps them, the ground and what was taken.
    let map = sim.map();
    let mut replayed = civ_land::earth::GroundDelta::new(map.width, map.height, map.cell_size_m);
    for w in &sim.land().earthworks {
        civ_land::earth::replay(w, map, &mut replayed);
    }
    for cell in 0..(map.width * map.height) as usize {
        assert!(
            (replayed.at(cell) - sim.land().ground.at(cell)).abs() < 1e-3,
            "cell {cell}"
        );
    }
    let dir = tempfile::tempdir().expect("tempdir");
    let saves = SaveDir::create(dir.path().join("saves"), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = persist::save(&mut sim, &saves, SaveKind::Manual, "pits").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loaded.land().earthworks, sim.land().earthworks);
    assert_eq!(loaded.land().ground, sim.land().ground);
    assert_eq!(loaded.land().deposits, sim.land().deposits);
}
