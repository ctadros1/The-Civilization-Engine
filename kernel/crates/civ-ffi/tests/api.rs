//! The C interface driven from Rust through the table `tce_get_api` hands out, as a host would
//! (ADR-0005): versions, a kernel that cannot start, frames in and out, buffers too small,
//! refused frames, the panel server and its token, and a world saved when the kernel stops.

// Calling C functions through raw pointers is what a host does; every call says why it is sound.
#![allow(unsafe_code)]

use std::io::{Read, Write};
use std::mem::MaybeUninit;
use std::net::TcpStream;
use std::path::Path;
use std::ptr;
use std::time::{Duration, Instant};

use civ_schema::flatbuffers::{self, FlatBufferBuilder};
use civ_schema::{WIRE_SCHEMA, wire};
use commons_wire::{FrameKind, FrameMeta};
use tce_kernel::{
    TCE_ABI_MAJOR, TCE_BAD_FRAME, TCE_BUFFER_TOO_SMALL, TCE_FAILED, TCE_INVALID_ARGUMENT, TCE_NONE,
    TCE_OK, TceApiV1, TceBytes, TceConfig, TceKernel, TcePanelConfig, TceVersions, tce_get_api,
};

const TIMEOUT: Duration = Duration::from_secs(120);

fn api() -> TceApiV1 {
    let mut table = MaybeUninit::<TceApiV1>::uninit();
    // SAFETY: room for a whole table.
    let code = unsafe { tce_get_api(TCE_ABI_MAJOR, table.as_mut_ptr(), size_of::<TceApiV1>()) };
    assert_eq!(code, TCE_OK);
    // SAFETY: `tce_get_api` wrote the whole table.
    unsafe { table.assume_init() }
}

fn bytes(text: &str) -> TceBytes {
    TceBytes {
        ptr: text.as_ptr(),
        len: text.len(),
    }
}

fn content_dir() -> String {
    civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
        .expect("content/ is above the crate")
        .to_string_lossy()
        .into_owned()
}

/// A kernel and the table it is driven through; destroyed when dropped.
struct Host {
    api: TceApiV1,
    kernel: *mut TceKernel,
    next_correlation: u64,
}

impl Host {
    fn create(content: &str, saves: &str) -> (Host, i32) {
        let api = api();
        let config = TceConfig {
            struct_size: size_of::<TceConfig>() as u32,
            flags: 0,
            content_dir: bytes(content),
            saves_dir: bytes(saves),
        };
        let mut kernel = ptr::null_mut();
        // SAFETY: a valid config whose strings outlive the call; `kernel` is writable.
        let code = unsafe { (api.create.expect("create"))(&config, &mut kernel) };
        assert!(!kernel.is_null(), "create returned no kernel ({code})");
        (
            Host {
                api,
                kernel,
                next_correlation: 1,
            },
            code,
        )
    }

    fn status(&self) -> i32 {
        // SAFETY: a live kernel.
        unsafe { (self.api.status.expect("status"))(self.kernel) }
    }

    fn last_error(&self) -> String {
        let last_error = self.api.last_error.expect("last_error");
        let mut len = 0;
        // SAFETY: a live kernel; a zero-capacity buffer asks for the size.
        let code = unsafe { last_error(self.kernel, ptr::null_mut(), 0, &mut len) };
        if code == TCE_OK {
            return String::new();
        }
        assert_eq!(code, TCE_BUFFER_TOO_SMALL);
        let mut buf = vec![0u8; len];
        // SAFETY: a live kernel and a buffer of `len` bytes.
        let code = unsafe { last_error(self.kernel, buf.as_mut_ptr(), len, &mut len) };
        assert_eq!(code, TCE_OK);
        String::from_utf8(buf).expect("UTF-8")
    }

    fn submit(&self, frame: &[u8]) -> i32 {
        // SAFETY: a live kernel and the frame's bytes for the call.
        unsafe { (self.api.submit.expect("submit"))(self.kernel, frame.as_ptr(), frame.len()) }
    }

    /// The next ordered frame, if one is waiting: asks for its size, then copies it.
    fn poll(&self) -> Option<Vec<u8>> {
        let poll = self.api.poll.expect("poll");
        let mut len = 0;
        // SAFETY: a live kernel; a zero-capacity buffer asks for the size.
        let code = unsafe { poll(self.kernel, ptr::null_mut(), 0, &mut len) };
        if code == TCE_NONE {
            return None;
        }
        assert_eq!(code, TCE_BUFFER_TOO_SMALL);
        let mut buf = vec![0u8; len];
        // SAFETY: a live kernel and a buffer of `len` bytes.
        let code = unsafe { poll(self.kernel, buf.as_mut_ptr(), len, &mut len) };
        assert_eq!(code, TCE_OK);
        buf.truncate(len);
        Some(buf)
    }

