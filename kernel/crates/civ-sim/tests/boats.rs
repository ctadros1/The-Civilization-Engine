//! The river for boats and the dugout as a thing (M8b slice CO, first part; ADR-0029 §2, §5, §7):
//! a dugout goes down the river valley's trunk faster than up it, only where today's stage is
//! above its draft, by the quickest way the oracle also finds; a boat is a record that belongs to
//! a household, stands at a bank, ages by the day, is gone once its life is spent, and saves and
//! goes on alike. A household that is no more passes its boats and carts to the household its
//! people join in the same settlement, and its loads in transit wherever that is (backlog D); and
//! there, too, the works it keeps on the land, finished, and its platforms wherever it goes. An
//! heir in another settlement takes its made boats and carts where they stand, and fetches them
//! home.

use std::path::Path;
use std::sync::OnceLock;

use civ_content::ContentRegistry;
use civ_land::boating::{Hull, RiverGraph, going, velocity};
use civ_land::boats::Boat;
use civ_sim::{NewWorld, Sim, persist};

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
            name: "Dugouts".to_owned(),
            seed,
            preset_id: PRESET.to_owned(),
            size_cells: 768,
            band_size: 30,
            neighbours: Vec::new(),
            neighbours_known: true,
            regime_id: String::new(),
        },
        content(),
        [11; 16],
    )
    .expect("generates")
}

/// The core dugout's catalog index and hull.
fn dugout(sim: &Sim) -> (usize, Hull) {
    let i = sim
        .rules()
        .catalog
        .vehicle_index("core:vehicle/dugout")
        .expect("the core has a dugout");
    let v = &sim.rules().catalog.vehicles[i];
    (i, v.hull())
}

#[test]
fn the_core_dugout_is_a_boat_made_with_an_axe_by_wood_shaping() {
    let sim = village(1);
    let (i, hull) = dugout(&sim);
    let catalog = &sim.rules().catalog;
    let v = &catalog.vehicles[i];
    assert_eq!(v.mode, civ_agents::params::VehicleMode::Boat);
    assert_eq!(v.payload_kg, 400.0);
    assert_eq!(hull.draft_m, 0.3);
    assert_eq!(
        v.technique.map(|t| catalog.techniques[t].id.as_str()),
        Some("core:technique/wood_shaping")
    );
    assert_eq!(
        v.tool.map(|g| catalog.goods[g].id.as_str()),
        Some("core:good/axe")
    );
    assert!((v.life_days() - 12.0 * 365.25).abs() < 1e-9);
}

