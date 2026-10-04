//! Homes chosen among building programs (M3b slice O, ADR-0009 §1): a household builds the
//! cheapest home, hut or bays, that covers its members and its goods, and its means buy more
//! floor; goods beyond the room under its roofs keep as in the open.

use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

use civ_agents::build::{self, HomeNeed, Shape};
use civ_content::ContentRegistry;
use civ_sim::{NewWorld, Sim};

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

fn program(id: &str) -> usize {
    content()
        .catalog
        .building_index(id)
        .unwrap_or_else(|| panic!("{id}"))
}

const CARRY: f64 = 20.0;

/// The floor of `shape` of program `p`, its living floor and its room for goods.
fn gives(p: usize, shape: Shape) -> (f64, f64, f64) {
    let def = &content().catalog.buildings[p];
    let c = def
        .shapes
        .iter()
        .find(|c| c.shape == shape)
        .expect("a shape of the program");
    (c.floor_m2, c.living_m2, c.storage_kg)
}

fn hours(p: usize, shape: Shape) -> f64 {
    let def = &content().catalog.buildings[p];
    def.shapes
        .iter()
        .find(|c| c.shape == shape)
        .expect("a shape of the program")
        .hours(CARRY)
}

#[test]
fn five_with_nothing_spare_get_the_cheaper_hut_and_room_to_live() {
    let (hut, longhouse) = (
        program("core:building/hut"),
        program("core:building/longhouse"),
    );
    let need = HomeNeed {
        residents: 5,
        storage_kg: 1_200.0,
    };
    let buildings = &content().catalog.buildings;
    let (p, shape) =
        build::first_home(buildings, &[hut, longhouse], need, CARRY, 0.0, 1e9).expect("a home");
    assert_eq!(p, hut, "a hut costs less than bays for five");
    let (floor, living, room) = gives(p, shape);
    assert!(living >= 30.0, "{living} m² to live in");
    assert!(room >= 1_200.0, "room for its goods: {room} kg");
    assert_eq!(floor, living);
    // The longhouse that would cover them costs more.
    let cheapest_bays = buildings[longhouse]
        .shapes
        .iter()
        .filter(|c| c.living_m2 >= 30.0 && c.storage_kg >= 1_200.0)
        .map(|c| c.hours(CARRY))
        .fold(f64::INFINITY, f64::min);
    assert!(cheapest_bays > hours(p, shape));
    // A household that may build only huts gets what it got before programs: the hut its
    // members need, a decimetre at a time.
    let only_huts = build::first_home(buildings, &[hut], need, CARRY, 0.0, 1e9);
    assert_eq!(only_huts, Some((hut, Shape::Round { radius: 310 })));
}

#[test]
fn surplus_grain_gets_a_loft_large_enough() {
    let (hut, longhouse) = (
        program("core:building/hut"),
        program("core:building/longhouse"),
    );
    let buildings = &content().catalog.buildings;
    // More grain than the largest hut holds.
    let largest_hut = buildings[hut]
        .shapes
        .iter()
        .map(|c| c.storage_kg)
        .fold(0.0, f64::max);
    let need = HomeNeed {
        residents: 5,
        storage_kg: largest_hut + 2_000.0,
    };
    let (p, shape) =
        build::first_home(buildings, &[hut, longhouse], need, CARRY, 0.0, 1e9).expect("a home");
    assert_eq!(p, longhouse);
    let Shape::Bays { lofts, .. } = shape else {
        panic!("{shape:?}");
    };
    assert!(
        lofts > 0,
        "a loft is cheaper room for goods than a bay: {shape:?}"
    );
    let (_, living, room) = gives(p, shape);
    assert!(
        room >= need.storage_kg,
        "{room} kg for {} kg",
        need.storage_kg
    );
    assert!(living >= 30.0);
    // A household that cannot build bays does what it can: the hut that holds the most.
    let (p, shape) = build::first_home(buildings, &[hut], need, CARRY, 0.0, 1e9).expect("a hut");
    assert_eq!(p, hut);
    assert_eq!(gives(p, shape).2, largest_hut);
}