    /// The latest snapshot newer than `after`, with its number.
    fn snapshot(&self, after: u64) -> Option<(Vec<u8>, u64)> {
        let mut buf = vec![0u8; 1 << 20];
        let (mut len, mut number) = (0, 0);
        // SAFETY: a live kernel and a buffer of the given size.
        let code = unsafe {
            (self.api.copy_snapshot.expect("copy_snapshot"))(
                self.kernel,
                after,
                buf.as_mut_ptr(),
                buf.len(),
                &mut len,
                &mut number,
            )
        };
        if code == TCE_NONE {
            return None;
        }
        assert_eq!(code, TCE_OK);
        buf.truncate(len);
        Some((buf, number))
    }

    /// A frame of `kind` carrying `payload`, with a fresh correlation id.
    fn request(&mut self, kind: FrameKind, payload: &[u8]) -> (u64, Vec<u8>) {
        let correlation = self.next_correlation;
        self.next_correlation += 1;
        let meta = FrameMeta {
            kind,
            schema: WIRE_SCHEMA,
            epoch: 0,
            sequence: correlation,
            correlation,
            sim_time: 0,
        };
        let frame = commons_wire::encode(&meta, payload, false).expect("frame");
        (correlation, frame)
    }

    /// Waits for the reply to `correlation`, keeping the other frames it passes.
    fn reply(&self, correlation: u64, passed: &mut Vec<Vec<u8>>) -> Vec<u8> {
        let started = Instant::now();
        loop {
            while let Some(frame) = self.poll() {
                if header(&frame).correlation == correlation {
                    return frame;
                }
                passed.push(frame);
            }
            assert!(started.elapsed() < TIMEOUT, "no reply to {correlation}");
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

impl Drop for Host {
    fn drop(&mut self) {
        // SAFETY: the kernel `create` returned, not used again.
        let code = unsafe { (self.api.destroy.expect("destroy"))(self.kernel) };
        assert_eq!(code, TCE_OK);
    }
}

fn header(frame: &[u8]) -> FrameMeta {
    commons_wire::decode(frame, &commons_wire::Limits::default())
        .expect("a frame")
        .header
        .meta
}

fn payload(frame: &[u8]) -> Vec<u8> {
    commons_wire::decode(frame, &commons_wire::Limits::default())
        .expect("a frame")
        .payload
        .to_vec()
}

fn new_world_command(preset: &str) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let preset_id = fbb.create_string(preset);
    let name = fbb.create_string("Library Valley");
    let body = wire::NewWorld::create(
        &mut fbb,
        &wire::NewWorldArgs {
            seed: 12,
            preset_id: Some(preset_id),
            size_cells: 256,
            name: Some(name),
            band_size: 32,
        },
    )
    .as_union_value();
    let root = wire::Command::create(
        &mut fbb,
        &wire::CommandArgs {
            body_type: wire::CommandBody::NewWorld,
            body: Some(body),
        },
    );
    fbb.finish(root, None);
    fbb.finished_data().to_vec()
}

#[test]
fn the_library_says_which_versions_it_speaks() {
    let api = api();
    let versions_of = api.versions.expect("versions");
    let mut versions = TceVersions {
        struct_size: size_of::<TceVersions>() as u32,
        ..TceVersions::default()
    };
    // SAFETY: a writable `TceVersions` with its size set.
    assert_eq!(unsafe { versions_of(&mut versions) }, TCE_OK);
    assert_eq!((versions.abi_major, versions.abi_minor), (1, 0));
    assert_eq!(versions.wire_major, u32::from(WIRE_SCHEMA.major));
    assert_eq!(versions.wire_minor, u32::from(WIRE_SCHEMA.minor));
    assert_eq!(versions.save_schema, civ_schema::SAVE_SCHEMA_VERSION);
    assert_eq!(versions.content_api, civ_content::KERNEL_CONTENT_API);
    let mut small = TceVersions {
        struct_size: 4,
        ..TceVersions::default()
    };
    // SAFETY: as above; the size is too small, so nothing is written.
    assert_eq!(unsafe { versions_of(&mut small) }, TCE_INVALID_ARGUMENT);
    assert_eq!(small.abi_major, 0);
}

#[test]
fn a_kernel_without_its_content_fails_and_says_why() {
    let saves = tempfile::tempdir().expect("temp dir");
    let missing = saves.path().join("no-content");
    let (host, code) = Host::create(&missing.to_string_lossy(), &saves.path().to_string_lossy());
    assert_eq!(code, TCE_FAILED);
    assert_eq!(host.status(), TCE_FAILED);
    assert!(
        host.last_error().contains("content"),
        "{}",
        host.last_error()
    );
    let mut len = 0;
    // SAFETY: a live (failed) kernel; a zero-capacity buffer.
    let code = unsafe { (host.api.poll.expect("poll"))(host.kernel, ptr::null_mut(), 0, &mut len) };
    assert_eq!(code, TCE_FAILED);
}

#[test]
fn bad_arguments_create_nothing() {
    let api = api();
    let create = api.create.expect("create");
    let mut kernel = ptr::null_mut();
    let config = TceConfig {
        struct_size: size_of::<TceConfig>() as u32,
        flags: 0,
        content_dir: TceBytes {
            ptr: ptr::null(),
            len: 0,
        },
        saves_dir: bytes("saves"),
    };
    // SAFETY: a valid config (with an empty content path) and a writable out pointer.
    assert_eq!(
        unsafe { create(&config, &mut kernel) },
        TCE_INVALID_ARGUMENT
    );
    assert!(kernel.is_null());
    let short = TceConfig {
        struct_size: 8,
        ..config
    };
    // SAFETY: as above, with a size smaller than this ABI's.
    assert_eq!(unsafe { create(&short, &mut kernel) }, TCE_INVALID_ARGUMENT);
    // SAFETY: null arguments are refused, not followed.
    assert_eq!(
        unsafe { create(ptr::null(), &mut kernel) },
        TCE_INVALID_ARGUMENT
    );
    assert!(kernel.is_null());
}

#[test]
fn a_host_creates_a_world_through_the_library_and_it_is_saved_on_destroy() {
    let saves = tempfile::tempdir().expect("temp dir");
    let (mut host, code) = Host::create(&content_dir(), &saves.path().to_string_lossy());
    assert_eq!(code, TCE_OK, "{}", host.last_error());
    assert_eq!(host.status(), TCE_OK);

    // The first ordered frame is the Welcome; no Hello is needed.
    let welcome = host.poll().expect("a welcome");
    assert_eq!(header(&welcome).kind, FrameKind::Welcome);
    let welcome = payload(&welcome);
    let welcome = flatbuffers::root::<wire::Welcome>(&welcome).expect("decodes");
    let preset = welcome
        .presets()
        .and_then(|p| p.iter().find(|p| p.is_default()))
        .and_then(|p| p.id())
        .expect("a default preset")
        .to_owned();

    // The first snapshot is number 1; a buffer too small keeps it for the next call.
    let (first, number) = host.snapshot(0).expect("a first snapshot");
    assert_eq!(header(&first).kind, FrameKind::Snapshot);
    assert_eq!(number, 1);
    let mut len = 0;
    let mut tiny = [0u8; 8];
    // SAFETY: a live kernel and an 8-byte buffer.
    let code = unsafe {
        (host.api.copy_snapshot.expect("copy_snapshot"))(
            host.kernel,
            0,
            tiny.as_mut_ptr(),
            tiny.len(),
            &mut len,
            ptr::null_mut(),
        )
    };
    assert_eq!(code, TCE_BUFFER_TOO_SMALL);
    assert!(len > tiny.len());

    // Junk, and frames of kinds a host does not send, are refused at once.
    assert_eq!(host.submit(b"not a frame"), TCE_BAD_FRAME);
    assert!(
        host.last_error().contains("malformed"),
        "{}",
        host.last_error()
    );
    let (_, response) = host.request(FrameKind::Response, &[]);
    assert_eq!(host.submit(&response), TCE_BAD_FRAME);

    // A command is answered through `poll`, and its world arrives in the snapshots.
    let (correlation, command) = host.request(FrameKind::Command, &new_world_command(&preset));
    assert_eq!(host.submit(&command), TCE_OK);
    let mut passed = Vec::new();
    let reply = host.reply(correlation, &mut passed);
    assert_eq!(header(&reply).kind, FrameKind::Response);
    let started = Instant::now();
    let mut after = number;
    let (name, epoch) = loop {
        if let Some((frame, n)) = host.snapshot(after) {
            assert!(n > after);
            after = n;
            let bytes = payload(&frame);
            let snapshot = flatbuffers::root::<wire::Snapshot>(&bytes).expect("decodes");
            if let Some(world) = snapshot.world()
                && snapshot.task().is_none()
                && snapshot.people().is_some_and(|p| !p.is_empty())
            {
                break (world.name().unwrap_or("").to_owned(), header(&frame).epoch);
            }
        }
        while let Some(frame) = host.poll() {
            passed.push(frame);
        }
        assert!(started.elapsed() < TIMEOUT, "no world");
        std::thread::sleep(Duration::from_millis(20));
    };
    assert_eq!(name, "Library Valley");
    assert_eq!(epoch, 1, "a new world is a new epoch");
    while let Some(frame) = host.poll() {
        passed.push(frame);
    }
    assert!(
        passed.iter().any(|f| header(f).kind == FrameKind::Events),
        "events came through poll"
    );

    // Destroying the kernel saves the changed world.
    drop(host);
    let saved = std::fs::read_dir(saves.path())
        .expect("saves")
        .flatten()
        .filter(|e| e.path().is_dir())
        .flat_map(|d| std::fs::read_dir(d.path()).into_iter().flatten().flatten())
        .any(|f| f.path().extension().is_some_and(|x| x == "tcesave"));
    assert!(saved, "no save after destroy");
}

/// The status line of the answer to an HTTP request to the panel server.
fn http_status(port: u16, request: &str) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connects");
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("timeout");
    stream.write_all(request.as_bytes()).expect("writes");
    let mut response = Vec::new();
    let mut chunk = [0u8; 512];
    while !response.windows(2).any(|w| w == b"\r\n") {
        let n = stream.read(&mut chunk).expect("reads");
        if n == 0 {
            break;
        }
        response.extend_from_slice(&chunk[..n]);
    }
    let text = String::from_utf8_lossy(&response);
    text.lines().next().unwrap_or("").to_owned()
}

fn upgrade(path: &str) -> String {
    format!(
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: Upgrade\r\nUpgrade: websocket\r\n\
         Sec-WebSocket-Version: 13\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\r\n"
    )
}

#[test]
fn the_panel_server_needs_its_token() {
    let saves = tempfile::tempdir().expect("temp dir");
    let (host, code) = Host::create(&content_dir(), &saves.path().to_string_lossy());
    assert_eq!(code, TCE_OK, "{}", host.last_error());
    let panel_url = host.api.panel_url.expect("panel_url");
    let url_of = |host: &Host| {
        let mut buf = [0u8; 256];
        let mut len = 0;
        // SAFETY: a live kernel and a 256-byte buffer.
        let code = unsafe { panel_url(host.kernel, buf.as_mut_ptr(), buf.len(), &mut len) };
        (code, String::from_utf8_lossy(&buf[..len]).into_owned())
    };
    assert_eq!(url_of(&host).0, TCE_NONE);

    let serve = host.api.serve_panels.expect("serve_panels");
    let config = TcePanelConfig {
        struct_size: size_of::<TcePanelConfig>() as u32,
        port: 0,
        reserved: 0,
        web_dir: TceBytes {
            ptr: ptr::null(),
            len: 0,
        },
    };
    // SAFETY: a live kernel and a valid config.
    assert_eq!(
        unsafe { serve(host.kernel, &config) },
        TCE_OK,
        "{}",
        host.last_error()
    );
    // SAFETY: as above; serving again changes nothing.
    assert_eq!(unsafe { serve(host.kernel, &config) }, TCE_OK);

    let (code, url) = url_of(&host);
    assert_eq!(code, TCE_OK);
    let rest = url
        .strip_prefix("http://127.0.0.1:")
        .expect("a local address");
    let (port, token) = rest.split_once("/?token=").expect("a token");
    let port: u16 = port.parse().expect("a port");
    assert_eq!(token.len(), 32, "128 bits as hex");

    let healthz = "GET /healthz HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n";
    assert!(http_status(port, healthz).contains("200"));
    assert!(http_status(port, &upgrade("/ws")).contains("403"));
    assert!(http_status(port, &upgrade("/ws?token=0000")).contains("403"));
    let with_token = upgrade(&format!("/ws?token={token}"));
    assert!(http_status(port, &with_token).contains("101"));
}
