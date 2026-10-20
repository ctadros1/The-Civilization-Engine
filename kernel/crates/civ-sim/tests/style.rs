//! Style v0 (M3b slice R; research 11-02): each band builds its own way and each home to its
//! household's taste, and families sent to a village bring theirs; once a year taste moves toward the settlement's admired new buildings, and
//! a household's next building follows the one that moved it most; a building that gave way is
//! admired by nobody; saves keep it all.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::params::{BuildingDef, Taste};
use civ_agents::style;
use civ_content::ContentRegistry;
use civ_core::{PermanentId, SimTime};
use civ_grammar::hut_params;
use civ_land::{Building, BuildingState, GroupState};
use civ_sim::{NewWorld, Sim, persist};
use commons_persist::{SaveDir, SaveKind};

/// The core content, but nothing new by chance: a building is built to its builders' taste.
/// The traits a hut shows: the roof's three; a hut has no front to move a taste (M8d slice DD).
const ROOF: [bool; Taste::TRAITS] = [true, true, true, false, false, false, false];

fn content() -> &'static ContentRegistry {
    static CONTENT: OnceLock<ContentRegistry> = OnceLock::new();
    CONTENT.get_or_init(|| {
        let root = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
            .expect("content/ is above the crate");
        let mut c = civ_content::load(&root)
            .registry
            .expect("the core content loads");
        c.people.params.style.innovation = 0.0;
        c
    })
}

fn world(seed: u64) -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Style".to_owned(),
            seed,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 512,
            band_size: 0,
            neighbours: Vec::new(),
            neighbours_known: false,
            regime_id: String::new(),
        },
        content(),
        [7; 16],
    )
    .expect("generates")
}

fn def<'a>(sim: &'a Sim, b: &Building) -> &'a BuildingDef {
    let catalog = &sim.rules().catalog;
    &catalog.buildings[catalog
        .building_index(&b.spec.program)
        .expect("its program")]
}

/// Building `b`'s traits, in whole units: pitch, eaves, overhang and the front's four.
fn traits(sim: &Sim, b: &Building) -> [i32; Taste::TRAITS] {
    let t = style::traits_of(&b.spec, def(sim, b)).traits();
    t.map(|v| v.round() as i32)
}

fn taste(sim: &Sim, household: PermanentId) -> Taste {
    sim.people()
        .household(household)
        .expect("the household")
        .taste
}

#[test]
fn each_band_builds_its_own_way_and_each_home_to_its_household_s_taste() {
    let mut sims = [world(3), world(4)];
    let mut means = Vec::new();
    for sim in &mut sims {
        sim.advance_minutes(30 * 24 * 60).expect("advances");
        assert!(!sim.land().buildings.is_empty(), "homes are begun");
        for b in &sim.land().buildings {
            let held = style::held_to(&taste(sim, b.household), def(sim, b));
            assert_eq!(traits(sim, b), held, "{}", b.id);
        }
        // A band's households lie close to each other: their own spread, not their band's.
        let pitches: Vec<f32> = sim
            .people()
            .households
            .iter()
            .map(|(_, h)| h.taste.pitch_centideg)
            .collect();
        let (lo, hi) = pitches
            .iter()
            .fold((f32::MAX, f32::MIN), |(lo, hi), &p| (lo.min(p), hi.max(p)));
        assert!(hi - lo < 700.0, "{lo} to {hi}");
        means.push(pitches.iter().sum::<f32>() / pitches.len() as f32);
    }
    assert_ne!(means[0], means[1], "two bands, two ways of building");
    // Families the observer sends together join the village but bring their own way of building.
    let sim = &mut sims[0];
    let village = sim.land().settlements[0].clone();
    let sent = sim
        .spawn_families(village.hearth_m, 3)
        .expect("the families arrive");
    assert!(sent.len() >= 2, "{}", sent.len());
    let pitches: Vec<f32> = sent
        .iter()
        .map(|s| {
            assert_eq!(s.settlement, village.id, "they join the village");
            taste(sim, s.household).pitch_centideg
        })
        .collect();
    let (lo, hi) = pitches
        .iter()
        .fold((f32::MAX, f32::MIN), |(lo, hi), &p| (lo.min(p), hi.max(p)));
    assert!(hi - lo < 700.0, "{lo} to {hi}");
    let theirs = pitches.iter().sum::<f32>() / pitches.len() as f32;
    assert_ne!(theirs, means[0], "their own way, not the village's");
}

