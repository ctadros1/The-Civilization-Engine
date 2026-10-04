//! The frame envelope shared by every engine's kernel boundary.
//!
//! One frame is a fixed 56-byte little-endian header followed by an opaque payload. The payload's
//! meaning belongs to the engine schema named in the header; this crate never looks inside it.
//! ADR-0001 in The Civilization Engine documents the layout and the delivery contract of each
//! frame kind.
//!
//! Decoding validates every header field, including the payload length against [`Limits`],
//! before the caller allocates anything sized by the frame. A malformed frame is an error, never a
//! partial result.

#![forbid(unsafe_code)]

use std::fmt;

/// Magic bytes at the start of every frame.
pub const MAGIC: [u8; 4] = *b"CWIR";
/// Layout version of the envelope itself. Independent of any engine schema version.
pub const ENVELOPE_VERSION: u8 = 1;
/// Size of the fixed header in bytes. The payload starts at this offset.
pub const HEADER_LEN: usize = 56;
/// Flags bit 0: the header carries a CRC-32 (IEEE) of the payload.
pub const FLAG_PAYLOAD_CRC32: u16 = 0x0001;
const KNOWN_FLAGS: u16 = FLAG_PAYLOAD_CRC32;
/// Default upper bound on a payload, checked before any allocation sized by the frame.
pub const DEFAULT_MAX_PAYLOAD_LEN: u32 = 64 * 1024 * 1024;

/// What a frame is for. The numeric values are part of the wire format: never renumber them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum FrameKind {
    /// Client greeting: names the schema it speaks.
    Hello = 1,
    /// Host greeting: names the schema it speaks and whether the handshake succeeded.
    Welcome = 2,
    /// Complete, independently usable state. Replaceable.
    Snapshot = 3,
    /// Incremental change relative to earlier frames of the same epoch. Ordered.
    Delta = 4,
    /// Things that happened. Ordered.
    Events = 5,
    /// A request that changes state.
    Command = 6,
    /// A request that only reads state.
    Query = 7,
    /// The answer to a command or query.
    Response = 8,
    /// A failed command or query, or (with correlation id 0) an unsolicited host error.
    Error = 9,
    /// Liveness only; carries no meaning.
    Heartbeat = 10,
}

impl FrameKind {
    /// Every kind, in numeric order.
    pub const ALL: [FrameKind; 10] = [
        FrameKind::Hello,
        FrameKind::Welcome,
        FrameKind::Snapshot,
        FrameKind::Delta,
        FrameKind::Events,
        FrameKind::Command,
        FrameKind::Query,
        FrameKind::Response,
        FrameKind::Error,
        FrameKind::Heartbeat,
    ];

    /// Parses the wire value of a kind.
    pub const fn from_u8(value: u8) -> Option<Self> {
        Some(match value {
            1 => FrameKind::Hello,
            2 => FrameKind::Welcome,
            3 => FrameKind::Snapshot,
            4 => FrameKind::Delta,
            5 => FrameKind::Events,
            6 => FrameKind::Command,
            7 => FrameKind::Query,
            8 => FrameKind::Response,
            9 => FrameKind::Error,
            10 => FrameKind::Heartbeat,
            _ => return None,
        })
    }

    /// The wire value of this kind.
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    /// A stable lowercase name, for logs and diagnostics.
    pub const fn name(self) -> &'static str {
        match self {
            FrameKind::Hello => "hello",
            FrameKind::Welcome => "welcome",
            FrameKind::Snapshot => "snapshot",
            FrameKind::Delta => "delta",
            FrameKind::Events => "events",
            FrameKind::Command => "command",
            FrameKind::Query => "query",
            FrameKind::Response => "response",
            FrameKind::Error => "error",
            FrameKind::Heartbeat => "heartbeat",
        }
    }

    /// The delivery contract a transport and a consumer must honour for this kind.
    pub const fn delivery(self) -> Delivery {
        match self {
            FrameKind::Snapshot => Delivery::Replaceable,
            FrameKind::Delta | FrameKind::Events => Delivery::Ordered,
            FrameKind::Command | FrameKind::Query => Delivery::Request,
            FrameKind::Response | FrameKind::Error => Delivery::Reply,
            FrameKind::Hello | FrameKind::Welcome | FrameKind::Heartbeat => Delivery::Control,
        }
    }

    /// Whether a frame of this kind must carry a non-zero correlation id.
    pub const fn requires_correlation(self) -> bool {
        matches!(
            self,
            FrameKind::Command | FrameKind::Query | FrameKind::Response
        )
    }
}

