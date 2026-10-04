//! Byte layout of the container (ADR-0002). All integers are little-endian.
//!
//! ```text
//! header   192 bytes   magic, versions, identity, fingerprint, label, header digest
//! chunks   variable    stored bytes of each chunk, back to back
//! manifest 8 + 64·n    chunk count, then one fixed entry per chunk
//! footer   48 bytes    manifest location, file length, manifest and body digests, end magic
//! ```

use std::fmt;

use xxhash_rust::xxh3::xxh3_64;

use crate::error::PersistError;

/// Magic bytes at the start of every snapshot.
pub const MAGIC: [u8; 8] = *b"COMPSNAP";
/// Magic bytes at the very end of a complete snapshot.
pub const FOOTER_MAGIC: [u8; 8] = *b"COMPEND1";
/// Version of the container layout this build reads and writes.
pub const CONTAINER_VERSION: u16 = 1;
/// Size of the fixed header.
pub const HEADER_LEN: usize = 192;
/// Size of the manifest's count prefix.
pub const MANIFEST_PREFIX_LEN: usize = 8;
/// Size of one manifest entry.
pub const MANIFEST_ENTRY_LEN: usize = 64;
/// Size of the fixed footer.
pub const FOOTER_LEN: usize = 48;
/// Largest label, in UTF-8 bytes.
pub const LABEL_LEN: usize = 48;

const HEADER_DIGEST_AT: usize = HEADER_LEN - 8;

/// Bounds the reader enforces before trusting any size declared by a file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// Largest number of chunks in one snapshot.
    pub max_chunks: u32,
    /// Largest decoded size of one chunk.
    pub max_chunk_raw_len: u64,
    /// Largest stored (possibly compressed) size of one chunk.
    pub max_chunk_stored_len: u64,
}

impl Default for Limits {
    fn default() -> Self {
        Limits {
            max_chunks: 1 << 20,
            max_chunk_raw_len: 1 << 30,
            max_chunk_stored_len: 1 << 30,
        }
    }
}

/// A section name: 1–8 bytes of lowercase ASCII letters, digits, `_` or `-`, zero-padded.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SectionTag([u8; 8]);

const fn tag_byte_ok(b: u8) -> bool {
    b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-'
}

impl SectionTag {
    /// A tag from a literal. Panics (at compile time when used in a `const`) on an invalid name.
    pub const fn new(name: &str) -> SectionTag {
        let bytes = name.as_bytes();
        assert!(
            !bytes.is_empty() && bytes.len() <= 8,
            "section tags are 1-8 bytes"
        );
        let mut out = [0u8; 8];
        let mut i = 0;
        while i < bytes.len() {
            assert!(tag_byte_ok(bytes[i]), "section tags use [a-z0-9_-]");
            out[i] = bytes[i];
            i += 1;
        }
        SectionTag(out)
    }

    /// A tag from runtime text.
    pub fn parse(name: &str) -> Result<SectionTag, PersistError> {
        let bytes = name.as_bytes();
        if bytes.is_empty() || bytes.len() > 8 || !bytes.iter().all(|&b| tag_byte_ok(b)) {
            return Err(PersistError::InvalidSectionTag {
                raw: bytes.to_vec(),
            });
        }
        let mut out = [0u8; 8];
        out[..bytes.len()].copy_from_slice(bytes);
        Ok(SectionTag(out))
    }

    /// A tag from its stored form, validating the padding.
    pub fn from_bytes(raw: [u8; 8]) -> Result<SectionTag, PersistError> {
        let len = raw.iter().position(|&b| b == 0).unwrap_or(8);
        let valid = len > 0
            && raw[..len].iter().all(|&b| tag_byte_ok(b))
            && raw[len..].iter().all(|&b| b == 0);
        if valid {
            Ok(SectionTag(raw))
        } else {
            Err(PersistError::InvalidSectionTag { raw: raw.to_vec() })
        }
    }

    /// The stored, zero-padded form.
    pub fn to_bytes(self) -> [u8; 8] {
        self.0
    }

    /// The name as text.
    pub fn as_str(&self) -> &str {
        let len = self.0.iter().position(|&b| b == 0).unwrap_or(8);
        // Construction guarantees ASCII, so this cannot fail.
        std::str::from_utf8(&self.0[..len]).unwrap_or("?")
    }
}

impl fmt::Debug for SectionTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SectionTag({})", self.as_str())
    }
}

impl fmt::Display for SectionTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// How a chunk's bytes are stored.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Codec {
    /// Stored as-is.
    Raw,
    /// Stored as one zstd frame.
    Zstd,
}

