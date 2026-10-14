//! Linear banks retain explicit borrow earth and their sub-grid crest through snapshots.
use civ_content::ContentRegistry;
use civ_land::{
    RectCm,
    banks::{self, BankSegment},
    earth::{self, EarthKind, Earthwork},
};
use civ_sim::{NewWorld, Sim, persist};
use commons_persist::{SaveDir, SaveKind};
use std::{path::Path, sync::OnceLock};
fn content() -> &'static ContentRegistry {
    static C: OnceLock<ContentRegistry> = OnceLock::new();
    C.get_or_init(|| {
        civ_content::load(
            &civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("root"),
        )
        .registry
        .expect("content")
    })
}
fn fixture() -> Sim {
    let mut sim = Sim::create_for_tests(
        &NewWorld {
            name: "Bank".into(),
            seed: 2,
            preset_id: "core:worldgen/river_valley".into(),
            size_cells: 512,
            band_size: 30,
            neighbours: vec![],
            neighbours_known: true,
            regime_id: String::new(),
        },
        content(),
        [7; 16],
    )
    .expect("world");
    let household = sim
        .people()
        .households
        .iter()
        .next()
        .expect("household")
        .1
        .id;
    let work = sim.allocate_id_for_tests();
    let borrow = sim.allocate_id_for_tests();
    let segment = BankSegment {
        work,
        borrow,
        a: [200.0, 200.0],
        b: [200.0, 224.0],
        height_m: 1.5,
        crest_width_m: 1.0,
        side_run: 2.0,
        compacted_per_bank: 0.9,
        settlement_m: 0.1,
        erosion_m: 0.0,
        breached: false,
        sediment_m3: 0.0,
        piping: 0.0,
    };
    let make = |id, kind, rect| Earthwork {
        id,
        kind,
        rect,
        level_cm: 0,
        side_run_cm: 200,
        plot: None,
        deposit: None,
        heap: None,
        household,
        cut_m3: 0.0,
        done: 0.0,
        version: earth::EARTH_VERSION,
        begun: sim.now(),
    };
    let records = [
        make(
            work,
            EarthKind::Bank,
            RectCm {
                x: 19300,
                y: 20000,
                w: 1400,
                h: 2400,
            },
        ),
        make(
            borrow,
            EarthKind::Pit,
            RectCm {
                x: 18000,
                y: 20000,
                w: 1000,
                h: 2400,
            },
        ),
    ];
    let map = sim.map().clone();
    let land = sim.land_mut_for_tests();
    land.earthworks.extend(records);
    banks::place_from_borrow(&segment, &mut land.earthworks, &map, &mut land.ground, 80.0)
        .expect("move soil");
    land.banks.push(segment);
    sim
}
#[test]
fn bank_geometry_borrow_earth_and_boundary_round_trip_exactly() {
    let mut sim = fixture();
    let map = sim.map().clone();
    let land = sim.land_mut_for_tests();
    let depths: Vec<_> = (0..map.cell_count())
        .map(|c| (2.0 - land.floodplain.cell_hand[c] - land.ground.at(c)).max(0.0))
        .collect();
    banks::overtop_day(
        &mut land.banks[0],
        &land.earthworks[0],
        &map,
        &land.floodplain,
        &depths,
        banks::BankParams::default(),
        &mut land.ground,
    );
    // Engineered fixture may be high above its drainage datum; force a local overflowing surface.
    if land.banks[0].sediment_m3 == 0.0 {
        let depths = vec![1000.0; map.cell_count()];
        banks::overtop_day(
            &mut land.banks[0],
            &land.earthworks[0],
            &map,
            &land.floodplain,
            &depths,
            banks::BankParams {
                erosion_per_head_day: 0.0000001,
                ..banks::BankParams::default()
            },
            &mut land.ground,
        );
    }
    assert!(land.banks[0].sediment_m3 > 0.0);
    land.banks[0].piping = 0.375;
    let d = tempfile::tempdir().expect("dir");
    let saves = SaveDir::create(d.path(), civ_schema::SAVE_EXTENSION).expect("saves");
    let saved = persist::save(&mut sim, &saves, SaveKind::Manual, "bank").expect("save");
    let loaded = persist::load(&saved.path, content()).expect("load");
    assert_eq!(sim.land().banks, loaded.land().banks);
    for (a, b) in persist::encode_sections(&sim)
        .iter()
        .zip(persist::encode_sections(&loaded))
    {
        assert_eq!(a.bytes, b.bytes, "{}", a.tag);
    }
    let bytes = civ_sim::frames::earthworks::earthworks_response(&loaded);
    let response =
        civ_schema::flatbuffers::root::<civ_schema::wire::Response>(&bytes).expect("wire");
    let bank = response
        .body_as_earthworks()
        .expect("earthworks")
        .works()
        .expect("works")
        .iter()
        .find(|w| w.kind() == 3)
        .expect("bank");
    assert_eq!(bank.bank_a().expect("a").x(), 200.0);
    assert_eq!(bank.bank_b().expect("b").y(), 224.0);
    assert!(bank.bank_height_m() > 0.65);
    assert!(bank.words().expect("words").contains("borrow pit"));
    let mut replayed =
        earth::GroundDelta::new(sim.map().width, sim.map().height, sim.map().cell_size_m);
    for w in &sim.land().earthworks {
        earth::replay(w, sim.map(), &mut replayed);
    }
    for b in &sim.land().banks {
        banks::replay(
            b,
            sim.land()
                .earthworks
                .iter()
                .find(|w| w.id == b.work)
                .expect("work"),
            sim.map(),
            &mut replayed,
        );
    }
    for c in 0..sim.map().cell_count() {
        assert!(
            (replayed.at(c) - sim.land().ground.at(c)).abs() < 1e-4,
            "cell {c}"
        );
    }
}
#[test]
fn malformed_geometry_duplicate_borrow_and_missing_sources_are_refused() {
    for variant in 0..11 {
        let mut sim = fixture();
        let land = sim.land_mut_for_tests();
        match variant {
            0 => land.banks[0].height_m = f64::NAN,
            1 => land.banks[0].compacted_per_bank = 0.0,
            2 => land.banks[0].borrow = land.banks[0].work,
            3 => {
                land.banks.push(land.banks[0]);
            }
            4 => land.earthworks[0].cut_m3 += 1.0,
            5 => land.earthworks[0].rect.x = -1,
            6 => land.earthworks[0].rect.w = 1,
            7 => land.banks[0].sediment_m3 = f64::NAN,
            8 => land.banks[0].piping = f64::NAN,
            9 => land.banks[0].piping = 1.5,
            _ => land.banks[0].sediment_m3 = 10000.0,
        }
        let d = tempfile::tempdir().expect("dir");
        let saves = SaveDir::create(d.path(), civ_schema::SAVE_EXTENSION).expect("saves");
        let saved =
            persist::save(&mut sim, &saves, SaveKind::Manual, "invalid").expect("container");
        assert!(
            persist::load(&saved.path, content()).is_err(),
            "variant {variant}"
        );
    }
}