impl fmt::Display for FrameKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// How frames of a kind may be delivered, dropped or reordered.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Delivery {
    /// A newer frame fully replaces an older one; a consumer may drop any but the newest.
    Replaceable,
    /// Frames apply in sequence order within an epoch; a gap means the consumer lost state and
    /// must recover from a snapshot.
    Ordered,
    /// A request; the reply echoes its correlation id.
    Request,
    /// A reply to a request, or an unsolicited message when the correlation id is zero.
    Reply,
    /// Connection management.
    Control,
}

/// The engine schema a payload is written in: a four-byte tag plus a semantic version.
///
/// Peers can exchange frames when the tag and the major version match. A minor difference means
/// additive changes only, which FlatBuffers-style schemas tolerate in both directions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SchemaId {
    /// Engine-chosen tag, for example `*b"TCE\0"`.
    pub tag: [u8; 4],
    /// Incremented for incompatible changes.
    pub major: u16,
    /// Incremented for additive changes.
    pub minor: u16,
}

impl SchemaId {
    /// Builds a schema id.
    pub const fn new(tag: [u8; 4], major: u16, minor: u16) -> Self {
        SchemaId { tag, major, minor }
    }

    /// Whether a peer speaking `other` can exchange frames with a peer speaking `self`.
    pub fn is_compatible_with(&self, other: &SchemaId) -> bool {
        self.tag == other.tag && self.major == other.major
    }

    /// The tag as text, with trailing NULs removed and anything unprintable escaped.
    pub fn tag_text(&self) -> String {
        let end = self.tag.iter().rposition(|&b| b != 0).map_or(0, |i| i + 1);
        self.tag[..end]
            .iter()
            .flat_map(|&b| std::ascii::escape_default(b))
            .map(char::from)
            .collect()
    }
}

impl fmt::Display for SchemaId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}.{}", self.tag_text(), self.major, self.minor)
    }
}

/// The caller-chosen fields of a frame header. Length, flags and checksum are derived on encode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameMeta {
    /// What the frame is for.
    pub kind: FrameKind,
    /// The schema the payload is written in.
    pub schema: SchemaId,
    /// Increments whenever the authoritative world is replaced; older epochs are discarded.
    pub epoch: u32,
    /// Monotonic per (epoch, kind) on one stream.
    pub sequence: u64,
    /// Request id for commands, queries and their replies; zero otherwise.
    pub correlation: u64,
    /// Simulation time the frame refers to, in an engine-defined unit; zero when meaningless.
    pub sim_time: i64,
}

/// A fully decoded header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameHeader {
    /// The caller-chosen fields.
    pub meta: FrameMeta,
    /// Flag bits; only [`FLAG_PAYLOAD_CRC32`] is defined.
    pub flags: u16,
    /// Length of the payload that follows the header.
    pub payload_len: u32,
    /// CRC-32 of the payload when [`FLAG_PAYLOAD_CRC32`] is set, zero otherwise.
    pub payload_crc32: u32,
}

impl FrameHeader {
    /// Whether the header carries a payload checksum.
    pub fn has_crc(&self) -> bool {
        self.flags & FLAG_PAYLOAD_CRC32 != 0
    }

    /// Total encoded length of the frame: header plus payload.
    pub fn frame_len(&self) -> usize {
        HEADER_LEN + self.payload_len as usize
    }
}

/// Bounds enforced by the decoder before anything sized by the frame is trusted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// Largest payload accepted, in bytes.
    pub max_payload_len: u32,
}

impl Default for Limits {
    fn default() -> Self {
        Limits {
            max_payload_len: DEFAULT_MAX_PAYLOAD_LEN,
        }
    }
}

