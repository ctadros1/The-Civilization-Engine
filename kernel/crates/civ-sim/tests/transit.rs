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

/// A world with a keeper whose door has dry land straight to the other settlement's hearth, and a
/// neighbour of its settlement whose door has dry land straight to the keeper's: the world, the
/// keeper, the neighbour, their settlement, the other settlement, the cells from the keeper's door
/// to that hearth, and those from the neighbour's door to the keeper's. A way serving carts runs
/// along each.
#[allow(clippy::type_complexity)]
fn two_doors_and_a_hearth() -> (
    Sim,
    PermanentId,
    PermanentId,
    PermanentId,
    PermanentId,
    Vec<u32>,
    Vec<u32>,
) {
    use civ_land::ways::{CART, Keeper, Rank, Segment, Surface, WALK, Way};
    let (mut sim, keeper, heir, home, market, cells, short) = (1..60u64)
        .find_map(|seed| {
            let sim = world(seed);
            let map = sim.map().clone();
            let cell = |at: (f32, f32)| civ_agents::population::cell_of(&map, at) as u32;
            let dry = |cells: &[u32]| cells.iter().all(|&c| map.water[c as usize] == WATER_LAND);
            let settlements: Vec<(PermanentId, (f32, f32))> = sim
                .land()
                .settlements
                .iter()
                .map(|s| (s.id, s.hearth_m))
                .collect();
            let homes: Vec<(PermanentId, PermanentId, (f32, f32))> = sim
                .people()
                .households
                .iter()
                .map(|(_, h)| h)
                .filter(|h| !h.members.is_empty())
                .filter_map(|h| Some((h.id, h.settlement?, h.home)))
                .collect();
            homes
                .iter()
                .find_map(|&(k, s, door)| {
                    let &(far, hearth) = settlements.iter().find(|x| x.0 != s)?;
                    let cells = straight(map.width, cell(door), cell(hearth));
                    if !dry(&cells) {
                        return None;
                    }
                    homes.iter().find_map(|&(n, ns, at)| {
                        let short = straight(map.width, cell(at), cell(door));
                        (n != k && ns == s && short.len() >= 2 && dry(&short))
                            .then(|| (k, n, s, far, cells.clone(), short))
                    })
                })
                .map(|(k, n, s, far, c, sh)| (sim, k, n, s, far, c, sh))
        })
        .expect("a world with two doors and dry land to the other hearth");
    let today = sim.now().day_index();
    for c in [cells.clone(), short.clone()] {
        let way = sim.allocate_id_for_tests();
        let land = sim.land_mut_for_tests();
        land.ways.add(Way {
            id: way,
            begun: today,
            segments: vec![Segment {
                cells: c,
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
    }
    (sim, keeper, heir, home, market, cells, short)
}

/// A motor cart's keeper with a road line posted, its cart at its door and coal for it: the
/// cart.
fn posted(
    sim: &mut Sim,
    keeper: PermanentId,
    home: PermanentId,
    market: PermanentId,
    door: u32,
) -> PermanentId {
    let today = sim.now().day_index();
    let engine = sim
        .rules()
        .catalog
        .vehicle_index("core:vehicle/road_engine")
        .expect("a road engine");
    let cart = sim.allocate_id_for_tests();
    let land = sim.land_mut_for_tests();
    land.carts.list.push(civ_land::boats::Boat {
        id: cart,
        vehicle: engine as u16,
        owner: keeper,
        at: door,
        made: today,
        wear: 0.0,
        labour_h: 0.0,
        work_h: 0.0,
        saves_h_year: 0.0,
        crew: (0, 0),
    });
    land.carts.changed();
    let coal = sim
        .rules()
        .catalog
        .goods
        .iter()
        .position(|g| g.id == "core:good/coal")
        .expect("coal");
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == keeper {
            x.stores[coal] = 5000.0;
        }
    }
    let year = today.div_euclid(civ_core::time::DAYS_PER_YEAR);
    sim.people_mut_for_tests()
        .contacts
        .years
        .entry((year, home, market))
        .or_default()
        .bought = 730;
    sim.with_ctx_for_tests(|pop, ctx| pop.review_road_line_for_tests(ctx, keeper));
    assert_eq!(sim.land().runs.roads.len(), 1, "a road line posted");
    cart
}

/// A road line passes with its motor cart (backlog D, twentieth item; the eighteenth item's "Not
/// yet"): its keeper's people joining a household of its settlement that keeps no line, the cart
/// goes to that household's door and the line with it, its runs now leaving from there along the
/// cart's way to the same hearth, its timetable and riders kept and its fare scaled to the run's
/// minutes; that household's people joining one of the other settlement, where a motor cart does
/// not go, the line runs no more; a keeper whose people all die leaves its line with whoever its
/// cart goes to, if anyone; and a line whose keeper is no more runs no more by the next midnight. Each saves and loads exactly.
#[test]
fn a_road_line_passes_with_its_motor_cart_and_runs_no_more_where_it_cannot() {
    let (mut sim, keeper, heir, home, market, cells, short) = two_doors_and_a_hearth();
    let cart = posted(&mut sim, keeper, home, market, cells[0]);
    {
        let l = &mut sim.land_mut_for_tests().runs.roads[0];
        (l.runs_a_day, l.riders) = (2, 17);
    }
    let before = sim.land().runs.roads[0].clone();
    let rev = sim.land().runs.revision();
    sim.with_ctx_for_tests(|pop, ctx| pop.merge_household_for_tests(ctx, keeper, heir));
    let cart_now = sim
        .land()
        .carts
        .list
        .iter()
        .find(|c| c.id == cart)
        .cloned()
        .expect("the cart");
    assert_eq!(
        (cart_now.owner, cart_now.at),
        (heir, short[0]),
        "wheeled to its door"
    );
    let roads = &sim.land().runs.roads;
    assert_eq!(roads.len(), 1);
    let l = &roads[0];
    assert_eq!(
        (
            l.id,
            l.operator,
            l.cart,
            l.market,
            l.runs_a_day,
            l.riders,
            l.since
        ),
        (before.id, heir, cart, market, 2, 17, before.since)
    );
    assert_eq!(l.cells[0], short[0], "its runs leave from the heir's door");
    assert_eq!(l.cells.last(), before.cells.last(), "to the same hearth");
    // Timed afresh along the way from its door (here a little nearer the hearth).
    assert_eq!(l.minutes.len(), l.cells.len());
    assert!((l.run_min() - before.run_min()).abs() > 1e-3);
    assert!(
        (f64::from(l.fare_kcal)
            - f64::from(before.fare_kcal) * f64::from(l.run_min()) / f64::from(before.run_min()))
        .abs()
            < 1e-3,
        "{} against {}",
        l.fare_kcal,
        before.fare_kcal
    );
    assert_ne!(sim.land().runs.revision(), rev);
    let saved = |sim: &mut Sim| {
        let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
        let saves =
            commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
        let saved = persist::save(sim, &saves, commons_persist::SaveKind::Manual, "transit")
            .expect("saves");
        let loaded = persist::load(&saved.path, content()).expect("loads");
        assert_eq!(loaded.land().runs, sim.land().runs);
    };
    saved(&mut sim);
    // Its people joining a household of the other settlement: the motor cart stays, and the line
    // runs no more.
    let away = sim
        .people()
        .households
        .iter()
        .map(|(_, h)| h)
        .find(|h| h.settlement == Some(market) && !h.members.is_empty())
        .map(|h| h.id)
        .expect("a household of the other settlement");
    sim.with_ctx_for_tests(|pop, ctx| pop.merge_household_for_tests(ctx, heir, away));
    assert!(sim.land().runs.roads.is_empty(), "nobody posts its runs");
    saved(&mut sim);
    // A keeper whose people all die: its household goes to its heir, and the line with the cart
    // where the heir may run it, or runs no more.
    let (mut sim, keeper, _, home, market, cells, _) = two_doors_and_a_hearth();
    let cart = posted(&mut sim, keeper, home, market, cells[0]);
    let posted_line = sim.land().runs.roads[0].clone();
    let members = sim
        .people()
        .household(keeper)
        .expect("keeper")
        .members
        .clone();
    for m in members {
        sim.with_ctx_for_tests(|pop, ctx| pop.die_for_tests(ctx, m));
    }
    assert!(sim.people().household(keeper).is_none());
    for l in &sim.land().runs.roads {
        assert_ne!(l.operator, keeper);
        let owner = sim
            .land()
            .carts
            .list
            .iter()
            .find(|c| c.id == cart)
            .map(|c| c.owner);
        assert_eq!(owner, Some(l.operator), "the line goes with its cart");
        assert!(sim.people().household(l.operator).is_some());
    }
    // A line whose keeper is no more, passed to nobody: by the next midnight it runs no more.
    let gone = sim.allocate_id_for_tests();
    if sim.land().runs.roads.is_empty() {
        let land = sim.land_mut_for_tests();
        land.runs.roads.push(posted_line);
        land.runs.changed();
    }
    sim.land_mut_for_tests().runs.roads[0].operator = gone;
    assert_eq!(sim.land().runs.roads.len(), 1, "until the midnight");
    let day = sim.now().day_index();
    sim.with_ctx_for_tests(|pop, ctx| pop.transit_reviews_for_tests(ctx, day));
    assert!(sim.land().runs.roads.is_empty());
    saved(&mut sim);
}

/// The chronicle's entries telling road lines, oldest first.
fn road_entries(sim: &Sim) -> Vec<civ_agents::ChronicleEvent> {
    sim.people()
        .chronicle
        .iter()
        .filter(|e| e.kind == civ_agents::ChronicleKind::RoadLine)
        .cloned()
        .collect()
}

/// A road line is told when it is posted, when it passes with its motor cart, and when it ends
/// (backlog D, twenty-second item): posted, naming its keeper's elder, its market, its runs and
/// fare, at its door in its settlement; passed, naming the heir's elder, at the heir's door;
/// ended where nobody could run it, naming nobody; ended by its keeper where nobody rode it in a
/// year or its cart is gone, saying which; and ended at midnight where its keeper is no more.
/// Each once, and the entries save and load.
#[test]
fn a_road_line_is_told_when_posted_passed_and_ended() {
    use civ_agents::RoadStep;
    let (mut sim, keeper, heir, home, market, cells, short) = two_doors_and_a_hearth();
    let map = sim.map().clone();
    let centre = |c: u32| civ_agents::population::cell_centre(&map, c as usize);
    let name = sim
        .land()
        .settlements
        .iter()
        .find(|s| s.id == market)
        .expect("the market")
        .name
        .clone();
    let elder_of = |sim: &Sim, e: &civ_agents::ChronicleEvent| sim.people().name_of(e.people[0]);
    posted(&mut sim, keeper, home, market, cells[0]);
    let fare = sim.land().runs.roads[0].fare_kcal;
    let told = road_entries(&sim);
    assert_eq!(told.len(), 1, "{told:?}");
    let t = &told[0];
    assert_eq!(t.number, f64::from(RoadStep::Posted as u8));
    assert_eq!(t.people.len(), 1);
    assert!(
        sim.people()
            .household(keeper)
            .expect("keeper")
            .members
            .contains(&t.people[0])
    );
    assert_eq!(
        (t.settlement, t.place),
        (Some(home), Some(centre(cells[0])))
    );
    assert_eq!(
        t.name,
        format!(
            "{}'s household began running its motor cart from its door to {name} and back, a \
             run a day, for {fare:.0} kcal of food a ride.",
            elder_of(&sim, t)
        )
    );
    // Passed with the cart to the heir: told once, at its door.
    sim.with_ctx_for_tests(|pop, ctx| pop.merge_household_for_tests(ctx, keeper, heir));
    let told = road_entries(&sim);
    assert_eq!(told.len(), 2);
    let t = &told[1];
    assert_eq!(t.number, f64::from(RoadStep::Passed as u8));
    assert!(
        sim.people()
            .household(heir)
            .expect("heir")
            .members
            .contains(&t.people[0])
    );
    assert_eq!(
        (t.settlement, t.place),
        (Some(home), Some(centre(short[0])))
    );
    assert_eq!(
        t.name,
        format!(
            "{}'s household took over the motor cart's runs to {name} with the cart, a run a day \
             from its door.",
            elder_of(&sim, t)
        )
    );
    // Its people join a household of the other settlement: ended, nobody named as stopping it,
    // the history saying whose it had been (backlog D, twenty-fourth item).
    let heir_elder = elder_of(&sim, t);
    let away = sim
        .people()
        .households
        .iter()
        .map(|(_, h)| h)
        .find(|h| h.settlement == Some(market) && !h.members.is_empty())
        .map(|h| h.id)
        .expect("a household of the other settlement");
    sim.with_ctx_for_tests(|pop, ctx| pop.merge_household_for_tests(ctx, heir, away));
    let told = road_entries(&sim);
    assert_eq!(told.len(), 3);
    let t = &told[2];
    assert_eq!(t.number, f64::from(RoadStep::Ended as u8));
    assert!(t.people.is_empty());
    assert_eq!(t.place, Some(centre(short[0])));
    assert!(t.settlement.is_some());
    assert_eq!(
        t.name,
        format!(
            "The motor cart's runs to {name}, which had been {heir_elder}'s household's, \
             stopped: nobody was left who could run them."
        )
    );
    // A keeper whose line nobody rode in a year stops it, and says so; posted again, it stops
    // it once its cart is gone; a line whose keeper is no more ends at midnight.
    let (mut sim, keeper, _, home, market, cells, _) = two_doors_and_a_hearth();
    posted(&mut sim, keeper, home, market, cells[0]);
    let day = sim.now().day_index();
    sim.land_mut_for_tests().runs.roads[0].since = day - civ_core::time::DAYS_PER_YEAR;
    sim.with_ctx_for_tests(|pop, ctx| pop.review_road_line_for_tests(ctx, keeper));
    assert!(sim.land().runs.roads.is_empty());
    let ended = |sim: &Sim| road_entries(sim).last().cloned().expect("told");
    let name = sim
        .land()
        .settlements
        .iter()
        .find(|s| s.id == market)
        .expect("the market")
        .name
        .clone();
    let t = ended(&sim);
    assert_eq!(t.number, f64::from(RoadStep::Ended as u8));
    assert_eq!(
        t.name,
        format!(
            "{}'s household stopped running its motor cart to {name}: nobody rode it in a year.",
            elder_of(&sim, &t)
        )
    );
    sim.with_ctx_for_tests(|pop, ctx| pop.review_road_line_for_tests(ctx, keeper));
    assert_eq!(sim.land().runs.roads.len(), 1, "posted again");
    sim.land_mut_for_tests().carts.list.clear();
    sim.with_ctx_for_tests(|pop, ctx| pop.review_road_line_for_tests(ctx, keeper));
    let t = ended(&sim);
    assert!(
        t.name.ends_with(&format!("to {name}: its cart was gone.")),
        "{}",
        t.name
    );
    assert_eq!(road_entries(&sim).len(), 4, "posted, ended, posted, ended");
    // A line whose keeper is no more.
    let line = civ_land::trains::RoadLine {
        operator: sim.allocate_id_for_tests(),
        ..posted_like(&sim, keeper, market, cells.clone())
    };
    sim.land_mut_for_tests().runs.roads.push(line);
    sim.with_ctx_for_tests(|pop, ctx| pop.transit_reviews_for_tests(ctx, day));
    assert!(sim.land().runs.roads.is_empty());
    let t = ended(&sim);
    assert!(t.people.is_empty());
    assert_eq!(t.settlement, Some(home), "the hearth nearest its door");
    assert_eq!(
        t.name,
        format!("The motor cart's runs to {name} stopped: nobody was left who could run them."),
        "with no record, whose it was is not said"
    );
    assert_eq!(road_entries(&sim).len(), 5);
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
    assert_eq!(road_entries(&loaded), road_entries(&sim));
}

/// A road line of `keeper`'s to `market` along `cells`, as posted, for scenes.
fn posted_like(
    sim: &Sim,
    keeper: PermanentId,
    market: PermanentId,
    cells: Vec<u32>,
) -> civ_land::trains::RoadLine {
    let minutes = (0..cells.len()).map(|i| i as f32).collect();
    civ_land::trains::RoadLine {
        id: PermanentId::from_raw(u64::MAX).expect("an id"),
        operator: keeper,
        cart: PermanentId::from_raw(u64::MAX - 1).expect("an id"),
        market,
        cells,
        minutes,
        runs_a_day: 1,
        fare_kcal: 10.0,
        riders: 0,
        since: sim.now().day_index(),
        last_run: -1,
    }
}

/// Whose a boat, cart or line taken up had been (backlog D, twenty-third item): a household that
/// moves to the other settlement leaves its handcart, standing at a neighbour's door, in the old
/// household's name, and the history keeps whose household that was (its elder, who went with
/// it); the neighbour taking it up is told so ("took up a handcart that had been Bo's
/// household's, which nobody was left to take, …"); the record saves and loads exactly, and once
/// nothing is left in the old household's name a month forgets it. A household that moves with
/// nothing left in its name leaves no record.
#[test]
fn a_cart_left_by_a_household_that_moved_is_told_as_whose_it_had_been() {
    let (mut sim, keeper, neighbour, _, market, _, short) = two_doors_and_a_hearth();
    let today = sim.now().day_index();
    let cart_v = sim
        .rules()
        .catalog
        .vehicle_index("core:vehicle/handcart")
        .expect("a handcart");
    let cart = sim.allocate_id_for_tests();
    let land = sim.land_mut_for_tests();
    land.carts.list.push(civ_land::boats::Boat {
        id: cart,
        vehicle: cart_v as u16,
        owner: keeper,
        at: short[0],
        made: today,
        wear: 0.0,
        labour_h: 0.0,
        work_h: 0.0,
        saves_h_year: 0.0,
        crew: (0, 0),
    });
    land.carts.changed();
    // A household that moves with nothing in its name leaves no record.
    let other = sim
        .people()
        .households
        .iter()
        .map(|(_, h)| h)
        .find(|h| {
            h.id != keeper
                && h.id != neighbour
                && h.settlement != Some(market)
                && !h.members.is_empty()
                && !sim.land().carts.list.iter().any(|c| c.owner == h.id)
                && !sim.land().boats.list.iter().any(|c| c.owner == h.id)
        })
        .map(|h| h.id);
    if let Some(other) = other {
        sim.with_ctx_for_tests(|pop, ctx| pop.relocate_for_tests(ctx, other, market));
        assert!(
            sim.people().gone_households.is_empty(),
            "nothing left, nothing kept"
        );
    }
    // The keeper's household moves: its elder, who goes with it, is kept.
    let members = sim
        .people()
        .household(keeper)
        .expect("keeper")
        .members
        .clone();
    sim.with_ctx_for_tests(|pop, ctx| pop.relocate_for_tests(ctx, keeper, market));
    assert!(
        sim.people().household(keeper).is_none(),
        "a new household now"
    );
    let gone = sim.people().gone_households.clone();
    assert_eq!(gone.len(), 1, "{gone:?}");
    assert_eq!(gone[0].0, keeper);
    assert!(members.contains(&gone[0].1));
    let owner = |sim: &Sim| {
        sim.land()
            .carts
            .list
            .iter()
            .find(|c| c.id == cart)
            .map(|c| c.owner)
    };
    assert_eq!(
        owner(&sim),
        Some(keeper),
        "the cart stays in the old household's name"
    );
    let saves_alike = |sim: &mut Sim| {
        let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
        let saves =
            commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
        let saved = persist::save(sim, &saves, commons_persist::SaveKind::Manual, "transit")
            .expect("saves");
        let loaded = persist::load(&saved.path, content()).expect("loads");
        assert_eq!(
            loaded.people().gone_households,
            sim.people().gone_households
        );
        let (x, y) = (
            persist::encode_sections(sim),
            persist::encode_sections(&loaded),
        );
        for (p, q) in x.iter().zip(&y) {
            assert!(p.bytes == q.bytes, "section `{}` differs", p.tag);
        }
    };
    saves_alike(&mut sim);
    // The neighbour takes it up, and is told whose it had been.
    let timber = sim.rules().catalog.vehicles[cart_v].good;
    let enough = sim.rules().catalog.vehicles[cart_v].material_kg * 2.0;
    sim.people_mut_for_tests()
        .households
        .iter_mut()
        .map(|(_, h)| h)
        .find(|h| h.id == neighbour)
        .expect("neighbour")
        .stores[timber] = enough;
    sim.with_ctx_for_tests(|pop, ctx| {
        pop.begin_vehicle_for_tests(ctx, neighbour, cart_v, short[0], 30.0)
    });
    assert_eq!(owner(&sim), Some(neighbour), "taken up");
    let told: Vec<_> = sim
        .people()
        .chronicle
        .iter()
        .filter(|e| e.kind == civ_agents::ChronicleKind::VehicleTakenUp)
        .cloned()
        .collect();
    assert_eq!(told.len(), 1);
    let elder = sim.people().name_of(told[0].people[0]);
    let was = sim.people().name_of(gone[0].1);
    assert_eq!(
        told[0].name,
        format!(
            "{elder}'s household took up a handcart that had been {was}'s household's, which \
             nobody was left to take, made and standing at its door."
        )
    );
    // Nothing left in its name: a month forgets it.
    sim.with_ctx_for_tests(|pop, ctx| pop.gone_households_month_for_tests(ctx));
    assert!(sim.people().gone_households.is_empty());
    saves_alike(&mut sim);
}

/// A road line whose keeper moved away ends at midnight saying whose it had been (backlog D,
/// twenty-fourth item): the keeper's household, holding no coal for its motor cart's trip (so
/// the cart cannot go with it, the twenty-fifth item), moves to the other settlement, its motor
/// cart and line left in the old household's name; the history keeps its elder; at the next
/// midnight the line is taken off and told as "The motor cart's runs to …, which had been Bo's
/// household's, stopped: nobody was left who could run them."
#[test]
fn a_road_line_left_by_a_keeper_that_moved_ends_saying_whose_it_had_been() {
    let (mut sim, keeper, _, home, market, cells, _) = two_doors_and_a_hearth();
    posted(&mut sim, keeper, home, market, cells[0]);
    let coal = sim
        .rules()
        .catalog
        .goods
        .iter()
        .position(|g| g.id == "core:good/coal")
        .expect("coal");
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == keeper {
            x.stores[coal] = 0.0;
        }
    }
    let name = sim
        .land()
        .settlements
        .iter()
        .find(|s| s.id == market)
        .expect("the market")
        .name
        .clone();
    sim.with_ctx_for_tests(|pop, ctx| pop.relocate_for_tests(ctx, keeper, market));
    let gone = sim.people().gone_households.clone();
    assert_eq!(gone.len(), 1, "{gone:?}");
    assert_eq!(gone[0].0, keeper);
    let was = sim.people().name_of(gone[0].1);
    assert_eq!(sim.land().runs.roads.len(), 1, "until the midnight");
    let day = sim.now().day_index();
    sim.with_ctx_for_tests(|pop, ctx| pop.transit_reviews_for_tests(ctx, day));
    assert!(sim.land().runs.roads.is_empty());
    let t = road_entries(&sim).last().cloned().expect("told");
    assert!(t.people.is_empty());
    assert_eq!(
        t.name,
        format!(
            "The motor cart's runs to {name}, which had been {was}'s household's, stopped: \
             nobody was left who could run them."
        )
    );
}

/// A household that moves whole takes its carts along a way to its new home (backlog D,
/// twenty-fifth item): with its motor cart and a handcart made and standing at its old door and
/// one being made beside them, the motor cart (in this content able to climb any grade) stays
/// while its household holds too little coal for the trip, and with coal is driven to its new
/// door, burning coal; the handcart goes only where a fetch would bring it home (its new owner
/// and its new door together, or neither); the one being made stays in the old household's
/// name. It saves and loads exactly.
#[test]
fn a_household_that_moves_takes_its_carts_along_a_way_to_its_new_home() {
    let (mut sim, keeper, _, _, market, cells, _) = two_doors_and_a_hearth();
    let members = sim
        .people()
        .household(keeper)
        .expect("keeper")
        .members
        .clone();
    sim.with_ctx_for_tests(|pop, ctx| pop.relocate_for_tests(ctx, keeper, market));
    let (to, new_home) = sim
        .people()
        .households
        .iter()
        .map(|(_, h)| h)
        .find(|h| h.members.contains(&members[0]))
        .map(|h| (h.id, h.home))
        .expect("moved");
    let map = sim.map().clone();
    let door = civ_agents::population::cell_of(&map, new_home) as u32;
    let catalog = &sim.rules().catalog;
    let hand_v = catalog
        .vehicle_index("core:vehicle/handcart")
        .expect("a handcart");
    let motor_v = catalog
        .vehicle_index("core:vehicle/road_engine")
        .expect("a road engine");
    let coal = catalog
        .goods
        .iter()
        .position(|g| g.id == "core:good/coal")
        .expect("coal");
    let today = sim.now().day_index();
    let cart = |sim: &mut Sim, v: usize, labour: f32| {
        let id = sim.allocate_id_for_tests();
        let land = sim.land_mut_for_tests();
        land.carts.list.push(civ_land::boats::Boat {
            id,
            vehicle: v as u16,
            owner: keeper,
            at: cells[0],
            made: today,
            wear: 0.0,
            labour_h: labour,
            work_h: 0.0,
            saves_h_year: 0.0,
            crew: (0, 0),
        });
        land.carts.changed();
        id
    };
    let (motor, hand, half) = (
        cart(&mut sim, motor_v, 0.0),
        cart(&mut sim, hand_v, 0.0),
        cart(&mut sim, hand_v, 40.0),
    );
    let at = |sim: &Sim, id: PermanentId| {
        sim.land()
            .carts
            .list
            .iter()
            .find(|c| c.id == id)
            .map(|c| (c.owner, c.at))
            .expect("the cart")
    };
    let set_coal = |sim: &mut Sim, kg: f64| {
        for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
            if x.id == to {
                x.stores[coal] = kg;
            }
        }
    };
    let moves = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| pop.vehicles_move_for_tests(ctx, keeper, to))
    };
    // Too little coal for the trip: the motor cart stays.
    set_coal(&mut sim, 0.001);
    let first = moves(&mut sim);
    assert_eq!(at(&sim, motor), (keeper, cells[0]), "no coal for the trip");
    let (owner, place) = at(&sim, hand);
    assert_eq!(
        owner == to,
        place == door,
        "the handcart goes whole or not at all"
    );
    assert_eq!(first, usize::from(owner == to));
    // With coal, it is driven there, burning coal.
    set_coal(&mut sim, 5000.0);
    let went = moves(&mut sim);
    assert_eq!(went, 1, "the motor cart, and nothing twice");
    assert_eq!(at(&sim, motor), (to, door));
    let left = sim.people().household(to).expect("moved").stores[coal];
    assert!(left < 5000.0, "{left} kg of coal left");
    // The one being made stays.
    assert_eq!(at(&sim, half), (keeper, cells[0]));
    assert_eq!(moves(&mut sim), 0, "nothing more goes");
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
    let (x, y) = (
        persist::encode_sections(&sim),
        persist::encode_sections(&loaded),
    );
    for (p, q) in x.iter().zip(&y) {
        assert!(p.bytes == q.bytes, "section `{}` differs", p.tag);
    }
    // Moving itself takes it: a keeper with a motor cart at its door and coal moves, and its
    // cart is at its new door, its new household's.
    let (mut sim, keeper, home, market, cells) = {
        let (s, k, _, h, m, c, _) = two_doors_and_a_hearth();
        (s, k, h, m, c)
    };
    let motor = posted(&mut sim, keeper, home, market, cells[0]);
    let members = sim
        .people()
        .household(keeper)
        .expect("keeper")
        .members
        .clone();
    sim.with_ctx_for_tests(|pop, ctx| pop.relocate_for_tests(ctx, keeper, market));
    let (to, new_home) = sim
        .people()
        .households
        .iter()
        .map(|(_, h)| h)
        .find(|h| h.members.contains(&members[0]))
        .map(|h| (h.id, h.home))
        .expect("moved");
    let c = sim
        .land()
        .carts
        .list
        .iter()
        .find(|c| c.id == motor)
        .cloned()
        .expect("the cart");
    assert_eq!(
        (c.owner, c.at),
        (
            to,
            civ_agents::population::cell_of(sim.map(), new_home) as u32
        ),
        "it went with its household"
    );
    assert!(
        sim.people().gone_households.is_empty(),
        "nothing left in the old household's name"
    );
}

