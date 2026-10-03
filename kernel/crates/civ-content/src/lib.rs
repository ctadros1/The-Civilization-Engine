//! The content compiler (research 01-10).
//!
//! Authored primitives live as TOML files in packs under `content/`. This crate loads them into
//! strict authoring types (unknown fields are errors), checks identities and cross-file rules,
//! and compiles an immutable [`ContentRegistry`]. Diagnostics carry stable codes and file
//! positions, and serialise to JSON for tools and agents.
//!
//! Two fingerprints (BLAKE3) are kept:
//!
//! - an **artifact** fingerprint per pack, over the bytes of its files;
//! - a **semantic** fingerprint over the effective compiled content. This is the one saves
//!   record, so reformatting a file does not mark every save as "content changed".
//!
//! Kinds so far: world-generation presets (M0); the people profile, the land profile, name lists,
//! activities, goods and crops (M1). Later milestones add technologies, offices and the rest of the
//! plan's primitives, each as a new `kind`.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::path::{Path, PathBuf};

use civ_agents::params::{Catalog, GoodUse, PeopleParams};
use civ_land::LandParams;
use civ_world::TerrainParams;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

mod activity;
mod crop;
mod good;
mod land;
mod names;
mod people;
mod worldgen;

/// Version of the authoring format this build understands.
pub const CONTENT_SCHEMA: u32 = 1;
/// Version of the kernel's content API (which kinds and meanings exist).
pub const KERNEL_CONTENT_API: u32 = 3;

/// How serious a diagnostic is. Errors prevent the registry from being built.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// The content cannot be used.
    Error,
    /// The content can be used, but something is probably wrong.
    Warning,
}

/// One finding about the content, with a stable code.
///
/// | Code | Meaning |
/// |---|---|
/// | E1001 | The file is not valid TOML, or does not match its kind's shape (unknown, missing or mistyped field) |
/// | E1002 | A pack directory has no `pack.toml` |
/// | E1003 | A pack needs a content schema or kernel content API this build does not have |
/// | E1004 | A file could not be read |
/// | E2001 | An id is not of the form `pack:kind/name` with lowercase `[a-z0-9_]` segments |
/// | E2002 | An id's pack differs from the pack it is in |
/// | E2003 | An id's kind segment does not match the file's `kind` |
/// | E2004 | The file's path does not match its id (`<kind>/<name>.toml`) |
/// | E2005 | Two definitions share an id |
/// | E2006 | A reference names something that is not defined (a name list, a land resource, a good, a crop) |
/// | E3001 | A value is out of its allowed range |
/// | E3002 | The `kind` is unknown, or missing |
/// | E3003 | Not exactly one world-generation preset is marked `default = true` |
/// | E3004 | Not exactly one people profile, or not exactly one land profile |
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    /// Stable code, for example `E2004`.
    pub code: &'static str,
    /// How serious it is.
    pub severity: Severity,
    /// File relative to the content root, with `/` separators.
    pub file: String,
    /// 1-based line, when known.
    pub line: Option<usize>,
    /// 1-based column, when known.
    pub column: Option<usize>,
    /// What is wrong.
    pub message: String,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let level = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        write!(f, "{level}[{}] {}", self.code, self.file)?;
        if let Some(line) = self.line {
            write!(f, ":{line}")?;
            if let Some(col) = self.column {
                write!(f, ":{col}")?;
            }
        }
        write!(f, ": {}", self.message)
    }
}

/// A loaded pack.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PackInfo {
    /// Pack id, for example `core`.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Human-facing version.
    pub version: String,
    /// What the pack is.
    pub description: String,
    /// BLAKE3 of the pack's files (paths and bytes).
    #[serde(serialize_with = "hex_bytes")]
    pub artifact_fingerprint: [u8; 32],
}

