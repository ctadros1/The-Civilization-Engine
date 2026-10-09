//! Saves round-trip exactly; damaged, foreign and inconsistent saves are refused; boundary
//! payloads describe the map faithfully (ADR-0002 §6, ADR-0001).

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

use civ_content::ContentRegistry;
use civ_grammar::Stage;
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
    /// The fields of the world as it was saved.
    first_fields: Vec<civ_land::Field>,
    /// Its plots and buildings.
    first_plots: Vec<civ_land::Plot>,
    first_buildings: Vec<civ_land::Building>,
    /// Who was partnered with whom, where each woman was in her cycle, and the unions.
    first_families: Vec<Family>,
    first_unions: Vec<civ_agents::Union>,
}

type Family = (
    civ_core::PermanentId,
    Option<civ_core::PermanentId>,
    civ_agents::Repro,
    f32,
    Option<civ_core::PermanentId>,
);

fn families(sim: &Sim) -> Vec<Family> {
    let mut out: Vec<Family> = sim
        .people()
        .people
        .iter()
        .map(|(_, p)| (p.id, p.partner, p.repro, p.fecundity, p.nursing))
        .collect();
    out.sort_by_key(|f| f.0);
    out
}

/// Removes the folders named `prefix…` in `tmp` that earlier runs left, an hour old or more.
fn clear_stale(tmp: &Path, prefix: &str) {
    let Ok(entries) = std::fs::read_dir(tmp) else {
        return;
    };
    for e in entries.flatten() {
        let old = e
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.elapsed().ok())
            .is_some_and(|age| age.as_secs() >= 3600);
        if old && e.file_name().to_string_lossy().starts_with(prefix) {
            let _ = std::fs::remove_dir_all(e.path());
        }
    }
}

