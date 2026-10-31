//! Sickness (M6a slice AZ, steps one and three; ADR-0021 §5, §8; research 05-03, 05-05): no
//! disease without an introduction; the observer brings one to a person, as if they took it
//! elsewhere; its course runs by its own clocks; the ill keep to their beds; a household takes it
//! from members who shed it; the severely ill die of it by their own draws, the record naming the
//! disease; the household tends its sick, and a day of care lowers that day's risk, a carer who
//! knows fluid replacement lowering cholera's further and teaching it at home; and infections and
//! their care save and load exactly. Sickness seen (M6a slice BA, step one; ADR-0021 §6): a
//! household's members see someone of theirs abed and its close kin learn of a death of it, and
//! word of it goes round as any word does.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::sickness::{Acquired, Outcome};
use civ_agents::{Behavior, Cause, ChronicleKind, OutbreakStep};
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
        d.care_rr = 1.0;
    });
    to_morning(&mut sim, 9 * 60);
    let (target, _) = someone_with_family(&sim);
    sim.plague(target, CHOLERA).expect("brought");
    let e = sim.people().sickness.episodes()[0];
    assert!(e.course.severe);
    let today = sim.now().day_index();
    // Alive through their first day ill; that day kills them at its end.
    sim.advance_minutes((e.course.ill_from - today) * DAY)
        .expect("lives");
    assert!(sim.people().person(target).is_some(), "alive on the day");
    to_morning(&mut sim, 1);
    assert!(sim.people().person(target).is_none(), "dead");
    let r = &sim.people().records[&target];
    assert_eq!(r.died.map(|(_, c)| c), Some(Cause::Disease));
    let e = sim.people().sickness.episodes()[0];
    assert_eq!(e.ended, Some((e.course.ill_from + 1, Outcome::Died)));
    let died = sim
        .people()
        .chronicle
        .iter()
        .rev()
        .find(|c| c.kind == ChronicleKind::Died && c.people.first() == Some(&target))
        .expect("the chronicle tells it");
    assert_eq!(died.name, "disease:cholera");
}

/// Someone of the largest household brought cholera that makes them severely ill for three days
/// from the day after next, nobody else taking it; `f` changes its definition further. Returns
/// them, their household's other members and the first day they are ill.
fn severely_ill(
    sim: &mut Sim,
    f: impl FnOnce(&mut civ_agents::params::DiseaseDef),
) -> (PermanentId, Vec<PermanentId>, i64) {
    set_cholera(sim, |d| {
        d.household_hazard = 0.0;
        d.symptomatic = 1.0;
        d.severe_by_age = vec![(0.0, 1.0)];
        d.severe_death_per_day = 1.0;
        d.incubation = civ_agents::params::Days { mean: 2.0, sd: 0.0 };
        d.ill = civ_agents::params::Days { mean: 3.0, sd: 0.0 };
        f(d);
    });
    to_morning(sim, 9 * 60);
    let (target, family) = someone_with_family(sim);
    sim.plague(target, CHOLERA).expect("brought");
    let e = sim.people().sickness.episodes()[0];
    assert!(e.course.severe);
    (target, family, e.course.ill_from)
}

/// Lives on to 00:01 on day `day`.
fn to_day(sim: &mut Sim, day: i64) {
    let now = sim.now();
    let left = day * DAY + 1 - now.minutes();
    assert!(left > 0, "day {day} has begun");
    sim.advance_minutes(left).expect("lives");
}

#[test]
fn the_household_tends_its_sick_and_a_day_of_care_lowers_the_chance_of_dying() {
    // Care that takes away all of the day's risk: they live while tended a day's care; where a
    // day's care asks a thousand hours, the few given take away almost nothing, and they die
    // after the first.
    for (need, lives) in [(2.0, true), (1000.0, false)] {
        let mut sim = village(3);
        let (target, family, ill_from) = severely_ill(&mut sim, |d| {
            d.care_rr = 0.0;
            d.care_h_per_day = need;
        });
        to_day(&mut sim, ill_from + 1);
        assert_eq!(
            sim.people().person(target).is_some(),
            lives,
            "a day's care of {need} h"
        );
        let e = sim.people().sickness.episodes()[0];
        let (first, by) = e.care.first.expect("someone came");
        assert_eq!(first, ill_from, "help came the first day");
        assert!(
            family.contains(&by),
            "a member of the household tended them"
        );
        assert_eq!(e.care.day, ill_from);
        assert!(e.care.hours >= 2.0, "{} h", e.care.hours);
        assert_eq!(e.care.rr, 1.0, "nobody knows a treatment");
        // The care given saves and loads exactly.
        let loaded = reloaded(&mut sim);
        assert_eq!(loaded.people().sickness.episodes()[0].care, e.care);
        if lives {
            // Tended day after day, they outlive the illness.
            to_day(&mut sim, ill_from + 3);
            assert!(sim.people().person(target).is_some(), "alive after it");
            let e = sim.people().sickness.episodes()[0];
            assert!(e.ended.is_none_or(|(_, o)| o == Outcome::Recovered));
            assert!(e.care.total_h >= 6.0, "{} h in all", e.care.total_h);
        }
    }
}

#[test]
fn a_carer_who_knows_fluid_replacement_brings_cholera_s_risk_down() {
    // Care alone changes nothing here; fluid replacement takes away all of the day's risk.
    for knows in [false, true] {
        let mut sim = village(3);
        let (target, family, ill_from) = severely_ill(&mut sim, |d| {
            d.care_rr = 1.0;
            for t in &mut d.treatments {
                t.1 = 0.0;
            }
        });
        assert!(!sim.rules().catalog.diseases.is_empty());
        let fluids = sim
            .rules()
            .catalog
            .technique_index("core:technique/fluid_replacement")
            .expect("content");
        // Whoever of the household tends them knows it, or nobody does.
        if knows {
            for &m in &family {
                sim.introduce_technique(m, "core:technique/fluid_replacement", false)
                    .expect("taught");
            }
        }
        to_day(&mut sim, ill_from + 1);
        assert_eq!(sim.people().person(target).is_some(), knows);
        let e = sim.people().sickness.episodes()[0];
        assert_eq!(e.care.rr == 0.0, knows, "{:?}", e.care);
        assert_eq!(
            sim.people().person(family[0]).expect("alive").knows(fluids),
            knows
        );
    }
}

#[test]
fn one_who_tends_beside_a_household_that_knows_fluid_replacement_learns_it() {
    let mut sim = village(3);
    let (_, family, ill_from) = severely_ill(&mut sim, |d| d.care_rr = 1.0);
    let fluids = sim
        .rules()
        .catalog
        .technique_index("core:technique/fluid_replacement")
        .expect("content");
    let (teacher, learner) = (family[0], family[1]);
    sim.introduce_technique(teacher, "core:technique/fluid_replacement", false)
        .expect("taught");
    to_day(&mut sim, ill_from);
    sim.advance_minutes(8 * 60).expect("lives");
    let household = sim.people().person(learner).expect("alive").household;
    // Twenty hours of tending, in sessions: they know it at the end and not before. Whatever
    // they tended in the hours before counts toward it.
    let before = sim
        .people()
        .person(learner)
        .expect("alive")
        .knows
        .iter()
        .find(|k| usize::from(k.technique) == fluids)
        .map_or(0.0, |k| f64::from(k.hours));
    let sessions = (20.0 - before).ceil().max(1.0) as u32;
    for session in 0..sessions {
        assert!(
            !sim.people().person(learner).expect("alive").knows(fluids),
            "known after {session} hours"
        );
        sim.with_ctx_for_tests(|pop, ctx| pop.tended_for_tests(ctx, learner, household, 60));
    }
    assert!(sim.people().person(learner).expect("alive").knows(fluids));
}

