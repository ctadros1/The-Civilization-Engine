//! Help with mending (research 11-04 §1.6): a household whose people may not mend its building,
//! too old for the build work, asks its kin, or failing kin a neighbour its people hold a tie
//! with, whose people mend it as their own; the asker owes them the work. A household with
//! someone who may do the work mends its own, as before.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::build::{self, Shape};
use civ_agents::params::BuildingDef;
use civ_agents::ties::Act;
use civ_content::ContentRegistry;
use civ_core::{PermanentId, SimTime};
use civ_grammar::{GroupKind, Stage};
use civ_land::{Building, GroupState};
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
    Sim::create_for_tests(
        &NewWorld {
            name: "Mending".to_owned(),
            seed,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 512,
            band_size: 0,
            neighbours: Vec::new(),
            neighbours_known: false,
            regime_id: String::new(),
        },
        content(),
        [9; 16],
    )
    .expect("generates")
}

fn hut_def() -> &'static BuildingDef {
    let catalog = &content().catalog;
    &catalog.buildings[catalog
        .building_index("core:building/hut")
        .expect("the hut")]
}

fn thatch() -> usize {
    content()
        .catalog
        .good_index("core:good/thatch")
        .expect("thatch")
}

fn building(sim: &Sim, id: PermanentId) -> &Building {
    sim.land()
        .buildings
        .iter()
        .find(|b| b.id == id)
        .expect("the building")
}

/// The covering of hut `hut`.
fn covering(sim: &Sim, hut: PermanentId) -> u32 {
    let b = building(sim, hut);
    civ_grammar::expand(&b.spec, &hut_def().rules)
        .expect("expands")
        .groups
        .iter()
        .find(|g| g.kind == GroupKind::Covering)
        .map(|g| g.id)
        .expect("a covering")
}

fn covering_loss(sim: &Sim, hut: PermanentId) -> f32 {
    let g = covering(sim, hut);
    building(sim, hut).group(g).expect("in place").loss
}

/// A world of two households of one settlement or more, its smallest household `asker` given a
/// finished hut at its home whose thatch has lost 0.6 (well past leaking), every other household
/// given thatch enough to mend it, and `asker`'s people made `age` years old: the world, the
/// asker, its hut and the other households of its settlement.
fn leaking(seed: u64, age: f64) -> (Sim, PermanentId, PermanentId, Vec<PermanentId>) {
    let mut sim = world(seed);
    let (asker, home, others) = {
        let pop = sim.people();
        let settled: Vec<_> = pop
            .households
            .iter()
            .map(|(_, h)| h)
            .filter(|h| h.settlement.is_some())
            .collect();
        let asker = settled
            .iter()
            .min_by_key(|h| (h.members.len(), h.id))
            .expect("a settled household");
        let others: Vec<PermanentId> = settled
            .iter()
            .filter(|h| h.id != asker.id && h.settlement == asker.settlement)
            .map(|h| h.id)
            .collect();
        (asker.id, asker.home, others)
    };
    assert!(!others.is_empty(), "a neighbour in its settlement");
    let spec = build::design_shape(
        hut_def(),
        &content().catalog.goods,
        Shape::Round { radius: 310 },
        home,
        None,
    )
    .expect("a hut");
    let hut = sim
        .place_building_for_tests(asker, spec, Stage::ALL.len() as u8)
        .expect("placed");
    let g = covering(&sim, hut);
    {
        let b = sim
            .land_mut_for_tests()
            .buildings
            .iter_mut()
            .find(|b| b.id == hut)
            .expect("the hut");
        let c = b
            .condition
            .iter_mut()
            .find(|c| c.group == g)
            .expect("in place");
        c.loss = 0.6;
        c.state = GroupState::Symptom;
    }
    let now = sim.now();
    let born = SimTime::from_minutes(now.minutes() - (age * 525_600.0) as i64);
    let pop = sim.people_mut_for_tests();
    for (_, p) in pop.people.iter_mut() {
        if p.household == asker {
            p.born = born;
        }
    }
    for (_, h) in pop.households.iter_mut() {
        if h.id != asker {
            h.stores[thatch()] += 1_500.0;
        }
    }
    (sim, asker, hut, others)
}

/// Lives `sim` a day at a time, at most `days`, until hut `hut`'s thatch is mended.
fn until_mended(sim: &mut Sim, hut: PermanentId, days: u32) -> bool {
    for _ in 0..days {
        sim.advance_minutes(24 * 60).expect("advances");
        if covering_loss(sim, hut) < 0.05 {
            return true;
        }
    }
    false
}

/// The tie `asker`'s elder holds to `helper`'s elder, if any.
fn tie(sim: &Sim, asker: PermanentId, helper: PermanentId) -> Option<civ_agents::ties::Tie> {
    let pop = sim.people();
    let elder = |h: PermanentId| pop.household(h).and_then(|x| x.members.first().copied());
    let (a, b) = (elder(asker)?, elder(helper)?);
    let tp = &sim.rules().people.ties;
    pop.ties.tie(a, b, sim.now().day_index(), tp)
}

/// Makes the first member of household `kin` a child of the first member of `asker`.
fn make_kin(sim: &mut Sim, asker: PermanentId, kin: PermanentId) {
    let pop = sim.people_mut_for_tests();
    let parent = pop
        .household(asker)
        .and_then(|x| x.members.first().copied());
    let child = pop.household(kin).and_then(|x| x.members.first().copied());
    let (Some(parent), Some(child)) = (parent, child) else {
        panic!("both households have someone");
    };
    let female = pop.person(parent).expect("alive").sex == civ_agents::Sex::Female;
    let r = pop.records.get_mut(&child).expect("on record");
    if female {
        r.mother = Some(parent);
    } else {
        r.father = Some(parent);
    }
    pop.forget_kin();
}