/// One world, generated once, run for a while and saved. A static is never dropped, so its folder
/// outlives the tests; each run first clears those earlier runs left.
fn fixture() -> &'static Fixture {
    static FIXTURE: OnceLock<Fixture> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let tmp = Path::new(env!("CARGO_TARGET_TMPDIR"));
        clear_stale(tmp, "roundtrip-");
        let dir = tempfile::Builder::new()
            .prefix("roundtrip-")
            .tempdir_in(tmp)
            .expect("temp dir");
        let mut sim = Sim::create(
            &NewWorld {
                name: "Test Valley".to_owned(),
                seed: 5,
                preset_id: PRESET.to_owned(),
                size_cells: SIDE,
                band_size: 0,
                neighbours: Vec::new(),
                neighbours_known: false,
                regime_id: String::new(),
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
            first_fields: sim.land().fields.clone(),
            first_plots: sim.land().plots.clone(),
            first_buildings: sim.land().buildings.clone(),
            first_families: families(&sim),
            first_unions: sim.people().unions.clone(),
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
    // 4 single-chunk world sections, 4 rasters of 2×2 tiles, 28 land, field, plot, building,
    // wear, market, firm, wealth, knowledge, deposits, earth, ties, polity, order, word, opinion,
    // norms, values, creeds, factions, influence, places and people sections.
    assert_eq!(again.chunks.len(), 4 + 4 * 4 + 28);
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
            neighbours: Vec::new(),
            neighbours_known: false,
            regime_id: String::new(),
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
fn fields_load_as_they_were_saved() {
    let fx = fixture();
    // In its first days of March the band marks out fields and starts breaking ground.
    assert!(!fx.first_fields.is_empty(), "the saved world has fields");
    assert_eq!(load_first().land().fields, fx.first_fields);
}

#[test]
fn a_version_4_save_without_its_fields_section_is_refused() {
    let sim = load_first();
    let sections: Vec<SectionData> = persist::encode_sections(&sim)
        .into_iter()
        .filter(|s| s.tag != agents::SECTION_FIELDS)
        .collect();
    let path = republish("no-fields", &fixture().first_info, &sections);
    assert!(persist::load(&path, content()).is_err());
}

#[test]
fn plots_and_buildings_load_as_they_were_saved() {
    let fx = fixture();
    // In their first days households claim ground for their homes and begin their huts.
    assert!(
        !fx.first_buildings.is_empty(),
        "the saved world has buildings"
    );
    let loaded = load_first();
    assert_eq!(loaded.land().plots, fx.first_plots);
    assert_eq!(loaded.land().buildings, fx.first_buildings);
    // Shelter is not saved: it follows from the buildings.
    for (_, h) in loaded.people().households.iter() {
        let roofed = loaded
            .land()
            .buildings
            .iter()
            .any(|b| b.household == h.id && b.roofed());
        assert_eq!(h.sheltered, roofed, "household {}", h.id);
    }
}

#[test]
fn couples_and_pregnancies_survive_a_save_and_load() {
    let fx = fixture();
    assert!(!fx.first_unions.is_empty(), "the saved world has couples");
    assert!(
        fx.first_families
            .iter()
            .any(|f| !matches!(f.2, civ_agents::Repro::Open)),
        "and women expecting or nursing"
    );
    let loaded = load_first();
    assert_eq!(families(&loaded), fx.first_families);
    assert_eq!(loaded.people().unions, fx.first_unions);
}

#[test]
fn slice_d_saves_load_with_their_couples_and_no_one_expecting() {
    // A schema-5 save has no couples or pregnancies: they are inferred from the households.
    let sim = load_first();
    let sections = persist::encode_sections(&sim);
    let mut info = fixture().first_info.clone();
    info.schema_version = persist::SCHEMA_V5;
    let path = republish("slice-d", &info, &sections);
    let mut migrated = persist::load(&path, content()).expect("a schema-5 save loads");
    assert!(migrated.is_dirty(), "the migration is new state");
    for (_, p) in migrated.people().people.iter() {
        assert_eq!(p.repro, civ_agents::Repro::Open);
        assert_eq!(p.nursing, None);
    }
    let couples = migrated
        .people()
        .people
        .iter()
        .filter(|(_, p)| p.partner.is_some())
        .count();
    assert!(
        couples
            >= 2 * fixture()
                .first_unions
                .len()
                .min(migrated.people().households.len())
    );
    migrated
        .advance_minutes(24 * 60)
        .expect("a migrated world runs");
}

#[test]
fn slice_e_saves_load_with_untrodden_ground() {
    // A schema-6 save has no wear section: nobody had worn the ground yet.
    let sim = load_first();
    let sections: Vec<SectionData> = persist::encode_sections(&sim)
        .into_iter()
        .filter(|s| s.tag != agents::SECTION_WEAR)
        .collect();
    let mut info = fixture().first_info.clone();
    info.schema_version = persist::SCHEMA_V6;
    let path = republish("slice-e", &info, &sections);
    let mut migrated = persist::load(&path, content()).expect("a schema-6 save loads");
    assert!(migrated.land().wear.tiles().is_empty());
    assert!(migrated.land().wear.trails().is_empty());
    migrated
        .advance_minutes(24 * 60)
        .expect("a migrated world runs");
    assert!(
        !migrated.land().wear.tiles().is_empty(),
        "and its people wear the ground again"
    );
}

#[test]
fn slice_f_saves_load_as_they_are() {
    // Schema 8 added only a chronicle kind (a family the observer sent).
    let sim = load_first();
    let sections = persist::encode_sections(&sim);
    let mut info = fixture().first_info.clone();
    info.schema_version = persist::SCHEMA_V7;
    let path = republish("slice-f", &info, &sections);
    let loaded = persist::load(&path, content()).expect("a schema-7 save loads");
    assert_eq!(loaded.people().living(), sim.people().living());
    assert_eq!(
        loaded.land().wear.tiles().len(),
        sim.land().wear.tiles().len()
    );
}

#[test]
fn slice_h_saves_load_with_no_markets_and_post_offers_again() {
    // A schema-9 save has no market section and no offers: markets start empty, and households
    // post their terms at their next weekly review.
    let sim = load_first();
    let sections: Vec<SectionData> = persist::encode_sections(&sim)
        .into_iter()
        .filter(|s| s.tag != agents::SECTION_MARKET)
        .collect();
    let mut info = fixture().first_info.clone();
    info.schema_version = persist::SCHEMA_V9;
    let path = republish("slice-h", &info, &sections);
    let mut migrated = persist::load(&path, content()).expect("a schema-9 save loads");
    assert!(migrated.is_dirty(), "the migration is new state");
    assert!(migrated.people().markets.is_empty());
    assert!(
        migrated
            .people()
            .households
            .iter()
            .all(|(_, h)| h.offers.is_empty())
    );
    migrated
        .advance_minutes(8 * 24 * 60)
        .expect("a migrated world runs");
    assert!(
        migrated
            .people()
            .households
            .iter()
            .any(|(_, h)| !h.offers.is_empty()),
        "and its households post offers again"
    );
}

#[test]
fn slice_i_saves_load_with_no_firms() {
    // A schema-10 save has no firms section: nobody had set up a workshop yet.
    let sim = load_first();
    let sections: Vec<SectionData> = persist::encode_sections(&sim)
        .into_iter()
        .filter(|s| s.tag != agents::SECTION_FIRMS)
        .collect();
    let mut info = fixture().first_info.clone();
    info.schema_version = persist::SCHEMA_V10;
    let path = republish("slice-i", &info, &sections);
    let mut migrated = persist::load(&path, content()).expect("a schema-10 save loads");
    assert!(migrated.is_dirty(), "the migration is new state");
    assert!(migrated.people().firms.is_empty());
    migrated
        .advance_minutes(24 * 60)
        .expect("a migrated world runs");
}

#[test]
fn slice_l_saves_load_with_what_founders_bring() {
    // A schema-12 save keeps nobody's knowledge: everyone gets what a founder of their age would
    // bring, and each settlement's record of what it knows begins at the load (ADR-0008 §7).
    let sim = load_first();
    let sections: Vec<SectionData> = persist::encode_sections(&sim)
        .into_iter()
        .filter(|s| s.tag != agents::SECTION_KNOW)
        .collect();
    let mut info = fixture().first_info.clone();
    info.schema_version = persist::SCHEMA_V12;
    let path = republish("slice-l", &info, &sections);
    let mut migrated = persist::load(&path, content()).expect("a schema-12 save loads");
    assert!(migrated.is_dirty(), "the migration is new state");
    let (catalog, params) = (&migrated.rules().catalog, &migrated.rules().people);
    let now = migrated.now();
    for t in 0..catalog.techniques.len() {
        let founders = params.knowledge.founders.iter().any(|&(x, _)| x == t);
        let age = civ_agents::knowledge::knowing_age(catalog, t, params.family.independent_age)
            .expect("gates work");
        for (_, p) in migrated.people().people.iter() {
            assert_eq!(
                p.knows(t),
                founders && p.age_years(now) >= age,
                "{}",
                p.given
            );
        }
    }
    let settlements = migrated.land().settlements.len();
    let records = &migrated.people().knowledge;
    assert_eq!(records.len(), params.knowledge.founders.len() * settlements);
    assert!(records.iter().all(|e| e.at == now
        && e.kind
            == civ_agents::knowledge::KnowledgeEventKind::Known(
                civ_agents::person::KnowSource::Founder
            )));
    migrated
        .advance_minutes(24 * 60)
        .expect("a migrated world runs");
}

#[test]
fn slice_m_saves_load_with_nobody_having_tried() {
    // A schema-13 save has no times of trying: everyone loads as never having tried.
    let sim = load_first();
    let sections = persist::encode_sections(&sim);
    let mut info = fixture().first_info.clone();
    info.schema_version = persist::SCHEMA_V13;
    let path = republish("slice-m", &info, &sections);
    let loaded = persist::load(&path, content()).expect("a schema-13 save loads");
    assert!(loaded.is_dirty(), "the migration is new state");
    assert_eq!(loaded.people().living(), sim.people().living());
}

/// A raised granary of two bays, turned off the map's axes: a frame building of version 1 within
/// the core program's ranges.
fn granary(at: (i32, i32)) -> civ_grammar::BuildingSpec {
    use civ_grammar::frame_params as fp;
    let mut params = [0; civ_grammar::PARAMS];
    params[fp::EAVE_CM] = 180;
    params[fp::PITCH_CENTIDEG] = 4500;
    params[fp::BAYS] = 2;
    params[fp::DOOR] = fp::door(fp::SIDE_LEFT, 1);
    params[fp::JOIST_CM] = 15;
    params[fp::OVERHANG_CM] = 60;
    params[fp::FLOOR_RAISE_CM] = 80;
    params[fp::POST_CM] = 15;
    params[fp::WALL_CM] = 15;
    civ_grammar::BuildingSpec {
        program: "core:building/granary".into(),
        version: civ_grammar::FRAME_VERSION,
        footprint: civ_grammar::Footprint::Rect {
            x: at.0,
            y: at.1,
            length: 500,
            width: 300,
            angle: 11_000,
        },
        storeys: 1,
        params,
        materials: vec![
            "core:good/timber".into(),
            "core:good/timber".into(),
            "core:good/thatch".into(),
            "core:good/timber".into(),
        ],
        style_seed: 0,
    }
}

#[test]
fn frame_buildings_and_their_plots_survive_a_save_and_load_exactly() {
    let mut sim = load_first();
    let (household, home) = {
        let b = sim.land().buildings.first().expect("a home under way");
        (b.household, b.spec.footprint.centre())
    };
    let spec = granary((home.0 + 2_000, home.1 + 500));
    let catalog = &sim.rules().catalog;
    let def = &catalog.buildings[catalog
        .building_index("core:building/granary")
        .expect("the granary")];
    let e = civ_grammar::expand(&spec, &def.rules).expect("a valid granary");
    assert!(e.storage_kg[civ_grammar::storage::RAISED] > 0.0);
    let id = sim
        .place_building_for_tests(household, spec.clone(), Stage::ALL.len() as u8)
        .expect("placed");
    // Loading checks the land: the plot, the building and their ids must be consistent.
    let dir = scratch_dir("frames");
    let saved = persist::save(&mut sim, &dir, SaveKind::Manual, "granary").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    let b = loaded
        .land()
        .buildings
        .iter()
        .find(|b| b.id == id)
        .expect("the granary");
    assert_eq!(
        b.spec, spec,
        "the design, footprint and all sixteen parameters"
    );
    let plot = loaded
        .land()
        .plots
        .iter()
        .find(|p| p.id == b.plot)
        .expect("its plot");
    assert_eq!(plot.use_, civ_land::PlotUse::Store);
    assert_eq!(loaded.land().buildings, sim.land().buildings);
    assert_eq!(loaded.land().plots, sim.land().plots);
    // Saved again, every section is what it was.
    let mut again = loaded;
    let resaved = persist::save(&mut again, &dir, SaveKind::Manual, "again").expect("saves");
    assert_eq!(digests(&resaved.chunks), digests(&saved.chunks));
}

/// `loaded`, from a save made before buildings had a condition (schema 16 and earlier,
/// ADR-0009 §8), has the buildings `saved` as they were, but for condition: the stage under way
/// was worked at middling skill, and the groups of its finished stages are in place and sound
/// as of loading.
fn loaded_without_condition(loaded: &Sim, saved: &[civ_land::Building]) {
    let bare = |b: &civ_land::Building| civ_land::Building {
        skill_h: 0.0,
        condition: Vec::new(),
        state: civ_land::BuildingState::Standing,
        repair: None,
        ..b.clone()
    };
    let buildings = &loaded.land().buildings;
    assert_eq!(
        buildings.iter().map(bare).collect::<Vec<_>>(),
        saved.iter().map(bare).collect::<Vec<_>>()
    );
    let catalog = &loaded.rules().catalog;
    for b in buildings {
        assert_eq!(
            b.skill_h,
            b.work_h * civ_agents::condition::MIDDLING_SKILL as f32
        );
        assert_eq!(b.state, civ_land::BuildingState::Standing);
        assert!(b.repair.is_none());
        let def = &catalog.buildings[catalog
            .building_index(&b.spec.program)
            .expect("its program")];
        let e = civ_grammar::expand(&b.spec, &def.rules).expect("expands");
        let done = usize::from(b.stage);
        assert_eq!(
            b.condition.len(),
            e.groups.iter().filter(|g| g.stage.index() < done).count()
        );
        assert!(b.condition.iter().all(|c| c.loss == 0.0
            && c.state == civ_land::GroupState::Sound
            && c.installed == loaded.now()));
    }
}

/// The first saved world with a finished hut placed beside the first building's household's
/// home: the world, the hut and its expansion.
fn with_a_finished_hut() -> (Sim, civ_core::PermanentId, civ_grammar::Expansion) {
    let mut sim = load_first();
    let (household, home) = {
        let b = sim.land().buildings.first().expect("a home under way");
        (b.household, b.spec.footprint.centre())
    };
    let catalog = sim.rules().catalog.clone();
    let def = &catalog.buildings[catalog
        .building_index("core:building/hut")
        .expect("the hut")];
    let spec = civ_agents::build::design_shape(
        def,
        &catalog.goods,
        civ_agents::build::Shape::Round { radius: 300 },
        (home.0 as f32 / 100.0 + 25.0, home.1 as f32 / 100.0),
        None,
    )
    .expect("a hut");
    let e = civ_grammar::expand(&spec, &def.rules).expect("expands");
    let id = sim
        .place_building_for_tests(household, spec, Stage::ALL.len() as u8)
        .expect("placed");
    (sim, id, e)
}

#[test]
fn a_buildings_condition_and_its_upkeep_survive_a_save_exactly() {
    // Schema 17 (ADR-0009 §4): each group's quality, loss and state, the building's state, the
    // skill its builders brought to the stage under way, and the upkeep under way.
    let (mut sim, hut, e) = with_a_finished_hut();
    let covering = e
        .groups
        .iter()
        .find(|g| g.kind == civ_grammar::GroupKind::Covering)
        .map(|g| g.id)
        .expect("a covering");
    let at = sim.now();
    for b in sim.land_mut_for_tests().buildings.iter_mut() {
        if b.id == hut {
            for c in &mut b.condition {
                c.loss = if c.group == covering { 0.4 } else { 0.05 };
                c.state = if c.group == covering {
                    civ_land::GroupState::Symptom
                } else {
                    civ_land::GroupState::Sound
                };
                c.repaired = at;
            }
            b.repair = Some(civ_land::Repair {
                group: covering,
                share: 0.4,
                work_h: 3.5,
            });
        } else if !b.finished() {
            b.skill_h = b.work_h * 0.75;
        }
    }
    let dir = scratch_dir("condition");
    let saved = persist::save(&mut sim, &dir, SaveKind::Manual, "condition").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loaded.land().buildings, sim.land().buildings);
    let mut again = loaded;
    let resaved = persist::save(&mut again, &dir, SaveKind::Manual, "again").expect("saves");
    assert_eq!(digests(&resaved.chunks), digests(&saved.chunks));
    // Wire 1.17: the map hears how it stands, what it shows and what is being mended.
    let payload = frames::buildings::buildings_response(&sim);
    let response = flatbuffers::root::<wire::Response>(&payload).expect("a response");
    let info = response
        .body_as_buildings()
        .expect("buildings")
        .buildings()
        .expect("a list")
        .iter()
        .find(|i| i.id() == hut.get())
        .expect("the hut");
    assert_eq!(info.state(), wire::BuildingState::Standing);
    let symptoms = info.symptoms().unwrap_or_default();
    assert_eq!(symptoms, "the thatch leaks");
    let u = &sim.rules().catalog.buildings[sim
        .rules()
        .catalog
        .building_index("core:building/hut")
        .expect("the hut")]
    .upkeep;
    let leak = (0.4 - u.covering.shows_at) / (1.0 - u.covering.shows_at);
    assert!(
        (f64::from(info.leak()) - leak).abs() < 1e-5,
        "{}",
        info.leak()
    );
    let groups = info.groups().expect("its groups");
    assert_eq!(groups.len(), e.groups.len());
    let c = groups
        .iter()
        .find(|g| g.id() == covering)
        .expect("the covering");
    assert_eq!(c.kind(), Some("covering"));
    assert_eq!(c.state(), wire::GroupState::Symptom);
    assert!((c.loss() - 0.4).abs() < 1e-6);
    let upkeep = info.upkeep().unwrap_or_default();
    assert!(upkeep.starts_with("mending the covering, "), "{upkeep}");
}

#[test]
fn slice_o_third_step_saves_load_with_their_buildings_sound() {
    // A schema-16 save: no building has a condition. Each finished stage's groups are put in
    // place sound as it loads, as if built at middling skill (ADR-0009 §8).
    let (sim, hut, e) = with_a_finished_hut();
    let sections = persist::encode_sections(&sim);
    let mut info = fixture().first_info.clone();
    info.schema_version = persist::SCHEMA_V16;
    let path = republish("slice-o-3", &info, &sections);
    let loaded = persist::load(&path, content()).expect("a schema-16 save loads");
    loaded_without_condition(&loaded, &sim.land().buildings);
    let b = loaded
        .land()
        .buildings
        .iter()
        .find(|b| b.id == hut)
        .expect("the hut");
    assert_eq!(b.condition.len(), e.groups.len());
    assert!(b.roofed());
    let h = loaded
        .people()
        .household(b.household)
        .expect("its household");
    assert!(h.sheltered && h.keeping.roofed_kg > 0.0);
}

#[test]
fn slice_n_saves_load_with_their_huts_as_they_were() {
    // A schema-14 save has round footprints and eight parameters a design: they load as huts of
    // version 1 with the other eight parameters 0.
    let sim = load_first();
    let sections = persist::encode_sections(&sim);
    let mut info = fixture().first_info.clone();
    info.schema_version = persist::SCHEMA_V14;
    let path = republish("slice-n", &info, &sections);
    let loaded = persist::load(&path, content()).expect("a schema-14 save loads");
    loaded_without_condition(&loaded, &fixture().first_buildings);
    assert!(
        loaded
            .land()
            .buildings
            .iter()
            .all(|b| b.spec.version == civ_grammar::HUT_VERSION
                && b.spec.params[8..].iter().all(|&p| p == 0))
    );
}

#[test]
fn a_workshop_keeps_its_firm_and_the_firm_its_busiest_hour_across_a_save() {
    // Schema 16 (ADR-0009 §7): a workshop names the firm it was built for, and a firm the most
    // people it had at work at once lately.
    let mut sim = load_first();
    let (household, home) = {
        let b = sim.land().buildings.first().expect("a home under way");
        (b.household, civ_agents::build::centre_m(&b.spec))
    };
    let catalog = sim.rules().catalog.clone();
    let (now, id) = (sim.now(), sim.allocate_id_for_tests());
    let founder = sim.people().household(household).expect("it").members[0];
    let sickle = catalog.good_index("core:good/sickle").expect("the sickle") as u16;
    let mut firm = civ_agents::firm::Firm::new(
        id,
        household,
        founder,
        None,
        sickle,
        catalog.goods.len(),
        now,
    );
    firm.saw_at_once(3, now.day_index());
    sim.people_mut_for_tests().firms.push(firm);
    let def = &catalog.buildings[catalog
        .building_index("core:building/workshop")
        .expect("the workshop")];
    let shape = civ_agents::build::Shape::Bays {
        bays: 2,
        storeys: 1,
        lofts: 1,
    };
    let spec = civ_agents::build::design_shape(
        def,
        &catalog.goods,
        shape,
        (home.0 + 20.0, home.1 + 5.0),
        Some(home),
    )
    .expect("designed");
    let shop = sim
        .place_building_for_tests(household, spec, Stage::ALL.len() as u8)
        .expect("placed");
    for b in sim.land_mut_for_tests().buildings.iter_mut() {
        if b.id == shop {
            b.firm = Some(id);
        }
    }
    let dir = scratch_dir("workshops");
    let saved = persist::save(&mut sim, &dir, SaveKind::Manual, "workshop").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loaded.land().buildings, sim.land().buildings);
    let f = loaded.people().firm(id).expect("the firm");
    assert_eq!((f.most_at_once, f.most_at_once_day), (3, now.day_index()));
    let mut again = loaded;
    let resaved = persist::save(&mut again, &dir, SaveKind::Manual, "again").expect("saves");
    assert_eq!(digests(&resaved.chunks), digests(&saved.chunks));
    // Wire 1.16: the map names the workshop's firm.
    let payload = frames::buildings::buildings_response(&sim);
    let response = flatbuffers::root::<wire::Response>(&payload).expect("a response");
    let info = response
        .body_as_buildings()
        .expect("buildings")
        .buildings()
        .expect("a list")
        .iter()
        .find(|i| i.id() == shop.get())
        .expect("the workshop");
    assert_eq!(info.firm(), id.get());
    let name = info.firm_name().unwrap_or_default();
    assert!(name.ends_with("'s sickle workshop"), "{name}");
    assert!(info.work_places() >= 3);
    // Its floor and room count beside the home's in the household's wealth.
    let w = sim
        .people()
        .wealth(
            &sim.rules().catalog,
            &sim.rules().people,
            &sim.rules().land,
            sim.land(),
            sim.now(),
        )
        .into_iter()
        .find(|w| w.household == household)
        .expect("its measures");
    let e = civ_grammar::expand(&info_spec(&sim, shop), &def.rules).expect("expands");
    assert!(w.roofed_m2 >= w.floor_m2 + e.floor_area_m2 - 1e-6, "{w:?}");
    assert!(w.storage_kg >= e.storage_total_kg() - 1e-6, "{w:?}");
}

