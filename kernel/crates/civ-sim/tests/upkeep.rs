//! The condition of buildings (M3b slice P, ADR-0009 §4–6): each group's quality drawn once from
//! its builders' skill when its stage is finished; loss that grows month by month with exposure;
//! a roof that leaks keeps less dry; upkeep that mends the worst of what shows before anything new
//! is built; a ruin that shelters nothing; and loads weighed each day, under which a weak part
//! gives way.

use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

use civ_agents::build::{self, Shape};
use civ_agents::condition;
use civ_agents::history::{Cause, ChronicleKind};
use civ_agents::params::BuildingDef;
use civ_agents::person::Flow;
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

fn longhouse_def() -> &'static BuildingDef {
    let catalog = &content().catalog;
    &catalog.buildings[catalog
        .building_index("core:building/longhouse")
        .expect("the longhouse")]
}

/// World `seed`'s first household in a finished longhouse of two bays with a loft over each, at
/// its home, holding `grain` kilograms of grain besides what it brought: the household and the
/// longhouse.
fn lofted(seed: u64, grain: f64) -> (Sim, PermanentId, PermanentId) {
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
    let shape = Shape::Bays {
        bays: 2,
        storeys: 1,
        lofts: 2,
    };
    let spec = build::design_shape(longhouse_def(), &content().catalog.goods, shape, home, None)
        .expect("a longhouse");
    let house = sim
        .place_building_for_tests(household, spec, Stage::ALL.len() as u8)
        .expect("placed");
    let g = good("core:good/grain");
    for (_, h) in sim.people_mut_for_tests().households.iter_mut() {
        if h.id == household {
            h.stores[g] += grain;
        }
    }
    shelter_again(&mut sim, household);
    (sim, household, house)
}

/// Kilograms of every good household `household` has had spoil.
fn spoiled(sim: &Sim, household: PermanentId) -> f64 {
    let flows = &sim.people().household(household).expect("it").flows;
    (0..content().catalog.goods.len())
        .map(|g| flows.get(Flow::Spoiled, g))
        .sum()
}

#[test]
fn a_full_loft_on_sound_joists_holds_and_on_poor_ones_gives_way() {
    // A full loft on joists made as a fair builder makes them holds: 5 m poles of 15 cm.
    let (mut sim, household, house) = lofted(3, 8_000.0);
    let before = spoiled(&sim, household);
    let loft_room = longhouse_def().shapes.iter().find(|c| {
        c.shape
            == Shape::Bays {
                bays: 2,
                storeys: 1,
                lofts: 2,
            }
    });
    let room = loft_room.expect("the shape").room_kg[civ_grammar::storage::LOFT];
    assert!(room > 3_000.0, "{room} kg in two lofts");
    let joists: Vec<u32> = building(&sim, house)
        .condition
        .iter()
        .filter(|c| GroupKind::of_group(c.group) == Some(GroupKind::LoftJoists))
        .map(|c| c.group)
        .collect();
    assert_eq!(joists.len(), 2);
    sim.advance_minutes(24 * 60).expect("advances");
    let b = building(&sim, house);
    assert!(
        joists
            .iter()
            .all(|&j| b.group(j).expect("in place").state != GroupState::Failed),
        "{:?}",
        b.condition
    );
    let spoiled_anyway = spoiled(&sim, household) - before;
    // Poles a novice chose badly give way under it: the loft's grain spills, part of it lost,
    // the house is damaged, and the chronicle says what fell, under what and why.
    let (mut sim, household, house) = lofted(3, 8_000.0);
    for b in sim.land_mut_for_tests().buildings.iter_mut() {
        if b.id == house {
            for c in &mut b.condition {
                if GroupKind::of_group(c.group) == Some(GroupKind::LoftJoists) {
                    c.quality = 0.2;
                }
            }
        }
    }
    let before = spoiled(&sim, household);
    let roofed_before = keeping(&sim, household).roofed_kg;
    sim.advance_minutes(24 * 60).expect("advances");
    let b = building(&sim, house);
    assert!(
        joists
            .iter()
            .all(|&j| b.group(j).expect("in place").state == GroupState::Failed),
        "{:?}",
        b.condition
    );
    assert_eq!(b.state, BuildingState::Damaged);
    assert!(b.roofed(), "the house keeps its roof");
    // What spilled from the full lofts was lost besides what spoils in a day anyway.
    let lost = spoiled(&sim, household) - before - spoiled_anyway;
    let expected = room * civ_agents::population::SPILLED;
    assert!(
        (lost - expected).abs() < 0.02 * expected,
        "{lost} kg lost, {expected} expected"
    );
    // Its lofts hold nothing now.
    let roofed_after = keeping(&sim, household).roofed_kg;
    assert!(
        (roofed_before - roofed_after - room).abs() < 1.0,
        "{roofed_before} then {roofed_after}"
    );
    let entry = sim
        .people()
        .chronicle
        .iter()
        .rev()
        .find(|e| e.kind == ChronicleKind::BuildingFailed)
        .expect("in the chronicle");
    assert!(
        entry
            .name
            .starts_with("longhouse lost its loft: the joists broke under 3.8 t of "),
        "{}",
        entry.name
    );
    assert!(
        entry.name.ends_with("they were poorly made"),
        "{}",
        entry.name
    );
    // What gave way is rebuilt whole when it is mended.
    let def = longhouse_def();
    let r = condition::repair_of(building(&sim, house), &def.upkeep).expect("a repair");
    assert!(joists.contains(&r.group));
    assert_eq!(r.share, 1.0);
}

