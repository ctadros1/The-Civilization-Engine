//! The engine thread: owns the current world and applies requests to it one at a time (the kernel
//! owns truth, plan §1).
//!
//! - While unpaused, the clock follows real time at the chosen speed. Each step is capped, so a
//!   stalled host falls behind instead of jumping ahead.
//! - World generation and loading run on worker threads and report back, so the engine stays
//!   responsive and generation can be cancelled.
//! - Autosave (plan §3.5): every in-game month or 5 real minutes, whichever comes first, while
//!   the world has changed; also before a world is replaced and at shutdown. The newest few
//!   autosaves are kept.
//! - A panic is contained: a crash snapshot is attempted for diagnosis, the world is dropped (its
//!   state can no longer be trusted), and the last good save is offered for recovery.

use std::collections::VecDeque;
use std::io;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use civ_core::SimTime;

use civ_content::ContentRegistry;
use civ_schema::{SAVE_EXTENSION, wire};
use civ_sim::{Mode, NewWorld, Sim, SimError, frames, persist};
use civ_world::GenError;
use commons_persist::{SaveDir, SaveKind};
use tokio::sync::{broadcast, oneshot, watch};

use crate::protocol::{self, EventRecord, RecoveryOffer, Request, SnapshotParts, TaskView};
use crate::session::{self, PathError, SessionMarker};

/// Longest stretch of real time one clock step may cover.
const MAX_STEP: Duration = Duration::from_millis(250);
/// Least time between snapshots while only the clock is changing.
const SNAPSHOT_INTERVAL: Duration = Duration::from_millis(100);
/// Least time between progress reports from a worker.
const PROGRESS_INTERVAL: Duration = Duration::from_millis(100);
/// Events kept for observers that connect later.
const HISTORY: usize = 200;
/// Share of each tick spent living the world on while running ahead; the rest keeps the observer
/// answered.
const RUN_AHEAD_SHARE: f64 = 0.8;
/// Simulated minutes an Accelerated day is lived in at a call, so commands are answered within
/// a tick however long a day takes (ADR-0028 §5; an engineering choice: about half a second a
/// chunk at ten thousand people on the container).
const ACCELERATED_CHUNK_MINUTES: i64 = 30;
/// Longest run ahead asked for at once, simulated minutes (a hundred years).
const MAX_RUN_AHEAD_MINUTES: i64 = 100 * civ_core::time::MINUTES_PER_YEAR;

/// How the engine runs.
#[derive(Clone, Debug)]
pub struct EngineConfig {
    /// The loaded content.
    pub content: Arc<ContentRegistry>,
    /// Where worlds' save directories live.
    pub saves_root: PathBuf,
    /// Real time between autosaves of a changing world.
    pub autosave_interval: Duration,
    /// Autosaves kept per world.
    pub keep_autosaves: usize,
    /// How often the clock is advanced.
    pub tick: Duration,
    /// How notables choose their moves in every world this session installs (ADR-0039 §3).
    pub deliberator: crate::deliberator::DeliberatorSetting,
}

impl EngineConfig {
    /// The plan's defaults: autosave every 5 real minutes, keep 5, tick at 20 Hz.
    pub fn new(content: Arc<ContentRegistry>, saves_root: PathBuf) -> Self {
        EngineConfig {
            content,
            saves_root,
            autosave_interval: Duration::from_secs(5 * 60),
            keep_autosaves: 5,
            tick: Duration::from_millis(50),
            deliberator: crate::deliberator::DeliberatorSetting::Rule,
        }
    }
}

/// The engine's answer to a request.
#[derive(Debug)]
pub enum Reply {
    /// A finished `Response` payload.
    Response(Vec<u8>),
    /// The request failed.
    Error(wire::ErrorCode, String),
}

/// A message to the engine thread.
#[derive(Debug)]
pub enum Msg {
    /// A request and where to send the answer.
    Request(Request, oneshot::Sender<Reply>),
    /// A worker reporting progress or its result.
    Worker(WorkerReport),
    /// Stop: autosave if needed, remove the session marker and exit.
    Shutdown,
}

/// What a worker thread reports.
#[derive(Debug)]
pub enum WorkerReport {
    /// Progress of a running task.
    Progress {
        /// Task id.
        task: u64,
        /// What it is doing.
        stage: String,
        /// Completion, 0–1.
        fraction: f32,
    },
    /// The task finished.
    Done {
        /// Task id.
        task: u64,
        /// How it ended.
        outcome: Box<Outcome>,
    },
}

/// How a task ended.
#[derive(Debug)]
pub enum Outcome {
    /// World generation finished or failed.
    Created(Result<Sim, String>),
    /// Loading finished or failed.
    Loaded {
        /// The save, relative to the saves root.
        file: String,
        /// The world, or why it was refused.
        result: Result<Sim, String>,
    },
    /// The task was cancelled.
    Cancelled,
}

/// The latest snapshot, ready to be framed.
#[derive(Debug)]
pub struct SnapshotOut {
    /// World epoch (ADR-0001): increments whenever the world is replaced.
    pub epoch: u32,
    /// Simulation time, minutes.
    pub sim_time: i64,
    /// The `Snapshot` payload.
    pub payload: Vec<u8>,
}

/// Events as they happen.
#[derive(Debug)]
pub struct EventsOut {
    /// World epoch when they happened.
    pub epoch: u32,
    /// Simulation time, minutes.
    pub sim_time: i64,
    /// The events.
    pub events: Vec<EventRecord>,
}

/// What observers' connections use to talk to the engine.
#[derive(Clone, Debug)]
pub struct Channels {
    /// Requests in.
    pub requests: mpsc::Sender<Msg>,
    /// The latest snapshot (replaceable delivery).
    pub snapshot: watch::Receiver<Arc<SnapshotOut>>,
    /// Events (ordered delivery); subscribe before reading `history`.
    pub events: broadcast::Sender<Arc<EventsOut>>,
    /// Recent events, for observers that connect later.
    pub history: Arc<Mutex<VecDeque<EventRecord>>>,
}

/// A running engine thread.
#[derive(Debug)]
pub struct EngineHandle {
    /// For connections.
    pub channels: Channels,
    thread: JoinHandle<()>,
}

impl EngineHandle {
    /// Stops the engine and waits for it: the world is autosaved if it changed, and the session
    /// marker is removed.
    pub fn shutdown(self) {
        let _ = self.channels.requests.send(Msg::Shutdown);
        let _ = self.thread.join();
    }
}

/// Starts the engine thread. `previous` is the marker an unclean previous session left behind.
pub fn start(config: EngineConfig, previous: Option<SessionMarker>) -> io::Result<EngineHandle> {
    let (tx, rx) = mpsc::channel();
    let engine = Engine::new(config, tx, previous)?;
    let channels = engine.channels();
    let thread = thread::Builder::new()
        .name("engine".to_owned())
        .spawn(move || engine.run(rx))?;
    Ok(EngineHandle { channels, thread })
}

struct World {
    sim: Sim,
    dir: SaveDir,
    last_autosave: Instant,
    last_autosave_unix_ms: i64,
    /// A pause asked for during an Accelerated day, which stops the clock at its midnight
    /// (ADR-0011 §2).
    pause_at_midnight: bool,
}

impl World {
    /// Whether the world stands where Accelerated mode may stop, publish and save: anywhere in
    /// Detailed mode, at a midnight in Accelerated (ADR-0011 §2).
    fn at_rest(&self) -> bool {
        self.sim.mode() != Mode::Accelerated || self.sim.now().minute_of_day() == 0
    }

    /// Lives an Accelerated day begun to its midnight, where it may be saved.
    fn finish_day(&mut self) -> Result<(), SimError> {
        if !self.at_rest() {
            self.sim.advance_to_midnight()?;
        }
        Ok(())
    }
}

struct Task {
    id: u64,
    view: TaskView,
    cancel: Option<Arc<AtomicBool>>,
}

/// Running ahead to a time, as fast as the machine allows (M1 slice G), or skipping there
/// (M9c slice DL, ADR-0032 §7).
struct RunAhead {
    task: u64,
    from: SimTime,
    until: SimTime,
    cancel: Arc<AtomicBool>,
    skip: Option<Skip>,
}

/// A skip under way: what it put aside, the chronicle's head when it began, and the kinds of entry
/// that stop it.
struct Skip {
    restore: civ_sim::skip::SkipRestore,
    from_seq: u64,
    stop_on: u8,
}

enum Origin {
    Created,
    Loaded(String),
}

/// The engine's state. Lives on the engine thread.
pub struct Engine {
    config: EngineConfig,
    to_self: mpsc::Sender<Msg>,
    world: Option<World>,
    task: Option<Task>,
    run_ahead: Option<RunAhead>,
    next_task: u64,
    recovery: Option<RecoveryOffer>,
    last_error: Option<String>,
    epoch: u32,
    next_event: u64,
    marker: SessionMarker,
    snapshot_tx: watch::Sender<Arc<SnapshotOut>>,
    snapshot_rx: watch::Receiver<Arc<SnapshotOut>>,
    events_tx: broadcast::Sender<Arc<EventsOut>>,
    history: Arc<Mutex<VecDeque<EventRecord>>>,
    changed: bool,
    urgent: bool,
    last_publish: Instant,
}

impl std::fmt::Debug for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Engine")
            .field("epoch", &self.epoch)
            .field(
                "world",
                &self.world.as_ref().map(|w| w.sim.meta().name.as_str()),
            )
            .field("task", &self.task.as_ref().map(|t| t.view.name.as_str()))
            .field("recovery", &self.recovery)
            .field("last_error", &self.last_error)
            .finish_non_exhaustive()
    }
}

fn panic_message(panic: &(dyn std::any::Any + Send)) -> String {
    panic
        .downcast_ref::<&str>()
        .map(|s| (*s).to_owned())
        .or_else(|| panic.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "unknown panic".to_owned())
}

fn ack(message: &str) -> Reply {
    Reply::Response(protocol::ack_response(message))
}

fn no_world() -> Reply {
    Reply::Error(
        wire::ErrorCode::NoWorld,
        "there is no world yet: create or load one".to_owned(),
    )
}

impl Engine {
    /// An engine with no world. Writes this session's marker; offers recovery when `previous`
    /// says the last session ended uncleanly.
    pub fn new(
        config: EngineConfig,
        to_self: mpsc::Sender<Msg>,
        previous: Option<SessionMarker>,
    ) -> io::Result<Engine> {
        std::fs::create_dir_all(&config.saves_root)?;
        let marker = SessionMarker::for_this_process();
        marker.write(&config.saves_root)?;
        let (snapshot_tx, snapshot_rx) = watch::channel(Arc::new(SnapshotOut {
            epoch: 0,
            sim_time: 0,
            payload: protocol::snapshot_payload(&SnapshotParts::default()),
        }));
        let (events_tx, _) = broadcast::channel(256);
        let mut engine = Engine {
            config,
            to_self,
            world: None,
            task: None,
            run_ahead: None,
            next_task: 0,
            recovery: None,
            last_error: None,
            epoch: 0,
            next_event: 0,
            marker,
            snapshot_tx,
            snapshot_rx,
            events_tx,
            history: Arc::new(Mutex::new(VecDeque::new())),
            changed: false,
            urgent: true,
            last_publish: Instant::now(),
        };
        engine.event(
            wire::EventKind::Info,
            format!(
                "civ-host {} started with content {}.",
                env!("CARGO_PKG_VERSION"),
                &engine.config.content.fingerprint_hex()[..12]
            ),
        );
        if let Some(previous) = previous {
            let reason = "The last session did not shut down cleanly.";
            engine.recovery = session::find_recovery(
                &engine.config.saves_root,
                &previous,
                &engine.config.content,
                reason,
            );
            let text = match &engine.recovery {
                Some(offer) => format!(
                    "{reason} “{}” can be recovered from {}.",
                    offer.world_name, offer.file
                ),
                None => format!("{reason} There is no save to recover."),
            };
            engine.event(wire::EventKind::Warning, text);
        }
        engine.publish();
        Ok(engine)
    }

    /// The channels connections use.
    pub fn channels(&self) -> Channels {
        Channels {
            requests: self.to_self.clone(),
            snapshot: self.snapshot_rx.clone(),
            events: self.events_tx.clone(),
            history: Arc::clone(&self.history),
        }
    }

    /// Runs until [`Msg::Shutdown`].
    pub fn run(mut self, rx: mpsc::Receiver<Msg>) {
        let mut last_tick = Instant::now();
        loop {
            let wait = self.config.tick.saturating_sub(last_tick.elapsed());
            match rx.recv_timeout(wait) {
                Ok(Msg::Shutdown) | Err(RecvTimeoutError::Disconnected) => break,
                Ok(msg) => self.guarded(|e| e.handle(msg)),
                Err(RecvTimeoutError::Timeout) => {}
            }
            let elapsed = last_tick.elapsed();
            if elapsed >= self.config.tick {
                last_tick = Instant::now();
                self.guarded(|e| e.tick(elapsed));
            }
            self.publish_if_due();
        }
        self.guarded(Engine::shutdown);
        self.publish();
    }

    /// Runs `f`, containing any panic (plan §3.5).
    pub fn guarded(&mut self, f: impl FnOnce(&mut Engine)) {
        if let Err(panic) = catch_unwind(AssertUnwindSafe(|| f(self))) {
            self.contain_panic(&panic_message(panic.as_ref()));
        }
    }

    /// Handles one message.
    pub fn handle(&mut self, msg: Msg) {
        match msg {
            Msg::Request(request, reply) => {
                let answer = self.request(request);
                let _ = reply.send(answer);
            }
            Msg::Worker(report) => self.worker(report),
            Msg::Shutdown => {}
        }
    }

