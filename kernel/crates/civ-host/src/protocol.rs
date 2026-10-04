//! The host's side of the observer protocol (ADR-0001): requests decoded from commons-wire
//! frames, and the payloads that belong to the session rather than the world (handshake,
//! snapshot, events, save lists, acknowledgements and errors). World payloads come from
//! `civ_sim::frames`.

use civ_content::ContentRegistry;
use civ_schema::flatbuffers::{self, FlatBufferBuilder, WIPOffset};
use civ_schema::{WIRE_SCHEMA, wire};
use civ_sim::frames::RasterQuery;
use civ_sim::{NewWorld, Sim};
use commons_wire::{FrameKind, FrameMeta, Sequencer, WireError};

/// Bounds on frames from clients: requests are small.
pub const CLIENT_LIMITS: commons_wire::Limits = commons_wire::Limits {
    max_payload_len: 64 * 1024,
};

/// Something a client asked for.
#[derive(Clone, Debug, PartialEq)]
pub enum Request {
    /// Generate a world.
    NewWorld(NewWorld),
    /// Save the current world.
    SaveWorld {
        /// Label for the save browser.
        label: String,
    },
    /// Load a save, by its path relative to the saves directory.
    LoadWorld {
        /// The `file` of a save-list entry.
        file: String,
    },
    /// Pause, resume or change speed.
    SetClock {
        /// Stop the clock.
        paused: bool,
        /// Simulated seconds per real second.
        speed: f32,
    },
    /// Cancel the running task.
    CancelTask,
    /// Accept or dismiss the recovery offer.
    RecoverWorld {
        /// Load the offered save.
        accept: bool,
    },
    /// Send families to a point on the map (god tool).
    SpawnFamily {
        /// Where, metres from the map's north-west corner.
        at: (f32, f32),
        /// How many come together (at least one).
        families: u32,
    },
    /// Run ahead to a time, unpaced and in full detail.
    RunUntil {
        /// Simulation minute to stop at.
        minute: i64,
    },
    /// Read part of a raster.
    GetRaster(RasterQuery),
    /// Read the rivers and lakes.
    GetHydrography {
        /// Polyline simplification tolerance, metres.
        tolerance_m: f32,
    },
    /// List the saves.
    ListSaves,
    /// Read the routes of trips under way.
    GetTrips {
        /// Trip ids.
        ids: Vec<u64>,
    },
    /// Read about one person.
    GetPerson {
        /// Permanent id.
        id: u64,
        /// Decision receipts to include.
        decisions: u32,
    },
    /// Read chronicle entries.
    GetChronicle {
        /// Entries after this sequence number.
        after_seq: u64,
        /// Most entries to return.
        limit: u32,
    },
    /// Read every field.
    GetFields,
    /// Read every building.
    GetBuildings,
    /// Read the worn ground and the trails.
    GetPaths,
    /// Read every settlement's market.
    GetMarkets,
    /// Read every workshop, in brief.
    GetFirms,
    /// Read one workshop's page.
    GetFirm {
        /// Permanent id.
        id: u64,
    },
    /// Read every settlement's wealth measures.
    GetWealth,
}

/// A long-running operation, as the snapshot shows it.
#[derive(Clone, Debug, PartialEq)]
pub struct TaskView {
    /// What is running.
    pub name: String,
    /// What it is doing now.
    pub stage: String,
    /// Completion, 0–1.
    pub fraction: f32,
    /// Whether it can be cancelled.
    pub cancellable: bool,
}

/// The save offered after an unclean shutdown or an internal error.
#[derive(Clone, Debug, PartialEq)]
pub struct RecoveryOffer {
    /// Why recovery is offered.
    pub reason: String,
    /// The save, relative to the saves directory.
    pub file: String,
    /// Its label.
    pub label: String,
    /// Its simulation time, minutes.
    pub sim_minute: i64,
    /// The world's name.
    pub world_name: String,
}

