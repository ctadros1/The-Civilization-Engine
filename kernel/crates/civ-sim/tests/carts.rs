//! Wheels on a lived world's ground (M8b slice CP, first part; ADR-0029 §2, §6, §7): a handcart
//! goes on the trails people wore and never across open ground, a river, or a log laid for
//! walkers, but across a deck as wide as it is; between two places on the trails it takes the
//! quickest way the oracle also finds.

use std::path::Path;
use std::sync::OnceLock;

use civ_content::ContentRegistry;
use civ_land::carting::{Terrain, Wheels};
use civ_land::crossings::{Crossing, CrossingOwner, CrossingState};
use civ_sim::{NewWorld, Sim};
use civ_world::{WATER_LAND, WATER_RIVER};

const PRESET: &str = "core:worldgen/river_valley";
const DAY: i64 = 24 * 60;

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

/// A village of world `seed`, with a fixed identity so it lives the same life every run.
fn village(seed: u64) -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Carts".to_owned(),
            seed,
            preset_id: PRESET.to_owned(),
            size_cells: 768,
            band_size: 30,
            neighbours: Vec::new(),
            neighbours_known: true,
            regime_id: String::new(),
        },
        content(),
        [13; 16],
    )
    .expect("generates")
}

/// A loaded handcart of the transport brief's priors (12-05 §2.3, 07-08 §2.1, 10-07 §1.2): 40 kg
/// of cart and 150 kg of load, 0.9 m across.
fn handcart() -> Wheels {
    Wheels {
        mass_kg: 190.0,
        width_m: 0.9,
        way_c: 0.02,
        trail_c: 0.05,
        pull_n: 200.0,
        power_w: 75.0,
        walk_m_s: 1.3,
        brake_grade: 0.15,
    }
}

/// A river cell with dry land either side of it, east and west or north and south: the banks
/// and the cell.
fn river_with_banks(sim: &Sim) -> ([u32; 2], u32) {
    let map = sim.map();
    let w = map.width;
    let land = |x: u32| map.water.get(x as usize) == Some(&WATER_LAND);
    (w..map.width * map.height - w)
        .filter(|&c| map.water[c as usize] == WATER_RIVER && c % w != 0 && c % w != w - 1)
        .find_map(|c| {
            [[c - 1, c + 1], [c - w, c + w]]
                .into_iter()
                .find(|b| b.iter().all(|&x| land(x)))
                .map(|b| (b, c))
        })
        .expect("a river cell with banks")
}

#[test]
fn a_cart_crosses_no_log_but_a_deck_as_wide_as_it_is() {
    let mut sim = village(1);
    let (banks, cell) = river_with_banks(&sim);
    let wheels = handcart();
    assert_eq!(
        sim.land().cart_resistance(sim.map(), cell, &wheels),
        None,
        "no crossing: water"
    );
    let system = sim
        .rules()
        .catalog
        .bridge_index("core:bridge/log_beam")
        .expect("the core has a log footbridge") as u16;
    let id = sim.allocate_id_for_tests();
    let now = sim.now();
    let household = sim
        .people()
        .households
        .iter()
        .map(|(_, h)| h.id)
        .min()
        .expect("a household");
    // The core's log footbridge: two logs of 20 cm side by side, 0.4 m of deck.
    sim.land_mut_for_tests().crossings.list.push(Crossing {
        id,
        system,
        banks,
        cells: vec![cell],
        span_m: 6.0,
        members: 2,
        diameter_cm: 20.0,
        quality: 1.0,
        loss: 0.0,
        owner: CrossingOwner::Household(household),
        labour_h: 0.0,
        skill_h: 0.0,
        begun: now,
        state: CrossingState::Open {
            since: now.day_index(),
        },
        shares: Vec::new(),
        crew: (0, 0),
    });
    sim.land_mut_for_tests().crossings.changed();
    assert_eq!(
        sim.land().cart_resistance(sim.map(), cell, &wheels),
        None,
        "a log is a walker's"
    );
    // Six members of 25 cm: a deck of 1.5 m takes the cart.
    let land = sim.land_mut_for_tests();
    land.crossings.list[0].members = 6;
    land.crossings.list[0].diameter_cm = 25.0;
    assert_eq!(
        sim.land().cart_resistance(sim.map(), cell, &wheels),
        Some(wheels.way_c)
    );
}