/// A keeper that moves reposts its runs from its new door through its own review (backlog D,
/// twenty-sixth item; the twenty-fifth item's "Not yet"): no new rule. Its motor cart goes with
/// it; its old line, whose keeper is no more, is taken off at midnight; at its review the new
/// household, keeping that cart and no line, posts nothing while its new settlement's people
/// run too few errands to any other, and once they run a dozen a year to the settlement it left,
/// posts runs there from its new door, told as posted.
#[test]
fn a_keeper_that_moves_reposts_its_runs_from_its_new_door() {
    use civ_agents::RoadStep;
    let (mut sim, keeper, _, home, market, cells, _) = two_doors_and_a_hearth();
    let motor = posted(&mut sim, keeper, home, market, cells[0]);
    let members = sim
        .people()
        .household(keeper)
        .expect("keeper")
        .members
        .clone();
    sim.with_ctx_for_tests(|pop, ctx| pop.relocate_for_tests(ctx, keeper, market));
    let (to, new_home) = sim
        .people()
        .households
        .iter()
        .map(|(_, h)| h)
        .find(|h| h.members.contains(&members[0]))
        .map(|h| (h.id, h.home))
        .expect("moved");
    let door = civ_agents::population::cell_of(sim.map(), new_home) as u32;
    let cart = sim
        .land()
        .carts
        .list
        .iter()
        .find(|c| c.id == motor)
        .cloned()
        .expect("the cart");
    assert_eq!((cart.owner, cart.at), (to, door), "its cart came with it");
    let day = sim.now().day_index();
    sim.with_ctx_for_tests(|pop, ctx| pop.transit_reviews_for_tests(ctx, day));
    assert!(
        sim.land().runs.roads.iter().all(|l| l.operator != keeper),
        "the old line is taken off"
    );
    // Too few errands from its new settlement: nothing posted.
    let review =
        |sim: &mut Sim| sim.with_ctx_for_tests(|pop, ctx| pop.review_road_line_for_tests(ctx, to));
    review(&mut sim);
    assert!(sim.land().runs.roads.iter().all(|l| l.operator != to));
    // A dozen errands a year and more to the settlement it left: posted from its new door.
    let year = day.div_euclid(civ_core::time::DAYS_PER_YEAR);
    sim.people_mut_for_tests()
        .contacts
        .years
        .entry((year, market, home))
        .or_default()
        .bought = 730;
    review(&mut sim);
    let l = sim
        .land()
        .runs
        .roads
        .iter()
        .find(|l| l.operator == to)
        .cloned()
        .expect("a road line posted");
    assert_eq!((l.cart, l.market, l.cells[0]), (motor, home, door));
    let told = road_entries(&sim);
    let t = told.last().expect("told");
    assert_eq!(t.number, f64::from(RoadStep::Posted as u8));
    assert_eq!(t.settlement, Some(market), "in its new settlement");
}

