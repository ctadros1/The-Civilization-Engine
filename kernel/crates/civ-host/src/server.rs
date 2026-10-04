//! The localhost server: the web shell's static files and the observer WebSocket at `/ws`.
//!
//! Each connection follows ADR-0001: the client says `Hello` and the host answers `Welcome`, then
//! sends the current snapshot and recent events, then live snapshots (replaceable), events
//! (ordered) and the replies to the client's commands and queries, matched by correlation id.
//!
//! The socket refuses pages served from anywhere but this machine (a browser's `Origin`), so a
//! web site open in the same browser cannot drive the host, and it can require a token in its
//! address (`/ws?token=…`), as the kernel's panel server does (research 14-10 §4.5, ADR-0005 §6).

use std::path::PathBuf;
use std::sync::{Arc, PoisonError};
use std::time::Duration;

use axum::Router;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{RawQuery, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use civ_schema::wire;
use commons_wire::{FrameKind, Sequencer};
use tokio::sync::{broadcast, mpsc, oneshot};
use tower_http::services::{ServeDir, ServeFile};

use crate::engine::{Channels, Msg, Reply};
use crate::protocol::{self, EventRecord, Incoming};

/// How long a client has to say hello.
const HELLO_TIMEOUT: Duration = Duration::from_secs(10);
/// Largest WebSocket message accepted from a client.
const MAX_CLIENT_MESSAGE: usize = 1 << 20;

const MISSING_WEB_SHELL: &str = r#"<!doctype html>
<html lang="en">
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>The Civilization Engine</title>
<body style="font: 16px/1.5 system-ui, sans-serif; max-width: 40em; margin: 4em auto; padding: 0 1em">
<h1>The web shell is not built</h1>
<p>The host is running, but there is no <code>web/dist</code> to serve. Build it once:</p>
<pre>cd web
npm ci
npm run build</pre>
<p>Then reload this page. <code>tools/run.sh</code> and <code>tools/run.ps1</code> build it for you.</p>
</body>
</html>
"#;

/// Who may open the observer socket, beyond the rule that pages must come from this machine.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Access {
    /// A token the socket's address must carry as `?token=…`; `None` lets anyone on this machine
    /// connect.
    pub token: Option<String>,
}

#[derive(Clone)]
struct AppState {
    channels: Channels,
    welcome: Arc<Vec<u8>>,
    access: Arc<Access>,
}

/// The HTTP application: `/ws`, `/healthz`, and the web shell from `web_dir` (or a page saying
/// how to build it).
pub fn router(
    channels: Channels,
    welcome: Vec<u8>,
    web_dir: Option<PathBuf>,
    access: Access,
) -> Router {
    let state = AppState {
        channels,
        welcome: Arc::new(welcome),
        access: Arc::new(access),
    };
    let app = Router::new()
        .route("/ws", get(upgrade))
        .route("/healthz", get(|| async { "ok" }))
        .with_state(state);
    match web_dir {
        Some(dir) => {
            let index = dir.join("index.html");
            app.fallback_service(ServeDir::new(dir).fallback(ServeFile::new(index)))
        }
        None => app.fallback(missing_web_shell),
    }
}

async fn missing_web_shell() -> Response {
    (StatusCode::SERVICE_UNAVAILABLE, Html(MISSING_WEB_SHELL)).into_response()
}

async fn upgrade(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    headers: HeaderMap,
    RawQuery(query): RawQuery,
) -> Response {
    let origin = headers
        .get(header::ORIGIN)
        .map(|v| v.to_str().unwrap_or("?"));
    if let Some(origin) = origin.filter(|o| !is_local_origin(o)) {
        tracing::warn!("refused an observer from {origin}");
        return (
            StatusCode::FORBIDDEN,
            "pages from other sites may not connect",
        )
            .into_response();
    }
    if let Some(token) = &state.access.token {
        let given = query
            .as_deref()
            .and_then(|q| q.split('&').find_map(|pair| pair.strip_prefix("token=")))
            .unwrap_or("");
        if !same_secret(given.as_bytes(), token.as_bytes()) {
            tracing::warn!("refused an observer without the session's token");
            return (
                StatusCode::FORBIDDEN,
                "this socket needs its session's token",
            )
                .into_response();
        }
    }
    ws.max_message_size(MAX_CLIENT_MESSAGE)
        .on_upgrade(move |socket| async move {
            if let Err(e) = connection(socket, state).await {
                tracing::debug!("an observer connection ended: {e}");
            }
        })
}