#[test]
fn a_cart_keeps_to_the_trails_people_wore_by_the_quickest_way() {
    let mut sim = village(1);
    // Two months: the trails round the camp worn and surveyed.
    sim.advance_minutes(61 * DAY).expect("lives");
    let (map, land) = (sim.map(), sim.land());
    let wheels = handcart();
    let cells = map.width * map.height;
    let trail: Vec<u32> = (0..cells)
        .filter(|&c| land.wear.is_trail(c as usize))
        .collect();
    assert!(trail.len() > 20, "trails were worn: {}", trail.len());
    let res = |c: u32| land.cart_resistance(map, c, &wheels);
    // Off the trails is open ground: no cart goes there.
    let open = (0..cells)
        .find(|&c| map.water[c as usize] == WATER_LAND && !land.wear.is_trail(c as usize))
        .expect("open ground");
    assert_eq!(res(open), None);
    assert!(
        trail
            .iter()
            .filter(|&&c| map.water[c as usize] == WATER_LAND)
            .all(|&c| res(c) == Some(wheels.trail_c))
    );
    let terrain = Terrain {
        elevation: &map.elevation,
        water: &map.water,
        decks: &[],
        width: map.width,
        height: map.height,
        cell_m: f64::from(map.cell_size_m),
    };
    let from = trail[0];
    let mut joined = 0;
    for &to in trail.iter().skip(1).step_by((trail.len() / 40).max(1)) {
        let Some(route) = terrain.route(&wheels, &res, &[from], &[to]) else {
            continue;
        };
        joined += 1;
        assert!(
            route.cells.iter().all(|&c| res(c).is_some()),
            "on the trails"
        );
        let oracle = terrain
            .oracle_seconds(&wheels, &res, &trail, &[from], &[to])
            .expect("the oracle finds it too");
        assert!(
            (oracle - route.seconds).abs() < 1e-6,
            "{} against {oracle}",
            route.seconds
        );
        // No faster than a walk over the metres it goes.
        assert!(route.seconds + 1e-9 >= route.metres / wheels.walk_m_s);
    }
    assert!(joined > 0, "some trail cells are joined for a cart");
}