/// Why bytes could not be encoded or decoded as a frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WireError {
    /// Fewer bytes than the structure requires.
    Truncated {
        /// Bytes required.
        needed: usize,
        /// Bytes present.
        available: usize,
    },
    /// The first four bytes are not [`MAGIC`].
    BadMagic {
        /// The bytes found instead.
        found: [u8; 4],
    },
    /// An envelope layout this decoder does not know.
    UnsupportedEnvelopeVersion {
        /// Version found.
        found: u8,
        /// Version this decoder reads.
        supported: u8,
    },
    /// A frame kind outside [`FrameKind::ALL`].
    UnknownKind {
        /// Wire value found.
        found: u8,
    },
    /// Flag bits this decoder does not know.
    UnknownFlags {
        /// Flags found.
        found: u16,
    },
    /// The reserved header word is not zero.
    ReservedNotZero {
        /// Value found.
        found: u32,
    },
    /// A checksum value is present although the checksum flag is clear.
    ChecksumWithoutFlag {
        /// Value found.
        found: u32,
    },
    /// The payload is larger than the configured limit (or than the format can express).
    PayloadTooLarge {
        /// Declared or supplied length.
        len: u64,
        /// Largest length allowed.
        max: u64,
    },
    /// The bytes present do not match the declared payload length.
    LengthMismatch {
        /// Total frame length the header declares.
        declared: usize,
        /// Bytes present.
        available: usize,
    },
    /// The payload does not match its checksum.
    ChecksumMismatch {
        /// Checksum stored in the header.
        expected: u32,
        /// Checksum of the payload received.
        computed: u32,
    },
    /// A command, query or response with correlation id zero.
    MissingCorrelation {
        /// The frame kind.
        kind: FrameKind,
    },
}

impl fmt::Display for WireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WireError::Truncated { needed, available } => {
                write!(
                    f,
                    "frame truncated: needed {needed} bytes, {available} available"
                )
            }
            WireError::BadMagic { found } => write!(f, "not a frame: magic {found:02x?}"),
            WireError::UnsupportedEnvelopeVersion { found, supported } => {
                write!(
                    f,
                    "envelope version {found}, this decoder reads {supported}"
                )
            }
            WireError::UnknownKind { found } => write!(f, "unknown frame kind {found}"),
            WireError::UnknownFlags { found } => write!(f, "unknown frame flags {found:#06x}"),
            WireError::ReservedNotZero { found } => {
                write!(f, "reserved header word is {found:#010x}, expected zero")
            }
            WireError::ChecksumWithoutFlag { found } => {
                write!(
                    f,
                    "checksum {found:#010x} present but the checksum flag is clear"
                )
            }
            WireError::PayloadTooLarge { len, max } => {
                write!(f, "payload of {len} bytes exceeds the limit of {max}")
            }
            WireError::LengthMismatch {
                declared,
                available,
            } => write!(
                f,
                "frame declares {declared} bytes but {available} are present"
            ),
            WireError::ChecksumMismatch { expected, computed } => write!(
                f,
                "payload checksum mismatch: header says {expected:#010x}, payload is {computed:#010x}"
            ),
            WireError::MissingCorrelation { kind } => {
                write!(f, "{kind} frame without a correlation id")
            }
        }
    }
}

impl std::error::Error for WireError {}

/// A decoded frame borrowing its payload from the input bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Frame<'a> {
    /// The decoded header.
    pub header: FrameHeader,
    /// The payload, exactly `header.payload_len` bytes.
    pub payload: &'a [u8],
}

/// Encodes one frame into a new buffer.
pub fn encode(meta: &FrameMeta, payload: &[u8], with_crc: bool) -> Result<Vec<u8>, WireError> {
    let mut out = Vec::with_capacity(HEADER_LEN + payload.len());
    encode_into(meta, payload, with_crc, &mut out)?;
    Ok(out)
}

