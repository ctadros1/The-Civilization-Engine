//! The observer protocol end to end over a real WebSocket (ADR-0001): handshake, snapshots,
//! events, commands and queries, saving and loading.

use std::net::Ipv4Addr;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use civ_host::engine::{self, EngineConfig, EngineHandle};
use civ_host::{protocol, server};
use civ_schema::flatbuffers::{self, FlatBufferBuilder, UnionWIPOffset, WIPOffset};
use civ_schema::{WIRE_SCHEMA, wire};
use commons_wire::{FrameKind, FrameMeta, SchemaId};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

const TIMEOUT: Duration = Duration::from_secs(120);

struct Host {
    addr: std::net::SocketAddr,
    engine: Option<EngineHandle>,
    _saves: tempfile::TempDir,
}

async fn start_host() -> Host {
    start_host_with(server::Access::default()).await
}

async fn start_host_with(access: server::Access) -> Host {
    let saves = tempfile::tempdir().expect("temp dir");
    let content_root = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
        .expect("content/ is above the crate");
    let content = Arc::new(
        civ_content::load(&content_root)
            .registry
            .expect("content loads"),
    );
    let engine = engine::start(
        EngineConfig::new(Arc::clone(&content), saves.path().to_owned()),
        None,
    )
    .expect("engine starts");
    let app = server::router(
        engine.channels.clone(),
        protocol::welcome_payload(&content),
        None,
        access,
    );
    let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .await
        .expect("binds");
    let addr = listener.local_addr().expect("address");
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    Host {
        addr,
        engine: Some(engine),
        _saves: saves,
    }
}

struct Frame {
    meta: FrameMeta,
    payload: Vec<u8>,
}

struct Client {
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
    next_correlation: u64,
    snapshot: Option<Frame>,
    events: Vec<(wire::EventKind, String)>,
}

fn finish<T>(mut fbb: FlatBufferBuilder<'_>, root: WIPOffset<T>) -> Vec<u8> {
    fbb.finish(root, None);
    fbb.finished_data().to_vec()
}

fn hello_payload() -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let client = fbb.create_string("ws-test");
    let root = wire::Hello::create(
        &mut fbb,
        &wire::HelloArgs {
            client: Some(client),
        },
    );
    finish(fbb, root)
}

fn command(
    body_type: wire::CommandBody,
    build: impl FnOnce(&mut FlatBufferBuilder<'static>) -> WIPOffset<UnionWIPOffset>,
) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let body = build(&mut fbb);
    let root = wire::Command::create(
        &mut fbb,
        &wire::CommandArgs {
            body_type,
            body: Some(body),
        },
    );
    finish(fbb, root)
}

fn query(
    body_type: wire::QueryBody,
    build: impl FnOnce(&mut FlatBufferBuilder<'static>) -> WIPOffset<UnionWIPOffset>,
) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let body = build(&mut fbb);
    let root = wire::Query::create(
        &mut fbb,
        &wire::QueryArgs {
            body_type,
            body: Some(body),
        },
    );
    finish(fbb, root)
}

impl Client {
    async fn connect(addr: std::net::SocketAddr, schema: SchemaId) -> (Client, Frame) {
        let (ws, _) = tokio_tungstenite::connect_async(format!("ws://{addr}/ws"))
            .await
            .expect("connects");
        let mut client = Client {
            ws,
            next_correlation: 1,
            snapshot: None,
            events: Vec::new(),
        };
        client
            .send_raw(FrameKind::Hello, 0, &hello_payload(), schema)
            .await;
        let first = client.next().await;
        (client, first)
    }

    async fn send_raw(
        &mut self,
        kind: FrameKind,
        correlation: u64,
        payload: &[u8],
        schema: SchemaId,
    ) {
        let meta = FrameMeta {
            kind,
            schema,
            epoch: 0,
            sequence: 0,
            correlation,
            sim_time: 0,
        };
        let bytes = commons_wire::encode(&meta, payload, true).expect("encodes");
        self.ws
            .send(Message::Binary(bytes.into()))
            .await
            .expect("sends");
    }

    async fn next(&mut self) -> Frame {
        loop {
            let message = tokio::time::timeout(TIMEOUT, self.ws.next())
                .await
                .expect("a frame arrives in time")
                .expect("the socket stays open")
                .expect("reads");
            if let Message::Binary(bytes) = message {
                let frame = commons_wire::decode(&bytes, &commons_wire::Limits::default())
                    .expect("a valid frame");
                return Frame {
                    meta: frame.header.meta,
                    payload: frame.payload.to_vec(),
                };
            }
        }
    }