impl Codec {
    pub(crate) fn to_u8(self) -> u8 {
        match self {
            Codec::Raw => 0,
            Codec::Zstd => 1,
        }
    }

    pub(crate) fn from_u8(value: u8) -> Option<Codec> {
        match value {
            0 => Some(Codec::Raw),
            1 => Some(Codec::Zstd),
            _ => None,
        }
    }
}

/// Identity and provenance of a snapshot: everything in the header except the layout fields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapshotInfo {
    /// Which engine wrote the file, for example `*b"TCE\0\0\0\0\0"`.
    pub engine_tag: [u8; 8],
    /// The engine's world-state schema version.
    pub schema_version: u32,
    /// The world this snapshot belongs to; stable across all its generations.
    pub world_id: [u8; 16],
    /// This snapshot's own identity.
    pub snapshot_id: [u8; 16],
    /// The snapshot this one continues from, if any (the save it was loaded from).
    pub parent_id: Option<[u8; 16]>,
    /// Position in the world's save directory; assigned when published.
    pub generation: u64,
    /// Wall-clock creation time, milliseconds since the Unix epoch.
    pub created_unix_ms: i64,
    /// Simulation time of the state, in the engine's unit.
    pub sim_time: i64,
    /// Fingerprint of the authored content the state was produced with.
    pub content_fingerprint: [u8; 32],
    /// Short human label, at most [`LABEL_LEN`] UTF-8 bytes without NUL.
    pub label: String,
}

impl SnapshotInfo {
    /// Info with the given identity and everything else zero or empty.
    pub fn new(engine_tag: [u8; 8], schema_version: u32, world_id: [u8; 16]) -> Self {
        SnapshotInfo {
            engine_tag,
            schema_version,
            world_id,
            snapshot_id: [0; 16],
            parent_id: None,
            generation: 0,
            created_unix_ms: 0,
            sim_time: 0,
            content_fingerprint: [0; 32],
            label: String::new(),
        }
    }

    /// Fails unless the snapshot was written by `engine_tag`.
    pub fn expect_engine(&self, engine_tag: [u8; 8]) -> Result<(), PersistError> {
        if self.engine_tag == engine_tag {
            Ok(())
        } else {
            Err(PersistError::WrongEngine {
                found: self.engine_tag,
                expected: engine_tag,
            })
        }
    }
}

/// One manifest entry: where a chunk is and how to check it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChunkInfo {
    /// Section the chunk belongs to.
    pub tag: SectionTag,
    /// Position of the chunk within its section, contiguous from zero.
    pub index: u32,
    /// Engine-defined version of the section's encoding.
    pub version: u32,
    /// How the bytes are stored.
    pub codec: Codec,
    /// Offset of the stored bytes from the start of the file.
    pub offset: u64,
    /// Length of the stored bytes.
    pub stored_len: u64,
    /// Length after decoding.
    pub raw_len: u64,
    /// xxh3-64 of the stored bytes.
    pub stored_digest: u64,
    /// xxh3-64 of the decoded bytes.
    pub raw_digest: u64,
}

pub(crate) struct Footer {
    pub manifest_offset: u64,
    pub manifest_len: u64,
    pub file_len: u64,
    pub manifest_digest: u64,
    pub body_digest: u64,
}

fn put_u16(out: &mut [u8], at: usize, v: u16) {
    out[at..at + 2].copy_from_slice(&v.to_le_bytes());
}

fn put_u32(out: &mut [u8], at: usize, v: u32) {
    out[at..at + 4].copy_from_slice(&v.to_le_bytes());
}

fn put_u64(out: &mut [u8], at: usize, v: u64) {
    out[at..at + 8].copy_from_slice(&v.to_le_bytes());
}

pub(crate) fn get_u16(b: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([b[at], b[at + 1]])
}

pub(crate) fn get_u32(b: &[u8], at: usize) -> u32 {
    let mut x = [0u8; 4];
    x.copy_from_slice(&b[at..at + 4]);
    u32::from_le_bytes(x)
}

pub(crate) fn get_u64(b: &[u8], at: usize) -> u64 {
    let mut x = [0u8; 8];
    x.copy_from_slice(&b[at..at + 8]);
    u64::from_le_bytes(x)
}

fn get_array<const N: usize>(b: &[u8], at: usize) -> [u8; N] {
    let mut x = [0u8; N];
    x.copy_from_slice(&b[at..at + N]);
    x
}