/// Appends one encoded frame to `out`.
pub fn encode_into(
    meta: &FrameMeta,
    payload: &[u8],
    with_crc: bool,
    out: &mut Vec<u8>,
) -> Result<(), WireError> {
    let payload_len = u32::try_from(payload.len()).map_err(|_| WireError::PayloadTooLarge {
        len: payload.len() as u64,
        max: u64::from(u32::MAX),
    })?;
    if meta.kind.requires_correlation() && meta.correlation == 0 {
        return Err(WireError::MissingCorrelation { kind: meta.kind });
    }
    let (flags, crc) = if with_crc {
        (FLAG_PAYLOAD_CRC32, crc32fast::hash(payload))
    } else {
        (0, 0)
    };
    out.reserve(HEADER_LEN + payload.len());
    out.extend_from_slice(&MAGIC);
    out.push(ENVELOPE_VERSION);
    out.push(meta.kind.as_u8());
    out.extend_from_slice(&flags.to_le_bytes());
    out.extend_from_slice(&meta.schema.tag);
    out.extend_from_slice(&meta.schema.major.to_le_bytes());
    out.extend_from_slice(&meta.schema.minor.to_le_bytes());
    out.extend_from_slice(&meta.epoch.to_le_bytes());
    out.extend_from_slice(&payload_len.to_le_bytes());
    out.extend_from_slice(&meta.sequence.to_le_bytes());
    out.extend_from_slice(&meta.correlation.to_le_bytes());
    out.extend_from_slice(&meta.sim_time.to_le_bytes());
    out.extend_from_slice(&crc.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(payload);
    Ok(())
}

fn read_u16(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}

fn read_u32(bytes: &[u8], at: usize) -> u32 {
    let mut b = [0u8; 4];
    b.copy_from_slice(&bytes[at..at + 4]);
    u32::from_le_bytes(b)
}

fn read_u64(bytes: &[u8], at: usize) -> u64 {
    let mut b = [0u8; 8];
    b.copy_from_slice(&bytes[at..at + 8]);
    u64::from_le_bytes(b)
}

/// Decodes and validates a header from the first [`HEADER_LEN`] bytes of `bytes`.
///
/// This checks everything the header can check on its own, including the payload length against
/// `limits`, but not the payload itself.
pub fn decode_header(bytes: &[u8], limits: &Limits) -> Result<FrameHeader, WireError> {
    if bytes.len() < HEADER_LEN {
        return Err(WireError::Truncated {
            needed: HEADER_LEN,
            available: bytes.len(),
        });
    }
    let magic = [bytes[0], bytes[1], bytes[2], bytes[3]];
    if magic != MAGIC {
        return Err(WireError::BadMagic { found: magic });
    }
    if bytes[4] != ENVELOPE_VERSION {
        return Err(WireError::UnsupportedEnvelopeVersion {
            found: bytes[4],
            supported: ENVELOPE_VERSION,
        });
    }
    let kind = FrameKind::from_u8(bytes[5]).ok_or(WireError::UnknownKind { found: bytes[5] })?;
    let flags = read_u16(bytes, 6);
    if flags & !KNOWN_FLAGS != 0 {
        return Err(WireError::UnknownFlags { found: flags });
    }
    let schema = SchemaId {
        tag: [bytes[8], bytes[9], bytes[10], bytes[11]],
        major: read_u16(bytes, 12),
        minor: read_u16(bytes, 14),
    };
    let epoch = read_u32(bytes, 16);
    let payload_len = read_u32(bytes, 20);
    let sequence = read_u64(bytes, 24);
    let correlation = read_u64(bytes, 32);
    let sim_time = read_u64(bytes, 40) as i64;
    let payload_crc32 = read_u32(bytes, 48);
    let reserved = read_u32(bytes, 52);
    if reserved != 0 {
        return Err(WireError::ReservedNotZero { found: reserved });
    }
    if flags & FLAG_PAYLOAD_CRC32 == 0 && payload_crc32 != 0 {
        return Err(WireError::ChecksumWithoutFlag {
            found: payload_crc32,
        });
    }
    if payload_len > limits.max_payload_len {
        return Err(WireError::PayloadTooLarge {
            len: u64::from(payload_len),
            max: u64::from(limits.max_payload_len),
        });
    }
    if kind.requires_correlation() && correlation == 0 {
        return Err(WireError::MissingCorrelation { kind });
    }
    Ok(FrameHeader {
        meta: FrameMeta {
            kind,
            schema,
            epoch,
            sequence,
            correlation,
            sim_time,
        },
        flags,
        payload_len,
        payload_crc32,
    })
}

/// Decodes exactly one frame occupying all of `bytes` (one WebSocket message, one FFI slot).
pub fn decode<'a>(bytes: &'a [u8], limits: &Limits) -> Result<Frame<'a>, WireError> {
    let (frame, rest) = split_first(bytes, limits)?;
    if !rest.is_empty() {
        return Err(WireError::LengthMismatch {
            declared: frame.header.frame_len(),
            available: bytes.len(),
        });
    }
    Ok(frame)
}

/// Decodes the first frame of a byte stream holding concatenated frames (a recording file) and
/// returns it with the remaining bytes.
pub fn split_first<'a>(
    bytes: &'a [u8],
    limits: &Limits,
) -> Result<(Frame<'a>, &'a [u8]), WireError> {
    let header = decode_header(bytes, limits)?;
    let frame_len = header.frame_len();
    if bytes.len() < frame_len {
        return Err(WireError::LengthMismatch {
            declared: frame_len,
            available: bytes.len(),
        });
    }
    let payload = &bytes[HEADER_LEN..frame_len];
    if header.has_crc() {
        let computed = crc32fast::hash(payload);
        if computed != header.payload_crc32 {
            return Err(WireError::ChecksumMismatch {
                expected: header.payload_crc32,
                computed,
            });
        }
    }
    Ok((Frame { header, payload }, &bytes[frame_len..]))
}

