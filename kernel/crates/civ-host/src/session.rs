//! The saves directory as the host sees it: the session marker that detects unclean shutdowns,
//! the recovery offer, the save list, and safe resolution of save paths sent by clients.
//!
//! Layout: `<saves>/<world-slug>-<id8>/g##########-<kind>.tcesave`, plus `<saves>/.session.json`
//! while a host is running.

use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use civ_content::ContentRegistry;
use civ_schema::SAVE_EXTENSION;
use civ_sim::persist;
use commons_persist::{Limits, SaveDir, SaveKind, SnapshotReader};
use serde::{Deserialize, Serialize};

use crate::protocol::{RecoveryOffer, SaveEntry};

const MARKER: &str = ".session.json";
const MARKER_TEMP: &str = ".session.json.tmp";
/// Most saves the save browser lists.
pub const MAX_LISTED: usize = 500;

/// Written while a host runs and removed when it stops cleanly. Finding one at start-up means the
/// previous session ended unexpectedly. It also serves as the plan's last-good-snapshot pointer
/// (§3.5).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionMarker {
    /// Process id of the host that wrote it.
    pub pid: u32,
    /// When that host started, Unix milliseconds.
    pub started_unix_ms: i64,
    /// The current world's save directory, relative to the saves root.
    pub world_dir: Option<String>,
    /// The newest save of the current world known to be good (verified when it was written or
    /// loaded), relative to the saves root.
    pub last_good: Option<String>,
}

impl SessionMarker {
    /// A marker for this process.
    pub fn for_this_process() -> Self {
        SessionMarker {
            pid: std::process::id(),
            started_unix_ms: commons_persist::now_unix_ms(),
            world_dir: None,
            last_good: None,
        }
    }

    /// The marker the previous session left, if it did not stop cleanly. An unreadable marker
    /// still means it did not.
    pub fn left_behind(root: &Path) -> Option<SessionMarker> {
        let text = fs::read_to_string(root.join(MARKER)).ok()?;
        Some(serde_json::from_str(&text).unwrap_or_default())
    }

    /// Writes the marker through a temp file and a rename, so it is never half-written.
    pub fn write(&self, root: &Path) -> io::Result<()> {
        let temp = root.join(MARKER_TEMP);
        fs::write(
            &temp,
            serde_json::to_vec_pretty(self).map_err(io::Error::other)?,
        )?;
        fs::rename(&temp, root.join(MARKER))
    }

    /// Removes the marker: the session ended cleanly.
    pub fn remove(root: &Path) -> io::Result<()> {
        match fs::remove_file(root.join(MARKER)) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            other => other,
        }
    }
}

/// `path` relative to `root`, with `/` separators; `None` if it is not under `root`.
pub fn relative(root: &Path, path: &Path) -> Option<String> {
    let rel = path.strip_prefix(root).ok()?;
    let parts = rel
        .components()
        .map(|c| match c {
            Component::Normal(s) => s.to_str().map(str::to_owned),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    (!parts.is_empty()).then(|| parts.join("/"))
}

/// Why a client's save path was refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathError {
    /// Not a plain relative path to a save file.
    Invalid(String),
    /// No such file.
    NotFound(String),
}

/// Resolves a save path sent by a client: it must be relative, stay inside the saves root, name
/// a `.tcesave` file and exist.
pub fn resolve(root: &Path, file: &str) -> Result<PathBuf, PathError> {
    let rel = Path::new(file);
    let plain = !file.is_empty()
        && !file.starts_with("./")
        && rel.components().all(|c| matches!(c, Component::Normal(_)));
    if !plain || rel.extension().and_then(|e| e.to_str()) != Some(SAVE_EXTENSION) {
        return Err(PathError::Invalid(format!(
            "`{file}` is not a save file name"
        )));
    }
    let path = root.join(rel);
    if path.is_file() {
        Ok(path)
    } else {
        Err(PathError::NotFound(format!("there is no save `{file}`")))
    }
}

/// Every save under `root`, newest first (at most [`MAX_LISTED`]). Reads headers and the `meta`
/// section only.
pub fn list_saves(root: &Path, content: &ContentRegistry) -> io::Result<Vec<SaveEntry>> {
    let mut out = Vec::new();
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(out),
        Err(e) => return Err(e),
    };
    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let dir_name = entry.file_name();
        let Some(dir_name) = dir_name.to_str() else {
            continue;
        };
        if dir_name.starts_with('.') {
            continue;
        }
        let dir = SaveDir::create(entry.path(), SAVE_EXTENSION)?;
        for generation in dir.scan()? {
            let Some(file_name) = generation.path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            let summary = persist::describe(&generation.path, content);
            let info = summary.info.as_ref();
            out.push(SaveEntry {
                file: format!("{dir_name}/{file_name}"),
                world_name: summary.world_name.clone(),
                world_id: info.map_or_else(String::new, |i| commons_persist::hex(&i.world_id)),
                label: info.map_or_else(String::new, |i| i.label.clone()),
                kind: generation.kind.as_str().to_owned(),
                generation: generation.generation,
                created_unix_ms: info.map_or(0, |i| i.created_unix_ms),
                sim_minute: info.map_or(0, |i| i.sim_time),
                size_bytes: summary.file_len,
                compatible: summary.compatible,
                content_changed: summary.content_changed,
                note: summary.note,
            });
        }
    }
    out.sort_by(|a, b| {
        b.created_unix_ms
            .cmp(&a.created_unix_ms)
            .then(b.generation.cmp(&a.generation))
    });
    out.truncate(MAX_LISTED);
    Ok(out)
}

