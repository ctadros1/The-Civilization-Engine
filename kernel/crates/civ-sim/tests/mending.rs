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