#[test]
fn an_elder_households_leaking_hut_is_mended_by_its_kin_who_are_owed_the_work() {
    let (mut sim, asker, hut, others) = leaking(3, 67.0);
    let kin = others[0];
    make_kin(&mut sim, asker, kin);
    assert!(
        until_mended(&mut sim, hut, 150),
        "mended within five months"
    );
    assert_eq!(
        sim.people().kin_households_for_tests(asker),
        vec![kin],
        "its only kin"
    );
    // Nobody of the asker's household did the work: the thatch came from the kin's stores.
    let pop = sim.people();
    let used = |h: PermanentId| {
        pop.household(h)
            .expect("it")
            .flows
            .get(civ_agents::person::Flow::Built, thatch())
    };
    assert_eq!(used(asker), 0.0);
    assert!(used(kin) > 0.0, "the kin brought the thatch");
    let t = tie(&sim, asker, kin).expect("a tie to the kin");
    assert_eq!(t.reason.map(|r| r.act), Some(Act::HelpedMend));
    assert!(t.help_h > 0.0, "the asker owes the work: {}", t.help_h);
    let back = tie(&sim, kin, asker).expect("the kin's tie back");
    assert!(back.help_h < 0.0, "the kin is owed: {}", back.help_h);
}

#[test]
fn with_no_kin_a_neighbour_its_people_hold_a_tie_with_mends_it() {
    let (mut sim, asker, hut, _) = leaking(3, 67.0);
    assert!(sim.people().kin_households_for_tests(asker).is_empty());
    let elder = |sim: &Sim, h: PermanentId| {
        sim.people()
            .household(h)
            .and_then(|x| x.members.first().copied())
            .expect("someone")
    };
    let me = elder(&sim, asker);
    // At first its people know nobody well; ties form at the hearth as the months pass.
    assert!(
        until_mended(&mut sim, hut, 150),
        "mended within five months"
    );
    let pop = sim.people();
    let helped: Vec<_> = pop
        .ties
        .of(me)
        .iter()
        .filter(|t| t.reason.map(|r| r.act) == Some(Act::HelpedMend))
        .collect();
    assert_eq!(helped.len(), 1, "one household mended it");
    let helper = pop.person(helped[0].to).expect("alive").household;
    assert_ne!(helper, asker);
    assert!(
        sim.people().kin_households_for_tests(asker).is_empty(),
        "no kin: a neighbour"
    );
    assert_eq!(
        elder(&sim, helper),
        helped[0].to,
        "asked through the tie its elder holds to theirs"
    );
    assert!(helped[0].help_h > 0.0, "the asker owes the work");
}

#[test]
fn a_household_with_someone_who_may_do_the_work_mends_its_own() {
    let (mut sim, asker, hut, others) = leaking(3, 40.0);
    make_kin(&mut sim, asker, others[0]);
    for (_, h) in sim.people_mut_for_tests().households.iter_mut() {
        if h.id == asker {
            h.stores[thatch()] += 1_500.0;
        }
    }
    assert!(
        until_mended(&mut sim, hut, 150),
        "mended within five months"
    );
    let pop = sim.people();
    assert!(
        pop.household(asker)
            .expect("it")
            .flows
            .get(civ_agents::person::Flow::Built, thatch())
            > 0.0,
        "its own thatch"
    );
    assert!(
        tie(&sim, asker, others[0])
            .is_none_or(|t| t.reason.map(|r| r.act) != Some(Act::HelpedMend)),
        "no help asked"
    );
}

/// A world of two households of one settlement or more, its smallest household `asker` with no
/// home at all, standing or begun, its first member made `elder` years old and the rest `young`:
/// the world, the asker and the other households of its settlement.
fn unhoused(seed: u64, elder: f64, young: f64) -> (Sim, PermanentId, Vec<PermanentId>) {
    let mut sim = world(seed);
    let (asker, others) = {
        let pop = sim.people();
        let settled: Vec<_> = pop
            .households
            .iter()
            .map(|(_, h)| h)
            .filter(|h| h.settlement.is_some())
            .collect();
        let asker = settled
            .iter()
            .filter(|h| h.members.len() >= 2)
            .min_by_key(|h| (h.members.len(), h.id))
            .expect("a settled household of two or more");
        let others: Vec<PermanentId> = settled
            .iter()
            .filter(|h| h.id != asker.id && h.settlement == asker.settlement)
            .map(|h| h.id)
            .collect();
        (asker.id, others)
    };
    assert!(!others.is_empty(), "a neighbour in its settlement");
    assert!(
        !sim.land().buildings.iter().any(|b| b.household == asker),
        "no home, standing or begun"
    );
    let now = sim.now();
    let born = |age: f64| SimTime::from_minutes(now.minutes() - (age * 525_600.0) as i64);
    let pop = sim.people_mut_for_tests();
    let first = pop
        .household(asker)
        .and_then(|x| x.members.first().copied())
        .expect("someone");
    for (_, p) in pop.people.iter_mut() {
        if p.household == asker {
            p.born = born(if p.id == first { elder } else { young });
        }
    }
    (sim, asker, others)
}

/// Household `household`'s home under a roof, if it has one.
fn roofed_home(sim: &Sim, household: PermanentId) -> Option<PermanentId> {
    let catalog = &sim.rules().catalog;
    sim.land()
        .buildings
        .iter()
        .find(|b| b.household == household && b.roofed() && catalog.is_dwelling(&b.spec.program))
        .map(|b| b.id)
}

