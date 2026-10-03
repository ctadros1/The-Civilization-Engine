//! A directory of snapshot generations for one world.
//!
//! Each save is a new file, `g{generation:010}-{kind}.{ext}`, and is never overwritten. Recovery
//! does not trust a "latest" pointer: it scans and validates the files themselves.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::error::PersistError;
use crate::format::SnapshotInfo;
use crate::writer::{DEFAULT_ZSTD_LEVEL, Published, SectionData, publish_file};

/// Why a generation was written. Only autosaves are pruned automatically.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SaveKind {
    /// Requested by the player.
    Manual,
    /// Written on the autosave schedule; rotated.
    Autosave,
    /// Written after a kernel failure, for diagnosis.
    Crash,
}

impl SaveKind {
    /// The name used in file names.
    pub fn as_str(self) -> &'static str {
        match self {
            SaveKind::Manual => "manual",
            SaveKind::Autosave => "auto",
            SaveKind::Crash => "crash",
        }
    }

    /// Parses a file-name kind.
    pub fn parse(text: &str) -> Option<SaveKind> {
        match text {
            "manual" => Some(SaveKind::Manual),
            "auto" => Some(SaveKind::Autosave),
            "crash" => Some(SaveKind::Crash),
            _ => None,
        }
    }
}

/// A generation file found on disk. Its contents have not been validated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenerationFile {
    /// Location.
    pub path: PathBuf,
    /// Generation number from the file name.
    pub generation: u64,
    /// Kind from the file name.
    pub kind: SaveKind,
}

/// The save directory of one world.
#[derive(Clone, Debug)]
pub struct SaveDir {
    dir: PathBuf,
    extension: String,
}

impl SaveDir {
    /// Opens (creating if needed) a save directory whose files use `extension`.
    pub fn create(dir: impl Into<PathBuf>, extension: &str) -> io::Result<Self> {
        let dir = dir.into();
        fs::create_dir_all(&dir)?;
        Ok(SaveDir {
            dir,
            extension: extension.to_owned(),
        })
    }

    /// The directory.
    pub fn path(&self) -> &Path {
        &self.dir
    }

    /// The file name a generation of `kind` gets.
    pub fn file_name(&self, generation: u64, kind: SaveKind) -> String {
        format!("g{generation:010}-{}.{}", kind.as_str(), self.extension)
    }

    /// Parses a generation file name; `None` for anything else, including temp files.
    pub fn parse_file_name(&self, name: &str) -> Option<(u64, SaveKind)> {
        let stem = name.strip_suffix(&format!(".{}", self.extension))?;
        let rest = stem.strip_prefix('g')?;
        let (number, kind) = rest.split_once('-')?;
        if number.len() != 10 || !number.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        Some((number.parse().ok()?, SaveKind::parse(kind)?))
    }

    /// Every generation file, oldest first.
    pub fn scan(&self) -> io::Result<Vec<GenerationFile>> {
        let mut found = Vec::new();
        for entry in fs::read_dir(&self.dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            let name = entry.file_name();
            let Some(name) = name.to_str() else { continue };
            if let Some((generation, kind)) = self.parse_file_name(name) {
                found.push(GenerationFile {
                    path: entry.path(),
                    generation,
                    kind,
                });
            }
        }
        found.sort_by_key(|g| g.generation);
        Ok(found)
    }

    /// One more than the highest generation present (1 for an empty directory).
    pub fn next_generation(&self) -> io::Result<u64> {
        Ok(self.scan()?.last().map_or(1, |g| g.generation + 1))
    }

    /// Publishes a new generation. Sets `info.generation` to the number it receives.
    pub fn publish(
        &self,
        kind: SaveKind,
        info: &mut SnapshotInfo,
        sections: &[SectionData],
    ) -> Result<Published, PersistError> {
        self.publish_with_level(kind, info, sections, DEFAULT_ZSTD_LEVEL)
    }

    /// Like [`SaveDir::publish`], with an explicit zstd level.
    pub fn publish_with_level(
        &self,
        kind: SaveKind,
        info: &mut SnapshotInfo,
        sections: &[SectionData],
        zstd_level: i32,
    ) -> Result<Published, PersistError> {
        // A concurrent writer can take the number between scanning and renaming. The rename
        // refuses to clobber, so retry with the next number.
        let mut last_error = None;
        for _ in 0..8 {
            let generation = self.next_generation()?;
            info.generation = generation;
            match publish_file(
                &self.dir,
                &self.file_name(generation, kind),
                info,
                sections,
                zstd_level,
            ) {
                Err(e @ PersistError::AlreadyExists { .. }) => last_error = Some(e),
                other => return other,
            }
        }
        Err(last_error.unwrap_or(PersistError::Incomplete {
            reason: "could not claim a generation number",
        }))
    }

    /// Deletes the oldest files of `kind` so that at most `keep` remain. Returns what was removed.
    pub fn prune(&self, kind: SaveKind, keep: usize) -> io::Result<Vec<PathBuf>> {
        let of_kind: Vec<GenerationFile> = self
            .scan()?
            .into_iter()
            .filter(|g| g.kind == kind)
            .collect();
        let excess = of_kind.len().saturating_sub(keep);
        let mut removed = Vec::with_capacity(excess);
        for g in of_kind.into_iter().take(excess) {
            fs::remove_file(&g.path)?;
            removed.push(g.path);
        }
        Ok(removed)
    }

    /// Leftover temp files from interrupted saves. They are never loaded.
    pub fn stray_temp_files(&self) -> io::Result<Vec<PathBuf>> {
        let mut found = Vec::new();
        for entry in fs::read_dir(&self.dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let Some(name) = name.to_str() else { continue };
            if name.starts_with(".tmp-") && name.ends_with(".part") {
                found.push(entry.path());
            }
        }
        Ok(found)
    }
}
