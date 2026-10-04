//! The condition of buildings (M3b slice P, ADR-0009 §4, §6): each group's quality drawn once from
//! its builders' skill when its stage is finished; loss that grows month by month with exposure;
//! a roof that leaks keeps less dry; upkeep that mends the worst of what shows before anything new
//! is built; and a ruin that shelters nothing.

use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

use civ_agents::build::{self, Shape};
use civ_agents::condition;
use civ_agents::params::BuildingDef;
use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_grammar::{Expansion, GroupKind, Stage};
use civ_land::{Building, BuildingState, GroupState};
use civ_sim::{NewWorld, Sim};

fn content() -> &'static ContentRegistry {
    static CONTENT: OnceLock<ContentRegistry> = OnceLock::new();
    CONTENT.get_or_init(|| {
        let root = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
            .expect("content/ is above the crate");
        civ_content::load(&root)
            .registry
            .expect("the core content loads")
    })
}

fn world(seed: u64) -> Sim {
    Sim::create(
        &NewWorld {
            name: "Upkeep".to_owned(),
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

fn hut_def() -> &'static BuildingDef {
    let catalog = &content().catalog;
    &catalog.buildings[catalog
        .building_index("core:building/hut")
        .expect("the hut")]
}

fn good(id: &str) -> usize {
    content()
        .catalog
        .good_index(id)
        .unwrap_or_else(|| panic!("{id}"))
}

/// World `seed`'s first household, given a finished hut at its home (its groups sound, as if
/// built at middling skill): the household and the hut.
fn housed(seed: u64) -> (Sim, PermanentId, PermanentId) {
    let mut sim = world(seed);
    let (household, home) = {
        let (_, h) = sim
            .people()
            .households
            .iter()
            .min_by_key(|(_, h)| h.id)
            .expect("a household");
        (h.id, h.home)
    };
    let spec = build::design_shape(
        hut_def(),
        &content().catalog.goods,
        Shape::Round { radius: 310 },
        home,
        None,
    )
    .expect("a hut");
    let hut = sim
        .place_building_for_tests(household, spec, Stage::ALL.len() as u8)
        .expect("placed");
    (sim, household, hut)
}

fn building(sim: &Sim, id: PermanentId) -> &Building {
    sim.land()
        .buildings
        .iter()
        .find(|b| b.id == id)
        .expect("the building")
}

fn building_mut(sim: &mut Sim, id: PermanentId) -> &mut Building {
    sim.land_mut_for_tests()
        .buildings
        .iter_mut()
        .find(|b| b.id == id)
        .expect("the building")
}

fn expansion(b: &Building) -> Expansion {
    civ_grammar::expand(&b.spec, &hut_def().rules).expect("expands")
}

fn group_of(b: &Building, kind: GroupKind) -> u32 {
    expansion(b)
        .groups
        .iter()
        .find(|g| g.kind == kind)
        .map(|g| g.id)
        .expect("a group of the kind")
}

fn loss(sim: &Sim, hut: PermanentId, group: u32) -> f32 {
    building(sim, hut).group(group).expect("in place").loss
}

/// The household's shelter derived again after a test changed its buildings by hand.
fn shelter_again(sim: &mut Sim, household: PermanentId) {
    let (now, land, rules) = (sim.now(), sim.land().clone(), sim.rules().clone());
    sim.people_mut_for_tests().buildings_changed(
        now,
        &land,
        &rules.catalog,
        &rules.people,
        household,
    );
}

fn keeping(sim: &Sim, household: PermanentId) -> civ_agents::person::Keeping {
    sim.people().household(household).expect("it").keeping
}

#[test]
fn a_stage_finished_puts_its_groups_in_place_and_builders_learn_by_building() {
    let mut sim = world(3);
    let skill = content()
        .catalog
        .skill_index("core:skill/building")
        .expect("the building skill");
    let before: Vec<(PermanentId, f64)> = sim
        .people()
        .people
        .iter()
        .map(|(_, p)| (p.id, p.skill(skill)))
        .collect();
    sim.advance_minutes(120 * 24 * 60).expect("advances");
    let mut stages_done = 0;
    for b in &sim.land().buildings {
        let e = expansion(b);
        let done = usize::from(b.stage);
        stages_done += done;
        // Exactly the groups of the stages finished are in place, each drawn once.
        for g in &e.groups {
            let placed = b.group(g.id);
            assert_eq!(placed.is_some(), g.stage.index() < done, "group {:x}", g.id);
            if let Some(c) = placed {
                assert!((condition::MIN_QUALITY..=1.0).contains(&c.quality), "{c:?}");
                assert!(c.installed >= b.started && c.installed <= sim.now());
            }
        }
        assert_eq!(
            b.condition.len(),
            e.groups.iter().filter(|g| g.stage.index() < done).count()
        );
    }
    assert!(
        stages_done >= 4,
        "homes are under way: {stages_done} stages done"
    );
    // Building is practice: those who built are more skilled, and nobody less.
    let mut learned = 0;
    for (id, was) in before {
        let Some(p) = sim.people().person(id) else {
            continue;
        };
        assert!(
            p.skill(skill) >= was - 1e-6,
            "{id}: {was} then {}",
            p.skill(skill)
        );
        if p.skill(skill) > was + 1e-3 {
            learned += 1;
        }
    }
    assert!(learned > 0, "someone learned by building");
}

#[test]
fn thatch_wears_by_the_month_and_posts_by_the_ground_they_stand_in() {
    let (mut sim, _, hut) = housed(3);
    let b = building(&sim, hut).clone();
    let (covering, posts) = (
        group_of(&b, GroupKind::Covering),
        group_of(&b, GroupKind::Posts),
    );
    assert_eq!(loss(&sim, hut, covering), 0.0);
    let mut days = 0;
    while loss(&sim, hut, covering) == 0.0 {
        sim.advance_minutes(24 * 60).expect("advances");
        days += 1;
        assert!(days <= 32, "a month passes");
    }
    assert!(
        sim.now().date().day <= 2,
        "on the first of the month: {}",
        sim.now().date()
    );
    let u = &hut_def().upkeep;
    let month = u.covering.per_year / 12.0;
    assert!((f64::from(loss(&sim, hut, covering)) - month).abs() < 1e-6);
    // Posts by the wetness of the habitat the hut stands in.
    let rules = sim.rules().clone();
    let (map, land) = (sim.map(), sim.land());
    let cell = civ_agents::population::cell_of(map, build::centre_m(&b.spec));
    let class = land.patches.class[land.patches.of_cell(cell, map.width)];
    let wetness = rules.land.habitats[usize::from(class)].wetness;
    let expected = u.posts.per_year / 12.0 * wetness;
    assert!(
        (f64::from(loss(&sim, hut, posts)) - expected).abs() < 1e-6,
        "{} against {expected}",
        loss(&sim, hut, posts)
    );
    // Roofed timber does not wear under a sound roof.
    let rafters = group_of(&b, GroupKind::RoofFrame);
    assert_eq!(loss(&sim, hut, rafters), 0.0);
}

#[test]
fn a_leaking_roof_keeps_less_and_its_household_mends_it() {
    let (mut sim, household, hut) = housed(3);
    let b = building(&sim, hut).clone();
    let covering = group_of(&b, GroupKind::Covering);
    let dry = keeping(&sim, household).roofed_kg;
    assert!(dry > 0.0);
    // The thatch has lost more than half: well past the share at which it leaks.
    let u = &hut_def().upkeep;
    let lost = 0.6;
    {
        let b = building_mut(&mut sim, hut);
        let c = b
            .condition
            .iter_mut()
            .find(|c| c.group == covering)
            .expect("in place");
        c.loss = lost;
        c.state = GroupState::Symptom;
    }
    shelter_again(&mut sim, household);
    let leak = condition::leak(building(&sim, hut), u);
    let expected = (f64::from(lost) - u.covering.shows_at) / (1.0 - u.covering.shows_at);
    assert!((leak - expected).abs() < 1e-6, "{leak}");
    let wet = keeping(&sim, household).roofed_kg;
    assert!((wet - dry * (1.0 - leak)).abs() < 1e-6, "{wet} of {dry}");
    let words = condition::symptoms(building(&sim, hut), u);
    assert!(words.contains("the thatch leaks"), "{words}");
    // With thatch at hand the household mends what is lost, fitting the work in among the rest
    // (research 11-06 §1.3: repair can wait while a house is lived in), and before the roof is
    // wanted for winter the room under it is dry again.
    let thatch = good("core:good/thatch");
    for (_, h) in sim.people_mut_for_tests().households.iter_mut() {
        if h.id == household {
            h.stores[thatch] += 1_500.0;
        }
    }
    let begun = sim.now();
    let deadline = build::roof_deadline(begun.day_index(), hut_def().roof_by_day);
    let mut started = None;
    while loss(&sim, hut, covering) >= 0.5 {
        sim.advance_minutes(24 * 60).expect("advances");
        if started.is_none() && building(&sim, hut).repair.is_some() {
            started = Some(sim.now());
        }
        assert!(
            sim.now().day_index() <= deadline,
            "mended before winter: {:?}",
            building(&sim, hut).repair
        );
    }
    let started = started.expect("a repair was under way");
    assert!(
        started.day_index() - begun.day_index() <= 7,
        "begun within the week, on {}",
        started.date()
    );
    let c = *building(&sim, hut).group(covering).expect("in place");
    // What wore while the work went on is still lost.
    assert!(c.loss < 0.05, "{c:?}");
    assert!(c.repaired > begun);
    assert_eq!(c.state, GroupState::Sound);
    assert!(building(&sim, hut).repair.is_none());
    assert!((keeping(&sim, household).roofed_kg - dry).abs() < 1.0);
    let used = sim
        .people()
        .household(household)
        .expect("it")
        .flows
        .get(civ_agents::person::Flow::Built, thatch);
    let roof = condition::mend_needs(&expansion(&b), covering, f64::from(lost)).expect("needs");
    let thatch_slot = hut_def()
        .materials
        .iter()
        .position(|&g| g == thatch)
        .expect("a thatch slot");
    assert!(
        (used - roof.materials_kg[thatch_slot]).abs() < 1.0,
        "{used} kg of thatch for {} kg",
        roof.materials_kg[thatch_slot]
    );
}

#[test]
fn a_ruin_shelters_nothing_and_its_household_builds_anew() {
    let (mut sim, household, hut) = housed(3);
    let posts = group_of(building(&sim, hut), GroupKind::Posts);
    {
        let b = building_mut(&mut sim, hut);
        let c = b
            .condition
            .iter_mut()
            .find(|c| c.group == posts)
            .expect("in place");
        c.loss = 0.999;
        // A month in sodden ground finishes them.
        condition::wear_month(b, &hut_def().upkeep, 10.0);
        assert_eq!(b.state, BuildingState::Ruin);
        assert!(!b.roofed());
    }
    shelter_again(&mut sim, household);
    let h = sim.people().household(household).expect("it");
    assert!(!h.sheltered, "a ruin is no roof");
    assert_eq!(
        condition::symptoms(building(&sim, hut), &hut_def().upkeep),
        "a ruin: its posts gave way"
    );
    // It begins a new home, and takes the ruin down once the new roof is on.
    let mut new_home = None;
    for _ in 0..240 {
        sim.advance_minutes(24 * 60).expect("advances");
        new_home = sim
            .land()
            .buildings
            .iter()
            .find(|b| b.household == household && b.id != hut)
            .map(|b| (b.id, b.roofed()));
        if new_home.is_some_and(|(_, roofed)| roofed) {
            break;
        }
    }
    let (id, roofed) = new_home.expect("a new home is begun");
    assert!(roofed, "its roof is on within the season");
    assert!(
        sim.land().buildings.iter().all(|b| b.id != hut),
        "the ruin is gone"
    );
    let h = sim.people().household(household).expect("it");
    assert!(h.sheltered);
    assert_eq!(h.home, build::centre_m(&building(&sim, id).spec));
}