/// Lives `sim` a day at a time, at most `days`, until household `household` has a home under a
/// roof.
fn until_roofed(sim: &mut Sim, household: PermanentId, days: u32) -> Option<PermanentId> {
    for _ in 0..days {
        sim.advance_minutes(24 * 60).expect("advances");
        if let Some(b) = roofed_home(sim, household) {
            return Some(b);
        }
    }
    None
}

#[test]
fn an_elder_and_a_child_with_no_home_have_one_begun_and_roofed_by_their_kin_who_are_owed_the_work()
{
    // Found by a smoke world (§8 24, sixth item): a grandparent of 67 and a child of 7, of an age
    // no build activity admits, never began a home and lived three years unroofed. They ask as
    // help with mending asks (research 11-04 §1.6): their kin begin their home, on their ground,
    // and build it with their own materials, and are owed the work.
    let (mut sim, asker, others) = unhoused(3, 67.0, 7.0);
    let kin = others[0];
    make_kin(&mut sim, asker, kin);
    let home = until_roofed(&mut sim, asker, 365).expect("a home roofed within the year");
    assert_eq!(
        sim.people().kin_households_for_tests(asker),
        vec![kin],
        "its only kin"
    );
    let b = building(&sim, home);
    assert_eq!(b.household, asker, "the asker's own home");
    let plot = sim
        .land()
        .plots
        .iter()
        .find(|p| p.id == b.plot)
        .expect("its plot");
    assert_eq!(plot.household, asker, "on the asker's ground");
    // Nobody of the asker's household did the work: the materials came from the kin's stores.
    let pop = sim.people();
    let built = |h: PermanentId| {
        let x = pop.household(h).expect("it");
        (0..sim.rules().catalog.goods.len())
            .map(|g| x.flows.get(civ_agents::person::Flow::Built, g))
            .sum::<f64>()
    };
    assert_eq!(built(asker), 0.0);
    assert!(built(kin) > 0.0, "the kin brought the materials");
    let t = tie(&sim, asker, kin).expect("a tie to the kin");
    assert_eq!(t.reason.map(|r| r.act), Some(Act::HelpedMend));
    assert!(t.help_h > 0.0, "the asker owes the work: {}", t.help_h);
    let back = tie(&sim, kin, asker).expect("the kin's tie back");
    assert!(back.help_h < 0.0, "the kin is owed: {}", back.help_h);
    let problems = pop.problems(sim.ids().peek_next(), sim.rules().catalog.activities.len());
    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn with_no_kin_a_neighbour_its_people_hold_a_tie_with_builds_its_first_home() {
    let (mut sim, asker, _) = unhoused(3, 67.0, 7.0);
    assert!(sim.people().kin_households_for_tests(asker).is_empty());
    let me = sim
        .people()
        .household(asker)
        .and_then(|x| x.members.first().copied())
        .expect("someone");
    // At first its people know nobody well; ties form at the hearth as the months pass.
    until_roofed(&mut sim, asker, 365).expect("a home roofed within the year");
    let pop = sim.people();
    let helped: Vec<_> = pop
        .ties
        .of(me)
        .iter()
        .filter(|t| t.reason.map(|r| r.act) == Some(Act::HelpedMend))
        .collect();
    assert_eq!(helped.len(), 1, "one household built it");
    let helper = pop.person(helped[0].to).expect("alive").household;
    assert_ne!(helper, asker);
    assert_eq!(
        pop.household(helper)
            .and_then(|x| x.members.first().copied()),
        Some(helped[0].to),
        "asked through the tie its elder holds to theirs"
    );
    assert!(helped[0].help_h > 0.0, "the asker owes the work");
}

#[test]
fn a_household_with_someone_of_age_builds_its_own_first_home() {
    let (mut sim, asker, others) = unhoused(3, 67.0, 30.0);
    make_kin(&mut sim, asker, others[0]);
    let home = until_roofed(&mut sim, asker, 365).expect("a home roofed within the year");
    assert_eq!(building(&sim, home).household, asker);
    let pop = sim.people();
    let x = pop.household(asker).expect("it");
    let built: f64 = (0..sim.rules().catalog.goods.len())
        .map(|g| x.flows.get(civ_agents::person::Flow::Built, g))
        .sum();
    assert!(built > 0.0, "its own materials");
    assert!(
        tie(&sim, asker, others[0])
            .is_none_or(|t| t.reason.map(|r| r.act) != Some(Act::HelpedMend)),
        "no help asked"
    );
}

/// The seventeenth coverage pass (§8 24, sixth's first-home slice, 5f721b6): once the kin roof the
/// first home they began for a household that asked, the steps at its roof follow the building's
/// owner, not the builder. From that hour the asker is sheltered under it with the room its own
/// roof gives, as a load would derive it; and the kin's own home, which they had to have before
/// beginning another's, stands where it stood, for it is no home the asker leaves.
#[test]
fn a_first_home_roofed_by_kin_shelters_the_asker_and_leaves_the_kins_own_home_standing() {
    let (mut sim, asker, others) = unhoused(3, 67.0, 7.0);
    let kin = others[0];
    make_kin(&mut sim, asker, kin);
    let catalog = &content().catalog;
    let homes_of = |sim: &Sim, h: PermanentId| -> Vec<PermanentId> {
        sim.land()
            .buildings
            .iter()
            .filter(|b| {
                b.household == h
                    && b.finished()
                    && b.standing()
                    && catalog.is_dwelling(&b.spec.program)
            })
            .map(|b| b.id)
            .collect()
    };
    // Lived until the asker's home is begun: the kin's own home then, and where they live.
    let mut kins = None;
    for _ in 0..365 {
        sim.advance_minutes(24 * 60).expect("advances");
        if sim.land().buildings.iter().any(|b| b.household == asker) {
            let home = sim.people().household(kin).expect("the kin").home;
            kins = Some((homes_of(&sim, kin), home));
            break;
        }
    }
    let (kin_homes, kin_home) = kins.expect("the asker's home begun within the year");
    assert!(
        !kin_homes.is_empty(),
        "the kin were housed when they began it"
    );
    // Lived an hour at a time until its roof is on, before the home is finished.
    let mut home = None;
    for _ in 0..365 * 24 {
        sim.advance_minutes(60).expect("advances");
        home = roofed_home(&sim, asker);
        if home.is_some() {
            break;
        }
    }
    let home = home.expect("a home roofed within the year");
    assert_eq!(building(&sim, home).household, asker);
    assert!(!building(&sim, home).finished(), "roofed, not yet finished");
    // Its shelter and the room its roof gives its goods are its own home's, as a load would
    // derive them now, not its builder's.
    let shelter = |sim: &Sim, h: PermanentId| {
        let x = sim.people().household(h).expect("it");
        (x.sheltered, x.keeping)
    };
    let (asker_now, kin_now) = (shelter(&sim, asker), shelter(&sim, kin));
    assert!(asker_now.0, "the asker is under its roof");
    sim.with_ctx_for_tests(|pop, ctx| pop.derive_shelter(ctx.land, ctx.catalog, ctx.params));
    assert_eq!(asker_now, shelter(&sim, asker), "the asker's own roof");
    assert_eq!(kin_now, shelter(&sim, kin), "the kin's own home");
    let pop = sim.people();
    for b in &kin_homes {
        assert!(
            sim.land().buildings.iter().any(|x| x.id == *b),
            "the kin's home {b:?} stands"
        );
    }
    assert_eq!(
        pop.household(kin).expect("the kin").home,
        kin_home,
        "the kin live where they lived"
    );
}

/// The seventeenth coverage pass (§8 24, sixth's first-home slice, 5f721b6): a household with no
/// home and nobody of an age to build asks only a household with someone of that age. Its kin,
/// all past the build ages themselves, are passed over for a neighbour its people hold a tie
/// with, who builds it; were the kin asked, nobody would ever begin it.
#[test]
fn kin_with_nobody_of_an_age_to_build_are_passed_over_for_a_tied_neighbour() {
    let (mut sim, asker, others) = unhoused(3, 67.0, 7.0);
    let kin = others[0];
    make_kin(&mut sim, asker, kin);
    let now = sim.now();
    let born = SimTime::from_minutes(now.minutes() - (66.0 * 525_600.0) as i64);
    for (_, p) in sim.people_mut_for_tests().people.iter_mut() {
        if p.household == kin {
            p.born = born;
        }
    }
    let home = until_roofed(&mut sim, asker, 365).expect("a home roofed within the year");
    assert_eq!(building(&sim, home).household, asker);
    assert!(
        tie(&sim, asker, kin).is_none_or(|t| t.reason.map(|r| r.act) != Some(Act::HelpedMend)),
        "not built by the kin"
    );
    let me = sim
        .people()
        .household(asker)
        .and_then(|x| x.members.first().copied())
        .expect("someone");
    let pop = sim.people();
    let helped: Vec<PermanentId> = pop
        .ties
        .of(me)
        .iter()
        .filter(|t| t.reason.map(|r| r.act) == Some(Act::HelpedMend))
        .filter_map(|t| pop.person(t.to).map(|p| p.household))
        .collect();
    assert_eq!(helped.len(), 1, "one household built it: {helped:?}");
    assert_ne!(helped[0], kin);
    assert_ne!(helped[0], asker);
}

/// §8 25's second item: the one asked for a first home must be able to build the home the asker
/// would begin, as it must be of an age to. Its kin know no way to build the hut its ground
/// calls for (the roundhouse technique, forgotten here), so they are passed over for a
/// neighbour its people hold a tie with, who knows it and builds it; were the kin asked, they
/// would be asked every day and nobody would ever begin it.
#[test]
fn kin_who_cannot_build_the_home_asked_for_are_passed_over_for_a_tied_neighbour() {
    let (mut sim, asker, others) = unhoused(3, 67.0, 7.0);
    let kin = others[0];
    make_kin(&mut sim, asker, kin);
    let roundhouse = sim
        .rules()
        .catalog
        .technique_index("core:technique/roundhouse")
        .expect("the hut's technique");
    for (_, p) in sim.people_mut_for_tests().people.iter_mut() {
        if p.household == kin {
            p.knows.retain(|k| usize::from(k.technique) != roundhouse);
        }
    }
    let home = until_roofed(&mut sim, asker, 365).expect("a home roofed within the year");
    assert_eq!(building(&sim, home).household, asker);
    let me = sim
        .people()
        .household(asker)
        .and_then(|x| x.members.first().copied())
        .expect("someone");
    let pop = sim.people();
    let helped: Vec<PermanentId> = pop
        .ties
        .of(me)
        .iter()
        .filter(|t| t.reason.map(|r| r.act) == Some(Act::HelpedMend))
        .filter_map(|t| pop.person(t.to).map(|p| p.household))
        .collect();
    assert_eq!(helped.len(), 1, "one household built it: {helped:?}");
    assert_ne!(helped[0], kin, "not the kin, who cannot build it");
    assert_ne!(helped[0], asker);
}

/// The one who stands for household `household` (the population's elder): its first
/// member of an age to keep a household, or else its first.
fn elder(sim: &Sim, household: PermanentId) -> PermanentId {
    let (pop, now) = (sim.people(), sim.now());
    let grown = sim.rules().people.family.independent_age;
    let x = pop.household(household).expect("the household");
    x.members
        .iter()
        .copied()
        .find(|&m| pop.person(m).is_some_and(|p| p.age_years(now) >= grown))
        .or_else(|| x.members.first().copied())
        .expect("someone")
}

/// Whether someone of household `household` is of an age a build activity admits now, and
/// whether someone of it may build the hut (of that age and knowing `technique`).
fn builders(sim: &Sim, household: PermanentId, technique: usize) -> (bool, bool) {
    let (pop, now) = (sim.people(), sim.now());
    let ages: Vec<(f64, f64)> = content()
        .catalog
        .activities
        .iter()
        .filter(|a| a.behavior == civ_agents::params::Behavior::Build)
        .map(|a| (a.min_age_years, a.max_age_years))
        .collect();
    let of_age = |p: &civ_agents::Person| {
        let age = p.age_years(now);
        ages.iter().any(|&(lo, hi)| age >= lo && age <= hi)
    };
    let people: Vec<&civ_agents::Person> = pop
        .household(household)
        .expect("the household")
        .members
        .iter()
        .filter_map(|&m| pop.person(m))
        .collect();
    (
        people.iter().any(|p| of_age(p)),
        people.iter().any(|p| of_age(p) && p.knows(technique)),
    )
}

/// The households of `asker`'s settlement, other than `asker` and `kin`, whose elders `asker`'s
/// people hold a tie with, in the order a household asks them for help (`mend_candidates`'s
/// order: the best regarded first, ties broken by the lower id), with that regard.
fn tied_in_order(sim: &Sim, asker: PermanentId, kin: PermanentId) -> Vec<(PermanentId, f64)> {
    let pop = sim.people();
    let (day, tp) = (sim.now().day_index(), &sim.rules().people.ties);
    let x = pop.household(asker).expect("the asker");
    let mut tied: Vec<(PermanentId, f64)> = Vec::new();
    for &m in &x.members {
        for t in pop.ties.of(m) {
            let Some(h) = pop.person(t.to).map(|p| p.household) else {
                continue;
            };
            let near = pop
                .household(h)
                .is_some_and(|o| !o.members.is_empty() && o.settlement == x.settlement);
            if h == asker || h == kin || !near || elder(sim, h) != t.to {
                continue;
            }
            let regard = t.regard_at(day, tp);
            match tied.iter_mut().find(|(o, _)| *o == h) {
                Some(e) => e.1 = e.1.max(regard),
                None => tied.push((h, regard)),
            }
        }
    }
    tied.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    tied
}

/// The eighteenth coverage pass (§9; §8 26): the one asked for a first home must both be able to
/// build the home the asker would begin and not refuse the asker (research 11-04 §1.6: help is
/// given as food is, unless the helper believes the asker took from it). The kin, who cannot
/// build the hut, are passed over; the best regarded neighbour, who could build it, believes the
/// asker's elder took from its store and refuses; the next, who can build it and does not
/// refuse, is asked, begins the home, builds its first stage and is owed the work. Were the
/// refuser asked, it would begin the home of one it holds a thief.
#[test]
fn a_tied_neighbour_who_could_build_a_first_home_but_refuses_the_asker_is_passed_over_for_the_next()
{
    use civ_agents::crime::{Belief, Incident, KnownAmount, Outcome, Source};
    let (mut sim, asker, others) = unhoused(3, 67.0, 7.0);
    let kin = others[0];
    make_kin(&mut sim, asker, kin);
    let roundhouse = sim
        .rules()
        .catalog
        .technique_index("core:technique/roundhouse")
        .expect("the hut's technique");
    assert_eq!(hut_def().technique, Some(roundhouse), "the hut needs it");
    for (_, p) in sim.people_mut_for_tests().people.iter_mut() {
        if p.household == kin {
            p.knows.retain(|k| usize::from(k.technique) != roundhouse);
        }
    }
    let program = sim
        .with_ctx_for_tests(|pop, ctx| pop.first_home_for_tests(ctx, asker))
        .expect("a first home it would lay out")
        .program;
    assert_eq!(
        program, "core:building/hut",
        "the asker's ground calls for a hut"
    );
    assert_eq!(
        builders(&sim, asker, roundhouse),
        (false, false),
        "nobody of the asker of an age to build"
    );
    assert_eq!(
        builders(&sim, kin, roundhouse),
        (true, false),
        "the kin are of an age to build, so would be asked, but cannot build the hut"
    );
    // The first two other neighbours who can build the hut: the first is made the best regarded
    // of those the asker's elder holds a tie with, the second the next.
    let able: Vec<PermanentId> = others[1..]
        .iter()
        .copied()
        .filter(|&h| builders(&sim, h, roundhouse).1)
        .collect();
    assert!(able.len() >= 2, "two neighbours who can build it: {able:?}");
    let (refuser, helper) = (able[0], able[1]);
    // Both housed from the first day, in a finished hut at their homes: either, asked, would
    // begin the asker's home at once (a household begins another's first home only once it is
    // housed itself).
    for h in [refuser, helper] {
        let home = sim.people().household(h).expect("it").home;
        let spec = build::design_shape(
            hut_def(),
            &content().catalog.goods,
            Shape::Round { radius: 310 },
            home,
            None,
        )
        .expect("a hut");
        sim.place_building_for_tests(h, spec, Stage::ALL.len() as u8)
            .expect("placed");
        assert!(housed(&sim, h), "{h:?} housed");
    }
    let me = elder(&sim, asker);
    assert_eq!(
        Some(me),
        sim.people()
            .household(asker)
            .and_then(|x| x.members.first().copied()),
        "the asker's elder is its first, of 67"
    );
    let (refuser_elder, helper_elder) = (elder(&sim, refuser), elder(&sim, helper));
    let (tp, day) = (sim.rules().people.ties.clone(), sim.now().day_index());
    {
        let pop = sim.people_mut_for_tests();
        pop.ties
            .record(me, refuser_elder, Act::Hearth, 800.0, 0.0, day, &tp);
        pop.ties
            .record(me, helper_elder, Act::Hearth, 400.0, 0.0, day, &tp);
    }
    // The refuser's elder saw the asker's elder take from its store today.
    let now = sim.now();
    let incident = {
        let order = &mut sim.people_mut_for_tests().order;
        let id = order.incidents.last().map_or(1, |i| i.id + 1);
        order.incidents.push(Incident {
            id,
            at: now,
            actor: me,
            actor_household: asker,
            target: refuser,
            place: (0.0, 0.0),
            outcome: Outcome::Taken,
            goods: Vec::new(),
            kcal: 1_500.0,
            seen_by: vec![refuser_elder],
            noticed: true,
            food_days: f32::NAN,
            richer: f32::NAN,
            across: None,
        });
        order.amounts.push(KnownAmount {
            household: refuser,
            incident: id,
            kcal: 1_500.0,
            lost: true,
            day,
        });
        id
    };
    sim.with_ctx_for_tests(|pop, ctx| {
        pop.come_to_believe_for_tests(
            ctx,
            Belief {
                holder: refuser_elder,
                incident,
                taker: Some(me),
                source: Source::Saw,
                from: None,
                origin: Some(refuser_elder),
                day,
            },
        );
    });
    // Who believes the asker's elder took, and from whom: someone of household `h`, that it took
    // from `from` within the days remembered.
    let believes = |sim: &Sim, h: PermanentId, from: &dyn Fn(PermanentId) -> bool| {
        let pop = sim.people();
        let since = sim.now().day_index() - i64::from(sim.rules().people.crime.remember_days);
        pop.household(h)
            .expect("it")
            .members
            .iter()
            .any(|&m| pop.order.believes_took(m, me, since, from))
    };
    // Whether household `h` refuses the asker's elder, as the engine has it: someone of it
    // believes that elder took from it, or from a household whose elder they regard enough.
    let refuses = |sim: &Sim, h: PermanentId| {
        let pop = sim.people();
        let (day, tp) = (sim.now().day_index(), &sim.rules().people.ties);
        let cp = &sim.rules().people.crime;
        let since = day - i64::from(cp.remember_days);
        pop.household(h).expect("it").members.iter().any(|&m| {
            pop.order.believes_took(m, me, since, &|target| {
                target == h
                    || pop.household(target).is_some_and(|x| !x.members.is_empty())
                        && pop.ties.regard(m, elder(sim, target), day, tp) >= cp.refuse_regard
            })
        })
    };
    // At first nobody but the refuser's elder knows of it.
    assert!(
        believes(&sim, refuser, &|t| t == refuser),
        "the refuser's elder saw it"
    );
    assert!(
        !believes(&sim, helper, &|_| true),
        "the helper knows nothing of it"
    );
    assert!(
        !believes(&sim, kin, &|_| true),
        "the kin know nothing of it"
    );
    let ordered = |sim: &Sim| {
        let tied = tied_in_order(sim, asker, kin);
        assert!(tied.len() >= 2, "two tied neighbours: {tied:?}");
        assert_eq!(
            (tied[0].0, tied[1].0),
            (refuser, helper),
            "after the kin, the refuser is asked first, then the helper: {tied:?}"
        );
    };
    let prerequisites = |sim: &Sim| {
        ordered(sim);
        assert!(
            believes(sim, refuser, &|t| t == refuser),
            "the refuser believes the asker's elder took from its store"
        );
        assert!(refuses(sim, refuser), "so it refuses the asker");
        assert!(!refuses(sim, helper), "the helper does not refuse it");
        assert!(!refuses(sim, kin), "nor do the kin");
        assert_eq!(
            builders(sim, kin, roundhouse),
            (true, false),
            "the kin of an age to build, but not knowing how"
        );
        assert!(
            builders(sim, refuser, roundhouse).1,
            "the refuser could build it"
        );
        assert!(
            builders(sim, helper, roundhouse).1,
            "the helper can build it"
        );
    };
    prerequisites(&sim);
    // Lived an hour at a time until its home is begun: as things stood when it was begun, and
    // begun by the helper, whose people set to work on it (a home is begun when someone of the
    // household asked sets to work on it).
    assert!(
        hour_by_hour(&mut sim, 365 * 24, |s| !dwellings(s, asker).is_empty()),
        "its home begun within the year"
    );
    prerequisites(&sim);
    let begun = dwellings(&sim, asker);
    assert_eq!(begun.len(), 1, "one home begun: {begun:?}");
    let at_work: Vec<PermanentId> = {
        let pop = sim.people();
        let mut v: Vec<PermanentId> = pop
            .people
            .iter()
            .map(|(_, p)| p)
            .filter(|p| p.act.target == civ_agents::person::Target::Building(begun[0].0))
            .map(|p| p.household)
            .collect();
        v.sort_unstable();
        v.dedup();
        v
    };
    assert_eq!(
        at_work,
        vec![helper],
        "begun by the helper, not the refuser ({refuser:?}) nor the kin ({kin:?})"
    );
    // The kin, known once the day has worked out who is kin, come before every tied neighbour.
    assert_eq!(
        sim.people().kin_households_for_tests(asker),
        vec![kin],
        "its only kin, asked first"
    );
    // Lived until its first stage is done: the stage built by the one asked when nothing was
    // begun, which the asker then owes.
    let mut staged = false;
    for _ in 0..365 {
        sim.advance_minutes(24 * 60).expect("advances");
        if dwellings(&sim, asker).iter().any(|&(_, stage)| stage >= 1) {
            staged = true;
            break;
        }
    }
    assert!(staged, "its first stage built within the year");
    let owed = |sim: &Sim, a: PermanentId, b: PermanentId| {
        let (day, tp) = (sim.now().day_index(), &sim.rules().people.ties);
        sim.people().ties.tie(a, b, day, tp).map(|t| t.help_h)
    };
    let built = |sim: &Sim, h: PermanentId| {
        let x = sim.people().household(h).expect("it");
        (0..sim.rules().catalog.goods.len())
            .map(|g| x.flows.get(civ_agents::person::Flow::Built, g))
            .sum::<f64>()
    };
    // Asked: the helper, through the tie the asker's elder holds to its elder; not the refuser,
    // nor the kin.
    let pop = sim.people();
    let helped: Vec<(PermanentId, PermanentId)> = pop
        .ties
        .of(me)
        .iter()
        .filter(|t| t.reason.map(|r| r.act) == Some(Act::HelpedMend))
        .filter_map(|t| pop.person(t.to).map(|p| (p.household, t.to)))
        .collect();
    assert_eq!(
        helped,
        vec![(helper, helper_elder)],
        "the helper built it, not the refuser ({refuser:?}) nor the kin ({kin:?})"
    );
    assert!(
        owed(&sim, me, helper_elder).is_some_and(|h| h > 0.0),
        "the asker owes the helper the work: {:?}",
        owed(&sim, me, helper_elder)
    );
    assert!(
        owed(&sim, helper_elder, me).is_some_and(|h| h < 0.0),
        "the helper is owed: {:?}",
        owed(&sim, helper_elder, me)
    );
    assert!(
        owed(&sim, me, refuser_elder).is_none_or(|h| h <= 0.0),
        "nothing owed the refuser: {:?}",
        owed(&sim, me, refuser_elder)
    );
    assert_eq!(built(&sim, asker), 0.0, "nobody of the asker did the work");
    assert!(
        built(&sim, helper) > 0.0,
        "the helper brought the materials"
    );
    // And so on until it is roofed, by whoever is asked for each stage once it is begun (any but
    // the refuser, who is never asked).
    let home = until_roofed(&mut sim, asker, 365).expect("a home roofed within the year");
    assert_eq!(
        building(&sim, home).household,
        asker,
        "the asker's own home"
    );
    assert!(
        owed(&sim, me, refuser_elder).is_none_or(|h| h <= 0.0),
        "nothing owed the refuser: {:?}",
        owed(&sim, me, refuser_elder)
    );
    assert_eq!(built(&sim, asker), 0.0, "nobody of the asker did the work");
    let pop = sim.people();
    let problems = pop.problems(sim.ids().peek_next(), sim.rules().catalog.activities.len());
    assert!(problems.is_empty(), "{problems:?}");
    saves_and_goes_on_alike(&mut sim, 48);
    let pop = sim.people();
    let problems = pop.problems(sim.ids().peek_next(), sim.rules().catalog.activities.len());
    assert!(problems.is_empty(), "{problems:?}");
}

/// Saves `sim`, loads it, and lives the two side by side an hour at a time for `hours`, every
/// section of their saves alike after each (ADR-0011 §4: who asks whom for help with building
/// work is derived, worked out once a day and not saved, so a world loaded that day works it out
/// afresh from what holds then, and must find what the unsaved one kept).
fn saves_and_goes_on_alike(sim: &mut Sim, hours: u32) {
    use civ_sim::persist;
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "mid-day").expect("saves");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
    let differing = |a: &Sim, b: &Sim| -> Vec<String> {
        let (a, b) = (persist::encode_sections(a), persist::encode_sections(b));
        assert_eq!(a.len(), b.len());
        a.iter()
            .zip(&b)
            .filter(|(x, y)| x.bytes != y.bytes)
            .map(|(x, _)| x.tag.to_string())
            .collect()
    };
    assert_eq!(differing(sim, &loaded), Vec::<String>::new(), "as loaded");
    for h in 1..=hours {
        sim.advance_minutes(60).expect("advances");
        loaded.advance_minutes(60).expect("advances");
        let d = differing(sim, &loaded);
        assert!(d.is_empty(), "{h} h after the save these differ: {d:?}");
    }
}