    fn request(&mut self, request: Request) -> Reply {
        match request {
            Request::NewWorld(new_world) => self.start_new_world(new_world),
            Request::SaveWorld { label } => self.save_manual(&label),
            Request::LoadWorld { file } => self.start_load(file),
            Request::SetClock { paused, speed } => self.set_clock(paused, speed),
            Request::CancelTask => self.cancel_task(),
            Request::RecoverWorld { accept } => self.recover(accept),
            Request::SpawnFamily { at, families } => self.spawn_families(at, families),
            Request::RunUntil {
                minute,
                skip,
                stop_on,
            } => self.start_run_ahead(minute, skip, stop_on),
            Request::IntroduceTechnique {
                person,
                technique,
                aware_only,
            } => self.introduce_technique(person, technique, aware_only),
            Request::PlaceDeposit {
                at,
                good,
                radius_m,
                exposed,
            } => self.place_deposit(at, &good, radius_m, exposed),
            Request::Whisper { person, claim } => self.whisper(person, claim),
            Request::TellOfIdeology { person, ideology } => self.tell_of_ideology(person, ideology),
            Request::SendAgitator { at, ideology } => self.send_agitator(at, ideology),
            Request::SendWave {
                at,
                households,
                days,
                months,
            } => self.send_wave(at, households, days, months),
            Request::Bless {
                person,
                curse,
                days,
                share,
            } => self.bless(person, curse, days, share),
            Request::Plague { person, disease } => self.plague(person, disease),
            Request::Drought { months, depth } => self.drought(months, depth),
            Request::ForceClimate {
                offset_c,
                wet_factor,
                ramp_years,
            } => self.force_climate(offset_c, wet_factor, ramp_years),
            Request::SetFire { building } => self.set_fire(building),
            Request::HandOver { .. } => Reply::Error(
                wire::ErrorCode::BadRequest,
                "only a connection to the observer socket can hand over the driving".to_owned(),
            ),
            Request::RaiseRiver {
                reach,
                discharge_m3s,
                days,
            } => self.raise_river(reach, discharge_m3s, days),
            Request::SendStorm {
                wind_speed_ms,
                direction_deg,
            } => self.send_storm(wind_speed_ms, direction_deg),
            Request::Shake {
                strength,
                settlement,
            } => self.shake(strength, settlement),
            Request::SetResolution {
                by_focus,
                focus,
                pins,
                follow,
            } => self.set_resolution(by_focus, focus, &pins, follow),
            Request::GetRaster(query) => match &self.world {
                None => no_world(),
                Some(w) => match frames::raster_response(
                    w.sim.map(),
                    &w.sim.land().ground,
                    w.sim.stats(),
                    &w.sim.land().flood_depths,
                    &query,
                ) {
                    Ok(payload) => Reply::Response(payload),
                    Err(e) => Reply::Error(wire::ErrorCode::BadRequest, e.to_string()),
                },
            },
            Request::GetHydrography { tolerance_m } => match &self.world {
                None => no_world(),
                Some(w) => Reply::Response(frames::hydrography_response(w.sim.map(), tolerance_m)),
            },
            Request::GetTrips { ids } => match &self.world {
                None => no_world(),
                Some(w) => match frames::people::trips_response(&w.sim, &ids) {
                    Ok(payload) => Reply::Response(payload),
                    Err(e) => Reply::Error(wire::ErrorCode::BadRequest, e.to_string()),
                },
            },
            Request::GetPerson { id, decisions } => match &self.world {
                None => no_world(),
                Some(w) => match frames::people::person_response(&w.sim, id, decisions) {
                    Ok(payload) => Reply::Response(payload),
                    Err(e) => Reply::Error(wire::ErrorCode::NotFound, e.to_string()),
                },
            },
            Request::GetChronicle { after_seq, limit } => match &self.world {
                None => no_world(),
                Some(w) => {
                    Reply::Response(frames::people::chronicle_response(&w.sim, after_seq, limit))
                }
            },
            Request::GetFields => match &self.world {
                None => no_world(),
                Some(w) => Reply::Response(frames::fields::fields_response(&w.sim)),
            },
            Request::GetBuildings => match &self.world {
                None => no_world(),
                Some(w) => Reply::Response(frames::buildings::buildings_response(&w.sim)),
            },
            Request::GetPaths => match &self.world {
                None => no_world(),
                Some(w) => Reply::Response(frames::paths::paths_response(&w.sim)),
            },
            Request::GetMarkets => match &self.world {
                None => no_world(),
                Some(w) => Reply::Response(frames::markets::markets_response(&w.sim)),
            },
            Request::GetFirms => match &self.world {
                None => no_world(),
                Some(w) => Reply::Response(frames::firms::firms_response(&w.sim)),
            },
            Request::GetFirm { id } => match &self.world {
                None => no_world(),
                Some(w) => match frames::firms::firm_response(&w.sim, id) {
                    Ok(payload) => Reply::Response(payload),
                    Err(e) => Reply::Error(wire::ErrorCode::NotFound, e.to_string()),
                },
            },
            Request::GetWealth => match &self.world {
                None => no_world(),
                Some(w) => Reply::Response(frames::wealth::wealth_response(&w.sim)),
            },
            Request::GetKnowledge => match &self.world {
                None => no_world(),
                Some(w) => Reply::Response(frames::knowledge::knowledge_response(&w.sim)),
            },
            Request::GetStanding => match &self.world {
                None => no_world(),
                Some(w) => Reply::Response(frames::standing::standing_response(&w.sim)),
            },
            Request::GetGovernment => match &self.world {
                None => no_world(),
                Some(w) => Reply::Response(frames::government::government_response(&w.sim)),
            },
            Request::GetOrder => match &self.world {
                None => no_world(),
                Some(w) => Reply::Response(frames::order::order_response(&w.sim)),
            },
            Request::GetDeposits => match &self.world {
                None => no_world(),
                Some(w) => Reply::Response(frames::deposits::deposits_response(&w.sim)),
            },
            Request::GetEarthworks => match &self.world {
                None => no_world(),
                Some(w) => Reply::Response(frames::earthworks::earthworks_response(&w.sim)),
            },
            Request::GetCrossings => match &self.world {
                None => no_world(),
                Some(w) => Reply::Response(frames::crossings::crossings_response(&w.sim)),
            },
            Request::GetVehicles => match &self.world {
                None => no_world(),
                Some(w) => Reply::Response(frames::vehicles::vehicles_response(&w.sim)),
            },
            Request::GetPlans => match &self.world {
                None => no_world(),
                Some(w) => Reply::Response(frames::plans::plans_response(&w.sim)),
            },

            Request::GetLand => match &self.world {
                None => no_world(),
                Some(w) => Reply::Response(frames::land::land_response(&w.sim)),
            },
            Request::GetWays => match &self.world {
                None => no_world(),
                Some(w) => Reply::Response(frames::ways::ways_response(&w.sim)),
            },
            Request::GetWater => match &self.world {
                None => no_world(),
                Some(w) => Reply::Response(frames::water::water_response(&w.sim)),
            },
            Request::GetIncidents {
                settlement,
                kind,
                before,
                limit,
            } => match &self.world {
                None => no_world(),
                Some(w) => Reply::Response(frames::incidents::incidents_response(
                    &w.sim,
                    civ_core::PermanentId::from_raw(settlement),
                    kind,
                    before,
                    limit as usize,
                )),
            },
            Request::GetIncident { kind, id } => match &self.world {
                None => no_world(),
                Some(w) => match frames::incidents::incident_response(&w.sim, kind, id) {
                    Some(payload) => Reply::Response(payload),
                    None => Reply::Error(
                        wire::ErrorCode::NotFound,
                        format!("no incident of kind {kind} numbered {id}"),
                    ),
                },
            },
            Request::GetOutbreak { id, limit } => match &self.world {
                None => no_world(),
                Some(w) => match frames::incidents::outbreak_response(&w.sim, id, limit as usize) {
                    Some(payload) => Reply::Response(payload),
                    None => Reply::Error(
                        wire::ErrorCode::NotFound,
                        format!("no outbreak numbered {id}"),
                    ),
                },
            },
            Request::GetConflagration { id } => match &self.world {
                None => no_world(),
                Some(w) => match frames::incidents::conflagration_response(&w.sim, id) {
                    Some(payload) => Reply::Response(payload),
                    None => Reply::Error(
                        wire::ErrorCode::NotFound,
                        format!("no conflagration numbered {id}"),
                    ),
                },
            },
            Request::GetFlood { id } => match &self.world {
                None => no_world(),
                Some(w) => match frames::incidents::flood_response(&w.sim, id) {
                    Some(payload) => Reply::Response(payload),
                    None => Reply::Error(
                        wire::ErrorCode::NotFound,
                        format!("no flood episode numbered {id}"),
                    ),
                },
            },
            Request::GetStorm { id } => match &self.world {
                None => no_world(),
                Some(w) => match frames::incidents::storm_response(&w.sim, id) {
                    Some(payload) => Reply::Response(payload),
                    None => Reply::Error(
                        wire::ErrorCode::NotFound,
                        format!("no storm episode numbered {id}"),
                    ),
                },
            },
            Request::GetQuake { id } => match &self.world {
                None => no_world(),
                Some(w) => match frames::incidents::quake_response(&w.sim, id) {
                    Some(payload) => Reply::Response(payload),
                    None => Reply::Error(
                        wire::ErrorCode::NotFound,
                        format!("no shaking numbered {id}"),
                    ),
                },
            },
            Request::GetCoverage { settlement, lens } => match &self.world {
                None => no_world(),
                Some(w) => match frames::coverage::coverage_response(
                    &w.sim,
                    civ_core::PermanentId::from_raw(settlement),
                    lens,
                ) {
                    Some(payload) => Reply::Response(payload),
                    None => Reply::Error(
                        wire::ErrorCode::BadRequest,
                        format!("no lens numbered {lens}"),
                    ),
                },
            },
            Request::GetWeather => match &self.world {
                None => no_world(),
                Some(w) => Reply::Response(frames::weather::weather_response(&w.sim)),
            },
            Request::ListSaves => {
                match session::list_saves(&self.config.saves_root, &self.config.content) {
                    Ok(entries) => Reply::Response(protocol::save_list_response(&entries)),
                    Err(e) => Reply::Error(
                        wire::ErrorCode::Internal,
                        format!("the saves folder could not be read: {e}"),
                    ),
                }
            }
        }
    }

    fn busy(&self) -> Option<Reply> {
        self.task.as_ref().map(|t| {
            Reply::Error(
                wire::ErrorCode::Busy,
                format!("{} is still running", t.view.name),
            )
        })
    }

    fn begin_task(&mut self, name: String, cancel: Option<Arc<AtomicBool>>) -> u64 {
        self.next_task += 1;
        self.task = Some(Task {
            id: self.next_task,
            view: TaskView {
                name,
                stage: "Starting".to_owned(),
                fraction: 0.0,
                cancellable: cancel.is_some(),
            },
            cancel,
        });
        self.last_error = None;
        self.urgent = true;
        self.next_task
    }

    fn start_new_world(&mut self, request: NewWorld) -> Reply {
        if let Some(busy) = self.busy() {
            return busy;
        }
        let Some(preset) = self.config.content.preset(&request.preset_id) else {
            return Reply::Error(
                wire::ErrorCode::BadRequest,
                format!("there is no world preset `{}`", request.preset_id),
            );
        };
        if !civ_sim::MAP_SIZES.contains(&request.size_cells) {
            return Reply::Error(
                wire::ErrorCode::BadRequest,
                format!(
                    "a {} cell map is not offered; choose one of {:?}",
                    request.size_cells,
                    civ_sim::MAP_SIZES
                ),
            );
        }
        if !request.regime_id.is_empty()
            && self
                .config
                .content
                .catalog
                .regime(&request.regime_id)
                .is_none()
        {
            return Reply::Error(
                wire::ErrorCode::BadRequest,
                format!("there is no property regime `{}`", request.regime_id),
            );
        }
        let name = civ_sim::normalize_name(&request.name);
        let request = NewWorld {
            name: name.clone(),
            ..request
        };
        let cancel = Arc::new(AtomicBool::new(false));
        let task = self.begin_task(
            format!("Creating “{name}” ({})", preset.name),
            Some(Arc::clone(&cancel)),
        );
        let tx = self.to_self.clone();
        let content = Arc::clone(&self.config.content);
        let spawned = thread::Builder::new()
            .name("worldgen".to_owned())
            .spawn(move || {
                let mut last: Option<Instant> = None;
                let mut progress = |p: civ_world::Progress| {
                    if last.is_none_or(|t| t.elapsed() >= PROGRESS_INTERVAL) || p.fraction >= 1.0 {
                        last = Some(Instant::now());
                        let _ = tx.send(Msg::Worker(WorkerReport::Progress {
                            task,
                            stage: p.stage.to_owned(),
                            fraction: p.fraction,
                        }));
                    }
                };
                let result = catch_unwind(AssertUnwindSafe(|| {
                    Sim::create(&request, &content, &mut progress, &cancel)
                }));
                let outcome = match result {
                    Ok(Ok(sim)) => Outcome::Created(Ok(sim)),
                    Ok(Err(SimError::Generation(GenError::Cancelled))) => Outcome::Cancelled,
                    Ok(Err(e)) => Outcome::Created(Err(e.to_string())),
                    Err(panic) => Outcome::Created(Err(format!(
                        "world generation crashed ({})",
                        panic_message(panic.as_ref())
                    ))),
                };
                let _ = tx.send(Msg::Worker(WorkerReport::Done {
                    task,
                    outcome: Box::new(outcome),
                }));
            });
        if let Err(e) = spawned {
            self.task = None;
            return Reply::Error(
                wire::ErrorCode::Internal,
                format!("world generation could not start: {e}"),
            );
        }
        ack(&format!("Creating “{name}”"))
    }

    fn start_load(&mut self, file: String) -> Reply {
        if let Some(busy) = self.busy() {
            return busy;
        }
        let path = match session::resolve(&self.config.saves_root, &file) {
            Ok(path) => path,
            Err(PathError::Invalid(why)) => return Reply::Error(wire::ErrorCode::BadRequest, why),
            Err(PathError::NotFound(why)) => return Reply::Error(wire::ErrorCode::NotFound, why),
        };
        let task = self.begin_task(format!("Loading {file}"), None);
        let tx = self.to_self.clone();
        let content = Arc::clone(&self.config.content);
        let reported = file.clone();
        let spawned = thread::Builder::new()
            .name("load".to_owned())
            .spawn(move || {
                let result = catch_unwind(AssertUnwindSafe(|| persist::load(&path, &content)));
                let result = match result {
                    Ok(Ok(sim)) => Ok(sim),
                    Ok(Err(e)) => Err(e.to_string()),
                    Err(panic) => Err(format!(
                        "loading crashed ({})",
                        panic_message(panic.as_ref())
                    )),
                };
                let _ = tx.send(Msg::Worker(WorkerReport::Done {
                    task,
                    outcome: Box::new(Outcome::Loaded {
                        file: reported,
                        result,
                    }),
                }));
            });
        if let Err(e) = spawned {
            self.task = None;
            return Reply::Error(
                wire::ErrorCode::Internal,
                format!("loading could not start: {e}"),
            );
        }
        ack(&format!("Loading {file}"))
    }

    fn save_manual(&mut self, label: &str) -> Reply {
        let label = match persist::clean_label(label) {
            l if l.is_empty() => "Manual save".to_owned(),
            l => l,
        };
        let Some(world) = self.world.as_mut() else {
            return no_world();
        };
        if let Err(e) = world.finish_day() {
            let text = format!("Saving failed: {e}");
            self.fail(text.clone());
            return Reply::Error(wire::ErrorCode::Internal, text);
        }
        let Some(world) = self.world.as_mut() else {
            return no_world();
        };
        match persist::save(&mut world.sim, &world.dir, SaveKind::Manual, &label) {
            Ok(published) => {
                let file = session::relative(&self.config.saves_root, &published.path);
                let text = format!(
                    "Saved “{}” as “{label}” (generation {}).",
                    world.sim.meta().name,
                    world.sim.generation()
                );
                self.marker.last_good.clone_from(&file);
                self.write_marker();
                self.event(wire::EventKind::Saved, text);
                self.urgent = true;
                ack(&format!("Saved {}", file.unwrap_or_default()))
            }
            Err(e) => {
                let text = format!("Saving failed: {e}");
                self.fail(text.clone());
                Reply::Error(wire::ErrorCode::Internal, text)
            }
        }
    }

    /// Settlements away from the observer's focus household-resolved, or not (ADR-0032 §6): the
    /// kernel chooses them at each midnight; ids that name no settlement or person are ignored.
    fn set_resolution(&mut self, by_focus: bool, focus: u64, pins: &[u64], follow: u64) -> Reply {
        let Some(world) = self.world.as_mut() else {
            return no_world();
        };
        let id = |raw: u64| civ_core::PermanentId::from_raw(raw);
        world.sim.set_resolution_by_focus(by_focus);
        world.sim.set_focus(id(focus));
        world.sim.set_follow(id(follow));
        let wanted: std::collections::BTreeSet<_> = pins.iter().filter_map(|&p| id(p)).collect();
        let held: Vec<_> = world
            .sim
            .focus_policy()
            .map(|p| p.pins.iter().copied().collect())
            .unwrap_or_default();
        for p in held {
            world.sim.pin_detailed(p, wanted.contains(&p));
        }
        for &p in &wanted {
            world.sim.pin_detailed(p, true);
        }
        self.urgent = true;
        ack(if by_focus {
            "Settlements away from the focus are lived by household from the next midnight"
        } else {
            "Every settlement is lived in detail from the next midnight"
        })
    }

    fn set_clock(&mut self, paused: bool, speed: f32) -> Reply {
        if self.run_ahead.is_some() {
            self.finish_run_ahead("Stopped running ahead");
        }
        let Some(world) = self.world.as_mut() else {
            return no_world();
        };
        if let Err(e) = world.sim.set_speed(speed) {
            return Reply::Error(wire::ErrorCode::BadRequest, e.to_string());
        }
        self.urgent = true;
        // An Accelerated day begun is lived to its midnight, where the clock stops.
        if paused && !world.sim.paused() && !world.at_rest() {
            world.pause_at_midnight = true;
            return ack("Pausing at midnight");
        }
        world.pause_at_midnight = false;
        world.sim.set_paused(paused);
        ack(if paused { "Paused" } else { "Running" })
    }

    fn cancel_task(&mut self) -> Reply {
        match self.task.as_ref().and_then(|t| t.cancel.as_ref()) {
            Some(cancel) => {
                cancel.store(true, Ordering::Relaxed);
                ack("Cancelling")
            }
            None => Reply::Error(
                wire::ErrorCode::BadRequest,
                "there is nothing to cancel".to_owned(),
            ),
        }
    }

    fn recover(&mut self, accept: bool) -> Reply {
        let Some(offer) = self.recovery.clone() else {
            return Reply::Error(
                wire::ErrorCode::BadRequest,
                "there is no recovery offer".to_owned(),
            );
        };
        if !accept {
            self.recovery = None;
            self.urgent = true;
            self.event(wire::EventKind::Info, "Recovery was dismissed.");
            return ack("Dismissed");
        }
        let reply = self.start_load(offer.file);
        if matches!(reply, Reply::Response(_)) {
            self.recovery = None;
        }
        reply
    }

    fn worker(&mut self, report: WorkerReport) {
        match report {
            WorkerReport::Progress {
                task,
                stage,
                fraction,
            } => {
                if let Some(t) = self.task.as_mut().filter(|t| t.id == task) {
                    t.view.stage = stage;
                    t.view.fraction = fraction;
                    self.changed = true;
                }
            }
            WorkerReport::Done { task, outcome } => {
                if self.task.as_ref().is_none_or(|t| t.id != task) {
                    return; // abandoned after a contained panic
                }
                self.task = None;
                self.urgent = true;
                match *outcome {
                    Outcome::Created(Ok(sim)) => self.install(sim, Origin::Created),
                    Outcome::Created(Err(why)) => {
                        self.fail(format!("The world could not be created: {why}"));
                    }
                    Outcome::Loaded {
                        file,
                        result: Ok(sim),
                    } => self.install(sim, Origin::Loaded(file)),
                    Outcome::Loaded {
                        file,
                        result: Err(why),
                    } => self.fail(format!("{file} could not be loaded: {why}")),
                    Outcome::Cancelled => {
                        self.event(wire::EventKind::Info, "World creation was cancelled.");
                    }
                }
            }
        }
    }

