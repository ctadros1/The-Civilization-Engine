//! Homes chosen among building programs (M3b slice O, ADR-0009 §1): a household builds the
//! cheapest home, hut or bays, that covers its members and its goods, and its means buy more
//! floor; goods beyond the room under its roofs keep as in the open, and a storehouse beside its
//! home raises them when they would lose more there than it costs (ADR-0009 §7).

use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

use civ_agents::build::{self, HomeNeed, Shape, StoreNeed};
use civ_content::ContentRegistry;
use civ_core::PermanentId;
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

fn good(id: &str) -> usize {
    content()
        .catalog
        .goods
        .iter()
        .position(|g| g.id == id)
        .unwrap_or_else(|| panic!("{id}"))
}

#[test]
fn a_granary_must_save_more_than_it_costs_and_be_paid_for() {
    let granary = program("core:building/granary");
    let (buildings, goods) = (&content().catalog.buildings, &content().catalog.goods);
    let grain = good("core:good/grain");
    let hours: Vec<f64> = buildings[granary]
        .shapes
        .iter()
        .map(|c| c.hours(CARRY))
        .collect();
    let cheapest = hours.iter().copied().fold(f64::INFINITY, f64::min);
    // Grain worth half an hour a kilogram, over three years.
    let cost = vec![0.5; goods.len()];
    let mut overflow = vec![0.0; goods.len()];
    let choose = |overflow: &[f64], budget: f64, time: f64| {
        build::storehouse(
            buildings,
            &[granary],
            &StoreNeed {
                overflow,
                cost_h: &cost,
                raised_factor: 2.0,
                horizon_days: 1095.0,
            },
            goods,
            CARRY,
            budget,
            time,
        )
    };
    // A tonne beyond the room saves less than any granary costs.
    overflow[grain] = 1_000.0;
    assert_eq!(choose(&overflow, 1e9, 1e9), None);
    // Eight tonnes are worth raising: the granary chosen saves more than it costs.
    overflow[grain] = 8_000.0;
    let (p, shape) = choose(&overflow, 1e9, 1e9).expect("a granary");
    assert_eq!(p, granary);
    let c = buildings[granary]
        .shapes
        .iter()
        .find(|c| c.shape == shape)
        .expect("its shape");
    let saves = build::store_saves_h(&overflow, goods, &cost, c.room_kg, 2.0, 1095.0);
    assert!(
        saves > c.hours(CARRY),
        "{saves} h saved for {} h",
        c.hours(CARRY)
    );
    // On a raised floor grain loses a quarter as fast as in the open here: 5 % a year under a
    // roof, twice as long raised, against 16 % in the open.
    let open = 0.5f64.powf(1095.0 / 1560.0);
    let raised = 0.5f64.powf(1095.0 / (4932.0 * 2.0));
    let held = c.room_kg[civ_grammar::storage::RAISED].min(8_000.0);
    assert!(
        (saves - held * (raised - open) * 0.5).abs() < 1e-6,
        "{saves}"
    );
    // Without the means to pay for one in full, or the time to build it, none.
    assert_eq!(choose(&overflow, cheapest - 1.0, 1e9), None);
    assert_eq!(choose(&overflow, 1e9, cheapest - 1.0), None);
    // Goods that keep no better under a roof are not worth one.
    let mut meat = vec![0.0; goods.len()];
    meat[good("core:good/meat")] = 8_000.0;
    assert_eq!(choose(&meat, 1e9, 1e9), None);
}

/// World `seed`'s first household, given a finished hut at its home, `extra` kilograms more of
/// good `id`, and an eldest adult who knows jointed framing: its id and its home.
fn housed_framers(seed: u64, id: &str, extra: f64) -> (Sim, PermanentId, (f32, f32)) {
    let mut sim = world(seed);
    let (household, who, home) = {
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
        (h.id, who, h.home)
    };
    let hut = &content().catalog.buildings[program("core:building/hut")];
    let spec = build::design_shape(
        hut,
        &content().catalog.goods,
        Shape::Round { radius: 310 },
        home,
        None,
    )
    .expect("a hut");
    sim.place_building_for_tests(household, spec, civ_grammar::Stage::ALL.len() as u8)
        .expect("placed");
    let g = good(id);
    for (_, h) in sim.people_mut_for_tests().households.iter_mut() {
        if h.id == household {
            h.stores[g] += extra;
        }
    }
    sim.introduce_technique(who, "core:technique/jointed_frame", false)
        .expect("introduced");
    (sim, household, home)
}