/// Issues sequence numbers per frame kind within the current epoch.
///
/// Starting a new epoch (a new or loaded world) resets every counter, so consumers can tell
/// frames of the new world from stragglers of the old one.
#[derive(Clone, Debug)]
pub struct Sequencer {
    epoch: u32,
    next: [u64; FrameKind::ALL.len() + 1],
}

impl Sequencer {
    /// A sequencer for `epoch`, with every counter at zero.
    pub fn new(epoch: u32) -> Self {
        Sequencer {
            epoch,
            next: [0; FrameKind::ALL.len() + 1],
        }
    }

    /// The current epoch.
    pub fn epoch(&self) -> u32 {
        self.epoch
    }

    /// Moves to the next epoch and resets every counter. Returns the new epoch.
    pub fn advance_epoch(&mut self) -> u32 {
        self.epoch = self.epoch.wrapping_add(1);
        self.next = [0; FrameKind::ALL.len() + 1];
        self.epoch
    }

    /// Returns the next sequence number for `kind` and advances its counter.
    pub fn next(&mut self, kind: FrameKind) -> u64 {
        let slot = &mut self.next[usize::from(kind.as_u8())];
        let value = *slot;
        *slot += 1;
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta(kind: FrameKind) -> FrameMeta {
        FrameMeta {
            kind,
            schema: SchemaId::new(*b"TCE\0", 1, 0),
            epoch: 3,
            sequence: 9,
            correlation: if kind.requires_correlation() { 77 } else { 0 },
            sim_time: -12,
        }
    }

    #[test]
    fn every_kind_round_trips_its_wire_value() {
        for kind in FrameKind::ALL {
            assert_eq!(FrameKind::from_u8(kind.as_u8()), Some(kind));
        }
        assert_eq!(FrameKind::from_u8(0), None);
        assert_eq!(FrameKind::from_u8(11), None);
    }

    #[test]
    fn encode_then_decode_preserves_every_field() {
        for kind in FrameKind::ALL {
            for with_crc in [false, true] {
                let bytes = encode(&meta(kind), b"payload", with_crc).expect("encodes");
                let frame = decode(&bytes, &Limits::default()).expect("decodes");
                assert_eq!(frame.header.meta, meta(kind));
                assert_eq!(frame.payload, b"payload");
                assert_eq!(frame.header.has_crc(), with_crc);
            }
        }
    }

    #[test]
    fn requests_without_correlation_are_refused_both_ways() {
        let mut m = meta(FrameKind::Command);
        m.correlation = 0;
        assert_eq!(
            encode(&m, b"", false),
            Err(WireError::MissingCorrelation {
                kind: FrameKind::Command
            })
        );
        let mut bytes = encode(&meta(FrameKind::Command), b"", false).expect("encodes");
        bytes[32..40].fill(0);
        assert_eq!(
            decode(&bytes, &Limits::default()),
            Err(WireError::MissingCorrelation {
                kind: FrameKind::Command
            })
        );
    }

    #[test]
    fn oversized_payloads_are_refused_before_allocation() {
        let bytes = encode(&meta(FrameKind::Snapshot), &[0u8; 100], false).expect("encodes");
        let limits = Limits {
            max_payload_len: 99,
        };
        assert_eq!(
            decode_header(&bytes, &limits),
            Err(WireError::PayloadTooLarge { len: 100, max: 99 })
        );
    }

    #[test]
    fn schema_compatibility_is_tag_and_major() {
        let a = SchemaId::new(*b"TCE\0", 1, 0);
        assert!(a.is_compatible_with(&SchemaId::new(*b"TCE\0", 1, 7)));
        assert!(!a.is_compatible_with(&SchemaId::new(*b"TCE\0", 2, 0)));
        assert!(!a.is_compatible_with(&SchemaId::new(*b"PRM\0", 1, 0)));
        assert_eq!(a.to_string(), "TCE 1.0");
    }

    #[test]
    fn sequencer_counts_per_kind_and_resets_on_epoch() {
        let mut s = Sequencer::new(5);
        assert_eq!(s.next(FrameKind::Snapshot), 0);
        assert_eq!(s.next(FrameKind::Snapshot), 1);
        assert_eq!(s.next(FrameKind::Events), 0);
        assert_eq!(s.advance_epoch(), 6);
        assert_eq!(s.next(FrameKind::Snapshot), 0);
    }
}
