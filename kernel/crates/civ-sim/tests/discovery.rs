//! Discovery and the first new crafts (M3b slice N, ADR-0008 §3): people try at a problem at
//! home and may find a technique that answers it, a find the chronicle names by its kind; what is
//! found is used (dried meat keeps); a find needs what it needs at home; and the rotary quern, a
//! far technique, comes in practice from elsewhere, as the observer's introduction.

use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

use civ_agents::ChronicleKind;
use civ_agents::history::FOUND_FIRST_ANYWHERE;
use civ_agents::person::KnowSource;
use civ_content::ContentRegistry;
use civ_core::PermanentId;
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

/// The core content with drying found by nearly every hour's trying (half of those trying find
/// it in three minutes), and needing `needs` at home to try.
fn quick_drying(needs: Option<&str>) -> ContentRegistry {
    let mut c = content().clone();
    let t = c
        .catalog
        .technique_index("core:technique/drying")
        .expect("drying");
    c.catalog.techniques[t].e50_h = 0.05;
    if let Some(id) = needs {
        let g = c.catalog.goods.iter().position(|g| g.id == id).expect(id);
        c.catalog.techniques[t].needs = vec![g];
    }
    c
}

fn world(content: &ContentRegistry, seed: u64) -> Sim {
    Sim::create(
        &NewWorld {
            name: "Finds".to_owned(),
            seed,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 512,
            band_size: 0,
            regime_id: String::new(),
        },
        content,
        &mut |_| {},
        &AtomicBool::new(false),
    )
    .expect("generates")
}

fn good(sim: &Sim, id: &str) -> usize {
    sim.rules()
        .catalog
        .goods
        .iter()
        .position(|g| g.id == id)
        .expect(id)
}

fn technique(sim: &Sim, id: &str) -> usize {
    sim.rules().catalog.technique_index(id).expect(id)
}

/// Gives the first household of two or more far more meat than it can eat before it spoils, and
/// firewood; its id and members.
fn stock_meat(sim: &mut Sim) -> (PermanentId, Vec<PermanentId>) {
    let (meat, firewood) = (good(sim, "core:good/meat"), good(sim, "core:good/firewood"));
    let pop = sim.people_mut_for_tests();
    let (_, h) = pop
        .households
        .iter_mut()
        .filter(|(_, h)| h.members.len() >= 2)
        .min_by_key(|(_, h)| h.id)
        .expect("a household");
    h.stores[meat] += 150.0;
    h.stores[firewood] += 60.0;
    (h.id, h.members.clone())
}

fn knowers(sim: &Sim, t: usize) -> Vec<PermanentId> {
    sim.people()
        .people
        .iter()
        .map(|(_, p)| p)
        .filter(|p| p.knows(t))
        .map(|p| p.id)
        .collect()
}

fn stores_of(sim: &Sim, household: PermanentId, g: usize) -> f64 {
    sim.people()
        .households
        .iter()
        .find(|(_, h)| h.id == household)
        .map_or(0.0, |(_, h)| h.stores[g])
}

#[test]
fn a_household_with_meat_spoiling_tries_finds_drying_and_dries_its_meat() {
    let variant = quick_drying(None);
    let mut sim = world(&variant, 3);
    let drying = technique(&sim, "core:technique/drying");
    assert!(knowers(&sim, drying).is_empty(), "nobody knows it at first");
    let (household, members) = stock_meat(&mut sim);
    let mut found = None;
    for _ in 0..20 {
        sim.advance_minutes(6 * 60).expect("advances");
        found = sim
            .people()
            .chronicle
            .iter()
            .find(|e| e.kind == ChronicleKind::TechniqueFound)
            .cloned();
        if found.is_some() {
            break;
        }
    }
    let found = found.expect("someone tried at the spoiling meat and found drying");
    let finder = found.people[0];
    assert!(
        members.contains(&finder),
        "a member of the household with the meat"
    );
    assert_eq!(found.number.round() as i64, FOUND_FIRST_ANYWHERE);
    let p = sim.people().person(finder).expect("alive");
    assert_eq!(p.know(drying).map(|k| k.source), Some(KnowSource::Found));
    assert!(p.tried.is_some(), "they had tried");
    // The settlement's record names the find.
    assert!(
        sim.people()
            .knowledge
            .iter()
            .any(|e| usize::from(e.technique) == drying && e.person == finder)
    );
    // And the household dries what would spoil.
    let dried = good(&sim, "core:good/dried_meat");
    let mut made = 0.0;
    for _ in 0..12 {
        sim.advance_minutes(6 * 60).expect("advances");
        made = stores_of(&sim, household, dried);
        if made > 0.0 {
            break;
        }
    }
    assert!(made > 0.0, "the household dried some of its meat");
}

#[test]
fn nobody_tries_toward_what_they_lack_the_means_for() {
    // Drying needs dried meat at home to try here: nobody has any, so nobody tries or finds it.
    let variant = quick_drying(Some("core:good/dried_meat"));
    let mut sim = world(&variant, 3);
    let drying = technique(&sim, "core:technique/drying");
    stock_meat(&mut sim);
    sim.advance_minutes(3 * 24 * 60).expect("advances");
    assert!(knowers(&sim, drying).is_empty());
    assert!(sim.people().people.iter().all(|(_, p)| p.tried.is_none()));
}

#[test]
fn the_observer_brings_in_the_rotary_quern_and_its_household_makes_one() {
    let mut sim = world(content(), 3);
    // After the founding rush: in a world's first weeks the eldest can spend every day on fields
    // and the first huts (in about one run in eight they made no quern in 30 days), which is
    // what they would do, but not what this tests.
    sim.advance_minutes(60 * 24 * 60).expect("advances");
    let rotary = technique(&sim, "core:technique/rotary_quern");
    let (stone, timber, quern) = (
        good(&sim, "core:good/stone"),
        good(&sim, "core:good/timber"),
        good(&sim, "core:good/rotary_quern"),
    );
    // The eldest adult of the first household learns it from the observer; the household has
    // stone and wood for one.
    let (household, who) = {
        let pop = sim.people();
        let now = sim.now();
        let (_, h) = pop
            .households
            .iter()
            .min_by_key(|(_, h)| h.id)
            .expect("a household");
        let who = h
            .members
            .iter()
            .filter_map(|m| pop.person(*m))
            .filter(|p| p.age_years(now) >= 18.0)
            .max_by(|a, b| a.age_years(now).total_cmp(&b.age_years(now)))
            .map(|p| p.id)
            .expect("an adult");
        (h.id, who)
    };
    for (_, h) in sim.people_mut_for_tests().households.iter_mut() {
        if h.id == household {
            h.stores[stone] += 60.0;
            h.stores[timber] += 10.0;
        }
    }
    sim.introduce_technique(who, "core:technique/rotary_quern", false)
        .expect("introduced");
    assert!(sim.people().person(who).is_some_and(|p| p.knows(rotary)));
    let mut has = 0.0;
    for _ in 0..30 {
        sim.advance_minutes(24 * 60).expect("advances");
        has = stores_of(&sim, household, quern);
        if has > 0.0 {
            break;
        }
    }
    assert!(has > 0.0, "the household made a rotary quern");
    // No other household wants one: none knows how to use it.
    let others: f64 = sim
        .people()
        .households
        .iter()
        .filter(|(_, h)| h.id != household)
        .map(|(_, h)| h.stores[quern])
        .sum();
    assert_eq!(others, 0.0);
}