/// Coverage (backlog E; M11b, third part): at its yearly review a keeper sets its runs a day by
/// the rides taken since it last weighed them, as many as a run seats each (a rider and what they
/// carry at 100 kg), at most four, and its fare by them: a run's fuel at its own cost over the
/// riders a run carries; and it counts afresh from that day.
#[test]
fn a_keepers_review_sets_its_runs_and_fare_by_the_rides_taken() {
    let (mut sim, keeper, _, home, market, cells, _) = two_doors_and_a_hearth();
    posted(&mut sim, keeper, home, market, cells[0]);
    let first = sim.land().runs.roads[0].clone();
    // Posted with 730 errands a year: two riders a run expected, so a run's fuel cost is twice
    // the fare.
    let cost = f64::from(first.fare_kcal) * 2.0;
    let seats = {
        let v = sim
            .rules()
            .catalog
            .vehicle_index("core:vehicle/road_engine")
            .expect("a road engine");
        (sim.rules().catalog.vehicles[v].payload_kg / 100.0).floor()
    };
    let day = sim.now().day_index();
    let review = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| pop.review_road_line_for_tests(ctx, keeper))
    };
    // A hundred days of riders, a direction's a day one and a half runs' seats.
    {
        let l = &mut sim.land_mut_for_tests().runs.roads[0];
        l.since = day - 100;
        l.riders = (2.0 * 100.0 * seats * 1.5) as u32;
    }
    review(&mut sim);
    let l = sim.land().runs.roads[0].clone();
    assert_eq!(l.runs_a_day, 2, "a run and a half's riders take two runs");
    let per_run = seats * 1.5 / 2.0;
    assert!(
        (f64::from(l.fare_kcal) - cost / per_run).abs() < 1e-3 * (cost / per_run),
        "fare {} against {}",
        l.fare_kcal,
        cost / per_run
    );
    assert_eq!((l.riders, l.since), (0, day), "counted afresh");
    // Far more riders than four runs seat: four runs.
    {
        let l = &mut sim.land_mut_for_tests().runs.roads[0];
        l.since = day - 10;
        l.riders = (2.0 * 10.0 * seats * 9.0) as u32;
    }
    review(&mut sim);
    assert_eq!(sim.land().runs.roads[0].runs_a_day, 4, "at most four");
    // A few riders: one run, its fare the whole run's fuel shared over at least one rider.
    {
        let l = &mut sim.land_mut_for_tests().runs.roads[0];
        l.since = day - 100;
        l.riders = 20;
    }
    review(&mut sim);
    let l = sim.land().runs.roads[0].clone();
    assert_eq!(l.runs_a_day, 1);
    assert!(
        (f64::from(l.fare_kcal) - cost).abs() < 1e-3 * cost,
        "fare {} against {}",
        l.fare_kcal,
        cost
    );
}