/// One entry of the event log.
#[derive(Clone, Debug, PartialEq)]
pub struct EventRecord {
    /// Position in the log, from 1.
    pub id: u64,
    /// Simulation time when it happened (0 without a world).
    pub sim_minute: i64,
    /// Wall-clock time, Unix milliseconds.
    pub unix_ms: i64,
    /// What kind of event.
    pub kind: wire::EventKind,
    /// What happened, in plain words.
    pub text: String,
}

/// One save, as the save browser lists it.
#[derive(Clone, Debug, PartialEq)]
pub struct SaveEntry {
    /// Path relative to the saves directory, with `/` separators.
    pub file: String,
    /// World name.
    pub world_name: String,
    /// World id, hex.
    pub world_id: String,
    /// Save label.
    pub label: String,
    /// `manual`, `auto` or `crash`.
    pub kind: String,
    /// Generation number.
    pub generation: u64,
    /// When it was written, Unix milliseconds.
    pub created_unix_ms: i64,
    /// Simulation time, minutes.
    pub sim_minute: i64,
    /// File size.
    pub size_bytes: u64,
    /// This build can load it.
    pub compatible: bool,
    /// It was made with different content than is loaded now.
    pub content_changed: bool,
    /// Why it cannot be loaded, or a remark.
    pub note: String,
}

/// Wraps a payload in a frame of `kind`, numbered by `seq`.
pub fn frame(
    seq: &mut Sequencer,
    kind: FrameKind,
    correlation: u64,
    sim_time: i64,
    payload: &[u8],
) -> Result<Vec<u8>, WireError> {
    let meta = FrameMeta {
        kind,
        schema: WIRE_SCHEMA,
        epoch: seq.epoch(),
        sequence: seq.next(kind),
        correlation,
        sim_time,
    };
    commons_wire::encode(&meta, payload, false)
}

/// A client's frame after it was checked, on any transport.
#[derive(Clone, Debug, PartialEq)]
pub enum Incoming {
    /// A request for the engine, and the correlation id its reply must carry.
    Request(u64, Request),
    /// Nothing to do: a heartbeat, or a hello once the session is open.
    Nothing,
}

/// A client's frame the host refuses: the correlation id to answer and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal {
    /// The refused frame's correlation id, or 0 when the frame could not be read at all.
    pub correlation: u64,
    /// The error to report.
    pub code: wire::ErrorCode,
    /// What was wrong, in words.
    pub message: String,
}

/// Reads a client's frame once the session is open: a command or query in a schema this host
/// speaks, or a heartbeat.
pub fn read_request(bytes: &[u8]) -> Result<Incoming, Refusal> {
    let refuse = |correlation, code, message: String| Refusal {
        correlation,
        code,
        message,
    };
    let frame = commons_wire::decode(bytes, &CLIENT_LIMITS).map_err(|e| {
        refuse(
            0,
            wire::ErrorCode::BadRequest,
            format!("malformed frame: {e}"),
        )
    })?;
    let meta = frame.header.meta;
    match meta.kind {
        FrameKind::Command | FrameKind::Query => {}
        FrameKind::Heartbeat | FrameKind::Hello => return Ok(Incoming::Nothing),
        other => {
            return Err(refuse(
                meta.correlation,
                wire::ErrorCode::BadRequest,
                format!("the host does not accept {other} frames"),
            ));
        }
    }
    if !meta.schema.is_compatible_with(&WIRE_SCHEMA) {
        return Err(refuse(
            meta.correlation,
            wire::ErrorCode::Incompatible,
            format!("this host speaks {WIRE_SCHEMA}, not {}", meta.schema),
        ));
    }
    decode_request(meta.kind, frame.payload)
        .map(|request| Incoming::Request(meta.correlation, request))
        .map_err(|why| refuse(meta.correlation, wire::ErrorCode::BadRequest, why))
}

