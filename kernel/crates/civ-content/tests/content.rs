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

/// The real people, land, names, activity, good, crop, building, recipe, skill and regime files
/// (`(path in the pack, body)`), which every world needs; fixtures add presets and override files.
fn people_files() -> Vec<(String, String)> {
    let mut out = Vec::new();
    for dir in [
        "people",
        "land",
        "names",
        "activity",
        "good",
        "crop",
        "building",
        "recipe",
        "skill",
        "regime",
        "technique",
    ] {
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
kernel_content_api = 41
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
    // Today's work is gated by the founders' techniques (ADR-0008 §1), which every founder knows.
    let c = &reg.catalog;
    let emmer = c
        .technique_index("core:technique/emmer_growing")
        .expect("growing emmer");
    // Field work needs growing emmer, except dunging, which needs manuring (M3c slice V).
    let manuring = c
        .technique_index("core:technique/manuring")
        .expect("manuring");
    for a in c.activities.iter().filter(|a| a.task.is_some()) {
        let needs = if a.id == "core:activity/manure_field" {
            manuring
        } else {
            emmer
        };
        assert_eq!(
            c.technique_of(a),
            Some(needs),
            "{} needs its technique",
            a.id
        );
    }
    // A midden returns to the fields part of the nitrogen its people's grain took off them,
    // never more (research 03-04 §5.5: no nutrient-creating manure loops): a person's share of
    // the heap holds less than the grain they would eat in a year if it were all their food.
    let people = &reg.people.params;
    let crop = &c.crops[people.farm.crop];
    let grain_kg = people.household.daily_kcal_per_person * 365.0 / c.goods[crop.good].kcal_per_kg;
    assert!(
        people.midden.n_kg_per_person_year < grain_kg * crop.grain_n,
        "a midden's {} kg of nitrogen a person against {} kg in a year's grain",
        people.midden.n_kg_per_person_year,
        grain_kg * crop.grain_n
    );
    let grind = &c.activities[c.index_of("core:activity/grind_grain").expect("grinding")];
    assert_eq!(
        c.technique_of(grind).map(|t| c.techniques[t].id.as_str()),
        Some("core:technique/quern_grinding"),
        "a make activity needs its recipe's technique"
    );
    let hut = &c.buildings[c.building_index("core:building/hut").expect("the hut")];
    assert_eq!(
        hut.technique.map(|t| c.techniques[t].id.as_str()),
        Some("core:technique/roundhouse")
    );
    assert_eq!(
        c.work_age(emmer),
        Some(7.0),
        "children tend crops from seven"
    );
    let roundhouse = c
        .technique_index("core:technique/roundhouse")
        .expect("roundhouse");
    assert_eq!(c.work_age(roundhouse), Some(12.0), "building, from twelve");
    // Founders bring today's work, pottery and the oven among it; drying and the rotary quern are
    // found or brought in.
    let founders: Vec<&str> = reg
        .people
        .params
        .knowledge
        .founders
        .iter()
        .map(|&(t, _)| c.techniques[t].id.as_str())
        .collect();
    assert_eq!(founders.len(), 11);
    assert!(founders.contains(&"core:technique/pottery"));
    assert!(founders.contains(&"core:technique/oven_baking"));
    assert!(founders.contains(&"core:technique/manuring"));
    for later in ["core:technique/drying", "core:technique/rotary_quern"] {
        assert!(!founders.contains(&later), "{later}");
    }
    assert!(c.techniques.iter().all(|t| t.upbringing));
    // Drying answers food lost to spoiling; the rotary quern needs both shaping crafts.
    let drying = &c.techniques[c.technique_index("core:technique/drying").expect("drying")];
    assert_eq!(drying.answers_spoilage.len(), 2);
    let rotary = &c.techniques[c
        .technique_index("core:technique/rotary_quern")
        .expect("rotary quern")];
    assert_eq!(rotary.requires.len(), 1);
    assert_eq!(rotary.requires[0].len(), 2);
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
fn buildings_check_their_numbers_and_materials() {
    let preset = real_preset();
    let reg = registry(load_fixture(&[("worldgen/river_valley.toml", &preset)]));
    let hut = &reg.catalog.buildings[reg.people.params.build.programs[0]];
    assert_eq!(hut.id, "core:building/hut");
    let goods: Vec<&str> = hut
        .materials
        .iter()
        .map(|&g| reg.catalog.goods[g].id.as_str())
        .collect();
    assert_eq!(
        goods,
        vec!["core:good/timber", "core:good/timber", "core:good/thatch"]
    );
    assert_eq!(hut.pitch_centideg, 4500);
    let real_hut = real("building/hut.toml");
    for (from, to, code, needle) in [
        (
            "grammar = \"hut\"",
            "grammar = \"tower\"",
            "E3001",
            "unknown grammar",
        ),
        (
            "pitch_deg = 45.0 ",
            "pitch_deg = 30.0 ",
            "E3001",
            "`pitch_deg`",
        ),
        ("post_h = 2.0 ", "post_h = 0.0 ", "E3001", "`rules.post_h`"),
        (
            "roof_by_day = 304",
            "roof_by_day = 400",
            "E3001",
            "`roof_by_day`",
        ),
        (
            "thatch = \"core:good/thatch\"",
            "thatch = \"core:good/grain\"",
            "E3001",
            "must be a material",
        ),
        (
            "thatch = \"core:good/thatch\"",
            "thatch = \"core:good/slate\"",
            "E2006",
            "core:good/slate",
        ),
    ] {
        let body = real_hut.replace(from, to);
        assert_ne!(body, real_hut, "{needle}: the edit applies");
        let report = load_fixture(&[
            ("worldgen/river_valley.toml", &preset),
            ("building/hut.toml", &body),
        ]);
        assert!(
            codes(&report).contains(&code),
            "{needle}: {:?}",
            codes(&report)
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.code == code && d.message.contains(needle)),
            "{needle}"
        );
    }
    // People build programs that exist, among them at least one home the founders know how to
    // build; a horizon for storehouses is a number of days.
    let programs = "programs = [\"core:building/hut\", \"core:building/longhouse\", \"core:building/granary\", \"core:building/workshop\"]";
    for (to, code, needle) in [
        (
            "programs = [\"core:building/hut\", \"core:building/palace\"]",
            "E2006",
            "build.programs",
        ),
        (
            "programs = [\"core:building/granary\"]",
            "E3001",
            "at least one dwelling",
        ),
        (
            "programs = [\"core:building/longhouse\", \"core:building/granary\"]",
            "E3001",
            "founders know how to build",
        ),
        ("programs = []", "E3001", "at least one dwelling"),
    ] {
        let people = real("people/early_farmers.toml").replace(programs, to);
        assert_ne!(people, real("people/early_farmers.toml"), "{to}");
        let report = load_fixture(&[
            ("worldgen/river_valley.toml", &preset),
            ("people/early_farmers.toml", &people),
        ]);
        assert_eq!(codes(&report), vec![code], "{to}");
        assert!(report.diagnostics[0].message.contains(needle), "{to}");
    }
    let people = real("people/early_farmers.toml")
        .replace("store_horizon_days = 1095 ", "store_horizon_days = 0 ");
    assert_ne!(people, real("people/early_farmers.toml"));
    let report = load_fixture(&[
        ("worldgen/river_valley.toml", &preset),
        ("people/early_farmers.toml", &people),
    ]);
    assert_eq!(codes(&report), vec!["E3001"]);
    assert!(
        report.diagnostics[0]
            .message
            .contains("build.store_horizon_days")
    );
    let homes: Vec<&str> = reg
        .people
        .params
        .build
        .programs
        .iter()
        .map(|&h| reg.catalog.buildings[h].id.as_str())
        .collect();
    assert_eq!(
        homes,
        [
            "core:building/hut",
            "core:building/longhouse",
            "core:building/granary",
            "core:building/workshop"
        ]
    );
    // Frame programs: a longhouse, a raised granary and a workshop, their rules from `[frame]`.
    let c = &reg.catalog;
    for (id, use_, raised) in [
        (
            "core:building/longhouse",
            civ_land::PlotUse::Dwelling,
            false,
        ),
        ("core:building/granary", civ_land::PlotUse::Store, true),
        ("core:building/workshop", civ_land::PlotUse::Work, false),
    ] {
        let b = &c.buildings[c.building_index(id).expect(id)];
        assert_eq!(b.grammar(), civ_grammar::Grammar::Frame, "{id}");
        assert_eq!(b.use_, use_, "{id}");
        assert_eq!(b.materials.len(), civ_grammar::frame_materials::COUNT);
        assert_eq!(
            b.technique.map(|t| c.techniques[t].id.as_str()),
            Some("core:technique/jointed_frame")
        );
        let civ_grammar::ProgramRules::Frame(r) = &b.rules else {
            panic!("{id} has frame rules")
        };
        assert_eq!(r.floor_raise_cm.0 > 0, raised, "{id}");
        // The least building its rules allow expands, built to its usual height and pitch.
        use civ_grammar::frame_params as fp;
        let mut params = [0; civ_grammar::PARAMS];
        params[fp::EAVE_CM] = b.eave_cm;
        params[fp::PITCH_CENTIDEG] = b.pitch_centideg;
        params[fp::BAYS] = r.bays.0;
        params[fp::DOOR] = fp::door(fp::SIDE_RIGHT, 0);
        params[fp::OVERHANG_CM] = r.overhang_cm.0;
        params[fp::FLOOR_RAISE_CM] = r.floor_raise_cm.0;
        params[fp::JOIST_CM] = if raised { r.joist_cm.0 } else { 0 };
        params[fp::POST_CM] = r.post_cm.0;
        params[fp::WALL_CM] = r.wall_cm.0;
        let spec = civ_grammar::BuildingSpec {
            program: id.to_owned(),
            version: civ_grammar::FRAME_VERSION,
            footprint: civ_grammar::Footprint::Rect {
                x: 100_000,
                y: 100_000,
                length: r.bays.0 * r.bay_cm.0,
                width: r.width_cm.0,
                angle: 0,
            },
            storeys: r.storeys.0,
            params,
            materials: b.materials.iter().map(|&g| c.goods[g].id.clone()).collect(),
            style_seed: 0,
        };
        let e = civ_grammar::expand(&spec, &b.rules).unwrap_or_else(|e| panic!("{id}: {e}"));
        assert!(e.total_labour_h() > 0.0, "{id}");
    }
    assert_eq!(hut.use_, civ_land::PlotUse::Dwelling);
    let real_longhouse = real("building/longhouse.toml");
    for (from, to, needle) in [
        ("use = \"dwelling\"", "use = \"palace\"", "unknown use"),
        ("ground = \"living\"", "ground = \"barn\"", "`frame.ground`"),
        ("bays = [1, 8]", "bays = [1, 12]", "`frame.bays`"),
        ("storeys = [1, 2]", "storeys = [1, 3]", "`frame.storeys`"),
        (
            "covering = \"core:good/thatch\"",
            "thatch = \"core:good/thatch\"",
            "`materials.covering` is missing",
        ),
        (
            "covering = [0.06, 0.15]",
            "covering = [0.06, 1.5]",
            "`upkeep.covering`",
        ),
        (
            "grammar = \"frame\"",
            "grammar = \"hut\"",
            "needs `[rules]`",
        ),
        ("decking_cm = 4 ", "decking_cm = 0 ", "`frame.decking_cm`"),
    ] {
        let body = real_longhouse.replace(from, to);
        assert_ne!(body, real_longhouse, "{needle}: the edit applies");
        let report = load_fixture(&[
            ("worldgen/river_valley.toml", &preset),
            ("building/longhouse.toml", &body),
        ]);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.message.contains(needle)),
            "{needle}: {:?}",
            report.diagnostics
        );
    }
    // A granary built to a raise and an overhang that put its ladder beyond the roof's edge
    // gives nothing to build, and no bay may be as short as a post is thick.
    let real_granary = real("building/granary.toml");
    for (edits, needle) in [
        (
            vec![
                ("overhang_cm = 60\n", "overhang_cm = 40\n"),
                ("floor_raise_cm = 80 ", "floor_raise_cm = 120 "),
                ("overhang_cm = [60, 120]", "overhang_cm = [40, 120]"),
            ],
            "nothing can be built to its design",
        ),
        (
            vec![("post_cm = [12, 30]", "post_cm = [12, 250]")],
            "`frame.bay_cm` must start above",
        ),
    ] {
        let mut body = real_granary.clone();
        for (from, to) in edits {
            assert!(body.contains(from), "{needle}: {from}");
            body = body.replace(from, to);
        }
        let report = load_fixture(&[
            ("worldgen/river_valley.toml", &preset),
            ("building/granary.toml", &body),
        ]);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.code == "E3001" && d.message.contains(needle)),
            "{needle}: {:?}",
            report.diagnostics
        );
    }
    // A material is never eaten, cooked, kept back or shared, and a roof never speeds spoiling.
    let timber = real("good/timber.toml");
    for (path, body, needle) in [
        (
            "good/timber.toml",
            timber.replace("shared = false", "shared = true"),
            "`shared` must be false",
        ),
        (
            "good/timber.toml",
            timber.replace("kcal_per_kg = 0.0", "kcal_per_kg = 10.0"),
            "not eaten",
        ),
        (
            "good/grain.toml",
            real("good/grain.toml").replace(
                "sheltered_half_life_days = 4932.0",
                "sheltered_half_life_days = 100.0",
            ),
            "never makes a good spoil faster",
        ),
        // Timber's strength (ADR-0009 §5): positive, its creep and its share kept in range.
        (
            "good/timber.toml",
            timber.replace("bending_mpa = 57.0", "bending_mpa = 0.0"),
            "`timber.bending_mpa`",
        ),
        (
            "good/timber.toml",
            timber.replace("sustained = 0.6", "sustained = 1.5"),
            "`timber.sustained`",
        ),
        (
            "good/grain.toml",
            format!(
                "{}\n[timber]\nbending_mpa = 1.0\ncompression_mpa = 1.0\nstiffness_gpa = 1.0\n\
                 creep = 1.0\nsustained = 0.5\n",
                real("good/grain.toml")
            ),
            "only a material takes a `[timber]` table",
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
            "eaten = \"never\"",
            "eaten = \"raw\"",
            "only food is eaten",
        ),
        (
            "good/meat.toml",
            "eaten = \"cooked\"",
            "eaten = \"boiled\"",
            "unknown `eaten`",
        ),
        (
            "good/sickle.toml",
            "life_h = 100.0",
            "life_h = 0.0",
            "`tool.life_h` must be positive",
        ),
        (
            "good/sickle.toml",
            "[tool]\nlife_h = 100.0\nper_worker = 1.0\nfixed = false\n",
            "",
            "a tool needs its `[tool]` table",
        ),
        (
            "good/seed_grain.toml",
            "reserve_for = \"core:good/grain\"",
            "reserve_for = \"core:good/firewood\"",
            "must name another food",
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
        (gather.replace("rate = 1.0", "rate = 0.0"), "`rate`"),
        (
            real("activity/rest.toml").replace(
                "behavior = \"rest\"",
                "behavior = \"rest\"\nrecipe = \"core:recipe/grind_grain\"",
            ),
            "only `make` activities take a `recipe`",
        ),
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

    // The month's storm on roofs (ADR-0012 §5) stays in range, and so does the snow they keep.
    let land = real("land/temperate_valley.toml")
        .replace("storm_median_kpa = 0.25", "storm_median_kpa = -1.0");
    assert_ne!(land, real("land/temperate_valley.toml"), "the edit applies");
    let report = load_fixture(&[
        ("worldgen/river_valley.toml", &preset),
        ("land/temperate_valley.toml", &land),
    ]);
    assert_eq!(codes(&report), vec!["E3001"]);
    assert!(
        report.diagnostics[0]
            .message
            .contains("`weather.storm_median_kpa`")
    );
    let land = real("land/temperate_valley.toml")
        .replace("roof_snow_shed_deg = 60.0", "roof_snow_shed_deg = 20.0");
    assert_ne!(land, real("land/temperate_valley.toml"), "the edit applies");
    let report = load_fixture(&[
        ("worldgen/river_valley.toml", &preset),
        ("land/temperate_valley.toml", &land),
    ]);
    assert_eq!(codes(&report), vec!["E3001"]);
    assert!(
        report.diagnostics[0]
            .message
            .contains("`weather.roof_snow_shed_deg` must be above"),
        "{}",
        report.diagnostics[0].message
    );

    // Builders never build weaker for what they have seen (ADR-0009 §6).
    let people = real("people/early_farmers.toml").replace("most = 2.0 ", "most = 0.5 ");
    assert_ne!(
        people,
        real("people/early_farmers.toml"),
        "the edit applies"
    );
    let report = load_fixture(&[
        ("worldgen/river_valley.toml", &preset),
        ("people/early_farmers.toml", &people),
    ]);
    assert_eq!(codes(&report), vec!["E3001"]);
    assert!(
        report.diagnostics[0]
            .message
            .contains("`build.caution.most`")
    );
    let people = real("people/early_farmers.toml")
        .replace("half_life_years = 8.0 ", "half_life_years = 0.0 ");
    assert_ne!(
        people,
        real("people/early_farmers.toml"),
        "the edit applies"
    );
    let report = load_fixture(&[
        ("worldgen/river_valley.toml", &preset),
        ("people/early_farmers.toml", &people),
    ]);
    assert_eq!(codes(&report), vec!["E3001"]);
    assert!(
        report.diagnostics[0]
            .message
            .contains("`build.caution.half_life_years`")
    );

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
        ("crop/emmer.toml", "ky = 1.15", "ky = 1.2"),
        ("building/hut.toml", "post_kg = 34.0", "post_kg = 35.0"),
        (
            "technique/quern_grinding.toml",
            "learn_h = 20.0",
            "learn_h = 21.0",
        ),
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

#[test]
fn recipes_and_skills_check_their_numbers_and_references() {
    let preset = real_preset();
    let grind = real("recipe/grind_grain.toml");
    let milling = real("skill/milling.toml");
    for (path, body, code, needle) in [
        (
            "recipe/grind_grain.toml",
            grind.replace("core:good/flour", "core:good/dust"),
            "E2006",
            "`outputs` refers to `core:good/dust`",
        ),
        (
            "recipe/grind_grain.toml",
            grind.replace("core:skill/milling", "core:skill/juggling"),
            "E2006",
            "`skill` refers to `core:skill/juggling`",
        ),
        (
            "recipe/grind_grain.toml",
            grind.replace(
                "tools = [\"core:good/quern\"]",
                "tools = [\"core:good/stone\"]",
            ),
            "E3001",
            "`tools` must name tools",
        ),
        (
            "recipe/grind_grain.toml",
            grind.replace(
                "outputs = [{ good = \"core:good/flour\", amount = 0.98 }]",
                "outputs = []",
            ),
            "E3001",
            "`outputs` must not be empty",
        ),
        (
            "recipe/grind_grain.toml",
            grind.replace("amount = 0.98", "amount = -1.0"),
            "E3001",
            "amounts must be positive",
        ),
        (
            "skill/milling.toml",
            milling.replace("t80_h = 150.0", "t80_h = 0.0"),
            "E3001",
            "`t80_h` must be positive",
        ),
        (
            "skill/milling.toml",
            milling.replace("founder_level = [0.4, 0.9]", "founder_level = [0.9, 0.4]"),
            "E3001",
            "`founder_level`",
        ),
        (
            "activity/grind_grain.toml",
            real("activity/grind_grain.toml")
                .replace("core:recipe/grind_grain", "core:recipe/spin_gold"),
            "E2006",
            "`recipe` refers to `core:recipe/spin_gold`",
        ),
        (
            "activity/reap.toml",
            real("activity/reap.toml").replace("core:good/sickle", "core:good/grain"),
            "E3001",
            "`tools` must name tools",
        ),
    ] {
        assert_ne!(body, real(path), "{needle}: the edit applies");
        let report = load_fixture(&[("worldgen/river_valley.toml", &preset), (path, &body)]);
        assert!(
            codes(&report).contains(&code),
            "{needle}: {:?}",
            codes(&report)
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.message.contains(needle)),
            "{needle}: {:#?}",
            report
                .diagnostics
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        );
    }
    // The real recipes and skills compile, with their goods, tools and skills resolved.
    let reg = registry(load(&repo_content()));
    let grind = reg
        .catalog
        .recipes
        .iter()
        .find(|r| r.id == "core:recipe/grind_grain")
        .expect("the quern recipe");
    let quern = reg
        .catalog
        .good_index("core:good/quern")
        .expect("the quern");
    assert_eq!(grind.tools, vec![quern]);
    assert_eq!(
        grind.skill.map(|k| reg.catalog.skills[k].id.as_str()),
        Some("core:skill/milling")
    );
    let seed = reg
        .catalog
        .good_index("core:good/seed_grain")
        .expect("seed");
    let grain = reg.catalog.good_index("core:good/grain").expect("grain");
    assert_eq!(reg.catalog.goods[seed].reserve_for, Some(grain));
    let reap = reg.catalog.index_of("core:activity/reap").expect("reap");
    let sickle = reg.catalog.good_index("core:good/sickle").expect("sickle");
    assert_eq!(reg.catalog.activities[reap].tools, vec![sickle]);
}

#[test]
fn techniques_check_their_references_gates_and_routes() {
    let preset = real_preset();
    let grind = real("recipe/grind_grain.toml");
    let knapping = real("technique/knapping.toml");
    let people = real("people/early_farmers.toml");
    let lone = knapping
        .replace("core:technique/knapping", "core:technique/weaving")
        .replace("Knapping sickle blades", "Weaving");
    for (files, code, needle) in [
        (
            vec![(
                "recipe/grind_grain.toml",
                grind.replace("core:technique/quern_grinding", "core:technique/magic"),
            )],
            "E2006",
            "`technique` refers to `core:technique/magic`",
        ),
        (
            vec![("technique/weaving.toml", lone.clone())],
            "E3005",
            "`core:technique/weaving` gates no work",
        ),
        (
            vec![
                (
                    "technique/knapping.toml",
                    knapping.replace(
                        "requires = []",
                        "requires = [[\"core:technique/stone_shaping\"]]",
                    ),
                ),
                (
                    "technique/stone_shaping.toml",
                    real("technique/stone_shaping.toml").replace(
                        "requires = []",
                        "requires = [[\"core:technique/knapping\"]]",
                    ),
                ),
            ],
            "E3006",
            "lead back to it",
        ),
        (
            vec![(
                "technique/knapping.toml",
                knapping.replace(
                    "requires = []",
                    "requires = [[\"core:technique/knapping\"]]",
                ),
            )],
            "E3001",
            "cannot require itself",
        ),
        (
            vec![(
                "technique/knapping.toml",
                knapping.replace("requires = []", "requires = [[]]"),
            )],
            "E3001",
            "every route in `requires`",
        ),
        (
            vec![(
                "technique/knapping.toml",
                knapping.replace("learn_h = 200.0", "learn_h = 0.0"),
            )],
            "E3001",
            "`learn_h` must be positive",
        ),
        (
            vec![(
                "technique/knapping.toml",
                knapping.replace("tried_in = []", "tried_in = [\"core:activity/juggle\"]"),
            )],
            "E2006",
            "`tried_in` refers to `core:activity/juggle`",
        ),
        (
            vec![(
                "technique/knapping.toml",
                knapping.replace("needs = []", "needs = [\"core:good/gold\"]"),
            )],
            "E2006",
            "`needs` refers to `core:good/gold`",
        ),
        (
            vec![(
                "technique/knapping.toml",
                knapping.replace("core:skill/knapping", "core:skill/juggling"),
            )],
            "E2006",
            "`domain` refers to `core:skill/juggling`",
        ),
        (
            vec![(
                "people/early_farmers.toml",
                people.replace(
                    "core:technique/pounding\", share = 1.0",
                    "core:technique/pounding\", share = 1.5",
                ),
            )],
            "E3001",
            "share of `core:technique/pounding` must be between 0 and 1",
        ),
        (
            vec![(
                "people/early_farmers.toml",
                people.replace("core:technique/pounding", "core:technique/mystery"),
            )],
            "E2006",
            "`knowledge.founders` refers to `core:technique/mystery`",
        ),
        (
            vec![(
                "activity/grind_grain.toml",
                real("activity/grind_grain.toml").replace(
                    "technique = \"\"",
                    "technique = \"core:technique/pounding\"",
                ),
            )],
            "E3001",
            "its recipe's: `technique` must be empty",
        ),
        (
            // Meal ground from bread (at either quern), and bread baked from meal: neither can
            // ever be worked.
            vec![
                (
                    "recipe/grind_grain.toml",
                    grind.replace(
                        "inputs = [{ good = \"core:good/grain\"",
                        "inputs = [{ good = \"core:good/bread\"",
                    ),
                ),
                (
                    "recipe/grind_grain_rotary.toml",
                    real("recipe/grind_grain_rotary.toml").replace(
                        "inputs = [{ good = \"core:good/grain\"",
                        "inputs = [{ good = \"core:good/bread\"",
                    ),
                ),
            ],
            "E3007",
            "recipe `core:recipe/grind_grain` can never be worked",
        ),
    ] {
        for (path, body) in &files {
            if repo_content().join("core").join(path).exists() {
                assert_ne!(*body, real(path), "{needle}: the edit to {path} applies");
            }
        }
        let mut all: Vec<(&str, &str)> = vec![("worldgen/river_valley.toml", &preset)];
        all.extend(files.iter().map(|(p, b)| (*p, b.as_str())));
        let report = load_fixture(&all);
        assert!(
            codes(&report).contains(&code),
            "{needle}: {:?}",
            codes(&report)
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.message.contains(needle)),
            "{needle}: {:#?}",
            report
                .diagnostics
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn pots_hold_goods_and_digging_needs_deposits_of_what_it_digs() {
    let preset = real_preset();
    // The real pot keeps 35 kg each, and clay is dug, not gathered.
    let reg = registry(load_fixture(&[("worldgen/river_valley.toml", &preset)]));
    let c = &reg.catalog;
    let pot = &c.goods[c
        .goods
        .iter()
        .position(|g| g.id == "core:good/pot")
        .expect("pot")];
    assert_eq!(pot.store.as_ref().map(|s| s.keeps_kg), Some(35.0));
    let dig = c
        .activities
        .iter()
        .find(|a| a.id == "core:activity/dig_clay")
        .expect("digging clay");
    assert_eq!(
        dig.digs.map(|g| c.goods[g].id.as_str()),
        Some("core:good/clay")
    );

    // A store needs its table, and only a store takes one.
    let bare = real("good/pot.toml").replace("[store]\nkeeps_kg = 35.0\n", "");
    assert_ne!(bare, real("good/pot.toml"), "the edit applies");
    let report = load_fixture(&[
        ("worldgen/river_valley.toml", &preset),
        ("good/pot.toml", &bare),
    ]);
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| d.code == "E3001" && d.message.contains("`[store]`"))
    );
    let stored = format!("{}\n[store]\nkeeps_kg = 5.0\n", real("good/clay.toml"));
    let report = load_fixture(&[
        ("worldgen/river_valley.toml", &preset),
        ("good/clay.toml", &stored),
    ]);
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| d.message.contains("only a store"))
    );

    // Digging stone, which the land lays down, is allowed; digging grain, which it does not, is not.
    let stone = real("activity/dig_clay.toml").replace("core:good/clay", "core:good/stone");
    let report = load_fixture(&[
        ("worldgen/river_valley.toml", &preset),
        (
            "activity/dig_stone.toml",
            &stone.replace("dig_clay", "dig_stone"),
        ),
    ]);
    assert_eq!(codes(&report), Vec::<&str>::new());
    let grain = real("activity/dig_clay.toml")
        .replace("core:good/clay", "core:good/grain")
        .replace("dig_clay", "dig_grain");
    let report = load_fixture(&[
        ("worldgen/river_valley.toml", &preset),
        ("activity/dig_grain.toml", &grain),
    ]);
    assert_eq!(codes(&report), vec!["E3001"]);
    assert!(
        report.diagnostics[0]
            .message
            .contains("lays no deposits of")
    );
}

