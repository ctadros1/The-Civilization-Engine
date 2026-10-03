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

/// A file of the real core pack, with LF line endings.
fn real(path: &str) -> String {
    std::fs::read_to_string(repo_content().join("core").join(path))
        .unwrap_or_else(|e| panic!("{path}: {e}"))
        .replace("\r\n", "\n")
}

/// The real people, land, names, activity, good and crop files (`(path in the pack, body)`), which
/// every world needs; fixtures add presets and override files.
fn people_files() -> Vec<(String, String)> {
    let mut out = Vec::new();
    for dir in ["people", "land", "names", "activity", "good", "crop"] {
        let mut paths: Vec<_> = std::fs::read_dir(repo_content().join("core").join(dir))
            .expect("real content directory")
            .map(|e| e.expect("entry").path())
            .collect();
        paths.sort();
        for path in paths {
            let name = path
                .file_name()
                .expect("name")
                .to_string_lossy()
                .into_owned();
            let rel = format!("{dir}/{name}");
            out.push((rel.clone(), real(&rel)));
        }
    }
    out
}

const PACK: &str = r#"
id = "core"
name = "Core"
version = "0.1.0"
content_schema = 1
kernel_content_api = 3
"#;

/// Writes a pack named `core` containing exactly the given files and loads it.
fn load_files(files: &[(&str, &str)]) -> LoadReport {
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

/// Like [`load_files`], with the real people, land, names, activities and goods underneath: the given
/// files are added, replacing a real file at the same path.
fn load_fixture(files: &[(&str, &str)]) -> LoadReport {
    let base = people_files();
    let mut all: Vec<(&str, &str)> = base
        .iter()
        .filter(|(path, _)| files.iter().all(|(p, _)| p != path))
        .map(|(p, b)| (p.as_str(), b.as_str()))
        .collect();
    all.extend_from_slice(files);
    load_files(&all)
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
    assert_eq!(reg.people.id, "core:people/early_farmers");
    assert!(!reg.people.params.names.female.is_empty(), "names resolved");
    assert_eq!(reg.land.id, "core:land/temperate_valley");
    let ids: Vec<&str> = reg
        .catalog
        .activities
        .iter()
        .map(|a| a.id.as_str())
        .collect();
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    assert_eq!(ids, sorted, "activities are in id order");
    for (activity, resource, good) in [
        ("gather_plants", "wild_plants", "core:good/wild_plant_food"),
        ("hunt", "game", "core:good/meat"),
        ("fish", "fish", "core:good/fish"),
        ("gather_wood", "deadwood", "core:good/firewood"),
    ] {
        let def = &reg.catalog.activities[reg
            .catalog
            .index_of(&format!("core:activity/{activity}"))
            .expect("the activity exists")];
        let r = def.resource.expect("gathering names a resource");
        let res = &reg.land.params.resources[r];
        assert_eq!(res.id, resource);
        assert_eq!(
            reg.catalog.goods[res.good].id, good,
            "{resource} yields {good}"
        );
    }
    let goods: Vec<&str> = reg.catalog.goods.iter().map(|g| g.id.as_str()).collect();
    let mut sorted = goods.clone();
    sorted.sort_unstable();
    assert_eq!(goods, sorted, "goods are in id order");
    assert_eq!(
        reg.catalog.goods[reg.people.params.band.provisions_good].id,
        "core:good/provisions"
    );
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
            "technology/pottery.toml",
            "kind = \"technology\"\nid = \"core:technology/pottery\"\n",
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
fn people_and_land_profiles_are_needed_exactly_once() {
    let preset = real_preset();
    let mut without_people: Vec<(String, String)> = people_files()
        .into_iter()
        .filter(|(p, _)| !p.starts_with("people/"))
        .collect();
    without_people.push(("worldgen/river_valley.toml".into(), preset.clone()));
    let files: Vec<(&str, &str)> = without_people
        .iter()
        .map(|(p, b)| (p.as_str(), b.as_str()))
        .collect();
    assert_eq!(codes(&load_files(&files)), vec!["E3004"]);

    let second = real("land/temperate_valley.toml")
        .replace("core:land/temperate_valley", "core:land/second");
    let report = load_fixture(&[
        ("worldgen/river_valley.toml", &preset),
        ("land/second.toml", &second),
    ]);
    assert_eq!(codes(&report), vec!["E3004"]);
    assert!(report.diagnostics[0].message.contains("found 2"));
}

#[test]
fn references_must_resolve() {
    let preset = real_preset();
    let people = real("people/early_farmers.toml")
        .replace("\"core:names/valley_folk\"", "\"core:names/elsewhere\"");
    let report = load_fixture(&[
        ("worldgen/river_valley.toml", &preset),
        ("people/early_farmers.toml", &people),
    ]);
    assert_eq!(codes(&report), vec!["E2006"]);
    assert!(
        report.diagnostics[0]
            .message
            .contains("core:names/elsewhere")
    );

    // A broken name list is reported once, not again as missing.
    let names = real("names/valley_folk.toml").replace("male = [", "mail = [");
    let report = load_fixture(&[
        ("worldgen/river_valley.toml", &preset),
        ("names/valley_folk.toml", &names),
    ]);
    assert_eq!(codes(&report), vec!["E1001"]);

    let gather = real("activity/gather_plants.toml").replace("\"wild_plants\"", "\"truffles\"");
    let report = load_fixture(&[
        ("worldgen/river_valley.toml", &preset),
        ("activity/gather_plants.toml", &gather),
    ]);
    assert_eq!(codes(&report), vec!["E2006"]);
    assert_eq!(
        report.diagnostics[0].file,
        "core/activity/gather_plants.toml"
    );

    // A land resource's good and the band's provisions are goods that must exist.
    let land =
        real("land/temperate_valley.toml").replace("\"core:good/meat\"", "\"core:good/venison\"");
    let report = load_fixture(&[
        ("worldgen/river_valley.toml", &preset),
        ("land/temperate_valley.toml", &land),
    ]);
    assert_eq!(codes(&report), vec!["E2006"]);
    assert!(report.diagnostics[0].message.contains("core:good/venison"));
    let people = real("people/early_farmers.toml")
        .replace("\"core:good/provisions\"", "\"core:good/barley_flour\"");
    let report = load_fixture(&[
        ("worldgen/river_valley.toml", &preset),
        ("people/early_farmers.toml", &people),
    ]);
    assert_eq!(codes(&report), vec!["E2006"]);
    // ... and provisions must be food.
    let people = real("people/early_farmers.toml")
        .replace("\"core:good/provisions\"", "\"core:good/firewood\"");
    let report = load_fixture(&[
        ("worldgen/river_valley.toml", &preset),
        ("people/early_farmers.toml", &people),
    ]);
    assert_eq!(codes(&report), vec!["E3001"]);
    assert!(report.diagnostics[0].message.contains("must be a food"));

    // The crop people grow, and the goods a crop yields and is sown from, must exist.
    let people =
        real("people/early_farmers.toml").replace("\"core:crop/emmer\"", "\"core:crop/rye\"");
    let report = load_fixture(&[
        ("worldgen/river_valley.toml", &preset),
        ("people/early_farmers.toml", &people),
    ]);
    assert_eq!(codes(&report), vec!["E2006"]);
    assert!(report.diagnostics[0].message.contains("core:crop/rye"));
    let crop =
        real("crop/emmer.toml").replace("\"core:good/seed_grain\"", "\"core:good/seed_corn\"");
    let report = load_fixture(&[
        ("worldgen/river_valley.toml", &preset),
        ("crop/emmer.toml", &crop),
    ]);
    assert!(codes(&report).contains(&"E2006"), "{:?}", codes(&report));
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| d.message.contains("core:good/seed_corn"))
    );
}