/// A world-generation preset: a kind of landscape, not a particular world.
#[derive(Clone, Debug, PartialEq)]
pub struct WorldgenPreset {
    /// Stable id, for example `core:worldgen/river_valley`.
    pub id: String,
    /// The pack it came from.
    pub pack: String,
    /// Display name.
    pub name: String,
    /// What kind of landscape it makes.
    pub description: String,
    /// Whether the new-world dialog offers it first.
    pub is_default: bool,
    /// The parameters it compiles to.
    pub params: TerrainParams,
}

/// How a world's people live: bodies, sleep, company, households, choices and founding bands.
#[derive(Clone, Debug, PartialEq)]
pub struct PeopleProfile {
    /// Stable id, for example `core:people/early_farmers`.
    pub id: String,
    /// The pack it came from.
    pub pack: String,
    /// Display name.
    pub name: String,
    /// Who these people are.
    pub description: String,
    /// The parameters it compiles to, with its name list resolved.
    pub params: PeopleParams,
}

/// How land is classified into habitats and what grows wild on it.
#[derive(Clone, Debug, PartialEq)]
pub struct LandProfile {
    /// Stable id, for example `core:land/temperate_valley`.
    pub id: String,
    /// The pack it came from.
    pub pack: String,
    /// Display name.
    pub name: String,
    /// The parameters it compiles to.
    pub params: LandParams,
}

/// The compiled, immutable content.
#[derive(Clone, Debug, PartialEq)]
pub struct ContentRegistry {
    /// Packs, in id order.
    pub packs: Vec<PackInfo>,
    /// World-generation presets, in id order.
    pub presets: Vec<WorldgenPreset>,
    /// The people profile.
    pub people: PeopleProfile,
    /// The land profile.
    pub land: LandProfile,
    /// Activities, goods and crops, each in id order, with references resolved.
    pub catalog: Catalog,
    /// BLAKE3 of the effective content (see the crate docs).
    pub fingerprint: [u8; 32],
}

impl ContentRegistry {
    /// The preset with `id`.
    pub fn preset(&self, id: &str) -> Option<&WorldgenPreset> {
        self.presets.iter().find(|p| p.id == id)
    }

    /// The preset marked as default (validation guarantees exactly one).
    pub fn default_preset(&self) -> &WorldgenPreset {
        self.presets
            .iter()
            .find(|p| p.is_default)
            .unwrap_or(&self.presets[0])
    }

    /// The semantic fingerprint as lowercase hex.
    pub fn fingerprint_hex(&self) -> String {
        hex(&self.fingerprint)
    }
}

/// The result of loading content: the registry when there were no errors, and every diagnostic.
#[derive(Debug)]
pub struct LoadReport {
    /// The compiled content, absent if any error was found.
    pub registry: Option<ContentRegistry>,
    /// Everything found, errors and warnings, in file order.
    pub diagnostics: Vec<Diagnostic>,
}

impl LoadReport {
    /// Number of errors.
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count()
    }

    /// The report as JSON: `{ "ok": bool, "fingerprint": hex|null, "packs": [...], "presets":
    /// [ids], "people": id|null, "land": id|null, "activities": [ids], "goods": [ids],
    /// "crops": [ids], "diagnostics": [...] }`.
    pub fn to_json(&self) -> String {
        let r = self.registry.as_ref();
        let json = serde_json::json!({
            "ok": r.is_some(),
            "fingerprint": r.map(ContentRegistry::fingerprint_hex),
            "packs": r.map(|r| r.packs.clone()).unwrap_or_default(),
            "presets": r
                .map(|r| r.presets.iter().map(|p| p.id.clone()).collect::<Vec<_>>())
                .unwrap_or_default(),
            "people": r.map(|r| r.people.id.clone()),
            "land": r.map(|r| r.land.id.clone()),
            "activities": r
                .map(|r| r.catalog.activities.iter().map(|a| a.id.clone()).collect::<Vec<_>>())
                .unwrap_or_default(),
            "goods": r
                .map(|r| r.catalog.goods.iter().map(|g| g.id.clone()).collect::<Vec<_>>())
                .unwrap_or_default(),
            "crops": r
                .map(|r| r.catalog.crops.iter().map(|c| c.id.clone()).collect::<Vec<_>>())
                .unwrap_or_default(),
            "diagnostics": self.diagnostics,
        });
        serde_json::to_string_pretty(&json).unwrap_or_else(|_| "{}".to_owned())
    }
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