/// Household `household`'s dwellings, with the stage each is at.
fn dwellings(sim: &Sim, household: PermanentId) -> Vec<(PermanentId, u8)> {
    let catalog = &sim.rules().catalog;
    sim.land()
        .buildings
        .iter()
        .filter(|b| b.household == household && catalog.is_dwelling(&b.spec.program))
        .map(|b| (b.id, b.stage))
        .collect()
}

/// Whether household `household` lives in a finished home of its own.
fn housed(sim: &Sim, household: PermanentId) -> bool {
    let catalog = &sim.rules().catalog;
    sim.land().buildings.iter().any(|b| {
        b.household == household
            && b.finished()
            && b.standing()
            && catalog.is_dwelling(&b.spec.program)
    })
}

/// Lives `sim` an hour at a time, at most `hours`, until `done` holds of it.
fn hour_by_hour(sim: &mut Sim, hours: u32, done: impl Fn(&Sim) -> bool) -> bool {
    for _ in 0..hours {
        sim.advance_minutes(60).expect("advances");
        if done(sim) {
            return true;
        }
    }
    false
}

/// §8 25's first item: a household asks its kin for a first home at the day's first need, and
/// the ask is kept until midnight. Saved on the hour its kin put the roof on, and again on the
/// hour they finish it (when it no longer asks), the world goes on as the unsaved one does for
/// two days each time: what the kin do for it is read against the home as it stands, a roof
/// already on or a home already finished, not against the morning's ask alone.
#[test]
fn a_first_home_saved_the_hour_kin_roof_it_and_the_hour_they_finish_it_goes_on_alike() {
    let (mut sim, asker, others) = unhoused(3, 67.0, 7.0);
    let kin = others[0];
    make_kin(&mut sim, asker, kin);
    for _ in 0..365 {
        sim.advance_minutes(24 * 60).expect("advances");
        if !dwellings(&sim, asker).is_empty() {
            break;
        }
    }
    assert_eq!(
        dwellings(&sim, asker).len(),
        1,
        "its home begun within the year"
    );
    assert!(
        hour_by_hour(&mut sim, 365 * 24, |s| roofed_home(s, asker).is_some()),
        "roofed within the year"
    );
    assert!(sim.now().minute_of_day() != 0, "roofed mid-day");
    assert!(!housed(&sim, asker), "roofed, not yet finished");
    saves_and_goes_on_alike(&mut sim, 48);
    assert!(!housed(&sim, asker), "not finished in those two days");
    assert!(
        hour_by_hour(&mut sim, 365 * 24, |s| housed(s, asker)),
        "finished within the year"
    );
    assert!(sim.now().minute_of_day() != 0, "finished mid-day");
    let t = tie(&sim, asker, kin).expect("a tie to the kin");
    assert_eq!(
        t.reason.map(|r| r.act),
        Some(Act::HelpedMend),
        "built by the kin"
    );
    saves_and_goes_on_alike(&mut sim, 48);
}