#[test]
fn a_dugout_goes_down_the_trunk_faster_than_up_it_and_only_where_it_is_deep_enough() {
    let sim = village(1);
    let (_, hull) = dugout(&sim);
    let map = sim.map();
    let land = sim.land();
    let graph = RiverGraph::new(map);
    assert!(!graph.is_empty(), "the river valley has a river");
    let hyd = &land.floodplain.reaches;
    let q = &land.reach_discharges;
    // The trunk: the reach with the most water today.
    let trunk = (0..hyd.len())
        .max_by(|&a, &b| q[a].total_cmp(&q[b]))
        .expect("a reach");
    let cells: Vec<u32> = map.reaches[trunk]
        .cells
        .iter()
        .copied()
        .filter(|&c| graph.is_river(c))
        .collect();
    assert!(
        cells.len() > 10,
        "the trunk is long enough: {}",
        cells.len()
    );
    let g = going(&hyd[trunk], f64::from(q[trunk]), hull);
    assert!(
        g.stage_m > hull.draft_m,
        "the trunk floats a dugout today: stage {} m",
        g.stage_m
    );
    let v = velocity(&hyd[trunk], f64::from(q[trunk]));
    assert!(v > 0.0 && (g.down_m_s.expect("downstream") - (hull.paddle_m_s + v)).abs() < 1e-12);
    let (top, bottom) = (cells[0], *cells.last().expect("cells"));
    let down = graph
        .route(hyd, q, hull, &[top], &[bottom])
        .expect("down the trunk");
    assert_eq!(down.cells.first(), Some(&top));
    assert_eq!(down.cells.last(), Some(&bottom));
    let oracle = |from: u32, to: u32| graph.oracle_seconds(hyd, q, hull, &[from], &[to]);
    assert!((down.seconds - oracle(top, bottom).expect("oracle")).abs() < 1e-6);
    match graph.route(hyd, q, hull, &[bottom], &[top]) {
        Some(up) => {
            assert!(
                up.seconds > down.seconds,
                "against the current is slower: {} s up, {} s down",
                up.seconds,
                down.seconds
            );
            assert!((up.seconds - oracle(bottom, top).expect("oracle")).abs() < 1e-6);
            assert!((up.metres - down.metres).abs() < 1e-6);
        }
        None => assert!(g.up_m_s.is_none(), "upstream closed only by the current"),
    }
    // The quickest way between any two cells of reaches deep enough today is the oracle's.
    let all: Vec<u32> = map
        .reaches
        .iter()
        .filter(|r| {
            let i = r.id as usize;
            going(&hyd[i], f64::from(q[i]), hull).down_m_s.is_some()
        })
        .flat_map(|r| r.cells.iter().copied())
        .filter(|&c| graph.is_river(c))
        .collect();
    assert!(
        all.len() > cells.len(),
        "more than the trunk floats a dugout"
    );
    let mut checked = 0;
    for k in 0..40 {
        let (a, b) = (
            all[(k * 7919) % all.len()],
            all[(k * 104_729 + 13) % all.len()],
        );
        let route = graph.route(hyd, q, hull, &[a], &[b]).map(|r| r.seconds);
        let truth = oracle(a, b);
        match (route, truth) {
            (Some(r), Some(t)) => {
                assert!((r - t).abs() < 1e-6, "{a} to {b}: {r} s against {t} s");
                checked += 1;
            }
            (None, None) => {}
            other => panic!("{a} to {b}: route and oracle disagree: {other:?}"),
        }
    }
    assert!(checked >= 5, "enough pairs joined: {checked}");
    // Low water closes the river; a hull that needs more water than any reach has goes nowhere.
    let low: Vec<f32> = q.iter().map(|x| x * 1e-4).collect();
    assert!(graph.route(hyd, &low, hull, &[top], &[bottom]).is_none());
    let deep = Hull {
        draft_m: 50.0,
        ..hull
    };
    assert!(graph.route(hyd, q, deep, &[top], &[bottom]).is_none());
}

/// Saves `sim`, loads it, and checks every section is the same and that the two go on alike for
/// `minutes`.
fn saves_and_goes_on_alike(sim: &mut Sim, minutes: i64) {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "boats").expect("saves");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
    let same = |a: &Sim, b: &Sim| {
        let (a, b) = (persist::encode_sections(a), persist::encode_sections(b));
        assert_eq!(a.len(), b.len());
        for (x, y) in a.iter().zip(&b) {
            assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
        }
    };
    same(sim, &loaded);
    sim.advance_minutes(minutes).expect("advances");
    loaded.advance_minutes(minutes).expect("advances");
    same(sim, &loaded);
}

#[test]
fn a_boat_belongs_to_a_household_stands_at_a_bank_ages_and_is_gone_when_spent() {
    let mut sim = village(1);
    let (vehicle, _) = dugout(&sim);
    let graph = RiverGraph::new(sim.map());
    // The bank nearest the first household's home.
    let (owner, home) = sim
        .people()
        .households
        .iter()
        .next()
        .map(|(_, h)| (h.id, h.home))
        .expect("a household");
    let map = sim.map();
    let w = map.width as usize;
    let cell_m = map.cell_size_m;
    let d = |c: u32| {
        let x = ((c as usize % w) as f32 + 0.5) * cell_m;
        let y = ((c as usize / w) as f32 + 0.5) * cell_m;
        (x - home.0).hypot(y - home.1)
    };
    let bank = (0..(map.width * map.height))
        .filter(|&c| !graph.boarding(map, c).is_empty())
        .min_by(|&a, &b| d(a).total_cmp(&d(b)).then(a.cmp(&b)))
        .expect("a bank");
    let id = sim.allocate_id_for_tests();
    let made = sim.now().day_index();
    let land = sim.land_mut_for_tests();
    land.boats.list.push(Boat {
        id,
        vehicle: vehicle as u16,
        owner,
        at: bank,
        made,
        wear: 0.0,
        labour_h: 0.0,
        work_h: 0.0,
        saves_h_year: 0.0,
        crew: (0, 0),
    });
    land.boats.changed();
    assert_eq!(sim.land().boats.of(owner).count(), 1);
    // A day spends a day of its twelve years.
    sim.advance_minutes(DAY).expect("lives");
    let wear = sim.land().boats.list[0].wear;
    let day = 1.0 / (12.0 * 365.25);
    assert!((f64::from(wear) - day).abs() < 1e-6, "worn {wear}");
    assert!(sim.land().boats.problems().is_empty());
    saves_and_goes_on_alike(&mut sim, DAY);
    // Near the end of its life, it is gone within two days.
    sim.land_mut_for_tests().boats.list[0].wear = 1.0 - day as f32;
    let revision = sim.land().boats.revision();
    sim.advance_minutes(2 * DAY).expect("lives");
    assert!(sim.land().boats.list.is_empty(), "past use, it is gone");
    assert_ne!(sim.land().boats.revision(), revision);
}