    fn install(&mut self, mut sim: Sim, origin: Origin) {
        self.preserve_current("Before switching worlds");
        let dir_name = persist::world_dir_name(sim.meta());
        let dir = match SaveDir::create(self.config.saves_root.join(&dir_name), SAVE_EXTENSION) {
            Ok(dir) => dir,
            Err(e) => {
                self.fail(format!("The world's save folder could not be created: {e}"));
                return;
            }
        };
        if matches!(origin, Origin::Loaded(_)) {
            // A loaded world waits for the player.
            sim.set_paused(true);
        }
        // The session's deliberator, never saved (ADR-0039 §3).
        let mode = self.config.deliberator.mode_for(&sim);
        sim.set_deliberation(mode);
        let name = sim.meta().name.clone();
        let content_changed = sim.content_changed();
        let created_text = {
            let (meta, stats, map) = (sim.meta(), sim.stats(), sim.map());
            let preset = self
                .config
                .content
                .preset(&meta.preset_id)
                .map_or(meta.preset_id.as_str(), |p| p.name.as_str());
            let (km_x, km_y) = map.extent_m();
            format!(
                "Created “{name}” from seed {} ({preset}, {:.0} × {:.0} km): {} river reaches, {} lakes.",
                meta.seed,
                km_x / 1000.0,
                km_y / 1000.0,
                stats.reaches,
                stats.lakes
            )
        };
        let date = sim.date();
        self.world = Some(World {
            sim,
            dir,
            last_autosave: Instant::now(),
            last_autosave_unix_ms: 0,
            pause_at_midnight: false,
        });
        self.epoch = self.epoch.wrapping_add(1);
        self.recovery = None;
        self.last_error = None;
        self.marker.world_dir = Some(dir_name);
        self.marker.last_good = match &origin {
            Origin::Loaded(file) => Some(file.clone()),
            Origin::Created => None,
        };
        self.write_marker();
        match origin {
            Origin::Created => {
                self.event(wire::EventKind::WorldCreated, created_text);
                self.autosave("World created");
            }
            Origin::Loaded(file) => {
                self.event(
                    wire::EventKind::Loaded,
                    format!("Loaded “{name}” from {file} ({date})."),
                );
                if content_changed {
                    self.event(
                        wire::EventKind::Warning,
                        format!(
                            "“{name}” was saved with different content than is loaded now; it \
                             continues with the current content."
                        ),
                    );
                }
            }
        }
        self.urgent = true;
    }

    /// Autosaves the current world if it changed since its last save.
    fn preserve_current(&mut self, label: &str) {
        if self.world.as_ref().is_some_and(|w| w.sim.is_dirty()) {
            self.autosave(label);
        }
    }

    fn autosave(&mut self, label: &str) {
        let Some(world) = self.world.as_mut() else {
            return;
        };
        world.last_autosave = Instant::now();
        if let Err(e) = world.finish_day() {
            self.fail(format!("Autosave failed: {e}"));
            return;
        }
        let Some(world) = self.world.as_mut() else {
            return;
        };
        match persist::save(&mut world.sim, &world.dir, SaveKind::Autosave, label) {
            Ok(published) => {
                world.last_autosave_unix_ms = commons_persist::now_unix_ms();
                let pruned = world
                    .dir
                    .prune(SaveKind::Autosave, self.config.keep_autosaves);
                let text = format!(
                    "Autosaved “{}” at {} (generation {}).",
                    world.sim.meta().name,
                    world.sim.date(),
                    world.sim.generation()
                );
                if let Err(e) = pruned {
                    tracing::warn!("old autosaves could not be removed: {e}");
                }
                self.marker.last_good = session::relative(&self.config.saves_root, &published.path);
                self.write_marker();
                self.event(wire::EventKind::Autosaved, text);
            }
            Err(e) => self.fail(format!("Autosave failed: {e}")),
        }
        self.urgent = true;
    }

    /// Advances the clock by real time and autosaves when due. At Max the world lives day after
    /// day for most of the tick (at least one), stopping at a midnight; at other Accelerated
    /// speeds a day at a time when its time has come (ADR-0011 §2).
    pub fn tick(&mut self, real: Duration) {
        if self.run_ahead.is_some() {
            self.tick_run_ahead();
            return;
        }
        let budget = self.config.tick.mul_f64(RUN_AHEAD_SHARE);
        let started = Instant::now();
        let Some(world) = self.world.as_mut() else {
            return;
        };
        let step = real.min(MAX_STEP).as_secs_f64();
        let mut advance = civ_sim::Advance::default();
        loop {
            match world
                .sim
                .advance_real_in_chunks(step, ACCELERATED_CHUNK_MINUTES)
            {
                Ok(a) => {
                    advance.minutes += a.minutes;
                    advance.days += a.days;
                    advance.months += a.months;
                    advance.years += a.years;
                }
                Err(e) => {
                    world.sim.set_paused(true);
                    self.fail(format!("The clock stopped: {e}"));
                    return;
                }
            }
            let max = world.sim.speed() == civ_sim::SPEED_MAX && !world.sim.paused();
            if !max || started.elapsed() >= budget {
                break;
            }
        }
        // Accelerated mode publishes and saves only at a midnight (ADR-0011 §2); a pause asked
        // for during the day takes effect there.
        let at_rest = world.at_rest();
        if at_rest && world.pause_at_midnight {
            world.pause_at_midnight = false;
            world.sim.set_paused(true);
            self.urgent = true;
        }
        if advance.minutes > 0 && at_rest {
            self.changed = true;
        }
        let due =
            advance.months > 0 || world.last_autosave.elapsed() >= self.config.autosave_interval;
        if due && at_rest && world.sim.is_dirty() {
            self.autosave("Autosave");
        }
    }

    fn spawn_families(&mut self, at: (f32, f32), families: u32) -> Reply {
        let Some(world) = self.world.as_mut() else {
            return no_world();
        };
        if families > civ_agents::MAX_SPAWN_FAMILIES {
            return Reply::Error(
                wire::ErrorCode::BadRequest,
                format!(
                    "at most {} families can be sent at once",
                    civ_agents::MAX_SPAWN_FAMILIES
                ),
            );
        }
        match world.sim.spawn_families(at, families) {
            Ok(spawned) => {
                let first = &spawned[0];
                let n: usize = spawned.iter().map(|s| s.people.len()).sum();
                let text = match (spawned.len(), first.founded) {
                    (1, true) => format!("A family of {n} made camp at {}", first.name),
                    (1, false) => format!("A family of {n} came to {}", first.name),
                    (k, true) => format!("{k} families, {n} people, made camp at {}", first.name),
                    (k, false) => format!("{k} families, {n} people, came to {}", first.name),
                };
                self.changed = true;
                self.urgent = true;
                self.event(wire::EventKind::Info, text.clone());
                ack(&text)
            }
            Err(e) => Reply::Error(wire::ErrorCode::BadRequest, e),
        }
    }

    fn introduce_technique(&mut self, person: u64, technique: u32, aware_only: bool) -> Reply {
        if let Some(busy) = self.busy() {
            return busy;
        }
        let Some(world) = self.world.as_mut() else {
            return no_world();
        };
        let Some(def) = world
            .sim
            .rules()
            .catalog
            .techniques
            .get(technique as usize)
            .cloned()
        else {
            return Reply::Error(
                wire::ErrorCode::BadRequest,
                format!("there is no technique {technique}"),
            );
        };
        let Some(id) = civ_core::PermanentId::from_raw(person) else {
            return Reply::Error(wire::ErrorCode::NotFound, "nobody has id 0".to_owned());
        };
        let name = world.sim.people().name_of(id);
        match world.sim.introduce_technique(id, &def.id, aware_only) {
            Ok(()) => {
                let what = def.name.to_lowercase();
                let text = if aware_only {
                    format!("{name} heard of {what}")
                } else {
                    format!("{name} learnt {what} from the observer")
                };
                self.changed = true;
                self.urgent = true;
                self.event(wire::EventKind::Info, text.clone());
                ack(&text)
            }
            Err(e) => Reply::Error(wire::ErrorCode::BadRequest, e),
        }
    }

    /// The observer whispers claim `claim` to `person` (M4c slice AJ, ADR-0016 §5).
    fn whisper(&mut self, person: u64, claim: u32) -> Reply {
        if let Some(busy) = self.busy() {
            return busy;
        }
        let Some(world) = self.world.as_mut() else {
            return no_world();
        };
        let Some(id) = civ_core::PermanentId::from_raw(person) else {
            return Reply::Error(wire::ErrorCode::NotFound, "nobody has id 0".to_owned());
        };
        let name = world.sim.people().name_of(id);
        match world.sim.whisper(id, claim) {
            Ok(r) => {
                let text = if r.repeat {
                    format!("{name} heard it again; it is fresh to them, and nothing more")
                } else {
                    format!("{name} heard it, from no one")
                };
                self.changed = true;
                self.urgent = true;
                self.event(wire::EventKind::Info, text.clone());
                ack(&text)
            }
            Err(e) => Reply::Error(wire::ErrorCode::BadRequest, e),
        }
    }

    /// The observer tells `person` of ideology `ideology` (M4c slice AJ, ADR-0016 §5).
    fn tell_of_ideology(&mut self, person: u64, ideology: u32) -> Reply {
        if let Some(busy) = self.busy() {
            return busy;
        }
        let Some(world) = self.world.as_mut() else {
            return no_world();
        };
        let Some(def) = world
            .sim
            .rules()
            .catalog
            .ideologies
            .get(ideology as usize)
            .cloned()
        else {
            return Reply::Error(
                wire::ErrorCode::BadRequest,
                format!("there is no ideology {ideology}"),
            );
        };
        let Some(id) = civ_core::PermanentId::from_raw(person) else {
            return Reply::Error(wire::ErrorCode::NotFound, "nobody has id 0".to_owned());
        };
        let name = world.sim.people().name_of(id);
        match world.sim.tell_of_ideology(id, &def.id) {
            Ok(r) => {
                let what = def.name.to_lowercase();
                let came = if r.holds {
                    "and holds to it".to_owned()
                } else {
                    format!(
                        "and did not take it up: it fits what they hold dear by {:+.2}",
                        r.fit
                    )
                };
                let text = if r.repeat {
                    format!("{name} had heard of {what} from the observer already, {came}")
                } else {
                    format!("{name} heard of {what}, {came}")
                };
                self.changed = true;
                self.urgent = true;
                self.event(wire::EventKind::Info, text.clone());
                ack(&text)
            }
            Err(e) => Reply::Error(wire::ErrorCode::BadRequest, e),
        }
    }

    /// The observer sends an agitator holding ideology `ideology` to `at` (M4c slice AJ).
    fn send_agitator(&mut self, at: (f32, f32), ideology: u32) -> Reply {
        if let Some(busy) = self.busy() {
            return busy;
        }
        let Some(world) = self.world.as_mut() else {
            return no_world();
        };
        let Some(def) = world
            .sim
            .rules()
            .catalog
            .ideologies
            .get(ideology as usize)
            .cloned()
        else {
            return Reply::Error(
                wire::ErrorCode::BadRequest,
                format!("there is no ideology {ideology}"),
            );
        };
        match world.sim.send_agitator(at, &def.id) {
            Ok((sent, _)) => {
                let name = sent
                    .people
                    .first()
                    .map_or_else(String::new, |&p| world.sim.people().name_of(p));
                let text = format!(
                    "{name}, who holds to {}, {} {}",
                    def.name.to_lowercase(),
                    if sent.founded {
                        "made camp at"
                    } else {
                        "came to"
                    },
                    sent.name
                );
                self.changed = true;
                self.urgent = true;
                self.event(wire::EventKind::Info, text.clone());
                ack(&text)
            }
            Err(e) => Reply::Error(wire::ErrorCode::BadRequest, e),
        }
    }

    /// The observer sends a migration wave to `at` (M5a slice AN).
    fn send_wave(&mut self, at: (f32, f32), households: u32, days: u32, months: u32) -> Reply {
        if let Some(busy) = self.busy() {
            return busy;
        }
        let Some(world) = self.world.as_mut() else {
            return no_world();
        };
        match world.sim.send_wave(at, households, days, months) {
            Ok((_, came)) => {
                let Some(first) = came.first() else {
                    return Reply::Error(
                        wire::ErrorCode::BadRequest,
                        "no household of the wave came".to_owned(),
                    );
                };
                let people: usize = came.iter().map(|s| s.people.len()).sum();
                let rest = households as usize - came.len();
                let text = format!(
                    "A wave of {households} households: {} ({people} people) {} {}{}",
                    came.len(),
                    if first.founded {
                        "made camp at"
                    } else {
                        "came to"
                    },
                    first.name,
                    match rest {
                        0 => String::new(),
                        n => format!(", {n} more to come in the days after"),
                    }
                );
                self.changed = true;
                self.urgent = true;
                self.event(wire::EventKind::Info, text.clone());
                ack(&text)
            }
            Err(e) => Reply::Error(wire::ErrorCode::BadRequest, e),
        }
    }

    /// The observer blesses or curses `person` (M4c slice AJ).
    fn bless(&mut self, person: u64, curse: bool, days: u32, share: f32) -> Reply {
        if let Some(busy) = self.busy() {
            return busy;
        }
        let Some(world) = self.world.as_mut() else {
            return no_world();
        };
        let Some(id) = civ_core::PermanentId::from_raw(person) else {
            return Reply::Error(wire::ErrorCode::NotFound, "nobody has id 0".to_owned());
        };
        let name = world.sim.people().name_of(id);
        match world.sim.bless(id, curse, days, share) {
            Ok(r) => {
                let what = if curse { "cursed" } else { "blessed" };
                let text = if r.repeat {
                    format!("{name} was {what} already; it holds at least {days} days from now")
                } else {
                    format!("{name} is {what} for {days} days")
                };
                self.changed = true;
                self.urgent = true;
                self.event(wire::EventKind::Info, text.clone());
                ack(&text)
            }
            Err(e) => Reply::Error(wire::ErrorCode::BadRequest, e),
        }
    }

    /// The observer brings disease number `disease` (in `Welcome.diseases`) to `person` (M6a
    /// slice AZ, ADR-0021 §8).
    fn plague(&mut self, person: u64, disease: u32) -> Reply {
        if let Some(busy) = self.busy() {
            return busy;
        }
        let Some(world) = self.world.as_mut() else {
            return no_world();
        };
        let Some(id) = civ_core::PermanentId::from_raw(person) else {
            return Reply::Error(wire::ErrorCode::NotFound, "nobody has id 0".to_owned());
        };
        let Some(def) = world.sim.rules().catalog.diseases.get(disease as usize) else {
            return Reply::Error(
                wire::ErrorCode::BadRequest,
                format!("no disease numbered {disease}"),
            );
        };
        let (key, what) = (def.id.clone(), def.name.to_lowercase());
        let name = world.sim.people().name_of(id);
        match world.sim.plague(id, &key) {
            Ok(_) => {
                let text = format!("{name} has {what}, as if they took it elsewhere");
                self.changed = true;
                self.urgent = true;
                self.event(wire::EventKind::Info, text.clone());
                ack(&text)
            }
            Err(e) => Reply::Error(wire::ErrorCode::BadRequest, e),
        }
    }

    /// The observer sets building `building`'s covering alight (M6b slice BC, ADR-0022 §14).
    fn set_fire(&mut self, building: u64) -> Reply {
        if let Some(busy) = self.busy() {
            return busy;
        }
        let Some(world) = self.world.as_mut() else {
            return no_world();
        };
        let Some(id) = civ_core::PermanentId::from_raw(building) else {
            return Reply::Error(wire::ErrorCode::NotFound, "no building has id 0".to_owned());
        };
        let name = frames::incidents::building_words(&world.sim, id);
        match world.sim.set_fire(id) {
            Ok(_) => {
                let text = format!("The observer set alight {name}");
                self.changed = true;
                self.urgent = true;
                self.event(wire::EventKind::Info, text.clone());
                ack(&text)
            }
            Err(e) => Reply::Error(wire::ErrorCode::BadRequest, e),
        }
    }

    /// The observer raises reach runoff for stated days (M6b slice BF, ADR-0022 §14).
    fn raise_river(&mut self, reach: u32, discharge_m3s: f32, days: u32) -> Reply {
        if let Some(busy) = self.busy() {
            return busy;
        }
        let Some(world) = self.world.as_mut() else {
            return no_world();
        };
        match world.sim.raise_river(reach as usize, discharge_m3s, days) {
            Ok(_) => {
                let text = format!(
                    "The observer raised reach {reach} to {discharge_m3s:.1} m³/s for {days} days"
                );
                self.changed = true;
                self.urgent = true;
                self.event(wire::EventKind::Info, text.clone());
                ack(&text)
            }
            Err(e) => Reply::Error(wire::ErrorCode::BadRequest, e),
        }
    }

    /// The observer sends a storm today of chosen strength and direction (M6b slice BF, ADR-0022 §14).
    fn send_storm(&mut self, wind_speed_ms: f32, direction_deg: f32) -> Reply {
        if let Some(busy) = self.busy() {
            return busy;
        }
        let Some(world) = self.world.as_mut() else {
            return no_world();
        };
        match world.sim.send_storm(wind_speed_ms, direction_deg) {
            Ok(_) => {
                let text = format!(
                    "The observer sent a gale of {wind_speed_ms:.1} m/s from {direction_deg:.0}°"
                );
                self.changed = true;
                self.urgent = true;
                self.event(wire::EventKind::Info, text.clone());
                ack(&text)
            }
            Err(e) => Reply::Error(wire::ErrorCode::BadRequest, e),
        }
    }

    /// The observer shakes the ground under settlement `settlement`, or the whole map when it is
    /// 0, with a lateral share `strength` of every building's weight (M8d slice DA, ADR-0031 §4).
    fn shake(&mut self, strength: f32, settlement: u64) -> Reply {
        if let Some(busy) = self.busy() {
            return busy;
        }
        let Some(world) = self.world.as_mut() else {
            return no_world();
        };
        let at = civ_core::PermanentId::from_raw(settlement);
        match world.sim.shake(f64::from(strength), at) {
            Ok(_) => {
                let q = world.sim.people().quakes.list.last();
                let count = |o: civ_agents::quake::Outcome| {
                    q.map_or(0, |q| q.shaken.iter().filter(|s| s.outcome == o).count())
                };
                let text = format!(
                    "The observer shook the ground with {strength:.2} of a building's weight: {} fell, {} cracked",
                    count(civ_agents::quake::Outcome::Fell),
                    count(civ_agents::quake::Outcome::Cracked),
                );
                self.changed = true;
                self.urgent = true;
                self.event(wire::EventKind::Info, text.clone());
                ack(&text)
            }
            Err(e) => Reply::Error(wire::ErrorCode::BadRequest, e),
        }
    }

