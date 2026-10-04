//! Saving and loading worlds (ADR-0002).
//!
//! A save is one `commons-persist` generation in the world's own directory,
//! `<saves>/<world-slug>-<id8>/g##########-<kind>.tcesave`. Its sections are FlatBuffers tables
//! from `civ-schema/schema/tce_save.fbs`, all at encoding version [`SECTION_VERSION`]:
//!
//! | Section | Chunks | Holds |
//! |---|---|---|
//! | `meta` | 1 | name, seed, preset, generation parameters, map size, climate |
//! | `clock` | 1 | time, speed, pause, scheduler and permanent-id state |
//! | `content` | 1 | fingerprint and packs of the content the state was produced with |
//! | `hydro` | 1 | lakes, river reaches, inflows from beyond the map |
//! | `r-elev`, `r-recv`, `r-water`, `r-lake` | one per 512² tile | the authoritative rasters |
//! | `land`, `settle`, `people`, `houses`, `history`, `receipts`, `events`, `fields`, `plots`, `builds` | 1 each | land and people (see [`agents`]) |
//!
//! The world id, the snapshot's own id and its parent live in the container header. Drainage area,
//! the walking grid and the people's indexes are derived and rebuilt on load.
//!
//! **Schema versions.** Version 2 (M1) added the land and people sections; version 3 replaced
//! food counted in kilocalories with goods; version 4 added fields; version 5 added plots and
//! buildings; version 6 added couples, pregnancies and unions. A version-1 save (M0) is migrated
//! as it loads: its
//! land is classified and grown from the loaded content, exactly as for a new world, and it has no
//! people yet, which is a valid world (ADR-0003 §4). A version-2 save is migrated as
//! [`agents`] describes. A migrated world is written at the current version the next time it is
//! saved.
//!
//! Loading refuses, never repairs. A file that is incomplete or corrupt, comes from another engine
//! or an unknown schema version, or describes a world that breaks its invariants is not loaded. A
//! world saved with different content still loads; the difference is reported.

use std::fmt;
use std::io::{Read, Seek};
use std::path::{Path, PathBuf};

use std::sync::Arc;

use civ_agents::{AgentEvent, Population};
use civ_content::ContentRegistry;
use civ_core::{IdAllocator, Scheduler, SimTime};
use civ_land::Land;
use civ_schema::flatbuffers::{self, FlatBufferBuilder, InvalidFlatbuffer};
use civ_schema::{SAVE_ENGINE_TAG, SAVE_SCHEMA_VERSION, save};
use civ_world::{Climate, Inflow, Lake, RiverReach, Terminus, WorldMap};
use commons_persist::{
    Compression, LABEL_LEN, Limits, PersistError, Published, SaveDir, SaveKind, SectionData,
    SectionTag, SnapshotInfo, SnapshotReader,
};

use crate::{ContentStamp, MAX_SPEED, PHASE_AGENT, Rules, Sim, SimEvent, WorldMeta};

pub mod agents;

/// The schema version of M0 saves, which have no land or people sections.
pub const SCHEMA_V1: u32 = 1;
/// The schema version of M1 slice A saves, which count food in kilocalories (see [`agents`]).
pub const SCHEMA_V2: u32 = 2;
/// The schema version of M1 slice B saves, which have goods but no fields (see [`agents`]).
pub const SCHEMA_V3: u32 = 3;
/// The schema version of M1 slice C saves, which have fields but no plots or buildings (see
/// [`agents`]).
pub const SCHEMA_V4: u32 = 4;
/// The schema version of M1 slice D saves, which have buildings but no couples or pregnancies
/// (see [`agents`]).
pub const SCHEMA_V5: u32 = 5;
/// The schema version of M1 slice E saves, which have couples but no worn ground (see
/// [`agents`]).
pub const SCHEMA_V6: u32 = 6;
/// The schema version of M1 slice F saves, which have worn ground and no families sent by the
/// observer (see [`agents`]).
pub const SCHEMA_V7: u32 = 7;
/// The schema version of M1 slice G saves, with families the observer sent and no tools or
/// skills (see [`agents`]).
pub const SCHEMA_V8: u32 = 8;

