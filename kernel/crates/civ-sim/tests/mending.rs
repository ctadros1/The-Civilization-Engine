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