/// The design of building `id`.
fn info_spec(sim: &Sim, id: civ_core::PermanentId) -> civ_grammar::BuildingSpec {
    sim.land()
        .buildings
        .iter()
        .find(|b| b.id == id)
        .map(|b| b.spec.clone())
        .expect("the building")
}

#[test]
fn slice_o_saves_load_with_no_workshop_naming_a_firm() {
    // A schema-15 save: no building names a firm, and nobody has been counted at work.
    let sim = load_first();
    let sections = persist::encode_sections(&sim);
    let mut info = fixture().first_info.clone();
    info.schema_version = persist::SCHEMA_V15;
    let path = republish("slice-o", &info, &sections);
    let loaded = persist::load(&path, content()).expect("a schema-15 save loads");
    loaded_without_condition(&loaded, &fixture().first_buildings);
    assert!(loaded.land().buildings.iter().all(|b| b.firm.is_none()));
}

#[test]
fn a_session_of_trying_survives_a_save_and_load() {
    let mut sim = load_first();
    let drying = sim
        .rules()
        .catalog
        .technique_index("core:technique/drying")
        .expect("drying");
    let now = sim.now();
    let someone = sim
        .people()
        .people
        .iter()
        .map(|(_, p)| p.id)
        .min()
        .expect("someone");
    for (_, p) in sim.people_mut_for_tests().people.iter_mut() {
        if p.id == someone {
            p.act.target = civ_agents::person::Target::Technique(drying as u16);
            p.tried = Some(now);
        }
    }
    let dir = scratch_dir("trying");
    let saved = persist::save(&mut sim, &dir, SaveKind::Manual, "trying").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    let p = loaded.people().person(someone).expect("alive");
    assert_eq!(
        p.act.target,
        civ_agents::person::Target::Technique(drying as u16)
    );
    assert_eq!(p.tried, Some(now));
    // Nobody else has tried.
    assert!(
        loaded
            .people()
            .people
            .iter()
            .all(|(_, q)| q.id == someone || q.tried.is_none())
    );
}

#[test]
fn a_save_without_its_knowledge_section_is_refused() {
    let sim = load_first();
    let sections: Vec<SectionData> = persist::encode_sections(&sim)
        .into_iter()
        .filter(|s| s.tag != agents::SECTION_KNOW)
        .collect();
    let path = republish("no-know", &fixture().first_info, &sections);
    assert!(persist::load(&path, content()).is_err());
}