#[test]
fn deposits_say_how_hard_their_bodies_are_to_dig_and_what_a_working_is_called() {
    let preset = real_preset();
    let reg = registry(load_fixture(&[("worldgen/river_valley.toml", &preset)]));
    let (c, land) = (&reg.catalog, &reg.land.params);
    let rule = |id: &str| {
        let g = c.goods.iter().position(|g| g.id == id).expect(id);
        land.deposits.iter().find(|r| r.good == g).expect("a rule")
    };
    // Clay is dug as earth; stone is quarried and a flint bed dug, both slower than earth.
    assert_eq!(
        (
            rule("core:good/clay").dig_h_per_m3,
            rule("core:good/clay").working.as_str()
        ),
        (0.0, "pit")
    );
    assert_eq!(
        (
            rule("core:good/stone").dig_h_per_m3,
            rule("core:good/stone").working.as_str()
        ),
        (12.0, "quarry")
    );
    assert_eq!(rule("core:good/toolstone").dig_h_per_m3, 12.0);
    for (activity, good) in [
        ("core:activity/quarry_stone", "core:good/stone"),
        ("core:activity/dig_flint", "core:good/toolstone"),
    ] {
        let a = c
            .activities
            .iter()
            .find(|a| a.id == activity)
            .expect(activity);
        assert_eq!(a.digs.map(|g| c.goods[g].id.as_str()), Some(good));
    }

    // Out of range, or no name: refused.
    let land_file = real("land/temperate_valley.toml");
    for (from, to, field) in [
        (
            "dig_h_per_m3 = 12.0\nworking = \"quarry\"",
            "dig_h_per_m3 = -1.0\nworking = \"quarry\"",
            "`dig_h_per_m3`",
        ),
        ("working = \"quarry\"", "working = \"\"", "`working`"),
    ] {
        let broken = land_file.replacen(from, to, 1);
        assert_ne!(broken, land_file, "the edit applies");
        let report = load_fixture(&[
            ("worldgen/river_valley.toml", &preset),
            ("land/temperate_valley.toml", &broken),
        ]);
        assert!(
            report.diagnostics.iter().any(|d| d.message.contains(field)),
            "{field}: {:?}",
            report.diagnostics
        );
    }
}

