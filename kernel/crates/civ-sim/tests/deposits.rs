//! Deposits in the ground (ADR-0010 §1-2; M3b slice Q): found by walking near them, laid down by
//! the observer, dug at a pit for clay that is made into pots, quarried for stone once loose stone
//! is gone, and kept across a save.

use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

use civ_agents::build::plot_clear;
use civ_agents::history::ChronicleKind;
use civ_agents::person::{Flow, Step, Target};
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
            neighbours: Vec::new(),
            neighbours_known: false,
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
    // Nobody claims a plot on the pit or its heap, ground that is otherwise clear.
    let mut bare = sim.land().clone();
    bare.earthworks.clear();
    for w in [pit, heap] {
        assert!(plot_clear(&bare, sim.map(), sim.nav(), &w.rect), "{w:?}");
        assert!(
            !plot_clear(sim.land(), sim.map(), sim.nav(), &w.rect),
            "{w:?}"
        );
    }
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
    // The observer hears of a pit deepening: the earthworks' revision changes with its earth.
    let rev = civ_sim::frames::earthworks::earthworks_rev(&sim);
    let at = sim
        .land()
        .earthworks
        .iter()
        .position(|w| w.kind == civ_land::earth::EarthKind::Pit)
        .expect("the pit");
    sim.land_mut_for_tests().earthworks[at].cut_m3 += 0.5;
    assert_ne!(civ_sim::frames::earthworks::earthworks_rev(&sim), rev);
}

#[test]
fn with_no_loose_stone_left_a_household_needing_stone_quarries_it() {
    let mut sim = world(3);
    let hearth = sim
        .land()
        .settlements
        .first()
        .expect("a settlement")
        .hearth_m;
    // Stone showing beside the hearth, which the village finds as its people pass.
    let stone = sim
        .place_deposit((hearth.0 + 30.0, hearth.1), "core:good/stone", 8.0, true)
        .expect("laid down");
    // No loose stone anywhere, so a quarry is the only way to stone.
    let loose = sim
        .rules()
        .land
        .resources
        .iter()
        .position(|r| r.id == "stone")
        .expect("loose stone");
    for s in sim.land_mut_for_tests().stocks[loose].iter_mut() {
        *s = 0.0;
    }
    // A household with no quern and no stone: it wants a quern, and stone to shape one.
    let goods = &content().catalog.goods;
    let index = |id: &str| goods.iter().position(|g| g.id == id).expect(id);
    let (quern, stone_good) = (index("core:good/quern"), index("core:good/stone"));
    let household = sim
        .people()
        .households
        .iter()
        .min_by_key(|(_, h)| h.id)
        .map(|(_, h)| h.id)
        .expect("a household");
    for (_, h) in sim.people_mut_for_tests().households.iter_mut() {
        if h.id == household {
            h.stores[quern] = 0.0;
            h.stores[stone_good] = 0.0;
        }
    }
    // Hour by hour, until someone has been seen at work in the quarry.
    let mut quarrying = None;
    for _ in 0..120 * 24 {
        sim.advance_minutes(60).expect("advances");
        quarrying = sim
            .people()
            .people
            .iter()
            .map(|(_, p)| p)
            .find(|p| {
                matches!(p.act.target, Target::Deposit(d) if d == stone)
                    && matches!(
                        p.act.steps.get(usize::from(p.act.step)),
                        Some(Step::Work { .. })
                    )
            })
            .map(|p| civ_sim::frames::people::doing(&sim, p));
        if quarrying.is_some() {
            break;
        }
    }
    let quarrying = quarrying.expect("someone quarries");
    assert!(
        quarrying.starts_with("quarrying stone at the stone quarry ")
            && quarrying.ends_with(" of home"),
        "{quarrying}"
    );
    // A quarry and its spoil heap, named so; the work goes on until stone has come out.
    for _ in 0..30 * 24 {
        if sim
            .land()
            .deposits
            .iter()
            .any(|d| d.id == stone && d.taken_kg > 0.0)
        {
            break;
        }
        sim.advance_minutes(60).expect("advances");
    }
    let deposit = *sim
        .land()
        .deposits
        .iter()
        .find(|d| d.id == stone)
        .expect("the stone");
    assert!(deposit.taken_kg > 0.0, "stone came out");
    let earth = &sim.land().earthworks;
    let pit = earth
        .iter()
        .find(|w| w.kind == civ_land::earth::EarthKind::Pit && w.deposit == Some(stone))
        .expect("a quarry");
    let heap = earth
        .iter()
        .find(|w| Some(w.id) == pit.heap)
        .expect("its spoil heap");
    let bytes = civ_sim::frames::earthworks::earthworks_response(&sim);
    let response =
        civ_schema::flatbuffers::root::<civ_schema::wire::Response>(&bytes).expect("decodes");
    let works = response
        .body_as_earthworks()
        .expect("earthworks")
        .works()
        .expect("works");
    let words = |id: civ_core::PermanentId| {
        works
            .iter()
            .find(|w| w.id() == id.get())
            .and_then(|w| w.words())
            .unwrap_or_default()
            .to_owned()
    };
    assert!(
        words(pit.id).contains("'s household's stone quarry, "),
        "{}",
        words(pit.id)
    );
    assert!(
        words(heap.id).starts_with("the spoil heap of a stone quarry, "),
        "{}",
        words(heap.id)
    );
    // What was broken out is what was heaped and carried home.
    let r = f64::from(deposit.body.radius_cm) / 100.0;
    let density = deposit.body.initial_kg
        / (std::f64::consts::PI * r * r * f64::from(deposit.body.thickness_cm) / 100.0);
    let carried_m3 = deposit.taken_kg * f64::from(deposit.body.quality) / density;
    let moved = f64::from(pit.cut_m3) - f64::from(heap.cut_m3);
    assert!(
        (moved - carried_m3).abs() < 1e-3 * (1.0 + carried_m3),
        "{moved} vs {carried_m3}"
    );
}

