use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use xxhash_rust::xxh3::{Xxh3, xxh3_64};

use crate::error::PersistError;
use crate::format::{
    ChunkInfo, Codec, FOOTER_LEN, Footer, HEADER_LEN, Limits, MAGIC, MANIFEST_ENTRY_LEN,
    MANIFEST_PREFIX_LEN, SectionTag, SnapshotInfo, decode_footer, decode_header, decode_manifest,
};

/// One decoded chunk of a section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectionChunk {
    /// Position within the section.
    pub index: u32,
    /// Engine-defined version of the section's encoding.
    pub version: u32,
    /// The decoded bytes, already checked against their digest.
    pub bytes: Vec<u8>,
}

/// Reads and validates a snapshot.
///
/// [`SnapshotReader::new`] checks the header, manifest and footer: everything needed to list the
/// snapshot, without reading chunk bodies. Chunks are checked as they are read.
/// [`SnapshotReader::verify_all`] checks every byte.
#[derive(Debug)]
pub struct SnapshotReader<R> {
    src: R,
    info: SnapshotInfo,
    chunks: Vec<ChunkInfo>,
    footer_body_digest: u64,
    body_len: u64,
    file_len: u64,
    limits: Limits,
}

impl SnapshotReader<BufReader<File>> {
    /// Opens a snapshot file.
    pub fn open_file(path: &Path, limits: Limits) -> Result<Self, PersistError> {
        let file = File::open(path)?;
        SnapshotReader::new(BufReader::new(file), limits)
    }
}

fn read_exact_at<R: Read + Seek>(src: &mut R, at: u64, buf: &mut [u8]) -> Result<(), PersistError> {
    src.seek(SeekFrom::Start(at))?;
    src.read_exact(buf)?;
    Ok(())
}

impl<R: Read + Seek> SnapshotReader<R> {
    /// Validates the fixed structure of a snapshot read from `src`.
    pub fn new(mut src: R, limits: Limits) -> Result<Self, PersistError> {
        let file_len = src.seek(SeekFrom::End(0))?;
        let mut magic = [0u8; 8];
        let magic_len = file_len.min(8) as usize;
        read_exact_at(&mut src, 0, &mut magic[..magic_len])?;
        if magic_len < 8 || magic != MAGIC {
            return if MAGIC.starts_with(&magic[..magic_len]) {
                Err(PersistError::Incomplete {
                    reason: "file ends inside the header",
                })
            } else {
                Err(PersistError::NotASnapshot)
            };
        }
        let min_len = (HEADER_LEN + MANIFEST_PREFIX_LEN + FOOTER_LEN) as u64;
        if file_len < min_len {
            return Err(PersistError::Incomplete {
                reason: "file is shorter than the fixed structure",
            });
        }

        let mut footer_bytes = [0u8; FOOTER_LEN];
        read_exact_at(&mut src, file_len - FOOTER_LEN as u64, &mut footer_bytes)?;
        let footer: Footer = decode_footer(&footer_bytes)?;
        if footer.file_len != file_len {
            return Err(PersistError::Incomplete {
                reason: "file length disagrees with the footer",
            });
        }
        let body_len = footer
            .manifest_offset
            .checked_add(footer.manifest_len)
            .ok_or(PersistError::FooterCorrupt {
                reason: "manifest bounds overflow",
            })?;
        if footer.manifest_offset < HEADER_LEN as u64
            || body_len.checked_add(FOOTER_LEN as u64) != Some(file_len)
        {
            return Err(PersistError::FooterCorrupt {
                reason: "manifest is not where the footer says",
            });
        }
        if footer.manifest_len < MANIFEST_PREFIX_LEN as u64
            || !(footer.manifest_len - MANIFEST_PREFIX_LEN as u64)
                .is_multiple_of(MANIFEST_ENTRY_LEN as u64)
        {
            return Err(PersistError::FooterCorrupt {
                reason: "manifest length is not a whole number of entries",
            });
        }
        let count = (footer.manifest_len - MANIFEST_PREFIX_LEN as u64) / MANIFEST_ENTRY_LEN as u64;
        if count > u64::from(limits.max_chunks) {
            return Err(PersistError::TooManyChunks {
                count,
                max: limits.max_chunks,
            });
        }

        let mut manifest = vec![0u8; footer.manifest_len as usize];
        read_exact_at(&mut src, footer.manifest_offset, &mut manifest)?;
        if xxh3_64(&manifest) != footer.manifest_digest {
            return Err(PersistError::ManifestCorrupt {
                reason: "manifest digest mismatch".to_owned(),
            });
        }
        let chunks = decode_manifest(&manifest)?;

        let mut header = [0u8; HEADER_LEN];
        read_exact_at(&mut src, 0, &mut header)?;
        let info = decode_header(&header)?;

        validate_layout(&chunks, footer.manifest_offset, &limits)?;

        Ok(SnapshotReader {
            src,
            info,
            chunks,
            footer_body_digest: footer.body_digest,
            body_len,
            file_len,
            limits,
        })
    }

    /// The snapshot's identity and provenance.
    pub fn info(&self) -> &SnapshotInfo {
        &self.info
    }

    /// The manifest, in file order.
    pub fn chunks(&self) -> &[ChunkInfo] {
        &self.chunks
    }

    /// Size of the file.
    pub fn file_len(&self) -> u64 {
        self.file_len
    }

    /// Whether any chunk of `tag` is present.
    pub fn has_section(&self, tag: SectionTag) -> bool {
        self.chunks.iter().any(|c| c.tag == tag)
    }