/// A made vehicle of catalog index `vehicle`, belonging to `owner` and standing at `at`.
fn vehicle(
    id: civ_core::PermanentId,
    vehicle: usize,
    owner: civ_core::PermanentId,
    at: u32,
) -> Boat {
    Boat {
        id,
        vehicle: vehicle as u16,
        owner,
        at,
        made: 0,
        wear: 0.0,
        labour_h: 0.0,
        work_h: 0.0,
        saves_h_year: 0.0,
        crew: (0, 0),
    }
}

#[test]
fn a_household_no_more_passes_its_boats_carts_and_loads_to_the_household_its_people_join() {
    // Two settlements, so that a household can join one in the other.
    let mut sim = Sim::create_for_tests(
        &NewWorld {
            name: "Heirs".to_owned(),
            seed: 1,
            preset_id: PRESET.to_owned(),
            size_cells: 768,
            band_size: 30,
            neighbours: vec![0],
            neighbours_known: true,
            regime_id: String::new(),
        },
        content(),
        [11; 16],
    )
    .expect("generates");
    assert_eq!(sim.land().settlements.len(), 2);
    let (boat_v, _) = dugout(&sim);
    let cart_v = sim
        .rules()
        .catalog
        .vehicle_index("core:vehicle/handcart")
        .expect("a handcart");
    let map = sim.map().clone();
    let door = |home: (f32, f32)| civ_agents::population::cell_of(&map, home) as u32;
    // Three households: two of one settlement and one of the other.
    let households: Vec<(
        civ_core::PermanentId,
        Option<civ_core::PermanentId>,
        (f32, f32),
    )> = sim
        .people()
        .households
        .iter()
        .map(|(_, x)| (x.id, x.settlement, x.home))
        .collect();
    let (a, settlement, a_home) = households[0];
    let (b, _, b_home) = *households
        .iter()
        .find(|h| h.0 != a && h.1 == settlement)
        .expect("a second household of the settlement");
    let (c, _, _) = *households
        .iter()
        .find(|h| h.1.is_some() && h.1 != settlement)
        .expect("a household of the other settlement");
    let graph = RiverGraph::new(&map);
    let bank = (0..(map.width * map.height))
        .find(|&c| !graph.boarding(&map, c).is_empty())
        .expect("a bank");
    let elsewhere = (door(a_home) + 1) % (map.width * map.height);
    let ids: Vec<civ_core::PermanentId> = (0..5).map(|_| sim.allocate_id_for_tests()).collect();
    let land = sim.land_mut_for_tests();
    // `a` has a made boat at a bank and one being made, a cart at its door and one out on the
    // ways; `b` has a cart of its own at its door.
    land.boats.list.push(vehicle(ids[0], boat_v, a, bank));
    land.boats.list.push(Boat {
        labour_h: 100.0,
        work_h: 40.0,
        ..vehicle(ids[1], boat_v, a, bank)
    });
    land.boats.changed();
    land.carts
        .list
        .push(vehicle(ids[2], cart_v, a, door(a_home)));
    land.carts.list.push(vehicle(ids[3], cart_v, a, elsewhere));
    land.carts
        .list
        .push(vehicle(ids[4], cart_v, b, door(b_home)));
    land.carts.changed();
    // A load `a` bought, in transit in its boat.
    let load = sim.allocate_id_for_tests();
    let goods = sim.rules().catalog.goods.len();
    let now = sim.now();
    let carrier = sim.people().household(a).expect("a").members[0];
    let other = sim.land().settlements[1].id;
    sim.people_mut_for_tests()
        .consignments
        .push(civ_agents::consignments::Consignment::new(
            load,
            (a, carrier, ids[0]),
            (other, settlement.expect("a settlement")),
            now,
            goods,
        ));
    let (boats_rev, carts_rev) = (sim.land().boats.revision(), sim.land().carts.revision());
    // `a`'s people join `b`, of the same settlement: everything passes to `b`.
    sim.with_ctx_for_tests(|pop, ctx| pop.merge_household_for_tests(ctx, a, b));
    assert!(sim.people().household(a).is_none(), "`a` is no more");
    let land = sim.land();
    assert!(land.boats.list.iter().all(|x| x.owner == b));
    assert_eq!(land.boats.made_of(b).count(), 1);
    assert_eq!(land.boats.of(b).count(), 2, "and the one being made");
    let unmade = &land.boats.list[1];
    assert_eq!(
        (unmade.at, unmade.work_h, unmade.labour_h),
        (bank, 40.0, 100.0)
    );
    assert_eq!(
        land.boats.list[0].at, bank,
        "a boat stays drawn up where it is"
    );
    assert!(land.carts.list.iter().all(|x| x.owner == b));
    let at: Vec<u32> = land.carts.list.iter().map(|x| x.at).collect();
    assert_eq!(
        at,
        vec![door(b_home), elsewhere, door(b_home)],
        "the cart at `a`'s door is wheeled to `b`'s; the one out stays out"
    );
    assert_ne!(land.boats.revision(), boats_rev);
    assert_ne!(land.carts.revision(), carts_rev);
    assert!(land.boats.problems().is_empty() && land.carts.problems().is_empty());
    assert_eq!(
        sim.people().consignments[0].owner,
        b,
        "the load is `b`'s now"
    );
    // `b`'s people join `c`, of the other settlement: the load passes, as `b`'s store does, but
    // the boats and carts stand where they are, `b`'s still, which nobody now reaches.
    let (boats_rev, carts_rev) = (sim.land().boats.revision(), sim.land().carts.revision());
    let left: Vec<(civ_core::PermanentId, u32)> = sim
        .land()
        .boats
        .list
        .iter()
        .chain(&sim.land().carts.list)
        .map(|x| (x.owner, x.at))
        .collect();
    sim.with_ctx_for_tests(|pop, ctx| pop.merge_household_for_tests(ctx, b, c));
    let now_left: Vec<(civ_core::PermanentId, u32)> = sim
        .land()
        .boats
        .list
        .iter()
        .chain(&sim.land().carts.list)
        .map(|x| (x.owner, x.at))
        .collect();
    assert_eq!(now_left, left);
    assert!(now_left.iter().all(|&(o, _)| o == b));
    assert_eq!(
        sim.land().boats.of(c).count() + sim.land().carts.of(c).count(),
        0
    );
    assert_eq!(
        (sim.land().boats.revision(), sim.land().carts.revision()),
        (boats_rev, carts_rev)
    );
    assert_eq!(
        sim.people().consignments[0].owner,
        c,
        "the load is `c`'s now"
    );
    saves_and_goes_on_alike(&mut sim, DAY);
}