#[test]
fn what_people_know_survives_a_save_and_load() {
    let mut sim = load_first();
    let young = sim
        .people()
        .people
        .iter()
        .map(|(_, p)| p)
        .filter(|p| p.knows.is_empty())
        .map(|p| p.id)
        .min()
        .expect("a child who knows nothing yet");
    sim.introduce_technique(young, "core:technique/knapping", true)
        .expect("heard of");
    let dir = scratch_dir("knows");
    let saved = persist::save(&mut sim, &dir, SaveKind::Manual, "knows").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    for (_, p) in sim.people().people.iter() {
        assert_eq!(
            loaded.people().person(p.id).map(|q| &q.knows),
            Some(&p.knows)
        );
    }
    assert_eq!(loaded.people().knowledge, sim.people().knowledge);
}

#[test]
fn a_save_without_its_firms_section_is_refused() {
    let sim = load_first();
    let sections: Vec<SectionData> = persist::encode_sections(&sim)
        .into_iter()
        .filter(|s| s.tag != agents::SECTION_FIRMS)
        .collect();
    let path = republish("no-firms", &fixture().first_info, &sections);
    assert!(persist::load(&path, content()).is_err());
}

#[test]
fn a_save_without_a_wealth_history_loads_with_none() {
    // The wealth section came partway through schema 12; its history is a measure, not state.
    let mut sim = load_first();
    let settlement = sim.land().settlements[0].id;
    let rules = sim.rules().clone();
    let (land, now) = (sim.land().clone(), sim.now());
    sim.people_mut_for_tests().record_wealth(
        &rules.catalog,
        &rules.people,
        &rules.land,
        &land,
        now,
        1,
    );
    assert_eq!(sim.people().wealth_years.len(), 1);
    assert_eq!(sim.people().wealth_years[0].spread.settlement, settlement);
    let sections: Vec<SectionData> = persist::encode_sections(&sim)
        .into_iter()
        .filter(|s| s.tag != agents::SECTION_WEALTH)
        .collect();
    let path = republish("no-wealth", &fixture().first_info, &sections);
    let loaded = persist::load(&path, content()).expect("loads");
    assert!(loaded.people().wealth_years.is_empty());
    assert_eq!(loaded.people().living(), sim.people().living());
}

#[test]
fn a_save_without_its_market_section_is_refused() {
    let sim = load_first();
    let sections: Vec<SectionData> = persist::encode_sections(&sim)
        .into_iter()
        .filter(|s| s.tag != agents::SECTION_MARKET)
        .collect();
    let path = republish("no-market", &fixture().first_info, &sections);
    assert!(persist::load(&path, content()).is_err());
}

#[test]
fn a_family_the_observer_sent_survives_a_save_and_load() {
    let mut sim = load_first();
    let hearth = sim.land().settlements[0].hearth_m;
    let sent = sim.spawn_family(hearth).expect("a family arrives");
    let dir = scratch_dir("sent");
    let saved = persist::save(&mut sim, &dir, SaveKind::Manual, "sent").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    let entry = loaded
        .people()
        .chronicle
        .iter()
        .find(|e| e.kind == civ_agents::ChronicleKind::FamilyArrived)
        .expect("the chronicle keeps it");
    assert_eq!(entry.people, sent.people);
    for id in &sent.people {
        assert_eq!(
            loaded.people().records[id].origin,
            civ_agents::Origin::Spawned
        );
    }
}

#[test]
fn a_save_without_its_wear_section_is_refused() {
    let sim = load_first();
    let sections: Vec<SectionData> = persist::encode_sections(&sim)
        .into_iter()
        .filter(|s| s.tag != agents::SECTION_WEAR)
        .collect();
    let path = republish("no-wear", &fixture().first_info, &sections);
    assert!(persist::load(&path, content()).is_err());
}

#[test]
fn worn_ground_and_its_trails_survive_a_save_and_load() {
    let mut sim = Sim::create(
        &NewWorld {
            name: "Trodden".to_owned(),
            seed: 5,
            preset_id: PRESET.to_owned(),
            size_cells: SIDE,
            band_size: 0,
            neighbours: Vec::new(),
            neighbours_known: false,
            regime_id: String::new(),
        },
        content(),
        &mut |_| {},
        &AtomicBool::new(false),
    )
    .expect("generates");
    // Past two monthly surveys.
    sim.advance_minutes(70 * 24 * 60).expect("advances");
    let dir = scratch_dir("trodden");
    let saved = persist::save(&mut sim, &dir, SaveKind::Manual, "trodden").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    let day = sim.now().day_index();
    let params = &sim.rules().land.paths;
    let before = sim.land().wear.current(day, params);
    let after = loaded.land().wear.current(day, params);
    assert!(!before.is_empty(), "the band wore the ground");
    assert_eq!(before.len(), after.len());
    for (a, b) in before.iter().zip(&after) {
        assert_eq!(a.index, b.index);
        assert_eq!(a.trail, b.trail, "trail states are kept as saved");
        let worst = a
            .wear
            .iter()
            .zip(&b.wear)
            .map(|(x, y)| (x - y).abs())
            .fold(0.0f32, f32::max);
        assert!(worst <= 1.0 / 65535.0, "wear kept to 16 bits: {worst}");
    }
    let trail_cells: u32 = after
        .iter()
        .map(|t| t.trail.iter().map(|w| w.count_ones()).sum::<u32>())
        .sum();
    assert!(trail_cells > 0, "trails were worn in 70 days");
    assert!(
        !loaded.land().wear.trails().is_empty(),
        "and are traced again on load"
    );
    assert!(loaded.land().wear.max_factor() > 0.0, "routes see them");
}

#[test]
fn a_save_without_its_builds_section_is_refused() {
    let sim = load_first();
    let sections: Vec<SectionData> = persist::encode_sections(&sim)
        .into_iter()
        .filter(|s| s.tag != agents::SECTION_BUILDS)
        .collect();
    let path = republish("no-builds", &fixture().first_info, &sections);
    assert!(persist::load(&path, content()).is_err());
}

#[test]
fn slice_c_saves_load_without_buildings_and_build_again() {
    let sim = load_first();
    let sections: Vec<SectionData> = persist::encode_sections(&sim)
        .into_iter()
        .filter(|s| s.tag != agents::SECTION_PLOTS && s.tag != agents::SECTION_BUILDS)
        .collect();
    let mut info = fixture().first_info.clone();
    info.schema_version = persist::SCHEMA_V4;
    let path = republish("slice-c", &info, &sections);
    let mut migrated = persist::load(&path, content()).expect("a schema-4 save loads");
    assert!(migrated.is_dirty(), "the migration is new state");
    assert!(migrated.land().buildings.is_empty() && migrated.land().plots.is_empty());
    assert_eq!(migrated.land().fields, fixture().first_fields);
    // Households begin their homes again within days: the fixture's world has a fresh identity
    // each run, and on some its first day goes to other work.
    for _ in 0..30 {
        migrated
            .advance_minutes(24 * 60)
            .expect("a migrated world runs");
        if !migrated.land().buildings.is_empty() {
            break;
        }
    }
    assert!(
        !migrated.land().buildings.is_empty(),
        "households begin their homes again"
    );
}