#[test]
fn taste_moves_toward_an_admired_new_building_and_the_next_home_follows_it() {
    let mut sim = world(3);
    // Until every household has a home finished.
    for _ in 0..180 {
        sim.advance_minutes(24 * 60).expect("advances");
        let all = sim.people().households.iter().all(|(_, h)| {
            sim.land()
                .buildings
                .iter()
                .any(|b| b.household == h.id && b.finished())
        });
        if all {
            break;
        }
    }
    let now = sim.now();
    let ids: Vec<PermanentId> = {
        let mut v: Vec<PermanentId> = sim.people().households.iter().map(|(_, h)| h.id).collect();
        v.sort();
        v
    };
    assert!(ids.len() >= 3, "a village");
    let (rich, other) = (ids[0], ids[1]);
    // The richest household, whose home was finished a month ago steep and tall; every other
    // building was finished long since, so it alone is new.
    let grain = sim
        .rules()
        .catalog
        .good_index("core:good/grain")
        .expect("grain");
    for (_, h) in sim.people_mut_for_tests().households.iter_mut() {
        if h.id == rich {
            h.stores[grain] += 5_000.0;
        }
    }
    let month_ago = SimTime::from_minutes(now.minutes() - 30 * 24 * 60);
    let long_ago = SimTime::from_minutes(now.minutes() - 400 * 24 * 60);
    let mut admired = None;
    for b in sim.land_mut_for_tests().buildings.iter_mut() {
        if b.household == rich && b.finished() && admired.is_none() {
            b.spec.params[hut_params::PITCH_CENTIDEG] = 5_500;
            b.spec.params[hut_params::EAVE_CM] = 240;
            b.stage_since = month_ago;
            admired = Some(b.id);
        } else {
            b.stage_since = long_ago;
        }
    }
    let admired = admired.expect("the rich household's home");
    let model = {
        let b = sim
            .land()
            .buildings
            .iter()
            .find(|b| b.id == admired)
            .expect("it");
        style::traits_of(&b.spec, def(&sim, b))
    };
    let before: Vec<(PermanentId, Taste)> = ids.iter().map(|&h| (h, taste(&sim, h))).collect();
    let rules = sim.rules().clone();
    let review = |sim: &mut Sim| {
        let (land, now) = (sim.land().clone(), sim.now());
        sim.people_mut_for_tests().review_tastes(
            &rules.catalog,
            &rules.people,
            &rules.land,
            &land,
            now,
        );
    };
    review(&mut sim);
    // Everyone else moved alpha of the way toward it, as its owner stands richest; its owner did
    // not move.
    let alpha = rules.people.style.alpha;
    for &(h, was) in &before {
        let now_taste = taste(&sim, h);
        if h == rich {
            assert_eq!(now_taste, was);
            continue;
        }
        let expected = style::moved_where(&was, &model, alpha, ROOF);
        for (a, b) in now_taste.traits().iter().zip(expected.traits()) {
            assert!(
                (a - b).abs() < 1e-3,
                "{h}: {now_taste:?} against {expected:?}"
            );
        }
        assert_eq!(
            sim.people().household(h).expect("it").admired,
            Some(admired)
        );
    }
    // A building that has given way is admired by nobody.
    let moved: Vec<(PermanentId, Taste)> = ids.iter().map(|&h| (h, taste(&sim, h))).collect();
    let keep = sim
        .land()
        .buildings
        .iter()
        .find(|b| b.id == admired)
        .cloned()
        .expect("it");
    let mut failed = keep.clone();
    failed.condition.first_mut().expect("its parts").state = GroupState::Failed;
    for b in sim.land_mut_for_tests().buildings.iter_mut() {
        if b.id == admired {
            *b = failed.clone();
        }
    }
    review(&mut sim);
    for &(h, was) in &moved {
        assert_eq!(taste(&sim, h), was, "{h} admired a failed building");
    }
    for b in sim.land_mut_for_tests().buildings.iter_mut() {
        if b.id == admired {
            *b = keep.clone();
        }
    }
    // Another household's home falls to ruin: its new home follows the building that moved its
    // taste, and is built to its taste as it now is.
    let old = sim
        .land()
        .buildings
        .iter()
        .find(|b| b.household == other && b.finished())
        .map(|b| b.id)
        .expect("its home");
    for b in sim.land_mut_for_tests().buildings.iter_mut() {
        if b.id == old {
            b.state = BuildingState::Ruin;
        }
    }
    let (now, land) = (sim.now(), sim.land().clone());
    sim.people_mut_for_tests()
        .buildings_changed(now, &land, &rules.catalog, &rules.people, other);
    let mut new_home = None;
    for _ in 0..120 {
        sim.advance_minutes(24 * 60).expect("advances");
        new_home = sim
            .land()
            .buildings
            .iter()
            .find(|b| b.household == other && b.id != old)
            .cloned();
        if new_home.is_some() {
            break;
        }
    }
    let new_home = new_home.expect("a new home is begun");
    assert_eq!(new_home.style_from, Some(admired));
    // The observer reads how it was built and what it followed, and what its household admires.
    let owner = civ_sim::frames::people::eldest_name(&sim, rich).expect("its eldest");
    let words =
        civ_sim::frames::buildings::style_words(&sim, &new_home, Some(def(&sim, &new_home)));
    assert!(words.starts_with("roof pitched "), "{words}");
    assert!(words.contains(&format!("after {owner}'s ")), "{words}");
    let taste_words = civ_sim::frames::people::taste_words(
        &sim,
        sim.people().household(other).expect("the household"),
    );
    assert!(
        taste_words.contains(&format!("; admiring {owner}'s ")),
        "{taste_words}"
    );
    let held = style::held_to(&taste(&sim, other), def(&sim, &new_home));
    assert_eq!(traits(&sim, &new_home), held);
    // A save keeps every taste, what each household admired and what each building followed.
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        SaveDir::create(dir.path().join("saves"), civ_schema::SAVE_EXTENSION).expect("save dir");
    let saved = persist::save(&mut sim, &saves, SaveKind::Manual, "style").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    for (_, h) in sim.people().households.iter() {
        let l = loaded.people().household(h.id).expect("loaded");
        assert_eq!((l.taste, l.admired), (h.taste, h.admired));
    }
    for b in &sim.land().buildings {
        let l = loaded
            .land()
            .buildings
            .iter()
            .find(|x| x.id == b.id)
            .expect("loaded");
        assert_eq!(l.style_from, b.style_from);
    }
}