/// Coverage (backlog E; M10b slice DX, second part; M11b, third part): the timetable trains and
/// road lines share leaves first at six in the morning and spreads a day's runs evenly over the
/// twelve hours from then (one at 06:00; two at 06:00 and 12:00; four three hours apart), at
/// least one run and at most four; a road line's runs back leave the far hearth a run's minutes,
/// rounded up, and a fifteen-minute turn after each run out.
#[test]
fn the_timetable_spreads_a_days_runs_over_twelve_hours_from_six() {
    let sim = world(1);
    let s = sim.land().settlements.clone();
    let mut l = posted_like(
        &sim,
        PermanentId::from_raw(7).expect("an id"),
        s[1].id,
        vec![1, 2, 3],
    );
    l.minutes = vec![0.0, 20.0, 41.5];
    let out = |l: &civ_land::trains::RoadLine| civ_agents::population::road_departures(l, false);
    let back = |l: &civ_land::trains::RoadLine| civ_agents::population::road_departures(l, true);
    l.runs_a_day = 1;
    assert_eq!(out(&l), vec![360]);
    assert_eq!(back(&l), vec![360 + 42 + 15]);
    l.runs_a_day = 2;
    assert_eq!(out(&l), vec![360, 720]);
    l.runs_a_day = 4;
    assert_eq!(out(&l), vec![360, 540, 720, 900]);
    assert_eq!(back(&l), vec![417, 597, 777, 957]);
    l.runs_a_day = 9;
    assert_eq!(out(&l).len(), 4, "at most four");
    l.runs_a_day = 0;
    assert_eq!(out(&l), vec![360], "at least one");
}