#[test]
fn ties_name_the_acts_the_engine_records_and_say_what_each_writes() {
    let preset = real_preset();
    let check = |from: &str, to: &str, says: &str| {
        let people = real("people/early_farmers.toml").replace(from, to);
        let report = load_fixture(&[
            ("worldgen/river_valley.toml", &preset),
            ("people/early_farmers.toml", &people),
        ]);
        let found = codes(&report);
        assert!(
            !found.is_empty() && found.iter().all(|&c| c == "E3001"),
            "{to}: {found:?}"
        );
        assert!(
            report.diagnostics.iter().any(|d| d.message.contains(says)),
            "{to}: {:?}",
            report.diagnostics
        );
    };
    // An act the engine does not record, which also leaves one unsaid.
    check(
        "act = \"traded\"",
        "act = \"bartered\"",
        "does not record: `bartered`",
    );
    check(
        "act = \"traded\"",
        "act = \"bartered\"",
        "what act `traded` writes",
    );
    check(
        "domain = \"provision\"",
        "domain = \"bounty\"",
        "unknown domain `bounty`",
    );
    check("room = 48", "room = 0", "`ties.room`");
    check(
        "act = \"hearth\", familiarity = 0.1",
        "act = \"hearth\", familiarity = 1.5",
        "`familiarity` must be between 0 and 1",
    );
}
