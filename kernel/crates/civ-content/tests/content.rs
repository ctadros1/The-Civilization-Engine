//! The real `content/` directory must load cleanly; each diagnostic code has a fixture that
//! triggers it; fingerprints respond to meaning, not formatting.

use std::path::{Path, PathBuf};

use civ_content::{ContentRegistry, LoadReport, load};

fn repo_content() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../content")
}

/// The river valley preset with LF line endings, so the edits below match on any checkout (Git for
/// Windows checks text out with CRLF by default).
fn real_preset() -> String {
    std::fs::read_to_string(repo_content().join("core/worldgen/river_valley.toml"))
        .expect("the river valley preset exists")
        .replace("\r\n", "\n")
}

const PACK: &str = r#"
id = "core"
name = "Core"
version = "0.1.0"
content_schema = 1
kernel_content_api = 1
"#;

/// Writes a pack named `core` containing the given files and loads it.
fn load_fixture(files: &[(&str, &str)]) -> LoadReport {
    let dir = tempfile::tempdir().expect("tempdir");
    let core = dir.path().join("core");
    std::fs::create_dir_all(core.join("worldgen")).expect("mkdir");
    std::fs::write(core.join("pack.toml"), PACK).expect("write pack");
    for (path, body) in files {
        let full = core.join(path);
        std::fs::create_dir_all(full.parent().expect("parent")).expect("mkdir");
        std::fs::write(full, body).expect("write");
    }
    load(dir.path())
}

fn codes(report: &LoadReport) -> Vec<&'static str> {
    report.diagnostics.iter().map(|d| d.code).collect()
}

fn registry(report: LoadReport) -> ContentRegistry {
    let diagnostics = report
        .diagnostics
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    report
        .registry
        .unwrap_or_else(|| panic!("content failed: {diagnostics:#?}"))
}

