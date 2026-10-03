use std::fmt;
use std::path::PathBuf;

use crate::format::SectionTag;

/// Why a snapshot could not be written or read.
///
/// Every read-side variant is a refusal. Nothing in this crate tries to salvage part of a damaged
/// file; recovery means choosing an older, valid generation.
#[derive(Debug)]
pub enum PersistError {
    /// The operating system reported an I/O failure.
    Io(std::io::Error),
    /// The file does not start with the container magic.
    NotASnapshot,
    /// The file ends before its structure does, or its completion footer is missing: a save that
    /// was interrupted while being written.
    Incomplete {
        /// What was missing.
        reason: &'static str,
    },
    /// The container layout version is not one this reader understands.
    UnsupportedContainerVersion {
        /// Version in the file.
        found: u16,
        /// Version this build reads.
        supported: u16,
    },
    /// The header's own digest does not match its bytes, or its declared length is wrong.
    HeaderCorrupt {
        /// What failed.
        reason: &'static str,
    },
    /// The manifest does not match its digest or describes an impossible layout.
    ManifestCorrupt {
        /// What failed.
        reason: String,
    },
    /// The footer describes an impossible layout.
    FooterCorrupt {
        /// What failed.
        reason: &'static str,
    },
    /// The whole-body digest does not match: some byte in the file changed after it was written.
    BodyCorrupt {
        /// Digest recorded in the footer.
        expected: u64,
        /// Digest of the bytes read.
        found: u64,
    },
    /// A chunk's stored or decoded bytes do not match their digest.
    ChunkCorrupt {
        /// Section of the chunk.
        tag: SectionTag,
        /// Chunk index within the section.
        index: u32,
        /// Which digest failed: `"stored"` or `"raw"`.
        stage: &'static str,
    },
    /// A chunk declares a size beyond the configured limits.
    ChunkTooLarge {
        /// Section of the chunk.
        tag: SectionTag,
        /// Chunk index within the section.
        index: u32,
        /// Declared length.
        len: u64,
        /// Largest length allowed.
        max: u64,
    },
    /// The manifest lists more chunks than the configured limit.
    TooManyChunks {
        /// Declared count.
        count: u64,
        /// Largest count allowed.
        max: u32,
    },
    /// A compressed chunk could not be decompressed to exactly its declared size.
    Decompress {
        /// Section of the chunk.
        tag: SectionTag,
        /// Chunk index within the section.
        index: u32,
        /// Codec error text.
        message: String,
    },
    /// A section name that is not 1–8 bytes of `[a-z0-9_-]`.
    InvalidSectionTag {
        /// The offending bytes.
        raw: Vec<u8>,
    },
    /// A label that is too long or contains NUL.
    InvalidLabel {
        /// What was wrong.
        reason: &'static str,
    },
    /// A codec byte this reader does not know.
    UnknownCodec {
        /// Section of the chunk.
        tag: SectionTag,
        /// Chunk index within the section.
        index: u32,
        /// The codec byte.
        value: u8,
    },
    /// Two chunks with the same section tag and index.
    DuplicateChunk {
        /// Section of the chunk.
        tag: SectionTag,
        /// Chunk index within the section.
        index: u32,
    },
    /// A required section is absent.
    MissingSection {
        /// The section.
        tag: SectionTag,
    },
    /// A section's chunk indices are not contiguous from zero.
    MissingChunk {
        /// The section.
        tag: SectionTag,
        /// The first missing index.
        index: u32,
    },
    /// The destination file already exists; generations are never overwritten.
    AlreadyExists {
        /// The existing file.
        path: PathBuf,
    },
    /// The snapshot belongs to a different engine.
    WrongEngine {
        /// Engine tag in the file.
        found: [u8; 8],
        /// Engine tag expected.
        expected: [u8; 8],
    },
    /// The operating system could not supply random bytes for an identifier.
    Random(String),
}

impl fmt::Display for PersistError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PersistError::Io(e) => write!(f, "I/O error: {e}"),
            PersistError::NotASnapshot => f.write_str("not a snapshot file (bad magic)"),
            PersistError::Incomplete { reason } => {
                write!(f, "incomplete snapshot (interrupted write?): {reason}")
            }
            PersistError::UnsupportedContainerVersion { found, supported } => write!(
                f,
                "snapshot container version {found}, this build reads version {supported}"
            ),
            PersistError::HeaderCorrupt { reason } => {
                write!(f, "snapshot header corrupt: {reason}")
            }
            PersistError::ManifestCorrupt { reason } => {
                write!(f, "snapshot manifest corrupt: {reason}")
            }
            PersistError::FooterCorrupt { reason } => {
                write!(f, "snapshot footer corrupt: {reason}")
            }
            PersistError::BodyCorrupt { expected, found } => write!(
                f,
                "snapshot body corrupt: digest {found:016x}, recorded {expected:016x}"
            ),
            PersistError::ChunkCorrupt { tag, index, stage } => {
                write!(
                    f,
                    "section `{tag}` chunk {index}: {stage} bytes fail their digest"
                )
            }
            PersistError::ChunkTooLarge {
                tag,
                index,
                len,
                max,
            } => write!(
                f,
                "section `{tag}` chunk {index} declares {len} bytes, limit is {max}"
            ),
            PersistError::TooManyChunks { count, max } => {
                write!(f, "manifest lists {count} chunks, limit is {max}")
            }
            PersistError::Decompress {
                tag,
                index,
                message,
            } => write!(
                f,
                "section `{tag}` chunk {index} does not decompress: {message}"
            ),
            PersistError::InvalidSectionTag { raw } => {
                write!(f, "invalid section tag {:?}", String::from_utf8_lossy(raw))
            }
            PersistError::InvalidLabel { reason } => write!(f, "invalid snapshot label: {reason}"),
            PersistError::UnknownCodec { tag, index, value } => {
                write!(f, "section `{tag}` chunk {index}: unknown codec {value}")
            }
            PersistError::DuplicateChunk { tag, index } => {
                write!(f, "section `{tag}` chunk {index} appears twice")
            }
            PersistError::MissingSection { tag } => write!(f, "required section `{tag}` is absent"),
            PersistError::MissingChunk { tag, index } => {
                write!(f, "section `{tag}` is missing chunk {index}")
            }
            PersistError::AlreadyExists { path } => write!(
                f,
                "{} already exists; snapshots are never overwritten",
                path.display()
            ),
            PersistError::WrongEngine { found, expected } => write!(
                f,
                "snapshot belongs to engine {:?}, expected {:?}",
                String::from_utf8_lossy(found).trim_end_matches('\0'),
                String::from_utf8_lossy(expected).trim_end_matches('\0')
            ),
            PersistError::Random(message) => {
                write!(f, "could not obtain random bytes: {message}")
            }
        }
    }
}

impl std::error::Error for PersistError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PersistError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for PersistError {
    fn from(e: std::io::Error) -> Self {
        PersistError::Io(e)
    }
}