/// A household of the other settlement, its home set at that settlement's hearth, where the way
/// from the keeper's door ends: the household.
fn heir_at_the_hearth(sim: &mut Sim, market: PermanentId) -> PermanentId {
    let hearth = sim
        .land()
        .settlements
        .iter()
        .find(|s| s.id == market)
        .map(|s| s.hearth_m)
        .expect("the other settlement");
    let heir = sim
        .people()
        .households
        .iter()
        .map(|(_, h)| h)
        .filter(|h| h.settlement == Some(market) && !h.members.is_empty())
        .map(|h| h.id)
        .min()
        .expect("a household of the other settlement");
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == heir {
            x.home = hearth;
        }
    }
    heir
}

/// Backlog D, the ninth item's "Not yet": a motor cart reaches an heir in another settlement
/// where the heir holds the fuel to drive it home (the twenty-fifth item's rule), and someone of
/// the heir's fetches it by driving it, burning that fuel; an heir without the fuel does not take
/// it, and one whose fuel is gone does not fetch it by driving.
#[test]
fn an_heir_far_off_takes_a_motor_cart_it_holds_the_fuel_for_and_fetches_it_by_driving() {
    let coal_of = |sim: &Sim| {
        sim.rules()
            .catalog
            .goods
            .iter()
            .position(|g| g.id == "core:good/coal")
            .expect("coal")
    };
    let set_coal = |sim: &mut Sim, h: PermanentId, kg: f64| {
        let coal = coal_of(sim);
        for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
            if x.id == h {
                x.stores[coal] = kg;
            }
        }
    };
    let owner_at = |sim: &Sim, id: PermanentId| {
        sim.land()
            .carts
            .list
            .iter()
            .find(|c| c.id == id)
            .map(|c| (c.owner, c.at))
            .expect("the cart")
    };
    let scene = || {
        let (mut sim, keeper, _, home, market, cells, _) = two_doors_and_a_hearth();
        let cart = posted(&mut sim, keeper, home, market, cells[0]);
        let heir = heir_at_the_hearth(&mut sim, market);
        (sim, keeper, heir, cart, cells)
    };
    // The heir holds no coal, and the keeper's passes with its store: so the keeper's is spent
    // first. Without the fuel to drive it home, the motor cart stays, the keeper's.
    let (mut sim, keeper, heir, cart, cells) = scene();
    set_coal(&mut sim, keeper, 0.0);
    set_coal(&mut sim, heir, 0.0);
    sim.with_ctx_for_tests(|pop, ctx| pop.merge_household_for_tests(ctx, keeper, heir));
    assert_eq!(
        owner_at(&sim, cart),
        (keeper, cells[0]),
        "no coal to drive it home"
    );
    // Holding coal for the drive, it is the heir's, standing where it was.
    let (mut sim, keeper, heir, cart, cells) = scene();
    set_coal(&mut sim, keeper, 0.0);
    set_coal(&mut sim, heir, 5000.0);
    sim.with_ctx_for_tests(|pop, ctx| pop.merge_household_for_tests(ctx, keeper, heir));
    assert_eq!(
        owner_at(&sim, cart),
        (heir, cells[0]),
        "the heir's, where it stood"
    );
    // Someone of the heir's would fetch it by driving: its fuel weighed at their own cost.
    let grown = sim.rules().people.family.independent_age;
    let now = sim.now();
    let fetcher = sim
        .people()
        .household(heir)
        .expect("the heir")
        .members
        .iter()
        .copied()
        .find(|&m| {
            sim.people()
                .person(m)
                .is_some_and(|p| p.age_years(now) >= grown)
        })
        .expect("a grown member");
    let facts = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| pop.vehicle_fetch_for_tests(ctx, fetcher))
    };
    let (id, trip_min, worth_h, fuel_h) = facts(&mut sim).expect("it would fetch the motor cart");
    assert_eq!(id, cart);
    assert!(
        trip_min > 0.0 && worth_h > 0.0 && fuel_h > 0.0,
        "{trip_min} {worth_h} {fuel_h}"
    );
    // Its coal gone, it would not fetch it by driving.
    set_coal(&mut sim, heir, 0.0);
    assert!(facts(&mut sim).is_none(), "no coal to drive it");
    set_coal(&mut sim, heir, 5000.0);
    // Lived on, someone of the heir's walks to it and drives it home, burning coal.
    let fetch = sim
        .rules()
        .catalog
        .activities
        .iter()
        .position(|x| x.behavior == civ_agents::params::Behavior::Fetch)
        .expect("fetching") as u16;
    let door = |sim: &Sim| {
        let home = sim.people().household(heir).expect("the heir").home;
        civ_agents::population::cell_of(sim.map(), home) as u32
    };
    let mut drove = false;
    for _ in 0..4 * 24 * 6 {
        sim.advance_minutes(10).expect("advances");
        drove |= sim.people().people.iter().any(|(_, p)| {
            p.household == heir
                && p.act.def == fetch
                && p.act.target == civ_agents::person::Target::Boat(cart)
                && matches!(
                    p.act.steps.get(usize::from(p.act.step)),
                    Some(civ_agents::person::Step::Haul { .. })
                )
                && p.trip.is_some()
        });
        if owner_at(&sim, cart).1 == door(&sim) {
            break;
        }
    }
    assert!(drove, "someone of the heir's drove it");
    assert_eq!(
        owner_at(&sim, cart),
        (heir, door(&sim)),
        "at the heir's door"
    );
    let left = sim.people().household(heir).expect("the heir").stores[coal_of(&sim)];
    assert!(left < 5000.0, "coal burned on the drive: {left} kg left");
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
    let (x, y) = (
        persist::encode_sections(&sim),
        persist::encode_sections(&loaded),
    );
    for (p, q) in x.iter().zip(&y) {
        assert!(p.bytes == q.bytes, "section `{}` differs", p.tag);
    }
}

