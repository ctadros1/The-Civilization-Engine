use std::collections::HashSet;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use xxhash_rust::xxh3::{Xxh3, xxh3_64};

use crate::error::PersistError;
use crate::format::{
    ChunkInfo, Codec, FOOTER_LEN, Footer, HEADER_LEN, SectionTag, SnapshotInfo, encode_footer,
    encode_header, encode_manifest,
};

/// zstd level used unless the caller chooses another. Level 1 favours speed (ADR-0002).
pub const DEFAULT_ZSTD_LEVEL: i32 = 1;

/// How the writer should try to store a chunk.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Compression {
    /// Store the bytes as-is.
    Raw,
    /// Compress with zstd; falls back to raw when compression does not shrink the chunk. The
    /// codec actually used is recorded per chunk.
    Zstd,
}

/// One chunk's worth of a section, ready to be written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectionData {
    /// Section the chunk belongs to.
    pub tag: SectionTag,
    /// Position within the section, contiguous from zero.
    pub index: u32,
    /// Engine-defined version of the section's encoding.
    pub version: u32,
    /// The decoded bytes.
    pub bytes: Vec<u8>,
    /// How to store them.
    pub compression: Compression,
}

impl SectionData {
    /// Bundles one chunk.
    pub fn new(
        tag: SectionTag,
        index: u32,
        version: u32,
        bytes: Vec<u8>,
        compression: Compression,
    ) -> Self {
        SectionData {
            tag,
            index,
            version,
            bytes,
            compression,
        }
    }
}

/// Streams a snapshot into any writer: header first, chunks as they are added, then manifest and
/// footer on [`SnapshotWriter::finish`].
pub struct SnapshotWriter<W: Write> {
    out: W,
    hasher: Xxh3,
    pos: u64,
    chunks: Vec<ChunkInfo>,
    seen: HashSet<(SectionTag, u32)>,
    zstd_level: i32,
}

impl<W: Write> std::fmt::Debug for SnapshotWriter<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SnapshotWriter")
            .field("pos", &self.pos)
            .field("chunks", &self.chunks.len())
            .field("zstd_level", &self.zstd_level)
            .finish_non_exhaustive()
    }
}

/// The result of finishing a snapshot.
#[derive(Debug)]
pub struct Finished<W> {
    /// The underlying writer, flushed.
    pub out: W,
    /// Total bytes written.
    pub file_len: u64,
    /// Digest of everything before the footer.
    pub body_digest: u64,
    /// The manifest as written.
    pub chunks: Vec<ChunkInfo>,
}

impl<W: Write> SnapshotWriter<W> {
    /// Writes the header and returns a writer ready for chunks.
    pub fn new(mut out: W, info: &SnapshotInfo) -> Result<Self, PersistError> {
        let header = encode_header(info)?;
        out.write_all(&header)?;
        let mut hasher = Xxh3::new();
        hasher.update(&header);
        Ok(SnapshotWriter {
            out,
            hasher,
            pos: HEADER_LEN as u64,
            chunks: Vec::new(),
            seen: HashSet::new(),
            zstd_level: DEFAULT_ZSTD_LEVEL,
        })
    }

    /// Changes the zstd level for subsequent chunks.
    pub fn set_zstd_level(&mut self, level: i32) {
        self.zstd_level = level;
    }

    /// Appends one chunk.
    pub fn add_chunk(
        &mut self,
        tag: SectionTag,
        index: u32,
        version: u32,
        raw: &[u8],
        compression: Compression,
    ) -> Result<&ChunkInfo, PersistError> {
        if !self.seen.insert((tag, index)) {
            return Err(PersistError::DuplicateChunk { tag, index });
        }
        let raw_digest = xxh3_64(raw);
        let compressed = match compression {
            Compression::Raw => None,
            Compression::Zstd => {
                let packed = zstd::bulk::compress(raw, self.zstd_level)?;
                (packed.len() < raw.len()).then_some(packed)
            }
        };
        let (codec, stored): (Codec, &[u8]) = match &compressed {
            Some(packed) => (Codec::Zstd, packed),
            None => (Codec::Raw, raw),
        };
        self.out.write_all(stored)?;
        self.hasher.update(stored);
        let info = ChunkInfo {
            tag,
            index,
            version,
            codec,
            offset: self.pos,
            stored_len: stored.len() as u64,
            raw_len: raw.len() as u64,
            stored_digest: xxh3_64(stored),
            raw_digest,
        };
        self.pos += info.stored_len;
        self.chunks.push(info);
        Ok(self.chunks.last().expect("just pushed"))
    }