#[test]
fn an_outbreak_begins_with_its_first_case_gathers_the_rest_and_ends_when_none_has_run_a_while() {
    let mut sim = village(3);
    set_cholera(&mut sim, |d| {
        d.household_hazard = 50.0;
        d.severe_death_per_day = 0.0;
    });
    to_morning(&mut sim, 9 * 60);
    let (target, family) = someone_with_family(&sim);
    sim.plague(target, CHOLERA).expect("brought");
    let began = sim.now().day_index();
    let outbreaks = sim.people().sickness.outbreaks().to_vec();
    assert_eq!(outbreaks.len(), 1);
    let o = outbreaks[0];
    assert_eq!((o.id, o.first, o.began, o.ended), (1, 1, began, None));
    let entry = |sim: &Sim, step: OutbreakStep| {
        sim.people()
            .chronicle
            .iter()
            .filter(|c| c.kind == ChronicleKind::Outbreak && c.number == f64::from(step as u8))
            .map(|c| (c.people.clone(), c.name.clone()))
            .collect::<Vec<_>>()
    };
    let opened = entry(&sim, OutbreakStep::Began);
    assert_eq!(opened.len(), 1);
    assert_eq!(opened[0].0, vec![target]);
    assert!(
        opened[0].1.starts_with("Cholera came to "),
        "{}",
        opened[0].1
    );
    // The household takes it from them: every case is part of the one outbreak, and it stays
    // open while any runs. Its records save and load exactly.
    let e = sim.people().sickness.episodes()[0];
    sim.advance_minutes((e.course.shed_from - began) * DAY)
        .expect("lives");
    let episodes = sim.people().sickness.episodes();
    assert!(episodes.len() > family.len(), "the household took it");
    assert!(episodes.iter().all(|e| e.outbreak == Some(1)));
    let loaded = reloaded(&mut sim);
    assert_eq!(
        loaded.people().sickness.outbreaks(),
        sim.people().sickness.outbreaks()
    );
    // It ends once no case has run for as long as a new infection could take to show (cholera's
    // 1.5 days and three spreads of 1: five days) and less than a dose of it is left in the
    // village's heaps, wells and water (what was shed into the heap decays there), and is told
    // with its counts.
    // (Cholera is the only disease here, and the village the only settlement.)
    let left = |sim: &Sim| {
        let c = &sim.people().contagion;
        c.loads.values().sum::<f64>()
            + c.stores.values().flat_map(|s| s.values()).sum::<f64>()
            + c.transit.iter().map(|t| t.amount).sum::<f64>()
    };
    let mut ended = None;
    let mut left_before = f64::INFINITY;
    for _ in 0..200 {
        left_before = left(&sim);
        sim.advance_minutes(DAY).expect("lives");
        if let Some(day) = sim.people().sickness.outbreaks()[0].ended {
            ended = Some(day);
            break;
        }
    }
    let ended = ended.expect("it ended");
    let s = &sim.people().sickness;
    let last = s
        .episodes()
        .iter()
        .filter_map(|e| e.ended.map(|(d, _)| d))
        .max()
        .expect("all ended");
    assert!(s.episodes().iter().all(|e| e.ended.is_some()));
    assert!(ended >= last + 5, "{ended} {last}");
    // It ended on the first day both held: five days, and less than a dose left.
    assert!(left(&sim) < 1.0, "{}", left(&sim));
    assert!(ended == last + 5 || left_before >= 1.0, "{left_before}");
    let closed = entry(&sim, OutbreakStep::Ended);
    assert_eq!(closed.len(), 1);
    let took = s.episodes().len();
    assert!(
        closed[0].1.contains(&format!("{took} took it")),
        "{}",
        closed[0].1
    );
    assert!(closed[0].1.contains("nobody died"), "{}", closed[0].1);
    // Another case after it ended begins another.
    let other = sim
        .people()
        .households
        .iter()
        .flat_map(|(_, x)| x.members.clone())
        .find(|m| !sim.people().sickness.of(*m).any(|_| true))
        .expect("someone who never took it");
    sim.plague(other, CHOLERA).expect("brought");
    let outbreaks = sim.people().sickness.outbreaks();
    assert_eq!(outbreaks.len(), 2);
    assert_eq!(outbreaks[1].first as usize, took + 1);
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

/// Cholera that nobody takes from anyone and that sheds nothing: only the one brought falls ill.
fn cholera_alone(sim: &mut Sim, deadly: bool) {
    set_cholera(sim, |d| {
        d.household_hazard = 0.0;
        d.shed_ill_per_day = 0.0;
        d.shed_silent_per_day = 0.0;
        d.symptomatic = 1.0;
        for s in &mut d.severe_by_age {
            s.1 = if deadly { 1.0 } else { 0.0 };
        }
        d.severe_death_per_day = if deadly { 1.0 } else { 0.0 };
    });
}

#[test]
fn a_household_sees_its_sick_and_word_of_it_goes_round() {
    use civ_agents::word::ClaimKind;
    let mut sim = village(3);
    cholera_alone(&mut sim, false);
    to_morning(&mut sim, 9 * 60);
    let (target, family) = someone_with_family(&sim);
    let household = sim.people().person(target).expect("alive").household;
    sim.plague(target, CHOLERA).expect("brought");
    // Nobody has seen anything before they lie abed.
    assert!(sim.people().word.sickness(household).is_none());
    let e = sim.people().sickness.episodes()[0];
    let ill_from = e.course.ill_from;
    while sim.now().day_index() <= ill_from {
        to_morning(&mut sim, 0);
    }
    let pop = sim.people();
    let c = *pop.word.sickness(household).expect("seen");
    assert_eq!(c.kind, ClaimKind::Sickness);
    assert!(c.day == ill_from || c.day == e.infected + 1, "{c:?}");
    // Its members saw it at first hand; nobody else did.
    for &m in family.iter().chain([&target]) {
        let h = pop
            .word
            .heard_by(m)
            .iter()
            .find(|h| h.claim == c.id)
            .expect("seen at home");
        assert_eq!((h.from, h.origin), (None, Some(m)));
    }
    assert!(
        pop.word
            .heard
            .iter()
            .filter(|h| h.claim == c.id && h.from.is_none())
            .all(|h| family.contains(&h.holder) || h.holder == target)
    );
    assert_eq!(
        pop.word
            .claims
            .iter()
            .filter(|c| c.kind == ClaimKind::Sickness)
            .count(),
        1,
        "one household had sickness"
    );
    // Word of it goes round: at the hearth and in others' homes, from those who were told.
    for _ in 0..10 {
        to_morning(&mut sim, 0);
    }
    let pop = sim.people();
    let told: Vec<_> = pop
        .word
        .heard
        .iter()
        .filter(|h| h.claim == c.id && h.from.is_some())
        .collect();
    assert!(!told.is_empty(), "word went round");
    assert!(told.iter().all(|h| h.origin.is_some()));
    // While it is news, the observer may whisper it to someone outside the household who has
    // not heard it; thirty days after it was first seen it is let go.
    let first = c.day;
    while sim.now().day_index() <= first + 31 {
        to_morning(&mut sim, 0);
    }
    let pop = sim.people();
    assert!(
        pop.word.claim(c.id).is_none(),
        "sickness first seen more than thirty days ago is no longer news"
    );
}

#[test]
fn kin_elsewhere_learn_of_a_death_by_sickness() {
    let mut sim = village(3);
    cholera_alone(&mut sim, true);
    to_morning(&mut sim, 9 * 60);
    // A founding band's households are its families, so make one of another household a child of
    // the one who will die, on record.
    let (dying, kin) = {
        let pop = sim.people();
        let (dying, _) = someone_with_family(&sim);
        let home = pop.person(dying).expect("alive").household;
        let kin = pop
            .people
            .iter()
            .map(|(_, p)| p)
            .find(|p| p.household != home)
            .map(|p| p.id)
            .expect("someone of another household");
        (dying, kin)
    };
    {
        let female = sim.people().person(dying).expect("alive").sex == civ_agents::Sex::Female;
        let pop = sim.people_mut_for_tests();
        let r = pop.records.get_mut(&kin).expect("on record");
        if female {
            r.mother = Some(dying);
        } else {
            r.father = Some(dying);
        }
        pop.forget_kin();
    }
    let household = sim.people().person(dying).expect("alive").household;
    sim.plague(dying, CHOLERA).expect("brought");
    for _ in 0..12 {
        to_morning(&mut sim, 0);
        if sim.people().person(dying).is_none() {
            break;
        }
    }
    assert!(sim.people().person(dying).is_none(), "died of it");
    let pop = sim.people();
    let c = pop.word.sickness(household).expect("seen");
    let h = pop
        .word
        .heard_by(kin)
        .iter()
        .find(|h| h.claim == c.id)
        .expect("their child elsewhere learns of the death");
    // They may have heard of the sickness already as word; they learn of it again at the death.
    let (died, _) = pop.records[&dying].died.expect("dead");
    assert_eq!(h.last, died.day_index(), "{h:?}");
}

/// A household of the village all brought cholera that keeps them severely ill for three days from
/// the day after next without killing them, each shedding `hazard` to those about them at home,
/// and an adult of another household made, by the records, the child of one of its members:
/// close kin living apart (M10c slice DZ). Returns the sick household, the kin and the first day
/// they are ill.
fn kin_apart_all_abed(sim: &mut Sim, hazard: f64) -> (PermanentId, PermanentId, i64) {
    set_cholera(sim, |d| {
        d.household_hazard = hazard;
        d.symptomatic = 1.0;
        d.severe_by_age = vec![(0.0, 1.0)];
        d.severe_death_per_day = 0.0;
        d.incubation = civ_agents::params::Days { mean: 2.0, sd: 0.0 };
        d.ill = civ_agents::params::Days { mean: 3.0, sd: 0.0 };
    });
    to_morning(sim, 9 * 60);
    let now = sim.now();
    let (sick, members) = sim
        .people()
        .households
        .iter()
        .map(|(_, x)| (x.id, x.members.clone()))
        .min_by_key(|(id, m)| (m.len(), *id))
        .expect("a household");
    let kin = sim
        .people()
        .households
        .iter()
        .filter(|(_, x)| x.id != sick)
        .flat_map(|(_, x)| x.members.iter().copied())
        .find(|&m| {
            sim.people()
                .person(m)
                .is_some_and(|p| p.age_years(now) >= 20.0)
        })
        .expect("an adult elsewhere");
    let pop = sim.people_mut_for_tests();
    pop.records.get_mut(&kin).expect("a record").mother = Some(members[0]);
    pop.forget_kin();
    for &m in &members {
        sim.plague(m, CHOLERA).expect("brought");
    }
    let ill_from = sim.people().sickness.episodes()[0].course.ill_from;
    (sick, kin, ill_from)
}

#[test]
fn kin_who_hear_of_a_home_all_abed_walk_there_to_tend_it_and_are_exposed_there() {
    // In both resolutions (ADR-0032 §2): Tend at another home is a block like Tend at home.
    for resolved in [false, true] {
        for heard in [false, true] {
            let mut sim = village(3);
            if resolved {
                sim.advance_to_midnight().expect("advances");
                let settlements: Vec<_> = sim.land().settlements.iter().map(|s| s.id).collect();
                sim.set_household_resolved(settlements);
            }
            let (sick, kin, ill_from) = kin_apart_all_abed(&mut sim, 0.0);
            to_day(&mut sim, ill_from);
            // The home's own see them abed; the kin hear of it, or not.
            let claim = sim.people().word.sickness(sick).expect("seen abed").id;
            if heard {
                sim.people_mut_for_tests()
                    .word
                    .hear(kin, claim, ill_from, None, None);
            }
            to_day(&mut sim, ill_from + 1);
            let by_kin = sim
                .people()
                .sickness
                .episodes()
                .iter()
                .filter(|e| e.care.first.is_some_and(|(_, by)| by == kin))
                .count();
            assert_eq!(
                by_kin > 0,
                heard,
                "resolved {resolved}, heard {heard}: {by_kin} tended by kin"
            );
            let tended = sim
                .people()
                .sickness
                .episodes()
                .iter()
                .filter(|e| e.care.first.is_some())
                .count();
            assert_eq!(tended > 0, heard, "nobody else tends a home all abed");
        }
    }
    // Where the sick shed it, the kin who sat with them may take it there, recorded as from that
    // home; a carer who never came does not.
    let mut sim = village(3);
    let (sick, kin, ill_from) = kin_apart_all_abed(&mut sim, 50.0);
    to_day(&mut sim, ill_from);
    let claim = sim.people().word.sickness(sick).expect("seen abed").id;
    sim.people_mut_for_tests()
        .word
        .hear(kin, claim, ill_from, None, None);
    to_day(&mut sim, ill_from + 1);
    let e = sim
        .people()
        .sickness
        .episodes()
        .iter()
        .find(|e| e.person == kin)
        .copied()
        .expect("the kin took it");
    assert_eq!(e.acquired, Acquired::Household { household: sick });
    assert_eq!(e.infected, ill_from, "taken the day they came");
}

/// A household of the village all brought cholera that keeps them severely ill without killing
/// them, but its youngest, made eight years old (old enough to go for help, too young to tend):
/// returns the household, that child and the first day the rest are ill (M10c slice DZ, second
/// part).
fn abed_but_a_child(sim: &mut Sim) -> (PermanentId, PermanentId, i64) {
    set_cholera(sim, |d| {
        d.household_hazard = 0.0;
        d.symptomatic = 1.0;
        d.severe_by_age = vec![(0.0, 1.0)];
        d.severe_death_per_day = 0.0;
        d.incubation = civ_agents::params::Days { mean: 2.0, sd: 0.0 };
        d.ill = civ_agents::params::Days { mean: 3.0, sd: 0.0 };
    });
    to_morning(sim, 9 * 60);
    let now = sim.now();
    let (sick, members) = sim
        .people()
        .households
        .iter()
        .map(|(_, x)| (x.id, x.members.clone()))
        .filter(|(_, m)| m.len() >= 2)
        .min_by_key(|(id, m)| (m.len(), *id))
        .expect("a household of two or more");
    let child = *members.last().expect("a member");
    for (_, p) in sim.people_mut_for_tests().people.iter_mut() {
        if p.id == child {
            p.born = now.plus_minutes(-8 * civ_core::time::MINUTES_PER_YEAR);
        }
    }
    for &m in members.iter().filter(|&&m| m != child) {
        sim.plague(m, CHOLERA).expect("brought");
    }
    let ill_from = sim.people().sickness.episodes()[0].course.ill_from;
    (sick, child, ill_from)
}

#[test]
fn a_household_short_of_care_calls_kin_or_someone_it_knows_and_pays_those_not_kin_a_fee() {
    use civ_agents::ledger::Channel;
    for kin in [true, false] {
        let mut sim = village(3);
        let (sick, child, ill_from) = abed_but_a_child(&mut sim);
        let now = sim.now();
        // An adult of another household: by the records a grown son of one of those abed, or
        // someone the child knows by a trade.
        let (other, adult) = sim
            .people()
            .households
            .iter()
            .filter(|(_, x)| x.id != sick)
            .find_map(|(_, x)| {
                let a = x.members.iter().copied().find(|&m| {
                    sim.people()
                        .person(m)
                        .is_some_and(|p| p.age_years(now) >= 20.0)
                })?;
                Some((x.id, a))
            })
            .expect("an adult elsewhere");
        let params = sim.rules().people.ties.clone();
        let day = now.day_index();
        let parent = sim.people().household(sick).expect("home").members[0];
        let pop = sim.people_mut_for_tests();
        if kin {
            pop.records.get_mut(&adult).expect("a record").mother = Some(parent);
            pop.forget_kin();
        } else {
            pop.ties.record(
                child,
                adult,
                civ_agents::ties::Act::Traded,
                1.0,
                0.0,
                day,
                &params,
            );
        }
        to_day(&mut sim, ill_from + 1);
        let call = *sim
            .people()
            .sickness
            .call_of(sick)
            .expect("the child called someone");
        assert_eq!(call.day, ill_from, "called the first day they lay ill");
        let fee = 0.5
            * sim.people().household(sick).expect("home").members.len() as f64
            * sim.rules().people.household.daily_kcal_per_person;
        if kin {
            assert_eq!(call.carer, other, "kin first");
            assert_eq!(call.fee_kcal, 0.0, "kin ask no fee");
        } else {
            assert_ne!(call.carer, sick);
            assert!(
                (f64::from(call.fee_kcal) - fee).abs() < 1e-3,
                "{}",
                call.fee_kcal
            );
        }
        // Someone of the household called came and tended them, and was paid that day.
        let carers = sim
            .people()
            .household(call.carer)
            .expect("the carer's household")
            .members
            .clone();
        assert!(
            sim.people()
                .sickness
                .episodes()
                .iter()
                .any(|e| e.care.first.is_some_and(|(_, by)| carers.contains(&by))),
            "kin {kin}: tended by the household called"
        );
        assert_eq!(call.paid, ill_from);
        let goods = &sim.rules().catalog.goods;
        let fees: f64 = (0..goods.len())
            .map(|g| sim.people().transfers.get(Channel::Fee, g) * goods[g].kcal_per_kg)
            .sum();
        if kin {
            assert_eq!(fees, 0.0);
        } else {
            assert!(
                (fees - fee).abs() < 1.0,
                "a day's fee paid in food: {fees} kcal"
            );
        }
        // The call saves and loads with the rest.
        let loaded = reloaded(&mut sim);
        assert_eq!(loaded.people().sickness.calls, sim.people().sickness.calls);
        // Once nobody of the household is ill, the call lapses.
        to_day(&mut sim, ill_from + 5);
        assert!(sim.people().sickness.call_of(sick).is_none());
    }
}

#[test]
fn a_household_calls_the_carer_whose_tended_it_saw_live_and_remembers_what_came_of_their_care() {
    use civ_agents::ties::{Act, Domain};
    // The child knows an adult of every other household; first nobody's care has been seen, then
    // the household has seen the sick an adult of one of them tended live five times.
    let mut first: Option<PermanentId> = None;
    for seen in [false, true] {
        let mut sim = village(3);
        let (sick, child, ill_from) = abed_but_a_child(&mut sim);
        let now = sim.now();
        let day = now.day_index();
        let adults: Vec<(PermanentId, PermanentId)> = sim
            .people()
            .households
            .iter()
            .filter(|(_, x)| x.id != sick)
            .filter_map(|(_, x)| {
                let a = x.members.iter().copied().find(|&m| {
                    sim.people()
                        .person(m)
                        .is_some_and(|p| p.age_years(now) >= 20.0)
                })?;
                Some((x.id, a))
            })
            .collect();
        let params = sim.rules().people.ties.clone();
        let members = sim.people().household(sick).expect("home").members.clone();
        // The household seen tending well: any other than the one called when none was.
        let reputed = first.and_then(|f| adults.iter().find(|(h, _)| *h != f).copied());
        let pop = sim.people_mut_for_tests();
        for &(_, a) in &adults {
            pop.ties
                .record(child, a, Act::Traded, 1.0, 0.0, day, &params);
        }
        if seen {
            let (_, a) = reputed.expect("another household");
            for &m in &members {
                pop.ties
                    .record(m, a, Act::TendedLived, 5.0, 0.0, day, &params);
            }
        }
        to_day(&mut sim, ill_from + 1);
        let call = *sim.people().sickness.call_of(sick).expect("called");
        if seen {
            assert_eq!(
                Some(call.carer),
                reputed.map(|(h, _)| h),
                "the household whose care they saw"
            );
        } else {
            first = Some(call.carer);
        }
        // Once the sick are well, those at home remember what came of the care, and the carer's
        // page says it.
        let carers = sim
            .people()
            .household(call.carer)
            .expect("the carer's household")
            .members
            .clone();
        to_day(&mut sim, ill_from + 5);
        let tended_by = sim
            .people()
            .sickness
            .episodes()
            .iter()
            .filter_map(|e| e.care.first.map(|(_, by)| by))
            .find(|by| carers.contains(by))
            .expect("tended by the household called");
        let day = sim.now().day_index();
        let esteem = sim
            .people()
            .ties
            .of(child)
            .iter()
            .find(|t| t.to == tended_by)
            .map(|t| t.at(day, &params).esteem(Domain::Craft, &params))
            .expect("the child knows the carer now");
        assert!(esteem > 0.5, "the tended lived: {esteem}");
        let words = civ_sim::frames::sickness::care_words(&sim, tended_by);
        assert!(
            words.starts_with("First to tend ") && words.contains(" of the sick of other homes: "),
            "{words}"
        );
        assert!(words.ends_with(" lived"), "{words}");
    }
}

/// A woman of the largest household with someone else of her household in it, made pregnant and
/// due today, so that she gives birth at the next day's turn (M10c slice EA): returns her and
/// another adult of her household.
fn due_today(sim: &mut Sim) -> (PermanentId, PermanentId) {
    use civ_agents::Repro;
    let now = sim.now();
    let (mother, other) = sim
        .people()
        .households
        .iter()
        .map(|(_, x)| x)
        .filter(|x| x.members.len() >= 2)
        .find_map(|x| {
            let adult = |m: &PermanentId| {
                sim.people()
                    .person(*m)
                    .is_some_and(|p| (18.0..40.0).contains(&p.age_years(now)))
            };
            let mother = x.members.iter().copied().find(|m| {
                adult(m)
                    && sim.people().person(*m).is_some_and(|p| {
                        p.sex == civ_agents::Sex::Female && matches!(p.repro, Repro::Open)
                    })
            })?;
            let other = x
                .members
                .iter()
                .copied()
                .find(|&m| m != mother && adult(&m))?;
            Some((mother, other))
        })
        .expect("a woman with another adult at home");
    for (_, p) in sim.people_mut_for_tests().people.iter_mut() {
        if p.id == mother {
            p.repro = Repro::Pregnant {
                conceived: now.plus_minutes(-270 * DAY),
                due: now,
                father: None,
                loss: false,
            };
        }
    }
    (mother, other)
}

/// The chronicle's name for `who`'s death, if they died.
fn death_named(sim: &Sim, who: PermanentId) -> Option<String> {
    sim.people()
        .chronicle
        .iter()
        .rev()
        .find(|c| c.kind == ChronicleKind::Died && c.people.first() == Some(&who))
        .map(|c| c.name.clone())
}

#[test]
fn a_birth_attended_by_one_who_knows_birth_care_is_survived_and_one_fresh_from_the_fevered_harms() {
    const ATTENDING: &str = "core:technique/birth_attending";
    // Every birth kills unattended; birth attending (made to prevent all bleeding and obstructed
    // labour here) leaves fever after the birth, all of the rate's share here; clean hands
    // prevents all of that.
    let set = |sim: &mut Sim, rate: f64, shares: [f64; 3]| {
        let rules = sim.rules_mut_for_tests().expect("the rules are not shared");
        rules.people.mortality.maternal_death_per_birth = rate;
        rules.people.mortality.birth.shares = shares;
        let t = rules.catalog.technique_index(ATTENDING).expect("content");
        rules.catalog.techniques[t].birth = [1.0, 0.0, 1.0];
        let c = rules
            .catalog
            .technique_index("core:technique/clean_hands")
            .expect("content");
        rules.catalog.techniques[c].birth = [0.0, 1.0, 0.0];
    };
    // Nobody attends: she dies, and the chronicle names childbirth as ever.
    let mut sim = village(3);
    to_morning(&mut sim, 9 * 60);
    set(&mut sim, 1.0, [0.3, 0.4, 0.3]);
    let (mother, _) = due_today(&mut sim);
    to_morning(&mut sim, 1);
    assert!(sim.people().person(mother).is_none(), "died unattended");
    assert_eq!(death_named(&sim, mother).as_deref(), Some("childbirth"));
    // Someone of her household knows birth attending and nothing of fever: she lives where the
    // rate is all bleeding and obstruction, and dies of fever where it is all fever.
    for (shares, lives) in [([0.5, 0.0, 0.5], true), ([0.0, 1.0, 0.0], false)] {
        let mut sim = village(3);
        to_morning(&mut sim, 9 * 60);
        set(&mut sim, 1.0, shares);
        let (mother, attendant) = due_today(&mut sim);
        sim.introduce_technique(attendant, ATTENDING, false)
            .expect("taught");
        to_morning(&mut sim, 1);
        assert_eq!(sim.people().person(mother).is_some(), lives, "{shares:?}");
        if !lives {
            assert_eq!(
                death_named(&sim, mother).as_deref(),
                Some("childbirth:infection")
            );
        }
    }
    // Half the births kill by fever: an attendant fresh from tending the ill without clean hands
    // doubles it, so she dies; knowing clean hands, she lives.
    for clean in [false, true] {
        let mut sim = village(3);
        to_morning(&mut sim, 9 * 60);
        set(&mut sim, 0.5, [0.0, 1.0, 0.0]);
        let (mother, attendant) = due_today(&mut sim);
        sim.introduce_technique(attendant, ATTENDING, false)
            .expect("taught");
        if clean {
            sim.introduce_technique(attendant, "core:technique/clean_hands", false)
                .expect("taught");
        }
        let day = sim.now().day_index();
        sim.people_mut_for_tests()
            .sickness
            .tended
            .insert(attendant, day);
        to_morning(&mut sim, 1);
        assert_eq!(
            sim.people().person(mother).is_some(),
            clean,
            "clean hands {clean}"
        );
        // Who tended the ill saves and loads with the rest.
        reloaded(&mut sim);
    }
}

#[test]
fn a_household_with_a_birth_due_and_nobody_to_attend_calls_one_it_heard_attend_and_pays_at_the_birth()
 {
    use civ_agents::Repro;
    use civ_agents::ledger::Channel;
    use civ_agents::ties::Domain;
    const ATTENDING: &str = "core:technique/birth_attending";
    let mut sim = village(3);
    to_morning(&mut sim, 9 * 60);
    {
        let rules = sim.rules_mut_for_tests().expect("the rules are not shared");
        rules.people.mortality.maternal_death_per_birth = 1.0;
        rules.people.mortality.birth.shares = [0.5, 0.0, 0.5];
        let t = rules.catalog.technique_index(ATTENDING).expect("content");
        rules.catalog.techniques[t].birth = [1.0, 0.0, 1.0];
    }
    let (mother, _) = due_today(&mut sim);
    let home = sim.people().person(mother).expect("alive").household;
    // Due in three days, nobody at home or of her kin knowing birth care.
    let now = sim.now();
    for (_, p) in sim.people_mut_for_tests().people.iter_mut() {
        if p.id == mother
            && let Repro::Pregnant { due, .. } = &mut p.repro
        {
            *due = now.plus_minutes(3 * DAY);
        }
    }
    // An adult of another household knows birth attending and once attended a birth in hers.
    let attendant = sim
        .people()
        .households
        .iter()
        .filter(|(_, x)| x.id != home)
        .flat_map(|(_, x)| x.members.iter().copied())
        .find(|&m| {
            sim.people()
                .person(m)
                .is_some_and(|p| p.age_years(now) >= 20.0)
        })
        .expect("an adult elsewhere");
    let theirs = sim.people().person(attendant).expect("alive").household;
    sim.introduce_technique(attendant, ATTENDING, false)
        .expect("taught");
    let day = now.day_index();
    sim.people_mut_for_tests()
        .sickness
        .attended
        .push(civ_agents::sickness::BirthAttended {
            attendant,
            household: home,
            day: day - 100,
            lived: true,
        });
    // Within the days before the birth, someone of hers calls the attendant's household.
    to_morning(&mut sim, 1);
    to_morning(&mut sim, 1);
    let call = *sim
        .people()
        .sickness
        .birth_call_of(home)
        .expect("called an attendant");
    assert_eq!(call.carer, theirs);
    let fee = 0.5
        * sim.people().household(home).expect("home").members.len() as f64
        * sim.rules().people.household.daily_kcal_per_person;
    assert!((f64::from(call.fee_kcal) - fee).abs() < 1e-3);
    assert!(
        sim.people().sickness.call_of(home).is_none(),
        "no call for the ill"
    );
    // The call saves and loads with the rest.
    reloaded(&mut sim);
    // At the birth the attendant comes: every birth kills unattended here, and this one she
    // lives through; it is kept as the attendant's, the fee is paid and the call answered.
    to_morning(&mut sim, 1);
    to_morning(&mut sim, 1);
    assert!(sim.people().person(mother).is_some(), "she lived, attended");
    let last = *sim.people().sickness.attended.last().expect("attended");
    assert_eq!(
        (last.attendant, last.household, last.lived),
        (attendant, home, true)
    );
    assert!(sim.people().sickness.birth_call_of(home).is_none());
    let goods = &sim.rules().catalog.goods;
    let fees: f64 = (0..goods.len())
        .map(|g| sim.people().transfers.get(Channel::Fee, g) * goods[g].kcal_per_kg)
        .sum();
    assert!(
        (fees - fee).abs() < 1.0,
        "the fee paid at the birth: {fees}"
    );
    // Those at home hold her in esteem for her craft; her page says the births she attended.
    let params = sim.rules().people.ties.clone();
    let today = sim.now().day_index();
    let esteem = sim
        .people()
        .ties
        .of(mother)
        .iter()
        .find(|t| t.to == attendant)
        .map(|t| t.at(today, &params).esteem(Domain::Craft, &params))
        .expect("the mother knows her now");
    assert!(esteem > 0.5, "{esteem}");
    let words = civ_sim::frames::sickness::care_words(&sim, attendant);
    assert!(
        words.contains("Attended 2 births: every mother lived"),
        "{words}"
    );
    reloaded(&mut sim);
}

/// A household that lost someone in the care of a carer it paid who knew no treatment, where
/// someone grown of the settlement knows fluid replacement, puts a licence before the gathering;
/// in force, it may pay only a household where someone knows it (M10c slice EC, third part).
#[test]
fn a_death_in_unknowing_paid_care_puts_a_licence_before_the_gathering() {
    use civ_agents::polity::{IssueKind, LawStatus, PolicyKind};
    use civ_agents::sickness::UnknowingCare;
    let mut sim = village(3);
    to_morning(&mut sim, 8 * 60);
    let (someone, _) = someone_with_family(&sim);
    let home = sim.people().person(someone).expect("alive").household;
    let settlement = sim
        .people()
        .household(home)
        .and_then(|x| x.settlement)
        .expect("settled");
    let pi = sim
        .people()
        .polities
        .iter()
        .position(|p| p.holds(Some(settlement)))
        .expect("its polity");
    let fluid = sim
        .rules()
        .catalog
        .technique_index("core:technique/fluid_replacement")
        .expect("fluid replacement");
    let now = sim.now();
    let others: Vec<PermanentId> = sim
        .people()
        .households
        .iter()
        .filter(|(_, x)| x.id != home && x.settlement == Some(settlement))
        .map(|(_, x)| x.id)
        .collect();
    let (knowing, unknowing) = (others[0], others[1]);
    let healer = sim
        .people()
        .household(knowing)
        .and_then(|x| {
            x.members.iter().copied().find(|&m| {
                sim.people()
                    .person(m)
                    .is_some_and(|p| p.age_years(now) >= 16.0)
            })
        })
        .expect("a grown member");
    let day = now.day_index();
    let cholera = cholera(&sim) as u16;
    let pop = sim.people_mut_for_tests();
    for (_, p) in pop.people.iter_mut() {
        p.knows.retain(|k| usize::from(k.technique) != fluid);
    }
    pop.sickness.unknowing.push(UnknowingCare {
        household: home,
        disease: cholera,
        day,
    });
    let cures =
        |sim: &mut Sim| sim.with_ctx_for_tests(|pop, ctx| pop.unknowing_care_for_tests(ctx, pi));
    assert!(cures(&mut sim).is_empty(), "nobody knows a treatment");
    for (_, p) in sim.people_mut_for_tests().people.iter_mut() {
        if p.id == healer {
            p.come_to_know(fluid, civ_agents::person::KnowSource::Observer, now);
        }
    }
    assert_eq!(cures(&mut sim), vec![fluid]);
    // Someone proposes the licence, against deaths in unknowing paid care.
    sim.people_mut_for_tests().every_adult_deliberates = true;
    let mut proposed = None;
    for _ in 0..24 {
        {
            let pop = sim.people_mut_for_tests();
            pop.polities[pi].gathering = None;
            for l in &mut pop.polities[pi].laws {
                if l.status == LawStatus::Proposed {
                    l.status = LawStatus::Rejected;
                }
            }
        }
        sim.with_ctx_for_tests(|pop, ctx| pop.review_for_tests(ctx, settlement));
        match sim.people().polities[pi].agenda().cloned() {
            Some(l) if l.kind == PolicyKind::Licence => {
                proposed = Some(l);
                break;
            }
            Some(_) => continue,
            None => break,
        }
    }
    let law = proposed.expect("a licence proposed");
    assert_eq!(law.issue, IssueKind::UnknowingCare);
    let licensed = |sim: &mut Sim, y: PermanentId| {
        sim.with_ctx_for_tests(|pop, ctx| pop.licensed_for_tests(ctx, home, y))
    };
    assert!(licensed(&mut sim, unknowing), "no licence yet");
    {
        let pop = sim.people_mut_for_tests();
        pop.every_adult_deliberates = false;
        let p = &mut pop.polities[pi];
        p.gathering = None;
        p.laws
            .iter_mut()
            .find(|l| l.id == law.id)
            .expect("the law")
            .status = LawStatus::InForce;
    }
    // In force: only a household where someone knows fluid replacement may be paid.
    assert!(!licensed(&mut sim, unknowing));
    assert!(licensed(&mut sim, knowing));
    // The record saves and loads alike.
    let loaded = reloaded(&mut sim);
    assert_eq!(
        loaded.people().sickness.unknowing,
        sim.people().sickness.unknowing
    );
}

const SMALLPOX: &str = "core:disease/smallpox";

/// The cases infected within 26 days of the first case of smallpox brought to a village of seed
/// `seed`, besides it: none of its own cases sheds before then, so they are its first generation
/// (the slice's probe of R0, 05-03 §2.2: 3.5-6 for a first meeting). And how many took it in
/// company at the hearth.
fn smallpox_first_generation(seed: u64) -> (usize, usize) {
    let mut sim = village(seed);
    to_morning(&mut sim, 8 * 60);
    let (index, _) = someone_with_family(&sim);
    sim.plague(index, SMALLPOX).expect("brought");
    let start = sim.now().day_index();
    sim.advance_minutes(28 * DAY).expect("lives");
    let disease = sim
        .rules()
        .catalog
        .disease_index(SMALLPOX)
        .expect("smallpox") as u16;
    let first: Vec<_> = sim
        .people()
        .sickness
        .episodes()
        .iter()
        .filter(|e| e.disease == disease && e.person != index && e.infected < start + 26)
        .collect();
    let company = first
        .iter()
        .filter(|e| matches!(e.acquired, Acquired::Company { .. }))
        .count();
    (first.len(), company)
}

#[test]
#[ignore = "a probe: cargo test --profile simcheck -p civ-sim --test sickness -- --ignored smallpox_r0 --nocapture"]
fn smallpox_r0() {
    let mut all = Vec::new();
    for seed in 1..=6 {
        let (n, company) = smallpox_first_generation(seed);
        eprintln!("seed {seed}: {n} infected by the first case ({company} in company)");
        all.push(n);
    }
    let mean = all.iter().sum::<usize>() as f64 / all.len() as f64;
    eprintln!("mean first generation {mean:.2}");
}

#[test]
#[ignore = "a probe: cargo test --profile simcheck -p civ-sim --test sickness -- --ignored smallpox_fatality --nocapture"]
fn smallpox_fatality() {
    let (mut cases, mut died) = (0, 0);
    for seed in 1..=4 {
        let mut sim = village(seed);
        to_morning(&mut sim, 8 * 60);
        let (index, _) = someone_with_family(&sim);
        sim.plague(index, SMALLPOX).expect("brought");
        sim.advance_minutes(150 * DAY).expect("lives");
        for e in sim.people().sickness.episodes() {
            if let Some((_, outcome)) = e.ended {
                cases += 1;
                died += usize::from(outcome == Outcome::Died);
            }
        }
    }
    eprintln!("{died} died of {cases} ended cases");
}

/// Smallpox brought to one person passes to people of other homes they keep company with at the
/// hearth, in both resolutions; the day's company, kept while it runs, saves mid-day and loads
/// alike (M10c slice ED, first part).
#[test]
fn smallpox_passes_in_company_at_the_hearth_to_those_of_other_homes() {
    for resolved in [false, true] {
        let mut sim = village(1);
        if resolved {
            sim.advance_to_midnight().expect("advances");
            let settlements: Vec<_> = sim.land().settlements.iter().map(|s| s.id).collect();
            sim.set_household_resolved(settlements);
            sim.advance_minutes(1).expect("advances");
            sim.advance_to_midnight().expect("advances");
            assert!(!sim.household_resolved().is_empty(), "household-resolved");
        }
        to_morning(&mut sim, 8 * 60);
        let (index, _) = someone_with_family(&sim);
        let home = sim.people().person(index).expect("alive").household;
        sim.plague(index, SMALLPOX).expect("brought");
        let disease = sim
            .rules()
            .catalog
            .disease_index(SMALLPOX)
            .expect("smallpox") as u16;
        if !resolved {
            // By the third evening, the day's company at the hearth is kept; it saves and loads
            // alike.
            sim.advance_minutes(2 * DAY + 12 * 60).expect("lives");
            assert!(!sim.people().met.is_empty(), "the day's company is kept");
            let loaded = reloaded(&mut sim);
            assert_eq!(loaded.people().met, sim.people().met);
        }
        // A resolved day meets fewer at the hearth, for shorter: given longer.
        let days = if resolved { 60 } else { 28 };
        sim.advance_minutes(days * DAY).expect("lives");
        let in_company: Vec<_> = sim
            .people()
            .sickness
            .episodes()
            .iter()
            .filter(|e| e.disease == disease)
            .filter_map(|e| match e.acquired {
                Acquired::Company { person } => Some((e.person, person)),
                _ => None,
            })
            .collect();
        assert!(
            !in_company.is_empty(),
            "resolved {resolved}: someone took it in company at the hearth"
        );
        let other_homes = in_company.iter().any(|&(p, _)| {
            sim.people()
                .sickness
                .episodes()
                .iter()
                .find(|e| e.person == p && e.disease == disease)
                .and_then(|e| e.household)
                != Some(home)
        });
        assert!(other_homes, "resolved {resolved}: of another home");
    }
}

/// Where someone died of smallpox, a household with ties to them believes it near and calls one
/// its members know who knows variolation; they come, and its people not protected take a mild
/// course of it from them, inoculated; the call saves and loads (M10c slice ED, second part).
#[test]
fn a_household_that_believes_smallpox_near_calls_a_variolator_and_its_people_are_inoculated() {
    let mut sim = village(1);
    to_morning(&mut sim, 8 * 60);
    let (index, _) = someone_with_family(&sim);
    sim.plague(index, SMALLPOX).expect("brought");
    let disease = sim
        .rules()
        .catalog
        .disease_index(SMALLPOX)
        .expect("smallpox") as u16;
    let variolation = sim
        .rules()
        .catalog
        .technique_index("core:technique/variolation")
        .expect("variolation");
    // Live until someone dies of it.
    let mut dead = None;
    for _ in 0..80 {
        sim.advance_minutes(DAY).expect("lives");
        dead = sim
            .people()
            .sickness
            .episodes()
            .iter()
            .find(|e| e.disease == disease && matches!(e.ended, Some((_, Outcome::Died))))
            .map(|e| e.person);
        if dead.is_some() {
            break;
        }
    }
    let dead = dead.expect("someone died of smallpox");
    let day = sim.now().day_index();
    // A household of people tied to the dead, with someone of it not protected, and someone its
    // members have ties with, of another household, alive and well, to learn variolation.
    let people = sim.people();
    let (home, member, variolator) = people
        .households
        .iter()
        .find_map(|(_, x)| {
            let tied = x
                .members
                .iter()
                .any(|&m| people.ties.of(m).iter().any(|t| t.to == dead));
            if !tied
                || !x
                    .members
                    .iter()
                    .any(|&m| !people.sickness.protected(m, disease, day))
            {
                return None;
            }
            x.members.iter().find_map(|&m| {
                let v = people.ties.of(m).iter().map(|t| t.to).find(|&v| {
                    people.person(v).is_some_and(|p| {
                        p.household != x.id
                            && !people.sickness.ill(v, day)
                            && people
                                .household(p.household)
                                .is_some_and(|y| y.settlement == x.settlement)
                    })
                })?;
                Some((x.id, m, v))
            })
        })
        .expect("a household tied to the dead, and a variolator they know");
    let now = sim.now();
    for (_, p) in sim.people_mut_for_tests().people.iter_mut() {
        if p.id == variolator {
            p.come_to_know(variolation, civ_agents::person::KnowSource::Observer, now);
        }
    }
    let theirs = sim.people().person(variolator).expect("alive").household;
    let called =
        sim.with_ctx_for_tests(|pop, ctx| pop.inoculation_call_for_tests(ctx, member, home));
    assert_eq!(
        called,
        Some(theirs),
        "they would call the variolator's household"
    );
    // They call, and the variolator comes: its people not protected take it, inoculated by them.
    let mut saved = false;
    let mut inoculated = Vec::new();
    // Every household that called someone to inoculate it: others tied to the dead may call
    // the variolator too.
    let mut callers = std::collections::BTreeSet::new();
    // Within the fortnight a call stands, the variolator answering others who called first as
    // their days allow. Looked at every ten minutes: a call made and answered within a
    // quarter-day was missed when this looked four times a day (M11d, ninth part: once fields
    // kept off outcrops, a household's call came and was answered between two looks).
    for _ in 0..14 * 24 * 6 {
        sim.advance_minutes(10).expect("lives");
        callers.extend(
            sim.people()
                .sickness
                .inoculation_calls
                .iter()
                .map(|c| c.caller),
        );
        if !saved && !sim.people().sickness.inoculation_calls.is_empty() {
            let loaded = reloaded(&mut sim);
            assert_eq!(
                loaded.people().sickness.inoculation_calls,
                sim.people().sickness.inoculation_calls
            );
            saved = true;
        }
        inoculated = sim
            .people()
            .sickness
            .episodes()
            .iter()
            .filter(|e| e.acquired == civ_agents::sickness::Acquired::Inoculated { by: variolator })
            .map(|e| (e.person, e.household))
            .collect();
        if inoculated.iter().any(|&(_, h)| h == Some(home)) {
            break;
        }
    }
    assert!(saved, "the call stood at a quarter-day");
    assert!(
        inoculated.iter().any(|&(_, h)| h == Some(home)),
        "someone of the calling household was inoculated: {inoculated:?}"
    );
    // Nobody is inoculated whose household did not call.
    assert!(
        inoculated
            .iter()
            .all(|&(_, h)| h.is_some_and(|h| callers.contains(&h))),
        "{inoculated:?} against callers {callers:?}"
    );
    // The variolator's page says whom they inoculated (fourth part).
    let n = inoculated.len();
    let whom = if n == 1 {
        "1 person".to_owned()
    } else {
        format!("{n} people")
    };
    let words = civ_sim::frames::sickness::care_words(&sim, variolator);
    assert!(
        words.contains(&format!("Inoculated {whom} against smallpox")),
        "{words}"
    );
}

/// Where someone died of smallpox, a household with ties to them that knows both one who knows
/// vaccination and one who knows only variolation calls the vaccinator; their household makes
/// matter for the call, they come, and its people not protected are vaccinated, a dose each, none
/// inoculated; those vaccinated save and load, and smallpox's hazards on them are turned away
/// (M10c slice ED, third part).
#[test]
fn a_household_calls_a_vaccinator_before_a_variolator_and_its_people_are_vaccinated() {
    let mut sim = village(1);
    to_morning(&mut sim, 8 * 60);
    let (index, _) = someone_with_family(&sim);
    sim.plague(index, SMALLPOX).expect("brought");
    let catalog = &sim.rules().catalog;
    let disease = catalog.disease_index(SMALLPOX).expect("smallpox") as u16;
    let variolation = catalog
        .technique_index("core:technique/variolation")
        .expect("variolation");
    let vaccination = catalog
        .technique_index("core:technique/vaccination")
        .expect("vaccination");
    let matter = catalog
        .good_index("core:good/vaccine_matter")
        .expect("matter");
    // Live until someone dies of it.
    let mut dead = None;
    for _ in 0..80 {
        sim.advance_minutes(DAY).expect("lives");
        dead = sim
            .people()
            .sickness
            .episodes()
            .iter()
            .find(|e| e.disease == disease && matches!(e.ended, Some((_, Outcome::Died))))
            .map(|e| e.person);
        if dead.is_some() {
            break;
        }
    }
    let dead = dead.expect("someone died of smallpox");
    let day = sim.now().day_index();
    // A household of people tied to the dead, with someone of it not protected, one of whom has
    // ties with two others of two other households of its settlement, alive and well.
    let people = sim.people();
    let (home, member, vaccinator, variolator) = people
        .households
        .iter()
        .find_map(|(_, x)| {
            let tied = x
                .members
                .iter()
                .any(|&m| people.ties.of(m).iter().any(|t| t.to == dead));
            if !tied
                || !x
                    .members
                    .iter()
                    .any(|&m| !people.sickness.protected(m, disease, day))
            {
                return None;
            }
            x.members.iter().find_map(|&m| {
                let mut known: Vec<(PermanentId, PermanentId)> = Vec::new();
                for v in people.ties.of(m).iter().map(|t| t.to) {
                    let Some(p) = people.person(v) else {
                        continue;
                    };
                    if p.household != x.id
                        && !people.sickness.ill(v, day)
                        && people
                            .household(p.household)
                            .is_some_and(|y| y.settlement == x.settlement)
                        && !known.iter().any(|k| k.1 == p.household)
                    {
                        known.push((v, p.household));
                    }
                }
                (known.len() >= 2).then(|| (x.id, m, known[0].0, known[1].0))
            })
        })
        .expect("a household tied to the dead, and two it knows of two households");
    let now = sim.now();
    for (_, p) in sim.people_mut_for_tests().people.iter_mut() {
        if p.id == vaccinator || p.id == variolator {
            p.come_to_know(variolation, civ_agents::person::KnowSource::Observer, now);
        }
        if p.id == vaccinator {
            p.come_to_know(vaccination, civ_agents::person::KnowSource::Observer, now);
        }
    }
    let theirs = sim.people().person(vaccinator).expect("alive").household;
    let called =
        sim.with_ctx_for_tests(|pop, ctx| pop.inoculation_call_for_tests(ctx, member, home));
    assert_eq!(
        called,
        Some(theirs),
        "they would call the vaccinator's household"
    );
    // They call; the vaccinator's household makes matter, and they come and vaccinate. While the
    // call stands the vaccinator's page says so (fourth part).
    let mut made = false;
    let mut called_words = false;
    let mut vaccinated = Vec::new();
    // Every household that called someone to inoculate it: others tied to the dead may call too.
    let mut callers = std::collections::BTreeSet::new();
    let home_members = sim.people().household(home).expect("home").members.clone();
    for _ in 0..40 {
        sim.advance_minutes(DAY / 8).expect("lives");
        callers.extend(
            sim.people()
                .sickness
                .inoculation_calls
                .iter()
                .map(|c| c.caller),
        );
        if !sim.people().sickness.inoculation_calls.is_empty() {
            let words = civ_sim::frames::sickness::care_words(&sim, vaccinator);
            called_words |= words.contains("Their household is called to inoculate the people of");
        }
        let people = sim.people();
        made |= people
            .household(theirs)
            .is_some_and(|y| y.stores.get(matter).copied().unwrap_or(0.0) > 0.0);
        vaccinated = people
            .sickness
            .vaccinated
            .iter()
            .filter(|v| v.disease == disease)
            .map(|v| v.person)
            .collect();
        if vaccinated.iter().any(|m| home_members.contains(m)) {
            break;
        }
    }
    assert!(made, "matter was made for the call");
    assert!(
        vaccinated.iter().any(|m| home_members.contains(m)),
        "someone of the calling household was vaccinated: {vaccinated:?}"
    );
    let people = sim.people();
    // Nobody is vaccinated whose household did not call.
    assert!(
        vaccinated.iter().all(|&m| people
            .person(m)
            .is_some_and(|p| callers.contains(&p.household))),
        "{vaccinated:?} against callers {callers:?}"
    );
    assert!(
        !people.sickness.episodes().iter().any(|e| matches!(
            e.acquired,
            civ_agents::sickness::Acquired::Inoculated { .. }
        )),
        "nobody was inoculated"
    );
    // Whole for 3-5 years, which turns away all but 5% of smallpox's hazards.
    let today = sim.now().day_index();
    let def = sim.rules().catalog.diseases[usize::from(disease)]
        .vaccination
        .expect("smallpox's vaccination");
    for v in &people.sickness.vaccinated {
        let years = (v.full_until - v.day) as f64 / 365.25;
        assert!((2.99..=5.01).contains(&years), "{years}");
        let share = people
            .sickness
            .unvaccinated_share(v.person, disease, today, &def);
        assert!((share - 0.05).abs() < 1e-9, "{share}");
    }
    // The observer reads it (fourth part): whom the vaccinator vaccinated, and by whom each
    // vaccinated was, whole for how long.
    assert!(called_words, "the call stood on the vaccinator's page");
    let n = people
        .sickness
        .vaccinated
        .iter()
        .filter(|v| v.by == Some(vaccinator))
        .count();
    assert_eq!(n, vaccinated.len());
    let whom = if n == 1 {
        "1 person".to_owned()
    } else {
        format!("{n} people")
    };
    let words = civ_sim::frames::sickness::care_words(&sim, vaccinator);
    assert!(
        words.contains(&format!("Vaccinated {whom} against smallpox")),
        "{words}"
    );
    let said = civ_sim::frames::sickness::sickness_words(&sim, vaccinated[0]);
    let by = people.name_of(vaccinator);
    assert!(
        said.first()
            .is_some_and(|w| w.starts_with("Vaccinated against smallpox")
                && w.contains(&format!(" by {by}, whole for "))),
        "{said:?}"
    );
    let loaded = reloaded(&mut sim);
    assert_eq!(
        loaded.people().sickness.vaccinated,
        sim.people().sickness.vaccinated
    );
}

/// Where someone of a household died of smallpox, which passes at home and in company, keeping
/// the sick apart comes before the gathering, against that death (M10c slice EE, first part).
#[test]
fn a_death_of_contagion_puts_keeping_the_sick_apart_before_the_gathering() {
    use civ_agents::polity::{IssueKind, LawStatus, PolicyKind};
    let mut sim = village(1);
    to_morning(&mut sim, 8 * 60);
    let (index, _) = someone_with_family(&sim);
    sim.plague(index, SMALLPOX).expect("brought");
    let disease = sim
        .rules()
        .catalog
        .disease_index(SMALLPOX)
        .expect("smallpox") as u16;
    let mut dead = None;
    for _ in 0..80 {
        sim.advance_minutes(DAY).expect("lives");
        dead = sim
            .people()
            .sickness
            .episodes()
            .iter()
            .find(|e| e.disease == disease && matches!(e.ended, Some((_, Outcome::Died))))
            .and_then(|e| e.household);
        if dead.is_some() {
            break;
        }
    }
    let home = dead.expect("someone died of smallpox");
    let settlement = sim
        .people()
        .household(home)
        .and_then(|x| x.settlement)
        .expect("settled");
    let pi = sim
        .people()
        .polities
        .iter()
        .position(|p| p.holds(Some(settlement)))
        .expect("its polity");
    sim.people_mut_for_tests().every_adult_deliberates = true;
    let mut proposed = None;
    for _ in 0..24 {
        {
            let pop = sim.people_mut_for_tests();
            pop.polities[pi].gathering = None;
            for l in &mut pop.polities[pi].laws {
                if l.status == LawStatus::Proposed {
                    l.status = LawStatus::Rejected;
                }
            }
        }
        sim.with_ctx_for_tests(|pop, ctx| pop.review_for_tests(ctx, settlement));
        match sim.people().polities[pi].agenda().cloned() {
            Some(l) if l.kind == PolicyKind::KeepApart => {
                proposed = Some(l);
                break;
            }
            Some(_) => continue,
            None => break,
        }
    }
    let law = proposed.expect("keeping the sick apart proposed");
    assert_eq!(law.issue, IssueKind::Contagion);
    reloaded(&mut sim);
}

/// Where its people had smallpox at home, a polity keeps a register of the sick only under one
/// who can write: with nobody who can, nobody proposes it; once someone can, it is proposed with
/// them named, and in force they enter in it, each midnight, the households whose sickness they
/// heard of; they walk a round of homes in daylight, entering each as seen, and where common work
/// is paid the store pays for the hours; the register saves and loads (M10c slice EE, second
/// part).
#[test]
fn a_register_of_the_sick_is_kept_by_one_who_can_write() {
    use civ_agents::polity::{IssueKind, LawStatus, PolicyKind};
    let mut sim = village(1);
    to_morning(&mut sim, 8 * 60);
    let (index, _) = someone_with_family(&sim);
    sim.plague(index, SMALLPOX).expect("brought");
    let home = sim.people().person(index).expect("alive").household;
    let settlement = sim
        .people()
        .household(home)
        .and_then(|x| x.settlement)
        .expect("settled");
    let pi = sim
        .people()
        .polities
        .iter()
        .position(|p| p.holds(Some(settlement)))
        .expect("its polity");
    let writing = sim
        .rules()
        .catalog
        .technique_index("core:technique/writing")
        .expect("writing");
    // Live until the sick lie abed, and their sickness is told.
    sim.advance_minutes(16 * DAY).expect("lives");
    let proposes = |sim: &mut Sim| {
        sim.people_mut_for_tests().every_adult_deliberates = true;
        let mut proposed = None;
        for _ in 0..24 {
            {
                let pop = sim.people_mut_for_tests();
                pop.polities[pi].gathering = None;
                for l in &mut pop.polities[pi].laws {
                    if l.status == LawStatus::Proposed {
                        l.status = LawStatus::Rejected;
                    }
                }
            }
            sim.with_ctx_for_tests(|pop, ctx| pop.review_for_tests(ctx, settlement));
            match sim.people().polities[pi].agenda().cloned() {
                Some(l) if l.kind == PolicyKind::KeepRegister => {
                    proposed = Some(l);
                    break;
                }
                Some(_) => continue,
                None => break,
            }
        }
        sim.people_mut_for_tests().every_adult_deliberates = false;
        proposed
    };
    assert!(
        proposes(&mut sim).is_none(),
        "nobody can write: no registrar"
    );
    // Someone grown of another household learns to write.
    let now = sim.now();
    let scribe = sim
        .people()
        .households
        .iter()
        .filter(|(_, x)| x.id != home && x.settlement == Some(settlement))
        .flat_map(|(_, x)| x.members.clone())
        .find(|&m| {
            sim.people()
                .person(m)
                .is_some_and(|p| p.age_years(now) >= 20.0)
        })
        .expect("a grown member of another household");
    let letters: Vec<usize> = ["reckoning", "reading", "writing"]
        .iter()
        .map(|t| {
            sim.rules()
                .catalog
                .technique_index(&format!("core:technique/{t}"))
                .expect("a letter technique")
        })
        .collect();
    for (_, p) in sim.people_mut_for_tests().people.iter_mut() {
        if p.id == scribe {
            for &t in &letters {
                p.come_to_know(t, civ_agents::person::KnowSource::Observer, now);
            }
        }
    }
    let law = proposes(&mut sim).expect("a register proposed");
    let holder = law.holder.expect("someone named");
    assert!(
        sim.people()
            .person(holder)
            .is_some_and(|p| p.knows(writing)),
        "the one named can write"
    );
    // Passed, with a law paying common work and coin in the store.
    let pay_work = sim
        .rules()
        .catalog
        .policies
        .iter()
        .position(|d| d.kind == PolicyKind::PayWork)
        .expect("a pay template");
    let coin = sim
        .rules()
        .catalog
        .good_index("core:good/coin")
        .expect("coin");
    let polity_id = sim.people().polities[pi].id;
    let pay_id = sim.allocate_id_for_tests();
    let today = sim.now().day_index();
    {
        let pop = sim.people_mut_for_tests();
        let p = &mut pop.polities[pi];
        p.gathering = None;
        let l = p.laws.iter_mut().find(|l| l.id == law.id).expect("the law");
        l.status = LawStatus::InForce;
        let mut pay = l.clone();
        pay.id = pay_id;
        pay.policy = pay_work as u16;
        pay.kind = PolicyKind::PayWork;
        pay.holder = None;
        pay.issue = IssueKind::CoinIdle;
        p.laws.push(pay);
        assert!(pop.strike_for_tests(coin, polity_id, polity_id, today, 100.0, 8.0, 1.0));
    }
    let holder_home = sim.people().person(holder).expect("alive").household;
    let coin_held = |sim: &Sim| {
        sim.people()
            .household(holder_home)
            .map_or(0.0, |x| x.stores.get(coin).copied().unwrap_or(0.0))
    };
    let before = coin_held(&sim);
    // In force: each midnight the registrar enters the sickness they heard of.
    let mut entered = false;
    for _ in 0..10 {
        sim.advance_minutes(DAY).expect("lives");
        if !sim.people().sickness.register.is_empty() {
            entered = true;
            break;
        }
    }
    assert!(entered, "the registrar entered the sickness they heard of");
    // Their tallies of where sickness comes from count every household the register holds as
    // sick (not those it holds only as seen on a round), and know where each draws.
    let (sick, draws) =
        sim.with_ctx_for_tests(|pop, ctx| pop.register_tally_for_tests(ctx, holder));
    let mut held: Vec<PermanentId> = sim
        .people()
        .sickness
        .register
        .iter()
        .filter(|r| r.kind != 2)
        .map(|r| r.household)
        .collect();
    held.sort_unstable();
    held.dedup();
    assert_eq!(sick, held);
    assert!(!draws.is_empty(), "they know where the registered draw");
    // They walk a round in daylight, entering the homes on it as seen, paid for the hours.
    let mut seen = false;
    for _ in 0..6 {
        if sim.people().sickness.register.iter().any(|r| r.kind == 2) {
            seen = true;
            break;
        }
        sim.advance_minutes(DAY).expect("lives");
    }
    assert!(seen, "the registrar walked a round and entered homes seen");
    let l = sim.people().polities[pi]
        .laws
        .iter()
        .find(|l| l.id == law.id)
        .expect("the law");
    assert!(l.watch.rounds > 0 && l.watch.minutes > 0.0, "{:?}", l.watch);
    assert!(
        coin_held(&sim) > before,
        "the registrar's household was paid: {} after {before}",
        coin_held(&sim)
    );
    // The polity's offices show the register (M10c slice EE, fourth part).
    let words = civ_sim::frames::government::services_words(&sim, &sim.people().polities[pi]);
    assert!(
        words
            .iter()
            .any(|w| w.starts_with("Register of the sick: ")),
        "{words:?}"
    );
    // A paid office asking letters: a household with a child of an age to learn sends them to
    // learn letters from the registrar, reckoning first, as reading and writing need it.
    let now = sim.now();
    let homes: Vec<PermanentId> = sim
        .people()
        .households
        .iter()
        .filter(|(_, x)| x.settlement == Some(settlement) && x.id != holder_home)
        .map(|(_, x)| x.id)
        .collect();
    let mut sent = None;
    for h in homes {
        sim.with_ctx_for_tests(|pop, ctx| pop.review_enrolment_for_tests(ctx, h));
        sent = sim
            .people()
            .enrolments
            .iter()
            .find(|e| e.teacher == holder && letters.contains(&usize::from(e.technique)))
            .copied();
        if sent.is_some() {
            break;
        }
    }
    let e = sent.expect("a child sent to learn letters");
    assert_eq!(usize::from(e.technique), letters[0], "reckoning first");
    assert!(
        sim.people()
            .person(e.learner)
            .is_some_and(|p| p.age_years(now) < 16.0)
    );
    let pop = sim.people();
    let polity = pop.polities[pi].id;
    assert!(pop.sickness.register.iter().all(|r| {
        r.polity == polity
            && pop
                .household(r.household)
                .is_none_or(|x| x.settlement == Some(settlement))
    }));
    let loaded = reloaded(&mut sim);
    assert_eq!(
        loaded.people().sickness.register,
        sim.people().sickness.register
    );
}

/// Where severe cases of smallpox lay with nobody tending them, the polity measures the days; where
/// it measured many (set here at sixty a year), someone who knows how to frame a building puts a
/// sickhouse raised together before the gathering against them; the measure saves and loads
/// (M10c slice EE, fourth part).
#[test]
fn severe_days_untended_put_a_sickhouse_before_the_gathering() {
    use civ_agents::polity::{IssueKind, LawStatus};
    let mut sim = village(1);
    to_morning(&mut sim, 8 * 60);
    let (index, _) = someone_with_family(&sim);
    // The whole household takes it at once: while its people lie ill, nobody well is at home to
    // tend the worst of them.
    let home = sim.people().person(index).expect("alive").household;
    let members = sim.people().household(home).expect("home").members.clone();
    for m in members {
        sim.plague(m, SMALLPOX).expect("brought");
    }
    let settlement = sim
        .people()
        .household(home)
        .and_then(|x| x.settlement)
        .expect("settled");
    let pi = sim
        .people()
        .polities
        .iter()
        .position(|p| p.holds(Some(settlement)))
        .expect("its polity");
    let untended = |sim: &Sim| {
        sim.people().polities[pi]
            .measured
            .iter()
            .map(|m| m.untended_days)
            .sum::<f64>()
    };
    for _ in 0..60 {
        sim.advance_minutes(DAY).expect("lives");
        if untended(&sim) > 0.0 {
            break;
        }
    }
    let days = untended(&sim);
    assert!(days > 0.0, "a severe case lay untended");
    // A day or two untended weighs little against a building's work: let the year's measure be
    // as an outbreak's of many weeks, that the weighing has something to weigh.
    {
        let pop = sim.people_mut_for_tests();
        if let Some(m) = pop.polities[pi].measured.last_mut() {
            m.untended_days = 60.0;
        }
    }
    // Everyone grown can frame a building.
    let frame = sim
        .rules()
        .catalog
        .technique_index("core:technique/jointed_frame")
        .expect("framing");
    let now = sim.now();
    for (_, p) in sim.people_mut_for_tests().people.iter_mut() {
        p.come_to_know(frame, civ_agents::person::KnowSource::Observer, now);
    }
    let sickhouse = sim
        .rules()
        .catalog
        .building_index("core:building/sickhouse")
        .expect("the program");
    let template = sim
        .rules()
        .catalog
        .policies
        .iter()
        .position(|d| d.raise.is_some_and(|r| r.program == sickhouse))
        .expect("the template");
    // A sickhouse repays its work to some household now.
    let gains = sim
        .with_ctx_for_tests(|pop, ctx| pop.rule_gains_for_tests(ctx, settlement, template as u16));
    assert!(gains.iter().any(|g| g.1 > 0.0), "{gains:?}");
    sim.people_mut_for_tests().every_adult_deliberates = true;
    let mut proposed = None;
    for _ in 0..30 {
        {
            let pop = sim.people_mut_for_tests();
            pop.polities[pi].gathering = None;
            for l in &mut pop.polities[pi].laws {
                if l.status == LawStatus::Proposed {
                    l.status = LawStatus::Rejected;
                }
            }
        }
        sim.with_ctx_for_tests(|pop, ctx| pop.review_for_tests(ctx, settlement));
        if let Some(l) = sim.people().polities[pi].agenda().cloned()
            && usize::from(l.policy) == template
        {
            proposed = Some(l);
            break;
        }
        sim.advance_minutes(DAY).expect("lives");
    }
    let law = proposed.expect("a sickhouse proposed");
    assert_eq!(law.issue, IssueKind::Untended);
    let loaded = reloaded(&mut sim);
    assert_eq!(
        loaded.people().polities[pi].measured,
        sim.people().polities[pi].measured
    );
}

/// A polity with a sickhouse standing finished (raised as its law would raise it): once smallpox
/// is brought, a severe case whom nobody tended the day before is admitted at midnight while beds
/// are left; those lying there are tended there by people of the polity; those no longer ill
/// leave; who lies there and who tended there save and load (M10c slice EE, fourth part).
#[test]
fn a_sickhouse_admits_the_untended_severe_and_they_are_tended_there() {
    use civ_agents::polity::{LawStatus, Outcome as Decided};
    let mut sim = village(1);
    to_morning(&mut sim, 8 * 60);
    let (index, _) = someone_with_family(&sim);
    let home = sim.people().person(index).expect("alive").household;
    let settlement = sim
        .people()
        .household(home)
        .and_then(|x| x.settlement)
        .expect("settled");
    let pi = sim
        .people()
        .polities
        .iter()
        .position(|p| p.holds(Some(settlement)))
        .expect("its polity");
    let sickhouse = sim
        .rules()
        .catalog
        .building_index("core:building/sickhouse")
        .expect("the program");
    let template = sim
        .rules()
        .catalog
        .policies
        .iter()
        .position(|d| d.raise.is_some_and(|r| r.program == sickhouse))
        .expect("the template");
    // Measured, it is proposed by someone who can frame it; passed, laid out and raised.
    let year = sim.now().date().year;
    sim.people_mut_for_tests().polities[pi]
        .measured_mut(year)
        .untended_days = 60.0;
    let frame = sim
        .rules()
        .catalog
        .technique_index("core:technique/jointed_frame")
        .expect("framing");
    let now = sim.now();
    for (_, p) in sim.people_mut_for_tests().people.iter_mut() {
        p.come_to_know(frame, civ_agents::person::KnowSource::Observer, now);
    }
    sim.people_mut_for_tests().every_adult_deliberates = true;
    let mut proposed = None;
    for _ in 0..30 {
        {
            let pop = sim.people_mut_for_tests();
            pop.polities[pi].gathering = None;
            for l in &mut pop.polities[pi].laws {
                if l.status == LawStatus::Proposed {
                    l.status = LawStatus::Rejected;
                }
            }
        }
        sim.with_ctx_for_tests(|pop, ctx| pop.review_for_tests(ctx, settlement));
        if let Some(l) = sim.people().polities[pi].agenda().cloned()
            && usize::from(l.policy) == template
        {
            proposed = Some(l);
            break;
        }
        sim.advance_minutes(DAY).expect("lives");
    }
    let law = proposed.expect("a sickhouse proposed");
    {
        let today = sim.now().day_index();
        let pop = sim.people_mut_for_tests();
        pop.every_adult_deliberates = false;
        let grown: Vec<_> = pop.people.iter().map(|(_, p)| (p.id, today)).collect();
        let p = &mut pop.polities[pi];
        p.gathering = None;
        let l = p.laws.iter_mut().find(|l| l.id == law.id).expect("the law");
        l.status = LawStatus::InForce;
        l.outcome = Some(Decided::Passed);
        l.known = grown;
    }
    sim.with_ctx_for_tests(|pop, ctx| pop.begin_public_raise_for_tests(ctx, pi, law.id));
    let building = sim.people().polities[pi]
        .raise_site_of(law.id)
        .and_then(|s| s.building)
        .expect("laid out");
    for _ in 0..8000 {
        if sim.people().polities[pi].sickhouse.is_some() {
            break;
        }
        sim.with_ctx_for_tests(|pop, ctx| pop.raise_work_for_tests(ctx, home, building, 4.0));
    }
    assert_eq!(sim.people().polities[pi].sickhouse, Some(building));
    let (_, beds) = sim.with_ctx_for_tests(|pop, ctx| pop.sickhouse_for_tests(ctx, pi));
    assert!(beds >= 4, "{beds} beds");
    // Smallpox comes; an untended severe case is admitted.
    sim.plague(index, SMALLPOX).expect("brought");
    let mut lying = Vec::new();
    for _ in 0..80 {
        sim.advance_minutes(DAY).expect("lives");
        let (now_lying, _) = sim.with_ctx_for_tests(|pop, ctx| pop.sickhouse_for_tests(ctx, pi));
        if !now_lying.is_empty() {
            lying = now_lying;
            break;
        }
    }
    assert!(!lying.is_empty(), "someone untended was admitted");
    assert!(lying.len() <= beds);
    let day = sim.now().day_index();
    for &m in &lying {
        assert!(
            sim.people().sickness.ill(m, day),
            "those lying there are ill"
        );
    }
    // They are tended there: someone tended there within days, and their care grew.
    let care = |sim: &Sim, m: PermanentId| -> f32 {
        sim.people()
            .sickness
            .of(m)
            .filter(|e| e.ended.is_none())
            .map(|e| e.care.total_h)
            .sum()
    };
    let before: Vec<f32> = lying.iter().map(|&m| care(&sim, m)).collect();
    let mut staffed = false;
    for _ in 0..12 {
        sim.advance_minutes(DAY / 4).expect("lives");
        if !sim.people().staffed.is_empty() {
            staffed = true;
            let loaded = reloaded(&mut sim);
            assert_eq!(loaded.people().staffed, sim.people().staffed);
            assert_eq!(
                loaded.people().sickness.admitted,
                sim.people().sickness.admitted
            );
            assert_eq!(
                loaded.people().polities[pi].sickhouse,
                sim.people().polities[pi].sickhouse
            );
            break;
        }
    }
    assert!(staffed, "someone tended at the sickhouse");
    let words = civ_sim::frames::government::services_words(&sim, &sim.people().polities[pi]);
    assert!(
        words
            .iter()
            .any(|w| w.starts_with("Sickhouse: ") && w.contains(&format!("of {beds} beds taken"))),
        "{words:?}"
    );
    let after: Vec<f32> = lying.iter().map(|&m| care(&sim, m)).collect();
    assert!(
        after.iter().zip(&before).any(|(a, b)| a > b),
        "their care grew: {before:?} -> {after:?}"
    );
    // Those no longer ill leave.
    sim.advance_minutes(40 * DAY).expect("lives");
    let day = sim.now().day_index();
    let pop = sim.people();
    assert!(
        pop.sickness
            .admitted
            .iter()
            .all(|a| pop.sickness.ill(a.person, day)),
        "only the ill lie there"
    );
}