    /// The observer holds back the rain over the settlement for `months` months, `depth`
    /// standard deviations dry (M6a slice BA, ADR-0021 §8).
    /// The observer forces the climate (wire 1.96, ADR-0032 §9): what follows is the world's.
    fn force_climate(&mut self, offset_c: f32, wet_factor: f32, ramp_years: u32) -> Reply {
        if let Some(busy) = self.busy() {
            return busy;
        }
        let Some(world) = self.world.as_mut() else {
            return no_world();
        };
        match world
            .sim
            .force_climate(f64::from(offset_c), f64::from(wet_factor), ramp_years)
        {
            Ok(_) => {
                let over = match ramp_years {
                    0 => "from tomorrow".to_owned(),
                    1 => "over a year".to_owned(),
                    n => format!("over {n} years"),
                };
                let text = format!(
                    "The observer changes the climate: {offset_c:+.1} °C and wet days' rain \
                     ×{wet_factor:.2}, {over}"
                );
                self.changed = true;
                self.urgent = true;
                self.event(wire::EventKind::Info, text.clone());
                ack(&text)
            }
            Err(e) => Reply::Error(wire::ErrorCode::BadRequest, e),
        }
    }

    fn drought(&mut self, months: u32, depth: f32) -> Reply {
        if let Some(busy) = self.busy() {
            return busy;
        }
        let Some(world) = self.world.as_mut() else {
            return no_world();
        };
        match world.sim.drought(months, f64::from(depth)) {
            Ok(reached) => {
                let until = world.sim.land().weather.hold.map_or(0, |(_, until)| until);
                let text = format!(
                    "The observer {} the rain for {months} months, {depth:.1} standard deviations \
                     dry, until {}",
                    if reached.repeat {
                        "again holds back"
                    } else {
                        "holds back"
                    },
                    civ_agents::polity::day_words(SimTime::from_minutes(until * 1440))
                );
                self.changed = true;
                self.urgent = true;
                self.event(wire::EventKind::Info, text.clone());
                ack(&text)
            }
            Err(e) => Reply::Error(wire::ErrorCode::BadRequest, e),
        }
    }

    fn place_deposit(&mut self, at: (f32, f32), good: &str, radius_m: f32, exposed: bool) -> Reply {
        if let Some(busy) = self.busy() {
            return busy;
        }
        let Some(world) = self.world.as_mut() else {
            return no_world();
        };
        match world.sim.place_deposit(at, good, radius_m, exposed) {
            Ok(_) => {
                let what = world.sim.rules().catalog.good_index(good).map_or_else(
                    || good.to_owned(),
                    |g| world.sim.rules().catalog.goods[g].name.to_lowercase(),
                );
                let text = if exposed {
                    format!("The observer laid down {what} showing at the surface")
                } else {
                    format!("The observer laid down {what} under the ground")
                };
                self.changed = true;
                self.urgent = true;
                self.event(wire::EventKind::Info, text.clone());
                ack(&text)
            }
            Err(e) => Reply::Error(wire::ErrorCode::BadRequest, e),
        }
    }

    fn start_run_ahead(&mut self, minute: i64, skip: bool, stop_on: u8) -> Reply {
        if let Some(busy) = self.busy() {
            return busy;
        }
        let Some(world) = self.world.as_mut() else {
            return no_world();
        };
        let from = world.sim.now();
        let until = SimTime::from_minutes(minute);
        if until <= from {
            return Reply::Error(
                wire::ErrorCode::BadRequest,
                "that time has already passed".to_owned(),
            );
        }
        if until.minutes() - from.minutes() > MAX_RUN_AHEAD_MINUTES {
            return Reply::Error(
                wire::ErrorCode::BadRequest,
                "running ahead is limited to a hundred years at a time".to_owned(),
            );
        }
        // A skip lives at Accelerated Max with every settlement household-resolved, from the
        // next midnight, until it ends (ADR-0032 §7); at Max the clock stops only at midnights,
        // so it runs to the first at or after the time asked.
        let day = civ_core::time::MINUTES_PER_DAY;
        let until = if skip {
            SimTime::from_minutes((until.minutes() + day - 1).div_euclid(day) * day)
        } else {
            until
        };
        let skip = skip.then(|| Skip {
            from_seq: world.sim.chronicle_head(),
            restore: world.sim.begin_skip(),
            stop_on,
        });
        let cancel = Arc::new(AtomicBool::new(false));
        let name = if skip.is_some() {
            format!("Skipping to {until}")
        } else {
            format!("Running ahead to {until}")
        };
        let skipping = skip.is_some();
        let task = self.begin_task(name, Some(Arc::clone(&cancel)));
        self.run_ahead = Some(RunAhead {
            task,
            from,
            until,
            cancel,
            skip,
        });
        ack(if skipping {
            "Skipping ahead"
        } else {
            "Running ahead"
        })
    }

    /// Lives the world on in whole hours for most of a tick, then shows how far it got.
    fn tick_run_ahead(&mut self) {
        let budget = self.config.tick.mul_f64(RUN_AHEAD_SHARE);
        let started = Instant::now();
        let (Some(run), Some(world)) = (self.run_ahead.as_ref(), self.world.as_mut()) else {
            self.run_ahead = None;
            return;
        };
        let (until, cancelled) = (run.until, run.cancel.load(Ordering::Relaxed));
        let (task, from) = (run.task, run.from);
        let stop = run.skip.as_ref().map(|s| (s.from_seq, s.stop_on));
        let (stopped, ran) = if stop.is_some() {
            ("Stopped skipping", "Skipped")
        } else {
            ("Stopped running ahead", "Ran ahead")
        };
        if cancelled {
            self.finish_run_ahead(stopped);
            return;
        }
        let mut failed = None;
        let mut marked = None;
        while world.sim.now() < until && started.elapsed() < budget {
            // A skip goes a day at a time, to each midnight, and looks for the entries that stop
            // it after each.
            let now = world.sim.now().minutes();
            let day = civ_core::time::MINUTES_PER_DAY;
            let step = if stop.is_some() {
                day - now.rem_euclid(day)
            } else {
                60
            };
            match world.sim.advance_minutes((until.minutes() - now).min(step)) {
                Ok(advance) => {
                    if advance.minutes > 0 {
                        self.changed = true;
                    }
                }
                Err(e) => {
                    failed = Some(e.to_string());
                    break;
                }
            }
            if let Some((from_seq, stop_on)) = stop
                && let Some(e) = world.sim.skip_stop(from_seq, stop_on)
            {
                marked = Some(world.sim.chronicle_words(e));
                break;
            }
        }
        let now = world.sim.now();
        let autosave_due =
            world.last_autosave.elapsed() >= self.config.autosave_interval && world.sim.is_dirty();
        if let Some(e) = failed {
            self.finish_run_ahead(stopped);
            self.fail(format!("The clock stopped: {e}"));
            return;
        }
        if let Some(words) = marked {
            self.finish_run_ahead(stopped);
            self.event(
                wire::EventKind::Info,
                format!("The skip stopped at: {words}"),
            );
            return;
        }
        if now >= until {
            self.finish_run_ahead(ran);
            return;
        }
        if let Some(t) = self.task.as_mut().filter(|t| t.id == task) {
            let span = (until.minutes() - from.minutes()).max(1) as f32;
            t.view.fraction = (now.minutes() - from.minutes()) as f32 / span;
            t.view.stage = now.to_string();
        }
        if autosave_due {
            self.autosave("Autosave");
        }
    }

    /// Ends running ahead: the clock is paused where it got to, and the world is saved.
    fn finish_run_ahead(&mut self, what: &str) {
        let Some(run) = self.run_ahead.take() else {
            return;
        };
        if self.task.as_ref().is_some_and(|t| t.id == run.task) {
            self.task = None;
        }
        let Some(world) = self.world.as_mut() else {
            return;
        };
        // A skip puts back the speed and the resolutions it set aside.
        if let Some(skip) = run.skip {
            world.sim.end_skip(skip.restore);
        }
        world.sim.set_paused(true);
        let date = world.sim.now();
        let dirty = world.sim.is_dirty();
        self.urgent = true;
        self.event(wire::EventKind::Info, format!("{what} to {date}"));
        if dirty {
            self.autosave("Autosave");
        }
    }

    fn contain_panic(&mut self, message: &str) {
        tracing::error!("the engine panicked: {message}");
        if let Some(cancel) = self.task.take().and_then(|t| t.cancel) {
            cancel.store(true, Ordering::Relaxed);
        }
        let mut crash_note = String::new();
        if let Some(mut world) = self.world.take() {
            // For diagnosis only: the state may be inconsistent, so this snapshot is never
            // offered for recovery.
            let crash = catch_unwind(AssertUnwindSafe(|| {
                persist::save(
                    &mut world.sim,
                    &world.dir,
                    SaveKind::Crash,
                    "Crash snapshot",
                )
            }));
            crash_note = match crash {
                Ok(Ok(published)) => format!(
                    " A crash snapshot was written to {}.",
                    session::relative(&self.config.saves_root, &published.path).unwrap_or_default()
                ),
                _ => " No crash snapshot could be written.".to_owned(),
            };
        }
        self.epoch = self.epoch.wrapping_add(1);
        let reason = format!("The engine hit an internal error ({message}).");
        self.recovery = session::find_recovery(
            &self.config.saves_root,
            &self.marker,
            &self.config.content,
            &reason,
        );
        self.fail(format!("{reason}{crash_note}"));
    }

    fn fail(&mut self, text: String) {
        tracing::warn!("{text}");
        self.last_error = Some(text.clone());
        self.event(wire::EventKind::Failure, text);
        self.urgent = true;
    }

    fn shutdown(&mut self) {
        if let Some(cancel) = self.task.as_ref().and_then(|t| t.cancel.as_ref()) {
            cancel.store(true, Ordering::Relaxed);
        }
        self.preserve_current("On exit");
        if let Err(e) = SessionMarker::remove(&self.config.saves_root) {
            tracing::warn!("the session marker could not be removed: {e}");
        }
        self.event(wire::EventKind::Info, "The host stopped.");
    }

    fn write_marker(&mut self) {
        if let Err(e) = self.marker.write(&self.config.saves_root) {
            tracing::warn!("the session marker could not be written: {e}");
        }
    }

    fn event(&mut self, kind: wire::EventKind, text: impl Into<String>) {
        let text = text.into();
        tracing::info!("{text}");
        self.next_event += 1;
        let sim_minute = self.world.as_ref().map_or(0, |w| w.sim.now().minutes());
        let record = EventRecord {
            id: self.next_event,
            sim_minute,
            unix_ms: commons_persist::now_unix_ms(),
            kind,
            text,
        };
        {
            let mut history = self.history.lock().unwrap_or_else(PoisonError::into_inner);
            history.push_back(record.clone());
            while history.len() > HISTORY {
                history.pop_front();
            }
        }
        let _ = self.events_tx.send(Arc::new(EventsOut {
            epoch: self.epoch,
            sim_time: sim_minute,
            events: vec![record],
        }));
    }

    fn publish_if_due(&mut self) {
        // Accelerated mode publishes only at a midnight (ADR-0011 §2): what changed or is urgent
        // partway through a day waits for its midnight, the day lived on to it.
        if self.world.as_ref().is_some_and(|w| !w.at_rest()) {
            return;
        }
        if self.urgent || (self.changed && self.last_publish.elapsed() >= SNAPSHOT_INTERVAL) {
            self.publish();
        }
    }