/// An earthwork of `household` of kind `kind`, `done` of the way, its id `id`.
fn earthwork(
    id: civ_core::PermanentId,
    kind: civ_land::earth::EarthKind,
    household: civ_core::PermanentId,
    done: f32,
) -> civ_land::earth::Earthwork {
    civ_land::earth::Earthwork {
        id,
        kind,
        rect: civ_land::RectCm {
            x: 1_000,
            y: 1_000,
            w: 400,
            h: 400,
        },
        level_cm: 0,
        side_run_cm: 150,
        plot: None,
        deposit: None,
        heap: None,
        household,
        cut_m3: 1.0,
        done,
        version: 0,
        begun: civ_core::SimTime::from_minutes(0),
    }
}

/// A one-segment way along `cells` serving `modes`, kept by `keeper`, built `built_m` of a
/// 2 m corridor.
fn kept_way(
    id: civ_core::PermanentId,
    cells: Vec<u32>,
    modes: u8,
    keeper: civ_land::ways::Keeper,
    built_m: f32,
) -> civ_land::ways::Way {
    use civ_land::ways::{Rank, Segment, Surface, Way};
    Way {
        id,
        begun: 0,
        segments: vec![Segment {
            cells,
            corridor_m: 2.0,
            built_m,
            usable_m: built_m,
            surface: Surface::Cleared,
            keeper,
            kept: 0,
        }],
        modes,
        trips: 0.0,
        trips_day: 0,
        rank: Rank::Lane,
        track: None,
    }
}