    /// Keeps a snapshot or the events of a frame; returns any other frame.
    fn absorb(&mut self, frame: Frame) -> Option<Frame> {
        match frame.meta.kind {
            FrameKind::Snapshot => self.snapshot = Some(frame),
            FrameKind::Events => {
                let events =
                    flatbuffers::root::<wire::Events>(&frame.payload).expect("events decode");
                for e in events.events().iter().flatten() {
                    self.events
                        .push((e.kind(), e.text().unwrap_or_default().to_owned()));
                }
            }
            _ => return Some(frame),
        }
        None
    }

    /// The next frame that is neither a snapshot nor events (those are kept).
    async fn next_other(&mut self) -> Frame {
        loop {
            let frame = self.next().await;
            if let Some(other) = self.absorb(frame) {
                return other;
            }
        }
    }

    /// Sends a request and waits for its reply.
    async fn request(&mut self, kind: FrameKind, payload: Vec<u8>) -> Frame {
        let correlation = self.next_correlation;
        self.next_correlation += 1;
        self.send_raw(kind, correlation, &payload, WIRE_SCHEMA)
            .await;
        loop {
            let frame = self.next_other().await;
            if frame.meta.correlation == correlation {
                return frame;
            }
        }
    }

    async fn ack(&mut self, payload: Vec<u8>) {
        let reply = self.request(FrameKind::Command, payload).await;
        assert_eq!(
            reply.meta.kind,
            FrameKind::Response,
            "{}",
            error_text(&reply)
        );
    }