#[test]
fn posts_rotted_through_bring_a_hut_down_and_those_it_kills_die_of_its_collapse() {
    let (mut sim, household, hut) = housed(3);
    let posts = group_of(building(&sim, hut), GroupKind::Posts);
    for b in sim.land_mut_for_tests().buildings.iter_mut() {
        if b.id == hut {
            for c in &mut b.condition {
                if c.group == posts {
                    c.loss = 0.9;
                    c.state = GroupState::Symptom;
                }
            }
        }
    }
    sim.advance_minutes(24 * 60).expect("advances");
    let b = building(&sim, hut);
    assert_eq!(b.state, BuildingState::Ruin);
    assert!(!sim.people().household(household).expect("it").sheltered);
    let entry = sim
        .people()
        .chronicle
        .iter()
        .rev()
        .find(|e| e.kind == ChronicleKind::BuildingFailed)
        .expect("in the chronicle")
        .clone();
    assert!(
        entry.name.starts_with("hut fell: its posts gave way"),
        "{}",
        entry.name
    );
    assert!(entry.name.ends_with("they had rotted"), "{}", entry.name);
    // Those it killed, if any, died of its collapse, and the entry names them.
    let dead: Vec<PermanentId> = entry.people.iter().skip(1).copied().collect();
    assert_eq!(dead.len(), entry.number as usize);
    for who in dead {
        let r = &sim.people().records[&who];
        assert!(matches!(r.died, Some((_, Cause::Collapse))), "{r:?}");
    }
}

#[test]
fn the_months_peak_is_one_storm_on_one_day_for_a_whole_settlement() {
    use civ_agents::population::{month_peak, peak_pa};
    use civ_core::SimTime;
    let peak = civ_land::PeakLoad {
        median_pa: 250.0,
        spread: 0.6,
    };
    let place = PermanentId::from_raw(7).expect("nonzero");
    let day =
        |year: i64, month: u8, day: u8| SimTime::from_date(year, month, day, 0, 0).expect("a date");
    let mut loads = Vec::new();
    for year in 1..=50 {
        for month in 1..=12u8 {
            // One draw a month: the same storm on whatever day of the month it is asked for...
            let (on, pa) = month_peak(11, place, day(year, month, 3), &peak);
            assert_eq!((on, pa), month_peak(11, place, day(year, month, 20), &peak));
            assert!((1..=28).contains(&on), "{on}");
            // ...striking the roofs on that day alone, so a roof mended after it does not meet
            // it again.
            let length = civ_core::time::MONTH_LENGTHS[usize::from(month) - 1] as u8;
            let struck: Vec<u8> = (1..=length)
                .filter(|&d| peak_pa(11, place, day(year, month, d), &peak) > 0.0)
                .collect();
            assert_eq!(struck, vec![on]);
            assert_eq!(peak_pa(11, place, day(year, month, on), &peak), pa);
            loads.push(pa);
        }
    }
    assert!(loads.windows(2).any(|w| w[0] != w[1]));
    loads.sort_by(f64::total_cmp);
    let median = loads[loads.len() / 2];
    assert!((median - 250.0).abs() < 40.0, "{median}");
    assert!(loads.iter().all(|&p| p > 0.0));
}

