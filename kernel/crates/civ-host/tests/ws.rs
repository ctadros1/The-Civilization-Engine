//! The observer protocol end to end over a real WebSocket (ADR-0001): handshake, snapshots,
//! events, commands and queries, saving and loading; and viewers, who drive or watch
//! (ADR-0039 §6).

use std::net::{Ipv4Addr, SocketAddr};
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
    start_host_on(access, None).await
}

/// A host whose every connection seems to come from `peer` when it is given (as from the local
/// network); otherwise each comes from where it does, here this machine.
async fn start_host_on(access: server::Access, peer: Option<SocketAddr>) -> Host {
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
        let _ = match peer {
            Some(peer) => {
                let app = app.layer(axum::extract::connect_info::MockConnectInfo(peer));
                axum::serve(listener, app).await
            }
            None => {
                axum::serve(
                    listener,
                    app.into_make_service_with_connect_info::<SocketAddr>(),
                )
                .await
            }
        };
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
    /// Who watches and who drives, as the host last said.
    viewers: Option<Seen>,
}

/// The viewers the host told of in an `Events` frame (wire 1.110).
#[derive(Clone, Debug, PartialEq)]
struct Seen {
    you: u64,
    role: wire::ViewerRole,
    driver: u64,
    /// Every connection's id and role, oldest first.
    viewers: Vec<(u64, wire::ViewerRole)>,
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
        Client::connect_with(addr, schema, "").await
    }

    /// Connects at `/ws` plus `query`.
    async fn connect_with(
        addr: std::net::SocketAddr,
        schema: SchemaId,
        query: &str,
    ) -> (Client, Frame) {
        let (ws, _) = tokio_tungstenite::connect_async(format!("ws://{addr}/ws{query}"))
            .await
            .expect("connects");
        let mut client = Client {
            ws,
            next_correlation: 1,
            snapshot: None,
            events: Vec::new(),
            viewers: None,
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
                if let Some(v) = events.viewers() {
                    self.viewers = Some(Seen {
                        you: v.you(),
                        role: v.role(),
                        driver: v.driver(),
                        viewers: v
                            .viewers()
                            .iter()
                            .flatten()
                            .map(|c| (c.id(), c.role()))
                            .collect(),
                    });
                }
            }
            _ => return Some(frame),
        }
        None
    }

    /// Waits until the host says something of the viewers that satisfies `accept`.
    async fn wait_viewers(&mut self, accept: impl Fn(&Seen) -> bool) -> Seen {
        loop {
            if let Some(seen) = self.viewers.as_ref().filter(|v| accept(v)) {
                return seen.clone();
            }
            let frame = self.next().await;
            if let Some(other) = self.absorb(frame) {
                panic!(
                    "unexpected {} frame while waiting for the viewers: {}",
                    other.meta.kind,
                    error_text(&other)
                );
            }
        }
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
    refusal_at(addr, origin, None, query).await
}

