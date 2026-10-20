//! A resolution per settlement (M9a slice DF, ADR-0032 §1, §3): a settlement switched to
//! household resolution has its people parked at a midnight and lives its day then; the switch
//! changes nobody, no holding and no good the books do not account for; a save of such a world
//! lives on as if never saved once the setting is made again; and a settlement brought back takes
//! up its day.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use civ_agents::person::Flow;
use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_core::time::MINUTES_PER_DAY;
use civ_sim::persist::{self, SECTION_META};
use civ_sim::{NewWorld, Sim};
use commons_persist::{SaveDir, SaveKind, SectionData};

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

/// Two founding groups on a river valley with a fixed identity, lived ten days to a midnight.
fn two_settlements() -> Sim {
    let mut sim = Sim::create_for_tests(
        &NewWorld {
            name: "Resolutions".to_owned(),
            seed: 3,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 768,
            band_size: 0,
            neighbours: vec![0],
            neighbours_known: true,
            regime_id: String::new(),
        },
        content(),
        [7; 16],
    )
    .expect("generates");
    assert_eq!(sim.land().settlements.len(), 2);
    sim.advance_minutes(10 * MINUTES_PER_DAY).expect("advances");
    sim.advance_to_midnight().expect("advances");
    sim
}

/// The second settlement's id.
fn second(sim: &Sim) -> PermanentId {
    sim.land().settlements[1].id
}

/// The people living in settlement `s`.
fn residents(sim: &Sim, s: PermanentId) -> Vec<PermanentId> {
    let people = sim.people();
    people
        .people
        .iter()
        .filter(|(_, p)| {
            people
                .household(p.household)
                .and_then(|x| x.settlement)
                .is_some_and(|x| x == s)
        })
        .map(|(_, p)| p.id)
        .collect()
}

fn save(sim: &mut Sim, dir: &Path, label: &str) -> PathBuf {
    let saves = SaveDir::create(dir, civ_schema::SAVE_EXTENSION).expect("save dir");
    persist::save(sim, &saves, SaveKind::Manual, label)
        .expect("saves")
        .path
}

fn load(path: &Path) -> Sim {
    persist::load(path, content()).expect("loads")
}

fn differing(a: &[SectionData], b: &[SectionData]) -> Vec<String> {
    if a.len() != b.len() {
        return vec![format!("{} sections against {}", a.len(), b.len())];
    }
    a.iter()
        .zip(b)
        .filter(|(x, y)| x.tag != SECTION_META && (x.tag != y.tag || x.bytes != y.bytes))
        .map(|(x, _)| format!("{} {}", x.tag, x.index))
        .collect()
}

fn assert_same(what: &str, a: &Sim, b: &Sim) {
    let d = differing(&persist::encode_sections(a), &persist::encode_sections(b));
    assert!(d.is_empty(), "{what}: sections differ: {d:?}");
}

#[test]
fn a_switch_undone_before_it_takes_effect_changes_nothing() {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let mut sim = two_settlements();
    let start = save(&mut sim, dir.path(), "start");
    let mut straight = load(&start);
    straight
        .advance_minutes(6 * MINUTES_PER_DAY)
        .expect("advances");
    // Set and undone at a midnight, then held and undone between midnights.
    let mut undone = load(&start);
    let s = second(&undone);
    undone.set_household_resolved([s]);
    undone.set_household_resolved([]);
    undone
        .advance_minutes(MINUTES_PER_DAY / 2)
        .expect("advances");
    undone.set_household_resolved([s]);
    undone.set_household_resolved([]);
    undone
        .advance_minutes(6 * MINUTES_PER_DAY - MINUTES_PER_DAY / 2)
        .expect("advances");
    assert!(undone.household_resolved().is_empty());
    assert_same("a switch undone", &straight, &undone);
}

#[test]
fn a_household_resolved_settlement_is_parked_and_lives_its_needs_at_midnight() {
    let mut sim = two_settlements();
    let s = second(&sim);
    let first = sim.land().settlements[0].id;
    // Held between midnights: nothing changes until the next one.
    sim.advance_minutes(MINUTES_PER_DAY / 3).expect("advances");
    sim.set_household_resolved([s]);
    assert!(sim.household_resolved().is_empty(), "held for midnight");
    sim.advance_to_midnight().expect("advances");
    assert_eq!(sim.household_resolved(), &BTreeSet::from([s]));
    let eaten = |sim: &Sim| {
        sim.people()
            .households
            .iter()
            .filter(|(_, x)| x.settlement == Some(s))
            .map(|(_, x)| {
                (0..x.stores.len())
                    .map(|g| x.flows.get(Flow::Eaten, g).abs())
                    .sum::<f64>()
            })
            .sum::<f64>()
    };
    let before = eaten(&sim);
    sim.advance_minutes(3 * MINUTES_PER_DAY + 600)
        .expect("advances");
    assert!(sim.people().resolution_problems().is_empty());
    let parked = residents(&sim, s);
    assert!(!parked.is_empty());
    for id in &parked {
        let p = sim.people().person(*id).expect("lives");
        assert!(p.act.steps.is_empty(), "{} is parked", p.given);
        let home = sim.people().household(p.household).expect("household").home;
        assert_eq!(p.pos, home, "{} stands at home", p.given);
        assert!(p.trip.is_none());
    }
    assert!(
        eaten(&sim) > before,
        "they eat from their stores at midnight"
    );
    // The other settlement lives through the day as ever.
    assert!(residents(&sim, first).iter().all(|id| {
        !sim.people()
            .person(*id)
            .expect("lives")
            .act
            .steps
            .is_empty()
    }));
}