/// Section: identity and provenance.
pub const SECTION_META: SectionTag = SectionTag::new("meta");
/// Section: clock and counters.
pub const SECTION_CLOCK: SectionTag = SectionTag::new("clock");
/// Section: the content the state was produced with.
pub const SECTION_CONTENT: SectionTag = SectionTag::new("content");
/// Section: lakes, reaches and inflows.
pub const SECTION_HYDRO: SectionTag = SectionTag::new("hydro");
/// Section: bed elevation tiles (f32).
pub const SECTION_ELEVATION: SectionTag = SectionTag::new("r-elev");
/// Section: receiver-code tiles (u8).
pub const SECTION_RECEIVERS: SectionTag = SectionTag::new("r-recv");
/// Section: water-class tiles (u8).
pub const SECTION_WATER: SectionTag = SectionTag::new("r-water");
/// Section: lake-id tiles (u32).
pub const SECTION_LAKE_ID: SectionTag = SectionTag::new("r-lake");

/// Encoding version of every section this build writes and reads.
pub const SECTION_VERSION: u32 = 1;
/// Raster tile side, in cells: an elevation tile is 1 MiB.
pub const TILE_CELLS: u32 = 512;
/// Largest map side a save may declare, in cells.
const MAX_SIDE: u32 = civ_world::MAX_CELLS;

/// Raster sections: tag, layer and bytes per sample.
const RASTERS: [(SectionTag, save::Layer, usize); 4] = [
    (SECTION_ELEVATION, save::Layer::Elevation, 4),
    (SECTION_RECEIVERS, save::Layer::Receivers, 1),
    (SECTION_WATER, save::Layer::Water, 1),
    (SECTION_LAKE_ID, save::Layer::LakeId, 4),
];

/// Why a save was not loaded.
#[derive(Debug)]
pub enum LoadError {
    /// The container is unreadable, incomplete or corrupt.
    Container(PersistError),
    /// The file comes from another engine, or uses a schema this build cannot read.
    Incompatible(String),
    /// A section is missing or malformed, or disagrees with another.
    Malformed(String),
    /// The world it holds breaks the world's invariants.
    Invalid(Vec<String>),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Container(e) => write!(f, "the save cannot be read: {e}"),
            LoadError::Incompatible(why) => write!(f, "the save is incompatible: {why}"),
            LoadError::Malformed(why) => write!(f, "the save is damaged: {why}"),
            LoadError::Invalid(problems) => write!(
                f,
                "the save holds an inconsistent world ({} problem(s)): {}",
                problems.len(),
                problems
                    .iter()
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        }
    }
}

impl std::error::Error for LoadError {}

impl From<PersistError> for LoadError {
    fn from(e: PersistError) -> Self {
        LoadError::Container(e)
    }
}

/// What the save browser shows about a file. Read from the header and the `meta` section only;
/// the rest of the file is verified when it is loaded.
#[derive(Clone, Debug, PartialEq)]
pub struct SaveSummary {
    /// Where the file is.
    pub path: PathBuf,
    /// Its size.
    pub file_len: u64,
    /// The header, when it could be read.
    pub info: Option<SnapshotInfo>,
    /// The world's name, when the `meta` section could be read.
    pub world_name: String,
    /// This build can load it (as far as the header and `meta` tell).
    pub compatible: bool,
    /// It was written with different content than is loaded now.
    pub content_changed: bool,
    /// Why it is not compatible, or a remark about it; empty when there is nothing to say.
    pub note: String,
}

/// Cuts a label to what a save header holds: control characters become spaces, and the text is
/// trimmed and cut to [`LABEL_LEN`] bytes.
pub fn clean_label(label: &str) -> String {
    let mut out = String::new();
    for c in label.trim().chars() {
        let c = if c.is_control() { ' ' } else { c };
        if out.len() + c.len_utf8() > LABEL_LEN {
            break;
        }
        out.push(c);
    }
    out.trim_end().to_owned()
}