/// Checks a client's `Hello`. Returns the client's name.
pub fn check_hello(frame: &commons_wire::Frame<'_>) -> Result<String, String> {
    if frame.header.meta.kind != FrameKind::Hello {
        return Err(format!(
            "the first frame must be hello, not {}",
            frame.header.meta.kind
        ));
    }
    let schema = frame.header.meta.schema;
    if !schema.is_compatible_with(&WIRE_SCHEMA) {
        return Err(format!(
            "this host speaks schema {WIRE_SCHEMA}; the client speaks {schema}"
        ));
    }
    let hello = flatbuffers::root::<wire::Hello>(frame.payload)
        .map_err(|e| format!("hello does not decode: {e}"))?;
    Ok(hello.client().unwrap_or("unknown client").to_owned())
}

/// Decodes a command or query payload.
pub fn decode_request(kind: FrameKind, payload: &[u8]) -> Result<Request, String> {
    let missing = |what: &str| format!("the {what} has no body");
    match kind {
        FrameKind::Command => {
            let command = flatbuffers::root::<wire::Command>(payload)
                .map_err(|e| format!("command does not decode: {e}"))?;
            match command.body_type() {
                wire::CommandBody::NewWorld => {
                    let b = command
                        .body_as_new_world()
                        .ok_or_else(|| missing("command"))?;
                    Ok(Request::NewWorld(NewWorld {
                        name: b.name().unwrap_or_default().to_owned(),
                        seed: b.seed(),
                        preset_id: b.preset_id().unwrap_or_default().to_owned(),
                        size_cells: b.size_cells(),
                        band_size: b.band_size(),
                        regime_id: b.regime_id().unwrap_or_default().to_owned(),
                    }))
                }
                wire::CommandBody::SaveWorld => {
                    let b = command
                        .body_as_save_world()
                        .ok_or_else(|| missing("command"))?;
                    Ok(Request::SaveWorld {
                        label: b.label().unwrap_or_default().to_owned(),
                    })
                }
                wire::CommandBody::LoadWorld => {
                    let b = command
                        .body_as_load_world()
                        .ok_or_else(|| missing("command"))?;
                    Ok(Request::LoadWorld {
                        file: b.file().unwrap_or_default().to_owned(),
                    })
                }
                wire::CommandBody::SetClock => {
                    let b = command
                        .body_as_set_clock()
                        .ok_or_else(|| missing("command"))?;
                    Ok(Request::SetClock {
                        paused: b.paused(),
                        speed: b.speed(),
                    })
                }
                wire::CommandBody::CancelTask => Ok(Request::CancelTask),
                wire::CommandBody::RecoverWorld => {
                    let b = command
                        .body_as_recover_world()
                        .ok_or_else(|| missing("command"))?;
                    Ok(Request::RecoverWorld { accept: b.accept() })
                }
                wire::CommandBody::SpawnFamily => {
                    let b = command
                        .body_as_spawn_family()
                        .ok_or_else(|| missing("command"))?;
                    let at = b
                        .at()
                        .map(|v| (v.x(), v.y()))
                        .ok_or_else(|| missing("point"))?;
                    Ok(Request::SpawnFamily {
                        at,
                        families: b.families().max(1),
                    })
                }
                wire::CommandBody::RunUntil => {
                    let b = command
                        .body_as_run_until()
                        .ok_or_else(|| missing("command"))?;
                    Ok(Request::RunUntil { minute: b.minute() })
                }
                other => Err(format!("unknown command {}", other.0)),
            }
        }
        FrameKind::Query => {
            let query = flatbuffers::root::<wire::Query>(payload)
                .map_err(|e| format!("query does not decode: {e}"))?;
            match query.body_type() {
                wire::QueryBody::GetRaster => {
                    let b = query.body_as_get_raster().ok_or_else(|| missing("query"))?;
                    Ok(Request::GetRaster(RasterQuery::from_wire(&b)))
                }
                wire::QueryBody::GetHydrography => {
                    let b = query
                        .body_as_get_hydrography()
                        .ok_or_else(|| missing("query"))?;
                    Ok(Request::GetHydrography {
                        tolerance_m: b.tolerance_m(),
                    })
                }
                wire::QueryBody::ListSaves => Ok(Request::ListSaves),
                wire::QueryBody::GetTrips => {
                    let b = query.body_as_get_trips().ok_or_else(|| missing("query"))?;
                    Ok(Request::GetTrips {
                        ids: b.ids().map(|v| v.iter().collect()).unwrap_or_default(),
                    })
                }
                wire::QueryBody::GetPerson => {
                    let b = query.body_as_get_person().ok_or_else(|| missing("query"))?;
                    Ok(Request::GetPerson {
                        id: b.id(),
                        decisions: b.decisions(),
                    })
                }
                wire::QueryBody::GetChronicle => {
                    let b = query
                        .body_as_get_chronicle()
                        .ok_or_else(|| missing("query"))?;
                    Ok(Request::GetChronicle {
                        after_seq: b.after_seq(),
                        limit: b.limit(),
                    })
                }
                wire::QueryBody::GetFields => Ok(Request::GetFields),
                wire::QueryBody::GetBuildings => Ok(Request::GetBuildings),
                wire::QueryBody::GetPaths => Ok(Request::GetPaths),
                wire::QueryBody::GetMarkets => Ok(Request::GetMarkets),
                wire::QueryBody::GetFirms => Ok(Request::GetFirms),
                wire::QueryBody::GetFirm => {
                    let b = query.body_as_get_firm().ok_or_else(|| missing("query"))?;
                    Ok(Request::GetFirm { id: b.id() })
                }
                wire::QueryBody::GetWealth => Ok(Request::GetWealth),
                other => Err(format!("unknown query {}", other.0)),
            }
        }
        other => Err(format!("{other} frames are not requests")),
    }
}