#[test]
fn slice_b_saves_load_without_fields_and_farm_again() {
    let sim = load_first();
    let sections: Vec<SectionData> = persist::encode_sections(&sim)
        .into_iter()
        .filter(|s| {
            ![
                agents::SECTION_FIELDS,
                agents::SECTION_PLOTS,
                agents::SECTION_BUILDS,
            ]
            .contains(&s.tag)
        })
        .collect();
    let mut info = fixture().first_info.clone();
    info.schema_version = persist::SCHEMA_V3;
    let path = republish("slice-b", &info, &sections);
    let mut migrated = persist::load(&path, content()).expect("a schema-3 save loads");
    assert!(migrated.is_dirty(), "the migration is new state");
    assert!(migrated.land().fields.is_empty());
    // In March they mark out fields again, on the first days the ground can be worked: rain,
    // snow lying or frost keep them off it (ADR-0012 §5).
    for _ in 0..30 {
        migrated
            .advance_minutes(24 * 60)
            .expect("a migrated world runs");
        if !migrated.land().fields.is_empty() {
            break;
        }
    }
    assert!(
        !migrated.land().fields.is_empty(),
        "in March they mark out fields again"
    );
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
    let (map, stats, ground) = (sim.map(), sim.stats(), &sim.land().ground);
    let query = |layer, level, x0, y0, width, height| RasterQuery {
        layer,
        level,
        x0,
        y0,
        width,
        height,
    };

    // Full resolution: dequantised elevation is within half a quantum of the ground in use.
    let bytes = frames::raster_response(
        map,
        ground,
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
            let cell = (20 + y) * SIDE as usize + 10 + x;
            let truth = map.elevation[cell] + ground.at(cell);
            assert!((value - truth).abs() <= half_quantum, "{value} vs {truth}");
        }
    }

    // A coarse level covers the map in fewer cells and is clipped at its edge.
    let bytes = frames::raster_response(
        map,
        ground,
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
        let bytes = frames::raster_response(map, ground, stats, &query(layer, 1, 0, 0, 64, 64))
            .expect("answers");
        let tile = decode_tile(&bytes);
        assert_eq!(tile.data().expect("data").len(), 64 * 64 * 4);
    }

    // Out of range and oversized regions are refused.
    let off_map = query(wire::RasterLayer::Elevation, 0, SIDE, 0, 1, 1);
    assert!(frames::raster_response(map, ground, stats, &off_map).is_err());
    let empty = query(wire::RasterLayer::Elevation, 0, 0, 0, 0, 10);
    assert!(frames::raster_response(map, ground, stats, &empty).is_err());
    let too_coarse = query(
        wire::RasterLayer::Elevation,
        frames::MAX_LEVEL + 1,
        0,
        0,
        1,
        1,
    );
    assert!(frames::raster_response(map, ground, stats, &too_coarse).is_err());

    // Elevation is the ground in use: a metre of earth taken from one cell lowers it there, at
    // full resolution and in the mean of a coarser level, and nowhere else.
    let mut dug = civ_land::earth::GroundDelta::new(map.width, map.height, map.cell_size_m);
    let cell_m = f64::from(map.cell_size_m);
    dug.add((30.5 * cell_m, 40.5 * cell_m), -cell_m * cell_m);
    let read = |ground: &civ_land::earth::GroundDelta, level: u8, x: u32, y: u32| {
        let bytes = frames::raster_response(
            map,
            ground,
            stats,
            &query(wire::RasterLayer::Elevation, level, x, y, 1, 1),
        )
        .expect("answers");
        let tile = decode_tile(&bytes);
        let data = tile.data().expect("data").bytes();
        f32::from(u16::from_le_bytes([data[0], data[1]])) * tile.scale() + tile.offset()
    };
    let quantum = decode_tile(
        &frames::raster_response(
            map,
            ground,
            stats,
            &query(wire::RasterLayer::Elevation, 0, 0, 0, 1, 1),
        )
        .expect("answers"),
    )
    .scale();
    assert!((read(ground, 0, 30, 40) - read(&dug, 0, 30, 40) - 1.0).abs() <= quantum + 1e-3);
    assert!((read(ground, 1, 15, 20) - read(&dug, 1, 15, 20) - 0.25).abs() <= quantum + 1e-3);
    assert!((read(ground, 0, 31, 40) - read(&dug, 0, 31, 40)).abs() <= 1e-6);
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
fn the_worn_ground_and_trails_are_described_as_surveyed() {
    let mut sim = load_first();
    // A save keeps the routing view people plan on: the fixture's three days of walking wore
    // the ground, but its paths were last surveyed when the world was made, before anyone walked.
    assert!(!sim.land().wear.tiles().is_empty());
    assert_eq!(sim.land().wear.surveyed_tiles().count(), 0);
    // Surveyed now, as the month's turn would.
    let (day, params) = (sim.now().day_index(), sim.rules().land.paths);
    sim.land_mut_for_tests().wear.survey(day, &params);
    let payload = frames::paths::paths_response(&sim);
    let response = flatbuffers::root::<wire::Response>(&payload).expect("a response");
    let paths = response.body_as_paths().expect("paths");
    assert_eq!(paths.rev(), frames::paths::paths_rev(&sim));
    assert_ne!(paths.rev(), 0, "a loaded world has been surveyed");
    assert_eq!(paths.tiles_x(), SIDE.div_ceil(64));
    assert_eq!(paths.tile_cells(), 64);
    let worn = paths.worn().expect("worn tiles");
    let wear = &sim.land().wear;
    // What was surveyed, not what has been walked since: the routing view as saved.
    assert_eq!(worn.len(), wear.surveyed_tiles().count());
    assert!(!worn.is_empty(), "three days of walking wore the ground");
    for (tile, (index, cells, trail)) in worn.iter().zip(wear.surveyed_tiles()) {
        assert_eq!(tile.index(), index);
        assert_eq!(tile.wear().expect("wear").bytes(), cells);
        assert_eq!(tile.trail().expect("trail").len(), trail.len());
    }
    let trails = paths.trails().expect("trails");
    assert_eq!(trails.len(), wear.trails().len());
    for (info, t) in trails.iter().zip(wear.trails()) {
        assert_eq!(info.points().expect("points").len(), t.points.len());
        assert!(info.length_m() > 0.0 && (0.0..=1.0).contains(&info.wear()));
    }
}

#[test]
fn every_building_is_described_with_its_expanded_shape() {
    let sim = load_first();
    let payload = frames::buildings::buildings_response(&sim);
    let response = flatbuffers::root::<wire::Response>(&payload).expect("a response");
    let list = response.body_as_buildings().expect("buildings");
    assert_eq!(list.rev(), frames::buildings::buildings_rev(&sim));
    assert_ne!(list.rev(), 0, "a world with buildings has a revision");
    let infos = list.buildings().expect("a list");
    assert_eq!(infos.len(), sim.land().buildings.len());
    for (info, b) in infos.iter().zip(&sim.land().buildings) {
        assert_eq!(info.id(), b.id.get());
        assert_eq!(info.program(), Some("Hut"));
        assert_eq!(info.stage(), b.stage);
        let centre = info.centre().expect("a centre");
        assert_eq!(
            (centre.x(), centre.y()),
            civ_agents::build::centre_m(&b.spec)
        );
        // The walls' outline goes round the centre, outside the wall line; the roof reaches past
        // it; the posts stand on the wall line.
        let outline = info.outline().expect("an outline");
        assert_eq!(outline.len(), 32);
        for p in outline.iter() {
            let r = (p.x() - centre.x()).hypot(p.y() - centre.y());
            assert!(r >= info.radius_m() && r < info.roof_radius_m(), "{r}");
        }
        let posts = info.posts().expect("posts");
        assert!(posts.len() >= 6);
        for p in posts.iter() {
            let r = (p.x() - centre.x()).hypot(p.y() - centre.y());
            assert!((r - info.radius_m()).abs() < 0.02, "{r}");
        }
        // It stands on its plot, and sleeps its household.
        let (min, size) = (
            info.plot_min().expect("plot"),
            info.plot_size().expect("plot"),
        );
        assert!(min.x() < centre.x() && centre.x() < min.x() + size.x());
        assert!(min.y() < centre.y() && centre.y() < min.y() + size.y());
        let members = sim
            .people()
            .household(b.household)
            .expect("household")
            .members
            .len();
        assert!(info.sleeps() as usize >= members);
        assert!(info.floor_m2() > 0.0);
        assert!(!info.status().unwrap_or_default().is_empty());
        // How it was built, in words (M3b slice R): a founder's hut follows no other.
        let style = info.style().unwrap_or_default();
        assert!(
            style.starts_with("roof pitched ") && style.contains(" m to the eaves"),
            "{style}"
        );
        assert_eq!(info.style_from(), 0);
        // Wire 1.14: a hut is a one-storey dwelling of the hut grammar, its cone seen as a
        // circle, all its floor for living.
        assert_eq!(info.grammar(), Some("hut"));
        assert_eq!(info.purpose(), Some("dwelling"));
        assert_eq!((info.storeys(), info.bays(), info.loft_bays()), (1, 1, 0));
        assert!(info.roof_outline().is_some_and(|r| r.is_empty()));
        assert!(info.ridge().is_some_and(|r| r.is_empty()));
        let by_use: Vec<f32> = info.floor_by_use().expect("floor by use").iter().collect();
        assert_eq!(by_use, vec![info.floor_m2(), 0.0, 0.0]);
        assert!(info.storage_kg().expect("storage").get(2) > 0.0);
        assert!(info.apex_m() > 2.0);
        // Nothing is kept in a hut before its roof is on.
        if !b.roofed() {
            assert!(info.stored().unwrap_or_default().is_empty());
        }
    }
}