#[test]
fn a_household_no_more_passes_the_works_it_keeps_to_the_household_its_people_join_nearby() {
    use civ_land::crossings::{Crossing, CrossingOwner, CrossingState};
    use civ_land::earth::EarthKind;
    use civ_land::ways::{CART, Keeper, RAIL, WALK};
    let mut sim = Sim::create_for_tests(
        &NewWorld {
            name: "Heirs".to_owned(),
            seed: 1,
            preset_id: PRESET.to_owned(),
            size_cells: 768,
            band_size: 30,
            neighbours: vec![0],
            neighbours_known: true,
            regime_id: String::new(),
        },
        content(),
        [11; 16],
    )
    .expect("generates");
    assert_eq!(sim.land().settlements.len(), 2);
    let households: Vec<(civ_core::PermanentId, Option<civ_core::PermanentId>)> = sim
        .people()
        .households
        .iter()
        .map(|(_, x)| (x.id, x.settlement))
        .collect();
    let (a, settlement) = households[0];
    let (b, _) = *households
        .iter()
        .find(|h| h.0 != a && h.1 == settlement)
        .expect("a second household of the settlement");
    let (c, _) = *households
        .iter()
        .find(|h| h.1.is_some() && h.1 != settlement)
        .expect("a household of the other settlement");
    let system = sim
        .rules()
        .catalog
        .bridge_index("core:bridge/log_beam")
        .expect("a log footbridge") as u16;
    let ids: Vec<civ_core::PermanentId> = (0..16).map(|_| sim.allocate_id_for_tests()).collect();
    let now = sim.now();
    let crossing = |id, state| Crossing {
        id,
        system,
        banks: [10, 12],
        cells: vec![11],
        span_m: 4.0,
        members: 2,
        diameter_cm: 30.0,
        quality: 1.0,
        loss: 0.0,
        owner: CrossingOwner::Household(a),
        labour_h: 80.0,
        skill_h: 0.0,
        begun: now,
        state,
        shares: Vec::new(),
        crew: (0, 0),
    };
    let wheel = |id| civ_land::power::Installation {
        id,
        reach: 0,
        at: [10.0, 10.0],
        head_m: 2.0,
        race_m3s: 0.5,
        efficiency: 0.3,
        rated_kw: 3.0,
        owner: a,
        building: None,
    };
    let land = sim.land_mut_for_tests();
    // `a` keeps a footbridge that stands, one fallen and one being built (20 of 80 hours).
    land.crossings
        .list
        .push(crossing(ids[0], CrossingState::Open { since: 0 }));
    land.crossings.list.push(crossing(
        ids[1],
        CrossingState::Failed {
            day: 0,
            why: civ_land::crossings::Collapse::OwnWeight,
        },
    ));
    land.crossings
        .list
        .push(crossing(ids[2], CrossingState::Building { work_h: 20.0 }));
    // A way it laid as wide as it is to be, a passing place it is still widening, its line of
    // track, and a way `b` keeps.
    land.ways.add(kept_way(
        ids[3],
        vec![100, 101, 102],
        WALK | CART,
        Keeper::Household(a),
        2.0,
    ));
    land.ways.add(kept_way(
        ids[4],
        vec![200],
        WALK | CART,
        Keeper::Household(a),
        1.0,
    ));
    land.ways.add(kept_way(
        ids[5],
        vec![300, 301],
        RAIL,
        Keeper::Household(a),
        2.0,
    ));
    land.ways.add(kept_way(
        ids[6],
        vec![400],
        WALK | CART,
        Keeper::Household(b),
        2.0,
    ));
    // Its earthworks: a platform being levelled, a pit and its heap, a bank dug and one half
    // dug; an engine at the pit; a wheel without a headrace and one whose race is half dug.
    land.earthworks
        .push(earthwork(ids[7], EarthKind::Platform, a, 0.5));
    land.earthworks
        .push(earthwork(ids[8], EarthKind::Pit, a, 0.2));
    land.earthworks
        .push(earthwork(ids[9], EarthKind::Spoil, a, 0.2));
    land.earthworks
        .push(earthwork(ids[10], EarthKind::Bank, a, 1.0));
    land.earthworks
        .push(earthwork(ids[11], EarthKind::Bank, a, 0.4));
    land.engines.push(civ_land::power::Engine {
        id: ids[12],
        pit: ids[8],
        owner: a,
        rated_kw: 5.0,
        efficiency: 0.01,
    });
    land.installations.push(wheel(ids[13]));
    land.installations.push(wheel(ids[14]));
    land.earthworks
        .push(earthwork(ids[14], EarthKind::Ditch, a, 0.5));
    let ways_rev = sim.land().ways.revision();
    let crossings_rev = sim.land().crossings.revision();
    // `a`'s people join `b`, of the same settlement: what `a` finished passes to `b`.
    sim.with_ctx_for_tests(|pop, ctx| pop.merge_household_for_tests(ctx, a, b));
    assert!(sim.people().household(a).is_none(), "`a` is no more");
    let land = sim.land();
    let owners: Vec<CrossingOwner> = land.crossings.list.iter().map(|x| x.owner).collect();
    assert_eq!(
        owners,
        vec![
            CrossingOwner::Household(b),
            CrossingOwner::Household(b),
            CrossingOwner::Household(a),
        ],
        "the footbridge being built is left for whoever takes it over"
    );
    assert_eq!(
        land.crossings.revision(),
        crossings_rev,
        "routes are not changed by whose a footbridge is"
    );
    let keepers: Vec<Keeper> = land
        .ways
        .list
        .iter()
        .map(|w| w.segments[0].keeper)
        .collect();
    assert_eq!(
        keepers,
        vec![
            Keeper::Household(b),
            Keeper::Household(a),
            Keeper::Household(a),
            Keeper::Household(b),
        ],
        "the way laid passes; the passing place being widened and the line do not"
    );
    assert_ne!(land.ways.revision(), ways_rev);
    let held: Vec<civ_core::PermanentId> = land.earthworks.iter().map(|e| e.household).collect();
    assert_eq!(
        held,
        vec![b, b, b, b, a, a],
        "all but what is still being dug"
    );
    assert_eq!(land.engines[0].owner, b);
    let wheels: Vec<civ_core::PermanentId> = land.installations.iter().map(|x| x.owner).collect();
    assert_eq!(wheels, vec![b, a], "the wheel whose race is half dug stays");
    // `b`'s people join `c`, of the other settlement: only the platform passes, with the plots
    // it levels; the rest stands as it was, `b`'s still, which nobody now reaches.
    let ways_rev = sim.land().ways.revision();
    sim.with_ctx_for_tests(|pop, ctx| pop.merge_household_for_tests(ctx, b, c));
    let land = sim.land();
    assert_eq!(land.crossings.list[0].owner, CrossingOwner::Household(b));
    assert_eq!(land.ways.list[0].segments[0].keeper, Keeper::Household(b));
    assert_eq!(land.ways.list[3].segments[0].keeper, Keeper::Household(b));
    assert_eq!(land.ways.revision(), ways_rev);
    let held: Vec<civ_core::PermanentId> = land.earthworks.iter().map(|e| e.household).collect();
    assert_eq!(held, vec![c, b, b, b, a, a]);
    assert_eq!(land.engines[0].owner, b);
    assert_eq!(land.installations[0].owner, b);
}