fn finish<T>(mut fbb: FlatBufferBuilder<'_>, root: WIPOffset<T>) -> Vec<u8> {
    fbb.finish(root, None);
    fbb.finished_data().to_vec()
}

/// The `Welcome` the host answers a `Hello` with.
pub fn welcome_payload(content: &ContentRegistry) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let presets: Vec<_> = content
        .presets
        .iter()
        .map(|p| {
            let id = fbb.create_string(&p.id);
            let name = fbb.create_string(&p.name);
            let description = fbb.create_string(&p.description);
            wire::PresetInfo::create(
                &mut fbb,
                &wire::PresetInfoArgs {
                    id: Some(id),
                    name: Some(name),
                    description: Some(description),
                    is_default: p.is_default,
                },
            )
        })
        .collect();
    let presets = fbb.create_vector(&presets);
    let activities: Vec<_> = content
        .catalog
        .activities
        .iter()
        .map(|a| {
            let id = fbb.create_string(&a.id);
            let name = fbb.create_string(&a.name);
            let doing = fbb.create_string(&a.doing);
            wire::ActivityInfo::create(
                &mut fbb,
                &wire::ActivityInfoArgs {
                    id: Some(id),
                    name: Some(name),
                    doing: Some(doing),
                },
            )
        })
        .collect();
    let activities = fbb.create_vector(&activities);
    let goods: Vec<_> = content
        .catalog
        .goods
        .iter()
        .map(|g| {
            let id = fbb.create_string(&g.id);
            let name = fbb.create_string(&g.name);
            let purpose = fbb.create_string(g.purpose.name());
            let eaten = fbb.create_string(g.eaten.name());
            wire::GoodInfo::create(
                &mut fbb,
                &wire::GoodInfoArgs {
                    id: Some(id),
                    name: Some(name),
                    purpose: Some(purpose),
                    kcal_per_kg: g.kcal_per_kg as f32,
                    eaten: Some(eaten),
                    tool_life_h: g.tool.as_ref().map_or(0.0, |t| t.life_h as f32),
                },
            )
        })
        .collect();
    let goods = fbb.create_vector(&goods);
    let skills: Vec<_> = content
        .catalog
        .skills
        .iter()
        .map(|k| {
            let id = fbb.create_string(&k.id);
            let name = fbb.create_string(&k.name);
            wire::SkillInfo::create(
                &mut fbb,
                &wire::SkillInfoArgs {
                    id: Some(id),
                    name: Some(name),
                },
            )
        })
        .collect();
    let skills = fbb.create_vector(&skills);
    let regimes: Vec<_> = content
        .catalog
        .regimes
        .iter()
        .map(|r| {
            let id = fbb.create_string(&r.id);
            let name = fbb.create_string(&r.name);
            let description = fbb.create_string(&r.description);
            let rules: Vec<_> = civ_sim::frames::wealth::regime_rules(r)
                .iter()
                .map(|s| fbb.create_string(s))
                .collect();
            let rules = fbb.create_vector(&rules);
            wire::RegimeInfo::create(
                &mut fbb,
                &wire::RegimeInfoArgs {
                    id: Some(id),
                    name: Some(name),
                    description: Some(description),
                    is_default: r.is_default,
                    rules: Some(rules),
                },
            )
        })
        .collect();
    let regimes = fbb.create_vector(&regimes);
    let crops: Vec<_> = content
        .catalog
        .crops
        .iter()
        .map(|c| {
            let id = fbb.create_string(&c.id);
            let name = fbb.create_string(&c.name);
            wire::CropInfo::create(
                &mut fbb,
                &wire::CropInfoArgs {
                    id: Some(id),
                    name: Some(name),
                    good: c.good as u16,
                    seed_good: c.seed_good as u16,
                },
            )
        })
        .collect();
    let crops = fbb.create_vector(&crops);
    let reasons: Vec<_> = civ_agents::Reason::ALL
        .iter()
        .map(|r| {
            let label = fbb.create_string(r.label());
            wire::ReasonInfo::create(
                &mut fbb,
                &wire::ReasonInfoArgs {
                    code: *r as u16,
                    label: Some(label),
                },
            )
        })
        .collect();
    let reasons = fbb.create_vector(&reasons);
    let band = &content.people.params.band;
    let map_sizes = fbb.create_vector(&civ_sim::MAP_SIZES);
    let multipliers = fbb.create_vector(&civ_sim::SPEED_MULTIPLIERS);
    let host = fbb.create_string("civ-host");
    let version = fbb.create_string(env!("CARGO_PKG_VERSION"));
    let fingerprint = fbb.create_string(&content.fingerprint_hex());
    let root = wire::Welcome::create(
        &mut fbb,
        &wire::WelcomeArgs {
            host: Some(host),
            version: Some(version),
            presets: Some(presets),
            map_sizes: Some(map_sizes),
            cell_size_m: civ_sim::CELL_SIZE_M as f32,
            content_fingerprint: Some(fingerprint),
            default_map_size: civ_sim::DEFAULT_MAP_SIZE,
            speed_1x: civ_sim::SPEED_1X,
            speed_multipliers: Some(multipliers),
            activities: Some(activities),
            reasons: Some(reasons),
            band_size_min: band.min_size,
            band_size_max: band.max_size,
            band_size_default: band.default_size,
            goods: Some(goods),
            crops: Some(crops),
            skills: Some(skills),
            regimes: Some(regimes),
        },
    );
    finish(fbb, root)
}