/// Directory name for a world's saves: a slug of its name and the first 8 hex digits of its id,
/// for example `old-river-valley-3f9a12c0`.
pub fn world_dir_name(meta: &WorldMeta) -> String {
    let mut slug = String::new();
    for c in meta.name.chars() {
        if slug.len() >= 32 {
            break;
        }
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    let slug = slug.trim_end_matches('-');
    let slug = if slug.is_empty() { "world" } else { slug };
    format!("{slug}-{}", &meta.world_id_hex()[..8])
}

/// Writes `sim` as a new generation in `dir`, verifies the written file, and records it as the
/// state's last snapshot.
pub fn save(
    sim: &mut Sim,
    dir: &SaveDir,
    kind: SaveKind,
    label: &str,
) -> Result<Published, PersistError> {
    let mut info = SnapshotInfo::new(SAVE_ENGINE_TAG, SAVE_SCHEMA_VERSION, sim.meta.world_id);
    info.snapshot_id = commons_persist::random_id()?;
    info.parent_id = sim.last_snapshot;
    info.created_unix_ms = commons_persist::now_unix_ms();
    info.sim_time = sim.now().minutes();
    info.content_fingerprint = sim.content.fingerprint;
    info.label = clean_label(label);
    let sections = encode_sections(sim);
    let published = dir.publish(kind, &mut info, &sections)?;
    // The plan's autosave rule: temp file, fsync, rename, verify checksum (§3.5).
    SnapshotReader::open_file(&published.path, Limits::default())?.verify_all()?;
    sim.last_snapshot = Some(info.snapshot_id);
    sim.generation = info.generation;
    sim.dirty = false;
    Ok(published)
}

/// Every section of a save of `sim`, in file order.
pub fn encode_sections(sim: &Sim) -> Vec<SectionData> {
    let map = &sim.map;
    let mut sections = vec![
        section(SECTION_META, 0, encode_meta(sim)),
        section(SECTION_CLOCK, 0, encode_clock(sim)),
        section(SECTION_CONTENT, 0, encode_content(&sim.content)),
        section(SECTION_HYDRO, 0, encode_hydro(map)),
    ];
    for (tag, layer, _) in RASTERS {
        for (index, tile) in tiles(map.width, map.height).enumerate() {
            sections.push(section(tag, index as u32, encode_tile(map, layer, tile)));
        }
    }
    sections.extend(agents::encode(sim));
    sections
}

/// Loads a save, verifying all of it. The world comes back exactly as saved, including whether
/// its clock was running; deciding to pause it is the caller's business.
pub fn load(path: &Path, content: &ContentRegistry) -> Result<Sim, LoadError> {
    let mut reader = SnapshotReader::open_file(path, Limits::default())?;
    check_compatible(reader.info())?;
    reader.verify_body()?;
    let info = reader.info().clone();

    let meta_bytes = single_chunk(&mut reader, SECTION_META)?;
    let meta =
        flatbuffers::root::<save::Meta>(&meta_bytes).map_err(|e| unreadable(SECTION_META, &e))?;
    let (width, height) = (meta.width(), meta.height());
    if !(1..=MAX_SIDE).contains(&width) || !(1..=MAX_SIDE).contains(&height) {
        return Err(LoadError::Malformed(format!(
            "the map is {width}×{height} cells; sides must be 1 to {MAX_SIDE}"
        )));
    }
    let world_meta = WorldMeta {
        world_id: info.world_id,
        name: meta.world_name().unwrap_or_default().to_owned(),
        seed: meta.seed(),
        preset_id: meta.preset_id().unwrap_or_default().to_owned(),
        generator_version: meta.generator_version(),
        created_unix_ms: meta.created_unix_ms(),
        params: meta
            .params()
            .map(|params| {
                params
                    .iter()
                    .map(|p| (p.name().unwrap_or_default().to_owned(), p.value()))
                    .collect()
            })
            .unwrap_or_default(),
    };
    let climate = Climate {
        precipitation_mm_per_yr: meta.precipitation_mm_per_yr(),
        evapotranspiration_mm_per_yr: meta.evapotranspiration_mm_per_yr(),
        lake_evaporation_mm_per_yr: meta.lake_evaporation_mm_per_yr(),
    };
    let (cell_size_m, sea_level_m) = (meta.cell_size_m(), meta.sea_level_m());

    let clock_bytes = single_chunk(&mut reader, SECTION_CLOCK)?;
    let clock = flatbuffers::root::<save::Clock>(&clock_bytes)
        .map_err(|e| unreadable(SECTION_CLOCK, &e))?;
    if clock.minute() != info.sim_time {
        return Err(LoadError::Malformed(
            "the header's time disagrees with the clock section".to_owned(),
        ));
    }
    let tiebreak = clock
        .tiebreak_state()
        .filter(|v| v.len() == 4)
        .map(|v| [v.get(0), v.get(1), v.get(2), v.get(3)])
        .ok_or_else(|| LoadError::Malformed("the scheduler state is missing".to_owned()))?;
    let (clock_minute, scheduler_seq, paused) =
        (clock.minute(), clock.scheduler_seq(), clock.paused());
    let next_permanent_id = clock.next_permanent_id();
    let speed = clock.speed();
    if !(speed.is_finite() && speed > 0.0) {
        return Err(LoadError::Malformed(format!(
            "the clock speed {speed} is invalid"
        )));
    }
    // A speed is a setting, not world state: a save from a build with faster speeds still loads.
    let speed = speed.clamp(1.0, MAX_SPEED);

    let content_bytes = single_chunk(&mut reader, SECTION_CONTENT)?;
    let saved_content = flatbuffers::root::<save::Content>(&content_bytes)
        .map_err(|e| unreadable(SECTION_CONTENT, &e))?;
    let saved_fingerprint: [u8; 32] = saved_content
        .fingerprint()
        .and_then(|v| v.bytes().try_into().ok())
        .ok_or_else(|| LoadError::Malformed("the content fingerprint is missing".to_owned()))?;
    if saved_fingerprint != info.content_fingerprint {
        return Err(LoadError::Malformed(
            "the header's content fingerprint disagrees with the content section".to_owned(),
        ));
    }

    let hydro_bytes = single_chunk(&mut reader, SECTION_HYDRO)?;
    let (lakes, reaches, inflows) = decode_hydro(&hydro_bytes)?;

    let mut rasters = Vec::with_capacity(RASTERS.len());
    for (tag, layer, sample) in RASTERS {
        rasters.push(read_raster(&mut reader, tag, layer, sample, width, height)?);
    }
    let [elevation, receivers, water, lake_id]: [Vec<u8>; 4] = rasters
        .try_into()
        .map_err(|_| LoadError::Malformed("raster sections are missing".to_owned()))?;

    let mut map = WorldMap {
        width,
        height,
        cell_size_m,
        sea_level_m,
        elevation: elevation
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect(),
        receivers,
        water,
        lake_id: lake_id
            .chunks_exact(4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect(),
        lakes,
        reaches,
        inflows,
        climate,
        drainage_area_m2: Vec::new(),
    };
    let problems = civ_world::validate(&map);
    if !problems.is_empty() {
        return Err(LoadError::Invalid(problems));
    }
    map.rebuild_derived();

    let rules = Arc::new(Rules::of(content));
    let now = SimTime::from_minutes(clock_minute);
    let (land, people, events, redecide) = if info.schema_version == SCHEMA_V1 {
        // An M0 world: land as a new world would have it, and nobody yet.
        let land = Land::create(&map, &rules.land, world_meta.seed, now.day_index());
        (land, Population::new(), Vec::new(), Vec::new())
    } else {
        let d = agents::decode(
            &mut reader,
            &rules,
            &map,
            next_permanent_id,
            info.schema_version,
            now,
            world_meta.seed,
        )?;
        (d.land, d.people, d.events, d.redecide)
    };
    let mut scheduler = Scheduler::restore(now, scheduler_seq, tiebreak, events)
        .ok_or_else(|| LoadError::Malformed("the scheduler state is invalid".to_owned()))?;
    // A person whose activity the loaded content no longer has decides again in a minute. Their
    // pending event carries the old version and does nothing.
    let mut people = people;
    let mut changed = info.schema_version != SAVE_SCHEMA_VERSION;
    for who in redecide {
        if let Some(version) = people.restart(who) {
            let _ = scheduler.schedule(
                now.plus_minutes(1),
                PHASE_AGENT,
                SimEvent::Agent(AgentEvent::Step {
                    person: who,
                    version,
                }),
            );
            changed = true;
        }
    }

    let mut sim = Sim::assemble(
        world_meta,
        map,
        scheduler,
        IdAllocator::from_next(next_permanent_id),
        ContentStamp::of(content),
        rules,
        land,
        people,
    );
    sim.paused = paused;
    sim.speed = speed;
    sim.content_changed = saved_fingerprint != content.fingerprint;
    sim.last_snapshot = Some(info.snapshot_id);
    sim.generation = info.generation;
    sim.dirty = changed;
    Ok(sim)
}

/// Reads what the save browser shows about a file. Never fails: problems become a note on an
/// incompatible entry.
pub fn describe(path: &Path, content: &ContentRegistry) -> SaveSummary {
    let mut summary = SaveSummary {
        path: path.to_owned(),
        file_len: std::fs::metadata(path).map_or(0, |m| m.len()),
        info: None,
        world_name: String::new(),
        compatible: false,
        content_changed: false,
        note: String::new(),
    };
    let mut reader = match SnapshotReader::open_file(path, Limits::default()) {
        Ok(reader) => reader,
        Err(e) => {
            summary.note = LoadError::Container(e).to_string();
            return summary;
        }
    };
    summary.info = Some(reader.info().clone());
    summary.content_changed = reader.info().content_fingerprint != content.fingerprint;
    if let Err(e) = check_compatible(reader.info()) {
        summary.note = e.to_string();
        return summary;
    }
    let name = single_chunk(&mut reader, SECTION_META).and_then(|bytes| {
        flatbuffers::root::<save::Meta>(&bytes)
            .map(|meta| meta.world_name().unwrap_or_default().to_owned())
            .map_err(|e| unreadable(SECTION_META, &e))
    });
    match name {
        Ok(name) => {
            summary.world_name = name;
            summary.compatible = true;
            if summary.content_changed {
                summary.note = "made with different content than is loaded now".to_owned();
            }
        }
        Err(e) => summary.note = e.to_string(),
    }
    summary
}

/// The location of one raster tile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Tile {
    x0: u32,
    y0: u32,
    width: u32,
    height: u32,
}

/// The tiles of a map, row by row: chunk `i` of a raster section is tile `i`.
fn tiles(width: u32, height: u32) -> impl Iterator<Item = Tile> {
    let (nx, ny) = (width.div_ceil(TILE_CELLS), height.div_ceil(TILE_CELLS));
    (0..ny).flat_map(move |ty| {
        (0..nx).map(move |tx| {
            let (x0, y0) = (tx * TILE_CELLS, ty * TILE_CELLS);
            Tile {
                x0,
                y0,
                width: TILE_CELLS.min(width - x0),
                height: TILE_CELLS.min(height - y0),
            }
        })
    })
}

fn section(tag: SectionTag, index: u32, bytes: Vec<u8>) -> SectionData {
    SectionData::new(tag, index, SECTION_VERSION, bytes, Compression::Zstd)
}

fn finish<T>(mut fbb: FlatBufferBuilder<'_>, root: flatbuffers::WIPOffset<T>) -> Vec<u8> {
    fbb.finish(root, None);
    fbb.finished_data().to_vec()
}

fn encode_meta(sim: &Sim) -> Vec<u8> {
    let (meta, map) = (&sim.meta, &sim.map);
    let mut fbb = FlatBufferBuilder::new();
    let params: Vec<_> = meta
        .params
        .iter()
        .map(|(name, value)| {
            let name = fbb.create_string(name);
            save::Param::create(
                &mut fbb,
                &save::ParamArgs {
                    name: Some(name),
                    value: *value,
                },
            )
        })
        .collect();
    let params = fbb.create_vector(&params);
    let world_name = fbb.create_string(&meta.name);
    let preset_id = fbb.create_string(&meta.preset_id);
    let root = save::Meta::create(
        &mut fbb,
        &save::MetaArgs {
            world_name: Some(world_name),
            seed: meta.seed,
            preset_id: Some(preset_id),
            generator_version: meta.generator_version,
            width: map.width,
            height: map.height,
            cell_size_m: map.cell_size_m,
            sea_level_m: map.sea_level_m,
            created_unix_ms: meta.created_unix_ms,
            params: Some(params),
            precipitation_mm_per_yr: map.climate.precipitation_mm_per_yr,
            evapotranspiration_mm_per_yr: map.climate.evapotranspiration_mm_per_yr,
            lake_evaporation_mm_per_yr: map.climate.lake_evaporation_mm_per_yr,
        },
    );
    finish(fbb, root)
}

fn encode_clock(sim: &Sim) -> Vec<u8> {
    let (now, seq, tiebreak) = sim.scheduler.state();
    let mut fbb = FlatBufferBuilder::new();
    let tiebreak = fbb.create_vector(&tiebreak);
    let root = save::Clock::create(
        &mut fbb,
        &save::ClockArgs {
            minute: now.minutes(),
            paused: sim.paused,
            speed: sim.speed,
            scheduler_seq: seq,
            tiebreak_state: Some(tiebreak),
            next_permanent_id: sim.ids.peek_next(),
        },
    );
    finish(fbb, root)
}

fn encode_content(content: &ContentStamp) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let packs: Vec<_> = content
        .packs
        .iter()
        .map(|p| {
            let id = fbb.create_string(&p.id);
            let version = fbb.create_string(&p.version);
            let artifact = fbb.create_vector(&p.artifact_fingerprint);
            save::Pack::create(
                &mut fbb,
                &save::PackArgs {
                    id: Some(id),
                    version: Some(version),
                    artifact_fingerprint: Some(artifact),
                },
            )
        })
        .collect();
    let packs = fbb.create_vector(&packs);
    let fingerprint = fbb.create_vector(&content.fingerprint);
    let root = save::Content::create(
        &mut fbb,
        &save::ContentArgs {
            fingerprint: Some(fingerprint),
            packs: Some(packs),
        },
    );
    finish(fbb, root)
}

