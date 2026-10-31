//! Climbing, lifts, taller frames and wind (ADR-0042). Members living above the ground climb their
//! storeys on every trip, and that time enters a home's design hours for the storeys they would
//! sleep on and a lodger's weighing of a room for the storey its floor lies on, so an upper floor
//! is taken only where it costs less than a ground floor would.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::build::{self, Shape};
use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_grammar::Stage;
use civ_sim::{NewWorld, Sim};

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

fn world(seed: u64) -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Tall".to_owned(),
            seed,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 512,
            band_size: 30,
            neighbours: Vec::new(),
            neighbours_known: false,
            regime_id: String::new(),
        },
        content(),
        [8; 16],
    )
    .expect("generates")
}

fn good(sim: &Sim, id: &str) -> usize {
    sim.rules()
        .catalog
        .goods
        .iter()
        .position(|g| g.id == id)
        .expect(id)
}

fn program(id: &str) -> &'static civ_agents::params::BuildingDef {
    let catalog = &content().catalog;
    &catalog.buildings[catalog.building_index(id).expect(id)]
}

fn members(sim: &Sim, h: PermanentId) -> usize {
    sim.people().household(h).expect("household").members.len()
}

/// Gives household `h` a finished building of `shape` and `program` centred at `at`.
fn give_home_at(
    sim: &mut Sim,
    h: PermanentId,
    program_id: &str,
    shape: Shape,
    at: (f32, f32),
) -> PermanentId {
    let spec = build::design_shape(
        program(program_id),
        &content().catalog.goods,
        shape,
        at,
        None,
    )
    .expect("a design");
    sim.place_building_for_tests(h, spec, Stage::ALL.len() as u8)
        .expect("placed")
}

/// A world's households of one settlement, by id, a day in, each with `kg` of provisions.
fn village(kg: f64) -> (Sim, Vec<PermanentId>) {
    let mut sim = world(3);
    sim.advance_minutes(DAY).expect("lives");
    let settlement = sim
        .people()
        .households
        .iter()
        .map(|(_, x)| x)
        .filter(|x| !x.members.is_empty())
        .find_map(|x| x.settlement)
        .expect("a settlement");
    let mut ids: Vec<PermanentId> = sim
        .people()
        .households
        .iter()
        .map(|(_, x)| x)
        .filter(|x| !x.members.is_empty() && x.settlement == Some(settlement))
        .map(|x| x.id)
        .collect();
    ids.sort_unstable();
    assert!(ids.len() >= 4, "{} households", ids.len());
    let provisions = good(&sim, "core:good/provisions");
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if ids.contains(&x.id) {
            x.stores[provisions] = kg;
        }
    }
    (sim, ids)
}

/// The centre of the place about `at` that people go to, metres, and its id.
fn place_about(sim: &Sim, at: (f32, f32)) -> ((f32, f32), u32) {
    let (w, cs) = (sim.map().width, sim.map().cell_size_m);
    let cell = (at.1 / cs) as u32 * w + (at.0 / cs) as u32;
    let place = civ_agents::travel::going_place(cell, w);
    (civ_agents::travel::going_centre(place, w, cs), place)
}

/// Household `h`'s members each went `trips` times to the place about `at` from home, as of today.
fn trips_to(sim: &mut Sim, h: PermanentId, at: (f32, f32), trips: f32) {
    let (_, place) = place_about(sim, at);
    let today = sim.now().day_index();
    let people = sim.people().household(h).expect("it").members.clone();
    for m in people {
        sim.people_mut_for_tests().travel.goings.insert(
            m,
            civ_agents::travel::Goings {
                day: today,
                to: vec![(place, trips)],
            },
        );
    }
}

fn climb(sim: &mut Sim, h: PermanentId, shape: Shape) -> f64 {
    sim.with_ctx_for_tests(|pop, ctx| {
        pop.design_climb_for_tests(ctx, h, "core:building/longhouse", shape)
    })
}