    /// Checks the whole-body digest by reading every byte before the footer.
    pub fn verify_body(&mut self) -> Result<(), PersistError> {
        self.src.seek(SeekFrom::Start(0))?;
        let mut hasher = Xxh3::new();
        let mut remaining = self.body_len;
        let mut buf = vec![0u8; 1 << 20];
        while remaining > 0 {
            let take = remaining.min(buf.len() as u64) as usize;
            self.src.read_exact(&mut buf[..take])?;
            hasher.update(&buf[..take]);
            remaining -= take as u64;
        }
        let found = hasher.digest();
        if found != self.footer_body_digest {
            return Err(PersistError::BodyCorrupt {
                expected: self.footer_body_digest,
                found,
            });
        }
        Ok(())
    }

    /// Checks the body digest and decodes every chunk against its digests.
    pub fn verify_all(&mut self) -> Result<(), PersistError> {
        self.verify_body()?;
        for chunk in self.chunks.clone() {
            self.read_chunk(&chunk)?;
        }
        Ok(())
    }

    /// Reads, checks and decodes one chunk.
    pub fn read_chunk(&mut self, chunk: &ChunkInfo) -> Result<Vec<u8>, PersistError> {
        let mut stored = vec![0u8; chunk.stored_len as usize];
        read_exact_at(&mut self.src, chunk.offset, &mut stored)?;
        if xxh3_64(&stored) != chunk.stored_digest {
            return Err(PersistError::ChunkCorrupt {
                tag: chunk.tag,
                index: chunk.index,
                stage: "stored",
            });
        }
        let raw = match chunk.codec {
            Codec::Raw => stored,
            Codec::Zstd => {
                // The declared raw length (already bounded by the limits) caps the output, so a
                // hostile chunk cannot decompress into unbounded memory.
                let raw = zstd::bulk::decompress(&stored, chunk.raw_len as usize).map_err(|e| {
                    PersistError::Decompress {
                        tag: chunk.tag,
                        index: chunk.index,
                        message: e.to_string(),
                    }
                })?;
                if raw.len() as u64 != chunk.raw_len {
                    return Err(PersistError::Decompress {
                        tag: chunk.tag,
                        index: chunk.index,
                        message: format!(
                            "decoded to {} bytes, manifest says {}",
                            raw.len(),
                            chunk.raw_len
                        ),
                    });
                }
                raw
            }
        };
        if xxh3_64(&raw) != chunk.raw_digest {
            return Err(PersistError::ChunkCorrupt {
                tag: chunk.tag,
                index: chunk.index,
                stage: "raw",
            });
        }
        Ok(raw)
    }

    /// Reads every chunk of `tag`, ordered by index. Indices must run contiguously from zero.
    /// Returns an empty list when the section is absent.
    pub fn read_section(&mut self, tag: SectionTag) -> Result<Vec<SectionChunk>, PersistError> {
        let mut parts: Vec<ChunkInfo> = self
            .chunks
            .iter()
            .filter(|c| c.tag == tag)
            .cloned()
            .collect();
        parts.sort_by_key(|c| c.index);
        let mut out = Vec::with_capacity(parts.len());
        for (expected, chunk) in parts.iter().enumerate() {
            if chunk.index as usize != expected {
                return Err(PersistError::MissingChunk {
                    tag,
                    index: expected as u32,
                });
            }
            out.push(SectionChunk {
                index: chunk.index,
                version: chunk.version,
                bytes: self.read_chunk(chunk)?,
            });
        }
        Ok(out)
    }

    /// Like [`SnapshotReader::read_section`], but an absent section is an error.
    pub fn require_section(&mut self, tag: SectionTag) -> Result<Vec<SectionChunk>, PersistError> {
        let parts = self.read_section(tag)?;
        if parts.is_empty() {
            return Err(PersistError::MissingSection { tag });
        }
        Ok(parts)
    }

    /// The limits this reader enforces.
    pub fn limits(&self) -> &Limits {
        &self.limits
    }
}

fn validate_layout(
    chunks: &[ChunkInfo],
    manifest_offset: u64,
    limits: &Limits,
) -> Result<(), PersistError> {
    let mut seen = HashSet::with_capacity(chunks.len());
    let mut cursor = HEADER_LEN as u64;
    for c in chunks {
        if !seen.insert((c.tag, c.index)) {
            return Err(PersistError::DuplicateChunk {
                tag: c.tag,
                index: c.index,
            });
        }
        if c.raw_len > limits.max_chunk_raw_len {
            return Err(PersistError::ChunkTooLarge {
                tag: c.tag,
                index: c.index,
                len: c.raw_len,
                max: limits.max_chunk_raw_len,
            });
        }
        if c.stored_len > limits.max_chunk_stored_len {
            return Err(PersistError::ChunkTooLarge {
                tag: c.tag,
                index: c.index,
                len: c.stored_len,
                max: limits.max_chunk_stored_len,
            });
        }
        if c.codec == Codec::Raw && c.stored_len != c.raw_len {
            return Err(PersistError::ManifestCorrupt {
                reason: format!(
                    "raw chunk `{}`/{} has different stored and raw lengths",
                    c.tag, c.index
                ),
            });
        }
        let end = c.offset.checked_add(c.stored_len);
        if c.offset != cursor || end.is_none_or(|e| e > manifest_offset) {
            return Err(PersistError::ManifestCorrupt {
                reason: format!(
                    "chunk `{}`/{} is not where the chunks before it end",
                    c.tag, c.index
                ),
            });
        }
        cursor = end.unwrap_or(u64::MAX);
    }
    if cursor != manifest_offset {
        return Err(PersistError::ManifestCorrupt {
            reason: "bytes between the last chunk and the manifest".to_owned(),
        });
    }
    Ok(())
}