#[test]
fn parking_changes_nobody_no_holding_and_no_good_beyond_the_books() {
    let mut sim = two_settlements();
    // Mid-morning, when people are out with loads.
    sim.advance_minutes(10 * 60).expect("advances");
    let s = second(&sim);
    let before = (
        sim.people().switch_invariants(sim.land()),
        sim.people().goods_held(),
        sim.people().flows(),
    );
    sim.with_ctx_for_tests(|people, ctx| {
        people.set_household_resolved(BTreeSet::from([s]));
        people.park_resolved(ctx);
    });
    let after = (
        sim.people().switch_invariants(sim.land()),
        sim.people().goods_held(),
        sim.people().flows(),
    );
    assert_eq!(before.0, after.0, "nobody and no holding changed");
    let gaps = civ_agents::population::unaccounted(
        before.1.len().max(after.1.len()),
        (&before.1, &before.2),
        (&after.1, &after.2),
    );
    assert!(gaps.is_empty(), "the books balance: {gaps:?}");
    assert!(
        residents(&sim, s)
            .iter()
            .all(|id| sim.people().person(*id).expect("lives").carrying.kg == 0.0),
        "what they carried is banked"
    );
}

#[test]
fn a_save_of_a_household_resolved_world_lives_on_as_if_never_saved() {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let mut sim = two_settlements();
    let s = second(&sim);
    sim.set_household_resolved([s]);
    sim.advance_minutes(3 * MINUTES_PER_DAY).expect("advances");
    let mid = save(&mut sim, dir.path(), "resolved");
    sim.advance_minutes(5 * MINUTES_PER_DAY).expect("advances");
    let mut resumed = load(&mid);
    assert!(resumed.household_resolved().is_empty(), "never saved");
    resumed.set_household_resolved([s]);
    resumed
        .advance_minutes(5 * MINUTES_PER_DAY)
        .expect("advances");
    assert_same("saved household-resolved and set again", &sim, &resumed);
}

#[test]
fn a_loaded_world_not_set_again_takes_up_its_day_at_once() {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let mut sim = two_settlements();
    let s = second(&sim);
    sim.set_household_resolved([s]);
    sim.advance_minutes(2 * MINUTES_PER_DAY).expect("advances");
    let mid = save(&mut sim, dir.path(), "resolved");
    let mut loaded = load(&mid);
    loaded.advance_minutes(1).expect("advances");
    assert!(loaded.people().resolution_problems().is_empty());
    assert!(
        residents(&loaded, s).iter().all(|id| !loaded
            .people()
            .person(*id)
            .expect("lives")
            .act
            .steps
            .is_empty()),
        "everyone decided"
    );
}

#[test]
fn a_settlement_brought_back_takes_up_its_day() {
    let mut sim = two_settlements();
    let s = second(&sim);
    sim.set_household_resolved([s]);
    sim.advance_minutes(3 * MINUTES_PER_DAY).expect("advances");
    let at_home: Vec<_> = residents(&sim, s)
        .iter()
        .map(|id| sim.people().person(*id).expect("lives").pos)
        .collect();
    sim.set_household_resolved([]);
    sim.advance_minutes(MINUTES_PER_DAY + 13 * 60)
        .expect("advances");
    assert!(sim.household_resolved().is_empty());
    assert!(sim.people().resolution_problems().is_empty());
    let now: Vec<_> = residents(&sim, s)
        .iter()
        .map(|id| sim.people().person(*id).expect("lives").pos)
        .collect();
    assert_ne!(at_home, now, "they are about their day again");
}

#[test]
fn switching_back_and_forth_keeps_everyone_and_their_holdings() {
    // The kernel checks every switch itself with debug checks on (the suite's profile); here a
    // settlement is switched every few days and its people are all still there, each household
    // where it was.
    let mut sim = two_settlements();
    let s = second(&sim);
    let people_before = residents(&sim, s).len();
    for i in 0..6 {
        let set: Vec<_> = if i % 2 == 0 { vec![s] } else { Vec::new() };
        sim.set_household_resolved(set);
        sim.advance_minutes(2 * MINUTES_PER_DAY).expect("advances");
        assert!(sim.people().resolution_problems().is_empty(), "switch {i}");
    }
    let people_after = residents(&sim, s).len();
    assert!(
        people_after + 2 >= people_before,
        "nobody lost to a switch: {people_before} then {people_after}"
    );
    assert!(sim.people().problems(u64::MAX, usize::MAX).is_empty());
}