#[test]
fn a_frame_building_is_described_with_its_gabled_roof_and_floors() {
    let mut sim = load_first();
    let (household, home) = {
        let b = sim.land().buildings.first().expect("a home under way");
        (b.household, b.spec.footprint.centre())
    };
    let spec = granary((home.0 + 2_000, home.1 + 500));
    let id = sim
        .place_building_for_tests(household, spec, Stage::ALL.len() as u8)
        .expect("placed");
    // Its only roof is the granary's, and its goods keep under it.
    let keeping = {
        let h = sim.people().household(household).expect("household");
        assert!(h.sheltered);
        h.keeping
    };
    assert_eq!((keeping.raised_kg, keeping.roofed_kg), (7_500.0, 0.0));
    let payload = frames::buildings::buildings_response(&sim);
    let response = flatbuffers::root::<wire::Response>(&payload).expect("a response");
    let list = response.body_as_buildings().expect("buildings");
    let info = list
        .buildings()
        .expect("a list")
        .iter()
        .find(|i| i.id() == id.get())
        .expect("the granary");
    assert_eq!(info.program(), Some("Granary"));
    assert_eq!(info.grammar(), Some("frame"));
    assert_eq!(info.purpose(), Some("store"));
    assert!(info.roofed());
    let size = info.size().expect("a size");
    assert_eq!((size.x(), size.y()), (5.0, 3.0));
    let turn = 11_000.0 / 65_536.0 * std::f32::consts::TAU;
    assert!((info.angle() - turn).abs() < 1e-5);
    assert_eq!((info.storeys(), info.bays(), info.loft_bays()), (1, 2, 0));
    // Four corners of wall and of roof, the roof's beyond the walls, and a ridge along the
    // length through the middle.
    let centre = info.centre().expect("a centre");
    let (outline, roof) = (
        info.outline().expect("walls"),
        info.roof_outline().expect("roof"),
    );
    assert_eq!((outline.len(), roof.len()), (4, 4));
    for (w, r) in outline.iter().zip(roof.iter()) {
        let d = |p: &wire::Vec2| (p.x() - centre.x()).hypot(p.y() - centre.y());
        assert!(d(r) > d(w), "the roof reaches past the walls");
    }
    let ridge = info.ridge().expect("a ridge");
    assert_eq!(ridge.len(), 2);
    let (a, b) = (ridge.get(0), ridge.get(1));
    assert!((((a.x() + b.x()) / 2.0) - centre.x()).abs() < 0.02);
    assert!((((a.y() + b.y()) / 2.0) - centre.y()).abs() < 0.02);
    // 6.2 m: the length and its overhang at both ends.
    assert!(((a.x() - b.x()).hypot(a.y() - b.y()) - 6.2).abs() < 0.02);
    // Its 15 m² are a store, raised and aired: no sleeping places, room for goods.
    assert!((info.floor_m2() - 15.0).abs() < 1e-4);
    let by_use: Vec<f32> = info.floor_by_use().expect("floor by use").iter().collect();
    assert_eq!(by_use, vec![0.0, 15.0, 0.0]);
    assert_eq!(info.sleeps(), 0);
    let storage = info.storage_kg().expect("storage");
    assert!(storage.get(0) > 0.0 && storage.get(1) == 0.0 && storage.get(2) == 0.0);
    // Its posts stand on the long walls at the three frame lines.
    assert_eq!(info.posts().expect("posts").len(), 6);
    let plot = (
        info.plot_min().expect("plot"),
        info.plot_size().expect("plot"),
    );
    for p in roof.iter() {
        assert!(p.x() >= plot.0.x() - 0.01 && p.x() <= plot.0.x() + plot.1.x() + 0.01);
        assert!(p.y() >= plot.0.y() - 0.01 && p.y() <= plot.0.y() + plot.1.y() + 0.01);
    }
    assert_eq!(info.status(), Some("finished"));
    // Wire 1.15: its raised floor takes the household's goods that keep better under a roof
    // first (its huts have no roof yet).
    let stored: Vec<f32> = info.stored_kg().expect("stored").iter().collect();
    assert!(
        stored[0] > 0.0 && stored[1] == 0.0 && stored[2] == 0.0,
        "{stored:?}"
    );
    let words = info.stored().unwrap_or_default();
    assert!(words.starts_with("raised floor: "), "{words}");
    assert!(words.contains("of 7.5 t, mostly"), "{words}");
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
    assert!(sim.set_speed(f32::NEG_INFINITY).is_err());
    assert!(sim.set_speed(civ_sim::MAX_PACED_SPEED * 2.0).is_err());
    // 60x, 600x and Max are Accelerated speeds (ADR-0011).
    for m in civ_sim::ACCELERATED_MULTIPLIERS {
        sim.set_speed(m * civ_sim::SPEED_1X)
            .expect("an Accelerated speed");
    }
    assert_eq!(sim.speed(), civ_sim::SPEED_MAX);
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
            neighbours: Vec::new(),
            neighbours_known: false,
            regime_id: String::new(),
        },
        content(),
        &mut |_| {},
        &AtomicBool::new(false),
    );
    assert!(matches!(result, Err(civ_sim::SimError::UnknownPreset(_))));
}

/// Rewrites the `houses` and `land` sections of a current save in their schema-2 shape: food in
/// kilocalories, patch memory without resources, and stocks without their goods and units (all
/// set to `stock`).
fn as_schema_2(sections: &mut [SectionData], food_kcal: f64, stock: f32) {
    for s in sections.iter_mut() {
        if s.tag == agents::SECTION_HOUSES {
            let root = flatbuffers::root::<save::Households>(&s.bytes).expect("houses decode");
            let mut fbb = FlatBufferBuilder::new();
            let list: Vec<_> = root
                .households()
                .expect("households")
                .iter()
                .map(|h| {
                    let members: Vec<u64> = h.members().expect("members").iter().collect();
                    let members = fbb.create_vector(&members);
                    let known = fbb.create_vector(&[save::KnownPatch::new(3, 1234.0, 2)]);
                    save::Household::create(
                        &mut fbb,
                        &save::HouseholdArgs {
                            id: h.id(),
                            members: Some(members),
                            home: h.home(),
                            settlement: h.settlement(),
                            food_kcal,
                            water_l: h.water_l(),
                            water_at: h.water_at(),
                            known: Some(known),
                            ..Default::default()
                        },
                    )
                })
                .collect();
            let list = fbb.create_vector(&list);
            let root = save::Households::create(
                &mut fbb,
                &save::HouseholdsArgs {
                    households: Some(list),
                    ..Default::default()
                },
            );
            fbb.finish(root, None);
            s.bytes = fbb.finished_data().to_vec();
        } else if s.tag == agents::SECTION_LAND {
            let l = flatbuffers::root::<save::Land>(&s.bytes).expect("land decodes");
            let mut fbb = FlatBufferBuilder::new();
            let habitats: Vec<_> = l
                .habitats()
                .expect("habitats")
                .iter()
                .map(|h| fbb.create_string(h))
                .collect();
            let habitats = fbb.create_vector(&habitats);
            let class = fbb.create_vector(l.class().expect("class").bytes());
            let richness: Vec<f32> = l.richness().expect("richness").iter().collect();
            let richness = fbb.create_vector(&richness);
            let resources: Vec<_> = l
                .resources()
                .expect("resources")
                .iter()
                .map(|r| fbb.create_string(r))
                .collect();
            let resources = fbb.create_vector(&resources);
            let stocks = vec![stock; l.stocks().expect("stocks").len()];
            let stocks = fbb.create_vector(&stocks);
            let root = save::Land::create(
                &mut fbb,
                &save::LandArgs {
                    cols: l.cols(),
                    rows: l.rows(),
                    patch_cells: l.patch_cells(),
                    cell_size_m: l.cell_size_m(),
                    habitats: Some(habitats),
                    class: Some(class),
                    richness: Some(richness),
                    resources: Some(resources),
                    stocks: Some(stocks),
                    stock_day: l.stock_day(),
                    climate_year: l.climate_year(),
                    climate_deviate: l.climate_deviate(),
                    climate_factor: l.climate_factor(),
                    ..Default::default()
                },
            );
            fbb.finish(root, None);
            s.bytes = fbb.finished_data().to_vec();
        }
    }
}

#[test]
fn slice_a_saves_load_with_their_food_as_provisions() {
    let sim = load_first();
    let mut sections = persist::encode_sections(&sim);
    as_schema_2(&mut sections, 30_000.0, 7.0);
    let mut info = fixture().first_info.clone();
    info.schema_version = persist::SCHEMA_V2;
    let path = republish("slice-a", &info, &sections);
    assert!(persist::describe(&path, content()).compatible);
    let mut migrated = persist::load(&path, content()).expect("a schema-2 save loads");
    assert!(migrated.is_dirty(), "the migration is new state");

    let rules = migrated.rules().clone();
    let provisions = rules.people.band.provisions_good;
    let kcal_per_kg = rules.catalog.goods[provisions].kcal_per_kg;
    assert!(!migrated.people().households.is_empty());
    for (_, h) in migrated.people().households.iter() {
        assert_eq!(h.stores.len(), rules.catalog.goods.len());
        assert!((h.stores[provisions] - 30_000.0 / kcal_per_kg).abs() < 1e-9);
        // Besides the provisions, only the tools a founder brings (save schema 9).
        let goods = &rules.catalog.goods;
        let held: f64 = h
            .stores
            .iter()
            .zip(goods)
            .filter(|(_, g)| g.tool.is_none())
            .map(|(kg, _)| kg)
            .sum();
        assert_eq!(held, h.stores[provisions]);
        let sickle = rules
            .catalog
            .good_index("core:good/sickle")
            .expect("sickles");
        assert!(h.stores[sickle] >= 1.0, "the household has sickles");
        assert_eq!(h.stores_at, migrated.now());
        assert!(h.known.is_empty(), "kilocalorie returns are forgotten");
    }
    // Stocks were not known to keep their units: every resource starts at equilibrium.
    let land = migrated.land();
    for r in 0..rules.land.resources.len() {
        for p in 0..land.patches.len() {
            let eq = land.equilibrium(&rules.land, r, p, land.stock_day) as f32;
            assert_eq!(land.stocks[r][p], eq, "resource {r}, patch {p}");
        }
    }
    migrated
        .advance_minutes(2 * 24 * 60)
        .expect("a migrated world runs");

    // Saved again, it is a current save that loads as it was.
    let saved = persist::save(
        &mut migrated,
        &scratch_dir("slice-a-again"),
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
    assert_eq!(reloaded.people().living(), migrated.people().living());
    assert_eq!(reloaded.land().stocks, migrated.land().stocks);
}

#[test]
fn stocks_carry_over_only_while_their_good_and_unit_are_unchanged() {
    let sim = load_first();
    let saved = sim.land().stocks.clone();
    let game = content().land.params.resource("game").expect("game");
    let mut changed = content().clone();
    changed.land.params.resources[game].unit_kg *= 2.0;
    changed.fingerprint[0] ^= 1;
    let loaded = persist::load(&fixture().first.path, &changed).expect("loads");
    let land = loaded.land();
    for (r, stock) in land.stocks.iter().enumerate() {
        if r == game {
            for (p, v) in stock.iter().enumerate() {
                let eq = land.equilibrium(&changed.land.params, r, p, land.stock_day) as f32;
                assert_eq!(*v, eq, "a game unit changed: patch {p} restarts");
            }
        } else {
            assert_eq!(*stock, saved[r], "resource {r} keeps its stocks");
        }
    }
}

#[test]
fn what_settlements_have_seen_of_their_buildings_survives_a_save_and_load() {
    let mut sim = load_first();
    let catalog = &sim.rules().catalog;
    let technique = catalog
        .technique_index("core:technique/jointed_frame")
        .expect("the jointed frame");
    let settlement = sim
        .people()
        .households
        .iter()
        .find_map(|(_, h)| h.settlement)
        .expect("a settlement");
    let now = sim.now();
    let seen = civ_agents::caution::Trust {
        settlement,
        technique: technique as u16,
        failures: 1.0,
        years: 12.0,
        at: now,
    };
    sim.people_mut_for_tests().trust.push(seen);
    let dir = scratch_dir("trust");
    let saved = persist::save(&mut sim, &dir, SaveKind::Manual, "trust").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loaded.people().trust, vec![seen]);
    // Wire 1.18: the knowledge panel says how much stronger its builders build, and why.
    let payload = frames::knowledge::knowledge_response(&loaded);
    let response = flatbuffers::root::<wire::Response>(&payload).expect("a response");
    let here = response
        .body_as_knowledge()
        .expect("knowledge")
        .settlements()
        .expect("a list")
        .iter()
        .find(|s| s.settlement() == settlement.get())
        .and_then(|s| s.techniques())
        .and_then(|ts| ts.iter().find(|t| usize::from(t.technique()) == technique))
        .expect("the jointed frame there");
    // One failure against 12 building-years is a rate of 0.083, so caution is 1.89.
    assert!((f64::from(here.caution()) - (1.0 + 1.0 / 12.0 / (1.0 / 12.0 + 0.01))).abs() < 1e-3);
    assert_eq!(
        here.trust(),
        Some("built 1.9 times as strong: failures weigh 1.0 against 12 building-years")
    );
    // A hut has nothing to size, so roundhouses are built as usual whatever was seen.
    let roundhouse = loaded
        .rules()
        .catalog
        .technique_index("core:technique/roundhouse")
        .expect("roundhouses");
    let mut sim = loaded;
    sim.people_mut_for_tests()
        .trust
        .push(civ_agents::caution::Trust {
            technique: roundhouse as u16,
            ..seen
        });
    let (caution, words) = frames::knowledge::trust_words(&sim, settlement, roundhouse);
    assert_eq!(caution, 1.0);
    assert_eq!(
        words,
        "built as usual, as a hut has nothing to size: failures weigh 1.0 against 12 building-years"
    );
}