/// A household with a home begun and its kin housed, whose one member of an age to build;
/// lived to 10:00 of the next day: the world, the household, its kin, that member and its home.
fn about_to_lose_its_builder() -> (Sim, PermanentId, PermanentId, PermanentId, PermanentId) {
    let (mut sim, asker, others) = unhoused(3, 67.0, 30.0);
    let kin = others[0];
    make_kin(&mut sim, asker, kin);
    for _ in 0..365 {
        sim.advance_minutes(24 * 60).expect("advances");
        if housed(&sim, kin) {
            break;
        }
    }
    assert!(housed(&sim, kin), "the kin housed within the year");
    assert!(!housed(&sim, asker), "the asker not yet");
    let home = dwellings(&sim, asker);
    assert_eq!(home.len(), 1, "its own home begun: {home:?}");
    hour_by_hour(&mut sim, 24, |s| s.now().minute_of_day() == 10 * 60);
    let now = sim.now();
    let pop = sim.people();
    let builders: Vec<PermanentId> = pop
        .household(asker)
        .expect("it")
        .members
        .iter()
        .copied()
        .filter(|m| {
            pop.person(*m)
                .is_some_and(|p| (12.0..=65.0).contains(&p.age_years(now)))
        })
        .collect();
    assert_eq!(builders.len(), 1, "{builders:?}");
    (sim, asker, kin, builders[0], home[0].0)
}