/// The fourteenth coverage pass (§8 21; the ninth item's motor cart, 712b326): an heir far off
/// holding exactly the fuel its drive home burns takes the motor cart, and one holding a gram
/// less does not; two motor carts and fuel for one: one passes, as moving takes one (the fuel is
/// reckoned across them); and fuel gone between the choice to fetch it and the drive: the drive
/// does not set out, nothing is burned, and the cart stands where it was.
#[test]
fn a_motor_cart_passes_far_off_on_exactly_its_fuel_one_at_a_time_and_is_not_driven_once_it_is_gone()
{
    let coal_of = |sim: &Sim| {
        sim.rules()
            .catalog
            .goods
            .iter()
            .position(|g| g.id == "core:good/coal")
            .expect("coal")
    };
    let set_coal = |sim: &mut Sim, h: PermanentId, kg: f64| {
        let coal = coal_of(sim);
        for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
            if x.id == h {
                x.stores[coal] = kg;
            }
        }
    };
    let owner_at = |sim: &Sim, id: PermanentId| {
        sim.land()
            .carts
            .list
            .iter()
            .find(|c| c.id == id)
            .map(|c| (c.owner, c.at))
            .expect("the cart")
    };
    let scene = || {
        let (mut sim, keeper, _, home, market, cells, _) = two_doors_and_a_hearth();
        let cart = posted(&mut sim, keeper, home, market, cells[0]);
        let heir = heir_at_the_hearth(&mut sim, market);
        set_coal(&mut sim, keeper, 0.0);
        (sim, keeper, heir, cart, cells)
    };
    let need = |sim: &mut Sim, heir: PermanentId, cart: PermanentId| {
        let (_, f, kg) = sim
            .with_ctx_for_tests(|pop, ctx| pop.drive_home_for_tests(ctx, heir, cart))
            .expect("a drive home");
        assert_eq!(f, coal_of(sim));
        kg
    };
    // Exactly the fuel: it passes.
    let (mut sim, keeper, heir, cart, cells) = scene();
    let kg = need(&mut sim, heir, cart);
    set_coal(&mut sim, heir, kg);
    sim.with_ctx_for_tests(|pop, ctx| pop.merge_household_for_tests(ctx, keeper, heir));
    assert_eq!(
        owner_at(&sim, cart),
        (heir, cells[0]),
        "exactly its fuel: {kg} kg"
    );
    // A gram short: it stays.
    let (mut sim, keeper, heir, cart, cells) = scene();
    let kg = need(&mut sim, heir, cart);
    set_coal(&mut sim, heir, kg - 1e-3);
    sim.with_ctx_for_tests(|pop, ctx| pop.merge_household_for_tests(ctx, keeper, heir));
    assert_eq!(
        owner_at(&sim, cart),
        (keeper, cells[0]),
        "a gram short of {kg} kg"
    );
    // Two motor carts where it stood, and fuel for one drive: one passes.
    let (mut sim, keeper, heir, cart, _) = scene();
    let second = {
        let c = sim
            .land()
            .carts
            .list
            .iter()
            .find(|c| c.id == cart)
            .cloned()
            .expect("the cart");
        let id = sim.allocate_id_for_tests();
        let land = sim.land_mut_for_tests();
        land.carts.list.push(civ_land::boats::Boat { id, ..c });
        land.carts.changed();
        id
    };
    let kg = need(&mut sim, heir, cart);
    set_coal(&mut sim, heir, 1.5 * kg);
    sim.with_ctx_for_tests(|pop, ctx| pop.merge_household_for_tests(ctx, keeper, heir));
    let owners = [owner_at(&sim, cart).0, owner_at(&sim, second).0];
    assert_eq!(
        owners.iter().filter(|&&o| o == heir).count(),
        1,
        "fuel for one drive of {kg} kg: {owners:?}"
    );
    // Chosen to be fetched, its fuel gone before the drive: it does not set out.
    let (mut sim, keeper, heir, cart, cells) = scene();
    set_coal(&mut sim, heir, 5000.0);
    sim.with_ctx_for_tests(|pop, ctx| pop.merge_household_for_tests(ctx, keeper, heir));
    assert_eq!(owner_at(&sim, cart), (heir, cells[0]));
    let fetch = sim
        .rules()
        .catalog
        .activities
        .iter()
        .position(|x| x.behavior == civ_agents::params::Behavior::Fetch)
        .expect("fetching") as u16;
    let fetching = |sim: &Sim| {
        sim.people().people.iter().find_map(|(_, p)| {
            (p.household == heir
                && p.act.def == fetch
                && p.act.target == civ_agents::person::Target::Boat(cart))
            .then_some(p.id)
        })
    };
    let mut who = None;
    for _ in 0..4 * 24 * 6 {
        sim.advance_minutes(10).expect("advances");
        who = fetching(&sim);
        if who.is_some() {
            break;
        }
    }
    let who = who.expect("someone of the heir's sets out to fetch it");
    set_coal(&mut sim, heir, 0.0);
    for _ in 0..24 * 6 {
        sim.advance_minutes(10).expect("advances");
        if sim
            .people()
            .person(who)
            .is_some_and(|p| p.act.target != civ_agents::person::Target::Boat(cart))
        {
            break;
        }
    }
    assert_eq!(
        owner_at(&sim, cart),
        (heir, cells[0]),
        "not driven without its fuel"
    );
    assert_eq!(
        sim.people().household(heir).expect("heir").stores[coal_of(&sim)],
        0.0,
        "nothing burned"
    );
}