/// Everything a snapshot shows.
#[derive(Debug, Default)]
pub struct SnapshotParts<'a> {
    /// The current world.
    pub sim: Option<&'a Sim>,
    /// The running task.
    pub task: Option<&'a TaskView>,
    /// The recovery offer.
    pub recovery: Option<&'a RecoveryOffer>,
    /// The last failure, for the error banner.
    pub last_error: Option<&'a str>,
    /// When the current world was last autosaved, Unix milliseconds (0 = not yet).
    pub last_autosave_unix_ms: i64,
}

/// A `Snapshot` payload.
pub fn snapshot_payload(parts: &SnapshotParts<'_>) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let world = parts
        .sim
        .map(|sim| civ_sim::frames::world_info(&mut fbb, sim));
    let clock = parts.sim.map(|sim| civ_sim::frames::clock(&mut fbb, sim));
    let people = parts
        .sim
        .map(|sim| civ_sim::frames::people::person_briefs(&mut fbb, sim));
    let settlements = parts
        .sim
        .map(|sim| civ_sim::frames::people::settlement_briefs(&mut fbb, sim));
    let chronicle_head = parts.sim.map_or(0, civ_sim::frames::people::chronicle_head);
    let fields_rev = parts.sim.map_or(0, civ_sim::frames::fields::fields_rev);
    let buildings_rev = parts
        .sim
        .map_or(0, civ_sim::frames::buildings::buildings_rev);
    let paths_rev = parts.sim.map_or(0, civ_sim::frames::paths::paths_rev);
    let markets_rev = parts.sim.map_or(0, civ_sim::frames::markets::markets_rev);
    let firms_rev = parts.sim.map_or(0, civ_sim::frames::firms::firms_rev);
    let wealth_rev = parts.sim.map_or(0, civ_sim::frames::wealth::wealth_rev);
    let task = parts.task.map(|t| {
        let name = fbb.create_string(&t.name);
        let stage = fbb.create_string(&t.stage);
        wire::Task::create(
            &mut fbb,
            &wire::TaskArgs {
                name: Some(name),
                stage: Some(stage),
                fraction: t.fraction,
                cancellable: t.cancellable,
            },
        )
    });
    let recovery = parts.recovery.map(|r| {
        let reason = fbb.create_string(&r.reason);
        let save_file = fbb.create_string(&r.file);
        let save_label = fbb.create_string(&r.label);
        let world_name = fbb.create_string(&r.world_name);
        wire::Recovery::create(
            &mut fbb,
            &wire::RecoveryArgs {
                reason: Some(reason),
                save_file: Some(save_file),
                save_label: Some(save_label),
                sim_minute: r.sim_minute,
                world_name: Some(world_name),
            },
        )
    });
    let last_error = parts.last_error.map(|e| fbb.create_string(e));
    let root = wire::Snapshot::create(
        &mut fbb,
        &wire::SnapshotArgs {
            world,
            clock,
            task,
            recovery,
            last_error,
            last_autosave_unix_ms: parts.last_autosave_unix_ms,
            people,
            settlements,
            chronicle_head,
            fields_rev,
            buildings_rev,
            paths_rev,
            markets_rev,
            firms_rev,
            wealth_rev,
        },
    );
    finish(fbb, root)
}

