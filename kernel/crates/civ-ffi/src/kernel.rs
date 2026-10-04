//! The kernel behind the C interface, in safe Rust: the engine thread `civ-host` runs, one
//! observer link to it, and, when asked, the panel server (ADR-0005). `exports` turns it into C
//! functions.

use std::collections::VecDeque;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use civ_host::engine::{
    self, Channels, EngineConfig, EngineHandle, EventsOut, Msg, Reply, SnapshotOut,
};
use civ_host::protocol::{self, EventRecord, Incoming};
use civ_host::server;
use civ_host::session::SessionMarker;
use civ_schema::wire;
use commons_wire::{FrameKind, Sequencer, WireError};
use tokio::sync::{broadcast, oneshot, watch};

/// How long stopping the panel server may take before its threads are left to end on their own.
const PANELS_STOP: Duration = Duration::from_secs(5);

/// Where a kernel finds its content and keeps its saves.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    /// The content folder (the repository's `content/`).
    pub content_dir: PathBuf,
    /// The saves folder; created when missing.
    pub saves_dir: PathBuf,
}

/// How the panel server runs.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PanelConfig {
    /// Port on 127.0.0.1; 0 picks a free one.
    pub port: u16,
    /// The built web shell (`web/dist`); without it the server says how to build it.
    pub web_dir: Option<PathBuf>,
}

/// How a kernel is doing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Health {
    /// Running.
    Running,
    /// It could not start; nothing runs.
    Failed,
    /// It failed while running and accepts nothing more.
    Faulted,
}

/// Why a call on the kernel did not do what was asked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refused {
    /// Nothing is waiting.
    Nothing,
    /// The frame needs this many bytes; it is kept for the next call.
    TooSmall(usize),
    /// The bytes are not a frame the kernel accepts.
    BadFrame(String),
    /// The kernel is not running.
    NotRunning(Health),
    /// The engine has stopped.
    Stopped,
    /// The panel server could not start.
    Panels(String),
    /// The kernel could not make one of its own frames: a bug. The kernel is now faulted.
    Broken(String),
}

impl From<WireError> for Refused {
    fn from(e: WireError) -> Self {
        Refused::Broken(format!("a frame could not be made: {e}"))
    }
}

/// A kernel: an engine thread with one observer link, and optionally the panel server.
#[derive(Debug)]
pub struct Kernel {
    state: Mutex<State>,
}

struct State {
    health: Health,
    error: String,
    engine: Option<EngineHandle>,
    link: Option<Link>,
    welcome: Vec<u8>,
    panels: Option<Panels>,
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State")
            .field("health", &self.health)
            .field("error", &self.error)
            .field("serving_panels", &self.panels.is_some())
            .finish_non_exhaustive()
    }
}

impl Kernel {
    /// Loads the content, opens the saves folder and starts the engine thread. A kernel that
    /// could not start is returned too, failed, with the reason in [`Kernel::last_error`].
    pub fn create(config: &Config) -> Kernel {
        match start(config) {
            Ok((engine, link, welcome)) => Kernel {
                state: Mutex::new(State {
                    health: Health::Running,
                    error: String::new(),
                    engine: Some(engine),
                    link: Some(link),
                    welcome,
                    panels: None,
                }),
            },
            Err(why) => Kernel::failed(why),
        }
    }