fn hex_bytes<S: serde::Serializer>(bytes: &[u8; 32], s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&hex(bytes))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PackFile {
    id: String,
    name: String,
    version: String,
    content_schema: u32,
    kernel_content_api: u32,
    #[serde(default)]
    description: String,
}

/// Finds the content root by walking up from `start` to the first directory containing
/// `content/core/pack.toml`.
pub fn find_content_root(start: &Path) -> Option<PathBuf> {
    let mut dir = Some(start);
    while let Some(d) = dir {
        let candidate = d.join("content");
        if candidate.join("core").join("pack.toml").is_file() {
            return Some(candidate);
        }
        dir = d.parent();
    }
    None
}

fn segment_ok(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        && s.as_bytes()[0].is_ascii_lowercase()
}

/// Splits `pack:kind/name`.
fn parse_id(id: &str) -> Option<(&str, &str, &str)> {
    let (pack, rest) = id.split_once(':')?;
    let (kind, name) = rest.split_once('/')?;
    (segment_ok(pack) && segment_ok(kind) && segment_ok(name)).then_some((pack, kind, name))
}

fn line_col(source: &str, offset: usize) -> (usize, usize) {
    let before = &source[..offset.min(source.len())];
    let line = before.matches('\n').count() + 1;
    let col = before
        .rfind('\n')
        .map_or(before.len(), |nl| before.len() - nl - 1)
        + 1;
    (line, col)
}

struct Collector {
    diagnostics: Vec<Diagnostic>,
}

impl Collector {
    fn push(&mut self, code: &'static str, file: &str, line: Option<usize>, message: String) {
        self.diagnostics.push(Diagnostic {
            code,
            severity: Severity::Error,
            file: file.to_owned(),
            line,
            column: None,
            message,
        });
    }

    fn toml_error(&mut self, file: &str, source: &str, err: &toml::de::Error) {
        let (line, column) = err
            .span()
            .map(|s| line_col(source, s.start))
            .map_or((None, None), |(l, c)| (Some(l), Some(c)));
        self.diagnostics.push(Diagnostic {
            code: "E1001",
            severity: Severity::Error,
            file: file.to_owned(),
            line,
            column,
            message: err.message().trim().to_owned(),
        });
    }
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

fn toml_files(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            toml_files(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "toml") {
            out.push(path);
        }
    }
    Ok(())
}

/// One parsed definition, before cross-references are resolved.
struct Def<T> {
    rel: String,
    pack: String,
    file: T,
    /// The parsed TOML, for the semantic fingerprint.
    table: toml::Table,
}

/// Everything parsed from every pack.
#[derive(Default)]
struct Parsed {
    presets: Vec<WorldgenPreset>,
    people: Vec<Def<people::PeopleFile>>,
    lands: Vec<Def<land::LandFile>>,
    names: Vec<Def<names::NamesFile>>,
    activities: Vec<Def<activity::ActivityFile>>,
    goods: Vec<Def<good::GoodFile>>,
    crops: Vec<Def<crop::CropFile>>,
    /// Files declaring `kind = "people"`, parsed or not (so a broken profile is not also
    /// reported as a missing one).
    people_files: usize,
    /// Files declaring `kind = "land"`, parsed or not.
    land_files: usize,
    /// Every `id` a file declares, parsed or not (so a reference to a broken definition is not
    /// also reported as a reference to a missing one).
    declared: HashSet<String>,
}

