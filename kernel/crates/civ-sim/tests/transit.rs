//! Road transit (M11b, third part; ADR-0036 §7): a motor cart's posted runs between its keeper's
//! door and another settlement's hearth, ridden for a fare in food.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::ledger::Channel;
use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_sim::{NewWorld, Sim, persist};
use civ_world::WATER_LAND;

/// The core content, its road engine able to pull up and brake down any grade (slowly, by its
/// power), so the scene need not hunt for a gentle way between two settlements.
fn content() -> &'static ContentRegistry {
    static C: OnceLock<ContentRegistry> = OnceLock::new();
    C.get_or_init(|| {
        let mut c = civ_content::load(
            &civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("root"),
        )
        .registry
        .expect("content");
        let engine = c
            .catalog
            .vehicle_index("core:vehicle/road_engine")
            .expect("a road engine");
        c.catalog.vehicles[engine].pull_n = 1e6;
        c.catalog.vehicles[engine].brake_grade = 10.0;
        c
    })
}

/// Two settlements founded apart, of seed `seed`.
fn world(seed: u64) -> Sim {
    let sim = Sim::create_for_tests(
        &NewWorld {
            name: "Transit".to_owned(),
            seed,
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
    sim
}

fn straight(width: u32, a: u32, b: u32) -> Vec<u32> {
    let w = i64::from(width);
    let (mut x, mut y) = (i64::from(a) % w, i64::from(a) / w);
    let (bx, by) = (i64::from(b) % w, i64::from(b) / w);
    let mut cells = vec![a];
    // A step at a time along x or y, never corner to corner, so the way never squeezes past a
    // stream on a diagonal.
    while (x, y) != (bx, by) {
        if (bx - x).abs() >= (by - y).abs() {
            x += (bx - x).signum();
        } else {
            y += (by - y).signum();
        }
        cells.push((y * w + x) as u32);
    }
    cells
}

fn food_kcal(sim: &Sim, channel: Channel) -> f64 {
    let goods = &sim.rules().catalog.goods;
    (0..goods.len())
        .map(|g| sim.people().transfers.get(channel, g) * goods[g].kcal_per_kg)
        .sum()
}

#[test]
fn a_motor_carts_keeper_posts_runs_to_a_market_and_riders_pay_their_fare_in_food() {
    use civ_land::ways::{CART, Keeper, Rank, Segment, Surface, WALK, Way};
    // The first seed with a keeper whose door has dry land straight to the other settlement's
    // hearth.
    let (mut sim, keeper, home, market, cells) = (1..40u64)
        .find_map(|seed| {
            let sim = world(seed);
            let map = sim.map().clone();
            let cell = |at: (f32, f32)| civ_agents::population::cell_of(&map, at) as u32;
            let settlements: Vec<(PermanentId, (f32, f32))> = sim
                .land()
                .settlements
                .iter()
                .map(|s| (s.id, s.hearth_m))
                .collect();
            let found = sim
                .people()
                .households
                .iter()
                .map(|(_, h)| h)
                .filter(|h| !h.members.is_empty())
                .find_map(|h| {
                    let s = h.settlement?;
                    let &(far, hearth) = settlements.iter().find(|x| x.0 != s)?;
                    let cells = straight(map.width, cell(h.home), cell(hearth));
                    cells
                        .iter()
                        .all(|&c| map.water[c as usize] == WATER_LAND)
                        .then_some((h.id, s, far, cells))
                });
            found.map(|(k, h, m, c)| (sim, k, h, m, c))
        })
        .expect("a world with a door with dry land to the other hearth");
    let map = sim.map().clone();
    let rider = sim
        .people()
        .households
        .iter()
        .map(|(_, h)| h)
        .find(|h| h.settlement == Some(home) && h.id != keeper && !h.members.is_empty())
        .map(|h| h.id)
        .expect("a neighbour");
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
    let coal = catalog
        .goods
        .iter()
        .position(|g| g.id == "core:good/coal")
        .expect("coal");
    let review = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| pop.review_road_line_for_tests(ctx, keeper))
    };
    // No motor cart: nothing posted.
    review(&mut sim);
    assert!(sim.land().runs.roads.is_empty());
    let cart = sim.allocate_id_for_tests();
    let land = sim.land_mut_for_tests();
    land.carts.list.push(civ_land::boats::Boat {
        id: cart,
        vehicle: engine as u16,
        owner: keeper,
        at: cells[0],
        made: today,
        wear: 0.0,
        labour_h: 0.0,
        work_h: 0.0,
        saves_h_year: 0.0,
        crew: (0, 0),
    });
    land.carts.changed();
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == keeper {
            x.stores[coal] = 5000.0;
        }
    }
    // A cart but no errands run there: nothing posted.
    review(&mut sim);
    assert!(sim.land().runs.roads.is_empty());
    let year = today.div_euclid(civ_core::time::DAYS_PER_YEAR);
    sim.people_mut_for_tests()
        .contacts
        .years
        .entry((year, home, market))
        .or_default()
        .bought = 730;
    review(&mut sim);
    let line = sim.land().runs.roads.first().cloned().expect("a road line");
    assert_eq!(
        (line.operator, line.cart, line.market, line.runs_a_day),
        (keeper, cart, market, 1)
    );
    assert!(line.fare_kcal > 0.0 && line.run_min() > 0.0, "{line:?}");
    let (first, last) = (line.cells[0], *line.cells.last().expect("cells"));
    // A neighbour at the station rides out on the day's run, paying the fare in food; the cart
    // burns the run's coal for its first rider only.
    let who = sim.people().household(rider).expect("rider").members[0];
    let station = civ_agents::population::cell_centre(&map, first as usize);
    let set_at = |sim: &mut Sim, p: PermanentId, at: (f32, f32)| {
        for (_, q) in sim.people_mut_for_tests().people.iter_mut() {
            if q.id == p {
                q.pos = at;
            }
        }
    };
    set_at(&mut sim, who, station);
    let coal_of = |sim: &Sim| sim.people().household(keeper).expect("keeper").stores[coal];
    let before = coal_of(&sim);
    let ride = |sim: &mut Sim, p, to| {
        sim.with_ctx_for_tests(|pop, ctx| pop.ride_in_block_for_tests(ctx, p, to))
    };
    assert!(ride(&mut sim, who, last));
    let burned = before - coal_of(&sim);
    assert!(burned > 0.0, "the run burned coal");
    assert!((food_kcal(&sim, Channel::Fare) - f64::from(line.fare_kcal)).abs() < 1e-3);
    assert_eq!(sim.land().runs.roads[0].riders, 1);
    let end = civ_agents::population::cell_centre(&map, last as usize);
    let pos = sim.people().person(who).expect("rider").pos;
    assert!((pos.0 - end.0).abs() < 1e-3 && (pos.1 - end.1).abs() < 1e-3);
    // Another rider of the same run: no more coal burned.
    let second = sim
        .people()
        .household(rider)
        .expect("rider")
        .members
        .get(1)
        .copied();
    if let Some(second) = second {
        set_at(&mut sim, second, station);
        assert!(ride(&mut sim, second, last));
        assert!((before - coal_of(&sim) - burned).abs() < 1e-9);
    }
    // Back the other way on the run home: its coal burned too.
    let mid = coal_of(&sim);
    assert!(ride(&mut sim, who, first));
    assert!(coal_of(&sim) < mid);
    // Without coal for a run, nobody rides it.
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == keeper {
            x.stores[coal] = 0.0;
        }
    }
    sim.land_mut_for_tests().runs.roads[0].last_run = -1;
    set_at(&mut sim, who, station);
    assert!(!ride(&mut sim, who, last));
    // A year's rides set its runs and fare; saved and loaded exactly.
    let before = sim.land().runs.clone();
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = persist::save(
        &mut sim,
        &saves,
        commons_persist::SaveKind::Manual,
        "transit",
    )
    .expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loaded.land().runs, before);
    // What the observer says: an errand ridden there, and the keeper's runs on its first building
    // once its home stands.
    let line_id = sim.land().runs.roads[0].id;
    let now_day = sim.now().day_index();
    sim.people_mut_for_tests().reports.errands.insert(
        rider,
        civ_agents::reports::Errand {
            market,
            seller: keeper,
            firm: false,
            good: 0,
            payment: 0,
            units: 1.0,
            home_h: 1.0,
            share: 0.5,
            day: now_day,
            by_boat: None,
            by_cart: None,
            by_ride: Some(civ_agents::reports::ByRide {
                line: line_id,
                travel_min: 90.0,
            }),
        },
    );
    let words = civ_sim::frames::markets::errand_words(&sim, rider);
    assert!(words.contains("motor cart there and back"), "{words}");
    sim.advance_minutes(10 * 24 * 60).expect("lives");
    if let Some(home_id) = sim
        .land()
        .buildings
        .iter()
        .find(|b| b.household == keeper)
        .map(|b| b.id)
        && !sim.land().runs.roads.is_empty()
    {
        let bytes = civ_sim::frames::buildings::buildings_response(&sim);
        let response =
            civ_schema::flatbuffers::root::<civ_schema::wire::Response>(&bytes).expect("wire");
        let words = response
            .body_as_buildings()
            .expect("buildings")
            .buildings()
            .expect("a list")
            .iter()
            .find(|b| b.id() == home_id.get())
            .expect("its home")
            .road()
            .unwrap_or_default()
            .to_owned();
        assert!(
            words.starts_with("Motor cart runs to ") && words.contains("a day, leaving at 06:00"),
            "{words}"
        );
    }
    // Its cart gone, the keeper stops its runs.
    sim.land_mut_for_tests().carts.list.clear();
    review(&mut sim);
    assert!(sim.land().runs.roads.is_empty());
}