fn encode_hydro(map: &WorldMap) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let lakes: Vec<_> = map
        .lakes
        .iter()
        .map(|l| {
            save::Lake::create(
                &mut fbb,
                &save::LakeArgs {
                    id: l.id,
                    level_m: l.level_m,
                    spill_m: l.spill_m,
                    closed: l.closed,
                    cell_count: l.cell_count,
                    area_m2: l.area_m2,
                    volume_m3: l.volume_m3,
                    catchment_m2: l.catchment_m2,
                    outlet_cell: l.outlet_cell.map_or(-1, i64::from),
                    centroid_x_m: l.centroid_m.0,
                    centroid_y_m: l.centroid_m.1,
                },
            )
        })
        .collect();
    let lakes = fbb.create_vector(&lakes);
    let reaches: Vec<_> = map
        .reaches
        .iter()
        .map(|r| {
            let cells = fbb.create_vector(&r.cells);
            let (terminus, terminus_lake) = match r.terminus {
                Terminus::Junction => (save::Terminus::Junction, 0),
                Terminus::Lake(id) => (save::Terminus::Lake, id),
                Terminus::Outlet => (save::Terminus::Outlet, 0),
                Terminus::Sink => (save::Terminus::Sink, 0),
            };
            save::Reach::create(
                &mut fbb,
                &save::ReachArgs {
                    id: r.id,
                    cells: Some(cells),
                    downstream: r.downstream.map_or(-1, i64::from),
                    terminus,
                    terminus_lake,
                    order: r.order,
                    discharge_m3s: r.discharge_m3s,
                    width_m: r.width_m,
                    drainage_area_km2: r.drainage_area_km2,
                },
            )
        })
        .collect();
    let reaches = fbb.create_vector(&reaches);
    let inflows: Vec<_> = map
        .inflows
        .iter()
        .map(|i| {
            save::Inflow::create(
                &mut fbb,
                &save::InflowArgs {
                    cell: i.cell,
                    area_m2: i.area_m2,
                },
            )
        })
        .collect();
    let inflows = fbb.create_vector(&inflows);
    let root = save::Hydro::create(
        &mut fbb,
        &save::HydroArgs {
            lakes: Some(lakes),
            reaches: Some(reaches),
            inflows: Some(inflows),
        },
    );
    finish(fbb, root)
}