#[test]
fn a_new_world_has_deposits_and_an_older_save_gains_the_same_on_loading() {
    let sim = load_first();
    let bodies = |s: &Sim| -> Vec<civ_land::deposits::Body> {
        s.land().deposits.iter().map(|d| d.body).collect()
    };
    // The core land profile lays down clay, stone and flint (ADR-0010 §1).
    let catalog = &sim.rules().catalog;
    let kinds: std::collections::BTreeSet<u16> = bodies(&sim).iter().map(|b| b.good).collect();
    assert!(!kinds.is_empty(), "the world has deposits");
    for id in ["core:good/clay", "core:good/stone", "core:good/toolstone"] {
        let g = catalog.good_index(id).expect("the good") as u16;
        assert!(kinds.contains(&g), "{id} lies somewhere");
    }
    // A schema-18 save, from before deposits were bodies, gains the bodies a new world of its
    // seed has, with ids of its own, and is changed by it.
    let sections: Vec<SectionData> = persist::encode_sections(&sim)
        .into_iter()
        .filter(|s| s.tag != agents::SECTION_DEPOSITS)
        .collect();
    let mut info = fixture().first_info.clone();
    info.schema_version = persist::SCHEMA_V18;
    let path = republish("slice-p", &info, &sections);
    let loaded = persist::load(&path, content()).expect("a schema-18 save loads");
    assert!(loaded.is_dirty(), "the migration is new state");
    assert_eq!(bodies(&loaded), bodies(&sim));
    // What has been taken survives a save and load.
    let mut sim = sim;
    sim.land_mut_for_tests().deposits[0].taken_kg = 125.0;
    let dir = scratch_dir("deposits");
    let saved = persist::save(&mut sim, &dir, SaveKind::Manual, "deposits").expect("saves");
    let again = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(again.land().deposits, sim.land().deposits);
}

#[test]
fn a_schema_39_save_loads_and_its_people_take_their_ideologies_the_next_midnight() {
    let sim = load_first();
    // A schema-39 save, from before ideologies (M4c slice AG, step four): nobody holds one when
    // it loads.
    let sections: Vec<SectionData> = persist::encode_sections(&sim)
        .into_iter()
        .filter(|s| s.tag != agents::SECTION_IDEOLOGIES)
        .collect();
    let mut info = fixture().first_info.clone();
    info.schema_version = persist::SCHEMA_V39;
    let path = republish("slice-ag3", &info, &sections);
    let mut loaded = persist::load(&path, content()).expect("a schema-39 save loads");
    assert!(loaded.people().ideologies.held.is_empty());
    // The next midnight everyone takes their start, as on the world's first night: each founder
    // brings what they brought then, by the same draw.
    loaded.advance_minutes(24 * 60).expect("advances");
    let brought = |s: &Sim| -> Vec<(civ_core::PermanentId, u16)> {
        s.people()
            .ideologies
            .held
            .iter()
            .filter(|h| h.from.is_none())
            .map(|h| (h.holder, h.ideology))
            .collect()
    };
    assert!(!brought(&sim).is_empty(), "some founders bring one");
    assert_eq!(brought(&loaded), brought(&sim));
    let last = loaded.people().people.iter().map(|(_, p)| p.id.get()).max();
    assert_eq!(Some(loaded.people().ideologies.seen), last);
}

#[test]
fn a_schema_40_save_loads_with_no_factions() {
    let sim = load_first();
    // A schema-40 save, from before factions (M4c slice AH): none have been founded.
    let sections: Vec<SectionData> = persist::encode_sections(&sim)
        .into_iter()
        .filter(|s| s.tag != agents::SECTION_FACTIONS)
        .collect();
    let mut info = fixture().first_info.clone();
    info.schema_version = persist::SCHEMA_V40;
    let path = republish("slice-ag4", &info, &sections);
    let mut loaded = persist::load(&path, content()).expect("a schema-40 save loads");
    assert!(loaded.people().factions.list.is_empty());
    assert!(loaded.people().factions.members.is_empty());
    loaded.advance_minutes(24 * 60).expect("goes on");
}

#[test]
fn a_schema_41_save_loads_with_no_petitions_and_no_law_replacing_another() {
    let sim = load_first();
    // A schema-41 save, from before petitions (M4c slice AH, step two): its factions section
    // carries none, and its laws name no law they replace.
    let mut info = fixture().first_info.clone();
    info.schema_version = persist::SCHEMA_V41;
    let path = republish("slice-ah1", &info, &persist::encode_sections(&sim));
    let mut loaded = persist::load(&path, content()).expect("a schema-41 save loads");
    assert!(loaded.people().factions.petitions.is_empty());
    assert!(
        loaded
            .people()
            .polities
            .iter()
            .flat_map(|p| &p.laws)
            .all(|l| l.ends.is_none())
    );
    loaded.advance_minutes(24 * 60).expect("goes on");
}

#[test]
fn a_schema_42_save_loads_with_no_refusals() {
    let sim = load_first();
    // A schema-42 save, from before refusals of a levy (M4c slice AH, step three): none were
    // called and no levy was kept back in one.
    let mut info = fixture().first_info.clone();
    info.schema_version = persist::SCHEMA_V42;
    let path = republish("slice-ah2", &info, &persist::encode_sections(&sim));
    let mut loaded = persist::load(&path, content()).expect("a schema-42 save loads");
    assert!(loaded.people().factions.refusals.is_empty());
    assert!(
        loaded
            .people()
            .polities
            .iter()
            .flat_map(|p| &p.laws)
            .all(|l| l.compliance.refused == 0)
    );
    loaded.advance_minutes(24 * 60).expect("goes on");
}

#[test]
fn a_schema_43_save_loads_with_no_revolts() {
    let sim = load_first();
    // A schema-43 save, from before revolts (M4c slice AI, step one): none were called and no
    // version of a custom was taken outside its procedure.
    let mut info = fixture().first_info.clone();
    info.schema_version = persist::SCHEMA_V43;
    let path = republish("slice-ah3", &info, &persist::encode_sections(&sim));
    let mut loaded = persist::load(&path, content()).expect("a schema-43 save loads");
    assert!(loaded.people().factions.revolts.is_empty());
    assert!(
        loaded
            .people()
            .polities
            .iter()
            .flat_map(|p| &p.versions)
            .all(|v| v.seized_by.is_none())
    );
    loaded.advance_minutes(24 * 60).expect("goes on");
}

#[test]
fn a_schema_44_save_loads_with_no_law_put_to_a_founding() {
    let sim = load_first();
    // A schema-44 save, from before repeals (M4c slice AI, step two): no law was put to a body
    // that took the deciding, and none was carried.
    let mut info = fixture().first_info.clone();
    info.schema_version = persist::SCHEMA_V44;
    let path = republish("slice-ai1", &info, &persist::encode_sections(&sim));
    let mut loaded = persist::load(&path, content()).expect("a schema-44 save loads");
    assert!(
        loaded
            .people()
            .polities
            .iter()
            .flat_map(|p| &p.laws)
            .all(|l| {
                l.issue != civ_agents::polity::IssueKind::Founding
                    && l.status != civ_agents::polity::LawStatus::Carried
            })
    );
    loaded.advance_minutes(24 * 60).expect("goes on");
}