/// The first building other than its hut that `household` begins within `days` days.
fn second_building(sim: &mut Sim, household: PermanentId, days: u32) -> Option<civ_land::Building> {
    for _ in 0..days {
        sim.advance_minutes(24 * 60).expect("advances");
        let found = sim
            .land()
            .buildings
            .iter()
            .find(|b| b.household == household && b.spec.program != "core:building/hut")
            .cloned();
        if found.is_some() {
            return found;
        }
    }
    None
}

#[test]
fn with_grain_beyond_its_roof_and_the_means_a_household_raises_a_granary_beside_its_home() {
    let (mut sim, household, home) = housed_framers(3, "core:good/grain", 8_000.0);
    let b = second_building(&mut sim, household, 20).expect("it begins a store");
    assert_eq!(b.spec.program, "core:building/granary");
    let (x, y) = build::centre_m(&b.spec);
    let away = f64::from(x - home.0).hypot(f64::from(y - home.1));
    assert!(away <= 30.0 + 1e-3, "{away} m from home");
    let plot = sim
        .land()
        .plots
        .iter()
        .find(|p| p.id == b.plot)
        .expect("its plot");
    assert_eq!(plot.use_, civ_land::PlotUse::Store);
    // Its door faces the home, and the household lives where it did.
    let def = &content().catalog.buildings[program("core:building/granary")];
    let e = civ_grammar::expand(&b.spec, &def.rules).expect("expands");
    let (dx, dy) = (
        f64::from(e.door.0) / 100.0 - f64::from(x),
        f64::from(e.door.1) / 100.0 - f64::from(y),
    );
    let (hx, hy) = (f64::from(home.0 - x), f64::from(home.1 - y));
    assert!(
        dx * hx + dy * hy > 0.0,
        "door ({dx}, {dy}) and home ({hx}, {hy})"
    );
    let lives = sim
        .people()
        .households
        .iter()
        .find(|(_, h)| h.id == household)
        .map(|(_, h)| h.home)
        .expect("the household");
    assert_eq!(lives, home);
}

#[test]
fn lacking_the_means_a_household_raises_no_granary() {
    // As much beyond its roof, but seed it keeps for sowing: it has nothing to spare for one.
    let (mut sim, household, _) = housed_framers(3, "core:good/seed_grain", 8_000.0);
    assert_eq!(second_building(&mut sim, household, 20), None);
}

#[test]
fn a_finished_granary_raises_what_the_hut_has_no_room_for() {
    let (mut sim, household, home) = housed_framers(3, "core:good/grain", 8_000.0);
    let goods = &content().catalog.goods;
    let grain = good("core:good/grain");
    let before = sim.people().household(household).expect("it").clone();
    let granary = &content().catalog.buildings[program("core:building/granary")];
    let shape = Shape::Bays {
        bays: 2,
        storeys: 1,
        lofts: 0,
    };
    let spec = build::design_shape(granary, goods, shape, (home.0 + 15.0, home.1), Some(home))
        .expect("a granary");
    sim.place_building_for_tests(household, spec, civ_grammar::Stage::ALL.len() as u8)
        .expect("placed");
    let after = sim.people().household(household).expect("it").clone();
    assert_eq!(after.keeping.raised_kg, 7_500.0);
    assert_eq!(after.keeping.roofed_kg, before.keeping.roofed_kg);
    // A year on, uneaten: the grain the hut had no room for lost 16 % in the open; on the
    // granary's floor it loses about 2.5 %.
    let year = civ_core::SimTime::from_minutes(sim.now().minutes() + 365 * 24 * 60);
    let kept = |h: &civ_agents::Household| h.stores_at_time(year, goods, &|_| 0.0)[grain];
    let (open, raised) = (kept(&before), kept(&after));
    assert!(raised > open + 500.0, "{raised} kg against {open} kg");
}