/// Whether a browser's `Origin` is a page served from this machine: `http` or `https` on
/// 127.0.0.1, `localhost` or `[::1]`, at any port. `null` (a file or sandboxed page) is not.
pub fn is_local_origin(origin: &str) -> bool {
    let Some(rest) = origin
        .strip_prefix("http://")
        .or_else(|| origin.strip_prefix("https://"))
    else {
        return false;
    };
    let authority = rest.split('/').next().unwrap_or("");
    let host = if authority.starts_with('[') {
        authority.split_inclusive(']').next().unwrap_or("")
    } else {
        authority.split(':').next().unwrap_or("")
    };
    matches!(
        host.to_ascii_lowercase().as_str(),
        "127.0.0.1" | "localhost" | "[::1]"
    )
}

/// Compares two secrets in time that does not depend on where they differ.
fn same_secret(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

async fn send(socket: &mut WebSocket, frame: Vec<u8>) -> Result<(), axum::Error> {
    socket.send(Message::Binary(frame.into())).await
}

/// The next binary message, skipping pings and text; `None` once the socket closes.
async fn next_binary(socket: &mut WebSocket) -> Option<Vec<u8>> {
    while let Some(Ok(message)) = socket.recv().await {
        match message {
            Message::Binary(bytes) => return Some(bytes.to_vec()),
            Message::Close(_) => return None,
            _ => {}
        }
    }
    None
}

fn error_frame(
    seq: &mut Sequencer,
    correlation: u64,
    code: wire::ErrorCode,
    message: &str,
) -> Result<Vec<u8>, commons_wire::WireError> {
    protocol::frame(
        seq,
        FrameKind::Error,
        correlation,
        0,
        &protocol::error_payload(code, message),
    )
}

async fn connection(mut socket: WebSocket, state: AppState) -> anyhow::Result<()> {
    let Ok(Some(first)) = tokio::time::timeout(HELLO_TIMEOUT, next_binary(&mut socket)).await
    else {
        return Ok(());
    };
    let mut snapshots = state.channels.snapshot.clone();
    let current = Arc::clone(&snapshots.borrow_and_update());
    let mut seq = Sequencer::new(current.epoch);
    let hello = commons_wire::decode(&first, &protocol::CLIENT_LIMITS)
        .map_err(|e| format!("malformed hello: {e}"))
        .and_then(|frame| protocol::check_hello(&frame));
    let client = match hello {
        Ok(client) => client,
        Err(why) => {
            tracing::warn!("refused an observer: {why}");
            send(
                &mut socket,
                error_frame(&mut seq, 0, wire::ErrorCode::Incompatible, &why)?,
            )
            .await?;
            let _ = socket.send(Message::Close(None)).await;
            return Ok(());
        }
    };
    tracing::info!("observer connected: {client}");
    send(
        &mut socket,
        protocol::frame(&mut seq, FrameKind::Welcome, 0, 0, &state.welcome)?,
    )
    .await?;

    // Subscribe before reading the history, so nothing falls between them (duplicates are
    // skipped by id).
    let mut events = state.channels.events.subscribe();
    send(
        &mut socket,
        protocol::frame(
            &mut seq,
            FrameKind::Snapshot,
            0,
            current.sim_time,
            &current.payload,
        )?,
    )
    .await?;
    let mut last_event = 0;
    if let Some(frame) = catch_up(&state, &mut seq, &mut last_event, current.sim_time)? {
        send(&mut socket, frame).await?;
    }

    let (reply_tx, mut replies) = mpsc::unbounded_channel::<(u64, Reply)>();
    loop {
        tokio::select! {
            incoming = socket.recv() => match incoming {
                None | Some(Ok(Message::Close(_))) => break,
                Some(Err(e)) => return Err(e.into()),
                Some(Ok(Message::Binary(bytes))) => {
                    if let Some(frame) = on_frame(&bytes, &state, &reply_tx, &mut seq)? {
                        send(&mut socket, frame).await?;
                    }
                }
                Some(Ok(_)) => {}
            },
            changed = snapshots.changed() => {
                if changed.is_err() {
                    break; // the engine stopped
                }
                let snapshot = Arc::clone(&snapshots.borrow_and_update());
                if snapshot.epoch != seq.epoch() {
                    seq = Sequencer::new(snapshot.epoch);
                }
                let frame = protocol::frame(
                    &mut seq,
                    FrameKind::Snapshot,
                    0,
                    snapshot.sim_time,
                    &snapshot.payload,
                )?;
                send(&mut socket, frame).await?;
            },
            batch = events.recv() => match batch {
                Ok(batch) => {
                    let fresh: Vec<EventRecord> = batch
                        .events
                        .iter()
                        .filter(|e| e.id > last_event)
                        .cloned()
                        .collect();
                    if let Some(last) = fresh.last() {
                        last_event = last.id;
                        let frame = protocol::frame(
                            &mut seq,
                            FrameKind::Events,
                            0,
                            batch.sim_time,
                            &protocol::events_payload(&fresh),
                        )?;
                        send(&mut socket, frame).await?;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    let sim_time = snapshots.borrow().sim_time;
                    if let Some(frame) = catch_up(&state, &mut seq, &mut last_event, sim_time)? {
                        send(&mut socket, frame).await?;
                    }
                }
                Err(broadcast::error::RecvError::Closed) => break,
            },
            Some((correlation, reply)) = replies.recv() => {
                let frame = match reply {
                    Reply::Response(payload) => {
                        protocol::frame(&mut seq, FrameKind::Response, correlation, 0, &payload)?
                    }
                    Reply::Error(code, message) => {
                        error_frame(&mut seq, correlation, code, &message)?
                    }
                };
                send(&mut socket, frame).await?;
            },
        }
    }
    tracing::info!("observer disconnected: {client}");
    Ok(())
}

/// An `Events` frame with the remembered events after `last_event`, if there are any.
fn catch_up(
    state: &AppState,
    seq: &mut Sequencer,
    last_event: &mut u64,
    sim_time: i64,
) -> Result<Option<Vec<u8>>, commons_wire::WireError> {
    let missed: Vec<EventRecord> = state
        .channels
        .history
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .iter()
        .filter(|e| e.id > *last_event)
        .cloned()
        .collect();
    let Some(last) = missed.last() else {
        return Ok(None);
    };
    *last_event = last.id;
    protocol::frame(
        seq,
        FrameKind::Events,
        0,
        sim_time,
        &protocol::events_payload(&missed),
    )
    .map(Some)
}

/// Handles a frame from the client. Returns a frame to send at once (an error), or `None` when
/// the request went to the engine; its reply arrives through `replies`.
fn on_frame(
    bytes: &[u8],
    state: &AppState,
    replies: &mpsc::UnboundedSender<(u64, Reply)>,
    seq: &mut Sequencer,
) -> Result<Option<Vec<u8>>, commons_wire::WireError> {
    let (correlation, request) = match protocol::read_request(bytes) {
        Ok(Incoming::Request(correlation, request)) => (correlation, request),
        Ok(Incoming::Nothing) => return Ok(None),
        Err(refusal) => {
            return error_frame(seq, refusal.correlation, refusal.code, &refusal.message).map(Some);
        }
    };
    let (tx, rx) = oneshot::channel();
    if state
        .channels
        .requests
        .send(Msg::Request(request, tx))
        .is_err()
    {
        return error_frame(
            seq,
            correlation,
            wire::ErrorCode::Internal,
            "the engine has stopped",
        )
        .map(Some);
    }
    let replies = replies.clone();
    tokio::spawn(async move {
        let reply = rx.await.unwrap_or_else(|_| {
            Reply::Error(
                wire::ErrorCode::Internal,
                "the engine failed while handling this request".to_owned(),
            )
        });
        let _ = replies.send((correlation, reply));
    });
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_origins_are_this_machine_at_any_port() {
        for origin in [
            "http://127.0.0.1:7420",
            "http://localhost:5173",
            "https://LOCALHOST",
            "http://[::1]:7420",
            "http://127.0.0.1",
        ] {
            assert!(is_local_origin(origin), "{origin}");
        }
        for origin in [
            "null",
            "https://example.com",
            "http://127.0.0.1.example.com",
            "http://localhost.example.com:7420",
            "http://example.com/127.0.0.1",
            "file://",
            "ws://127.0.0.1:7420",
            "",
        ] {
            assert!(!is_local_origin(origin), "{origin}");
        }
    }

    #[test]
    fn secrets_match_only_when_equal() {
        assert!(same_secret(b"abc", b"abc"));
        assert!(!same_secret(b"abc", b"abd"));
        assert!(!same_secret(b"abc", b"abcd"));
        assert!(same_secret(b"", b""));
    }
}