#[test]
fn schema_99_migrates_to_no_banks_and_refuses_new_records_in_old_headers() {
    for with_bank in [false, true] {
        let mut sim = fixture();
        if !with_bank {
            sim.land_mut_for_tests().banks.clear();
            sim.land_mut_for_tests().earthworks.clear();
            let map = sim.map();
            let ground = earth::GroundDelta::new(map.width, map.height, map.cell_size_m);
            sim.land_mut_for_tests().ground = ground;
        }
        let dir = tempfile::tempdir().expect("dir");
        let saves = SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("saves");
        let saved = persist::save(&mut sim, &saves, SaveKind::Manual, "old").expect("save");
        let reader = commons_persist::SnapshotReader::open_file(
            &saved.path,
            commons_persist::Limits::default(),
        )
        .expect("reader");
        let mut info = reader.info().clone();
        info.schema_version = persist::SCHEMA_V99;
        let legacy = commons_persist::publish_file(
            dir.path(),
            "legacy.tcesave",
            &info,
            &persist::encode_sections(&sim),
            commons_persist::DEFAULT_ZSTD_LEVEL,
        )
        .expect("publish");
        let loaded = persist::load(&legacy.path, content());
        if with_bank {
            assert!(loaded.is_err());
        } else {
            assert!(loaded.expect("99 loads").land().banks.is_empty());
        }
    }
}

#[test]
fn schema_100_keeps_banks_without_inventing_sediment_and_refuses_new_stocks() {
    for with_sediment in [false, true] {
        let mut sim = fixture();
        if with_sediment {
            sim.land_mut_for_tests().banks[0].sediment_m3 = 1.0;
        }
        let dir = tempfile::tempdir().expect("dir");
        let saves = SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("saves");
        let saved = persist::save(&mut sim, &saves, SaveKind::Manual, "old bank").expect("save");
        let reader = commons_persist::SnapshotReader::open_file(
            &saved.path,
            commons_persist::Limits::default(),
        )
        .expect("reader");
        let mut info = reader.info().clone();
        info.schema_version = persist::SCHEMA_V100;
        let legacy = commons_persist::publish_file(
            dir.path(),
            "legacy.tcesave",
            &info,
            &persist::encode_sections(&sim),
            commons_persist::DEFAULT_ZSTD_LEVEL,
        )
        .expect("publish");
        let loaded = persist::load(&legacy.path, content());
        if with_sediment {
            assert!(loaded.is_err());
        } else {
            let loaded = loaded.expect("100 loads");
            assert_eq!(loaded.land().banks, sim.land().banks);
            assert_eq!(loaded.land().ground, sim.land().ground);
        }
    }
}