/// A workshop of `household` making sickles, as if `at_once` of its people had just worked for it
/// at once: its id.
fn busy_firm(sim: &mut Sim, household: PermanentId, at_once: u8) -> PermanentId {
    let (now, goods) = (sim.now(), content().catalog.goods.len());
    let sickle = good("core:good/sickle") as u16;
    let id = sim.allocate_id_for_tests();
    let pop = sim.people_mut_for_tests();
    let (founder, settlement) = {
        let h = pop.household(household).expect("the household");
        (h.members[0], h.settlement)
    };
    let mut firm =
        civ_agents::firm::Firm::new(id, household, founder, settlement, sickle, goods, now);
    firm.saw_at_once(at_once, now.day_index());
    pop.firms.push(firm);
    id
}

#[test]
fn a_firm_with_more_at_work_than_a_home_has_room_for_builds_a_workshop() {
    // Its means are timber it can spare, which takes no room under a roof: no store is wanted.
    let (mut sim, household, home) = housed_framers(3, "core:good/timber", 40_000.0);
    let firm = busy_firm(&mut sim, household, 3);
    let b = second_building(&mut sim, household, 20).expect("it begins a workshop");
    assert_eq!(b.spec.program, "core:building/workshop");
    assert_eq!(b.firm, Some(firm), "built for the firm");
    let (x, y) = build::centre_m(&b.spec);
    let away = f64::from(x - home.0).hypot(f64::from(y - home.1));
    assert!(away <= 30.0 + 1e-3, "{away} m from home");
    let plot = sim
        .land()
        .plots
        .iter()
        .find(|p| p.id == b.plot)
        .expect("its plot");
    assert_eq!(plot.use_, civ_land::PlotUse::Work);
    // Places for all three, and the household lives where it did.
    let def = &content().catalog.buildings[program("core:building/workshop")];
    let e = civ_grammar::expand(&b.spec, &def.rules).expect("expands");
    assert!(e.work_places >= 3, "{} places", e.work_places);
    let lives = sim.people().household(household).expect("it").home;
    assert_eq!(lives, home);
}

#[test]
fn a_firm_its_home_has_room_for_builds_no_workshop() {
    // Two at work at once fit in a home: a firm alone is no reason to build.
    let (mut sim, household, _) = housed_framers(3, "core:good/timber", 40_000.0);
    busy_firm(&mut sim, household, 2);
    let b = second_building(&mut sim, household, 20);
    assert!(
        b.as_ref()
            .is_none_or(|b| b.spec.program != "core:building/workshop"),
        "{b:?}"
    );
}