/// Loads, validates and compiles every pack under `root`.
pub fn load(root: &Path) -> LoadReport {
    let mut c = Collector {
        diagnostics: Vec::new(),
    };
    let mut packs = Vec::new();
    let mut parsed = Parsed::default();
    let mut seen_ids = HashSet::new();

    let mut pack_dirs: Vec<PathBuf> = match std::fs::read_dir(root) {
        Ok(entries) => entries
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.is_dir())
            .collect(),
        Err(e) => {
            c.push(
                "E1004",
                &root.display().to_string(),
                None,
                format!("cannot read the content directory: {e}"),
            );
            return LoadReport {
                registry: None,
                diagnostics: c.diagnostics,
            };
        }
    };
    pack_dirs.sort();

    for pack_dir in pack_dirs {
        let manifest_path = pack_dir.join("pack.toml");
        let manifest_rel = relative(root, &manifest_path);
        let Ok(source) = std::fs::read_to_string(&manifest_path) else {
            c.push(
                "E1002",
                &relative(root, &pack_dir),
                None,
                "pack directory has no readable pack.toml".to_owned(),
            );
            continue;
        };
        let manifest: PackFile = match toml::from_str(&source) {
            Ok(m) => m,
            Err(e) => {
                c.toml_error(&manifest_rel, &source, &e);
                continue;
            }
        };
        if manifest.content_schema != CONTENT_SCHEMA
            || manifest.kernel_content_api != KERNEL_CONTENT_API
        {
            c.push(
                "E1003",
                &manifest_rel,
                None,
                format!(
                    "pack needs content schema {} / kernel content API {}; this build has {} / {}",
                    manifest.content_schema,
                    manifest.kernel_content_api,
                    CONTENT_SCHEMA,
                    KERNEL_CONTENT_API
                ),
            );
            continue;
        }
        let dir_name = pack_dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if !segment_ok(&manifest.id) || manifest.id != dir_name {
            c.push(
                "E2001",
                &manifest_rel,
                None,
                format!(
                    "pack id `{}` must be lowercase [a-z0-9_] and match its directory `{dir_name}`",
                    manifest.id
                ),
            );
            continue;
        }

        let mut files = Vec::new();
        if let Err(e) = toml_files(&pack_dir, &mut files) {
            c.push("E1004", &relative(root, &pack_dir), None, e.to_string());
            continue;
        }
        let mut artifact = blake3::Hasher::new();
        for path in &files {
            let rel = relative(root, path);
            let source = match std::fs::read_to_string(path) {
                Ok(s) => s,
                Err(e) => {
                    c.push("E1004", &rel, None, e.to_string());
                    continue;
                }
            };
            artifact.update(rel.as_bytes());
            artifact.update(&[0]);
            artifact.update(&(source.len() as u64).to_le_bytes());
            artifact.update(source.as_bytes());
            if path == &manifest_path {
                continue;
            }
            compile_file(
                &mut c,
                &manifest.id,
                &rel,
                &source,
                &mut seen_ids,
                &mut parsed,
            );
        }
        packs.push(PackInfo {
            id: manifest.id,
            name: manifest.name,
            version: manifest.version,
            description: manifest.description,
            artifact_fingerprint: *artifact.finalize().as_bytes(),
        });
    }

    let defaults = parsed.presets.iter().filter(|p| p.is_default).count();
    if defaults != 1 {
        c.push(
            "E3003",
            "",
            None,
            format!(
                "exactly one world-generation preset must set `default = true`; found {defaults}"
            ),
        );
    }
    let (people, land, catalog) = resolve(&mut c, &parsed);

    let has_errors = c.diagnostics.iter().any(|d| d.severity == Severity::Error);
    let registry = match (has_errors, people, land) {
        (false, Some(people), Some(land)) => {
            let fingerprint = semantic_fingerprint(&parsed.presets, &parsed_tables(&parsed));
            let mut presets = parsed.presets;
            presets.sort_by(|a, b| a.id.cmp(&b.id));
            packs.sort_by(|a, b| a.id.cmp(&b.id));
            Some(ContentRegistry {
                packs,
                presets,
                people,
                land,
                catalog,
                fingerprint,
            })
        }
        _ => None,
    };
    LoadReport {
        registry,
        diagnostics: c.diagnostics,
    }
}