/// §8 25's first item: a household whose one member of an age to build dies mid-day is from
/// then on a household with a home begun that nobody of it may work on, and asks its kin
/// (`members_changed`). A world saved the hour it dies goes on as the unsaved one does: before,
/// the unsaved world kept the morning's asks, in which it asked nobody, until midnight, while
/// the loaded one had its kin at work on its home that afternoon.
#[test]
fn a_world_saved_the_hour_an_askers_last_builder_dies_goes_on_alike() {
    let (mut sim, asker, kin, builder, home) = about_to_lose_its_builder();
    sim.die_for_tests(builder);
    assert!(sim.people().person(builder).is_none());
    assert!(
        sim.people().household(asker).is_some(),
        "the elder lives on"
    );
    let done = |sim: &Sim| {
        let b = building(sim, home);
        (b.stage, b.work_h)
    };
    let before = done(&sim);
    saves_and_goes_on_alike(&mut sim, 48);
    // Nobody of it may work on its home now: the work done on it is its kin's.
    assert!(done(&sim) > before, "{before:?} → {:?}", done(&sim));
    assert!(sim.people().household(kin).is_some());
}

/// §8 25's first item: who is of an age to build changes as the hours pass, not at midnight. A
/// household whose people of 30 are made to pass the build ages at noon, its home begun, is
/// saved at 13:00: the loaded world goes on as the unsaved one does, for the day's asks read
/// everyone's age as it was at the day's start, as a world loaded that day reads it.
#[test]
fn a_world_saved_after_an_askers_builders_pass_the_build_ages_mid_day_goes_on_alike() {
    let (mut sim, asker, _, builder, _) = about_to_lose_its_builder();
    hour_by_hour(&mut sim, 24, |s| s.now().minute_of_day() == 60);
    let noon = sim.now().day_index() * 1440 + 12 * 60;
    let past = build_hut_max_age_minutes() + 1;
    let pop = sim.people_mut_for_tests();
    let p = pop
        .people
        .iter_mut()
        .map(|(_, p)| p)
        .find(|p| p.id == builder)
        .expect("alive");
    p.born = SimTime::from_minutes(noon - past);
    hour_by_hour(&mut sim, 24, |s| s.now().minute_of_day() == 13 * 60);
    let age = sim
        .people()
        .person(builder)
        .expect("alive")
        .age_years(sim.now());
    assert!(age > 65.0, "past the build ages now: {age}");
    assert!(sim.people().household(asker).is_some());
    saves_and_goes_on_alike(&mut sim, 48);
}

/// The oldest a build activity admits, in minutes of age.
fn build_hut_max_age_minutes() -> i64 {
    let catalog = &content().catalog;
    let max = catalog
        .activities
        .iter()
        .filter(|a| a.behavior == civ_agents::params::Behavior::Build)
        .map(|a| a.max_age_years)
        .fold(0.0_f64, f64::max);
    assert_eq!(max, 65.0, "core's build ages");
    (max * 525_600.0) as i64
}