#[test]
fn a_firm_works_in_its_workshop_once_roofed_and_the_building_outlasts_it() {
    let (mut sim, household, home) = housed_framers(3, "core:good/grain", 0.0);
    let firm = busy_firm(&mut sim, household, 3);
    let catalog = content().catalog.clone();
    let def = &catalog.buildings[program("core:building/workshop")];
    let shape = Shape::Bays {
        bays: 2,
        storeys: 1,
        lofts: 0,
    };
    let at = (home.0 + 15.0, home.1);
    let spec = build::design_shape(def, &catalog.goods, shape, at, Some(home)).expect("designed");
    let site = build::centre_m(&spec);
    // Walls up but no roof: it works at home.
    let id = sim
        .place_building_for_tests(household, spec, 2)
        .expect("placed");
    for b in sim.land_mut_for_tests().buildings.iter_mut() {
        if b.id == id {
            b.firm = Some(firm);
        }
    }
    assert_eq!(sim.people().firm_site(sim.land(), &catalog, firm), None);
    // Roofed, it works there.
    for b in sim.land_mut_for_tests().buildings.iter_mut() {
        if b.id == id {
            b.stage = civ_grammar::Stage::ALL.len() as u8;
        }
    }
    assert_eq!(
        sim.people().firm_site(sim.land(), &catalog, firm),
        Some(site)
    );
    // Its firm closes; the building stays, and another of the household's firms works in it.
    let now = sim.now();
    for f in sim.people_mut_for_tests().firms.iter_mut() {
        if f.id == firm {
            f.closed = Some((now, civ_agents::firm::Exit::Idle));
        }
    }
    let b = sim
        .land()
        .buildings
        .iter()
        .find(|b| b.id == id)
        .expect("still standing");
    assert!(sim.people().free_workshop(&catalog, b));
    let next = sim.allocate_id_for_tests();
    {
        let goods = catalog.goods.len();
        let pop = sim.people_mut_for_tests();
        let founder = pop.household(household).expect("it").members[0];
        pop.firms.push(civ_agents::firm::Firm::new(
            next, household, founder, None, 0, goods, now,
        ));
    }
    // Until the building is handed on the new firm works at home; by the next day it has it,
    // and the map hears of the change.
    assert_eq!(sim.people().firm_site(sim.land(), &catalog, next), None);
    let rev = civ_sim::frames::buildings::buildings_rev(&sim);
    sim.advance_minutes(24 * 60).expect("advances");
    assert_eq!(
        sim.people().firm_site(sim.land(), &catalog, next),
        Some(site)
    );
    let b = sim
        .land()
        .buildings
        .iter()
        .find(|b| b.id == id)
        .expect("still standing");
    assert_eq!(b.firm, Some(next));
    assert_ne!(civ_sim::frames::buildings::buildings_rev(&sim), rev);
}

#[test]
fn a_closed_firms_workshop_goes_to_the_oldest_firm_without_one() {
    let (mut sim, household, home) = housed_framers(3, "core:good/grain", 0.0);
    let catalog = content().catalog.clone();
    let def = &catalog.buildings[program("core:building/workshop")];
    let shape = Shape::Bays {
        bays: 2,
        storeys: 1,
        lofts: 0,
    };
    let spec = build::design_shape(
        def,
        &catalog.goods,
        shape,
        (home.0 + 15.0, home.1),
        Some(home),
    )
    .expect("designed");
    // A workshop no firm works in, and two firms without one.
    let shop = sim
        .place_building_for_tests(household, spec, civ_grammar::Stage::ALL.len() as u8)
        .expect("placed");
    let older = busy_firm(&mut sim, household, 1);
    let newer = busy_firm(&mut sim, household, 1);
    assert!(older < newer);
    sim.advance_minutes(24 * 60).expect("advances");
    let b = sim
        .land()
        .buildings
        .iter()
        .find(|b| b.id == shop)
        .expect("standing");
    assert_eq!(b.firm, Some(older));
    assert_eq!(sim.people().firm_site(sim.land(), &catalog, newer), None);
}

#[test]
fn a_firm_busier_than_any_workshop_gets_the_largest() {
    let catalog = &content().catalog;
    let shop = program("core:building/workshop");
    let most = catalog.buildings[shop]
        .shapes
        .iter()
        .map(|c| c.work_places)
        .max()
        .expect("shapes");
    let largest = build::workshop(&catalog.buildings, &[shop], most, CARRY, 1e9, 1e9);
    assert!(largest.is_some());
    assert_eq!(
        build::workshop(&catalog.buildings, &[shop], most + 20, CARRY, 1e9, 1e9),
        largest
    );
}

#[test]
fn a_firm_too_busy_to_house_leaves_the_next_firm_its_workshop() {
    // The means pay for a small workshop, not the largest: the busiest firm gets none, and the
    // next gets its own rather than nothing being built.
    let (mut sim, household, _) = housed_framers(3, "core:good/timber", 40_000.0);
    let crowded = busy_firm(&mut sim, household, 60);
    let busy = busy_firm(&mut sim, household, 3);
    let b = second_building(&mut sim, household, 20).expect("it begins a workshop");
    assert_eq!(b.spec.program, "core:building/workshop");
    assert_eq!(b.firm, Some(busy), "not for {crowded}");
}

