//! Saves round-trip exactly; damaged, foreign and inconsistent saves are refused; boundary
//! payloads describe the map faithfully (ADR-0002 §6, ADR-0001).

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

use civ_content::ContentRegistry;
use civ_schema::flatbuffers::{self, FlatBufferBuilder};
use civ_schema::{SAVE_ENGINE_TAG, SAVE_SCHEMA_VERSION, save, wire};
use civ_sim::frames::{self, RasterQuery};
use civ_sim::persist::{self, LoadError, SECTION_RECEIVERS, agents};
use civ_sim::{NewWorld, Sim};
use commons_persist::{
    ChunkInfo, DEFAULT_ZSTD_LEVEL, Published, SaveDir, SaveKind, SectionData, SnapshotInfo,
};

/// A map side just over one save tile, so raster sections have full and partial tiles.
const SIDE: u32 = 576;
const PRESET: &str = "core:worldgen/river_valley";

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

struct Fixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
    first: Published,
    first_info: SnapshotInfo,
}

/// One world, generated once, run for a while and saved.
fn fixture() -> &'static Fixture {
    static FIXTURE: OnceLock<Fixture> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
        let mut sim = Sim::create(
            &NewWorld {
                name: "Test Valley".to_owned(),
                seed: 5,
                preset_id: PRESET.to_owned(),
                size_cells: SIDE,
                band_size: 0,
            },
            content(),
            &mut |_| {},
            &AtomicBool::new(false),
        )
        .expect("generates");
        sim.advance_minutes(3 * 24 * 60 + 7).expect("advances");
        sim.set_speed(288.0).expect("3x is a valid speed");
        sim.set_paused(false);
        let saves = SaveDir::create(dir.path().join("first"), civ_schema::SAVE_EXTENSION)
            .expect("save dir");
        let first =
            persist::save(&mut sim, &saves, SaveKind::Manual, "Before the flood").expect("saves");
        let first_info = commons_persist::SnapshotReader::open_file(
            &first.path,
            commons_persist::Limits::default(),
        )
        .expect("opens")
        .info()
        .clone();
        Fixture {
            root: dir.path().to_owned(),
            _dir: dir,
            first,
            first_info,
        }
    })
}

fn load_first() -> Sim {
    persist::load(&fixture().first.path, content()).expect("the first save loads")
}

fn digests(chunks: &[ChunkInfo]) -> Vec<(String, u32, u32, u64, u64)> {
    chunks
        .iter()
        .map(|c| {
            (
                c.tag.to_string(),
                c.index,
                c.version,
                c.raw_len,
                c.raw_digest,
            )
        })
        .collect()
}

fn scratch_dir(name: &str) -> SaveDir {
    SaveDir::create(fixture().root.join(name), civ_schema::SAVE_EXTENSION).expect("save dir")
}

#[test]
fn save_load_save_keeps_every_section_digest() {
    let fx = fixture();
    let mut loaded = load_first();
    assert_eq!(loaded.generation(), 1);
    assert_eq!(loaded.last_snapshot(), Some(fx.first_info.snapshot_id));
    assert!(!loaded.is_dirty());
    assert!(!loaded.content_changed());
    assert!(!loaded.paused(), "a running clock stays running");
    assert_eq!(loaded.speed(), 288.0);
    assert_eq!(loaded.now().minutes(), fx.first_info.sim_time);
    assert_eq!(loaded.meta().name, "Test Valley");
    assert_eq!(loaded.meta().seed, 5);
    assert_eq!(loaded.meta().world_id, fx.first_info.world_id);

    let again = persist::save(
        &mut loaded,
        &scratch_dir("again"),
        SaveKind::Manual,
        "again",
    )
    .expect("saves again");
    assert_eq!(digests(&fx.first.chunks), digests(&again.chunks));
    // 4 single-chunk world sections, 4 rasters of 2×2 tiles, 7 land and people sections.
    assert_eq!(again.chunks.len(), 4 + 4 * 4 + 7);
    assert!(loaded.people().living() > 0, "the founding band was saved");

    let info = commons_persist::SnapshotReader::open_file(&again.path, Default::default())
        .expect("opens")
        .info()
        .clone();
    assert_eq!(info.parent_id, Some(fx.first_info.snapshot_id));
    assert_eq!(info.world_id, fx.first_info.world_id);
}