/// Household `h` left with 30 kg of flour and nothing else to eat.
fn flour_only(sim: &mut Sim, h: civ_core::PermanentId) {
    let catalog = &content().catalog;
    let flour = catalog.good_index("core:good/flour").expect("flour");
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == h {
            for (g, d) in catalog.goods.iter().enumerate() {
                if d.purpose == civ_agents::params::GoodUse::Food
                    && d.eaten != civ_agents::params::Eaten::Never
                {
                    x.stores[g] = 0.0;
                }
            }
            x.stores[flour] += 30.0;
        }
    }
}

#[test]
fn a_household_builds_an_oven_of_clay_it_digs_and_bakes_in_it() {
    let mut sim = world(3);
    let hearth = sim
        .land()
        .settlements
        .first()
        .expect("a settlement")
        .hearth_m;
    // Clay showing beside the hearth, which the village finds as its people pass.
    let clay = sim
        .place_deposit((hearth.0 + 30.0, hearth.1), "core:good/clay", 8.0, true)
        .expect("laid down");
    let catalog = &content().catalog;
    let oven = catalog.good_index("core:good/oven").expect("the oven");
    let baking = catalog
        .activities
        .iter()
        .position(|a| a.id == "core:activity/bake_bread_oven")
        .expect("baking in the oven") as u16;
    // Households with a whole oven, and how much of one each holds.
    let ovens = |sim: &Sim| -> Vec<(civ_core::PermanentId, f64)> {
        sim.people()
            .households
            .iter()
            .map(|(_, h)| (h.id, h.stores.get(oven).copied().unwrap_or(0.0)))
            .filter(|&(_, n)| n >= 1.0 - 1e-9)
            .collect()
    };
    // How much of its ovens household `h` has worn away by baking in them.
    let worn = |sim: &Sim, h: civ_core::PermanentId| {
        sim.people()
            .household(h)
            .map_or(0.0, |h| h.flows.get(Flow::Worn, oven))
    };
    // Founders know ovens but bring none, as an oven stays where it is built. They want one: its
    // clay is dug and it is built, over days of work.
    let held = |sim: &Sim| -> f64 {
        sim.people()
            .households
            .iter()
            .map(|(_, h)| h.stores.get(oven).copied().unwrap_or(0.0))
            .sum()
    };
    assert_eq!(held(&sim), 0.0, "founders bring no oven");
    // The first household with an oven begun is left nothing to eat but flour: it bakes at the
    // hearth until its oven is whole, as one partly built bakes nothing (baking in it would wear
    // it).
    let mut hungry = None;
    for _ in 0..180 * 24 {
        sim.advance_minutes(60).expect("advances");
        for (_, h) in sim.people().households.iter() {
            assert_eq!(
                worn(&sim, h.id),
                0.0,
                "{} baked in an oven not yet whole",
                h.id
            );
        }
        if !ovens(&sim).is_empty() {
            break;
        }
        if hungry.is_none() {
            hungry = sim
                .people()
                .households
                .iter()
                .find(|(_, h)| h.stores.get(oven).copied().unwrap_or(0.0) > 0.0)
                .map(|(_, h)| h.id);
            if let Some(h) = hungry {
                flour_only(&mut sim, h);
            }
        }
    }
    assert!(hungry.is_some(), "an oven was begun");
    let built = ovens(&sim);
    assert!(!built.is_empty(), "an oven was built");
    assert!(
        sim.land()
            .deposits
            .iter()
            .any(|d| d.id == clay && d.taken_kg > 0.0),
        "of clay dug from the pit"
    );
    // With flour and nothing ready to eat, its household bakes in it, hour by hour until someone
    // is seen at it, and it wears with use. It is made to know no other way to bake: a small
    // household's batch costs less wood at the hearth.
    let (owner, _) = built[0];
    flour_only(&mut sim, owner);
    let hearth_baking = catalog
        .technique_index("core:technique/hearth_baking")
        .expect("baking at the hearth") as u16;
    for (_, p) in sim.people_mut_for_tests().people.iter_mut() {
        if p.household == owner {
            p.knows.retain(|k| k.technique != hearth_baking);
        }
    }
    let mut seen = None;
    for _ in 0..10 * 24 {
        sim.advance_minutes(60).expect("advances");
        seen = sim
            .people()
            .people
            .iter()
            .map(|(_, p)| p)
            .find(|p| {
                p.act.def == baking
                    && matches!(
                        p.act.steps.get(usize::from(p.act.step)),
                        Some(Step::Work { .. })
                    )
            })
            .map(|p| civ_sim::frames::people::doing(&sim, p));
        if seen.is_some() {
            break;
        }
    }
    assert_eq!(seen.as_deref(), Some("baking bread in the oven"));
    sim.advance_minutes(24 * 60).expect("advances");
    assert!(
        worn(&sim, owner) > 0.0,
        "an oven wears as it is used: {:?} then {:?}",
        built,
        ovens(&sim)
    );
}
