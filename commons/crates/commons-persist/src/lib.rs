//! The chunked snapshot container shared by every engine's save system.
//!
//! A snapshot is one file holding a fixed header, independently compressed chunks grouped into
//! named sections, a manifest, and a completion footer. ADR-0002 in The Civilization Engine
//! documents the byte layout. The posture is Prometheus's: **refusal, never repair**. A file
//! that cannot be read exactly is not read approximately.
//!
//! The container knows nothing about what a section means. Engines choose section tags,
//! encode their payloads (TCE uses FlatBuffers), and decide which schema versions they accept.
//!
//! ```no_run
//! use commons_persist::{Compression, SaveDir, SaveKind, SectionData, SectionTag, SnapshotInfo};
//!
//! # fn main() -> Result<(), commons_persist::PersistError> {
//! const CLOCK: SectionTag = SectionTag::new("clock");
//! let dir = SaveDir::create("saves/my-world", "save")?;
//! let mut info = SnapshotInfo::new(*b"MYENGINE", 1, [7; 16]);
//! let sections = [SectionData::new(CLOCK, 0, 1, 42u64.to_le_bytes().to_vec(), Compression::Zstd)];
//! let published = dir.publish(SaveKind::Manual, &mut info, &sections)?;
//! println!("wrote {}", published.path.display());
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]

mod error;
mod format;
mod generations;
mod reader;
mod writer;

pub use error::PersistError;
pub use format::{
    CONTAINER_VERSION, ChunkInfo, Codec, FOOTER_LEN, FOOTER_MAGIC, HEADER_LEN, LABEL_LEN, Limits,
    MAGIC, MANIFEST_ENTRY_LEN, MANIFEST_PREFIX_LEN, SectionTag, SnapshotInfo,
};
pub use generations::{GenerationFile, SaveDir, SaveKind};
pub use reader::{SectionChunk, SnapshotReader};
pub use writer::{
    Compression, DEFAULT_ZSTD_LEVEL, Finished, Published, SectionData, SnapshotWriter, publish_file,
};

/// A fresh random 16-byte identifier, for world and snapshot ids.
pub fn random_id() -> Result<[u8; 16], PersistError> {
    let mut id = [0u8; 16];
    getrandom::fill(&mut id).map_err(|e| PersistError::Random(e.to_string()))?;
    Ok(id)
}

/// Milliseconds since the Unix epoch according to the system clock (0 if the clock is before it).
pub fn now_unix_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
}

/// Lowercase hex of an identifier or fingerprint, for logs and file names.
pub fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(out, "{b:02x}");
    }
    out
}