#[test]
fn a_loaded_map_equals_a_regenerated_one() {
    // Generation is deterministic on one build, so the saved world can be compared with a fresh
    // one from the same seed, including the derived drainage rebuilt on load.
    let loaded = load_first();
    let fresh = Sim::create(
        &NewWorld {
            name: "Again".to_owned(),
            seed: 5,
            preset_id: PRESET.to_owned(),
            size_cells: SIDE,
            band_size: 0,
        },
        content(),
        &mut |_| {},
        &AtomicBool::new(false),
    )
    .expect("generates");
    assert!(
        **loaded.map() == **fresh.map(),
        "terrain differs after a round trip"
    );
    assert_eq!(loaded.meta().params, fresh.meta().params);
}

#[test]
fn damaged_saves_are_refused() {
    let bytes = std::fs::read(&fixture().first.path).expect("reads");
    let dir = scratch_dir("damaged");

    let mut flipped = bytes.clone();
    let middle = flipped.len() / 2;
    flipped[middle] ^= 0x5a;
    let path = dir.path().join("flipped.tcesave");
    std::fs::write(&path, &flipped).expect("writes");
    let err = persist::load(&path, content()).expect_err("a flipped byte is refused");
    assert!(matches!(err, LoadError::Container(_)), "{err}");

    let path = dir.path().join("truncated.tcesave");
    std::fs::write(&path, &bytes[..bytes.len() / 2]).expect("writes");
    let err = persist::load(&path, content()).expect_err("a truncated file is refused");
    assert!(matches!(err, LoadError::Container(_)), "{err}");

    let summary = persist::describe(&path, content());
    assert!(!summary.compatible);
    assert!(!summary.note.is_empty());
}

fn republish(name: &str, info: &SnapshotInfo, sections: &[SectionData]) -> PathBuf {
    let dir = scratch_dir(name);
    commons_persist::publish_file(
        dir.path(),
        "g0000000001-manual.tcesave",
        info,
        sections,
        DEFAULT_ZSTD_LEVEL,
    )
    .expect("publishes")
    .path
}

#[test]
fn foreign_engines_and_unknown_schemas_are_refused() {
    let sim = load_first();
    let sections = persist::encode_sections(&sim);
    let mut info = fixture().first_info.clone();

    info.engine_tag = *b"OTHER\0\0\0";
    let path = republish("foreign", &info, &sections);
    let err = persist::load(&path, content()).expect_err("another engine's save is refused");
    assert!(matches!(err, LoadError::Incompatible(_)), "{err}");
    assert!(!persist::describe(&path, content()).compatible);

    info.engine_tag = SAVE_ENGINE_TAG;
    info.schema_version = SAVE_SCHEMA_VERSION + 1;
    let path = republish("future", &info, &sections);
    let err = persist::load(&path, content()).expect_err("an unknown schema is refused");
    assert!(matches!(err, LoadError::Incompatible(_)), "{err}");
}

#[test]
fn an_inconsistent_world_is_refused_even_with_good_checksums() {
    let sim = load_first();
    let w = sim.map().width as usize;
    let mut sections = persist::encode_sections(&sim);
    // Point two interior neighbours at each other: a receiver cycle.
    let tile = sections
        .iter_mut()
        .find(|s| s.tag == SECTION_RECEIVERS && s.index == 0)
        .expect("first receiver tile");
    let decoded = flatbuffers::root::<save::RasterTile>(&tile.bytes).expect("decodes");
    let tile_w = decoded.width() as usize;
    let mut data = decoded.data().expect("data").bytes().to_vec();
    let a = 40 * tile_w + 40;
    data[a] = 0; // east
    data[a + 1] = 4; // west
    let mut fbb = FlatBufferBuilder::new();
    let data = fbb.create_vector(&data);
    let root = save::RasterTile::create(
        &mut fbb,
        &save::RasterTileArgs {
            layer: decoded.layer(),
            x0: decoded.x0(),
            y0: decoded.y0(),
            width: decoded.width(),
            height: decoded.height(),
            data: Some(data),
        },
    );
    fbb.finish(root, None);
    tile.bytes = fbb.finished_data().to_vec();
    assert!(w > 41);

    let path = republish("cycle", &fixture().first_info, &sections);
    match persist::load(&path, content()) {
        Err(LoadError::Invalid(problems)) => {
            assert!(problems.iter().any(|p| p.contains("cycle")), "{problems:?}");
        }
        other => panic!("expected an invalid-world refusal, got {other:?}"),
    }
}