#[test]
fn a_building_nobody_can_work_on_holds_nothing_else_up() {
    let (mut sim, household, _) = housed_framers(3, "core:good/grain", 8_000.0);
    let granary = second_building(&mut sim, household, 20).expect("it begins a store");
    assert_eq!(granary.spec.program, "core:building/granary");
    // Nobody in the household knows framing any more: the granary waits.
    let framing = content()
        .catalog
        .technique_index("core:technique/jointed_frame")
        .expect("framing") as u16;
    for (_, p) in sim.people_mut_for_tests().people.iter_mut() {
        if p.household == household {
            p.knows.retain(|k| k.technique != framing);
        }
    }
    let catalog = &content().catalog;
    for _ in 0..30 {
        sim.advance_minutes(24 * 60).expect("advances");
        // Something else is planned, or begun, while it waits.
        let planned = sim
            .people()
            .planned_home(household)
            .is_some_and(|spec| spec.program == "core:building/hut");
        let begun = sim
            .land()
            .buildings
            .iter()
            .any(|b| b.household == household && b.id != granary.id && !b.finished());
        if planned || begun {
            let g = sim
                .land()
                .buildings
                .iter()
                .find(|b| b.id == granary.id)
                .expect("still there");
            assert_eq!(g.stage, granary.stage, "nobody works on it");
            assert!(catalog.building_index(&g.spec.program).is_some());
            return;
        }
    }
    panic!("the household planned nothing else in thirty days");
}

#[test]
fn a_household_seeks_level_ground_before_it_levels_and_never_builds_on_ground_too_steep() {
    let sim = world(3);
    let catalog = &content().catalog;
    let hut = &catalog.buildings[program("core:building/hut")];
    let home = sim
        .people()
        .households
        .iter()
        .min_by_key(|(_, h)| h.id)
        .map(|(_, h)| h.home)
        .expect("a household");
    let shape = Shape::Round { radius: 250 };
    let design_at = |at: (f32, f32)| build::design_shape(hut, &catalog.goods, shape, at, None);
    let site = |levelling: &dyn Fn(&civ_land::RectCm) -> Option<f64>| {
        build::home_site(
            sim.land(),
            sim.map(),
            sim.nav(),
            hut,
            &design_at,
            home,
            None,
            30.0,
            levelling,
        )
        .map(|spec| build::plot_rect(&spec, hut))
    };
    // Level ground at home: it builds there.
    let at_home = site(&|_| Some(0.0)).expect("a site");
    // Home's ground would have to be levelled and there is level ground beside it: it builds
    // beside it.
    let beside = site(&|r| Some(if *r == at_home { 0.8 } else { 0.0 })).expect("a site");
    assert_ne!(beside, at_home);
    // All of it would have to be levelled: the plot that needs least, the nearest among equals.
    assert_eq!(
        site(&|r| Some(if *r == at_home { 1.5 } else { 0.6 })),
        Some(beside)
    );
    assert_eq!(
        site(&|r| Some(if *r == at_home { 0.6 } else { 1.5 })),
        Some(at_home)
    );
    // Too steep at home and level beside it: beside it. Too steep everywhere: nowhere.
    assert_eq!(site(&|r| (*r != at_home).then_some(0.0)), Some(beside));
    assert_eq!(site(&|_| None), None);
}