#[test]
fn a_front_is_built_to_taste_and_moves_taste_only_as_far_as_its_viewers_could_build_it() {
    use civ_agents::build::{FireDesign, Shape, design_cautious};
    use civ_grammar::frame_params as fp;
    let mut sim = world(3);
    for _ in 0..180 {
        sim.advance_minutes(24 * 60).expect("advances");
        let all = sim.people().households.iter().all(|(_, h)| {
            sim.land()
                .buildings
                .iter()
                .any(|b| b.household == h.id && b.finished())
        });
        if all {
            break;
        }
    }
    let mut ids: Vec<PermanentId> = sim.people().households.iter().map(|(_, h)| h.id).collect();
    ids.sort();
    assert!(ids.len() >= 3, "a village");
    let (owner, learned, unlearned) = (ids[0], ids[1], ids[2]);
    let catalog = sim.rules().catalog.clone();
    let longhouse = &catalog.buildings[catalog
        .building_index("core:building/longhouse")
        .expect("the longhouse")];
    let home = sim
        .land()
        .buildings
        .iter()
        .find(|b| b.household == owner && b.finished())
        .map(|b| civ_agents::build::centre_m(&b.spec))
        .expect("the owner's home");
    // A frame designed to a front of 30 % open walls, 1.5 h of ornament a square metre, lime
    // wash and symmetry: three bays put its door in the middle; two set out no symmetry, and
    // builders who cannot burn lime daub it instead.
    let front = [4_800, 190, 50, 300, 15, 2, 1];
    let design = |bays: i32, lime: bool| {
        design_cautious(
            longhouse,
            &catalog.goods,
            Shape::Bays {
                bays,
                storeys: 1,
                lofts: 0,
            },
            (home.0 + 15.0, home.1),
            Some(home),
            1.0,
            Some(front),
            Some(FireDesign {
                knows_lime: lime,
                ..FireDesign::default()
            }),
        )
        .expect("designed")
    };
    let spec = design(3, true);
    assert_eq!(
        [
            spec.params[fp::OPENINGS],
            spec.params[fp::ORNAMENT],
            spec.params[fp::FINISH],
            spec.params[fp::SYMMETRY]
        ],
        [300, 15, 2, 1]
    );
    let plain = design(2, false);
    assert_eq!(
        (plain.params[fp::FINISH], plain.params[fp::SYMMETRY]),
        (1, 0)
    );
    // It is the year's one new building.
    let long_ago = SimTime::from_minutes(sim.now().minutes() - 400 * 24 * 60);
    for b in sim.land_mut_for_tests().buildings.iter_mut() {
        b.stage_since = long_ago;
    }
    let id = sim
        .place_building_for_tests(owner, spec, civ_grammar::Stage::ALL.len() as u8)
        .expect("placed");
    let model = {
        let b = sim
            .land()
            .buildings
            .iter()
            .find(|b| b.id == id)
            .expect("it");
        assert!(b.finished());
        style::traits_of(&b.spec, def(&sim, b))
    };
    assert_eq!(
        (
            model.openings_permille,
            model.ornament_dh,
            model.finish,
            model.symmetry
        ),
        (300.0, 15.0, 2.0, 1.0)
    );
    let words = {
        let b = sim
            .land()
            .buildings
            .iter()
            .find(|b| b.id == id)
            .expect("it");
        civ_sim::frames::buildings::style_words(&sim, b, Some(def(&sim, b)))
    };
    assert!(
        words.contains(
            "30 % of the walls open, 1.5 h of ornament a square metre, lime-washed, set out \
             about the middle"
        ),
        "{words}"
    );
    // One household knows how to frame it and burn lime; another knows neither.
    for m in sim.people().household(learned).expect("it").members.clone() {
        for t in [
            "core:technique/jointed_frame",
            "core:technique/lime_burning",
        ] {
            sim.introduce_technique(m, t, false).expect("taught");
        }
    }
    let unknown = |sim: &Sim, t: &str| {
        let t = sim.rules().catalog.technique_index(t).expect("it");
        sim.people()
            .household(unlearned)
            .expect("it")
            .members
            .iter()
            .filter_map(|&m| sim.people().person(m))
            .all(|p| !p.knows(t))
    };
    assert!(unknown(&sim, "core:technique/jointed_frame"));
    let was = (taste(&sim, learned), taste(&sim, unlearned));
    let rules = sim.rules().clone();
    let (land, now) = (sim.land().clone(), sim.now());
    sim.people_mut_for_tests().review_tastes(
        &rules.catalog,
        &rules.people,
        &rules.land,
        &land,
        now,
    );
    let now_taste = (taste(&sim, learned), taste(&sim, unlearned));
    let moved = |k: usize, before: &Taste, after: &Taste| {
        (after.traits()[k] - before.traits()[k]).abs() > 1e-3
    };
    // Those who could build it move toward all seven; the rest toward its roof, its openings
    // and its symmetry, not its ornament and finish.
    for k in 0..Taste::TRAITS {
        assert!(moved(k, &was.0, &now_taste.0), "{k}: {was:?} {now_taste:?}");
        let could = !(k == 4 || k == 5);
        assert_eq!(
            moved(k, &was.1, &now_taste.1),
            could,
            "{k}: {was:?} {now_taste:?}"
        );
    }
    // Tastes of seven traits survive a save.
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        SaveDir::create(dir.path().join("saves"), civ_schema::SAVE_EXTENSION).expect("save dir");
    let saved = persist::save(&mut sim, &saves, SaveKind::Manual, "front").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    for &h in &ids {
        assert_eq!(taste(&loaded, h), taste(&sim, h));
    }
}