#[test]
fn a_schema_45_save_loads_with_no_coups() {
    let sim = load_first();
    // A schema-45 save, from before coups (M4c slice AI, step three): none was called.
    let mut info = fixture().first_info.clone();
    info.schema_version = persist::SCHEMA_V45;
    let path = republish("slice-ai2", &info, &persist::encode_sections(&sim));
    let mut loaded = persist::load(&path, content()).expect("a schema-45 save loads");
    assert!(loaded.people().factions.coups.is_empty());
    loaded.advance_minutes(24 * 60).expect("goes on");
}

#[test]
fn a_schema_46_save_loads_with_no_encounters() {
    let sim = load_first();
    // A schema-46 save, from before force (M4c slice AI, step four): no watcher came to take what
    // a finding owed, and nobody was struck.
    let mut info = fixture().first_info.clone();
    info.schema_version = persist::SCHEMA_V46;
    let path = republish("slice-ai3", &info, &persist::encode_sections(&sim));
    let mut loaded = persist::load(&path, content()).expect("a schema-46 save loads");
    assert!(loaded.people().order.encounters.is_empty());
    loaded.advance_minutes(24 * 60).expect("goes on");
}

#[test]
fn a_schema_47_save_loads_with_no_influences() {
    let sim = load_first();
    // A schema-47 save, from before the observer's interventions (M4c slice AJ): the observer had
    // whispered to nobody and told nobody of an ideology.
    let mut info = fixture().first_info.clone();
    info.schema_version = persist::SCHEMA_V47;
    let path = republish("slice-ai4", &info, &persist::encode_sections(&sim));
    let mut loaded = persist::load(&path, content()).expect("a schema-47 save loads");
    assert!(loaded.people().influences.list.is_empty());
    loaded.advance_minutes(24 * 60).expect("goes on");
}

#[test]
fn a_schema_48_save_loads_with_no_one_blessed() {
    let sim = load_first();
    // A schema-48 save, from before agitators, blessings and curses (M4c slice AJ, step two).
    let mut info = fixture().first_info.clone();
    info.schema_version = persist::SCHEMA_V48;
    let path = republish("slice-aj1", &info, &persist::encode_sections(&sim));
    let mut loaded = persist::load(&path, content()).expect("a schema-48 save loads");
    let day = loaded.now().day_index();
    let pop = loaded.people();
    assert!(
        pop.people
            .iter()
            .all(|(_, p)| pop.influences.luck(p.id, day).is_none())
    );
    loaded.advance_minutes(24 * 60).expect("goes on");
}

#[test]
fn a_schema_56_save_loads_with_no_convergence_on_record() {
    // A schema-56 save, from before the convergence record (M5b slice AQ).
    let sim = load_first();
    assert!(sim.people().convergence.gaps.is_empty());
    let mut info = fixture().first_info.clone();
    info.schema_version = persist::SCHEMA_V56;
    let path = republish("slice-ap", &info, &persist::encode_sections(&sim));
    let mut loaded = persist::load(&path, content()).expect("a schema-56 save loads");
    assert!(loaded.people().convergence.gaps.is_empty());
    assert!(loaded.people().convergence.carried.is_empty());
    loaded.advance_minutes(24 * 60).expect("goes on");
}

#[test]
fn a_schema_55_save_loads_with_no_price_report_or_purchase_between_settlements() {
    // A schema-55 save, from before households bought from other settlements (M5b slice AP).
    let sim = load_first();
    assert!(sim.people().reports.held.is_empty());
    let mut info = fixture().first_info.clone();
    info.schema_version = persist::SCHEMA_V55;
    let path = republish("slice-ao", &info, &persist::encode_sections(&sim));
    let mut loaded = persist::load(&path, content()).expect("a schema-55 save loads");
    assert!(loaded.people().reports.held.is_empty());
    assert!(
        loaded
            .people()
            .contacts
            .years
            .values()
            .all(|c| c.bought == 0 && c.missed.iter().all(|&n| n == 0))
    );
    loaded.advance_minutes(24 * 60).expect("goes on");
}

#[test]
fn a_schema_54_save_loads_with_no_coalition_gathered() {
    // A schema-54 save, from before households gathered to found settlements (M5a slice AO).
    let sim = load_first();
    assert!(sim.people().coalitions.is_empty());
    let mut info = fixture().first_info.clone();
    info.schema_version = persist::SCHEMA_V54;
    let path = republish("slice-an2", &info, &persist::encode_sections(&sim));
    let mut loaded = persist::load(&path, content()).expect("a schema-54 save loads");
    assert!(loaded.people().coalitions.is_empty());
    loaded.advance_minutes(24 * 60).expect("goes on");
}

#[test]
fn a_schema_53_save_loads_with_no_wave_sent() {
    // A schema-53 save, from before migration waves (M5a slice AN, step two).
    let sim = load_first();
    assert!(sim.people().influences.waves.is_empty());
    let mut info = fixture().first_info.clone();
    info.schema_version = persist::SCHEMA_V53;
    let path = republish("slice-an1", &info, &persist::encode_sections(&sim));
    let mut loaded = persist::load(&path, content()).expect("a schema-53 save loads");
    assert!(loaded.people().influences.waves.is_empty());
    loaded.advance_minutes(24 * 60).expect("goes on");
}

#[test]
fn a_schema_52_save_loads_with_no_household_leaning_to_move() {
    // A schema-52 save, from before households moved between settlements (M5a slice AN): no
    // leanings and no reviews prompted were kept.
    let mut sim = load_first();
    sim.people_mut_for_tests().review_due.clear();
    let mut info = fixture().first_info.clone();
    info.schema_version = persist::SCHEMA_V52;
    let path = republish("slice-am3", &info, &persist::encode_sections(&sim));
    let mut loaded = persist::load(&path, content()).expect("a schema-52 save loads");
    assert!(loaded.people().leanings.is_empty());
    assert!(loaded.people().review_due.is_empty());
    loaded.advance_minutes(24 * 60).expect("goes on");
}

#[test]
fn a_schema_51_save_loads_with_no_visit_or_search_on_record() {
    // A schema-51 save, from before visits (M5a slice AM, step two): no contacts between
    // settlements and no failed search for a partner were kept.
    let mut sim = load_first();
    sim.people_mut_for_tests().unmatched.clear();
    let mut info = fixture().first_info.clone();
    info.schema_version = persist::SCHEMA_V51;
    let path = republish("slice-am1", &info, &persist::encode_sections(&sim));
    let mut loaded = persist::load(&path, content()).expect("a schema-51 save loads");
    assert!(loaded.people().contacts.years.is_empty());
    assert!(loaded.people().unmatched.is_empty());
    loaded.advance_minutes(24 * 60).expect("goes on");
}

#[test]
fn a_schema_50_save_loads_knowing_no_other_place() {
    // A schema-50 save, from before the places households know (M5a slice AM, ADR-0018 §4).
    let sim = load_first();
    let mut info = fixture().first_info.clone();
    info.schema_version = persist::SCHEMA_V50;
    let path = republish("slice-ak", &info, &persist::encode_sections(&sim));
    let mut loaded = persist::load(&path, content()).expect("a schema-50 save loads");
    assert!(loaded.people().known_places.known.is_empty());
    loaded.advance_minutes(24 * 60).expect("goes on");
}

#[test]
fn a_schema_49_save_loads_with_everyone_s_residence_inferred() {
    let mut sim = load_first();
    // A schema-49 save, from before residence histories (M5a slice AK, ADR-0018 §2): nobody's
    // stays were kept.
    for r in sim.people_mut_for_tests().records.values_mut() {
        r.residence.clear();
    }
    let mut info = fixture().first_info.clone();
    info.schema_version = persist::SCHEMA_V49;
    let path = republish("slice-aj2", &info, &persist::encode_sections(&sim));
    let mut loaded = persist::load(&path, content()).expect("a schema-49 save loads");
    let pop = loaded.people();
    assert!(
        pop.residence_problems().is_empty(),
        "{:?}",
        pop.residence_problems()
    );
    let village = loaded.land().settlements[0].id;
    for r in pop.records.values() {
        let first = r.residence.first().expect("a stay inferred");
        assert_eq!(first.settlement, Some(village));
        let why = match r.origin {
            civ_agents::Origin::Founder => civ_agents::ResidenceWhy::Founder,
            civ_agents::Origin::Born => civ_agents::ResidenceWhy::Born,
            civ_agents::Origin::Spawned => civ_agents::ResidenceWhy::Arrived,
        };
        assert_eq!(first.why, why);
    }
    loaded.advance_minutes(24 * 60).expect("goes on");
}

#[test]
fn slice_q_saves_load_with_each_household_s_taste_drawn_as_its_band_s() {
    let mut sim = load_first();
    // Tastes no band would bring, which the migration replaces.
    for (_, h) in sim.people_mut_for_tests().households.iter_mut() {
        h.taste = civ_agents::params::Taste::default();
    }
    // A schema-21 save, from before taste in building, gives each household the taste its band
    // would have brought (M3b slice R): one band to a settlement.
    let mut info = fixture().first_info.clone();
    info.schema_version = persist::SCHEMA_V21;
    let path = republish("slice-q", &info, &persist::encode_sections(&sim));
    let loaded = persist::load(&path, content()).expect("a schema-21 save loads");
    assert!(loaded.is_dirty(), "the migration is new state");
    let (style, seed) = (&loaded.rules().people.style, loaded.meta().seed);
    assert!(!loaded.people().households.is_empty());
    for (_, h) in loaded.people().households.iter() {
        let band = h.settlement.unwrap_or(h.id);
        assert_eq!(
            h.taste,
            civ_agents::style::founding_taste(style, seed, band, h.id),
            "{}",
            h.id
        );
    }
}