/// An `Events` payload.
pub fn events_payload(events: &[EventRecord]) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let items: Vec<_> = events
        .iter()
        .map(|e| {
            let text = fbb.create_string(&e.text);
            wire::Event::create(
                &mut fbb,
                &wire::EventArgs {
                    id: e.id,
                    sim_minute: e.sim_minute,
                    unix_ms: e.unix_ms,
                    kind: e.kind,
                    text: Some(text),
                },
            )
        })
        .collect();
    let items = fbb.create_vector(&items);
    let root = wire::Events::create(
        &mut fbb,
        &wire::EventsArgs {
            events: Some(items),
        },
    );
    finish(fbb, root)
}

/// An `ErrorInfo` payload.
pub fn error_payload(code: wire::ErrorCode, message: &str) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let message = fbb.create_string(message);
    let root = wire::ErrorInfo::create(
        &mut fbb,
        &wire::ErrorInfoArgs {
            code,
            message: Some(message),
        },
    );
    finish(fbb, root)
}

fn response<T>(
    mut fbb: FlatBufferBuilder<'_>,
    body_type: wire::ResponseBody,
    body: WIPOffset<T>,
) -> Vec<u8> {
    let root = wire::Response::create(
        &mut fbb,
        &wire::ResponseArgs {
            body_type,
            body: Some(body.as_union_value()),
        },
    );
    finish(fbb, root)
}

