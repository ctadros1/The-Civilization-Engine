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
use civ_sim::{NewWorld, Sim, SimError, frames, persist};
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
}

struct Task {
    id: u64,
    view: TaskView,
    cancel: Option<Arc<AtomicBool>>,
}

/// Running ahead to a time in full detail, as fast as the machine allows (M1 slice G).
struct RunAhead {
    task: u64,
    from: SimTime,
    until: SimTime,
    cancel: Arc<AtomicBool>,
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
            Request::RunUntil { minute } => self.start_run_ahead(minute),
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
            Request::GetRaster(query) => match &self.world {
                None => no_world(),
                Some(w) => match frames::raster_response(
                    w.sim.map(),
                    &w.sim.land().ground,
                    w.sim.stats(),
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
        world.sim.set_paused(paused);
        self.urgent = true;
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
            match world.sim.advance_real(step) {
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
        if advance.minutes > 0 {
            self.changed = true;
        }
        let due =
            advance.months > 0 || world.last_autosave.elapsed() >= self.config.autosave_interval;
        if due && world.sim.is_dirty() {
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

    fn start_run_ahead(&mut self, minute: i64) -> Reply {
        if let Some(busy) = self.busy() {
            return busy;
        }
        let Some(world) = self.world.as_ref() else {
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
        let cancel = Arc::new(AtomicBool::new(false));
        let name = format!("Running ahead to {until}");
        let task = self.begin_task(name, Some(Arc::clone(&cancel)));
        self.run_ahead = Some(RunAhead {
            task,
            from,
            until,
            cancel,
        });
        ack("Running ahead")
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
        if cancelled {
            self.finish_run_ahead("Stopped running ahead");
            return;
        }
        let mut failed = None;
        while world.sim.now() < until && started.elapsed() < budget {
            let step = (until.minutes() - world.sim.now().minutes()).min(60);
            match world.sim.advance_minutes(step) {
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
        }
        let now = world.sim.now();
        let autosave_due =
            world.last_autosave.elapsed() >= self.config.autosave_interval && world.sim.is_dirty();
        if let Some(e) = failed {
            self.finish_run_ahead("Stopped running ahead");
            self.fail(format!("The clock stopped: {e}"));
            return;
        }
        if now >= until {
            self.finish_run_ahead("Ran ahead");
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
                minute: target.minutes()
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
                minute: target.minutes() + 10
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
                minute: start.minutes()
            })),
            Some(wire::ErrorCode::BadRequest)
        );
    }

    #[test]
    fn running_ahead_stops_when_cancelled() {
        let mut h = Harness::new(None, None);
        h.create("Stop");
        let start = h.engine.world.as_ref().expect("a world").sim.now();
        let target = start.plus_minutes(5 * civ_core::time::MINUTES_PER_YEAR);
        assert!(matches!(
            h.ask(Request::RunUntil {
                minute: target.minutes()
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