#[test]
fn a_hut_on_sloping_ground_has_its_plot_levelled_first_and_the_ground_keeps_it() {
    use civ_land::earth;
    let mut sim = world(3);
    let catalog = &content().catalog;
    let hut = &catalog.buildings[program("core:building/hut")];
    let (household, home) = {
        let (_, h) = sim
            .people()
            .households
            .iter()
            .min_by_key(|(_, h)| h.id)
            .expect("a household");
        (h.id, h.home)
    };
    let lev = sim.rules().people.build.levelling;
    // Somewhere near home whose ground drops between levelling's threshold and the most built on.
    let shape = Shape::Round { radius: 250 };
    let mut found = None;
    'search: for r in 1..60 {
        for k in 0..16 {
            let a = f64::from(k) * std::f64::consts::TAU / 16.0;
            let at = (
                home.0 + (f64::from(r) * 8.0 * a.cos()) as f32,
                home.1 + (f64::from(r) * 8.0 * a.sin()) as f32,
            );
            let Some(spec) = build::design_shape(hut, &catalog.goods, shape, at, None) else {
                continue;
            };
            let rect = build::plot_rect(&spec, hut);
            let drop = earth::drop_across(&rect, &|x, y| earth::bed_height(sim.map(), (x, y)));
            if drop > lev.from_m + 0.2 && drop < lev.most_m - 0.2 {
                found = Some(spec);
                break 'search;
            }
        }
    }
    let spec = found.expect("sloping ground near home");
    let house = sim
        .place_building_for_tests(household, spec, 0)
        .expect("placed");
    let plot = sim
        .land()
        .buildings
        .iter()
        .find(|b| b.id == house)
        .expect("the hut")
        .plot;
    let work = *sim
        .land()
        .earthworks
        .iter()
        .find(|w| w.plot == Some(plot))
        .expect("a platform for its plot");
    assert_eq!(work.done, 0.0);
    assert!(work.cut_m3 > 0.0, "{work:?}");
    // The household levels it with the first stage's first hours.
    for _ in 0..30 {
        sim.advance_minutes(24 * 60).expect("advances");
        let w = sim
            .land()
            .earthworks
            .iter()
            .find(|w| w.id == work.id)
            .expect("there");
        if w.done >= 1.0 {
            break;
        }
    }
    let done = sim
        .land()
        .earthworks
        .iter()
        .find(|w| w.id == work.id)
        .expect("there")
        .done;
    assert!(done > 0.0, "levelling began");
    // The boundary says what it is and which tiles of ground it changed (wire 1.20).
    let bytes = civ_sim::frames::earthworks::earthworks_response(&sim);
    let response =
        civ_schema::flatbuffers::root::<civ_schema::wire::Response>(&bytes).expect("decodes");
    let list = response.body_as_earthworks().expect("earthworks");
    assert_ne!(list.rev(), 0);
    assert_eq!(
        list.rev(),
        civ_sim::frames::earthworks::earthworks_rev(&sim)
    );
    let info = list
        .works()
        .expect("works")
        .iter()
        .find(|w| w.id() == work.id.get())
        .expect("the platform");
    assert_eq!((info.plot(), info.building()), (plot.get(), house.get()));
    let words = info.words().unwrap_or_default();
    assert!(
        words.starts_with("the plot of ") && words.contains("'s hut") && words.contains("levelled"),
        "{words}"
    );
    assert_eq!(list.tile_cells(), earth::DELTA_TILE);
    let tiles = list.tiles().expect("tiles");
    assert!(!tiles.is_empty());
    for t in tiles.iter() {
        assert!(
            sim.land()
                .ground
                .tiles()
                .any(|(i, g)| i == t.index() && g.rev == t.rev())
        );
    }
    // Earth is moved, not made: what is cut is filled, give or take its sides at the map's edge.
    let ground = &sim.land().ground;
    assert!(
        ground.net_m3().abs() < 0.05 * f64::from(work.cut_m3) + 0.5,
        "{} of {}",
        ground.net_m3(),
        work.cut_m3
    );
    // The records reproduce the ground (ADR-0010 §3).
    let map = sim.map();
    let mut replayed = earth::GroundDelta::new(map.width, map.height, map.cell_size_m);
    for w in &sim.land().earthworks {
        earth::replay(w, map, &mut replayed);
    }
    for cell in 0..(map.width * map.height) as usize {
        assert!(
            (replayed.at(cell) - ground.at(cell)).abs() < 1e-3,
            "cell {cell}"
        );
    }
    // A save keeps the records and the ground.
    let dir = tempfile::tempdir().expect("tempdir");
    let saves =
        commons_persist::SaveDir::create(dir.path().join("saves"), civ_schema::SAVE_EXTENSION)
            .expect("dir");
    let saved =
        civ_sim::persist::save(&mut sim, &saves, commons_persist::SaveKind::Manual, "earth")
            .expect("saves");
    let loaded = civ_sim::persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loaded.land().earthworks, sim.land().earthworks);
    assert_eq!(loaded.land().ground, sim.land().ground);
}