fn encode_tile(map: &WorldMap, layer: save::Layer, tile: Tile) -> Vec<u8> {
    let w = map.width as usize;
    let sample = if matches!(layer, save::Layer::Receivers | save::Layer::Water) {
        1
    } else {
        4
    };
    let mut data = Vec::with_capacity(tile.width as usize * tile.height as usize * sample);
    for y in tile.y0..tile.y0 + tile.height {
        let start = y as usize * w + tile.x0 as usize;
        let span = start..start + tile.width as usize;
        match layer {
            save::Layer::Elevation => {
                for v in &map.elevation[span] {
                    data.extend_from_slice(&v.to_le_bytes());
                }
            }
            save::Layer::Receivers => data.extend_from_slice(&map.receivers[span]),
            save::Layer::Water => data.extend_from_slice(&map.water[span]),
            _ => {
                for v in &map.lake_id[span] {
                    data.extend_from_slice(&v.to_le_bytes());
                }
            }
        }
    }
    let mut fbb = FlatBufferBuilder::with_capacity(data.len() + 128);
    let data = fbb.create_vector(&data);
    let root = save::RasterTile::create(
        &mut fbb,
        &save::RasterTileArgs {
            layer,
            x0: tile.x0,
            y0: tile.y0,
            width: tile.width,
            height: tile.height,
            data: Some(data),
        },
    );
    finish(fbb, root)
}