#[test]
fn m0_saves_load_as_worlds_without_people() {
    let sim = load_first();
    let m0_sections: Vec<SectionData> = persist::encode_sections(&sim)
        .into_iter()
        .filter(|s| {
            ![
                agents::SECTION_LAND,
                agents::SECTION_SETTLE,
                agents::SECTION_PEOPLE,
                agents::SECTION_HOUSES,
                agents::SECTION_HISTORY,
                agents::SECTION_RECEIPTS,
                agents::SECTION_EVENTS,
            ]
            .contains(&s.tag)
        })
        .collect();
    let mut info = fixture().first_info.clone();
    info.schema_version = persist::SCHEMA_V1;
    let path = republish("m0", &info, &m0_sections);
    assert!(persist::describe(&path, content()).compatible);
    let mut migrated = persist::load(&path, content()).expect("an M0 save loads");
    assert_eq!(migrated.people().living(), 0, "nobody yet");
    assert!(
        !migrated.land().patches.is_empty(),
        "the land is classified"
    );
    assert_eq!(
        migrated.land().stocks.len(),
        content().land.params.resources.len()
    );
    assert!(migrated.is_dirty(), "the migration is new state");
    migrated
        .advance_minutes(3 * 24 * 60)
        .expect("an empty world runs");

    // Saved again, it is a version-2 save.
    let saved = persist::save(
        &mut migrated,
        &scratch_dir("m0-again"),
        SaveKind::Manual,
        "",
    )
    .expect("saves");
    let info = commons_persist::SnapshotReader::open_file(&saved.path, Default::default())
        .expect("opens")
        .info()
        .clone();
    assert_eq!(info.schema_version, SAVE_SCHEMA_VERSION);
    let reloaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(reloaded.people().living(), 0);
}

#[test]
fn a_version_2_save_without_its_people_sections_is_refused() {
    let sim = load_first();
    let sections: Vec<SectionData> = persist::encode_sections(&sim)
        .into_iter()
        .filter(|s| s.tag != agents::SECTION_HOUSES)
        .collect();
    let path = republish("no-houses", &fixture().first_info, &sections);
    assert!(persist::load(&path, content()).is_err());
}

#[test]
fn people_whose_activity_left_the_content_decide_again() {
    let sim = load_first();
    // The activity most people are doing.
    let mut counts = std::collections::BTreeMap::new();
    for (_, p) in sim.people().people.iter() {
        *counts.entry(p.act.def).or_insert(0usize) += 1;
    }
    let (&busiest, &doing) = counts.iter().max_by_key(|(_, n)| **n).expect("people");
    let gone = sim.rules().catalog.activities[usize::from(busiest)]
        .id
        .clone();
    let mut changed = content().clone();
    changed.catalog.activities.retain(|a| a.id != gone);
    changed.fingerprint[0] ^= 1;

    let mut loaded = persist::load(&fixture().first.path, &changed).expect("still loads");
    assert!(loaded.content_changed());
    assert!(loaded.is_dirty(), "{doing} people must decide again");
    loaded.advance_minutes(2).expect("advances");
    let catalog = &loaded.rules().catalog;
    assert!(catalog.index_of(&gone).is_none());
    for (_, p) in loaded.people().people.iter() {
        assert!(usize::from(p.act.def) < catalog.activities.len());
    }
}