#[test]
fn more_means_never_give_less_floor() {
    let (hut, longhouse) = (
        program("core:building/hut"),
        program("core:building/longhouse"),
    );
    let buildings = &content().catalog.buildings;
    for (programs, need) in [
        (
            vec![hut, longhouse],
            HomeNeed {
                residents: 5,
                storage_kg: 1_000.0,
            },
        ),
        (
            vec![hut, longhouse],
            HomeNeed {
                residents: 12,
                storage_kg: 2_500.0,
            },
        ),
        (
            vec![longhouse],
            HomeNeed {
                residents: 4,
                storage_kg: 800.0,
            },
        ),
    ] {
        let mut last = 0.0;
        for budget in [0.0, 50.0, 100.0, 200.0, 400.0, 800.0, 1_600.0, 3_200.0, 1e9] {
            let (p, shape) =
                build::first_home(buildings, &programs, need, CARRY, budget, 1e9).expect("a home");
            let (floor, _, _) = gives(p, shape);
            assert!(floor + 1e-9 >= last, "{budget} h: {floor} m² after {last}");
            last = floor;
        }
        // Short of time, never less than what covers its need.
        let (p, shape) =
            build::first_home(buildings, &programs, need, CARRY, 1e9, 0.0).expect("a home");
        let (p0, base) =
            build::first_home(buildings, &programs, need, CARRY, 0.0, 1e9).expect("a home");
        assert_eq!((p, shape), (p0, base));
    }
    // A household of twelve outgrows the largest hut, and bays house it.
    let (p, shape) = build::first_home(
        buildings,
        &[hut, longhouse],
        HomeNeed {
            residents: 12,
            storage_kg: 2_500.0,
        },
        CARRY,
        0.0,
        1e9,
    )
    .expect("a home");
    assert_eq!(p, longhouse);
    assert!(gives(p, shape).1 >= 6.0 + 4.8 * 12.0);
}

#[test]
fn a_frame_home_stands_where_it_is_designed_with_its_door_toward_the_hearth() {
    let longhouse = program("core:building/longhouse");
    let def = &content().catalog.buildings[longhouse];
    let shape = Shape::Bays {
        bays: 3,
        storeys: 1,
        lofts: 1,
    };
    // The hearth lies 20 m south.
    let spec = build::design_shape(
        def,
        &content().catalog.goods,
        shape,
        (100.0, 100.0),
        Some((100.0, 120.0)),
    )
    .expect("a design");
    let rules = &def.rules;
    let e = civ_grammar::expand(&spec, rules).expect("it expands");
    let (dx, dy, facing) = e.door;
    assert_eq!(facing, 16_384, "the door faces south, toward the hearth");
    assert!(dy > 10_000 && (dx - 10_000).abs() <= 1, "{dx} {dy}");
    assert_eq!(build::floor_m2(&spec), e.floor_area_m2);
    assert_eq!(build::centre_m(&spec), (100.0, 100.0));
    // A hut shape is no frame design.
    assert!(
        build::design_shape(
            def,
            &content().catalog.goods,
            Shape::Round { radius: 300 },
            (0.0, 0.0),
            None
        )
        .is_none()
    );
}

fn world(seed: u64) -> Sim {
    Sim::create(
        &NewWorld {
            name: "Homes".to_owned(),
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

#[test]
fn founders_who_know_only_huts_build_huts() {
    let mut sim = world(3);
    sim.advance_minutes(20 * 24 * 60).expect("advances");
    assert!(!sim.land().buildings.is_empty(), "homes are begun");
    for b in &sim.land().buildings {
        assert_eq!(b.spec.program, "core:building/hut");
    }
}

#[test]
fn a_household_that_knows_framing_and_has_grain_to_keep_builds_bays_with_lofts() {
    let mut sim = world(3);
    let grain = content()
        .catalog
        .goods
        .iter()
        .position(|g| g.id == "core:good/grain")
        .expect("grain");
    // Before anyone plans a home: the eldest adult of the first household learns jointed framing
    // from the observer, and the household holds more grain than the largest hut has room for.
    let (household, who) = {
        let pop = sim.people();
        let now = sim.now();
        let (_, h) = pop
            .households
            .iter()
            .min_by_key(|(_, h)| h.id)
            .expect("a household");
        let who = h
            .members
            .iter()
            .filter_map(|m| pop.person(*m))
            .filter(|p| p.age_years(now) >= 18.0)
            .max_by(|a, b| a.age_years(now).total_cmp(&b.age_years(now)))
            .map(|p| p.id)
            .expect("an adult");
        (h.id, who)
    };
    for (_, h) in sim.people_mut_for_tests().households.iter_mut() {
        if h.id == household {
            h.stores[grain] += 7_000.0;
        }
    }
    sim.introduce_technique(who, "core:technique/jointed_frame", false)
        .expect("introduced");
    let mut home = None;
    for _ in 0..20 {
        sim.advance_minutes(24 * 60).expect("advances");
        home = sim
            .land()
            .buildings
            .iter()
            .find(|b| b.household == household)
            .cloned();
        if home.is_some() {
            break;
        }
    }
    let home = home.expect("it begins a home");
    assert_eq!(home.spec.program, "core:building/longhouse");
    let lofts = home.spec.params[civ_grammar::frame_params::LOFT_BAYS];
    assert!(lofts != 0, "lofts for its grain");
    let def = &content().catalog.buildings[program("core:building/longhouse")];
    let e = civ_grammar::expand(&home.spec, &def.rules).expect("expands");
    assert!(
        e.storage_total_kg() >= 7_000.0,
        "{} kg",
        e.storage_total_kg()
    );
    // Every other household knows only huts.
    for b in sim
        .land()
        .buildings
        .iter()
        .filter(|b| b.household != household)
    {
        assert_eq!(b.spec.program, "core:building/hut");
    }
}