#[test]
fn an_heir_in_another_settlement_fetches_the_boat_and_cart_it_took_by_paddling_and_pulling_them_home()
 {
    // Two settlements on the water, so that a household can join one in the other.
    let mut sim = Sim::create_for_tests(
        &NewWorld {
            name: "Heirs".to_owned(),
            seed: 34,
            preset_id: PRESET.to_owned(),
            size_cells: 768,
            band_size: 30,
            neighbours: vec![0],
            neighbours_known: true,
            regime_id: String::new(),
        },
        content(),
        [11; 16],
    )
    .expect("generates");
    let (boat_v, hull) = dugout(&sim);
    let cart_v = sim
        .rules()
        .catalog
        .vehicle_index("core:vehicle/handcart")
        .expect("a handcart");
    let map = sim.map().clone();
    let door = |home: (f32, f32)| civ_agents::population::cell_of(&map, home) as u32;
    let households: Vec<(
        civ_core::PermanentId,
        Option<civ_core::PermanentId>,
        (f32, f32),
    )> = sim
        .people()
        .households
        .iter()
        .map(|(_, x)| (x.id, x.settlement, x.home))
        .collect();
    // Two homes, one of each settlement, each near water that carries a dugout, and a way a
    // cart could take between them over dry land.
    let wheels = sim.rules().catalog.vehicles[cart_v].wheels(0.0, 1.4);
    let cell = |at: (f32, f32)| civ_agents::population::cell_of(&map, at) as u32;
    let lane = |x: (f32, f32), y: (f32, f32)| {
        let land = sim.land();
        let decks = land.cart_decks(&map);
        let dry =
            |c: u32| (map.water[c as usize] == civ_world::WATER_LAND).then_some(wheels.trail_c);
        land.cart_terrain(&map, &decks)
            .route(&wheels, &dry, &[cell(x)], &[cell(y)])
            .map(|r| r.cells)
    };
    let on_water = |at: (f32, f32)| {
        let land = sim.land();
        let (rivers, hyd, q) = (
            &land.rivers,
            &land.floodplain.reaches,
            &land.reach_discharges,
        );
        rivers
            .nearest_bank(at, 1_500.0, |c| rivers.floats(&map, c, hyd, q, hull))
            .is_some()
    };
    let settlement = households[0].1;
    let (a, a_home, c, c_home, lane) = households
        .iter()
        .filter(|h| h.1 == settlement && on_water(h.2))
        .flat_map(|x| {
            households
                .iter()
                .filter(|h| h.1.is_some() && h.1 != settlement && on_water(h.2))
                .map(move |y| (x.0, x.2, y.0, y.2))
        })
        .find_map(|(a, x, c, y)| lane(x, y).map(|l| (a, x, c, y, l)))
        .expect("two homes on the water that a cart could go between");
    // `a`'s boat is drawn up at the bank nearest its home where the water carries a dugout today;
    // one more is being made there; its cart stands at its door.
    let land = sim.land();
    let (rivers, hyd, q) = (
        &land.rivers,
        &land.floodplain.reaches,
        &land.reach_discharges,
    );
    let floats = |c: u32| rivers.floats(&map, c, hyd, q, hull);
    let bank = rivers
        .nearest_bank(a_home, 1_500.0, floats)
        .expect("a bank near `a`'s home");
    let near_c = rivers
        .nearest_bank(c_home, 1_500.0, floats)
        .expect("a bank near `c`'s home");
    let apart = |x: (f32, f32), y: (f32, f32)| (x.0 - y.0).hypot(x.1 - y.1);
    assert!(
        rivers
            .route(
                hyd,
                q,
                hull,
                &rivers.boarding(&map, bank),
                &rivers.boarding(&map, near_c)
            )
            .is_some(),
        "the water links them today"
    );
    let ids: Vec<civ_core::PermanentId> = (0..4).map(|_| sim.allocate_id_for_tests()).collect();
    let (boat, unmade, cart) = (ids[0], ids[1], ids[2]);
    let land = sim.land_mut_for_tests();
    // A lane laid there that nobody keeps, as wide as a cart needs.
    land.ways.add(kept_way(
        ids[3],
        lane,
        civ_land::ways::WALK | civ_land::ways::CART,
        civ_land::ways::Keeper::Nobody,
        2.0,
    ));
    land.boats.list.push(Boat {
        saves_h_year: 30.0,
        ..vehicle(boat, boat_v, a, bank)
    });
    land.boats.list.push(Boat {
        labour_h: 100.0,
        work_h: 40.0,
        ..vehicle(unmade, boat_v, a, bank)
    });
    land.boats.changed();
    land.carts.list.push(vehicle(cart, cart_v, a, door(a_home)));
    land.carts.changed();
    let (boats_rev, carts_rev) = (sim.land().boats.revision(), sim.land().carts.revision());
    // `a`'s people join `c`, of the other settlement: its made boat and cart are `c`'s now,
    // standing where they were; the one being made stays `a`'s, where it is.
    sim.with_ctx_for_tests(|pop, ctx| pop.merge_household_for_tests(ctx, a, c));
    let land = sim.land();
    let find = |list: &[Boat], id| list.iter().find(|x| x.id == id).cloned().expect("there");
    let (b, u, k) = (
        find(&land.boats.list, boat),
        find(&land.boats.list, unmade),
        find(&land.carts.list, cart),
    );
    assert_eq!(
        (b.owner, b.at),
        (c, bank),
        "the boat is `c`'s, where it was"
    );
    assert_eq!(
        (k.owner, k.at),
        (c, door(a_home)),
        "the cart is `c`'s, at the old door"
    );
    assert_eq!(
        (u.owner, u.at, u.work_h),
        (a, bank, 40.0),
        "the one being made stays"
    );
    assert_ne!(land.boats.revision(), boats_rev);
    assert_ne!(land.carts.revision(), carts_rev);
    assert!(land.boats.problems().is_empty() && land.carts.problems().is_empty());
    // Someone of `c` walks to each and brings it home: the boat paddled to a bank near home, the
    // cart pulled to the door, each a trip of its own.
    let fetch = sim
        .rules()
        .catalog
        .activities
        .iter()
        .position(|x| x.behavior == civ_agents::params::Behavior::Fetch)
        .expect("fetching") as u16;
    let (mut paddled, mut pulled, mut saved) = (false, false, false);
    for _ in 0..4 * 24 * 6 {
        sim.advance_minutes(10).expect("advances");
        for (_, p) in sim.people().people.iter() {
            if p.household != c || p.act.def != fetch {
                continue;
            }
            let step = p.act.steps.get(usize::from(p.act.step));
            match (p.act.target, step) {
                (
                    civ_agents::person::Target::Boat(x),
                    Some(civ_agents::person::Step::Paddle { .. }),
                ) if x == boat => {
                    paddled |= p.trip.as_ref().is_some_and(|t| t.afloat);
                }
                (
                    civ_agents::person::Target::Boat(x),
                    Some(civ_agents::person::Step::Haul { .. }),
                ) if x == cart => {
                    pulled |= p.trip.as_ref().is_some_and(|t| !t.afloat);
                }
                _ => {}
            }
        }
        // Saved and loaded with someone on the way, the world goes on alike.
        if !saved && (paddled || pulled) {
            saves_and_goes_on_alike(&mut sim, 60);
            saved = true;
        }
        let land = sim.land();
        let (b, k) = (find(&land.boats.list, boat), find(&land.carts.list, cart));
        if b.at != bank && k.at != door(a_home) {
            break;
        }
    }
    let land = sim.land();
    let (b, k) = (find(&land.boats.list, boat), find(&land.carts.list, cart));
    assert!(paddled, "someone of `c` paddled the boat");
    assert!(pulled, "someone of `c` pulled the cart");
    assert_eq!(b.owner, c);
    assert!(
        apart(rivers_centre(&sim, b.at), c_home) <= 1_500.0,
        "the boat is drawn up near `c`'s home"
    );
    assert_eq!(
        (k.owner, k.at),
        (c, door(c_home)),
        "the cart stands at `c`'s door"
    );
    assert!(sim.land().boats.problems().is_empty() && sim.land().carts.problems().is_empty());
    // The one being made stands where it was, `a`'s still.
    let u = find(&sim.land().boats.list, unmade);
    assert_eq!((u.owner, u.at), (a, bank));
    saves_and_goes_on_alike(&mut sim, DAY);
}

fn rivers_centre(sim: &Sim, c: u32) -> (f32, f32) {
    sim.land().rivers.centre(c)
}