pub(crate) fn encode_header(info: &SnapshotInfo) -> Result<[u8; HEADER_LEN], PersistError> {
    let label = info.label.as_bytes();
    if label.len() > LABEL_LEN {
        return Err(PersistError::InvalidLabel {
            reason: "longer than 48 bytes",
        });
    }
    if label.contains(&0) {
        return Err(PersistError::InvalidLabel {
            reason: "contains NUL",
        });
    }
    let mut h = [0u8; HEADER_LEN];
    h[0..8].copy_from_slice(&MAGIC);
    put_u16(&mut h, 8, CONTAINER_VERSION);
    put_u16(&mut h, 10, HEADER_LEN as u16);
    put_u32(&mut h, 12, 0); // flags, none defined
    h[16..24].copy_from_slice(&info.engine_tag);
    put_u32(&mut h, 24, info.schema_version);
    put_u32(&mut h, 28, 0); // reserved
    h[32..48].copy_from_slice(&info.world_id);
    h[48..64].copy_from_slice(&info.snapshot_id);
    h[64..80].copy_from_slice(&info.parent_id.unwrap_or([0; 16]));
    put_u64(&mut h, 80, info.generation);
    put_u64(&mut h, 88, info.created_unix_ms as u64);
    put_u64(&mut h, 96, info.sim_time as u64);
    h[104..136].copy_from_slice(&info.content_fingerprint);
    h[136..136 + label.len()].copy_from_slice(label);
    let digest = xxh3_64(&h[..HEADER_DIGEST_AT]);
    put_u64(&mut h, HEADER_DIGEST_AT, digest);
    Ok(h)
}

pub(crate) fn decode_header(h: &[u8]) -> Result<SnapshotInfo, PersistError> {
    debug_assert_eq!(h.len(), HEADER_LEN);
    if h[0..8] != MAGIC {
        return Err(PersistError::NotASnapshot);
    }
    let version = get_u16(h, 8);
    if version != CONTAINER_VERSION {
        return Err(PersistError::UnsupportedContainerVersion {
            found: version,
            supported: CONTAINER_VERSION,
        });
    }
    if usize::from(get_u16(h, 10)) != HEADER_LEN {
        return Err(PersistError::HeaderCorrupt {
            reason: "declared header length is wrong",
        });
    }
    if xxh3_64(&h[..HEADER_DIGEST_AT]) != get_u64(h, HEADER_DIGEST_AT) {
        return Err(PersistError::HeaderCorrupt {
            reason: "header digest mismatch",
        });
    }
    if get_u32(h, 12) != 0 || get_u32(h, 28) != 0 {
        return Err(PersistError::HeaderCorrupt {
            reason: "reserved header fields are not zero",
        });
    }
    let label_bytes = &h[136..136 + LABEL_LEN];
    let label_len = label_bytes
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(LABEL_LEN);
    if label_bytes[label_len..].iter().any(|&b| b != 0) {
        return Err(PersistError::HeaderCorrupt {
            reason: "label padding is not zero",
        });
    }
    let label = std::str::from_utf8(&label_bytes[..label_len])
        .map_err(|_| PersistError::HeaderCorrupt {
            reason: "label is not UTF-8",
        })?
        .to_owned();
    let parent: [u8; 16] = get_array(h, 64);
    Ok(SnapshotInfo {
        engine_tag: get_array(h, 16),
        schema_version: get_u32(h, 24),
        world_id: get_array(h, 32),
        snapshot_id: get_array(h, 48),
        parent_id: if parent == [0; 16] {
            None
        } else {
            Some(parent)
        },
        generation: get_u64(h, 80),
        created_unix_ms: get_u64(h, 88) as i64,
        sim_time: get_u64(h, 96) as i64,
        content_fingerprint: get_array(h, 104),
        label,
    })
}

pub(crate) fn encode_manifest(chunks: &[ChunkInfo]) -> Vec<u8> {
    let mut out = vec![0u8; MANIFEST_PREFIX_LEN + MANIFEST_ENTRY_LEN * chunks.len()];
    put_u32(&mut out, 0, chunks.len() as u32);
    for (i, c) in chunks.iter().enumerate() {
        let at = MANIFEST_PREFIX_LEN + i * MANIFEST_ENTRY_LEN;
        out[at..at + 8].copy_from_slice(&c.tag.to_bytes());
        put_u32(&mut out, at + 8, c.index);
        put_u32(&mut out, at + 12, c.version);
        out[at + 16] = c.codec.to_u8();
        put_u64(&mut out, at + 24, c.offset);
        put_u64(&mut out, at + 32, c.stored_len);
        put_u64(&mut out, at + 40, c.raw_len);
        put_u64(&mut out, at + 48, c.stored_digest);
        put_u64(&mut out, at + 56, c.raw_digest);
    }
    out
}