#[test]
fn a_hut_s_daub_is_dug_from_a_pit_beside_it_as_its_walls_go_up() {
    use civ_grammar::Stage;
    use civ_land::earth::{self, EarthKind};
    let mut sim = world(3);
    let catalog = &content().catalog;
    // Until a hut has its walls.
    let walled =
        |b: &civ_land::Building| b.finished() || b.stage().is_some_and(|s| s > Stage::Walls);
    let mut hut = None;
    for _ in 0..60 {
        sim.advance_minutes(24 * 60).expect("advances");
        hut = sim.land().buildings.iter().find(|b| walled(b)).cloned();
        if hut.is_some() {
            break;
        }
    }
    let hut = hut.expect("a hut has its walls");
    let def = &catalog.buildings[program(&hut.spec.program)];
    let daub = civ_grammar::expand(&hut.spec, &def.rules)
        .expect("expands")
        .daub_m3(None);
    assert!(daub > 1.0, "{daub}");
    // Its daub pit holds what its walls do: no more, as nothing has been mended yet.
    let pit = *sim
        .land()
        .earthworks
        .iter()
        .find(|w| w.kind == EarthKind::Pit && w.plot == Some(hut.plot))
        .expect("a daub pit");
    assert_eq!(pit.deposit, None);
    assert!(
        (f64::from(pit.cut_m3) - daub).abs() < 1e-3 * daub,
        "{} of {daub}",
        pit.cut_m3
    );
    // Beside the plot, clear of it, and sized to go about a metre down with it, at least a pit's
    // side across.
    let plot = sim
        .land()
        .plots
        .iter()
        .find(|p| p.id == hut.plot)
        .expect("its plot")
        .rect;
    assert!(!pit.rect.near(&plot, 0), "{pit:?}");
    assert!(pit.rect.near(&plot, 1_300), "{pit:?}");
    let side = f64::from(pit.rect.w) / 100.0;
    assert!(side >= sim.rules().people.digging.pit_side_m, "{side}");
    assert!(pit.depth_m() <= 1.0 + 1e-6, "{}", pit.depth_m());
    // The ground lost it, and the records reproduce the ground.
    let map = sim.map();
    let mut replayed = earth::GroundDelta::new(map.width, map.height, map.cell_size_m);
    for w in &sim.land().earthworks {
        earth::replay(w, map, &mut replayed);
    }
    for cell in 0..(map.width * map.height) as usize {
        assert!(
            (replayed.at(cell) - sim.land().ground.at(cell)).abs() < 1e-3,
            "cell {cell}"
        );
    }
    // Nobody builds on it, and the observer names it.
    assert!(!build::plot_clear(sim.land(), map, sim.nav(), &pit.rect));
    let bytes = civ_sim::frames::earthworks::earthworks_response(&sim);
    let response =
        civ_schema::flatbuffers::root::<civ_schema::wire::Response>(&bytes).expect("decodes");
    let info = response
        .body_as_earthworks()
        .expect("earthworks")
        .works()
        .expect("works")
        .iter()
        .find(|w| w.id() == pit.id.get())
        .expect("the daub pit");
    assert_eq!((info.kind(), info.deposit()), (1, 0));
    let words = info.words().unwrap_or_default();
    assert!(
        words.starts_with("the daub pit of ")
            && words.contains("'s hut, ")
            && words.ends_with(" m deep"),
        "{words}"
    );
}
