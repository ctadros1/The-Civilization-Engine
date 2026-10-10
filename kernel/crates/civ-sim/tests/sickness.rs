//! Sickness (M6a slice AZ, step one; ADR-0021 §5, §8; research 05-03): no disease without an
//! introduction; the observer brings one to a person, as if they took it elsewhere; its course
//! runs by its own clocks; the ill keep to their beds; a household takes it from members who shed
//! it; the severely ill die of it by their own draws, the record naming the disease; and
//! infections save and load exactly.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::sickness::{Acquired, Outcome};
use civ_agents::{Behavior, Cause, ChronicleKind};
use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_sim::{NewWorld, Sim, persist};

const PRESET: &str = "core:worldgen/river_valley";
const CHOLERA: &str = "core:disease/cholera";
const DAY: i64 = 24 * 60;

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

/// A village of world `seed`, with a fixed identity so it lives the same life every run.
fn village(seed: u64) -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Fevermere".to_owned(),
            seed,
            preset_id: PRESET.to_owned(),
            size_cells: 768,
            band_size: 30,
            neighbours: Vec::new(),
            neighbours_known: true,
            regime_id: String::new(),
        },
        content(),
        [9; 16],
    )
    .expect("generates")
}

/// Lives on to `minute` past the next midnight.
fn to_morning(sim: &mut Sim, minute: i64) {
    let rest = DAY - sim.now().minutes().rem_euclid(DAY) + minute;
    sim.advance_minutes(rest).expect("advances");
}

/// The index of cholera in the catalog.
fn cholera(sim: &Sim) -> usize {
    sim.rules()
        .catalog
        .disease_index(CHOLERA)
        .expect("cholera is content")
}

/// Sets cholera's numbers for a test: `f` changes its definition.
fn set_cholera(sim: &mut Sim, f: impl FnOnce(&mut civ_agents::params::DiseaseDef)) {
    let i = cholera(sim);
    let rules = sim.rules_mut_for_tests().expect("the rules are not shared");
    f(&mut rules.catalog.diseases[i]);
}

/// A member of the largest household, and that household's other members.
fn someone_with_family(sim: &Sim) -> (PermanentId, Vec<PermanentId>) {
    let x = sim
        .people()
        .households
        .iter()
        .map(|(_, x)| x)
        .max_by_key(|x| (x.members.len(), std::cmp::Reverse(x.id)))
        .expect("a household");
    assert!(x.members.len() >= 2, "a household of two or more");
    (x.members[0], x.members[1..].to_vec())
}

#[test]
fn a_world_with_no_introduction_never_has_a_case() {
    let mut sim = village(3);
    to_morning(&mut sim, 0);
    sim.advance_minutes(20 * DAY).expect("lives");
    assert!(sim.people().sickness.episodes().is_empty());
    assert!(
        sim.people()
            .records
            .values()
            .all(|r| r.died.is_none_or(|(_, c)| c != Cause::Disease))
    );
}

#[test]
fn the_observer_brings_a_disease_to_one_person_and_its_course_runs() {
    let mut sim = village(3);
    set_cholera(&mut sim, |d| {
        d.household_hazard = 0.0;
        d.severe_death_per_day = 0.0;
    });
    to_morning(&mut sim, 9 * 60);
    let (target, _) = someone_with_family(&sim);
    let today = sim.now().day_index();
    assert!(sim.plague(target, "core:disease/nothing").is_err());
    let reached = sim.plague(target, CHOLERA).expect("brought");
    let index = cholera(&sim);
    let e = sim.people().sickness.episodes()[0];
    assert_eq!(sim.people().sickness.episodes().len(), 1);
    assert_eq!(
        (e.person, usize::from(e.disease), e.infected),
        (target, index, today)
    );
    assert_eq!(
        e.acquired,
        Acquired::Observer {
            influence: reached.id
        }
    );
    let record = sim
        .people()
        .influences
        .list
        .iter()
        .find(|i| i.id == reached.id)
        .expect("recorded");
    assert_eq!(record.kind, civ_agents::influence::InfluenceKind::Plague);
    assert_eq!(record.subject as usize, index);
    assert!(sim.people().chronicle.iter().any(|c| {
        c.kind == ChronicleKind::Influence
            && c.name.contains("cholera, as if they took it elsewhere")
    }));
    // A second time is refused while it runs in them.
    let again = sim.plague(target, CHOLERA).expect_err("refused");
    assert!(again.contains("cannot take cholera now"), "{again}");
    // Its course runs and ends; they are protected after it.
    let end = e.course.shed_until.max(e.course.ill_until);
    assert!(end > today && end <= today + 40, "{:?}", e.course);
    sim.advance_minutes((end - today + 1) * DAY).expect("lives");
    let e = sim.people().sickness.episodes()[0];
    assert_eq!(e.ended.map(|(_, o)| o), Some(Outcome::Recovered));
    assert!(e.ended.is_some_and(|(d, _)| d >= end));
    assert!(sim.people().sickness.quiet());
    let day = sim.now().day_index();
    assert!(sim.people().sickness.protected(target, index as u16, day));
    assert_eq!(
        sim.people().sickness.episodes().len(),
        1,
        "nobody else took it"
    );
}