    /// A kernel that could not start.
    pub fn failed(why: String) -> Kernel {
        Kernel {
            state: Mutex::new(State {
                health: Health::Failed,
                error: why,
                engine: None,
                link: None,
                welcome: Vec::new(),
                panels: None,
            }),
        }
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        // A panic while the lock was held faults the kernel (`fault`), so the state behind a
        // poisoned lock is never used to run anything again.
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// How the kernel is doing.
    pub fn health(&self) -> Health {
        self.lock().health
    }

    /// The last failure, in words; empty when there was none.
    pub fn last_error(&self) -> String {
        self.lock().error.clone()
    }

    /// Marks the kernel as failed while running: it accepts nothing more.
    pub fn fault(&self, why: String) {
        let mut state = self.lock();
        state.health = Health::Faulted;
        state.error = why;
    }

    /// Queues one frame from the host: a command, query or heartbeat. Its reply comes through
    /// [`Kernel::poll`].
    pub fn submit(&self, bytes: &[u8]) -> Result<(), Refused> {
        let mut state = self.lock();
        let result = state.link_mut().and_then(|link| link.submit(bytes));
        state.note(result)
    }

    /// Copies the next ordered frame into `buf` and returns its length.
    pub fn poll(&self, buf: &mut [u8]) -> Result<usize, Refused> {
        let mut state = self.lock();
        let result = state.link_mut().and_then(|link| {
            link.pump()?;
            let front = link.ordered.front().ok_or(Refused::Nothing)?;
            let n = front.len();
            let dest = buf.get_mut(..n).ok_or(Refused::TooSmall(n))?;
            dest.copy_from_slice(front);
            link.ordered.pop_front();
            Ok(n)
        });
        state.note(result)
    }

    /// Copies the latest snapshot into `buf` if its number is greater than `after`. Returns its
    /// length and number.
    pub fn copy_snapshot(&self, after: u64, buf: &mut [u8]) -> Result<(usize, u64), Refused> {
        let mut state = self.lock();
        let result = state.link_mut().and_then(|link| {
            link.pump()?;
            let (number, frame) = link.snapshot.as_ref().ok_or(Refused::Nothing)?;
            if *number <= after {
                return Err(Refused::Nothing);
            }
            let n = frame.len();
            let dest = buf.get_mut(..n).ok_or(Refused::TooSmall(n))?;
            dest.copy_from_slice(frame);
            Ok((n, *number))
        });
        state.note(result)
    }

    /// Starts the panel server on 127.0.0.1, unless it is already running.
    pub fn serve_panels(&self, config: &PanelConfig) -> Result<(), Refused> {
        let mut state = self.lock();
        let result = state.serve_panels(config);
        state.note(result)
    }

    /// The address the panels open, with its port and token, once the server runs.
    pub fn panel_url(&self) -> Option<String> {
        self.lock().panels.as_ref().map(|p| p.url.clone())
    }

    /// Stops the panel server, then the engine (autosaving a changed world), and waits for every
    /// thread the kernel started.
    pub fn destroy(self) {
        let mut state = self
            .state
            .into_inner()
            .unwrap_or_else(PoisonError::into_inner);
        if let Some(panels) = state.panels.take() {
            panels.stop();
        }
        state.link = None;
        if let Some(engine) = state.engine.take() {
            engine.shutdown();
        }
    }
}

impl State {
    fn link_mut(&mut self) -> Result<&mut Link, Refused> {
        if self.health != Health::Running {
            return Err(Refused::NotRunning(self.health));
        }
        self.link.as_mut().ok_or(Refused::Stopped)
    }

    /// Remembers why a call failed, for `last_error`, and faults the kernel on a bug.
    fn note<T>(&mut self, result: Result<T, Refused>) -> Result<T, Refused> {
        match &result {
            Err(Refused::BadFrame(why)) | Err(Refused::Panels(why)) => self.error = why.clone(),
            Err(Refused::Stopped) => self.error = "the engine has stopped".to_owned(),
            Err(Refused::Broken(why)) => {
                self.health = Health::Faulted;
                self.error = why.clone();
            }
            _ => {}
        }
        result
    }