    /// Chunks written so far.
    pub fn chunks(&self) -> &[ChunkInfo] {
        &self.chunks
    }

    /// Writes the manifest and footer, flushes, and returns the writer.
    pub fn finish(mut self) -> Result<Finished<W>, PersistError> {
        let manifest = encode_manifest(&self.chunks);
        let manifest_offset = self.pos;
        self.out.write_all(&manifest)?;
        self.hasher.update(&manifest);
        let manifest_len = manifest.len() as u64;
        let body_digest = self.hasher.digest();
        let file_len = manifest_offset + manifest_len + FOOTER_LEN as u64;
        let footer = encode_footer(&Footer {
            manifest_offset,
            manifest_len,
            file_len,
            manifest_digest: xxh3_64(&manifest),
            body_digest,
        });
        self.out.write_all(&footer)?;
        self.out.flush()?;
        Ok(Finished {
            out: self.out,
            file_len,
            body_digest,
            chunks: self.chunks,
        })
    }
}

/// A snapshot that has been durably published.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Published {
    /// Final location of the file.
    pub path: PathBuf,
    /// Size of the file.
    pub file_len: u64,
    /// Digest of everything before the footer.
    pub body_digest: u64,
    /// The manifest as written.
    pub chunks: Vec<ChunkInfo>,
}

/// Writes a complete snapshot to `dir/file_name` crash-safely.
///
/// The bytes go to a uniquely named temporary file in the same directory, which is flushed and
/// synced before being renamed, without replacing anything, to its final name. An existing file
/// is never overwritten: publishing to a taken name fails with [`PersistError::AlreadyExists`].
/// If the process dies part-way, only an ignorable `.tmp-*.part` file remains.
pub fn publish_file(
    dir: &Path,
    file_name: &str,
    info: &SnapshotInfo,
    sections: &[SectionData],
    zstd_level: i32,
) -> Result<Published, PersistError> {
    let final_path = dir.join(file_name);
    if final_path.exists() {
        return Err(PersistError::AlreadyExists { path: final_path });
    }
    let mut temp = tempfile::Builder::new()
        .prefix(".tmp-")
        .suffix(".part")
        .tempfile_in(dir)?;
    let finished = {
        let mut writer = SnapshotWriter::new(BufWriter::new(temp.as_file_mut()), info)?;
        writer.set_zstd_level(zstd_level);
        for s in sections {
            writer.add_chunk(s.tag, s.index, s.version, &s.bytes, s.compression)?;
        }
        let finished = writer.finish()?;
        let Finished {
            out,
            file_len,
            body_digest,
            chunks,
        } = finished;
        out.into_inner()
            .map_err(|e| PersistError::Io(e.into_error()))?;
        (file_len, body_digest, chunks)
    };
    temp.as_file().sync_all()?;
    temp.persist_noclobber(&final_path).map_err(|e| {
        if e.error.kind() == std::io::ErrorKind::AlreadyExists {
            PersistError::AlreadyExists {
                path: final_path.clone(),
            }
        } else {
            PersistError::Io(e.error)
        }
    })?;
    sync_dir(dir);
    let (file_len, body_digest, chunks) = finished;
    Ok(Published {
        path: final_path,
        file_len,
        body_digest,
        chunks,
    })
}

/// Makes the rename itself durable where the platform allows it.
#[cfg(unix)]
fn sync_dir(dir: &Path) {
    if let Ok(d) = std::fs::File::open(dir) {
        let _ = d.sync_all();
    }
}

/// Windows has no directory sync through std. The file itself was synced before the rename, and
/// older generations stay valid if the rename is lost.
#[cfg(not(unix))]
fn sync_dir(_dir: &Path) {}