#[test]
fn a_road_engine_goes_as_a_cart_goes_at_its_engines_speed_and_only_with_coal_for_the_trip() {
    // M11b, second part (ADR-0036 §7): a motor cart is a cart whose puller is its engine.
    use civ_land::ways::{CART, Keeper, Rank, Segment, Surface, WALK, Way};
    let mut sim = village(1);
    sim.advance_minutes(10 * DAY).expect("lives");
    let map = sim.map().clone();
    let w = i64::from(map.width);
    let cell = |at: (f32, f32)| civ_agents::population::cell_of(&map, at) as u32;
    let straight = |a: u32, b: u32| {
        let (mut x, mut y) = (i64::from(a) % w, i64::from(a) / w);
        let (bx, by) = (i64::from(b) % w, i64::from(b) / w);
        let mut cells = vec![a];
        while (x, y) != (bx, by) {
            x += (bx - x).signum();
            y += (by - y).signum();
            cells.push((y * w + x) as u32);
        }
        cells
    };
    let homes: Vec<(civ_core::PermanentId, (f32, f32))> = sim
        .people()
        .households
        .iter()
        .map(|(_, h)| (h.id, h.home))
        .collect();
    // A household and another some cells off, a straight line of dry land between their doors.
    let (owner, from, to_at, cells) = homes
        .iter()
        .flat_map(|a| homes.iter().map(move |b| (a, b)))
        .find_map(|(&(a, at_a), &(b, at_b))| {
            let cells = straight(cell(at_a), cell(at_b));
            (a != b
                && cells.len() >= 5
                && cells.iter().all(|&c| map.water[c as usize] == WATER_LAND))
            .then_some((a, cell(at_a), at_b, cells))
        })
        .expect("two doors with dry land between");
    // A way serving carts, wide enough for the engine, from door to door.
    let way = sim.allocate_id_for_tests();
    let today = sim.now().day_index();
    let land = sim.land_mut_for_tests();
    land.ways.add(Way {
        id: way,
        begun: today,
        segments: vec![Segment {
            cells: cells.clone(),
            corridor_m: 4.0,
            built_m: 4.0,
            usable_m: 4.0,
            surface: Surface::Cleared,
            keeper: Keeper::Nobody,
            kept: today,
        }],
        modes: WALK | CART,
        trips: 0.0,
        trips_day: today,
        rank: Rank::Lane,
        track: None,
    });
    land.ways.changed();
    let catalog = &sim.rules().catalog;
    let engine = catalog
        .vehicle_index("core:vehicle/road_engine")
        .expect("a road engine");
    let handcart = catalog
        .vehicle_index("core:vehicle/handcart")
        .expect("a handcart");
    let coal = catalog
        .goods
        .iter()
        .position(|g| g.id == "core:good/coal")
        .expect("coal");
    let def = catalog.vehicles[engine].clone();
    let goods = catalog.goods.clone();
    let put = |sim: &mut Sim, vehicle: usize| {
        let id = sim.allocate_id_for_tests();
        let land = sim.land_mut_for_tests();
        land.carts.list.clear();
        land.carts.list.push(civ_land::boats::Boat {
            id,
            vehicle: vehicle as u16,
            owner,
            at: from,
            made: today,
            wear: 0.0,
            labour_h: 0.0,
            work_h: 0.0,
            saves_h_year: 0.0,
            crew: (0, 0),
        });
        land.carts.changed();
        id
    };
    let set_coal = |sim: &mut Sim, kg: f64| {
        for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
            if x.id == owner {
                x.stores[coal] = kg;
            }
        }
    };
    let way_of = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| pop.cart_way_for_tests(ctx, owner, to_at))
    };
    // A handcart there and back: no fuel.
    put(&mut sim, handcart);
    let (by_hand, none) = way_of(&mut sim).expect("a handcart goes along the way");
    assert_eq!(none, None);
    // The road engine without coal goes nowhere.
    let cart = put(&mut sim, engine);
    set_coal(&mut sim, 0.0);
    assert_eq!(way_of(&mut sim), None);
    // With coal it goes, quicker than the handcart, burning its power's work over its efficiency.
    set_coal(&mut sim, 1_000.0);
    let (by_engine, fuel) = way_of(&mut sim).expect("the engine goes with coal");
    assert!(
        by_engine < by_hand,
        "{by_engine} h against {by_hand} h by hand"
    );
    let (f, kg) = fuel.expect("it burns coal");
    let want = 3.6 * (def.power_w / 1000.0 * by_engine) / (25.8 * 0.05);
    assert_eq!(f, coal);
    assert!((kg - want).abs() < 1e-6, "{kg} against {want}");
    // Without enough for the trip it goes nowhere either.
    set_coal(&mut sim, kg * 0.5);
    assert_eq!(way_of(&mut sim), None);
    // At the seller's door the trip's coal is burned.
    set_coal(&mut sim, 1_000.0);
    let trip = civ_agents::reports::ByCart {
        cart,
        travel_min: (by_engine * 60.0) as f32,
        load_min: 1.0,
        unload_min: 1.0,
    };
    sim.with_ctx_for_tests(|pop, ctx| pop.burn_trip_fuel_for_tests(ctx, owner, trip));
    let left = sim.people().household(owner).expect("owner").stores[coal];
    let burned = def
        .trip_fuel(&goods, f64::from(trip.travel_min))
        .expect("fuel")
        .1;
    assert!((1_000.0 - left - burned).abs() < 1e-6, "{left}");
}