/// Reports a reference to something not defined (E2006), unless a file declares it (a broken
/// definition is already reported; one error is enough).
fn missing(c: &mut Collector, parsed: &Parsed, rel: &str, field: &str, id: &str) {
    if !parsed.declared.contains(id) {
        c.push(
            "E2006",
            rel,
            None,
            format!("`{field}` refers to `{id}`, which is not defined"),
        );
    }
}

/// Checks the profiles (E3004) and resolves cross-references (E2006).
fn resolve(
    c: &mut Collector,
    parsed: &Parsed,
) -> (Option<PeopleProfile>, Option<LandProfile>, Catalog) {
    for (count, what) in [
        (parsed.people_files, "people profile (`kind = \"people\"`)"),
        (parsed.land_files, "land profile (`kind = \"land\"`)"),
    ] {
        if count != 1 {
            c.push(
                "E3004",
                "",
                None,
                format!("the content needs exactly one {what}; found {count}"),
            );
        }
    }
    let mut goods: Vec<_> = parsed.goods.iter().filter_map(|d| d.file.def()).collect();
    goods.sort_by(|a, b| a.id.cmp(&b.id));
    let good_index = |id: &str| goods.iter().position(|g| g.id == id);
    let land = match parsed.lands.as_slice() {
        [d] => {
            for r in &d.file.resource {
                if good_index(&r.good).is_none() {
                    missing(
                        c,
                        parsed,
                        &d.rel,
                        &format!("resource `{}` good", r.id),
                        &r.good,
                    );
                }
            }
            d.file.params(&good_index).map(|params| LandProfile {
                id: d.file.id.clone(),
                pack: d.pack.clone(),
                name: d.file.name.clone(),
                params,
            })
        }
        _ => None,
    };
    let mut crops = Vec::new();
    for d in &parsed.crops {
        for (field, id) in [("good", &d.file.good), ("seed_good", &d.file.seed_good)] {
            if good_index(id).is_none() {
                missing(c, parsed, &d.rel, field, id);
            }
        }
        crops.extend(d.file.params(&good_index));
    }
    crops.sort_by(|a, b| a.id.cmp(&b.id));
    let people = match parsed.people.as_slice() {
        [d] => {
            let names = parsed.names.iter().find(|n| n.file.id == d.file.names);
            if names.is_none() {
                missing(c, parsed, &d.rel, "names", &d.file.names);
            }
            let crop = crops.iter().position(|x| x.id == d.file.farm.crop);
            if crop.is_none() {
                missing(c, parsed, &d.rel, "farm.crop", &d.file.farm.crop);
            }
            let provisions = &d.file.band.provisions_good;
            let good = good_index(provisions);
            match good.map(|g| &goods[g]) {
                None => missing(c, parsed, &d.rel, "band.provisions_good", provisions),
                Some(g) if g.purpose != GoodUse::Food => c.push(
                    "E3001",
                    &d.rel,
                    None,
                    format!("`band.provisions_good` must be a food; `{provisions}` is not"),
                ),
                Some(_) => {}
            }
            match (names, good, crop) {
                (Some(n), Some(g), Some(crop)) if goods[g].purpose == GoodUse::Food => {
                    Some(PeopleProfile {
                        id: d.file.id.clone(),
                        pack: d.pack.clone(),
                        name: d.file.name.clone(),
                        description: d.file.description.clone(),
                        params: d.file.params(n.file.params(), g, crop),
                    })
                }
                _ => None,
            }
        }
        _ => None,
    };
    let mut activities = Vec::new();
    for d in &parsed.activities {
        let resource = match (&d.file.resource, &land) {
            (None, _) => None,
            (Some(r), Some(l)) => match l.params.resource(r) {
                Some(i) => Some(i),
                None => {
                    c.push(
                        "E2006",
                        &d.rel,
                        None,
                        format!(
                            "`resource` refers to `{r}`, which the land profile `{}` does not \
                             define",
                            l.id
                        ),
                    );
                    continue;
                }
            },
            // Without a land profile, an error (E3004, or the land profile's own) is already
            // reported.
            (Some(_), None) => continue,
        };
        activities.extend(d.file.def(resource));
    }
    activities.sort_by(|a, b| a.id.cmp(&b.id));
    (
        people,
        land,
        Catalog {
            activities,
            goods,
            crops,
        },
    )
}