    /// Publishes a snapshot of the current state.
    pub fn publish(&mut self) {
        let world = self.world.as_ref();
        let payload = protocol::snapshot_payload(&SnapshotParts {
            sim: world.map(|w| &w.sim),
            task: self.task.as_ref().map(|t| &t.view),
            recovery: self.recovery.as_ref(),
            last_error: self.last_error.as_deref(),
            last_autosave_unix_ms: world.map_or(0, |w| w.last_autosave_unix_ms),
        });
        let sim_time = world.map_or(0, |w| w.sim.now().minutes());
        self.snapshot_tx.send_replace(Arc::new(SnapshotOut {
            epoch: self.epoch,
            sim_time,
            payload,
        }));
        self.changed = false;
        self.urgent = false;
        self.last_publish = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use civ_schema::flatbuffers;
    use std::path::Path;

    const PRESET: &str = "core:worldgen/river_valley";

    fn content() -> Arc<ContentRegistry> {
        let root = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
            .expect("content/ is above the crate");
        Arc::new(civ_content::load(&root).registry.expect("content loads"))
    }

    struct Harness {
        engine: Engine,
        rx: mpsc::Receiver<Msg>,
        dir: tempfile::TempDir,
    }

    impl Harness {
        fn new(previous: Option<SessionMarker>, dir: Option<tempfile::TempDir>) -> Harness {
            let dir = dir.unwrap_or_else(|| tempfile::tempdir().expect("temp dir"));
            let (tx, rx) = mpsc::channel();
            let config = EngineConfig::new(content(), dir.path().to_owned());
            let engine = Engine::new(config, tx, previous).expect("engine");
            Harness { engine, rx, dir }
        }

        fn ask(&mut self, request: Request) -> Reply {
            let (tx, mut rx) = oneshot::channel();
            self.engine.guarded(|e| e.handle(Msg::Request(request, tx)));
            rx.try_recv().unwrap_or(Reply::Error(
                wire::ErrorCode::Internal,
                "no reply".to_owned(),
            ))
        }

        /// Feeds worker reports to the engine until the running task ends.
        fn finish_task(&mut self) {
            while self.engine.task.is_some() {
                let msg = self
                    .rx
                    .recv_timeout(Duration::from_secs(120))
                    .expect("the worker reports");
                self.engine.guarded(|e| e.handle(msg));
            }
        }

        fn create(&mut self, name: &str) {
            let reply = self.ask(Request::NewWorld(NewWorld {
                name: name.to_owned(),
                seed: 9,
                preset_id: PRESET.to_owned(),
                size_cells: 256,
                band_size: 0,
                neighbours: Vec::new(),
                neighbours_known: false,
                regime_id: String::new(),
            }));
            assert!(matches!(reply, Reply::Response(_)), "{reply:?}");
            self.finish_task();
            assert!(self.engine.world.is_some(), "{:?}", self.engine.last_error);
        }

        fn saves(&mut self) -> Vec<protocol::SaveEntry> {
            session::list_saves(self.dir.path(), &self.engine.config.content).expect("lists")
        }
    }

    fn error_code(reply: &Reply) -> Option<wire::ErrorCode> {
        match reply {
            Reply::Error(code, _) => Some(*code),
            Reply::Response(_) => None,
        }
    }

    #[test]
    fn running_ahead_lives_the_world_on_and_pauses_where_it_arrives() {
        let mut h = Harness::new(None, None);
        h.create("Ahead");
        let start = h.engine.world.as_ref().expect("a world").sim.now();
        let target = start.plus_minutes(2 * civ_core::time::MINUTES_PER_DAY);
        assert!(matches!(
            h.ask(Request::RunUntil {
                minute: target.minutes(),
                skip: false,
                stop_on: 0,
            }),
            Reply::Response(_)
        ));
        let task = h.engine.task.as_ref().map(|t| t.view.clone());
        assert!(
            task.as_ref()
                .is_some_and(|t| t.cancellable && t.name.starts_with("Running ahead")),
            "{task:?}"
        );
        // Another long job waits for it.
        assert_eq!(
            error_code(&h.ask(Request::RunUntil {
                minute: target.minutes() + 10,
                skip: false,
                stop_on: 0,
            })),
            Some(wire::ErrorCode::Busy)
        );
        for _ in 0..10_000 {
            if h.engine.run_ahead.is_none() {
                break;
            }
            h.engine.tick(h.engine.config.tick);
        }
        let world = h.engine.world.as_ref().expect("a world");
        assert_eq!(world.sim.now(), target, "it stops where it was asked to");
        assert!(world.sim.paused(), "and waits there");
        assert!(h.engine.task.is_none());
        let history = h.engine.history.lock().expect("history");
        assert!(history.iter().any(|e| e.text.starts_with("Ran ahead to")));
        drop(history);
        // The past cannot be run to.
        assert_eq!(
            error_code(&h.ask(Request::RunUntil {
                minute: start.minutes(),
                skip: false,
                stop_on: 0,
            })),
            Some(wire::ErrorCode::BadRequest)
        );
    }

    #[test]
    fn at_max_a_pause_asked_for_during_a_day_takes_effect_at_its_midnight() {
        let mut h = Harness::new(None, None);
        h.create("Chunks");
        let world = h.engine.world.as_mut().expect("a world");
        world.sim.set_paused(false);
        world.sim.set_speed(civ_sim::SPEED_MAX).expect("Max");
        // Accelerated from the next midnight; then partway through a day, as the host's chunks
        // leave it.
        world.sim.advance_to_midnight().expect("advances");
        assert_eq!(world.sim.mode(), Mode::Accelerated);
        world
            .sim
            .advance_real_in_chunks(0.001, ACCELERATED_CHUNK_MINUTES)
            .expect("advances");
        assert_eq!(
            world.sim.now().minute_of_day(),
            ACCELERATED_CHUNK_MINUTES,
            "a chunk into the day"
        );
        let reply = h.ask(Request::SetClock {
            paused: true,
            speed: civ_sim::SPEED_MAX,
        });
        assert!(matches!(&reply, Reply::Response(_)), "{reply:?}");
        let world = h.engine.world.as_ref().expect("a world");
        assert!(
            world.pause_at_midnight,
            "the pause waits for the day's midnight"
        );
        assert!(!world.sim.paused(), "the day is lived on to its midnight");
        for _ in 0..10_000 {
            if h.engine.world.as_ref().is_some_and(|w| w.sim.paused()) {
                break;
            }
            h.engine.tick(h.engine.config.tick);
        }
        let world = h.engine.world.as_ref().expect("a world");
        assert!(world.sim.paused(), "the clock stops");
        assert_eq!(world.sim.now().minute_of_day(), 0, "at the day's midnight");
        // A save asked for during a day is made at its midnight.
        let world = h.engine.world.as_mut().expect("a world");
        world.sim.set_paused(false);
        world
            .sim
            .advance_real_in_chunks(0.001, ACCELERATED_CHUNK_MINUTES)
            .expect("advances");
        assert!(matches!(
            h.ask(Request::SaveWorld {
                label: "Midday".to_owned()
            }),
            Reply::Response(_)
        ));
        let world = h.engine.world.as_ref().expect("a world");
        assert_eq!(world.sim.now().minute_of_day(), 0, "saved at the midnight");
    }

    #[test]
    fn at_max_a_paced_speed_chosen_during_a_day_takes_effect_at_its_midnight() {
        let mut h = Harness::new(None, None);
        h.create("Slower");
        let world = h.engine.world.as_mut().expect("a world");
        world.sim.set_paused(false);
        world.sim.set_speed(civ_sim::SPEED_MAX).expect("Max");
        world.sim.advance_to_midnight().expect("advances");
        world
            .sim
            .advance_real_in_chunks(0.001, ACCELERATED_CHUNK_MINUTES)
            .expect("advances");
        let day = world.sim.now().day_index();
        assert_ne!(world.sim.now().minute_of_day(), 0, "partway through a day");
        assert!(matches!(
            h.ask(Request::SetClock {
                paused: false,
                speed: 10.0,
            }),
            Reply::Response(_)
        ));
        // The day is lived on to its midnight in chunks, as at Max, not at ten times real time.
        for _ in 0..100 {
            if h.engine.world.as_ref().expect("a world").sim.mode() == Mode::Detailed {
                break;
            }
            h.engine.tick(h.engine.config.tick);
        }
        let world = h.engine.world.as_ref().expect("a world");
        assert_eq!(world.sim.mode(), Mode::Detailed, "10x from the midnight");
        assert_eq!(world.sim.now().day_index(), day + 1, "that day's midnight");
    }

    #[test]
    fn at_max_nothing_is_published_partway_through_a_day() {
        let mut h = Harness::new(None, None);
        h.create("Midday");
        let world = h.engine.world.as_mut().expect("a world");
        world.sim.set_paused(false);
        world.sim.set_speed(civ_sim::SPEED_MAX).expect("Max");
        world.sim.advance_to_midnight().expect("advances");
        world
            .sim
            .advance_real_in_chunks(0.001, ACCELERATED_CHUNK_MINUTES)
            .expect("advances");
        assert_ne!(world.sim.now().minute_of_day(), 0, "partway through a day");
        // A midnight's change not yet published, and something urgent, partway through the day.
        h.engine.publish();
        let before = h.engine.snapshot_rx.borrow().sim_time;
        h.engine.changed = true;
        h.engine.urgent = true;
        h.engine.last_publish = Instant::now() - 2 * SNAPSHOT_INTERVAL;
        let world = h.engine.world.as_mut().expect("a world");
        world
            .sim
            .advance_real_in_chunks(0.001, ACCELERATED_CHUNK_MINUTES)
            .expect("advances");
        h.engine.publish_if_due();
        assert_eq!(h.engine.snapshot_rx.borrow().sim_time, before, "held");
        assert!(
            h.engine.changed && h.engine.urgent,
            "and kept for the midnight"
        );
        let world = h.engine.world.as_mut().expect("a world");
        world.sim.advance_to_midnight().expect("advances");
        h.engine.publish_if_due();
        let published = h.engine.snapshot_rx.borrow().sim_time;
        assert!(
            published > before && published % (24 * 60) == 0,
            "{published}"
        );
    }

    #[test]
    fn skipping_lives_at_max_by_household_and_puts_the_speed_back() {
        // M9c slice DL (ADR-0032 §7).
        let mut h = Harness::new(None, None);
        h.create("Skip");
        let world = h.engine.world.as_ref().expect("a world");
        let (start, speed) = (world.sim.now(), world.sim.speed());
        let target = start.plus_minutes(3 * civ_core::time::MINUTES_PER_DAY);
        // At Max the clock stops at midnights: the skip runs to the first at or after it.
        let day = civ_core::time::MINUTES_PER_DAY;
        let midnight = SimTime::from_minutes((target.minutes() + day - 1).div_euclid(day) * day);
        assert!(matches!(
            h.ask(Request::RunUntil {
                minute: target.minutes(),
                skip: true,
                stop_on: 0,
            }),
            Reply::Response(_)
        ));
        let task = h.engine.task.as_ref().map(|t| t.view.name.clone());
        assert!(
            task.as_ref().is_some_and(|n| n.starts_with("Skipping to")),
            "{task:?}"
        );
        // While it runs: Max, and the policy that lives every settlement by household (the
        // kernel's tests show it does; how many days one tick lives depends on the machine).
        let sim = &h.engine.world.as_ref().expect("a world").sim;
        assert_eq!(sim.speed(), civ_sim::SPEED_MAX);
        assert!(sim.focus_policy().is_some());
        for _ in 0..10_000 {
            if h.engine.run_ahead.is_none() {
                break;
            }
            h.engine.tick(h.engine.config.tick);
        }
        let world = h.engine.world.as_ref().expect("a world");
        assert_eq!(world.sim.now(), midnight);
        assert!(world.sim.paused());
        assert_eq!(world.sim.speed(), speed, "the speed put back");
        assert!(world.sim.focus_policy().is_none());
        let history = h.engine.history.lock().expect("history");
        assert!(history.iter().any(|e| e.text.starts_with("Skipped to")));
    }

    #[test]
    fn running_ahead_stops_when_cancelled() {
        let mut h = Harness::new(None, None);
        h.create("Stop");
        let start = h.engine.world.as_ref().expect("a world").sim.now();
        let target = start.plus_minutes(5 * civ_core::time::MINUTES_PER_YEAR);
        assert!(matches!(
            h.ask(Request::RunUntil {
                minute: target.minutes(),
                skip: false,
                stop_on: 0,
            }),
            Reply::Response(_)
        ));
        h.engine.tick(h.engine.config.tick);
        assert!(matches!(h.ask(Request::CancelTask), Reply::Response(_)));
        h.engine.tick(h.engine.config.tick);
        assert!(h.engine.run_ahead.is_none() && h.engine.task.is_none());
        let world = h.engine.world.as_ref().expect("a world");
        assert!(world.sim.now() > start && world.sim.now() < target);
        assert!(world.sim.paused());
    }

    #[test]
    fn the_observer_can_send_a_family() {
        let mut h = Harness::new(None, None);
        h.create("Families");
        let (hearth, before) = {
            let sim = &h.engine.world.as_ref().expect("a world").sim;
            (sim.land().settlements[0].hearth_m, sim.people().living())
        };
        let reply = h.ask(Request::SpawnFamily {
            at: hearth,
            families: 1,
        });
        let Reply::Response(ack) = reply else {
            panic!("a family arrives: {reply:?}");
        };
        let message = flatbuffers::root::<wire::Response>(&ack)
            .expect("decodes")
            .body_as_ack()
            .and_then(|a| a.message())
            .unwrap_or_default()
            .to_owned();
        assert!(message.starts_with("A family of "), "{message}");
        let sim = &h.engine.world.as_ref().expect("a world").sim;
        assert!(sim.people().living() > before);
        assert_eq!(
            error_code(&h.ask(Request::SpawnFamily {
                at: (-1.0, -1.0),
                families: 1
            })),
            Some(wire::ErrorCode::BadRequest)
        );

        let households = |h: &Harness| {
            let sim = &h.engine.world.as_ref().expect("a world").sim;
            sim.people().households.len()
        };
        let before = households(&h);
        let reply = h.ask(Request::SpawnFamily {
            at: hearth,
            families: 3,
        });
        let Reply::Response(ack) = reply else {
            panic!("three families arrive: {reply:?}");
        };
        let message = flatbuffers::root::<wire::Response>(&ack)
            .expect("decodes")
            .body_as_ack()
            .and_then(|a| a.message())
            .unwrap_or_default()
            .to_owned();
        assert!(message.starts_with("3 families, "), "{message}");
        assert_eq!(households(&h), before + 3);
        assert_eq!(
            error_code(&h.ask(Request::SpawnFamily {
                at: hearth,
                families: civ_agents::MAX_SPAWN_FAMILIES + 1
            })),
            Some(wire::ErrorCode::BadRequest),
            "no more than the host allows at once"
        );
        assert_eq!(households(&h), before + 3);
    }

    #[test]
    fn the_earthworks_are_read_with_the_tiles_they_changed() {
        let mut h = Harness::new(None, None);
        assert_eq!(
            error_code(&h.ask(Request::GetEarthworks)),
            Some(wire::ErrorCode::NoWorld)
        );
        h.create("Earthworks");
        let (rev, tiles_x, works) = {
            let sim = &h.engine.world.as_ref().expect("a world").sim;
            (
                frames::earthworks::earthworks_rev(sim),
                sim.map().width.div_ceil(civ_land::earth::DELTA_TILE),
                sim.land().earthworks.len(),
            )
        };
        let Reply::Response(payload) = h.ask(Request::GetEarthworks) else {
            panic!("the earthworks are read");
        };
        let response = flatbuffers::root::<wire::Response>(&payload).expect("decodes");
        let list = response.body_as_earthworks().expect("earthworks");
        assert_eq!(list.rev(), rev);
        assert_eq!(list.tile_cells(), civ_land::earth::DELTA_TILE);
        assert_eq!(list.tiles_x(), tiles_x);
        let listed = list.works().expect("a list");
        assert_eq!(listed.len(), works);
        assert!(
            listed
                .iter()
                .all(|w| !w.words().unwrap_or_default().is_empty())
        );
    }

    #[test]
    fn the_ways_are_read_with_their_stretches_rank_and_keeper() {
        use civ_land::ways::{Keeper, Rank, Segment, Surface, WALK, Way};
        let mut h = Harness::new(None, None);
        assert_eq!(
            error_code(&h.ask(Request::GetWays)),
            Some(wire::ErrorCode::NoWorld)
        );
        h.create("Ways");
        let read = |h: &mut Harness| {
            let Reply::Response(payload) = h.ask(Request::GetWays) else {
                panic!("the ways are read");
            };
            payload
        };
        let payload = read(&mut h);
        let response = flatbuffers::root::<wire::Response>(&payload).expect("decodes");
        let list = response.body_as_ways().expect("ways");
        assert_eq!(list.rev(), 0);
        assert_eq!(list.ways().map_or(0, |l| l.len()), 0);
        // A lane of ten cells eastward, cleared by the first household today, laid by hand.
        let sim = &mut h.engine.world.as_mut().expect("a world").sim;
        let household = sim
            .people()
            .households
            .iter()
            .map(|(_, x)| x.id)
            .min()
            .expect("a household");
        let id = sim.allocate_id_for_tests();
        let (w, today) = (sim.map().width, sim.now().day_index());
        let cells: Vec<u32> = (0..10).map(|k| 40 * w + 40 + k).collect();
        sim.land_mut_for_tests().ways.add(Way {
            id,
            begun: today,
            segments: vec![Segment {
                cells,
                corridor_m: 4.0,
                built_m: 1.5,
                usable_m: 1.5,
                surface: Surface::Cleared,
                keeper: Keeper::Household(household),
                kept: today,
            }],
            modes: WALK,
            trips: 0.0,
            trips_day: today,
            rank: Rank::Lane,
            track: None,
        });
        let rev = frames::ways::ways_rev(sim);
        assert_ne!(rev, 0);
        let payload = read(&mut h);
        let response = flatbuffers::root::<wire::Response>(&payload).expect("decodes");
        let list = response.body_as_ways().expect("ways");
        assert_eq!(list.rev(), rev);
        let way = list.ways().expect("a list").get(0);
        assert_eq!((way.id(), way.rank(), way.modes()), (id.get(), 0, 1));
        assert_eq!(way.rank_name(), Some("lane"));
        let words = way.words().unwrap_or_default();
        assert!(words.starts_with("Lane of 72 m, 1.5 m wide"), "{words}");
        let s = way.segments().expect("its stretches").get(0);
        // A straight line keeps only its ends.
        assert_eq!(s.points().map_or(0, |p| p.len()), 4);
        assert_eq!((s.usable_m(), s.corridor_m()), (1.5, 4.0));
        assert_eq!(s.surface(), Some("cleared ground"));
        assert!((s.condition() - 1.0).abs() < 1e-6);
        assert!(s.keeper().unwrap_or_default().ends_with("'s household"));
    }

    #[test]
    fn the_boats_and_carts_are_read_with_whose_they_are_and_how_far_made() {
        let mut h = Harness::new(None, None);
        assert_eq!(
            error_code(&h.ask(Request::GetVehicles)),
            Some(wire::ErrorCode::NoWorld)
        );
        h.create("Vehicles");
        let read = |h: &mut Harness| {
            let Reply::Response(payload) = h.ask(Request::GetVehicles) else {
                panic!("the vehicles are read");
            };
            payload
        };
        let payload = read(&mut h);
        let response = flatbuffers::root::<wire::Response>(&payload).expect("decodes");
        let list = response.body_as_vehicles().expect("vehicles");
        assert_eq!(list.rev(), 0);
        assert_eq!(list.vehicles().map_or(0, |l| l.len()), 0);
        // A dugout made and half worn, and a handcart 12 hours into its 40, both of the first
        // household and at its door.
        let sim = &mut h.engine.world.as_mut().expect("a world").sim;
        let household = sim
            .people()
            .households
            .iter()
            .map(|(_, x)| x.id)
            .min()
            .expect("a household");
        let door = civ_agents::population::cell_of(
            sim.map(),
            sim.people().household(household).expect("it").home,
        ) as u32;
        let index = |sim: &civ_sim::Sim, id: &str| {
            sim.rules().catalog.vehicle_index(id).expect("in the core") as u16
        };
        let (dugout, handcart) = (
            index(sim, "core:vehicle/dugout"),
            index(sim, "core:vehicle/handcart"),
        );
        let (boat, cart) = (sim.allocate_id_for_tests(), sim.allocate_id_for_tests());
        let today = sim.now().day_index();
        let one = |id, vehicle, wear, labour_h, work_h| civ_land::boats::Boat {
            id,
            vehicle,
            owner: household,
            at: door,
            made: today,
            wear,
            labour_h,
            work_h,
            saves_h_year: 0.0,
            crew: (0, 0),
        };
        let land = sim.land_mut_for_tests();
        land.boats.list.push(one(boat, dugout, 0.5, 0.0, 0.0));
        land.boats.changed();
        land.carts.list.push(one(cart, handcart, 0.0, 40.0, 12.0));
        land.carts.changed();
        let rev = frames::vehicles::vehicles_rev(sim);
        assert_ne!(rev, 0);
        let payload = read(&mut h);
        let response = flatbuffers::root::<wire::Response>(&payload).expect("decodes");
        let list = response.body_as_vehicles().expect("vehicles");
        assert_eq!(list.rev(), rev);
        let all = list.vehicles().expect("a list");
        assert_eq!(all.len(), 2);
        let (b, c) = (all.get(0), all.get(1));
        assert_eq!(
            (b.id(), b.mode(), b.name()),
            (boat.get(), 0, Some("Dugout"))
        );
        assert_eq!((c.id(), c.mode()), (cart.get(), 1));
        assert_eq!((b.made(), b.out_with(), b.load_kg()), (1.0, 0, 0.0));
        assert!((c.made() - 0.3).abs() < 1e-6 && b.wear() == 0.5);
        assert_eq!(b.owner(), household.get());
        let words = |v: wire::VehicleInfo| v.words().unwrap_or_default().to_owned();
        assert!(
            words(b).ends_with("dugout, made in year 1, half worn"),
            "{}",
            words(b)
        );
        assert!(
            words(c).ends_with("handcart, being made: 12 of 40 hours' work done"),
            "{}",
            words(c)
        );
    }

    #[test]
    fn the_land_map_is_read_from_what_was_recorded_and_changes_nothing() {
        // M8c slice CX (ADR-0030 §1): every plot, labelled paid, reckoned or nothing recorded,
        // drawn from what was recorded; reading it changes no digest.
        let mut h = Harness::new(None, None);
        assert_eq!(
            error_code(&h.ask(Request::GetLand)),
            Some(wire::ErrorCode::NoWorld)
        );
        h.create("Land");
        h.engine
            .world
            .as_mut()
            .expect("a world")
            .sim
            .advance_minutes(20 * 1440)
            .expect("lives");
        let sim = &h.engine.world.as_ref().expect("a world").sim;
        let before: Vec<_> = civ_sim::persist::encode_sections(sim)
            .into_iter()
            .map(|s| s.bytes)
            .collect();
        let Reply::Response(payload) = h.ask(Request::GetLand) else {
            panic!("the land map is read");
        };
        let sim = &h.engine.world.as_ref().expect("a world").sim;
        let after: Vec<_> = civ_sim::persist::encode_sections(sim)
            .into_iter()
            .map(|s| s.bytes)
            .collect();
        assert_eq!(before, after, "reading the land map changes nothing");
        let response = flatbuffers::root::<wire::Response>(&payload).expect("decodes");
        let land = response.body_as_land().expect("land");
        assert_eq!(land.rev(), frames::land::land_rev(sim));
        let plots = land.plots().expect("plots");
        assert_eq!(plots.len(), sim.land().plots.len());
        assert!(!plots.is_empty(), "homes were laid");
        assert!(
            plots.iter().any(|p| p.weighed() > 0),
            "a plot chosen by its receipt"
        );
        for (p, q) in plots.iter().zip(&sim.land().plots) {
            assert_eq!(p.weighed(), q.receipt.map_or(0, |r| r.weighed));
            assert_eq!(p.paid_kcal(), frames::land::paid_kcal(sim, q) as f32);
        }
    }

    #[test]
    fn the_plans_are_read_in_their_layers_with_what_stands_on_them() {
        use civ_agents::polity::{Encroachment, IssueKind, LawStatus};
        use civ_land::ways::{Keeper, Rank, Segment, Surface, WALK, Way};
        let mut h = Harness::new(None, None);
        assert_eq!(
            error_code(&h.ask(Request::GetPlans)),
            Some(wire::ErrorCode::NoWorld)
        );
        h.create("Plans");
        let read = |h: &mut Harness| {
            let Reply::Response(payload) = h.ask(Request::GetPlans) else {
                panic!("the plans are read");
            };
            payload
        };
        let payload = read(&mut h);
        let response = flatbuffers::root::<wire::Response>(&payload).expect("decodes");
        let list = response.body_as_plans().expect("plans");
        assert_eq!(list.rev(), 0);
        assert_eq!(list.plans().map_or(0, |l| l.len()), 0);
        // Two days on, someone of the first household puts the first layout forward.
        let sim = &mut h.engine.world.as_mut().expect("a world").sim;
        sim.advance_minutes(2 * 24 * 60).expect("lives");
        let settlement = sim.land().settlements[0].id;
        let (household, sponsor) = sim
            .people()
            .households
            .iter()
            .filter(|(_, x)| x.settlement == Some(settlement) && !x.members.is_empty())
            .map(|(_, x)| (x.id, x.members[0]))
            .min()
            .expect("a household");
        let law = sim
            .with_ctx_for_tests(|pop, ctx| {
                pop.propose_plan_for_tests(ctx, settlement, sponsor, 0, IssueKind::NoGround)
            })
            .expect("a plan put forward");
        let proposed = frames::plans::plans_rev(sim);
        assert_ne!(proposed, 0);
        let payload = read(&mut h);
        let response = flatbuffers::root::<wire::Response>(&payload).expect("decodes");
        let list = response.body_as_plans().expect("plans");
        assert_eq!(list.rev(), proposed);
        let all = list.plans().expect("a list");
        assert_eq!(all.len(), 1);
        let p = all.get(0);
        assert_eq!((p.law(), p.version(), p.in_force()), (law.get(), 1, false));
        let sim = &h.engine.world.as_ref().expect("a world").sim;
        let pi = sim
            .people()
            .polities
            .iter()
            .position(|x| x.holds(Some(settlement)))
            .expect("its polity");
        let laid = sim.people().polities[pi]
            .plan_of(law)
            .expect("its plan")
            .layout
            .clone();
        let n = |v: Option<flatbuffers::Vector<'_, wire::PlanRect>>| v.map_or(0, |v| v.len());
        assert_eq!(n(p.streets()), laid.streets.len());
        assert_eq!(n(p.sites()), laid.sites.len());
        assert_eq!(n(p.parcels()), laid.parcels.len());
        assert!(n(p.parcels()) > 0);
        assert_eq!(n(p.occupied()), 0);
        let first = laid.parcels[0].rect;
        let r = p.parcels().expect("parcels").get(0);
        assert_eq!(
            (r.x(), r.y(), r.w(), r.h()),
            (
                first.x as f32 / 100.0,
                first.y as f32 / 100.0,
                first.w as f32 / 100.0,
                first.h as f32 / 100.0
            )
        );
        let words = p.words().unwrap_or_default().to_owned();
        assert!(
            words.contains("version 1") && words.ends_with("put forward"),
            "{words}"
        );
        // A way laid over reserved ground: one cell whose centre lies on a street or the square.
        let sim = &mut h.engine.world.as_mut().expect("a world").sim;
        let map = sim.map();
        let on = |r: &civ_land::RectCm, c: u32| {
            let (x, y) = civ_agents::population::cell_centre(map, c as usize);
            let (x, y) = (f64::from(x) * 100.0, f64::from(y) * 100.0);
            x >= f64::from(r.x)
                && x < f64::from(r.x + r.w)
                && y >= f64::from(r.y)
                && y < f64::from(r.y + r.h)
        };
        let cell = laid
            .streets
            .iter()
            .chain(laid.sites.iter().map(|s| &s.rect))
            .find_map(|r| {
                let (x0, y0) = (r.x as f32 / 100.0, r.y as f32 / 100.0);
                let (x1, y1) = (x0 + r.w as f32 / 100.0, y0 + r.h as f32 / 100.0);
                let step = map.cell_size_m / 2.0;
                let mut y = y0;
                while y < y1 {
                    let mut x = x0;
                    while x < x1 {
                        let c = civ_agents::population::cell_of(map, (x, y)) as u32;
                        if on(r, c) {
                            return Some(c);
                        }
                        x += step;
                    }
                    y += step;
                }
                None
            })
            .expect("a cell on reserved ground");
        let (id, today) = (sim.allocate_id_for_tests(), sim.now().day_index());
        sim.land_mut_for_tests().ways.add(Way {
            id,
            begun: today,
            segments: vec![Segment {
                cells: vec![cell],
                corridor_m: 4.0,
                built_m: 1.5,
                usable_m: 1.5,
                surface: Surface::Cleared,
                keeper: Keeper::Household(household),
                kept: today,
            }],
            modes: WALK,
            trips: 0.0,
            trips_day: today,
            rank: Rank::Lane,
            track: None,
        });
        assert_ne!(frames::plans::plans_rev(sim), proposed);
        let payload = read(&mut h);
        let response = flatbuffers::root::<wire::Response>(&payload).expect("decodes");
        let p = response
            .body_as_plans()
            .and_then(|l| l.plans())
            .expect("a list")
            .get(0);
        assert!(n(p.built()) >= 1);
        let words = p.words().unwrap_or_default().to_owned();
        assert!(words.contains("; a way already runs over "), "{words}");
        // In force, with a building of the household's standing on one of its streets.
        let street = *laid.streets.first().expect("a street");
        let sim = &mut h.engine.world.as_mut().expect("a world").sim;
        let building = sim
            .land()
            .buildings
            .iter()
            .find(|b| b.standing())
            .map(|b| (b.id, b.plot))
            .expect("a building");
        {
            let land = sim.land_mut_for_tests();
            let q = land
                .plots
                .iter_mut()
                .find(|q| q.id == building.1)
                .expect("its plot");
            q.rect.x = street.x;
            q.rect.y = street.y;
        }
        let day = sim.now().day_index();
        {
            let polity = &mut sim.people_mut_for_tests().polities[pi];
            polity.law_mut(law).expect("the law").status = LawStatus::InForce;
            polity.encroachments.push(Encroachment {
                day,
                law,
                household,
                building: building.0,
                seen_by: Vec::new(),
                order: None,
            });
        }
        let rev = frames::plans::plans_rev(sim);
        assert!(rev != 0 && rev != proposed);
        let payload = read(&mut h);
        let response = flatbuffers::root::<wire::Response>(&payload).expect("decodes");
        let p = response
            .body_as_plans()
            .and_then(|l| l.plans())
            .expect("a list")
            .get(0);
        assert!(p.in_force());
        assert_eq!(n(p.occupied()), 1);
        let words = p.words().unwrap_or_default().to_owned();
        assert!(
            words.contains(", in force")
                && words.ends_with("; 1 building stands on its reserved ground"),
            "{words}"
        );
        // Its sponsor keeps it, and none of its parcels is given yet (M8c slice CU).
        assert!(words.contains(", in force; kept by "), "{words}");
        assert!(words.contains("; 0 of "), "{words}");
    }

    #[test]
    fn the_crossings_are_read_with_their_state_in_words() {
        use civ_land::crossings::{Crossing, CrossingOwner, CrossingState};
        let mut h = Harness::new(None, None);
        assert_eq!(
            error_code(&h.ask(Request::GetCrossings)),
            Some(wire::ErrorCode::NoWorld)
        );
        h.create("Crossings");
        let read = |h: &mut Harness| {
            let Reply::Response(payload) = h.ask(Request::GetCrossings) else {
                panic!("the crossings are read");
            };
            payload
        };
        let payload = read(&mut h);
        let response = flatbuffers::root::<wire::Response>(&payload).expect("decodes");
        let list = response.body_as_crossings().expect("crossings");
        assert_eq!(list.rev(), 0);
        assert_eq!(list.crossings().map_or(0, |l| l.len()), 0);
        // A log being built by the first household, laid by hand.
        let sim = &mut h.engine.world.as_mut().expect("a world").sim;
        let household = sim
            .people()
            .households
            .iter()
            .map(|(_, x)| x.id)
            .min()
            .expect("a household");
        let system = sim
            .rules()
            .catalog
            .bridge_index("core:bridge/log_beam")
            .expect("a log footbridge") as u16;
        let bearing_m = sim.rules().catalog.bridges[usize::from(system)].bearing_m as f32;
        let id = sim.allocate_id_for_tests();
        let (w, now) = (sim.map().width, sim.now());
        let cell = 40 * w + 40;
        sim.land_mut_for_tests().crossings.list.push(Crossing {
            id,
            system,
            banks: [cell - 1, cell + 1],
            cells: vec![cell],
            span_m: 3.0,
            members: 2,
            diameter_cm: 20.0,
            quality: 0.0,
            loss: 0.0,
            owner: CrossingOwner::Household(household),
            labour_h: 72.0,
            skill_h: 0.0,
            begun: now,
            state: CrossingState::Building { work_h: 18.0 },
            shares: Vec::new(),
            crew: (0, 0),
        });
        let rev = frames::crossings::crossings_rev(sim);
        assert_ne!(rev, 0);
        let payload = read(&mut h);
        let response = flatbuffers::root::<wire::Response>(&payload).expect("decodes");
        let list = response.body_as_crossings().expect("crossings");
        assert_eq!(list.rev(), rev);
        let c = list.crossings().expect("a list").get(0);
        assert_eq!((c.id(), c.state(), c.owner_kind()), (id.get(), 0, 0));
        assert_eq!(c.owner(), household.get());
        assert_eq!(c.system(), Some("Log footbridge"));
        assert!(c.ax() < c.bx() && (c.ay() - c.by()).abs() < 1e-3);
        // Its members are the span and a bearing on either bank long.
        assert!((c.length_m() - (3.0 + 2.0 * bearing_m)).abs() < 1e-4);
        let words = c.words().unwrap_or_default();
        assert!(
            words.contains("log footbridge, being built: 25% of 72 hours"),
            "{words}"
        );
    }

    #[test]
    fn the_water_is_read_with_its_wells_in_words_and_a_household_s_draws_in_the_inspector() {
        use civ_agents::uses::{Place, Worked};
        use civ_land::fields::RectCm;
        use civ_land::wells::{Well, WellState};
        let mut h = Harness::new(None, None);
        assert_eq!(
            error_code(&h.ask(Request::GetWater)),
            Some(wire::ErrorCode::NoWorld)
        );
        h.create("Water");
        let read = |h: &mut Harness| {
            let Reply::Response(payload) = h.ask(Request::GetWater) else {
                panic!("the water is read");
            };
            payload
        };
        let payload = read(&mut h);
        let response = flatbuffers::root::<wire::Response>(&payload).expect("decodes");
        let water = response.body_as_water().expect("water");
        assert_eq!(water.wells().map_or(0, |l| l.len()), 0);
        assert!(water.flow() > 0.0);
        let before = water.rev();
        // An open well of the first household, its water standing at the water table, and one
        // load drawn there and two at a bank today.
        let sim = &mut h.engine.world.as_mut().expect("a world").sim;
        let household = sim
            .people()
            .households
            .iter()
            .map(|(_, x)| x.id)
            .min()
            .expect("a household");
        let (person, settlement) = {
            let x = sim.people().household(household).expect("the household");
            (x.members[0], x.settlement.expect("a settlement"))
        };
        let system = sim
            .rules()
            .catalog
            .well_index("core:well/timber_lined")
            .expect("a timber-lined well");
        let id = sim.allocate_id_for_tests();
        let (w, now) = (sim.map().width, sim.now());
        let cell = 40 * w + 40;
        let (cx, cy) = civ_agents::population::cell_centre(sim.map(), cell as usize);
        let ground = sim.map().elevation[cell as usize];
        let head = sim.land().ground_water(cell as usize, w, 0.6, 50.0).0;
        let depth = (f64::from(ground) - head + 1.0) as f32;
        sim.land_mut_for_tests().wells.list.push(Well {
            id,
            system,
            cell,
            rect: RectCm {
                x: (cx * 100.0) as i32 - 75,
                y: (cy * 100.0) as i32 - 75,
                w: 150,
                h: 150,
            },
            household,
            polity: None,
            ground_m: ground,
            depth_m: depth,
            target_m: depth,
            work_h: 100.0,
            skill_h: 100.0,
            quality: 0.6,
            loss: 0.1,
            mend_h: 0.0,
            cover_h: 0.0,
            covered: false,
            worth: 1.0,
            level_m: head,
            level_at: now,
            drawn_l: 0.0,
            begun: now,
            state: WellState::Open { since: now },
            crew: (0, 0),
        });
        let bank = cell + 3;
        for place in [
            Place::Source(cell),
            Place::Source(bank),
            Place::Source(bank),
        ] {
            sim.people_mut_for_tests().uses.worked(Worked {
                place,
                day: now.day_index(),
                household,
                settlement,
                kcal: 0.0,
                person: Some(person),
            });
        }
        let rev = frames::water::water_rev(sim);
        assert_ne!(rev, 0);
        assert_ne!(rev, before);
        let payload = read(&mut h);
        let response = flatbuffers::root::<wire::Response>(&payload).expect("decodes");
        let water = response.body_as_water().expect("water");
        assert_eq!(water.rev(), rev);
        let well = water.wells().expect("a list").get(0);
        assert_eq!((well.id(), well.state()), (id.get(), 1));
        assert_eq!(well.household(), household.get());
        assert_eq!(well.system(), Some("Timber-lined well"));
        assert!((well.water_m() - 1.0).abs() < 1e-3, "{}", well.water_m());
        let words = well.words().unwrap_or_default();
        assert!(
            words.contains("timber-lined well, open since year 1"),
            "{words}"
        );
        assert!(
            words.contains("1.0 m of water standing") && words.contains("a tenth of its lining"),
            "{words}"
        );
        let banks = water.banks().expect("banks");
        assert_eq!(banks.len(), 1);
        assert_eq!(banks.get(0).trips(), 2);
        // The inspector tells where the household went for water today.
        let Reply::Response(payload) = h.ask(Request::GetPerson {
            id: person.get(),
            decisions: 0,
        }) else {
            panic!("the person is read");
        };
        let response = flatbuffers::root::<wire::Response>(&payload).expect("decodes");
        let info = response.body_as_person_info().expect("a person");
        let words = info.water().unwrap_or_default();
        assert!(
            words.starts_with(
                "Their household went for water 3 times today: twice to the water's edge, \
                 once to its own well; each of them uses"
            ),
            "{words}"
        );
    }

    #[test]
    fn the_weather_is_read_month_by_month_and_told_on_the_clock() {
        let mut h = Harness::new(None, None);
        assert_eq!(
            error_code(&h.ask(Request::GetWeather)),
            Some(wire::ErrorCode::NoWorld)
        );
        h.create("Weather");
        let (rev, months) = {
            let sim = &h.engine.world.as_ref().expect("a world").sim;
            (
                frames::weather::weather_rev(sim),
                sim.land().weather.months.len(),
            )
        };
        let Reply::Response(payload) = h.ask(Request::GetWeather) else {
            panic!("the weather is read");
        };
        let response = flatbuffers::root::<wire::Response>(&payload).expect("decodes");
        let report = response.body_as_weather_report().expect("a weather report");
        assert_eq!(report.rev(), rev);
        assert!(rev > 0);
        let listed = report.months().expect("months");
        // A new world has lived the year before its founding: twelve months or more.
        assert_eq!(listed.len(), months);
        assert!(months >= 12);
        assert!(listed.iter().all(|m| m.usual_mm() > 0.0 && m.days() > 0));
        let today = report.today().expect("today");
        assert!(today.words().unwrap_or_default().contains("°C"));
        // Wire 1.25: the snow line, infinitely high when snow lies nowhere.
        let line = h
            .engine
            .world
            .as_ref()
            .expect("a world")
            .sim
            .land()
            .weather
            .snow_line_m();
        assert_eq!(f64::from(today.snow_line_m()), line as f32 as f64);
    }

    #[test]
    fn the_observer_can_lay_down_a_deposit_and_read_them_all() {
        let mut h = Harness::new(None, None);
        h.create("Deposits");
        let (hearth, before) = {
            let sim = &h.engine.world.as_ref().expect("a world").sim;
            (
                sim.land().settlements[0].hearth_m,
                sim.land().deposits.len(),
            )
        };
        let reply = h.ask(Request::PlaceDeposit {
            at: hearth,
            good: "core:good/clay".to_owned(),
            radius_m: 8.0,
            exposed: true,
        });
        let Reply::Response(ack) = reply else {
            panic!("clay is laid down: {reply:?}");
        };
        let message = flatbuffers::root::<wire::Response>(&ack)
            .expect("decodes")
            .body_as_ack()
            .and_then(|a| a.message())
            .unwrap_or_default()
            .to_owned();
        assert_eq!(
            message,
            "The observer laid down clay showing at the surface"
        );
        assert_eq!(
            error_code(&h.ask(Request::PlaceDeposit {
                at: hearth,
                good: "core:good/nothing".to_owned(),
                radius_m: 8.0,
                exposed: true,
            })),
            Some(wire::ErrorCode::BadRequest)
        );
        let Reply::Response(payload) = h.ask(Request::GetDeposits) else {
            panic!("the deposits are read");
        };
        let response = flatbuffers::root::<wire::Response>(&payload).expect("decodes");
        let deposits = response.body_as_deposits().expect("deposits");
        let list = deposits.deposits().expect("a list");
        assert_eq!(list.len(), before + 1);
        let laid = list.get(list.len() - 1);
        assert!(laid.exposed());
        assert!((laid.radius_m() - 8.0).abs() < 1e-6);
        assert!((laid.x() - hearth.0).abs() < 0.01 && (laid.y() - hearth.1).abs() < 0.01);
        assert!(laid.left_kg() > 0.0);
        assert_ne!(deposits.rev(), 0);
    }

    #[test]
    fn the_observer_can_tell_someone_of_an_ideology_but_not_whisper_what_is_not_news() {
        let mut h = Harness::new(None, None);
        h.create("Influences");
        let person = {
            let sim = &h.engine.world.as_ref().expect("a world").sim;
            let now = sim.now();
            let grown = sim.rules().people.family.independent_age;
            sim.people()
                .people
                .iter()
                .map(|(_, p)| p)
                .filter(|p| p.age_years(now) >= grown)
                .filter(|p| !sim.people().ideologies.holds(p.id, 0))
                .map(|p| p.id.get())
                .min()
                .expect("an adult")
        };
        let reply = h.ask(Request::TellOfIdeology {
            person,
            ideology: 0,
        });
        let Reply::Response(ack) = reply else {
            panic!("told: {reply:?}");
        };
        let message = flatbuffers::root::<wire::Response>(&ack)
            .expect("decodes")
            .body_as_ack()
            .and_then(|a| a.message())
            .unwrap_or_default()
            .to_owned();
        assert!(message.contains("heard of"), "{message}");
        let sim = &h.engine.world.as_ref().expect("a world").sim;
        assert_eq!(sim.people().influences.list.len(), 1);
        assert_eq!(
            error_code(&h.ask(Request::Whisper {
                person,
                claim: 9_999,
            })),
            Some(wire::ErrorCode::BadRequest)
        );
        assert_eq!(
            error_code(&h.ask(Request::TellOfIdeology {
                person,
                ideology: 999,
            })),
            Some(wire::ErrorCode::BadRequest)
        );
        // A blessing, and an agitator sent to the hearth.
        let Reply::Response(_) = h.ask(Request::Bless {
            person,
            curse: false,
            days: 30,
            share: 0.25,
        }) else {
            panic!("blessed");
        };
        assert_eq!(
            error_code(&h.ask(Request::Bless {
                person,
                curse: true,
                days: 30,
                share: 0.75,
            })),
            Some(wire::ErrorCode::BadRequest)
        );
        let hearth = h
            .engine
            .world
            .as_ref()
            .expect("a world")
            .sim
            .land()
            .settlements[0]
            .hearth_m;
        let reply = h.ask(Request::SendAgitator {
            at: hearth,
            ideology: 0,
        });
        let Reply::Response(ack) = reply else {
            panic!("sent: {reply:?}");
        };
        let message = flatbuffers::root::<wire::Response>(&ack)
            .expect("decodes")
            .body_as_ack()
            .and_then(|a| a.message())
            .unwrap_or_default()
            .to_owned();
        assert!(message.contains("who holds to"), "{message}");
        let sim = &h.engine.world.as_ref().expect("a world").sim;
        assert_eq!(sim.people().influences.list.len(), 3);
        // A migration wave to the hearth: refused outside its ranges, else the first come at once.
        assert_eq!(
            error_code(&h.ask(Request::SendWave {
                at: hearth,
                households: 51,
                days: 1,
                months: 6,
            })),
            Some(wire::ErrorCode::BadRequest)
        );
        let reply = h.ask(Request::SendWave {
            at: hearth,
            households: 6,
            days: 2,
            months: 6,
        });
        let Reply::Response(ack) = reply else {
            panic!("sent: {reply:?}");
        };
        let message = flatbuffers::root::<wire::Response>(&ack)
            .expect("decodes")
            .body_as_ack()
            .and_then(|a| a.message())
            .unwrap_or_default()
            .to_owned();
        assert!(
            message.starts_with("A wave of 6 households: 3 ") && message.contains("3 more"),
            "{message}"
        );
        let sim = &h.engine.world.as_ref().expect("a world").sim;
        assert_eq!(sim.people().influences.list.len(), 4);
        assert_eq!(sim.people().influences.waves.len(), 1);
    }

    #[test]
    fn the_observer_can_bring_a_disease_to_someone_once_and_the_inspector_tells_of_it() {
        let mut h = Harness::new(None, None);
        h.create("Sickness");
        let (person, cholera) = {
            let sim = &h.engine.world.as_ref().expect("a world").sim;
            let person = sim
                .people()
                .people
                .iter()
                .map(|(_, p)| p.id.get())
                .min()
                .expect("someone");
            let cholera = sim
                .rules()
                .catalog
                .disease_index("core:disease/cholera")
                .expect("cholera in the core content");
            (person, u32::try_from(cholera).expect("a small index"))
        };
        let reply = h.ask(Request::Plague {
            person,
            disease: cholera,
        });
        let Reply::Response(ack) = reply else {
            panic!("brought: {reply:?}");
        };
        let message = flatbuffers::root::<wire::Response>(&ack)
            .expect("decodes")
            .body_as_ack()
            .and_then(|a| a.message())
            .unwrap_or_default()
            .to_owned();
        assert!(
            message.ends_with("has cholera, as if they took it elsewhere"),
            "{message}"
        );
        // Someone already infected cannot be given it again; nor can nobody, nor a disease the
        // content does not have.
        assert_eq!(
            error_code(&h.ask(Request::Plague {
                person,
                disease: cholera,
            })),
            Some(wire::ErrorCode::BadRequest)
        );
        assert_eq!(
            error_code(&h.ask(Request::Plague {
                person: 0,
                disease: cholera,
            })),
            Some(wire::ErrorCode::NotFound)
        );
        assert_eq!(
            error_code(&h.ask(Request::Plague {
                person,
                disease: 999,
            })),
            Some(wire::ErrorCode::BadRequest)
        );
        let Reply::Response(payload) = h.ask(Request::GetPerson {
            id: person,
            decisions: 0,
        }) else {
            panic!("the person is read");
        };
        let response = flatbuffers::root::<wire::Response>(&payload).expect("decodes");
        let info = response.body_as_person_info().expect("a person");
        let lines: Vec<&str> = info.sickness().expect("sickness words").iter().collect();
        assert_eq!(lines.len(), 1, "{lines:?}");
        assert!(
            lines[0].starts_with("Took cholera today")
                && lines[0].contains("; brought by the observer, as if they took it elsewhere"),
            "{lines:?}"
        );
    }

    #[test]
    fn the_observer_shakes_the_ground_and_is_told_what_fell() {
        let mut h = Harness::new(None, None);
        let shake = |strength: f32, settlement: u64| Request::Shake {
            strength,
            settlement,
        };
        assert_eq!(
            error_code(&h.ask(shake(0.2, 0))),
            Some(wire::ErrorCode::NoWorld)
        );
        h.create("Shaking");
        assert_eq!(
            error_code(&h.ask(shake(0.9, 0))),
            Some(wire::ErrorCode::BadRequest)
        );
        assert_eq!(
            error_code(&h.ask(shake(0.2, 999_999))),
            Some(wire::ErrorCode::BadRequest)
        );
        let settlement = h
            .engine
            .world
            .as_ref()
            .expect("a world")
            .sim
            .land()
            .settlements[0]
            .id
            .get();
        // The strongest the tool sends, which reaches the host in single precision.
        let reply = h.ask(shake(0.6, settlement));
        let Reply::Response(ack) = reply else {
            panic!("shaken: {reply:?}");
        };
        let message = flatbuffers::root::<wire::Response>(&ack)
            .expect("decodes")
            .body_as_ack()
            .and_then(|a| a.message())
            .unwrap_or_default()
            .to_owned();
        assert!(
            message.starts_with("The observer shook the ground with 0.60 of a building's weight: "),
            "{message}"
        );
        let sim = &h.engine.world.as_ref().expect("a world").sim;
        let q = sim.people().quakes.list.last().expect("a record");
        assert_eq!(q.settlement.map(|s| s.get()), Some(settlement));
    }

    #[test]
    fn what_the_shaking_brings_down_is_kept_out_of_the_failures_row() {
        let mut h = Harness::new(None, None);
        h.create("Shaken failures");
        let failed = |sim: &civ_sim::Sim| {
            sim.people()
                .chronicle
                .iter()
                .filter(|e| e.kind == civ_agents::ChronicleKind::BuildingFailed)
                .count()
        };
        let counted = |sim: &civ_sim::Sim| {
            let mut world = crate::dashboard::WorldRun::default();
            crate::dashboard::count_failures(sim, &mut world);
            world.lived_failures + world.empty_failures
        };
        // A longhouse of two storeys of untied rubble beside the first home, which the strongest
        // shaking overturns.
        {
            use civ_agents::build::{self, FireDesign, Shape};
            let sim = &mut h.engine.world.as_mut().expect("a world").sim;
            let (household, home) = sim
                .people()
                .households
                .iter()
                .map(|(_, x)| (x.id, x.home))
                .min_by_key(|x| x.0)
                .expect("a household");
            let catalog = &sim.rules().catalog;
            let def = &catalog.buildings[catalog
                .building_index("core:building/longhouse")
                .expect("longhouse")];
            let spec = build::design_cautious(
                def,
                &catalog.goods,
                Shape::Bays {
                    bays: 2,
                    storeys: 2,
                    lofts: 0,
                },
                (home.0 + 25.0, home.1),
                None,
                1.0,
                None,
                Some(FireDesign {
                    caution_thatch: 1.0,
                    caution_wattle: 2.0,
                    knows_rubble: true,
                    knows_mud_brick: false,
                    knows_firing: false,
                    knows_lime: false,
                    knows_systems: [false; 4],
                }),
            )
            .expect("design");
            sim.place_building_for_tests(household, spec, 5)
                .expect("placed");
        }
        let (told, before) = {
            let sim = &h.engine.world.as_ref().expect("a world").sim;
            (failed(sim), counted(sim))
        };
        let reply = h.ask(Request::Shake {
            strength: 0.6,
            settlement: 0,
        });
        assert!(matches!(reply, Reply::Response(_)), "{reply:?}");
        let sim = &h.engine.world.as_ref().expect("a world").sim;
        assert!(failed(sim) > told, "the longhouse fell and is told");
        assert_eq!(
            counted(sim),
            before,
            "but it is the shaking's, not a failure"
        );
    }

    #[test]
    fn the_shaking_is_listed_with_its_page_and_what_fell_or_cracked() {
        let mut h = Harness::new(None, None);
        assert_eq!(
            error_code(&h.ask(Request::GetQuake { id: 1 })),
            Some(wire::ErrorCode::NoWorld)
        );
        h.create("Shaken incidents");
        // A finished hut beside the first household's home, so the shaking reaches something.
        {
            let sim = &mut h.engine.world.as_mut().expect("a world").sim;
            let (household, home) = sim
                .people()
                .households
                .iter()
                .map(|(_, x)| (x.id, x.home))
                .min_by_key(|x| x.0)
                .expect("a household");
            let catalog = &sim.rules().catalog;
            let def = &catalog.buildings[catalog.building_index("core:building/hut").expect("hut")];
            let spec = civ_agents::build::design_shape(
                def,
                &catalog.goods,
                civ_agents::build::Shape::Round { radius: 310 },
                (home.0 + 40.0, home.1),
                None,
            )
            .expect("a hut");
            sim.place_building_for_tests(household, spec, 5)
                .expect("placed");
        }
        assert_eq!(
            error_code(&h.ask(Request::GetQuake { id: 1 })),
            Some(wire::ErrorCode::NotFound)
        );
        let reply = h.ask(Request::Shake {
            strength: 0.6,
            settlement: 0,
        });
        assert!(matches!(reply, Reply::Response(_)), "{reply:?}");
        let q = {
            let sim = &h.engine.world.as_ref().expect("a world").sim;
            sim.people().quakes.list.last().expect("a record").clone()
        };
        assert!(!q.shaken.is_empty(), "the hut is reached");
        let count = |o: civ_agents::quake::Outcome| {
            q.shaken.iter().filter(|s| s.outcome == o).count() as u32
        };
        let fell = count(civ_agents::quake::Outcome::Fell);
        let cracked = count(civ_agents::quake::Outcome::Cracked);
        let deaths: u32 = q.shaken.iter().map(|s| u32::from(s.deaths)).sum();
        // The shaking is listed with the counts the kernel recorded.
        let Reply::Response(payload) = h.ask(Request::GetIncidents {
            settlement: 0,
            kind: 12,
            before: None,
            limit: 0,
        }) else {
            panic!("the incidents are read");
        };
        let response = flatbuffers::root::<wire::Response>(&payload).expect("decodes");
        let incidents = response.body_as_incidents().expect("incidents");
        assert_eq!(
            incidents.rows().map_or(0, |r| r.len()),
            0,
            "a shaking has no rows"
        );
        let listed: Vec<_> = incidents.quakes().expect("quakes").iter().collect();
        assert_eq!(listed.len(), 1);
        let row = listed[0];
        assert_eq!(
            (row.id(), row.at(), row.settlement(), row.observer()),
            (q.id, q.at.minutes(), 0, q.influence)
        );
        assert_eq!(
            (row.reached(), row.fell(), row.cracked(), row.deaths()),
            (q.shaken.len() as u32, fell, cracked, deaths)
        );
        assert!(
            row.words()
                .unwrap_or_default()
                .starts_with("The ground shook over the whole map on "),
            "{:?}",
            row.words()
        );
        // Its page: the same row, sent by the observer, and every building that fell or cracked.
        let Reply::Response(payload) = h.ask(Request::GetQuake { id: q.id }) else {
            panic!("its page is read");
        };
        let response = flatbuffers::root::<wire::Response>(&payload).expect("decodes");
        let info = response.body_as_quake_info().expect("a shaking's page");
        let page_row = info.row().expect("its row");
        assert_eq!(
            (
                page_row.reached(),
                page_row.fell(),
                page_row.cracked(),
                page_row.deaths()
            ),
            (q.shaken.len() as u32, fell, cracked, deaths)
        );
        let cause = info.cause_link().expect("its cause");
        assert_eq!(
            (cause.relation(), cause.target(), cause.id()),
            (0, 0, u64::from(q.influence))
        );
        let members: Vec<(u8, u32, u64, u32)> = info
            .incidents()
            .expect("its buildings")
            .iter()
            .map(|r| (r.kind(), r.id(), r.who(), r.quake()))
            .collect();
        let want: Vec<(u8, u32, u64, u32)> = q
            .shaken
            .iter()
            .filter(|s| s.outcome != civ_agents::quake::Outcome::Stood)
            .zip(1u32..)
            .map(|(s, id)| (11, id, s.building.get(), q.id))
            .collect();
        assert_eq!(members, want);
        // Buildings that fell or cracked are incidents of their own, each linked to the shaking.
        let Reply::Response(payload) = h.ask(Request::GetIncidents {
            settlement: 0,
            kind: 11,
            before: None,
            limit: 0,
        }) else {
            panic!("the incidents are read");
        };
        let response = flatbuffers::root::<wire::Response>(&payload).expect("decodes");
        let incidents = response.body_as_incidents().expect("incidents");
        let mut rows: Vec<(u8, u32, u64, u32)> = incidents
            .rows()
            .expect("rows")
            .iter()
            .map(|r| (r.kind(), r.id(), r.who(), r.quake()))
            .collect();
        rows.sort_unstable();
        assert_eq!(rows, want);
        let total = incidents
            .totals()
            .expect("totals")
            .iter()
            .find(|t| t.kind() == 11)
            .expect("a total for buildings shaken");
        assert_eq!(total.all(), fell + cracked);
        if let Some(&(_, id, _, _)) = want.first() {
            let Reply::Response(payload) = h.ask(Request::GetIncident { kind: 11, id }) else {
                panic!("its page is read");
            };
            let response = flatbuffers::root::<wire::Response>(&payload).expect("decodes");
            let info = response.body_as_incident_info().expect("a page");
            assert!(
                info.links()
                    .expect("links")
                    .iter()
                    .any(|l| l.relation() == 1 && l.target() == 12 && l.id() == u64::from(q.id))
            );
        }
        // The chronicle's entry for the shaking, and for each building that fell, link to it.
        let sim = &h.engine.world.as_ref().expect("a world").sim;
        let linked: Vec<_> = sim
            .people()
            .chronicle
            .iter()
            .filter(|e| e.at == q.at)
            .filter_map(|e| frames::incidents::chronicle_link(sim, e).map(|l| (e.kind, l)))
            .collect();
        assert!(
            linked.contains(&(civ_agents::history::ChronicleKind::Influence, (12, q.id))),
            "{linked:?}"
        );
        let failed = linked
            .iter()
            .filter(|(k, l)| {
                *k == civ_agents::history::ChronicleKind::BuildingFailed && *l == (12, q.id)
            })
            .count();
        assert_eq!(failed as u32, fell);
    }

    #[test]
    fn the_observer_sets_a_hut_alight_and_its_fire_is_listed_with_its_page() {
        let mut h = Harness::new(None, None);
        assert_eq!(
            error_code(&h.ask(Request::SetFire { building: 1 })),
            Some(wire::ErrorCode::NoWorld)
        );
        h.create("Fire");
        // A finished hut beside the first household's home.
        let hut = {
            let sim = &mut h.engine.world.as_mut().expect("a world").sim;
            let (household, home) = sim
                .people()
                .households
                .iter()
                .map(|(_, x)| (x.id, x.home))
                .min_by_key(|x| x.0)
                .expect("a household");
            let catalog = &sim.rules().catalog;
            let def = &catalog.buildings[catalog.building_index("core:building/hut").expect("hut")];
            let spec = civ_agents::build::design_shape(
                def,
                &catalog.goods,
                civ_agents::build::Shape::Round { radius: 310 },
                (home.0 + 40.0, home.1),
                None,
            )
            .expect("a hut");
            // Every stage done: foundation, frame, walls, roof and finish.
            sim.place_building_for_tests(household, spec, 5)
                .expect("placed")
        };
        let reply = h.ask(Request::SetFire {
            building: hut.get(),
        });
        let Reply::Response(ack) = reply else {
            panic!("set alight: {reply:?}");
        };
        let message = flatbuffers::root::<wire::Response>(&ack)
            .expect("decodes")
            .body_as_ack()
            .and_then(|a| a.message())
            .unwrap_or_default()
            .to_owned();
        assert!(
            message.starts_with("The observer set alight the hut of "),
            "{message}"
        );
        // Not again while it burns, nor a building that is not there.
        assert_eq!(
            error_code(&h.ask(Request::SetFire {
                building: hut.get()
            })),
            Some(wire::ErrorCode::BadRequest)
        );
        assert_eq!(
            error_code(&h.ask(Request::SetFire { building: 0 })),
            Some(wire::ErrorCode::NotFound)
        );
        let Reply::Response(payload) = h.ask(Request::GetIncidents {
            settlement: 0,
            kind: 4,
            before: None,
            limit: 0,
        }) else {
            panic!("the incidents are read");
        };
        let response = flatbuffers::root::<wire::Response>(&payload).expect("decodes");
        let rows: Vec<(u8, u32, u64)> = response
            .body_as_incidents()
            .and_then(|i| i.rows())
            .expect("rows")
            .iter()
            .map(|r| (r.kind(), r.id(), r.who()))
            .collect();
        assert_eq!(rows, vec![(4, 1, hut.get())]);
        let Reply::Response(payload) = h.ask(Request::GetIncident { kind: 4, id: 1 }) else {
            panic!("its page is read");
        };
        let response = flatbuffers::root::<wire::Response>(&payload).expect("decodes");
        assert!(response.body_as_incident_info().is_some());
        assert_eq!(
            error_code(&h.ask(Request::GetConflagration { id: 1 })),
            Some(wire::ErrorCode::NotFound)
        );
    }

    #[test]
    fn the_incidents_list_a_case_the_observer_brought_with_its_page_and_its_outbreak() {
        let mut h = Harness::new(None, None);
        let list = Request::GetIncidents {
            settlement: 0,
            kind: 0,
            before: None,
            limit: 0,
        };
        assert_eq!(
            error_code(&h.ask(list.clone())),
            Some(wire::ErrorCode::NoWorld)
        );
        h.create("Incidents");
        let (person, cholera) = {
            let sim = &h.engine.world.as_ref().expect("a world").sim;
            let person = sim
                .people()
                .people
                .iter()
                .map(|(_, p)| p.id.get())
                .min()
                .expect("someone");
            let cholera = sim
                .rules()
                .catalog
                .disease_index("core:disease/cholera")
                .expect("cholera in the core content");
            (person, u32::try_from(cholera).expect("a small index"))
        };
        let Reply::Response(_) = h.ask(Request::Plague {
            person,
            disease: cholera,
        }) else {
            panic!("brought");
        };
        let Reply::Response(payload) = h.ask(list) else {
            panic!("the incidents are read");
        };
        let response = flatbuffers::root::<wire::Response>(&payload).expect("decodes");
        let incidents = response.body_as_incidents().expect("incidents");
        let rows: Vec<(u8, u32, u64, u32)> = incidents
            .rows()
            .expect("rows")
            .iter()
            .filter(|r| r.kind() == 2)
            .map(|r| (r.kind(), r.id(), r.who(), r.outbreak()))
            .collect();
        assert_eq!(rows, vec![(2, 1, person, 1)]);
        let Reply::Response(payload) = h.ask(Request::GetIncident { kind: 2, id: 1 }) else {
            panic!("its page is read");
        };
        let response = flatbuffers::root::<wire::Response>(&payload).expect("decodes");
        let info = response.body_as_incident_info().expect("a page");
        assert_eq!(
            info.steps()
                .expect("steps")
                .iter()
                .next()
                .and_then(|s| s.label()),
            Some("Took it")
        );
        let Reply::Response(payload) = h.ask(Request::GetOutbreak { id: 1, limit: 0 }) else {
            panic!("its outbreak is read");
        };
        let response = flatbuffers::root::<wire::Response>(&payload).expect("decodes");
        let outbreak = response.body_as_outbreak_info().expect("an outbreak");
        assert!(
            outbreak
                .cause()
                .is_some_and(|c| c.contains("brought by the observer")),
            "{:?}",
            outbreak.cause()
        );
        assert_eq!(
            error_code(&h.ask(Request::GetIncident { kind: 2, id: 99 })),
            Some(wire::ErrorCode::NotFound)
        );
        assert_eq!(
            error_code(&h.ask(Request::GetOutbreak { id: 99, limit: 0 })),
            Some(wire::ErrorCode::NotFound)
        );
        // The Water lens reads every home (wire 1.66); a lens there is not is refused.
        let Reply::Response(payload) = h.ask(Request::GetCoverage {
            settlement: 0,
            lens: 1,
        }) else {
            panic!("the lens is read");
        };
        let response = flatbuffers::root::<wire::Response>(&payload).expect("decodes");
        let lens = response.body_as_coverage().expect("a lens");
        assert_eq!((lens.lens(), lens.title()), (1, Some("Water")));
        assert!(lens.homes().is_some_and(|h| !h.is_empty()));
        assert_eq!(lens.summary().map(|s| s.len()), Some(2));
        assert_eq!(
            error_code(&h.ask(Request::GetCoverage {
                settlement: 0,
                lens: 9
            })),
            Some(wire::ErrorCode::BadRequest)
        );
    }

    #[test]
    fn the_observer_can_change_the_climate_and_the_weather_panel_tells_of_it() {
        // M9c slice DM (wire 1.96, ADR-0032 §9).
        let mut h = Harness::new(None, None);
        h.create("Warm years");
        let reply = h.ask(Request::ForceClimate {
            offset_c: 2.0,
            wet_factor: 0.8,
            ramp_years: 10,
        });
        let Reply::Response(ack) = reply else {
            panic!("forced: {reply:?}");
        };
        let message = flatbuffers::root::<wire::Response>(&ack)
            .expect("decodes")
            .body_as_ack()
            .and_then(|a| a.message())
            .unwrap_or_default()
            .to_owned();
        assert_eq!(
            message,
            "The observer changes the climate: +2.0 °C and wet days' rain ×0.80, over 10 years"
        );
        for (offset_c, wet_factor, ramp_years) in [
            (11.0, 1.0, 1),
            (0.0, 0.2, 1),
            (0.0, 1.0, 101),
            (f32::NAN, 1.0, 1),
        ] {
            assert_eq!(
                error_code(&h.ask(Request::ForceClimate {
                    offset_c,
                    wet_factor,
                    ramp_years
                })),
                Some(wire::ErrorCode::BadRequest)
            );
        }
        let Reply::Response(payload) = h.ask(Request::GetWeather) else {
            panic!("the weather is read");
        };
        let response = flatbuffers::root::<wire::Response>(&payload).expect("decodes");
        let report = response.body_as_weather_report().expect("a report");
        let climate = report.climate().expect("the change is told");
        assert!(
            climate.starts_with("The observer has changed the climate: +0.0 °C")
                && climate.contains("reaching +2.0 °C and ×0.80 by"),
            "{climate}"
        );
        let world = h.engine.world.as_ref().expect("a world");
        assert_eq!(world.sim.land().weather.forcings.len(), 1);
    }

    #[test]
    fn the_observer_can_hold_back_the_rain_and_the_weather_panel_tells_of_it() {
        let mut h = Harness::new(None, None);
        assert_eq!(
            error_code(&h.ask(Request::Drought {
                months: 6,
                depth: 1.5,
            })),
            Some(wire::ErrorCode::NoWorld)
        );
        h.create("Dry years");
        let reply = h.ask(Request::Drought {
            months: 6,
            depth: 1.5,
        });
        let Reply::Response(ack) = reply else {
            panic!("held: {reply:?}");
        };
        let message = flatbuffers::root::<wire::Response>(&ack)
            .expect("decodes")
            .body_as_ack()
            .and_then(|a| a.message())
            .unwrap_or_default()
            .to_owned();
        assert!(
            message.starts_with(
                "The observer holds back the rain for 6 months, 1.5 standard deviations dry, until"
            ),
            "{message}"
        );
        // Out of bounds, in length or depth, is refused.
        for (months, depth) in [(2, 1.0), (25, 1.0), (6, 0.0), (6, 3.5), (6, f32::NAN)] {
            assert_eq!(
                error_code(&h.ask(Request::Drought { months, depth })),
                Some(wire::ErrorCode::BadRequest),
                "{months} months, {depth} deep"
            );
        }
        let Reply::Response(payload) = h.ask(Request::GetWeather) else {
            panic!("the weather is read");
        };
        let response = flatbuffers::root::<wire::Response>(&payload).expect("decodes");
        let report = response.body_as_weather_report().expect("a report");
        let drought = report.drought().expect("the drought is told");
        assert!(
            drought.starts_with("The observer holds back the rain, 1.5 standard deviations dry"),
            "{drought}"
        );
        let months = report.months().expect("months");
        assert!(
            !months.is_empty() && months.get(months.len() - 1).held(),
            "this month is held"
        );
    }

    #[test]
    fn create_save_list_and_load() {
        let mut h = Harness::new(None, None);
        h.create("River Test");
        assert_eq!(h.engine.epoch, 1);
        let saves = h.saves();
        assert_eq!(saves.len(), 1, "a new world is autosaved at once");
        assert_eq!(saves[0].kind, "auto");
        assert_eq!(saves[0].world_name, "River Test");
        assert!(h.engine.marker.last_good.is_some());

        assert!(matches!(
            h.ask(Request::SaveWorld {
                label: "  Mine  ".to_owned()
            }),
            Reply::Response(_)
        ));
        let saves = h.saves();
        assert_eq!(saves.len(), 2);
        assert_eq!(saves[0].label, "Mine");
        assert!(saves.iter().all(|s| s.compatible));

        let Reply::Response(listed) = h.ask(Request::ListSaves) else {
            panic!("lists saves");
        };
        let listed = flatbuffers::root::<wire::Response>(&listed)
            .expect("decodes")
            .body_as_save_list()
            .expect("a save list");
        assert_eq!(listed.saves().map_or(0, |s| s.len()), 2);

        assert!(matches!(
            h.ask(Request::SetClock {
                paused: false,
                speed: civ_sim::SPEED_1X
            }),
            Reply::Response(_)
        ));
        let file = saves[1].file.clone();
        assert!(matches!(
            h.ask(Request::LoadWorld { file }),
            Reply::Response(_)
        ));
        h.finish_task();
        let world = h.engine.world.as_ref().expect("loaded");
        assert!(world.sim.paused(), "a loaded world waits for the player");
        assert_eq!(h.engine.epoch, 2);
        assert_eq!(h.engine.last_error, None);
    }

    #[test]
    fn bad_and_busy_requests_are_refused() {
        let mut h = Harness::new(None, None);
        assert_eq!(
            error_code(&h.ask(Request::SaveWorld {
                label: String::new()
            })),
            Some(wire::ErrorCode::NoWorld)
        );
        assert_eq!(
            error_code(&h.ask(Request::LoadWorld {
                file: "../x.tcesave".to_owned()
            })),
            Some(wire::ErrorCode::BadRequest)
        );
        assert_eq!(
            error_code(&h.ask(Request::LoadWorld {
                file: "w/g0000000001-manual.tcesave".to_owned()
            })),
            Some(wire::ErrorCode::NotFound)
        );
        let mut new_world = NewWorld {
            name: "X".to_owned(),
            seed: 1,
            preset_id: PRESET.to_owned(),
            size_cells: 300,
            band_size: 0,
            neighbours: Vec::new(),
            neighbours_known: false,
            regime_id: String::new(),
        };
        assert_eq!(
            error_code(&h.ask(Request::NewWorld(new_world.clone()))),
            Some(wire::ErrorCode::BadRequest)
        );
        new_world.size_cells = 256;
        new_world.preset_id = "core:worldgen/nowhere".to_owned();
        assert_eq!(
            error_code(&h.ask(Request::NewWorld(new_world.clone()))),
            Some(wire::ErrorCode::BadRequest)
        );
        new_world.preset_id = PRESET.to_owned();
        new_world.regime_id = "core:regime/nobody".to_owned();
        assert_eq!(
            error_code(&h.ask(Request::NewWorld(new_world.clone()))),
            Some(wire::ErrorCode::BadRequest)
        );
        new_world.regime_id = "core:regime/village".to_owned();
        assert!(matches!(
            h.ask(Request::NewWorld(new_world.clone())),
            Reply::Response(_)
        ));
        assert_eq!(
            error_code(&h.ask(Request::NewWorld(new_world))),
            Some(wire::ErrorCode::Busy)
        );
        h.finish_task();
        assert_eq!(
            error_code(&h.ask(Request::SetClock {
                paused: false,
                speed: 0.0
            })),
            Some(wire::ErrorCode::BadRequest)
        );
    }

    #[test]
    fn cancelled_generation_leaves_no_world() {
        let mut h = Harness::new(None, None);
        assert!(matches!(
            h.ask(Request::NewWorld(NewWorld {
                name: "Big".to_owned(),
                seed: 3,
                preset_id: PRESET.to_owned(),
                size_cells: 2048,
                band_size: 0,
                neighbours: Vec::new(),
                neighbours_known: false,
                regime_id: String::new(),
            })),
            Reply::Response(_)
        ));
        assert!(matches!(h.ask(Request::CancelTask), Reply::Response(_)));
        h.finish_task();
        assert!(h.engine.world.is_none());
        assert!(h.saves().is_empty());
        assert_eq!(
            error_code(&h.ask(Request::CancelTask)),
            Some(wire::ErrorCode::BadRequest)
        );
    }

    #[test]
    fn a_panic_is_contained_and_the_last_good_save_offered() {
        let mut h = Harness::new(None, None);
        h.create("Fragile");
        h.engine.guarded(|_| panic!("deliberate test panic"));
        assert!(h.engine.world.is_none());
        let error = h.engine.last_error.clone().unwrap_or_default();
        assert!(error.contains("deliberate test panic"), "{error}");
        let offer = h.engine.recovery.clone().expect("recovery is offered");
        assert!(offer.file.ends_with("-auto.tcesave"), "{offer:?}");
        assert!(
            h.saves().iter().any(|s| s.kind == "crash"),
            "a crash snapshot is written for diagnosis"
        );

        assert!(matches!(
            h.ask(Request::RecoverWorld { accept: true }),
            Reply::Response(_)
        ));
        h.finish_task();
        assert_eq!(
            h.engine.world.as_ref().map(|w| w.sim.meta().name.as_str()),
            Some("Fragile")
        );
        assert!(h.engine.recovery.is_none());
    }

    #[test]
    fn an_unclean_exit_is_offered_for_recovery_next_time() {
        let mut first = Harness::new(None, None);
        first.create("Interrupted");
        let Harness { engine, dir, .. } = first;
        drop(engine); // no shutdown: the marker stays behind

        let previous = SessionMarker::left_behind(dir.path());
        assert!(previous.is_some());
        let mut second = Harness::new(previous, Some(dir));
        let offer = second.engine.recovery.clone().expect("recovery is offered");
        assert_eq!(offer.world_name, "Interrupted");

        second.engine.shutdown();
        assert_eq!(SessionMarker::left_behind(second.dir.path()), None);
    }

    #[test]
    fn autosaves_rotate() {
        let mut h = Harness::new(None, None);
        h.engine.config.autosave_interval = Duration::ZERO;
        h.engine.config.keep_autosaves = 2;
        h.create("Busy Valley");
        assert!(matches!(
            h.ask(Request::SetClock {
                paused: false,
                speed: civ_sim::MAX_DETAILED_SPEED
            }),
            Reply::Response(_)
        ));
        for _ in 0..4 {
            h.engine.tick(Duration::from_millis(200));
        }
        let autosaves = h.saves().iter().filter(|s| s.kind == "auto").count();
        assert_eq!(autosaves, 2);
        let world = h.engine.world.as_ref().expect("world");
        assert!(world.sim.generation() >= 4);
        assert!(!world.sim.is_dirty());
    }
}