/// A `Response` acknowledging a command.
pub fn ack_response(message: &str) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let message = fbb.create_string(message);
    let ack = wire::Ack::create(
        &mut fbb,
        &wire::AckArgs {
            message: Some(message),
        },
    );
    response(fbb, wire::ResponseBody::Ack, ack)
}

/// A `Response` listing saves.
pub fn save_list_response(entries: &[SaveEntry]) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let items: Vec<_> = entries
        .iter()
        .map(|s| {
            let file = fbb.create_string(&s.file);
            let world_name = fbb.create_string(&s.world_name);
            let label = fbb.create_string(&s.label);
            let kind = fbb.create_string(&s.kind);
            let note = fbb.create_string(&s.note);
            let world_id = fbb.create_string(&s.world_id);
            wire::SaveEntry::create(
                &mut fbb,
                &wire::SaveEntryArgs {
                    file: Some(file),
                    world_name: Some(world_name),
                    label: Some(label),
                    kind: Some(kind),
                    generation: s.generation,
                    created_unix_ms: s.created_unix_ms,
                    sim_minute: s.sim_minute,
                    size_bytes: s.size_bytes,
                    compatible: s.compatible,
                    content_changed: s.content_changed,
                    note: Some(note),
                    world_id: Some(world_id),
                },
            )
        })
        .collect();
    let items = fbb.create_vector(&items);
    let list = wire::SaveList::create(&mut fbb, &wire::SaveListArgs { saves: Some(items) });
    response(fbb, wire::ResponseBody::SaveList, list)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set_clock_command(paused: bool, speed: f32) -> Vec<u8> {
        let mut fbb = FlatBufferBuilder::new();
        let body = wire::SetClock::create(&mut fbb, &wire::SetClockArgs { paused, speed });
        let root = wire::Command::create(
            &mut fbb,
            &wire::CommandArgs {
                body_type: wire::CommandBody::SetClock,
                body: Some(body.as_union_value()),
            },
        );
        finish(fbb, root)
    }

    fn load_command(file: &str) -> Vec<u8> {
        let mut fbb = FlatBufferBuilder::new();
        let file = fbb.create_string(file);
        let body = wire::LoadWorld::create(&mut fbb, &wire::LoadWorldArgs { file: Some(file) });
        let root = wire::Command::create(
            &mut fbb,
            &wire::CommandArgs {
                body_type: wire::CommandBody::LoadWorld,
                body: Some(body.as_union_value()),
            },
        );
        finish(fbb, root)
    }

    #[test]
    fn commands_decode_into_requests() {
        assert_eq!(
            decode_request(FrameKind::Command, &set_clock_command(false, 288.0)),
            Ok(Request::SetClock {
                paused: false,
                speed: 288.0
            })
        );
        assert_eq!(
            decode_request(
                FrameKind::Command,
                &load_command("w-1/g0000000001-manual.tcesave")
            ),
            Ok(Request::LoadWorld {
                file: "w-1/g0000000001-manual.tcesave".to_owned()
            })
        );
        let mut fbb = FlatBufferBuilder::new();
        let at = wire::Vec2::new(120.5, 64.0);
        let body = wire::SpawnFamily::create(
            &mut fbb,
            &wire::SpawnFamilyArgs {
                at: Some(&at),
                families: 0,
            },
        );
        let root = wire::Command::create(
            &mut fbb,
            &wire::CommandArgs {
                body_type: wire::CommandBody::SpawnFamily,
                body: Some(body.as_union_value()),
            },
        );
        assert_eq!(
            decode_request(FrameKind::Command, &finish(fbb, root)),
            Ok(Request::SpawnFamily {
                at: (120.5, 64.0),
                families: 1
            })
        );
        let mut fbb = FlatBufferBuilder::new();
        let body = wire::RunUntil::create(&mut fbb, &wire::RunUntilArgs { minute: 525_600 });
        let root = wire::Command::create(
            &mut fbb,
            &wire::CommandArgs {
                body_type: wire::CommandBody::RunUntil,
                body: Some(body.as_union_value()),
            },
        );
        assert_eq!(
            decode_request(FrameKind::Command, &finish(fbb, root)),
            Ok(Request::RunUntil { minute: 525_600 })
        );
    }

    #[test]
    fn workshop_queries_decode_into_requests() {
        let query = |body_type, body: Option<_>| {
            let mut fbb = FlatBufferBuilder::new();
            let body = body.map(|id| {
                wire::GetFirm::create(&mut fbb, &wire::GetFirmArgs { id }).as_union_value()
            });
            let body = body.or_else(|| {
                Some(wire::GetFirms::create(&mut fbb, &wire::GetFirmsArgs {}).as_union_value())
            });
            let root = wire::Query::create(&mut fbb, &wire::QueryArgs { body_type, body });
            finish(fbb, root)
        };
        assert_eq!(
            decode_request(FrameKind::Query, &query(wire::QueryBody::GetFirms, None)),
            Ok(Request::GetFirms)
        );
        assert_eq!(
            decode_request(FrameKind::Query, &query(wire::QueryBody::GetFirm, Some(42))),
            Ok(Request::GetFirm { id: 42 })
        );
    }

    #[test]
    fn the_wealth_query_and_a_regime_choice_decode() {
        let mut fbb = FlatBufferBuilder::new();
        let body = wire::GetWealth::create(&mut fbb, &wire::GetWealthArgs {});
        let root = wire::Query::create(
            &mut fbb,
            &wire::QueryArgs {
                body_type: wire::QueryBody::GetWealth,
                body: Some(body.as_union_value()),
            },
        );
        assert_eq!(
            decode_request(FrameKind::Query, &finish(fbb, root)),
            Ok(Request::GetWealth)
        );
        let mut fbb = FlatBufferBuilder::new();
        let regime_id = fbb.create_string("core:regime/village");
        let body = wire::NewWorld::create(
            &mut fbb,
            &wire::NewWorldArgs {
                seed: 7,
                regime_id: Some(regime_id),
                ..Default::default()
            },
        );
        let root = wire::Command::create(
            &mut fbb,
            &wire::CommandArgs {
                body_type: wire::CommandBody::NewWorld,
                body: Some(body.as_union_value()),
            },
        );
        match decode_request(FrameKind::Command, &finish(fbb, root)) {
            Ok(Request::NewWorld(w)) => {
                assert_eq!((w.seed, w.regime_id.as_str()), (7, "core:regime/village"));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn junk_and_wrong_kinds_are_refused() {
        assert!(decode_request(FrameKind::Command, &[0, 1, 2, 3]).is_err());
        assert!(decode_request(FrameKind::Snapshot, &[]).is_err());
        let mut fbb = FlatBufferBuilder::new();
        let root = wire::Command::create(&mut fbb, &wire::CommandArgs::default());
        let empty = finish(fbb, root);
        assert!(decode_request(FrameKind::Command, &empty).is_err());
    }

    #[test]
    fn frames_carry_the_schema_and_count_per_kind() {
        let mut seq = Sequencer::new(4);
        let a = frame(&mut seq, FrameKind::Snapshot, 0, 10, &[1, 2]).expect("encodes");
        let b = frame(&mut seq, FrameKind::Snapshot, 0, 11, &[3]).expect("encodes");
        let c = frame(&mut seq, FrameKind::Events, 0, 11, &[]).expect("encodes");
        let limits = commons_wire::Limits::default();
        let a = commons_wire::decode(&a, &limits).expect("decodes");
        let b = commons_wire::decode(&b, &limits).expect("decodes");
        let c = commons_wire::decode(&c, &limits).expect("decodes");
        assert_eq!(a.header.meta.schema, WIRE_SCHEMA);
        assert_eq!(a.header.meta.epoch, 4);
        assert_eq!((a.header.meta.sequence, b.header.meta.sequence), (0, 1));
        assert_eq!(c.header.meta.sequence, 0);
        // Responses must answer a request.
        assert!(frame(&mut seq, FrameKind::Response, 0, 0, &[]).is_err());
    }
}