#[test]
fn crops_and_field_work_check_their_numbers() {
    let preset = real_preset();
    for (from, to, needle) in [
        (
            "sow_until_day = 125",
            "sow_until_day = 70",
            "sow_from_day <= sow_until_day",
        ),
        ("grow_days = 120", "grow_days = 300", "within the year"),
        (
            "standing_loss_per_day = 0.02",
            "standing_loss_per_day = 0.0",
            "an unreaped crop ends",
        ),
        (
            "seed_kg_per_ha = 90.0",
            "seed_kg_per_ha = 0.0",
            "`seed_kg_per_ha`",
        ),
    ] {
        let body = real("crop/emmer.toml").replace(from, to);
        assert_ne!(body, real("crop/emmer.toml"), "{needle}: the edit applies");
        let report = load_fixture(&[
            ("worldgen/river_valley.toml", &preset),
            ("crop/emmer.toml", &body),
        ]);
        assert!(
            codes(&report).contains(&"E3001"),
            "{needle}: {:?}",
            codes(&report)
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.message.contains(needle)),
            "{needle}"
        );
    }
    // A farm activity names its task; other activities do not take one.
    let sow = real("activity/sow.toml");
    for (path, body, needle) in [
        (
            "activity/sow.toml",
            sow.replace("task = \"sow\"\n", ""),
            "names the field `task`",
        ),
        (
            "activity/sow.toml",
            sow.replace("task = \"sow\"", "task = \"plough\""),
            "unknown task",
        ),
        (
            "activity/rest.toml",
            real("activity/rest.toml")
                .replace("behavior = \"rest\"", "behavior = \"rest\"\ntask = \"sow\""),
            "only `farm` activities",
        ),
    ] {
        assert_ne!(body, real(path), "{needle}: the edit applies");
        let report = load_fixture(&[("worldgen/river_valley.toml", &preset), (path, &body)]);
        assert_eq!(codes(&report), vec!["E3001"], "{needle}");
        assert!(report.diagnostics[0].message.contains(needle), "{needle}");
    }
}