#[test]
fn the_repository_content_is_clean() {
    let report = load(&repo_content());
    assert!(
        report.diagnostics.is_empty(),
        "{:#?}",
        report
            .diagnostics
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    let reg = registry(report);
    assert_eq!(reg.default_preset().id, "core:worldgen/river_valley");
    assert!(reg.preset("core:worldgen/ria_coast").is_some());
    assert_eq!(reg.packs.len(), 1);
}

#[test]
fn unknown_fields_are_errors_with_a_line() {
    let body = real_preset().replace("[uplift]", "[uplift]\nuplfit_mm_per_yr = 1.0");
    let report = load_fixture(&[("worldgen/river_valley.toml", &body)]);
    assert_eq!(codes(&report), vec!["E1001", "E3003"]);
    assert!(report.diagnostics[0].line.is_some());
    assert!(report.registry.is_none());
}

#[test]
fn missing_fields_are_errors() {
    let body = real_preset().replace("talus_slope = 0.85\n", "");
    let report = load_fixture(&[("worldgen/river_valley.toml", &body)]);
    assert!(codes(&report).contains(&"E1001"));
}

#[test]
fn malformed_ids_are_rejected() {
    let body = real_preset().replace("core:worldgen/river_valley", "core/river_valley");
    assert!(codes(&load_fixture(&[("worldgen/river_valley.toml", &body)])).contains(&"E2001"));
}

#[test]
fn ids_must_match_pack_kind_and_path() {
    let wrong_pack =
        real_preset().replace("core:worldgen/river_valley", "other:worldgen/river_valley");
    assert!(
        codes(&load_fixture(&[(
            "worldgen/river_valley.toml",
            &wrong_pack
        )]))
        .contains(&"E2002")
    );

    let wrong_kind =
        real_preset().replace("core:worldgen/river_valley", "core:terrain/river_valley");
    let report = load_fixture(&[("terrain/river_valley.toml", &wrong_kind)]);
    assert!(codes(&report).contains(&"E2003"));

    let report = load_fixture(&[("worldgen/elsewhere.toml", &real_preset())]);
    assert!(codes(&report).contains(&"E2004"));
}

#[test]
fn duplicate_ids_are_rejected() {
    let body = real_preset();
    let report = load_fixture(&[
        ("worldgen/river_valley.toml", &body),
        ("worldgen/sub/river_valley.toml", &body),
    ]);
    assert!(codes(&report).contains(&"E2005"));
}

#[test]
fn out_of_range_values_are_rejected() {
    let body = real_preset().replace("outlet_edges = 1 ", "outlet_edges = 5 ");
    let report = load_fixture(&[("worldgen/river_valley.toml", &body)]);
    assert_eq!(codes(&report), vec!["E3001"]);
    assert!(report.diagnostics[0].message.contains("outlet_edges"));
}

#[test]
fn unknown_or_missing_kinds_are_rejected() {
    let report = load_fixture(&[
        ("worldgen/river_valley.toml", &real_preset()),
        (
            "goods/grain.toml",
            "kind = \"good\"\nid = \"core:good/grain\"\n",
        ),
        ("notes/x.toml", "title = \"no kind here\"\n"),
    ]);
    assert_eq!(codes(&report), vec!["E3002", "E3002"]);
}

#[test]
fn exactly_one_default_preset_is_required() {
    let no_default = real_preset().replace("default = true", "default = false");
    assert_eq!(
        codes(&load_fixture(&[(
            "worldgen/river_valley.toml",
            &no_default
        )])),
        vec!["E3003"]
    );
    let second = real_preset().replace("core:worldgen/river_valley", "core:worldgen/second");
    assert_eq!(
        codes(&load_fixture(&[
            ("worldgen/river_valley.toml", &real_preset()),
            ("worldgen/second.toml", &second),
        ])),
        vec!["E3003"]
    );
}

#[test]
fn unsupported_schemas_are_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");
    let core = dir.path().join("core");
    std::fs::create_dir_all(&core).expect("mkdir");
    std::fs::write(
        core.join("pack.toml"),
        PACK.replace("content_schema = 1", "content_schema = 9"),
    )
    .expect("write");
    let report = load(dir.path());
    assert!(codes(&report).contains(&"E1003"));
}

#[test]
fn the_semantic_fingerprint_follows_meaning_not_formatting() {
    let base = registry(load_fixture(&[(
        "worldgen/river_valley.toml",
        &real_preset(),
    )]));

    // Comments and blank lines change the bytes, so the artifact fingerprint changes, but the
    // semantic fingerprint does not.
    let reformatted = format!("# a new comment\n\n{}", real_preset());
    let same = registry(load_fixture(&[(
        "worldgen/river_valley.toml",
        &reformatted,
    )]));
    assert_eq!(same.fingerprint, base.fingerprint);
    assert_ne!(
        same.packs[0].artifact_fingerprint,
        base.packs[0].artifact_fingerprint
    );

    // So do line endings: a save made from a Windows checkout (CRLF) loads elsewhere without a
    // spurious "content changed" note.
    let crlf = real_preset().replace('\n', "\r\n");
    let same = registry(load_fixture(&[("worldgen/river_valley.toml", &crlf)]));
    assert_eq!(same.fingerprint, base.fingerprint);

    // A changed value changes it.
    let changed = real_preset().replace("talus_slope = 0.85", "talus_slope = 0.86");
    let different = registry(load_fixture(&[("worldgen/river_valley.toml", &changed)]));
    assert_ne!(different.fingerprint, base.fingerprint);
}

#[test]
fn json_reports_are_machine_readable() {
    let report = load_fixture(&[("worldgen/elsewhere.toml", &real_preset())]);
    let json: serde_json::Value = serde_json::from_str(&report.to_json()).expect("valid JSON");
    assert_eq!(json["ok"], false);
    assert_eq!(json["diagnostics"][0]["code"], "E2004");
    assert_eq!(json["diagnostics"][0]["severity"], "error");
}
