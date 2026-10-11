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
    // Twenty hours of tending, in sessions: they know it at the end and not before.
    for session in 0..20 {
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
    // 1.5 days and three spreads of 1: five days), and is told with its counts.
    let mut ended = None;
    for _ in 0..80 {
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
    assert_eq!(ended, last + 5);
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
