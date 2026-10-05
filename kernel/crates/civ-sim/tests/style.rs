//! Style v0 (M3b slice R; research 11-02): each band builds its own way and each home to its
//! household's taste, and families sent to a village bring theirs; once a year taste moves toward the settlement's admired new buildings, and
//! a household's next building follows the one that moved it most; a building that gave way is
//! admired by nobody; saves keep it all.

use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

use civ_agents::params::{BuildingDef, Taste};
use civ_agents::style;
use civ_content::ContentRegistry;
use civ_core::{PermanentId, SimTime};
use civ_grammar::hut_params;
use civ_land::{Building, BuildingState, GroupState};
use civ_sim::{NewWorld, Sim, persist};
use commons_persist::{SaveDir, SaveKind};

/// The core content, but nothing new by chance: a building is built to its builders' taste.
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
    Sim::create(
        &NewWorld {
            name: "Style".to_owned(),
            seed,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 512,
            band_size: 0,
            regime_id: String::new(),
        },
        content(),
        &mut |_| {},
        &AtomicBool::new(false),
    )
    .expect("generates")
}

fn def<'a>(sim: &'a Sim, b: &Building) -> &'a BuildingDef {
    let catalog = &sim.rules().catalog;
    &catalog.buildings[catalog
        .building_index(&b.spec.program)
        .expect("its program")]
}

/// Building `b`'s traits, in whole units: pitch, eaves, overhang.
fn traits(sim: &Sim, b: &Building) -> [i32; 3] {
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
        let expected = style::moved(&was, &model, alpha);
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