fn unreadable(tag: SectionTag, e: &InvalidFlatbuffer) -> LoadError {
    LoadError::Malformed(format!("section `{tag}` does not decode: {e}"))
}

fn check_compatible(info: &SnapshotInfo) -> Result<(), LoadError> {
    info.expect_engine(SAVE_ENGINE_TAG)
        .map_err(|e| LoadError::Incompatible(e.to_string()))?;
    if !(SCHEMA_V1..=SAVE_SCHEMA_VERSION).contains(&info.schema_version) {
        return Err(LoadError::Incompatible(format!(
            "it uses world schema version {}; this build reads versions {SCHEMA_V1} to \
             {SAVE_SCHEMA_VERSION}",
            info.schema_version
        )));
    }
    Ok(())
}

fn check_version(tag: SectionTag, version: u32) -> Result<(), LoadError> {
    if version == SECTION_VERSION {
        Ok(())
    } else {
        Err(LoadError::Incompatible(format!(
            "section `{tag}` has encoding version {version}; this build reads version \
             {SECTION_VERSION}"
        )))
    }
}

fn single_chunk<R: Read + Seek>(
    reader: &mut SnapshotReader<R>,
    tag: SectionTag,
) -> Result<Vec<u8>, LoadError> {
    let mut chunks = reader.require_section(tag)?;
    if chunks.len() != 1 {
        return Err(LoadError::Malformed(format!(
            "section `{tag}` has {} chunks, expected 1",
            chunks.len()
        )));
    }
    let chunk = chunks.remove(0);
    check_version(tag, chunk.version)?;
    Ok(chunk.bytes)
}