#[test]
fn changed_content_is_noted_not_refused() {
    let mut changed = content().clone();
    changed.fingerprint[0] ^= 1;
    let sim = persist::load(&fixture().first.path, &changed).expect("still loads");
    assert!(sim.content_changed());
    let summary = persist::describe(&fixture().first.path, &changed);
    assert!(summary.compatible);
    assert!(summary.content_changed);
    assert!(summary.note.contains("content"));
}

#[test]
fn describe_reads_the_header_and_name() {
    let summary = persist::describe(&fixture().first.path, content());
    assert!(summary.compatible, "{}", summary.note);
    assert!(!summary.content_changed);
    assert_eq!(summary.world_name, "Test Valley");
    let info = summary.info.expect("header");
    assert_eq!(info.label, "Before the flood");
    assert_eq!(info.generation, 1);
    assert!(summary.file_len > 0);
}

fn decode_tile(bytes: &[u8]) -> wire::RasterTile<'_> {
    flatbuffers::root::<wire::Response>(bytes)
        .expect("a response")
        .body_as_raster_tile()
        .expect("a raster tile")
}

#[test]
fn raster_tiles_match_the_map() {
    let sim = load_first();
    let (map, stats) = (sim.map(), sim.stats());
    let query = |layer, level, x0, y0, width, height| RasterQuery {
        layer,
        level,
        x0,
        y0,
        width,
        height,
    };

    // Full resolution: dequantised elevation is within half a quantum of the map.
    let bytes = frames::raster_response(
        map,
        stats,
        &query(wire::RasterLayer::Elevation, 0, 10, 20, 100, 50),
    )
    .expect("answers");
    let tile = decode_tile(&bytes);
    assert_eq!((tile.width(), tile.height()), (100, 50));
    assert_eq!((tile.full_width(), tile.full_height()), (SIDE, SIDE));
    assert_eq!(tile.format(), wire::RasterFormat::U16);
    let data = tile.data().expect("data").bytes();
    assert_eq!(data.len(), 100 * 50 * 2);
    let half_quantum = tile.scale() / 2.0 + 1e-3;
    for y in 0..50usize {
        for x in 0..100usize {
            let i = 2 * (y * 100 + x);
            let raw = f32::from(u16::from_le_bytes([data[i], data[i + 1]]));
            let value = raw * tile.scale() + tile.offset();
            let truth = map.elevation[(20 + y) * SIDE as usize + 10 + x];
            assert!((value - truth).abs() <= half_quantum, "{value} vs {truth}");
        }
    }

    // A coarse level covers the map in fewer cells and is clipped at its edge.
    let bytes = frames::raster_response(
        map,
        stats,
        &query(wire::RasterLayer::Water, 3, 0, 0, 1000, 1000),
    )
    .expect("answers");
    let tile = decode_tile(&bytes);
    assert_eq!((tile.full_width(), tile.width()), (SIDE / 8, SIDE / 8));
    assert_eq!(tile.format(), wire::RasterFormat::U8);
    let data = tile.data().expect("data").bytes();
    assert!(data.iter().all(|&c| c <= 3));

    for layer in [wire::RasterLayer::DrainageArea, wire::RasterLayer::LakeId] {
        let bytes =
            frames::raster_response(map, stats, &query(layer, 1, 0, 0, 64, 64)).expect("answers");
        let tile = decode_tile(&bytes);
        assert_eq!(tile.data().expect("data").len(), 64 * 64 * 4);
    }

    // Out of range and oversized regions are refused.
    let off_map = query(wire::RasterLayer::Elevation, 0, SIDE, 0, 1, 1);
    assert!(frames::raster_response(map, stats, &off_map).is_err());
    let empty = query(wire::RasterLayer::Elevation, 0, 0, 0, 0, 10);
    assert!(frames::raster_response(map, stats, &empty).is_err());
    let too_coarse = query(
        wire::RasterLayer::Elevation,
        frames::MAX_LEVEL + 1,
        0,
        0,
        1,
        1,
    );
    assert!(frames::raster_response(map, stats, &too_coarse).is_err());
}