#[test]
fn goods_check_their_purpose() {
    let preset = real_preset();
    for (path, from, to, needle) in [
        (
            "good/meat.toml",
            "purpose = \"food\"",
            "purpose = \"toy\"",
            "unknown purpose",
        ),
        (
            "good/meat.toml",
            "kcal_per_kg = 1500.0",
            "kcal_per_kg = 0.0",
            "positive `kcal_per_kg`",
        ),
        (
            "good/firewood.toml",
            "half_life_days = 0.0",
            "half_life_days = 30.0",
            "a fuel keeps",
        ),
        (
            "good/firewood.toml",
            "cooked = false",
            "cooked = true",
            "only food can need cooking",
        ),
    ] {
        let body = real(path).replace(from, to);
        assert_ne!(body, real(path), "{needle}: the edit applies");
        let report = load_fixture(&[("worldgen/river_valley.toml", &preset), (path, &body)]);
        assert!(
            codes(&report).contains(&"E3001"),
            "{needle}: {:?}",
            codes(&report)
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.message.contains(needle)),
            "{needle}"
        );
    }
}

#[test]
fn activities_check_their_behavior() {
    let preset = real_preset();
    let gather = real("activity/gather_plants.toml");
    for (body, needle) in [
        (
            gather.replace("\"gather\"", "\"fly\""),
            "unknown behavior `fly`",
        ),
        (
            gather.replace("resource = \"wild_plants\"\n", ""),
            "names the `resource`",
        ),
        (
            real("activity/rest.toml").replace(
                "behavior = \"rest\"",
                "behavior = \"rest\"\nresource = \"wild_plants\"",
            ),
            "only `gather` activities",
        ),
        (gather.replace("par = 3.0", "par = 0.5"), "`par`"),
        (
            gather.replace("min_minutes = 60", "min_minutes = 600"),
            "minutes",
        ),
    ] {
        let path = if body.contains("core:activity/rest") {
            "activity/rest.toml"
        } else {
            "activity/gather_plants.toml"
        };
        let report = load_fixture(&[("worldgen/river_valley.toml", &preset), (path, &body)]);
        assert_eq!(codes(&report), vec!["E3001"], "{needle}");
        assert!(
            report.diagnostics[0].message.contains(needle),
            "{needle}: {}",
            report.diagnostics[0].message
        );
    }
}