/// As [`refusal`], with the `Host` the request names when `host` is given (the address a page on
/// the local network reached this host at).
async fn refusal_at(
    addr: std::net::SocketAddr,
    origin: Option<&str>,
    host: Option<&str>,
    query: &str,
) -> Option<u16> {
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;
    let mut request = format!("ws://{addr}/ws{query}")
        .into_client_request()
        .expect("request");
    if let Some(origin) = origin {
        request
            .headers_mut()
            .insert("Origin", origin.parse().expect("header value"));
    }
    if let Some(host) = host {
        request
            .headers_mut()
            .insert("Host", host.parse().expect("header value"));
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
        lan: false,
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

fn set_clock() -> Vec<u8> {
    command(wire::CommandBody::SetClock, |fbb| {
        wire::SetClock::create(
            fbb,
            &wire::SetClockArgs {
                paused: false,
                speed: 60.0,
            },
        )
        .as_union_value()
    })
}

fn hand_over(to: u64) -> Vec<u8> {
    command(wire::CommandBody::HandOver, |fbb| {
        wire::HandOver::create(fbb, &wire::HandOverArgs { to }).as_union_value()
    })
}

fn list_saves() -> Vec<u8> {
    query(wire::QueryBody::ListSaves, |fbb| {
        wire::ListSaves::create(fbb, &wire::ListSavesArgs {}).as_union_value()
    })
}

fn error_message(frame: &Frame) -> String {
    assert_eq!(frame.meta.kind, FrameKind::Error);
    flatbuffers::root::<wire::ErrorInfo>(&frame.payload)
        .expect("error decodes")
        .message()
        .unwrap_or_default()
        .to_owned()
}

/// A spectator's command is refused by name; a driver's reaches the engine (which, with no world,
/// answers that there is none).
async fn assert_watches(client: &mut Client) {
    let refused = client.request(FrameKind::Command, set_clock()).await;
    assert_eq!(error_code(&refused), Some(wire::ErrorCode::NotDriver));
    assert_eq!(error_message(&refused), server::SPECTATOR_REFUSAL);
    let answered = client.request(FrameKind::Query, list_saves()).await;
    assert_eq!(
        answered.meta.kind,
        FrameKind::Response,
        "{}",
        error_text(&answered)
    );
}

async fn assert_drives(client: &mut Client) {
    let reached = client.request(FrameKind::Command, set_clock()).await;
    assert_eq!(error_code(&reached), Some(wire::ErrorCode::NoWorld));
}

async fn stop(mut host: Host) {
    let engine = host.engine.take().expect("engine");
    tokio::task::spawn_blocking(move || engine.shutdown())
        .await
        .expect("shuts down");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_first_local_viewer_drives_and_the_others_watch() {
    let host = start_host().await;
    let (mut ada, welcome) = Client::connect(host.addr, WIRE_SCHEMA).await;
    assert_eq!(welcome.meta.kind, FrameKind::Welcome);
    let first = ada.wait_viewers(|_| true).await;
    assert_eq!(first.role, wire::ViewerRole::Driver);
    assert_eq!(first.driver, first.you);

    let (mut bo, _) = Client::connect(host.addr, WIRE_SCHEMA).await;
    let second = bo.wait_viewers(|_| true).await;
    assert_eq!(second.role, wire::ViewerRole::Spectator);
    assert_eq!(second.driver, first.you);
    assert_eq!(
        second.viewers,
        vec![
            (first.you, wire::ViewerRole::Driver),
            (second.you, wire::ViewerRole::Spectator)
        ]
    );
    // The driver is told who came.
    ada.wait_viewers(|v| v.viewers.len() == 2).await;

    // The spectator receives snapshots as the driver does, and is answered its queries; its
    // commands, a hand-over among them, are refused by name.
    bo.wait_snapshot(|s| s.world().is_none()).await;
    assert_watches(&mut bo).await;
    let refused = bo.request(FrameKind::Command, hand_over(second.you)).await;
    assert_eq!(error_code(&refused), Some(wire::ErrorCode::NotDriver));
    assert_drives(&mut ada).await;
    stop(host).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_driver_hands_over_and_when_it_leaves_the_oldest_local_viewer_drives() {
    let host = start_host().await;
    let (mut ada, _) = Client::connect(host.addr, WIRE_SCHEMA).await;
    let a = ada.wait_viewers(|_| true).await.you;
    let (mut bo, _) = Client::connect(host.addr, WIRE_SCHEMA).await;
    let b = bo.wait_viewers(|_| true).await.you;
    let (mut cy, _) = Client::connect(host.addr, WIRE_SCHEMA).await;
    let c = cy.wait_viewers(|_| true).await.you;
    ada.wait_viewers(|v| v.viewers.len() == 3).await;

    // Only someone connected can take over, and the driver cannot hand to itself.
    let unknown = ada.request(FrameKind::Command, hand_over(99)).await;
    assert_eq!(error_code(&unknown), Some(wire::ErrorCode::NotFound));
    let itself = ada.request(FrameKind::Command, hand_over(a)).await;
    assert_eq!(error_code(&itself), Some(wire::ErrorCode::BadRequest));

    // The driver hands over to the newest; everyone is told.
    let handed = ada.request(FrameKind::Command, hand_over(c)).await;
    assert_eq!(
        handed.meta.kind,
        FrameKind::Response,
        "{}",
        error_text(&handed)
    );
    let told = ada.wait_viewers(|v| v.driver == c).await;
    assert_eq!(told.role, wire::ViewerRole::Spectator);
    bo.wait_viewers(|v| v.driver == c).await;
    let took = cy.wait_viewers(|v| v.driver == c).await;
    assert_eq!(took.role, wire::ViewerRole::Driver);
    assert_watches(&mut ada).await;
    assert_drives(&mut cy).await;

    // The driver leaves: the oldest left from this machine drives, and is told so.
    cy.ws.close(None).await.expect("closes");
    let promoted = ada.wait_viewers(|v| v.viewers.len() == 2).await;
    assert_eq!(promoted.driver, a);
    assert_eq!(promoted.role, wire::ViewerRole::Driver);
    let seen = bo.wait_viewers(|v| v.viewers.len() == 2).await;
    assert_eq!(seen.driver, a);
    assert_drives(&mut ada).await;
    assert_watches(&mut bo).await;
    assert_eq!(seen.you, b);
    stop(host).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_local_network_needs_the_token_and_watches() {
    let lan = server::Access {
        token: Some("0123abcd".to_owned()),
        lan: true,
    };
    let peer = SocketAddr::from(([192, 168, 1, 9], 50_000));
    let host = start_host_on(lan, Some(peer)).await;
    assert_eq!(refusal(host.addr, None, "").await, Some(403));
    assert_eq!(refusal(host.addr, None, "?token=0123abce").await, Some(403));
    // A page this host served at its address on the network may connect; other sites may not.
    let served = "192.168.1.5:7420";
    assert_eq!(
        refusal_at(
            host.addr,
            Some(&format!("http://{served}")),
            Some(served),
            "?token=0123abcd"
        )
        .await,
        None
    );
    assert_eq!(
        refusal_at(
            host.addr,
            Some("http://192.168.1.77:7420"),
            Some(served),
            "?token=0123abcd"
        )
        .await,
        Some(403)
    );
    assert_eq!(
        refusal_at(
            host.addr,
            Some("https://example.com"),
            Some(served),
            "?token=0123abcd"
        )
        .await,
        Some(403)
    );

    // With the token, a viewer from the network watches; nobody on this machine is connected, so
    // nobody drives.
    let (mut far, welcome) = Client::connect_with(host.addr, WIRE_SCHEMA, "?token=0123abcd").await;
    assert_eq!(welcome.meta.kind, FrameKind::Welcome);
    let seen = far.wait_viewers(|_| true).await;
    assert_eq!(seen.role, wire::ViewerRole::Spectator);
    assert_eq!(seen.driver, 0);
    assert_watches(&mut far).await;
    stop(host).await;

    // Open to the network with no token set, nobody may connect.
    let host = start_host_on(
        server::Access {
            token: None,
            lan: true,
        },
        Some(peer),
    )
    .await;
    assert_eq!(refusal(host.addr, None, "").await, Some(403));
    stop(host).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn by_default_a_page_from_the_network_is_refused() {
    let host = start_host().await;
    let served = "192.168.1.5:7420";
    assert_eq!(
        refusal_at(
            host.addr,
            Some(&format!("http://{served}")),
            Some(served),
            ""
        )
        .await,
        Some(403)
    );
    stop(host).await;
}