/// `(id, parsed TOML)` of every definition other than presets, for the semantic fingerprint.
fn parsed_tables(parsed: &Parsed) -> Vec<(&str, &toml::Table)> {
    fn tables<T>(defs: &[Def<T>], id: impl Fn(&T) -> &str) -> Vec<(&str, &toml::Table)> {
        defs.iter().map(|d| (id(&d.file), &d.table)).collect()
    }
    let mut all = tables(&parsed.people, |f| &f.id);
    all.extend(tables(&parsed.lands, |f| &f.id));
    all.extend(tables(&parsed.names, |f| &f.id));
    all.extend(tables(&parsed.activities, |f| &f.id));
    all.extend(tables(&parsed.goods, |f| &f.id));
    all.extend(tables(&parsed.crops, |f| &f.id));
    all
}

fn parse<T: DeserializeOwned>(c: &mut Collector, rel: &str, source: &str) -> Option<T> {
    match toml::from_str(source) {
        Ok(f) => Some(f),
        Err(e) => {
            c.toml_error(rel, source, &e);
            None
        }
    }
}

/// Checks a definition's id against its pack, kind and path (E2001–E2005). Returns `false` when
/// the id cannot be used at all.
fn check_identity(
    c: &mut Collector,
    pack: &str,
    rel: &str,
    id: &str,
    id_kind: &str,
    seen_ids: &mut HashSet<String>,
) -> bool {
    let Some((id_pack, kind_segment, name)) = parse_id(id) else {
        c.push(
            "E2001",
            rel,
            None,
            format!("id `{id}` is not of the form pack:kind/name"),
        );
        return false;
    };
    if id_pack != pack {
        c.push(
            "E2002",
            rel,
            None,
            format!("id `{id}` names pack `{id_pack}` but the file is in pack `{pack}`"),
        );
    }
    if kind_segment != id_kind {
        c.push(
            "E2003",
            rel,
            None,
            format!("id `{id}` has kind segment `{kind_segment}`; this kind uses `{id_kind}`"),
        );
    }
    let expected_path = format!("{pack}/{id_kind}/{name}.toml");
    if rel != expected_path {
        c.push(
            "E2004",
            rel,
            None,
            format!("a definition with id `{id}` belongs at `{expected_path}`"),
        );
    }
    if !seen_ids.insert(id.to_owned()) {
        c.push("E2005", rel, None, format!("id `{id}` is defined twice"));
    }
    true
}

fn range_problems(c: &mut Collector, rel: &str, problems: Vec<String>) {
    for problem in problems {
        c.push("E3001", rel, None, problem);
    }
}

const KINDS: [&str; 7] = [
    worldgen::KIND,
    people::KIND,
    land::KIND,
    names::KIND,
    activity::KIND,
    good::KIND,
    crop::KIND,
];