    fn serve_panels(&mut self, config: &PanelConfig) -> Result<(), Refused> {
        if self.health != Health::Running {
            return Err(Refused::NotRunning(self.health));
        }
        if self.panels.is_some() {
            return Ok(());
        }
        let channels = self
            .engine
            .as_ref()
            .ok_or(Refused::Stopped)?
            .channels
            .clone();
        let panels = Panels::start(channels, self.welcome.clone(), config)
            .map_err(|e| Refused::Panels(format!("the panel server could not start: {e}")))?;
        self.panels = Some(panels);
        Ok(())
    }
}

fn start(config: &Config) -> Result<(EngineHandle, Link, Vec<u8>), String> {
    let report = civ_content::load(&config.content_dir);
    let errors = report.error_count();
    let Some(content) = report.registry else {
        let first = report
            .diagnostics
            .first()
            .map(|d| format!(", first: {d}"))
            .unwrap_or_default();
        return Err(format!(
            "the content in {} could not be loaded ({errors} error(s){first})",
            config.content_dir.display()
        ));
    };
    std::fs::create_dir_all(&config.saves_dir).map_err(|e| {
        format!(
            "the saves folder {} could not be created: {e}",
            config.saves_dir.display()
        )
    })?;
    let welcome = protocol::welcome_payload(&content);
    let previous = SessionMarker::left_behind(&config.saves_dir);
    let engine = engine::start(
        EngineConfig::new(Arc::new(content), config.saves_dir.clone()),
        previous,
    )
    .map_err(|e| format!("the engine could not start: {e}"))?;
    match Link::open(engine.channels.clone(), &welcome) {
        Ok(link) => Ok((engine, link, welcome)),
        Err(e) => {
            engine.shutdown();
            Err(format!("the first frames could not be made: {e}"))
        }
    }
}

/// The host's session with the engine (ADR-0005 §4): its frame sequencer, the ordered frames
/// waiting to be polled, the replies still being worked on, and the latest snapshot.
struct Link {
    channels: Channels,
    seq: Sequencer,
    ordered: VecDeque<Vec<u8>>,
    pending: VecDeque<(u64, oneshot::Receiver<Reply>)>,
    events: broadcast::Receiver<Arc<EventsOut>>,
    last_event: u64,
    snapshots: watch::Receiver<Arc<SnapshotOut>>,
    /// The latest snapshot frame and its number.
    snapshot: Option<(u64, Vec<u8>)>,
    snapshots_taken: u64,
}

impl Link {
    fn open(channels: Channels, welcome: &[u8]) -> Result<Link, WireError> {
        let mut snapshots = channels.snapshot.clone();
        let current = Arc::clone(&snapshots.borrow_and_update());
        // Subscribe before reading the history, so nothing falls between them (duplicates are
        // skipped by id).
        let events = channels.events.subscribe();
        let mut link = Link {
            channels,
            seq: Sequencer::new(current.epoch),
            ordered: VecDeque::new(),
            pending: VecDeque::new(),
            events,
            last_event: 0,
            snapshots,
            snapshot: None,
            snapshots_taken: 0,
        };
        let welcome = protocol::frame(&mut link.seq, FrameKind::Welcome, 0, 0, welcome)?;
        link.ordered.push_back(welcome);
        link.take_snapshot(&current)?;
        link.catch_up(current.sim_time)?;
        Ok(link)
    }

    fn take_snapshot(&mut self, snapshot: &SnapshotOut) -> Result<(), WireError> {
        if snapshot.epoch != self.seq.epoch() {
            self.seq = Sequencer::new(snapshot.epoch);
        }
        let frame = protocol::frame(
            &mut self.seq,
            FrameKind::Snapshot,
            0,
            snapshot.sim_time,
            &snapshot.payload,
        )?;
        self.snapshots_taken += 1;
        self.snapshot = Some((self.snapshots_taken, frame));
        Ok(())
    }

    /// Brings the link up to date: the latest snapshot, replies that are ready and new events.
    fn pump(&mut self) -> Result<(), Refused> {
        if self.snapshots.has_changed().unwrap_or(false) {
            let latest = Arc::clone(&self.snapshots.borrow_and_update());
            self.take_snapshot(&latest)?;
        }
        let mut waiting = VecDeque::with_capacity(self.pending.len());
        while let Some((correlation, mut reply)) = self.pending.pop_front() {
            let frame = match reply.try_recv() {
                Ok(Reply::Response(payload)) => {
                    protocol::frame(&mut self.seq, FrameKind::Response, correlation, 0, &payload)?
                }
                Ok(Reply::Error(code, message)) => self.error_frame(correlation, code, &message)?,
                Err(oneshot::error::TryRecvError::Empty) => {
                    waiting.push_back((correlation, reply));
                    continue;
                }
                Err(oneshot::error::TryRecvError::Closed) => self.error_frame(
                    correlation,
                    wire::ErrorCode::Internal,
                    "the engine failed while handling this request",
                )?,
            };
            self.ordered.push_back(frame);
        }
        self.pending = waiting;
        loop {
            match self.events.try_recv() {
                Ok(batch) => self.push_events(&batch.events, batch.sim_time)?,
                Err(broadcast::error::TryRecvError::Lagged(_)) => {
                    let sim_time = self.snapshots.borrow().sim_time;
                    self.catch_up(sim_time)?;
                }
                Err(_) => break,
            }
        }
        Ok(())
    }