#[test]
fn hydrography_has_every_reach_and_lake() {
    let sim = load_first();
    let map = sim.map();
    let bytes = frames::hydrography_response(map, 4.0);
    let hydro = flatbuffers::root::<wire::Response>(&bytes)
        .expect("a response")
        .body_as_hydrography()
        .expect("hydrography");
    let reaches = hydro.reaches().expect("reaches");
    assert_eq!(reaches.len(), map.reaches.len());
    assert!(!map.reaches.is_empty(), "the test world should have rivers");
    let (extent_x, extent_y) = map.extent_m();
    for reach in reaches {
        let points = reach.points().expect("points");
        assert!(!points.is_empty());
        for p in points {
            assert!(p.x() >= 0.0 && f64::from(p.x()) <= extent_x);
            assert!(p.y() >= 0.0 && f64::from(p.y()) <= extent_y);
        }
        assert!(reach.downstream() < reaches.len() as i32);
    }
    assert_eq!(hydro.lakes().map_or(0, |l| l.len()), map.lakes.len());
}

#[test]
fn snapshot_tables_decode() {
    let sim = load_first();
    let mut fbb = FlatBufferBuilder::new();
    let world = frames::world_info(&mut fbb, &sim);
    let clock = frames::clock(&mut fbb, &sim);
    let snapshot = wire::Snapshot::create(
        &mut fbb,
        &wire::SnapshotArgs {
            world: Some(world),
            clock: Some(clock),
            ..Default::default()
        },
    );
    fbb.finish(snapshot, None);
    let decoded = flatbuffers::root::<wire::Snapshot>(fbb.finished_data()).expect("decodes");
    let world = decoded.world().expect("world");
    assert_eq!(world.name(), Some("Test Valley"));
    assert_eq!(world.width(), SIDE);
    assert_eq!(world.generation(), 1);
    assert_eq!(world.world_id().map(str::len), Some(32));
    let clock = decoded.clock().expect("clock");
    // Created at year 1, March 1, 06:00 and run for 3 days and 7 minutes.
    assert_eq!(
        (clock.year(), clock.month(), clock.day(), clock.hour()),
        (1, 3, 4, 6)
    );
    assert_eq!(clock.minute_of_hour(), 7);
    assert_eq!(clock.season(), Some("spring"));
}

#[test]
fn the_clock_runs_only_when_unpaused() {
    let mut sim = load_first();
    sim.set_paused(true);
    assert_eq!(sim.advance_real(10.0).expect("ok").minutes, 0);

    sim.set_paused(false);
    sim.set_speed(civ_sim::SPEED_1X).expect("1x");
    let start = sim.now();
    // 10 real seconds at 96 simulated seconds per second is 16 minutes.
    assert_eq!(sim.advance_real(10.0).expect("ok").minutes, 16);
    // Fractions of a minute carry over.
    assert_eq!(sim.advance_real(0.3).expect("ok").minutes, 0);
    assert_eq!(sim.advance_real(0.4).expect("ok").minutes, 1);
    assert_eq!(sim.now().minutes() - start.minutes(), 17);
    assert!(sim.is_dirty());

    let advance = sim.advance_minutes(40 * 24 * 60).expect("ok");
    assert_eq!(advance.days, 40);
    assert_eq!(advance.months, 1);
    assert_eq!(advance.years, 0);

    assert!(sim.set_speed(0.0).is_err());
    assert!(sim.set_speed(f32::NAN).is_err());
    assert!(sim.set_speed(civ_sim::MAX_SPEED * 2.0).is_err());
}

#[test]
fn unknown_presets_are_refused() {
    let result = Sim::create(
        &NewWorld {
            name: String::new(),
            seed: 1,
            preset_id: "core:worldgen/nowhere".to_owned(),
            size_cells: 256,
            band_size: 0,
        },
        content(),
        &mut |_| {},
        &AtomicBool::new(false),
    );
    assert!(matches!(result, Err(civ_sim::SimError::UnknownPreset(_))));
}