fn compile_file(
    c: &mut Collector,
    pack: &str,
    rel: &str,
    source: &str,
    seen_ids: &mut HashSet<String>,
    parsed: &mut Parsed,
) {
    let table: toml::Table = match toml::from_str(source) {
        Ok(t) => t,
        Err(e) => {
            c.toml_error(rel, source, &e);
            return;
        }
    };
    let kind = table.get("kind").and_then(|k| k.as_str()).unwrap_or("");
    if let Some(id) = table.get("id").and_then(|k| k.as_str()) {
        parsed.declared.insert(id.to_owned());
    }
    fn def<T>(rel: &str, pack: &str, file: T, table: toml::Table) -> Def<T> {
        Def {
            rel: rel.to_owned(),
            pack: pack.to_owned(),
            file,
            table,
        }
    }
    match kind {
        worldgen::KIND => {
            let Some(file) = parse::<worldgen::PresetFile>(c, rel, source) else {
                return;
            };
            debug_assert_eq!(file.kind, worldgen::KIND, "dispatched on kind");
            if !check_identity(c, pack, rel, &file.id, worldgen::ID_KIND, seen_ids) {
                return;
            }
            let params = file.params();
            if let Err(why) = params.validate() {
                range_problems(c, rel, why.split("; ").map(str::to_owned).collect());
            }
            parsed.presets.push(WorldgenPreset {
                id: file.id,
                pack: pack.to_owned(),
                name: file.name,
                description: file.description,
                is_default: file.default,
                params,
            });
        }
        people::KIND => {
            parsed.people_files += 1;
            let Some(file) = parse::<people::PeopleFile>(c, rel, source) else {
                return;
            };
            debug_assert_eq!(file.kind, people::KIND, "dispatched on kind");
            if check_identity(c, pack, rel, &file.id, people::ID_KIND, seen_ids) {
                range_problems(c, rel, file.problems());
                parsed.people.push(def(rel, pack, file, table));
            }
        }
        land::KIND => {
            parsed.land_files += 1;
            let Some(file) = parse::<land::LandFile>(c, rel, source) else {
                return;
            };
            debug_assert_eq!(file.kind, land::KIND, "dispatched on kind");
            if check_identity(c, pack, rel, &file.id, land::ID_KIND, seen_ids) {
                range_problems(c, rel, file.problems());
                parsed.lands.push(def(rel, pack, file, table));
            }
        }
        names::KIND => {
            let Some(file) = parse::<names::NamesFile>(c, rel, source) else {
                return;
            };
            debug_assert_eq!(file.kind, names::KIND, "dispatched on kind");
            if check_identity(c, pack, rel, &file.id, names::ID_KIND, seen_ids) {
                range_problems(c, rel, file.problems());
                parsed.names.push(def(rel, pack, file, table));
            }
        }
        activity::KIND => {
            let Some(file) = parse::<activity::ActivityFile>(c, rel, source) else {
                return;
            };
            debug_assert_eq!(file.kind, activity::KIND, "dispatched on kind");
            if check_identity(c, pack, rel, &file.id, activity::ID_KIND, seen_ids) {
                range_problems(c, rel, file.problems());
                parsed.activities.push(def(rel, pack, file, table));
            }
        }
        good::KIND => {
            let Some(file) = parse::<good::GoodFile>(c, rel, source) else {
                return;
            };
            debug_assert_eq!(file.kind, good::KIND, "dispatched on kind");
            if check_identity(c, pack, rel, &file.id, good::ID_KIND, seen_ids) {
                range_problems(c, rel, file.problems());
                parsed.goods.push(def(rel, pack, file, table));
            }
        }
        crop::KIND => {
            let Some(file) = parse::<crop::CropFile>(c, rel, source) else {
                return;
            };
            debug_assert_eq!(file.kind, crop::KIND, "dispatched on kind");
            if check_identity(c, pack, rel, &file.id, crop::ID_KIND, seen_ids) {
                range_problems(c, rel, file.problems());
                parsed.crops.push(def(rel, pack, file, table));
            }
        }
        "" => c.push(
            "E3002",
            rel,
            None,
            "file has no `kind`; every definition names its kind".to_owned(),
        ),
        other => c.push(
            "E3002",
            rel,
            None,
            format!("unknown kind `{other}` (known: {})", KINDS.join(", ")),
        ),
    }
}