pub(crate) fn decode_manifest(m: &[u8]) -> Result<Vec<ChunkInfo>, PersistError> {
    let corrupt = |reason: &str| PersistError::ManifestCorrupt {
        reason: reason.to_owned(),
    };
    if m.len() < MANIFEST_PREFIX_LEN {
        return Err(corrupt("shorter than its prefix"));
    }
    let count = get_u32(m, 0) as usize;
    if get_u32(m, 4) != 0 {
        return Err(corrupt("reserved word is not zero"));
    }
    if m.len() != MANIFEST_PREFIX_LEN + count * MANIFEST_ENTRY_LEN {
        return Err(corrupt("length disagrees with chunk count"));
    }
    let mut chunks = Vec::with_capacity(count);
    for i in 0..count {
        let at = MANIFEST_PREFIX_LEN + i * MANIFEST_ENTRY_LEN;
        let tag = SectionTag::from_bytes(get_array(m, at))?;
        let index = get_u32(m, at + 8);
        let codec_byte = m[at + 16];
        let codec = Codec::from_u8(codec_byte).ok_or(PersistError::UnknownCodec {
            tag,
            index,
            value: codec_byte,
        })?;
        if m[at + 17..at + 24].iter().any(|&b| b != 0) {
            return Err(corrupt("entry reserved bytes are not zero"));
        }
        chunks.push(ChunkInfo {
            tag,
            index,
            version: get_u32(m, at + 12),
            codec,
            offset: get_u64(m, at + 24),
            stored_len: get_u64(m, at + 32),
            raw_len: get_u64(m, at + 40),
            stored_digest: get_u64(m, at + 48),
            raw_digest: get_u64(m, at + 56),
        });
    }
    Ok(chunks)
}

pub(crate) fn encode_footer(f: &Footer) -> [u8; FOOTER_LEN] {
    let mut out = [0u8; FOOTER_LEN];
    put_u64(&mut out, 0, f.manifest_offset);
    put_u64(&mut out, 8, f.manifest_len);
    put_u64(&mut out, 16, f.file_len);
    put_u64(&mut out, 24, f.manifest_digest);
    put_u64(&mut out, 32, f.body_digest);
    out[40..48].copy_from_slice(&FOOTER_MAGIC);
    out
}

pub(crate) fn decode_footer(b: &[u8]) -> Result<Footer, PersistError> {
    debug_assert_eq!(b.len(), FOOTER_LEN);
    if b[40..48] != FOOTER_MAGIC {
        return Err(PersistError::Incomplete {
            reason: "no completion footer",
        });
    }
    Ok(Footer {
        manifest_offset: get_u64(b, 0),
        manifest_len: get_u64(b, 8),
        file_len: get_u64(b, 16),
        manifest_digest: get_u64(b, 24),
        body_digest: get_u64(b, 32),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn section_tags_validate() {
        assert_eq!(SectionTag::new("terrain").as_str(), "terrain");
        assert_eq!(
            SectionTag::parse("a_b-9").map(|t| t.to_bytes()).ok(),
            Some(*b"a_b-9\0\0\0")
        );
        assert!(SectionTag::parse("").is_err());
        assert!(SectionTag::parse("Upper").is_err());
        assert!(SectionTag::parse("ninechars").is_err());
        assert!(SectionTag::from_bytes(*b"ab\0c\0\0\0\0").is_err());
        assert!(SectionTag::from_bytes([0; 8]).is_err());
    }

    #[test]
    fn header_round_trips_and_detects_tampering() {
        let mut info = SnapshotInfo::new(*b"TESTENG\0", 3, [1; 16]);
        info.snapshot_id = [2; 16];
        info.parent_id = Some([3; 16]);
        info.generation = 12;
        info.created_unix_ms = -1;
        info.sim_time = 99;
        info.content_fingerprint = [4; 32];
        info.label = "before the flood".into();
        let h = encode_header(&info).expect("encodes");
        assert_eq!(decode_header(&h).expect("decodes"), info);

        let mut bad = h;
        bad[140] ^= 1;
        assert!(matches!(
            decode_header(&bad),
            Err(PersistError::HeaderCorrupt { .. })
        ));
    }

    #[test]
    fn labels_are_bounded() {
        let mut info = SnapshotInfo::new(*b"TESTENG\0", 1, [0; 16]);
        info.label = "x".repeat(49);
        assert!(encode_header(&info).is_err());
        info.label = "a\0b".into();
        assert!(encode_header(&info).is_err());
        info.label = "é".repeat(24); // 48 bytes exactly
        assert!(encode_header(&info).is_ok());
    }
}