/// Whether a save is complete and its body digest matches.
fn intact(path: &Path) -> bool {
    SnapshotReader::open_file(path, Limits::default())
        .and_then(|mut reader| reader.verify_body())
        .is_ok()
}

/// The save to offer for recovery: the newest manual or autosave generation of the marker's world
/// that is intact and loadable by this build, else its last good save. Crash snapshots are kept
/// for diagnosis and never offered.
pub fn find_recovery(
    root: &Path,
    marker: &SessionMarker,
    content: &ContentRegistry,
    reason: &str,
) -> Option<RecoveryOffer> {
    let mut candidates = Vec::new();
    if let Some(dir) = marker.world_dir.as_deref() {
        let dir = root.join(dir);
        if dir.is_dir()
            && let Ok(files) = SaveDir::create(&dir, SAVE_EXTENSION).and_then(|d| d.scan())
        {
            candidates.extend(
                files
                    .into_iter()
                    .rev()
                    .filter(|g| g.kind != SaveKind::Crash)
                    .map(|g| g.path),
            );
        }
    }
    if let Some(path) = marker
        .last_good
        .as_deref()
        .and_then(|f| resolve(root, f).ok())
    {
        candidates.push(path);
    }
    for path in candidates {
        let summary = persist::describe(&path, content);
        let (Some(info), Some(file)) = (summary.info.as_ref(), relative(root, &path)) else {
            continue;
        };
        if summary.compatible && intact(&path) {
            return Some(RecoveryOffer {
                reason: reason.to_owned(),
                file,
                label: info.label.clone(),
                sim_minute: info.sim_time,
                world_name: summary.world_name.clone(),
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_paths_stay_inside_the_saves_root() {
        let dir = tempfile::tempdir().expect("temp dir");
        let root = dir.path();
        fs::create_dir_all(root.join("w-1")).expect("mkdir");
        fs::write(root.join("w-1/g0000000001-manual.tcesave"), b"x").expect("write");

        assert!(resolve(root, "w-1/g0000000001-manual.tcesave").is_ok());
        for bad in [
            "",
            "../w-1/g0000000001-manual.tcesave",
            "w-1/../../etc/passwd.tcesave",
            "/etc/passwd.tcesave",
            "w-1/g0000000001-manual.txt",
            "./w-1/g0000000001-manual.tcesave",
        ] {
            assert!(
                matches!(resolve(root, bad), Err(PathError::Invalid(_))),
                "{bad:?} was accepted"
            );
        }
        assert!(matches!(
            resolve(root, "w-1/g0000000002-manual.tcesave"),
            Err(PathError::NotFound(_))
        ));
    }

    #[test]
    fn the_marker_round_trips_and_is_removed() {
        let dir = tempfile::tempdir().expect("temp dir");
        let root = dir.path();
        assert_eq!(SessionMarker::left_behind(root), None);
        let mut marker = SessionMarker::for_this_process();
        marker.world_dir = Some("w-1".to_owned());
        marker.write(root).expect("writes");
        assert_eq!(SessionMarker::left_behind(root), Some(marker));
        SessionMarker::remove(root).expect("removes");
        assert_eq!(SessionMarker::left_behind(root), None);
        SessionMarker::remove(root).expect("removing twice is fine");

        fs::write(root.join(MARKER), b"{ not json").expect("write");
        assert_eq!(
            SessionMarker::left_behind(root),
            Some(SessionMarker::default()),
            "a damaged marker still means an unclean exit"
        );
    }

    #[test]
    fn relative_paths_use_forward_slashes() {
        let root = Path::new("saves");
        let path = root.join("w-1").join("g0000000001-auto.tcesave");
        assert_eq!(
            relative(root, &path).as_deref(),
            Some("w-1/g0000000001-auto.tcesave")
        );
        assert_eq!(relative(root, Path::new("elsewhere/x")), None);
    }
}