/// BLAKE3 over a canonical encoding of the effective content. Formatting, key order and comments
/// do not affect it:
///
/// - presets in id order, each field by name with numbers as exact IEEE bits;
/// - every other definition in id order, as its parsed TOML with keys sorted, every number as the
///   IEEE bits of its value (so `3` and `3.0` are the same), and a type tag before each value.
///
/// Every field of those kinds is required or has no default, so the parsed TOML is exactly what
/// they compile to.
fn semantic_fingerprint(presets: &[WorldgenPreset], others: &[(&str, &toml::Table)]) -> [u8; 32] {
    let mut h = blake3::Hasher::new();
    h.update(b"tce-content/2\n");
    let mut sorted: BTreeMap<&str, &WorldgenPreset> = BTreeMap::new();
    for p in presets {
        sorted.insert(&p.id, p);
    }
    for (id, p) in sorted {
        for text in [id, p.name.as_str(), p.description.as_str()] {
            h.update(&(text.len() as u64).to_le_bytes());
            h.update(text.as_bytes());
        }
        h.update(&[u8::from(p.is_default)]);
        for (name, value) in p.params.named_values() {
            h.update(name.as_bytes());
            h.update(&[0]);
            h.update(&value.to_bits().to_le_bytes());
        }
    }
    let others: BTreeMap<&str, &toml::Table> = others.iter().copied().collect();
    for (id, table) in others {
        hash_text(&mut h, id);
        hash_table(&mut h, table);
    }
    *h.finalize().as_bytes()
}

fn hash_text(h: &mut blake3::Hasher, text: &str) {
    h.update(&(text.len() as u64).to_le_bytes());
    h.update(text.as_bytes());
}

fn hash_table(h: &mut blake3::Hasher, table: &toml::Table) {
    let mut keys: Vec<&String> = table.keys().collect();
    keys.sort();
    h.update(b"t");
    h.update(&(keys.len() as u64).to_le_bytes());
    for key in keys {
        hash_text(h, key);
        hash_value(h, &table[key.as_str()]);
    }
}

fn hash_value(h: &mut blake3::Hasher, value: &toml::Value) {
    match value {
        toml::Value::String(s) => {
            h.update(b"s");
            hash_text(h, s);
        }
        toml::Value::Integer(i) => {
            h.update(b"n");
            h.update(&(*i as f64).to_bits().to_le_bytes());
        }
        toml::Value::Float(f) => {
            h.update(b"n");
            h.update(&f.to_bits().to_le_bytes());
        }
        toml::Value::Boolean(b) => {
            h.update(b"b");
            h.update(&[u8::from(*b)]);
        }
        toml::Value::Datetime(d) => {
            h.update(b"d");
            hash_text(h, &d.to_string());
        }
        toml::Value::Array(items) => {
            h.update(b"a");
            h.update(&(items.len() as u64).to_le_bytes());
            for item in items {
                hash_value(h, item);
            }
        }
        toml::Value::Table(t) => hash_table(h, t),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_parse_strictly() {
        assert_eq!(
            parse_id("core:worldgen/river_valley"),
            Some(("core", "worldgen", "river_valley"))
        );
        for bad in [
            "core/worldgen/x",
            "Core:worldgen/x",
            "core:worldgen/",
            "core:worldgen/x-y",
            ":a/b",
            "core:1a/b",
        ] {
            assert_eq!(parse_id(bad), None, "{bad}");
        }
    }

    #[test]
    fn line_and_column_are_one_based() {
        let src = "a = 1\nbb = 2\n";
        assert_eq!(line_col(src, 0), (1, 1));
        assert_eq!(line_col(src, 6), (2, 1));
        assert_eq!(line_col(src, 8), (2, 3));
    }
}