#[test]
fn profiles_check_their_ranges_and_fields() {
    let preset = real_preset();
    let land = real("land/temperate_valley.toml").replace(
        "production_per_ha_yr = [0.0, 9.0, 6.0, 5.0, 2.5]",
        "production_per_ha_yr = [0.0, 9.0]",
    );
    let report = load_fixture(&[
        ("worldgen/river_valley.toml", &preset),
        ("land/temperate_valley.toml", &land),
    ]);
    assert_eq!(codes(&report), vec!["E3001"]);
    assert!(
        report.diagnostics[0]
            .message
            .contains("one production figure per habitat")
    );

    // Unknown fields are caught inside nested tables too.
    let people = real("people/early_farmers.toml").replace("[sleep]", "[sleep]\ntau_awak_h = 18.0");
    let report = load_fixture(&[
        ("worldgen/river_valley.toml", &preset),
        ("people/early_farmers.toml", &people),
    ]);
    assert_eq!(codes(&report), vec!["E1001"]);
    assert!(report.diagnostics[0].line.is_some());

    let people = real("people/early_farmers.toml").replace("min_size = 30", "min_size = 55");
    let report = load_fixture(&[
        ("worldgen/river_valley.toml", &preset),
        ("people/early_farmers.toml", &people),
    ]);
    assert_eq!(codes(&report), vec!["E3001"]);

    // A resource grows as a plant or as animals, not both.
    let land = real("land/temperate_valley.toml").replace(
        "[resource.animal]\n# Animals per hectare",
        "[resource.plant]\nproduction_per_ha_yr = [0.0, 0.0, 0.0, 0.0, 0.0]\nloss_per_day = 0.1\n\
         season = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]\n\
         [resource.animal]\n# Animals per hectare",
    );
    assert_ne!(land, real("land/temperate_valley.toml"), "the edit applies");
    let report = load_fixture(&[
        ("worldgen/river_valley.toml", &preset),
        ("land/temperate_valley.toml", &land),
    ]);
    assert_eq!(codes(&report), vec!["E3001"]);
    assert!(report.diagnostics[0].message.contains("exactly one of"));
}

#[test]
fn the_fingerprint_covers_every_kind() {
    let preset = real_preset();
    let base = registry(load_fixture(&[("worldgen/river_valley.toml", &preset)]));

    // Key order, comments and `3` for `3.0` keep the meaning.
    let gather = real("activity/gather_plants.toml");
    let reordered = format!(
        "# moved par to the end\n{}par = 3\n",
        gather.replace("par = 3.0\n", "")
    );
    let same = registry(load_fixture(&[
        ("worldgen/river_valley.toml", &preset),
        ("activity/gather_plants.toml", &reordered),
    ]));
    assert_eq!(same.fingerprint, base.fingerprint);

    for (path, from, to) in [
        ("activity/gather_plants.toml", "par = 3.0", "par = 3.1"),
        ("people/early_farmers.toml", "w_play = 3.0", "w_play = 3.5"),
        (
            "land/temperate_valley.toml",
            "loss_per_day = 0.03",
            "loss_per_day = 0.04",
        ),
        ("names/valley_folk.toml", "\"Arden\"", "\"Arlen\""),
        (
            "good/meat.toml",
            "half_life_days = 3.0",
            "half_life_days = 4.0",
        ),
        ("crop/emmer.toml", "grow_days = 120", "grow_days = 125"),
    ] {
        let body = real(path).replace(from, to);
        assert_ne!(body, real(path), "{path}: the edit applies");
        let changed = registry(load_fixture(&[
            ("worldgen/river_valley.toml", &preset),
            (path, &body),
        ]));
        assert_ne!(changed.fingerprint, base.fingerprint, "{path}");
    }
}

#[test]
fn json_reports_are_machine_readable() {
    let report = load_fixture(&[("worldgen/elsewhere.toml", &real_preset())]);
    let json: serde_json::Value = serde_json::from_str(&report.to_json()).expect("valid JSON");
    assert_eq!(json["ok"], false);
    assert_eq!(json["diagnostics"][0]["code"], "E2004");
    assert_eq!(json["diagnostics"][0]["severity"], "error");

    let ok = load_fixture(&[("worldgen/river_valley.toml", &real_preset())]);
    let json: serde_json::Value = serde_json::from_str(&ok.to_json()).expect("valid JSON");
    assert_eq!(json["ok"], true);
    assert_eq!(json["people"], "core:people/early_farmers");
    assert_eq!(json["land"], "core:land/temperate_valley");
    assert!(json["activities"].as_array().is_some_and(|a| a.len() >= 7));
    assert!(
        json["goods"]
            .as_array()
            .is_some_and(|a| a.iter().any(|g| g == "core:good/firewood"))
    );
}