    /// Waits until a snapshot satisfies `accept`; returns its epoch.
    async fn wait_snapshot(&mut self, accept: impl Fn(&wire::Snapshot<'_>) -> bool) -> u32 {
        loop {
            if let Some(frame) = &self.snapshot {
                let snapshot =
                    flatbuffers::root::<wire::Snapshot>(&frame.payload).expect("snapshot decodes");
                if accept(&snapshot) {
                    return frame.meta.epoch;
                }
            }
            let frame = self.next().await;
            if let Some(other) = self.absorb(frame) {
                panic!(
                    "unexpected {} frame while waiting for a snapshot: {}",
                    other.meta.kind,
                    error_text(&other)
                );
            }
        }
    }
}

fn error_text(frame: &Frame) -> String {
    if frame.meta.kind != FrameKind::Error {
        return String::new();
    }
    let error = flatbuffers::root::<wire::ErrorInfo>(&frame.payload).expect("error decodes");
    format!(
        "{:?}: {}",
        error.code(),
        error.message().unwrap_or_default()
    )
}

fn error_code(frame: &Frame) -> Option<wire::ErrorCode> {
    (frame.meta.kind == FrameKind::Error).then(|| {
        flatbuffers::root::<wire::ErrorInfo>(&frame.payload)
            .expect("error decodes")
            .code()
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn an_observer_creates_saves_and_loads_a_world() {
    let mut host = start_host().await;
    let (mut client, welcome) = Client::connect(host.addr, WIRE_SCHEMA).await;

    assert_eq!(welcome.meta.kind, FrameKind::Welcome);
    let welcome = flatbuffers::root::<wire::Welcome>(&welcome.payload).expect("welcome decodes");
    assert!(welcome.presets().is_some_and(|p| !p.is_empty()));
    assert_eq!(
        welcome.map_sizes().map(|s| s.iter().collect::<Vec<_>>()),
        Some(civ_sim::MAP_SIZES.to_vec())
    );
    assert_eq!(welcome.speed_1x(), civ_sim::SPEED_1X);
    assert_eq!(welcome.default_map_size(), civ_sim::DEFAULT_MAP_SIZE);
    // The property regimes a new world can choose, one of them the default, each with its rules.
    let regimes: Vec<_> = welcome.regimes().iter().flatten().collect();
    assert!(regimes.len() >= 2, "{}", regimes.len());
    assert_eq!(regimes.iter().filter(|r| r.is_default()).count(), 1);
    assert!(
        regimes
            .iter()
            .all(|r| r.rules().is_some_and(|l| !l.is_empty()))
    );
    assert!(
        regimes
            .iter()
            .any(|r| r.id() == Some("core:regime/village"))
    );
    let preset = welcome
        .presets()
        .and_then(|p| p.iter().find(|p| p.is_default()))
        .and_then(|p| p.id())
        .expect("a default preset")
        .to_owned();

    // No world yet: an empty snapshot, and queries are refused.
    let epoch = client.wait_snapshot(|s| s.world().is_none()).await;
    assert_eq!(epoch, 0);
    let refused = client
        .request(
            FrameKind::Query,
            query(wire::QueryBody::GetHydrography, |fbb| {
                wire::GetHydrography::create(fbb, &wire::GetHydrographyArgs { tolerance_m: 4.0 })
                    .as_union_value()
            }),
        )
        .await;
    assert_eq!(error_code(&refused), Some(wire::ErrorCode::NoWorld));

    // Create a world and watch it arrive.
    let preset_for_command = preset.clone();
    client
        .ack(command(wire::CommandBody::NewWorld, move |fbb| {
            let preset_id = fbb.create_string(&preset_for_command);
            let name = fbb.create_string("Socket Valley");
            let regime_id = fbb.create_string("core:regime/village");
            wire::NewWorld::create(
                fbb,
                &wire::NewWorldArgs {
                    seed: 12,
                    preset_id: Some(preset_id),
                    size_cells: 256,
                    name: Some(name),
                    band_size: 32,
                    regime_id: Some(regime_id),
                    neighbours: None,
                    neighbours_known: false,
                },
            )
            .as_union_value()
        }))
        .await;
    let epoch = client
        .wait_snapshot(|s| s.world().is_some() && s.task().is_none())
        .await;
    assert_eq!(epoch, 1);
    {
        let snapshot = client
            .snapshot
            .as_ref()
            .expect("a snapshot")
            .payload
            .clone();
        let world = flatbuffers::root::<wire::Snapshot>(&snapshot)
            .expect("decodes")
            .world()
            .map(|w| {
                (
                    w.regime_id().map(str::to_owned),
                    w.regime_name().map(str::to_owned),
                )
            });
        assert_eq!(
            world,
            Some((
                Some("core:regime/village".to_owned()),
                Some("Village fields".to_owned())
            )),
            "the world lives under the regime it was made with"
        );
    }

    // Read a raster and the rivers.
    let tile = client
        .request(
            FrameKind::Query,
            query(wire::QueryBody::GetRaster, |fbb| {
                wire::GetRaster::create(
                    fbb,
                    &wire::GetRasterArgs {
                        layer: wire::RasterLayer::Elevation,
                        level: 1,
                        x0: 0,
                        y0: 0,
                        width: 1024,
                        height: 1024,
                    },
                )
                .as_union_value()
            }),
        )
        .await;
    assert_eq!(tile.meta.kind, FrameKind::Response, "{}", error_text(&tile));
    let tile = flatbuffers::root::<wire::Response>(&tile.payload)
        .expect("decodes")
        .body_as_raster_tile()
        .expect("a raster tile")
        .width();
    assert_eq!(tile, 128);
    let rivers = client
        .request(
            FrameKind::Query,
            query(wire::QueryBody::GetHydrography, |fbb| {
                wire::GetHydrography::create(fbb, &wire::GetHydrographyArgs { tolerance_m: 4.0 })
                    .as_union_value()
            }),
        )
        .await;
    assert_eq!(rivers.meta.kind, FrameKind::Response);

    // Run the clock and see it move.
    client
        .ack(command(wire::CommandBody::SetClock, |fbb| {
            wire::SetClock::create(
                fbb,
                &wire::SetClockArgs {
                    paused: false,
                    speed: civ_sim::MAX_DETAILED_SPEED,
                },
            )
            .as_union_value()
        }))
        .await;
    let start = civ_core::time::DEFAULT_WORLD_START.minutes();
    client
        .wait_snapshot(|s| s.clock().is_some_and(|c| c.minute() > start + 5))
        .await;

    // The founding band is in the snapshot; the chronicle, a person and trips can be asked for.
    let snapshot = client
        .snapshot
        .as_ref()
        .expect("a snapshot")
        .payload
        .clone();
    let snapshot = flatbuffers::root::<wire::Snapshot>(&snapshot).expect("decodes");
    let people: Vec<(u64, u64)> = snapshot
        .people()
        .iter()
        .flatten()
        .map(|p| (p.id(), p.trip()))
        .collect();
    assert_eq!(people.len(), 32, "the band asked for");
    assert_eq!(snapshot.settlements().map(|s| s.len()), Some(1));
    assert_eq!(snapshot.chronicle_head(), 2);
    let chronicle = client
        .request(
            FrameKind::Query,
            query(wire::QueryBody::GetChronicle, |fbb| {
                wire::GetChronicle::create(
                    fbb,
                    &wire::GetChronicleArgs {
                        after_seq: 0,
                        limit: 10,
                    },
                )
                .as_union_value()
            }),
        )
        .await;
    let chronicle = flatbuffers::root::<wire::Response>(&chronicle.payload)
        .expect("decodes")
        .body_as_chronicle()
        .expect("a chronicle");
    let entries: Vec<_> = chronicle.entries().iter().flatten().collect();
    assert_eq!(entries.len(), 2);
    assert!(
        entries[1]
            .spans()
            .iter()
            .flatten()
            .any(|s| s.kind() == wire::SpanKind::Settlement),
        "the settlement is a link"
    );
    let first = people[0].0;
    let person = client
        .request(
            FrameKind::Query,
            query(wire::QueryBody::GetPerson, move |fbb| {
                wire::GetPerson::create(
                    fbb,
                    &wire::GetPersonArgs {
                        id: first,
                        decisions: 4,
                    },
                )
                .as_union_value()
            }),
        )
        .await;
    assert_eq!(
        person.meta.kind,
        FrameKind::Response,
        "{}",
        error_text(&person)
    );
    let person = flatbuffers::root::<wire::Response>(&person.payload)
        .expect("decodes")
        .body_as_person_info()
        .expect("a person");
    assert!(person.alive());
    assert!(!person.name().unwrap_or_default().is_empty());
    assert!(!person.doing().unwrap_or_default().is_empty());
    assert!(person.decisions().is_some_and(|d| !d.is_empty()));
    let asked: Vec<u64> = people.iter().map(|p| p.1).filter(|&t| t != 0).collect();
    let asked_for_query = asked.clone();
    let trips = client
        .request(
            FrameKind::Query,
            query(wire::QueryBody::GetTrips, move |fbb| {
                let ids = fbb.create_vector(&asked_for_query);
                wire::GetTrips::create(fbb, &wire::GetTripsArgs { ids: Some(ids) }).as_union_value()
            }),
        )
        .await;
    let trips = flatbuffers::root::<wire::Response>(&trips.payload)
        .expect("decodes")
        .body_as_trips()
        .expect("trips");
    for t in trips.trips().iter().flatten() {
        assert!(asked.contains(&t.id()));
        assert!(t.points().is_some_and(|p| p.len() >= 2));
    }

    // The wealth measures: one settlement, its households as they stand, no year ended yet.
    let wealth = client
        .request(
            FrameKind::Query,
            query(wire::QueryBody::GetWealth, |fbb| {
                wire::GetWealth::create(fbb, &wire::GetWealthArgs {}).as_union_value()
            }),
        )
        .await;
    assert_eq!(
        wealth.meta.kind,
        FrameKind::Response,
        "{}",
        error_text(&wealth)
    );
    let wealth = flatbuffers::root::<wire::Response>(&wealth.payload)
        .expect("decodes")
        .body_as_wealth()
        .expect("wealth");
    assert_eq!(wealth.regime_name(), Some("Village fields"));
    let settlements: Vec<_> = wealth.settlements().iter().flatten().collect();
    assert_eq!(settlements.len(), 1);
    let now = settlements[0].now().expect("measures as they stand");
    assert_eq!(now.people(), 32);
    assert!(settlements[0].households().is_some_and(|h| !h.is_empty()));
    assert!(settlements[0].history().is_some_and(|h| h.is_empty()));

    // Save, list, load.
    client
        .ack(command(wire::CommandBody::SaveWorld, |fbb| {
            let label = fbb.create_string("Over the wire");
            wire::SaveWorld::create(fbb, &wire::SaveWorldArgs { label: Some(label) })
                .as_union_value()
        }))
        .await;
    let listed = client
        .request(
            FrameKind::Query,
            query(wire::QueryBody::ListSaves, |fbb| {
                wire::ListSaves::create(fbb, &wire::ListSavesArgs {}).as_union_value()
            }),
        )
        .await;
    let listed = flatbuffers::root::<wire::Response>(&listed.payload)
        .expect("decodes")
        .body_as_save_list()
        .expect("a save list");
    let saves: Vec<(String, String)> = listed
        .saves()
        .iter()
        .flatten()
        .map(|s| {
            (
                s.label().unwrap_or_default().to_owned(),
                s.file().unwrap_or_default().to_owned(),
            )
        })
        .collect();
    let manual = saves
        .iter()
        .find(|(label, _)| label == "Over the wire")
        .map(|(_, file)| file.clone())
        .expect("the manual save is listed");
    client
        .ack(command(wire::CommandBody::LoadWorld, move |fbb| {
            let file = fbb.create_string(&manual);
            wire::LoadWorld::create(fbb, &wire::LoadWorldArgs { file: Some(file) }).as_union_value()
        }))
        .await;
    let epoch = client
        .wait_snapshot(|s| {
            s.task().is_none() && s.clock().is_some_and(|c| c.paused()) && s.world().is_some()
        })
        .await;
    assert_eq!(epoch, 2, "loading replaces the world");

    // Malformed input gets an error, not a dropped connection.
    client
        .ws
        .send(Message::Binary(vec![1, 2, 3].into()))
        .await
        .expect("sends");
    let error = client.next_other().await;
    assert_eq!(error_code(&error), Some(wire::ErrorCode::BadRequest));

    let kinds: Vec<wire::EventKind> = client.events.iter().map(|(k, _)| *k).collect();
    for expected in [
        wire::EventKind::WorldCreated,
        wire::EventKind::Autosaved,
        wire::EventKind::Saved,
        wire::EventKind::Loaded,
    ] {
        assert!(
            kinds.contains(&expected),
            "{expected:?} missing from {kinds:?}"
        );
    }

    let engine = host.engine.take().expect("engine");
    tokio::task::spawn_blocking(move || engine.shutdown())
        .await
        .expect("shuts down");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_client_with_another_schema_major_is_refused() {
    let mut host = start_host().await;
    let other = SchemaId::new(*b"TCE\0", WIRE_SCHEMA.major + 1, 0);
    let (_client, reply) = Client::connect(host.addr, other).await;
    assert_eq!(error_code(&reply), Some(wire::ErrorCode::Incompatible));
    let engine = host.engine.take().expect("engine");
    tokio::task::spawn_blocking(move || engine.shutdown())
        .await
        .expect("shuts down");
}

/// Opens the socket as a page from `origin` would, at `/ws` plus `query`. Returns the HTTP status
/// of a refusal, or `None` when the socket opened.
async fn refusal(addr: std::net::SocketAddr, origin: Option<&str>, query: &str) -> Option<u16> {
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;
    let mut request = format!("ws://{addr}/ws{query}")
        .into_client_request()
        .expect("request");
    if let Some(origin) = origin {
        request
            .headers_mut()
            .insert("Origin", origin.parse().expect("header value"));
    }
    match tokio_tungstenite::connect_async(request).await {
        Ok(_) => None,
        Err(tokio_tungstenite::tungstenite::Error::Http(response)) => {
            Some(response.status().as_u16())
        }
        Err(e) => panic!("the socket failed otherwise: {e}"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pages_from_other_sites_are_refused() {
    let mut host = start_host().await;
    let own = format!("http://127.0.0.1:{}", host.addr.port());
    assert_eq!(
        refusal(host.addr, Some("https://example.com"), "").await,
        Some(403)
    );
    assert_eq!(
        refusal(host.addr, Some("http://127.0.0.1.example.com"), "").await,
        Some(403)
    );
    assert_eq!(refusal(host.addr, Some("null"), "").await, Some(403));
    assert_eq!(refusal(host.addr, Some(&own), "").await, None);
    assert_eq!(
        refusal(host.addr, Some("http://localhost:5173"), "").await,
        None
    );
    assert_eq!(refusal(host.addr, None, "").await, None);
    let engine = host.engine.take().expect("engine");
    tokio::task::spawn_blocking(move || engine.shutdown())
        .await
        .expect("shuts down");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_host_with_a_token_refuses_sockets_without_it() {
    let mut host = start_host_with(server::Access {
        token: Some("0123abcd".to_owned()),
    })
    .await;
    assert_eq!(refusal(host.addr, None, "").await, Some(403));
    assert_eq!(refusal(host.addr, None, "?token=0123abce").await, Some(403));
    assert_eq!(refusal(host.addr, None, "?token=0123abcd").await, None);
    assert_eq!(refusal(host.addr, None, "?a=1&token=0123abcd").await, None);
    let engine = host.engine.take().expect("engine");
    tokio::task::spawn_blocking(move || engine.shutdown())
        .await
        .expect("shuts down");
}