#[test]
fn the_ill_keep_to_their_beds() {
    let mut sim = village(3);
    set_cholera(&mut sim, |d| {
        d.household_hazard = 0.0;
        d.severe_death_per_day = 0.0;
        d.symptomatic = 1.0;
        d.ill.mean = 6.0;
        d.ill.sd = 0.0;
    });
    to_morning(&mut sim, 9 * 60);
    let (target, _) = someone_with_family(&sim);
    sim.plague(target, CHOLERA).expect("brought");
    let e = sim.people().sickness.episodes()[0];
    let today = sim.now().day_index();
    // On the second day of the illness, every half hour of a day: abed, asleep, eating or resting.
    to_morning(&mut sim, 0);
    sim.advance_minutes((e.course.ill_from - today) * DAY)
        .expect("lives");
    assert!(sim.people().sickness.ill(target, sim.now().day_index()));
    let catalog = sim.rules().catalog.activities.clone();
    for _ in 0..48 {
        sim.advance_minutes(30).expect("lives");
        let p = sim.people().person(target).expect("alive");
        let b = catalog[usize::from(p.act.def)].behavior;
        assert!(
            matches!(b, Behavior::Sleep | Behavior::Eat | Behavior::Rest),
            "{b:?} while ill"
        );
    }
}

#[test]
fn a_household_takes_it_from_a_member_who_sheds_it() {
    let mut sim = village(3);
    set_cholera(&mut sim, |d| {
        d.household_hazard = 50.0;
        d.severe_death_per_day = 0.0;
    });
    to_morning(&mut sim, 9 * 60);
    let (target, family) = someone_with_family(&sim);
    let household = sim.people().person(target).expect("alive").household;
    sim.plague(target, CHOLERA).expect("brought");
    let e = sim.people().sickness.episodes()[0];
    let today = sim.now().day_index();
    // The day's turn on the first day it is shed: everyone else at home takes it then.
    sim.advance_minutes((e.course.shed_from - today) * DAY)
        .expect("lives");
    let episodes = sim.people().sickness.episodes();
    for m in &family {
        let theirs = episodes
            .iter()
            .find(|x| x.person == *m)
            .unwrap_or_else(|| panic!("{m} took it"));
        assert_eq!(theirs.acquired, Acquired::Household { household });
        assert_eq!(theirs.infected, e.course.shed_from);
    }
    assert_eq!(
        episodes.len(),
        1 + family.len(),
        "nobody outside the household"
    );
}

#[test]
fn the_severely_ill_die_of_it_and_the_record_names_the_disease() {
    let mut sim = village(3);
    set_cholera(&mut sim, |d| {
        d.household_hazard = 0.0;
        d.symptomatic = 1.0;
        d.severe_by_age = vec![(0.0, 1.0)];
        d.severe_death_per_day = 1.0;
    });
    to_morning(&mut sim, 9 * 60);
    let (target, _) = someone_with_family(&sim);
    sim.plague(target, CHOLERA).expect("brought");
    let e = sim.people().sickness.episodes()[0];
    assert!(e.course.severe);
    let today = sim.now().day_index();
    sim.advance_minutes((e.course.ill_from - today) * DAY)
        .expect("lives");
    assert!(sim.people().person(target).is_none(), "dead");
    let r = &sim.people().records[&target];
    assert_eq!(r.died.map(|(_, c)| c), Some(Cause::Disease));
    let e = sim.people().sickness.episodes()[0];
    assert_eq!(e.ended, Some((e.course.ill_from, Outcome::Died)));
    let died = sim
        .people()
        .chronicle
        .iter()
        .rev()
        .find(|c| c.kind == ChronicleKind::Died && c.people.first() == Some(&target))
        .expect("the chronicle tells it");
    assert_eq!(died.name, "disease:cholera");
}

/// A save of `sim` loaded again, checked to be the same section for section.
fn reloaded(sim: &mut Sim) -> Sim {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "sick").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    same(sim, &loaded);
    loaded
}

fn same(a: &Sim, b: &Sim) {
    let (a, b) = (persist::encode_sections(a), persist::encode_sections(b));
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(&b) {
        assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
    }
}

#[test]
fn infections_save_load_and_go_on_alike() {
    let mut sim = village(3);
    to_morning(&mut sim, 9 * 60);
    let (target, _) = someone_with_family(&sim);
    sim.plague(target, CHOLERA).expect("brought");
    sim.advance_minutes(2 * DAY + 5 * 60).expect("lives");
    assert!(!sim.people().sickness.quiet());
    let mut loaded = reloaded(&mut sim);
    assert_eq!(
        loaded.people().sickness.episodes(),
        sim.people().sickness.episodes()
    );
    sim.advance_minutes(3 * DAY).expect("advances");
    loaded.advance_minutes(3 * DAY).expect("advances");
    same(&sim, &loaded);
}