#[test]
fn a_loft_that_gives_way_makes_its_settlement_build_frames_stronger_for_a_while() {
    use civ_grammar::frame_params as fp;
    let catalog = &content().catalog;
    let def = longhouse_def();
    let shape = Shape::Bays {
        bays: 2,
        storeys: 1,
        lofts: 2,
    };
    let sized = |caution: f64| {
        let spec = build::design_cautious(def, &catalog.goods, shape, (0.0, 0.0), None, caution)
            .expect("a longhouse");
        (spec.params[fp::JOIST_CM], spec.params[fp::POST_CM])
    };
    // Builders who have seen nothing fail build as usual; twice as cautious, joists of 19 cm
    // (15 x 2^(1/3): bending strength goes as d^3) and posts of 18 (15 x 2^(1/4): buckling goes
    // as d^4); never past what the program allows.
    assert_eq!(sized(1.0), (15, 15));
    assert_eq!(sized(2.0), (19, 18));
    assert_eq!(sized(100.0), (25, 30));
    assert_eq!(
        build::design_shape(def, &catalog.goods, shape, (0.0, 0.0), None),
        build::design_cautious(def, &catalog.goods, shape, (0.0, 0.0), None, 1.0)
    );

    // A loft on poles a novice chose badly gives way under a full load...
    let (mut sim, household, house) = lofted(3, 8_000.0);
    for b in sim.land_mut_for_tests().buildings.iter_mut() {
        if b.id == house {
            for c in &mut b.condition {
                if GroupKind::of_group(c.group) == Some(GroupKind::LoftJoists) {
                    c.quality = 0.2;
                }
            }
        }
    }
    let settlement = sim.people().household(household).and_then(|h| h.settlement);
    assert!(settlement.is_some(), "the household lives in a settlement");
    let p = sim.rules().people.build.caution;
    let caution = |sim: &Sim| {
        sim.people()
            .caution(settlement, def.technique, sim.now(), &p)
    };
    assert_eq!(caution(&sim), 1.0);
    sim.advance_minutes(24 * 60).expect("advances");
    assert_eq!(building(&sim, house).state, BuildingState::Damaged);
    // ...and the settlement builds its next longhouse's joists and posts stronger: one failure
    // among fewer than ten building-years is a rate of 0.1 a year or more, ten times the half
    // rate, so caution is nine tenths of the way to its most.
    let after = caution(&sim);
    assert!(after > 1.9 && after < 2.0, "{after}");
    let (joist, post) = sized(after);
    assert!(joist >= 19 && post >= 18, "{joist} and {post} cm");
    let trust = *sim
        .people()
        .trust
        .iter()
        .find(|t| Some(t.settlement) == settlement)
        .expect("remembered");
    assert!(trust.failures >= 1.0, "{trust:?}");
    // Each month every standing building of the technique adds a month of standing; years
    // without failures bring caution back toward 1 (the half-life is the profile's).
    let to_month = {
        let d = sim.now().date();
        let length = civ_core::time::MONTH_LENGTHS[usize::from(d.month) - 1];
        (length - i64::from(d.day) + 2) * 24 * 60
    };
    sim.advance_minutes(to_month).expect("advances");
    let later = sim
        .people()
        .trust
        .iter()
        .find(|t| Some(t.settlement) == settlement)
        .copied()
        .expect("remembered");
    assert!(later.years > 0.0, "{later:?}");
    let years_on = later.caution(
        sim.now()
            .plus_minutes(16 * civ_core::time::MINUTES_PER_YEAR),
        &p,
    );
    assert!(years_on < after, "{years_on} after {after}");
}

#[test]
fn worn_daub_is_renewed_with_earth_dug_beside_the_hut() {
    use civ_land::earth::EarthKind;
    let (mut sim, household, hut) = housed(3);
    let b = building(&sim, hut).clone();
    let infill = group_of(&b, GroupKind::Infill);
    let daub = expansion(&b)
        .groups
        .iter()
        .find(|g| g.id == infill)
        .expect("the walls")
        .daub_m3();
    assert!(daub > 1.0, "{daub}");
    // Placed whole, as by a test: it has no daub pit yet.
    let pit_of = |sim: &Sim| {
        sim.land()
            .earthworks
            .iter()
            .find(|w| w.kind == EarthKind::Pit && w.plot == Some(b.plot))
            .copied()
    };
    assert!(pit_of(&sim).is_none());
    // Its walls have lost more than half their daub; with wattle rods at hand they are mended.
    let lost = 0.6;
    {
        let b = building_mut(&mut sim, hut);
        let c = b
            .condition
            .iter_mut()
            .find(|c| c.group == infill)
            .expect("in place");
        c.loss = lost;
        c.state = GroupState::Symptom;
    }
    let timber = good("core:good/timber");
    for (_, h) in sim.people_mut_for_tests().households.iter_mut() {
        if h.id == household {
            h.stores[timber] += 500.0;
        }
    }
    for _ in 0..60 {
        sim.advance_minutes(24 * 60).expect("advances");
        if loss(&sim, hut, infill) < 0.05 {
            break;
        }
    }
    assert!(loss(&sim, hut, infill) < 0.05, "mended");
    // The daub renewed was dug from a pit begun beside it: what was lost of the walls' earth.
    let pit = pit_of(&sim).expect("a daub pit");
    assert_eq!(pit.deposit, None);
    let dug = f64::from(pit.cut_m3);
    assert!(
        dug >= daub * f64::from(lost) * 0.99 && dug <= daub * 0.7,
        "{dug} m³ of the walls' {daub}"
    );
}