#[test]
fn schema_102_keeps_banks_undamaged_and_refuses_piping_in_old_headers() {
    for with_piping in [false, true] {
        let mut sim = fixture();
        if with_piping {
            sim.land_mut_for_tests().banks[0].piping = 0.25;
        }
        let dir = tempfile::tempdir().expect("dir");
        let saves = SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("saves");
        let saved = persist::save(&mut sim, &saves, SaveKind::Manual, "old bank").expect("save");
        let reader = commons_persist::SnapshotReader::open_file(
            &saved.path,
            commons_persist::Limits::default(),
        )
        .expect("reader");
        let mut info = reader.info().clone();
        info.schema_version = persist::SCHEMA_V102;
        let legacy = commons_persist::publish_file(
            dir.path(),
            "legacy.tcesave",
            &info,
            &persist::encode_sections(&sim),
            commons_persist::DEFAULT_ZSTD_LEVEL,
        )
        .expect("publish");
        let loaded = persist::load(&legacy.path, content());
        if with_piping {
            assert!(loaded.is_err());
        } else {
            let loaded = loaded.expect("102 loads");
            assert_eq!(loaded.land().banks, sim.land().banks);
            assert!(loaded.land().banks.iter().all(|b| b.piping == 0.0));
        }
    }
}

#[test]
fn lived_day_erodes_once_and_repeated_depth_queries_do_not() {
    let sim = fixture();
    let map = sim.map().clone();
    let mut land = sim.land().clone();
    let mut params = sim.rules().land.clone();
    params.banks.erosion_per_head_day = 0.00000001;
    // The saved construction fixture is not beside a river. Give this calendar-only check
    // an explicit flat compartment connected to reach 0; it is not a generated flood example.
    let mut reach = land.floodplain.reaches[0].clone();
    reach.cells = (0..map.cell_count() as u32).collect();
    reach.hands = vec![0.0; map.cell_count()];
    reach.cumulative_v_m3 = vec![0.0; map.cell_count()];
    reach.transit_time_s = 60.0;
    land.floodplain = civ_world::hydraulics::WorldFloodplain {
        reaches: vec![reach],
        cell_reach: vec![0; map.cell_count()],
        cell_hand: vec![0.0; map.cell_count()],
    };
    // Engineering flood: every reach is loaded, avoiding any scripted household response.
    let until_day = land.weather.today.day + 4;
    land.river_forcings = map
        .reaches
        .iter()
        .map(|reach| civ_land::RiverForcing {
            reach: reach.id,
            discharge_m3s: 1_000_000.0,
            until_day,
            influence: 0,
        })
        .collect();
    let flows = land.river_flows(&map, &params, land.weather.today.day);
    land.update_floods(&map, flows);
    let day = land.stock_day + 1;
    land.advance_to_day(&params, &map, day);
    assert!(
        land.banks[0].sediment_m3 > 0.0,
        "the controlled river flood must reach the crest"
    );
    assert!(
        !land.banks[0].breached,
        "the two-day fixture must leave a residual crest"
    );
    let bank = land.banks[0];
    let ground = land.ground.clone();
    land.update_floods(&map, land.reach_discharges.clone());
    land.update_floods(&map, land.reach_discharges.clone());
    land.advance_to_day(&params, &map, day);
    assert_eq!(land.banks[0], bank);
    assert_eq!(land.ground, ground);
    land.advance_to_day(&params, &map, day + 1);
    assert!(land.banks[0].sediment_m3 > bank.sediment_m3);
}

#[test]
fn a_one_day_flood_charges_its_elapsed_erosion_before_the_forcing_expires() {
    let sim = fixture();
    let map = sim.map().clone();
    let mut land = sim.land().clone();
    let mut params = sim.rules().land.clone();
    params.banks.erosion_per_head_day = 0.00000001;
    // The saved construction fixture is not beside a river. Give this calendar-only check
    // an explicit flat compartment connected to reach 0; it is not a generated flood example.
    let mut reach = land.floodplain.reaches[0].clone();
    reach.cells = (0..map.cell_count() as u32).collect();
    reach.hands = vec![0.0; map.cell_count()];
    reach.cumulative_v_m3 = vec![0.0; map.cell_count()];
    reach.transit_time_s = 60.0;
    land.floodplain = civ_world::hydraulics::WorldFloodplain {
        reaches: vec![reach],
        cell_reach: vec![0; map.cell_count()],
        cell_hand: vec![0.0; map.cell_count()],
    };

    let until_day = land.weather.today.day + 1;
    land.river_forcings = map
        .reaches
        .iter()
        .map(|reach| civ_land::RiverForcing {
            reach: reach.id,
            discharge_m3s: 1_000_000.0,
            until_day,
            influence: 0,
        })
        .collect();
    let flows = land.river_flows(&map, &params, land.weather.today.day);
    land.update_floods(&map, flows);
    let day = land.stock_day + 1;
    land.advance_to_day(&params, &map, day);
    assert!(
        land.banks[0].sediment_m3 > 0.0,
        "the elapsed loaded day must count"
    );
    let bank = land.banks[0];
    land.advance_to_day(&params, &map, day + 1);
    assert_eq!(
        land.banks[0], bank,
        "the expired override must not charge a further day"
    );
}