    fn error_frame(
        &mut self,
        correlation: u64,
        code: wire::ErrorCode,
        message: &str,
    ) -> Result<Vec<u8>, WireError> {
        protocol::frame(
            &mut self.seq,
            FrameKind::Error,
            correlation,
            0,
            &protocol::error_payload(code, message),
        )
    }

    fn push_events(&mut self, events: &[EventRecord], sim_time: i64) -> Result<(), WireError> {
        let fresh: Vec<EventRecord> = events
            .iter()
            .filter(|e| e.id > self.last_event)
            .cloned()
            .collect();
        let Some(last) = fresh.last() else {
            return Ok(());
        };
        self.last_event = last.id;
        let frame = protocol::frame(
            &mut self.seq,
            FrameKind::Events,
            0,
            sim_time,
            &protocol::events_payload(&fresh),
        )?;
        self.ordered.push_back(frame);
        Ok(())
    }

    /// Queues an `Events` frame with the remembered events after the last one sent.
    fn catch_up(&mut self, sim_time: i64) -> Result<(), WireError> {
        let missed: Vec<EventRecord> = self
            .channels
            .history
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .iter()
            .filter(|e| e.id > self.last_event)
            .cloned()
            .collect();
        self.push_events(&missed, sim_time)
    }

    fn submit(&mut self, bytes: &[u8]) -> Result<(), Refused> {
        match protocol::read_request(bytes) {
            Ok(Incoming::Nothing) => Ok(()),
            Ok(Incoming::Request(correlation, request)) => {
                let (tx, rx) = oneshot::channel();
                self.channels
                    .requests
                    .send(Msg::Request(request, tx))
                    .map_err(|_| Refused::Stopped)?;
                self.pending.push_back((correlation, rx));
                Ok(())
            }
            Err(refusal) => Err(Refused::BadFrame(refusal.message)),
        }
    }
}

/// The panel server: the web shell and the observer socket on 127.0.0.1, on a small runtime of
/// its own, with a token in its address (ADR-0005 §6).
struct Panels {
    runtime: tokio::runtime::Runtime,
    stop: oneshot::Sender<()>,
    served: tokio::task::JoinHandle<()>,
    url: String,
}

impl Panels {
    fn start(
        channels: Channels,
        welcome: Vec<u8>,
        config: &PanelConfig,
    ) -> Result<Panels, std::io::Error> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .thread_name("tce-panels")
            .enable_all()
            .build()?;
        let listener = runtime.block_on(tokio::net::TcpListener::bind((
            Ipv4Addr::LOCALHOST,
            config.port,
        )))?;
        let port = listener.local_addr()?.port();
        let token = token()?;
        let app = server::router(
            channels,
            welcome,
            config.web_dir.clone(),
            server::Access {
                token: Some(token.clone()),
            },
        );
        let (stop, stopped) = oneshot::channel::<()>();
        let served = runtime.spawn(async move {
            let shutdown = async {
                let _ = stopped.await;
            };
            if let Err(e) = axum::serve(listener, app)
                .with_graceful_shutdown(shutdown)
                .await
            {
                tracing::warn!("the panel server stopped: {e}");
            }
        });
        Ok(Panels {
            runtime,
            stop,
            served,
            url: format!("http://127.0.0.1:{port}/?token={token}"),
        })
    }

    fn stop(self) {
        let _ = self.stop.send(());
        let served = self.served;
        let _ = self
            .runtime
            .block_on(async { tokio::time::timeout(PANELS_STOP, served).await });
        self.runtime.shutdown_timeout(PANELS_STOP);
    }
}

/// A fresh random token: 128 bits from the operating system, as hex.
fn token() -> Result<String, std::io::Error> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|e| std::io::Error::other(e.to_string()))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}