#[test]
fn a_homes_design_counts_the_climb_of_members_sleeping_above_the_ground() {
    let (mut sim, ids) = village(5_000.0);
    let h = ids[0];
    let n = members(&sim, h);
    assert!(n >= 2, "{n} members");
    let home = sim.people().household(h).expect("it").home;
    let one = Shape::Bays {
        bays: 2,
        storeys: 1,
        lofts: 0,
    };
    let narrow = Shape::Bays {
        bays: 1,
        storeys: 2,
        lofts: 0,
    };
    let wide = Shape::Bays {
        bays: 8,
        storeys: 2,
        lofts: 0,
    };
    // Nobody has gone anywhere: no climbing counted.
    trips_to(&mut sim, h, home, 0.0);
    assert_eq!(climb(&mut sim, h, narrow), 0.0);
    trips_to(&mut sim, h, home, 50.0);
    // One storey: nobody climbs.
    assert_eq!(climb(&mut sim, h, one), 0.0);
    // Two storeys whose first holds them all: nobody climbs.
    assert_eq!(climb(&mut sim, h, wide), 0.0);
    // One bay of two storeys: those whose floor lies above the first storey climb on each trip.
    let def = program("core:building/longhouse");
    let cost = def
        .shapes
        .iter()
        .find(|c| c.shape == narrow)
        .expect("the narrow shape");
    let (base, per) = (6.0, 4.8);
    let storeys = civ_agents::climb::members_storeys(&cost.living_by_storey, base, per, n);
    let up = storeys.iter().filter(|&&t| t > 0).count();
    assert!(up > 0 && up < n, "{storeys:?}");
    let h50 = climb(&mut sim, h, narrow);
    assert!(h50 > 0.0);
    // In proportion to the trips and to those upstairs, at 15 s up and 8 s down a storey.
    trips_to(&mut sim, h, home, 100.0);
    let h100 = climb(&mut sim, h, narrow);
    assert!((h100 - 2.0 * h50).abs() < 1e-9 * h100, "{h50} {h100}");
    let trips_year = h100 * 3600.0 / 23.0 * n as f64 / up as f64;
    assert!(
        (civ_agents::climb::climb_h_year(&storeys, trips_year, |t| 23.0 * t as f64) - h100).abs()
            < 1e-9
    );
}

/// A village where `a` and `b` each have a longhouse of eight bays and two storeys, 40 metres
/// either side of `lodger`'s home, `b` already letting all of its upper storey to `c`; `lodger`'s
/// members have gone 500 times each (about two trips a day) to the place about its home.
fn two_landlords() -> (Sim, [PermanentId; 4]) {
    let (mut sim, ids) = village(5_000.0);
    let (a, b, c, lodger) = (ids[0], ids[1], ids[2], ids[3]);
    // The lodger lives at the middle of the place its people go to, the two homes either side.
    let (home, _) = place_about(&sim, sim.people().household(lodger).expect("it").home);
    for (_, h) in sim.people_mut_for_tests().households.iter_mut() {
        if h.id == lodger {
            h.home = home;
        }
    }
    let tall = Shape::Bays {
        bays: 8,
        storeys: 2,
        lofts: 0,
    };
    let x = 40.0;
    let ba = give_home_at(
        &mut sim,
        a,
        "core:building/longhouse",
        tall,
        (home.0, home.1 - x),
    );
    let bb = give_home_at(
        &mut sim,
        b,
        "core:building/longhouse",
        tall,
        (home.0, home.1 + x),
    );
    trips_to(&mut sim, lodger, home, 500.0);
    // `b`'s upper storey is all let to `c`.
    let def = program("core:building/longhouse");
    let upper = def
        .shapes
        .iter()
        .find(|s| s.shape == tall)
        .expect("tall")
        .living_by_storey[1];
    let _ = ba;
    let day = sim.now().day_index();
    let provisions = good(&sim, "core:good/provisions");
    let own_home = sim.people().household(c).expect("c").home;
    sim.people_mut_for_tests()
        .letting
        .lets
        .push(civ_agents::letting::Let {
            landlord: b,
            lodger: c,
            building: bb,
            floor_m2: upper,
            good: provisions,
            rent_kg: 1.0,
            begun: day,
            due: day + 30,
            next_review: day + 30,
            unpaid: 0,
            own_home,
        });
    (sim, [a, b, c, lodger])
}

#[test]
fn a_lodger_weighs_the_climb_to_its_rooms_storey_so_upper_floors_let_for_less() {
    let (mut sim, [a, b, _, lodger]) = two_landlords();
    assert!(a < b);
    // The two rooms are alike in all but their storey: `a`'s lies on its upper storey, `b`'s on
    // its first, its upper storey let. Equally near, and asking the same, the first is taken.
    sim.with_ctx_for_tests(|pop, ctx| pop.letting_day_for_tests(ctx, &[lodger]));
    let l = sim
        .people()
        .letting
        .lodging(lodger)
        .cloned()
        .expect("it lodges");
    assert_eq!(l.landlord, b, "the room on the first storey");
    // Asking a fiftieth of a food-day less, the upper room still costs more than the first
    // storey's by its climb; asking two fifths, it is taken: an upper floor lets for less.
    let prior = sim
        .rules()
        .people
        .letting
        .asking_rent_food_days_per_person_month;
    for (asking, taken) in [(prior - 0.02, b), (prior * 0.4, a)] {
        let (mut sim, [a2, _, _, lodger]) = two_landlords();
        assert_eq!(a2, a);
        sim.people_mut_for_tests().letting.asking.insert(a, asking);
        sim.with_ctx_for_tests(|pop, ctx| pop.letting_day_for_tests(ctx, &[lodger]));
        let l = sim
            .people()
            .letting
            .lodging(lodger)
            .cloned()
            .expect("it lodges");
        assert_eq!(l.landlord, taken, "asking {asking}");
    }
}
