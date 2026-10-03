//! Where the host finds its content, saves and web shell.
//!
//! By default everything is relative to the repository: `content/` is found by walking up from
//! the current directory, then from the executable; `saves/` and `web/dist/` sit next to it.

use std::path::{Path, PathBuf};

use anyhow::{Context, bail};

/// The folders the host uses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layout {
    /// The content root (the folder holding `core/pack.toml`).
    pub content: PathBuf,
    /// Where worlds' save directories go.
    pub saves: PathBuf,
    /// The built web shell, if there is one (a folder holding `index.html`).
    pub web: Option<PathBuf>,
}

fn find_content() -> Option<PathBuf> {
    let from_cwd = std::env::current_dir()
        .ok()
        .and_then(|dir| civ_content::find_content_root(&dir));
    from_cwd.or_else(|| {
        std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().and_then(civ_content::find_content_root))
    })
}

/// Resolves the layout, filling in what was not given.
pub fn locate(
    content: Option<PathBuf>,
    saves: Option<PathBuf>,
    web: Option<PathBuf>,
) -> anyhow::Result<Layout> {
    let content = match content {
        Some(dir) => dir,
        None => find_content().context(
            "could not find the content folder (content/core/pack.toml) above the current \
             directory; run from inside the repository or pass --content",
        )?,
    };
    if !content.join("core").join("pack.toml").is_file() {
        bail!(
            "{} is not a content folder: it has no core/pack.toml",
            content.display()
        );
    }
    let root = content.parent().unwrap_or(Path::new(".")).to_owned();
    let saves = saves.unwrap_or_else(|| root.join("saves"));
    let web = web
        .unwrap_or_else(|| root.join("web").join("dist"))
        .canonicalize()
        .ok()
        .filter(|dir| dir.join("index.html").is_file());
    Ok(Layout {
        content,
        saves,
        web,
    })
}