/// Reads a raster section into one row-major buffer of little-endian samples.
fn read_raster<R: Read + Seek>(
    reader: &mut SnapshotReader<R>,
    tag: SectionTag,
    layer: save::Layer,
    sample: usize,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, LoadError> {
    let chunks = reader.require_section(tag)?;
    let expected: Vec<Tile> = tiles(width, height).collect();
    if chunks.len() != expected.len() {
        return Err(LoadError::Malformed(format!(
            "section `{tag}` has {} tiles, expected {}",
            chunks.len(),
            expected.len()
        )));
    }
    let mut out = vec![0u8; width as usize * height as usize * sample];
    for (chunk, tile) in chunks.iter().zip(&expected) {
        check_version(tag, chunk.version)?;
        let decoded =
            flatbuffers::root::<save::RasterTile>(&chunk.bytes).map_err(|e| unreadable(tag, &e))?;
        let found = Tile {
            x0: decoded.x0(),
            y0: decoded.y0(),
            width: decoded.width(),
            height: decoded.height(),
        };
        if decoded.layer() != layer || found != *tile {
            return Err(LoadError::Malformed(format!(
                "tile {} of section `{tag}` is not the tile that belongs there",
                chunk.index
            )));
        }
        let data = decoded.data().map_or(&[][..], |d| d.bytes());
        let row_bytes = tile.width as usize * sample;
        if data.len() != row_bytes * tile.height as usize {
            return Err(LoadError::Malformed(format!(
                "tile {} of section `{tag}` has {} bytes, expected {}",
                chunk.index,
                data.len(),
                row_bytes * tile.height as usize
            )));
        }
        for (row, src) in data.chunks_exact(row_bytes).enumerate() {
            let at = ((tile.y0 as usize + row) * width as usize + tile.x0 as usize) * sample;
            out[at..at + row_bytes].copy_from_slice(src);
        }
    }
    Ok(out)
}

type Hydro = (Vec<Lake>, Vec<RiverReach>, Vec<Inflow>);

fn decode_hydro(bytes: &[u8]) -> Result<Hydro, LoadError> {
    let hydro =
        flatbuffers::root::<save::Hydro>(bytes).map_err(|e| unreadable(SECTION_HYDRO, &e))?;
    let bad = |what: String| LoadError::Malformed(format!("section `hydro`: {what}"));

    let mut lakes = Vec::new();
    for l in hydro.lakes().iter().flatten() {
        let outlet_cell = match l.outlet_cell() {
            -1 => None,
            v => Some(u32::try_from(v).map_err(|_| bad(format!("lake {} outlet {v}", l.id())))?),
        };
        lakes.push(Lake {
            id: l.id(),
            level_m: l.level_m(),
            spill_m: l.spill_m(),
            closed: l.closed(),
            cell_count: l.cell_count(),
            area_m2: l.area_m2(),
            volume_m3: l.volume_m3(),
            catchment_m2: l.catchment_m2(),
            outlet_cell,
            centroid_m: (l.centroid_x_m(), l.centroid_y_m()),
        });
    }

    let reach_count = hydro.reaches().map_or(0, |r| r.len());
    let mut reaches = Vec::with_capacity(reach_count);
    for r in hydro.reaches().iter().flatten() {
        let downstream = match r.downstream() {
            -1 => None,
            v if (0..reach_count as i64).contains(&v) => Some(v as u32),
            v => {
                return Err(bad(format!(
                    "reach {} flows into missing reach {v}",
                    r.id()
                )));
            }
        };
        let terminus = match r.terminus() {
            save::Terminus::Junction => Terminus::Junction,
            save::Terminus::Lake => Terminus::Lake(r.terminus_lake()),
            save::Terminus::Outlet => Terminus::Outlet,
            save::Terminus::Sink => Terminus::Sink,
            other => return Err(bad(format!("reach {} has terminus {}", r.id(), other.0))),
        };
        reaches.push(RiverReach {
            id: r.id(),
            cells: r.cells().map(|c| c.iter().collect()).unwrap_or_default(),
            downstream,
            terminus,
            order: r.order(),
            discharge_m3s: r.discharge_m3s(),
            width_m: r.width_m(),
            drainage_area_km2: r.drainage_area_km2(),
        });
    }

    let inflows = hydro
        .inflows()
        .iter()
        .flatten()
        .map(|i| Inflow {
            cell: i.cell(),
            area_m2: i.area_m2(),
        })
        .collect();
    Ok((lakes, reaches, inflows))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiles_cover_a_map_exactly_once() {
        for (w, h) in [(512, 512), (1000, 600), (64, 2048), (1536, 513)] {
            let mut covered = vec![0u8; w as usize * h as usize];
            for t in tiles(w, h) {
                assert!(t.width <= TILE_CELLS && t.height <= TILE_CELLS);
                for y in t.y0..t.y0 + t.height {
                    for x in t.x0..t.x0 + t.width {
                        covered[(y * w + x) as usize] += 1;
                    }
                }
            }
            assert!(covered.iter().all(|&c| c == 1), "{w}×{h}");
        }
    }

    #[test]
    fn labels_fit_the_header() {
        assert_eq!(clean_label("  Before the flood\n"), "Before the flood");
        let long = "ö".repeat(40);
        let cut = clean_label(&long);
        assert!(cut.len() <= LABEL_LEN);
        assert!(cut.chars().all(|c| c == 'ö'));
    }

    #[test]
    fn world_directories_are_slugged() {
        let meta = WorldMeta {
            world_id: [0xab; 16],
            name: "Old River — Valley!".to_owned(),
            seed: 1,
            preset_id: String::new(),
            generator_version: 1,
            created_unix_ms: 0,
            params: Vec::new(),
        };
        assert_eq!(world_dir_name(&meta), "old-river-valley-abababab");
        let unnamed = WorldMeta {
            name: "ÆØÅ".to_owned(),
            ..meta
        };
        assert_eq!(world_dir_name(&unnamed), "world-abababab");
    }
}
