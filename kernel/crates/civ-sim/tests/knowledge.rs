//! Knowledge carried by people (M3b slice M, ADR-0008): founders arrive knowing their work and
//! children learn it at its age; work needs its technique, and someone who does not know it learns
//! by working beside someone who does; a technique is lost with its last knower there; the
//! observer can introduce one; and older saves load with what founders know.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::ChronicleKind;
use civ_agents::knowledge::{KnowledgeEventKind, knowing_age};
use civ_agents::person::KnowSource;
use civ_content::ContentRegistry;
use civ_core::{PermanentId, SimTime};
use civ_sim::persist::{self, SECTION_CONTENT, SECTION_META, agents};
use civ_sim::{NewWorld, Sim};
use commons_persist::{SaveDir, SaveKind};

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

fn world(content: &ContentRegistry, seed: u64) -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Knowledge".to_owned(),
            seed,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 512,
            band_size: 0,
            neighbours: Vec::new(),
            neighbours_known: false,
            regime_id: String::new(),
        },
        content,
        [7; 16],
    )
    .expect("generates")
}

fn technique(sim: &Sim, id: &str) -> usize {
    sim.rules().catalog.technique_index(id).expect(id)
}

#[test]
fn founders_know_their_work_and_the_settlement_records_what_they_brought() {
    let sim = world(content(), 3);
    let (catalog, params) = (&sim.rules().catalog, &sim.rules().people);
    let now = sim.now();
    let settlement = sim.land().settlements[0].id;
    let founders: Vec<usize> = params.knowledge.founders.iter().map(|&(t, _)| t).collect();
    for (t, def) in catalog.techniques.iter().enumerate() {
        if !founders.contains(&t) {
            // What founders do not bring nobody knows yet: it is found, or brought in.
            assert!(sim.people().people.iter().all(|(_, p)| p.know(t).is_none()));
            assert!(!sim.people().known_in(settlement, t), "{}", def.name);
            continue;
        }
        let age = knowing_age(catalog, t, params.family.independent_age).expect("gates work");
        // What only some founders bring (laying out by cord and stake, M8c slice CT), only
        // those of its age may know.
        let share = params
            .knowledge
            .founders
            .iter()
            .find(|f| f.0 == t)
            .map_or(0.0, |f| f.1);
        if share < 1.0 {
            assert!(
                sim.people()
                    .people
                    .iter()
                    .all(|(_, p)| !p.knows(t) || p.age_years(now) >= age),
                "{}",
                def.name
            );
            continue;
        }
        for (_, p) in sim.people().people.iter() {
            assert_eq!(
                p.knows(t),
                p.age_years(now) >= age,
                "{} aged {:.1} and {}",
                p.given,
                p.age_years(now),
                def.name
            );
        }
        assert!(
            sim.people().known_in(settlement, t),
            "{} is known",
            def.name
        );
    }
    let brought = sim
        .people()
        .knowledge
        .iter()
        .filter(|e| e.kind == KnowledgeEventKind::Known(KnowSource::Founder))
        .count();
    // One record each of what the settlement's founders brought.
    let known = founders
        .iter()
        .filter(|&&t| sim.people().known_in(settlement, t))
        .count();
    assert_eq!(brought, known, "one record each");
    assert!(known >= founders.len() - 1);
}

#[test]
fn a_child_learns_the_household_s_work_on_reaching_its_age() {
    let mut sim = world(content(), 3);
    let emmer = technique(&sim, "core:technique/emmer_growing");
    let now = sim.now();
    // The youngest child not yet old enough to tend crops (from 7) is made a day short of 7.
    let child = sim
        .people()
        .people
        .iter()
        .map(|(_, p)| p)
        .filter(|p| p.age_years(now) < 7.0)
        .max_by_key(|p| p.id)
        .map(|p| p.id)
        .expect("the band has young children");
    let born = SimTime::from_minutes(now.minutes() - (7.0 * 525_600.0) as i64 + 12 * 60);
    let pop = sim.people_mut_for_tests();
    let h = pop
        .people
        .iter_mut()
        .map(|(_, p)| p)
        .find(|p| p.id == child)
        .expect("child");
    h.born = born;
    assert!(!h.knows(emmer));
    sim.advance_minutes(2 * 24 * 60).expect("advances");
    let p = sim.people().person(child).expect("alive");
    assert!(p.knows(emmer), "brought up with it at seven");
    let KnowSource::Upbringing(from) = p.know(emmer).expect("known").source else {
        panic!("learnt at home");
    };
    let teacher = sim.people().person(from).expect("a living teacher");
    assert_eq!(teacher.household, p.household, "from their own household");
    assert!(teacher.knows(emmer));
    // Upbringing is everyday work: the chronicle does not note it.
    assert!(
        !sim.people()
            .chronicle
            .iter()
            .any(|e| e.kind == ChronicleKind::TechniqueLearned)
    );
}

#[test]
fn work_first_done_once_grown_is_learnt_at_home_before_growing_up() {
    // Shaping stone gates making an axe or a quern, from 16, after people may keep a household
    // of their own (from 15): a child learns it at home in the year before growing up, so nobody
    // leaves home without it.
    let mut sim = world(content(), 3);
    let stone = technique(&sim, "core:technique/stone_shaping");
    let grown = sim.rules().people.family.independent_age;
    let now = sim.now();
    let child = sim
        .people()
        .people
        .iter()
        .map(|(_, p)| p)
        .filter(|p| p.age_years(now) < grown - 1.0 && !p.knows(stone))
        .max_by(|a, b| a.age_years(now).total_cmp(&b.age_years(now)))
        .map(|p| p.id)
        .expect("the band has a child");
    let age_min = ((grown - 1.0) * 525_600.0) as i64;
    let born = SimTime::from_minutes(now.minutes() - age_min + 12 * 60);
    for (_, p) in sim.people_mut_for_tests().people.iter_mut() {
        if p.id == child {
            p.born = born;
        }
    }
    sim.advance_minutes(2 * 24 * 60).expect("advances");
    let p = sim.people().person(child).expect("alive");
    assert!(
        matches!(
            p.know(stone).map(|k| k.source),
            Some(KnowSource::Upbringing(_))
        ),
        "brought up with it a year before growing up: {:?}",
        p.know(stone)
    );
}

#[test]
fn the_observer_can_introduce_a_technique() {
    let mut sim = world(content(), 3);
    let emmer = technique(&sim, "core:technique/emmer_growing");
    let now = sim.now();
    let mut young: Vec<(PermanentId, f64)> = sim
        .people()
        .people
        .iter()
        .map(|(_, p)| (p.id, p.age_years(now)))
        .filter(|(_, a)| *a < 7.0)
        .collect();
    young.sort_by_key(|(id, _)| *id);
    let [(first, _), (second, _), ..] = young[..] else {
        panic!("the band has two young children");
    };
    sim.introduce_technique(first, "core:technique/emmer_growing", false)
        .expect("introduced");
    let p = sim.people().person(first).expect("alive");
    assert!(p.knows(emmer));
    assert_eq!(p.know(emmer).map(|k| k.source), Some(KnowSource::Observer));
    let last = sim.people().chronicle.last().expect("an entry");
    assert_eq!(last.kind, ChronicleKind::TechniqueIntroduced);
    assert_eq!(last.people, vec![first]);
    assert_eq!(last.number, 0.0);
    assert!(
        sim.introduce_technique(first, "core:technique/emmer_growing", false)
            .is_err(),
        "nobody learns what they know"
    );
    sim.introduce_technique(second, "core:technique/emmer_growing", true)
        .expect("heard of");
    let p = sim.people().person(second).expect("alive");
    assert!(!p.knows(emmer) && p.know(emmer).is_some(), "heard of only");
    assert_eq!(sim.people().chronicle.last().map(|e| e.number), Some(1.0));
    assert!(
        sim.introduce_technique(first, "core:technique/flying", false)
            .is_err()
    );
}

#[test]
fn a_technique_is_lost_with_the_last_who_knew_it_there() {
    let mut sim = world(content(), 3);
    let knapping = technique(&sim, "core:technique/knapping");
    let settlement = sim.land().settlements[0].id;
    let now = sim.now();
    // Only the eldest knapper keeps the craft.
    let last = sim
        .people()
        .people
        .iter()
        .map(|(_, p)| p)
        .filter(|p| p.knows(knapping))
        .min_by_key(|p| (p.born, p.id))
        .map(|p| p.id)
        .expect("a knapper");
    for (_, p) in sim.people_mut_for_tests().people.iter_mut() {
        if p.id != last {
            p.knows.retain(|k| usize::from(k.technique) != knapping);
        }
    }
    let before = sim.people().chronicle.len();
    sim.die_for_tests(last);
    assert!(!sim.people().known_in(settlement, knapping));
    let lost: Vec<_> = sim.people().chronicle[before..]
        .iter()
        .filter(|e| e.kind == ChronicleKind::TechniqueLost)
        .collect();
    assert_eq!(lost.len(), 1, "one loss, noted once");
    assert_eq!(lost[0].people, vec![last]);
    assert_eq!(lost[0].settlement, Some(settlement));
    // Nobody is learning it, and the households' sickles remain.
    assert_eq!(lost[0].number, civ_agents::population::LOST_MADE_REMAIN);
    let record = sim
        .people()
        .knowledge
        .iter()
        .rev()
        .find(|e| usize::from(e.technique) == knapping)
        .expect("recorded");
    assert_eq!(record.kind, KnowledgeEventKind::Lost);
    assert_eq!(record.person, last);
    assert!(record.at >= now);
}

#[test]
fn an_older_save_gains_a_technique_founders_bring_that_the_content_has_added() {
    let mut sim = world(content(), 3);
    let pottery = technique(&sim, "core:technique/pottery");
    let knapping = technique(&sim, "core:technique/knapping");
    let settlement = sim.land().settlements[0].id;
    // Knapping is lost there before the save: a technique the save names stays as it was.
    for (_, p) in sim.people_mut_for_tests().people.iter_mut() {
        p.knows.retain(|k| usize::from(k.technique) != knapping);
    }
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        SaveDir::create(dir.path().join("saves"), civ_schema::SAVE_EXTENSION).expect("save dir");
    let saved = persist::save(&mut sim, &saves, SaveKind::Manual, "older").expect("saves");
    // Content in which pottery is new: the save names it by an id this content does not have.
    let mut newer = content().clone();
    newer.catalog.techniques[pottery].id = "core:technique/pottery_anew".to_owned();
    let loaded = persist::load(&saved.path, &newer).expect("loads");
    // Everyone knows it as a founder of their age would bring it: as they did when founded.
    for (_, p) in sim.people().people.iter() {
        let q = loaded.people().person(p.id).expect("loaded");
        assert_eq!(q.knows(pottery), p.knows(pottery), "{}", p.given);
        assert!(!q.knows(knapping), "{} learnt knapping again", p.given);
    }
    assert!(sim.people().people.iter().any(|(_, p)| p.knows(pottery)));
    // The settlement's record notes it once, as brought by its eldest knower, now.
    let noted: Vec<_> = loaded
        .people()
        .knowledge
        .iter()
        .filter(|e| usize::from(e.technique) == pottery)
        .collect();
    assert_eq!(noted.len(), 1);
    assert_eq!(noted[0].settlement, settlement);
    assert_eq!(
        noted[0].kind,
        KnowledgeEventKind::Known(KnowSource::Founder)
    );
    assert_eq!(noted[0].at, sim.now());
    assert!(loaded.people().known_in(settlement, pottery));
    let records = |s: &Sim| {
        s.people()
            .knowledge
            .iter()
            .filter(|e| usize::from(e.technique) == knapping)
            .count()
    };
    assert_eq!(
        records(&loaded),
        records(&sim),
        "knapping's record is as it was"
    );
}

#[test]
fn someone_who_does_not_know_a_craft_learns_it_working_beside_a_knower() {
    // A variant of the core content in which knapping is a craft learnt by working beside a
    // knapper (not brought up with), quickly enough for a test.
    let mut variant = content().clone();
    let knapping = variant
        .catalog
        .technique_index("core:technique/knapping")
        .expect("knapping");
    variant.catalog.techniques[knapping].upbringing = false;
    variant.catalog.techniques[knapping].learn_h = 2.0;
    let mut sim = world(&variant, 3);
    let now = sim.now();
    // A household of a knapper and another grown member, who forgets the craft; it has no
    // sickles, and flint and wood to make them.
    let pop = sim.people();
    let (household, learner) = pop
        .households
        .iter()
        .map(|(_, h)| h)
        .filter_map(|h| {
            let grown: Vec<_> = h
                .members
                .iter()
                .filter_map(|m| pop.person(*m))
                .filter(|p| p.age_years(now) >= 16.0 && p.knows(knapping))
                .collect();
            (grown.len() >= 2).then(|| (h.id, grown[1].id))
        })
        .min_by_key(|(h, _)| *h)
        .expect("a household with two grown knappers");
    let goods = sim.rules().catalog.goods.clone();
    let good = |id: &str| goods.iter().position(|g| g.id == id).expect(id);
    let (sickle, toolstone, timber) = (
        good("core:good/sickle"),
        good("core:good/toolstone"),
        good("core:good/timber"),
    );
    let pop = sim.people_mut_for_tests();
    for (_, p) in pop.people.iter_mut() {
        if p.id == learner {
            p.knows.retain(|k| usize::from(k.technique) != knapping);
        }
    }
    for (_, h) in pop.households.iter_mut() {
        if h.id == household {
            h.stores[sickle] = 0.0;
            h.stores[toolstone] += 20.0;
            h.stores[timber] += 20.0;
        }
    }
    let mut learnt = false;
    for _ in 0..60 {
        sim.advance_minutes(24 * 60).expect("advances");
        if sim
            .people()
            .person(learner)
            .is_some_and(|p| p.knows(knapping))
        {
            learnt = true;
            break;
        }
    }
    assert!(learnt, "the learner learnt knapping beside a knapper");
    let p = sim.people().person(learner).expect("alive");
    let KnowSource::Taught(teacher) = p.know(knapping).expect("known").source else {
        panic!("taught");
    };
    let t = sim.people().person(teacher).expect("the teacher lives");
    assert!(t.knows(knapping));
    assert_eq!(t.household, household, "a member of their household");
    let entry = sim
        .people()
        .chronicle
        .iter()
        .find(|e| e.kind == ChronicleKind::TechniqueLearned)
        .expect("a craft learnt is noted");
    assert_eq!(entry.people, vec![learner, teacher]);
}

/// The core content with no technique the founders bring gating any work.
fn ungated() -> ContentRegistry {
    let mut ungated = content().clone();
    let founders: Vec<usize> = ungated
        .people
        .params
        .knowledge
        .founders
        .iter()
        .map(|&(t, _)| t)
        .collect();
    // What nobody knows at first (drying, the rotary quern) still needs knowing in both.
    let open = |t: &mut Option<usize>| {
        if t.is_some_and(|x| founders.contains(&x)) {
            *t = None;
        }
    };
    for a in &mut ungated.catalog.activities {
        open(&mut a.technique);
    }
    for r in &mut ungated.catalog.recipes {
        open(&mut r.technique);
    }
    for b in &mut ungated.catalog.buildings {
        open(&mut b.technique);
    }
    ungated
}

/// Saves `sim`, loads the save with the core content and with no technique the founders bring
/// gating any work, and lives `days` in both: they must do the same, and come to the same.
/// People do the same things in the same places, the chronicle tells the same story (but for
/// what was known, learnt or lost) and everything saved is the same but what people know and the
/// content itself. Returns the world with the techniques.
fn same_life(sim: &mut Sim, days: i64) -> Sim {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        SaveDir::create(dir.path().join("saves"), civ_schema::SAVE_EXTENSION).expect("save dir");
    let saved = persist::save(sim, &saves, SaveKind::Manual, "both").expect("saves");
    let mut gated = persist::load(&saved.path, content()).expect("loads");
    let mut free = persist::load(&saved.path, &ungated()).expect("loads without techniques");
    for s in [&mut gated, &mut free] {
        s.advance_minutes(days * 24 * 60).expect("advances");
    }
    let doing = |s: &Sim| -> Vec<_> {
        s.people()
            .people
            .iter()
            .map(|(_, p)| {
                (
                    p.id,
                    p.pos,
                    p.act.def,
                    p.act.target,
                    p.act.step,
                    p.energy_kcal,
                )
            })
            .collect()
    };
    assert_eq!(doing(&gated), doing(&free), "people do the same");
    let story = |s: &Sim| -> Vec<_> {
        s.people()
            .chronicle
            .iter()
            .filter(|e| {
                !matches!(
                    e.kind,
                    ChronicleKind::TechniqueFound
                        | ChronicleKind::TechniqueLearned
                        | ChronicleKind::TechniqueLost
                        | ChronicleKind::TechniqueIntroduced
                )
            })
            .map(|e| (e.at, e.kind, e.people.clone(), e.settlement, e.number))
            .collect()
    };
    assert!(
        story(&gated) == story(&free),
        "the chronicle tells the same story"
    );
    let skip = [
        SECTION_META,
        SECTION_CONTENT,
        agents::SECTION_PEOPLE,
        agents::SECTION_HISTORY,
        agents::SECTION_KNOW,
    ];
    let sections = |s: &Sim| -> Vec<_> {
        persist::encode_sections(s)
            .into_iter()
            .filter(|x| !skip.contains(&x.tag))
            .map(|x| (x.tag.to_string(), x.index, x.bytes))
            .collect()
    };
    let (a, b) = (sections(&gated), sections(&free));
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(&b) {
        assert!(x == y, "section {} {} differs", x.0, x.1);
    }
    gated
}

#[test]
fn knowing_every_technique_changes_nothing_people_do() {
    // Every founder brings every technique of today's work and children are brought up with them
    // as they reach their work's age, so a world lives exactly as it would if none of them gated
    // any work.
    let mut sim = world(content(), 3);
    // The youngest child not yet old enough to tend crops reaches it half a day after the save.
    let now = sim.now();
    let child = sim
        .people()
        .people
        .iter()
        .map(|(_, p)| p)
        .filter(|p| p.age_years(now) < 7.0)
        .max_by_key(|p| p.id)
        .map(|p| p.id)
        .expect("the band has young children");
    let born = SimTime::from_minutes(now.minutes() - (7.0 * 525_600.0) as i64 + 12 * 60);
    for (_, p) in sim.people_mut_for_tests().people.iter_mut() {
        if p.id == child {
            p.born = born;
        }
    }
    let gated = same_life(&mut sim, 120);
    let tended = gated
        .people()
        .person(child)
        .and_then(|p| p.know(technique(&gated, "core:technique/emmer_growing")))
        .map(|k| k.source);
    assert!(
        matches!(tended, Some(KnowSource::Upbringing(_))),
        "the child was brought up with emmer: {tended:?}"
    );
}

#[test]
#[ignore = "lives ten years twice, some minutes; run with --ignored"]
fn ten_years_knowing_every_technique_change_nothing() {
    let mut sim = world(content(), 3);
    let gated = same_life(&mut sim, 10 * 365);
    // And everyone old enough for each technique's work knows it, but for any who reached its
    // age since the day began, who learn it at their first such work or the next day.
    let (catalog, params) = (&gated.rules().catalog, &gated.rules().people);
    let now = gated.now();
    for &(t, _) in &params.knowledge.founders {
        let age = knowing_age(catalog, t, params.family.independent_age).expect("gates work");
        for (_, p) in gated.people().people.iter() {
            let years = p.age_years(now);
            assert!(
                !p.knows(t) || years >= age,
                "{} knows it too young",
                p.given
            );
            assert!(
                p.knows(t) || years < age + 1.0 / 365.0,
                "{} does not know it",
                p.given
            );
        }
    }
}

/// Someone enrolled to learn a craft from a knower of another household studies it with them,
/// is credited at midnight up to what was taught, pays the day's fee, and comes to know it from
/// them; in both resolutions (M10c slice EB, second part; ADR-0035 §5). A variant of the core
/// content in which knapping is learnt in three hours, never at home.
#[test]
fn a_learner_enrolled_with_a_knower_of_another_household_studies_with_them_and_learns() {
    use civ_agents::knowledge::Enrolment;
    use civ_agents::ledger::Channel;
    for resolved in [false, true] {
        let mut variant = content().clone();
        let knapping = variant
            .catalog
            .technique_index("core:technique/knapping")
            .expect("knapping");
        variant.catalog.techniques[knapping].upbringing = false;
        variant.catalog.techniques[knapping].learn_h = 3.0;
        let mut sim = world(&variant, 3);
        if resolved {
            sim.advance_to_midnight().expect("advances");
            let settlements: Vec<_> = sim.land().settlements.iter().map(|s| s.id).collect();
            sim.set_household_resolved(settlements);
            sim.advance_minutes(1).expect("advances");
            sim.advance_to_midnight().expect("advances");
            assert!(!sim.household_resolved().is_empty(), "household-resolved");
        }
        let now = sim.now();
        let adult = |sim: &Sim, h: PermanentId| {
            sim.people()
                .household(h)
                .expect("household")
                .members
                .iter()
                .copied()
                .find(|&m| {
                    sim.people()
                        .person(m)
                        .is_some_and(|p| p.age_years(now) >= 20.0 && p.knows(knapping))
                })
        };
        let homes: Vec<PermanentId> = sim.people().households.iter().map(|(_, x)| x.id).collect();
        let (teacher, learner, payer) = homes
            .iter()
            .find_map(|&a| {
                let t = adult(&sim, a)?;
                let b = homes
                    .iter()
                    .copied()
                    .find(|&b| b != a && adult(&sim, b).is_some())?;
                Some((t, adult(&sim, b)?, b))
            })
            .expect("two households with grown knappers");
        let day = sim.now().day_index();
        // Nobody of the learner's household knaps, so the teacher is the only one to learn from.
        let home = sim
            .people()
            .household(payer)
            .expect("household")
            .members
            .clone();
        let pop = sim.people_mut_for_tests();
        for (_, p) in pop.people.iter_mut() {
            if home.contains(&p.id) {
                p.knows.retain(|k| usize::from(k.technique) != knapping);
            }
        }
        pop.enrolments.push(Enrolment {
            teacher,
            learner,
            technique: knapping as u16,
            fee_kcal: 1_000.0,
            payer,
            since: day,
            paid: -1,
        });
        let mut learnt = None;
        for d in 0..10 {
            sim.advance_minutes(24 * 60).expect("advances");
            if d == 0 {
                // Mid-way, the enrolment and its sessions save and load alike.
                let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
                let saves = SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
                let saved =
                    persist::save(&mut sim, &saves, SaveKind::Manual, "taught").expect("saves");
                let loaded = persist::load(&saved.path, &variant).expect("loads");
                assert_eq!(loaded.people().enrolments, sim.people().enrolments);
                assert_eq!(loaded.people().studied, sim.people().studied);
            }
            let p = sim.people().person(learner).expect("alive");
            if p.knows(knapping) {
                learnt = p
                    .knows
                    .iter()
                    .find(|k| usize::from(k.technique) == knapping)
                    .map(|k| k.source);
                break;
            }
        }
        assert_eq!(
            learnt,
            Some(KnowSource::Taught(teacher)),
            "resolved {resolved}: learnt knapping from the teacher"
        );
        assert!(
            sim.people().enrolments.is_empty(),
            "the enrolment ends once learnt"
        );
        let goods = &sim.rules().catalog.goods;
        let fees: f64 = (0..goods.len())
            .map(|g| sim.people().transfers.get(Channel::Fee, g) * goods[g].kcal_per_kg)
            .sum();
        assert!(
            fees >= 1_000.0 - 1.0,
            "resolved {resolved}: fees paid: {fees}"
        );
    }
}

/// A household that bought pots twice from a potter's household, none of its own knowing
/// pottery, sends its oldest child of learning age to learn it from the potter, for a day's fee
/// in food a day of study, and the child studies with them (M10c slice EB, second part).
#[test]
fn a_household_that_buys_what_it_cannot_make_sends_a_child_to_learn_it_from_the_maker() {
    use civ_agents::ledger::{Channel, Trade};
    let mut sim = world(content(), 3);
    sim.advance_minutes(2 * 24 * 60).expect("advances");
    let pottery = sim
        .rules()
        .catalog
        .technique_index("core:technique/pottery")
        .expect("pottery");
    let pot = sim
        .rules()
        .catalog
        .goods
        .iter()
        .position(|g| g.id == "core:good/pot")
        .expect("pot") as u16;
    let now = sim.now();
    let homes: Vec<PermanentId> = sim.people().households.iter().map(|(_, x)| x.id).collect();
    // A household with someone of fewer than fifteen years, made ten, and a potter's household.
    let buyer = homes
        .iter()
        .copied()
        .find(|&h| sim.people().household(h).expect("home").members.len() >= 2)
        .expect("a household of two or more");
    let child = *sim
        .people()
        .household(buyer)
        .expect("home")
        .members
        .last()
        .expect("a member");
    let (seller, potter) = homes
        .iter()
        .copied()
        .filter(|&h| h != buyer)
        .find_map(|h| {
            let p = sim
                .people()
                .household(h)?
                .members
                .iter()
                .copied()
                .find(|&m| {
                    sim.people()
                        .person(m)
                        .is_some_and(|p| p.age_years(now) >= 20.0)
                })?;
            Some((h, p))
        })
        .expect("another household with an adult");
    let settlement = sim
        .people()
        .household(buyer)
        .and_then(|x| x.settlement)
        .expect("settled");
    let members = sim.people().household(buyer).expect("home").members.clone();
    let pop = sim.people_mut_for_tests();
    for (_, p) in pop.people.iter_mut() {
        if members.contains(&p.id) {
            p.knows.retain(|k| usize::from(k.technique) != pottery);
        }
        if p.id == child {
            p.born = now.plus_minutes(-10 * 365 * 24 * 60);
        }
        if p.id == potter && !p.knows(pottery) {
            p.learn(pottery, 1_000.0, potter, 1.0, now);
        }
    }
    let market = pop
        .markets
        .iter_mut()
        .find(|m| m.settlement == settlement)
        .expect("a market");
    for _ in 0..2 {
        market.recent.push_back(Trade {
            at: now,
            seller,
            buyer,
            good: pot,
            units: 1.0,
            payment: 0,
            paid: 1.0,
            channel: Channel::Barter,
            from: None,
        });
    }
    sim.with_ctx_for_tests(|pop, ctx| pop.review_enrolment_for_tests(ctx, buyer));
    let e = *sim.people().enrolments.first().expect("enrolled");
    assert_eq!(
        (e.learner, e.technique as usize, e.payer),
        (child, pottery, buyer)
    );
    assert_eq!(
        sim.people().person(e.teacher).map(|p| p.household),
        Some(seller),
        "taught by the seller's household"
    );
    let fee = 0.5 * members.len() as f64 * sim.rules().people.household.daily_kcal_per_person;
    assert!((f64::from(e.fee_kcal) - fee).abs() < 1e-3, "{}", e.fee_kcal);
    // The child studies with the teacher: credited at midnight.
    sim.advance_minutes(3 * 24 * 60).expect("advances");
    let hours = sim
        .people()
        .person(child)
        .expect("alive")
        .knows
        .iter()
        .find(|k| usize::from(k.technique) == pottery)
        .map_or(0.0, |k| f64::from(k.hours));
    assert!(hours > 0.0, "the child studied pottery: {hours} h");
    // The person pages say it (wire 1.105).
    let craft = sim.rules().catalog.techniques[pottery].name.to_lowercase();
    let child_words = civ_sim::frames::people::teaching_words(&sim, child);
    assert!(
        child_words.starts_with(&format!("Studies {craft} with "))
            && child_words.contains(" of 300 hours, for "),
        "{child_words}"
    );
    let teacher_words = civ_sim::frames::people::teaching_words(&sim, e.teacher);
    assert!(
        teacher_words.starts_with(&format!("Teaches {craft} to "))
            && teacher_words.contains(" (for "),
        "{teacher_words}"
    );
    // A second review enrols nobody else for it: the child is enrolled.
    sim.with_ctx_for_tests(|pop, ctx| pop.review_enrolment_for_tests(ctx, buyer));
    assert_eq!(sim.people().enrolments.len(), 1);
}

/// A household that could not hold the fee to send a child to learn pottery, which someone grown
/// of the settlement knows, puts a school before the gathering; kept, the schoolteacher takes the
/// child without a fee, and where work is paid the store pays their household for the hours they
/// teach (M10c slice EB, third part).
#[test]
fn a_household_that_cannot_hold_the_fee_puts_a_school_before_the_gathering_and_its_child_learns_there()
 {
    use civ_agents::ledger::{Channel, Trade};
    use civ_agents::polity::{IssueKind, LawStatus, PolicyKind};
    let mut sim = world(content(), 3);
    sim.advance_minutes(2 * 24 * 60).expect("advances");
    let catalog = &sim.rules().catalog;
    let pottery = catalog
        .technique_index("core:technique/pottery")
        .expect("pottery");
    let pot = catalog
        .goods
        .iter()
        .position(|g| g.id == "core:good/pot")
        .expect("pot") as u16;
    let coin = catalog
        .goods
        .iter()
        .position(|g| g.id == "core:good/coin")
        .expect("coin");
    let pay_work = catalog
        .policies
        .iter()
        .position(|d| d.id == "core:policy/pay_work")
        .expect("pay_work");
    let now = sim.now();
    let homes: Vec<PermanentId> = sim.people().households.iter().map(|(_, x)| x.id).collect();
    let buyer = homes
        .iter()
        .copied()
        .find(|&h| sim.people().household(h).expect("home").members.len() >= 2)
        .expect("a household of two or more");
    let settlement = sim
        .people()
        .household(buyer)
        .and_then(|x| x.settlement)
        .expect("settled");
    let child = *sim
        .people()
        .household(buyer)
        .expect("home")
        .members
        .last()
        .expect("a member");
    let (seller, potter) = homes
        .iter()
        .copied()
        .filter(|&h| h != buyer)
        .find_map(|h| {
            let x = sim.people().household(h)?;
            if x.settlement != Some(settlement) {
                return None;
            }
            let p = x.members.iter().copied().find(|&m| {
                sim.people()
                    .person(m)
                    .is_some_and(|p| p.age_years(now) >= 20.0)
            })?;
            Some((h, p))
        })
        .expect("another household of the settlement with an adult");
    let pi = sim
        .people()
        .polities
        .iter()
        .position(|p| p.holds(Some(settlement)))
        .expect("its polity");
    let members = sim.people().household(buyer).expect("home").members.clone();
    let pop = sim.people_mut_for_tests();
    for (_, p) in pop.people.iter_mut() {
        if members.contains(&p.id) {
            p.knows.retain(|k| usize::from(k.technique) != pottery);
        }
        if p.id == child {
            p.born = now.plus_minutes(-10 * 365 * 24 * 60);
        }
        if p.id == potter && !p.knows(pottery) {
            p.learn(pottery, 1_000.0, potter, 1.0, now);
        }
    }
    // The buyer holds nothing to pay a fee with, for now.
    let mut held = Vec::new();
    for (_, x) in pop.households.iter_mut() {
        if x.id == buyer {
            let empty = vec![0.0; x.stores.len()];
            held = std::mem::replace(&mut x.stores, empty);
        }
    }
    let market = pop
        .markets
        .iter_mut()
        .find(|m| m.settlement == settlement)
        .expect("a market");
    for _ in 0..2 {
        market.recent.push_back(Trade {
            at: now,
            seller,
            buyer,
            good: pot,
            units: 1.0,
            payment: 0,
            paid: 1.0,
            channel: Channel::Barter,
            from: None,
        });
    }
    let issue = |sim: &mut Sim| sim.with_ctx_for_tests(|pop, ctx| pop.untaught_for_tests(ctx, pi));
    assert!(!issue(&mut sim), "nobody untaught yet");
    // It cannot hold the fee: nobody is enrolled, and it is remembered.
    sim.with_ctx_for_tests(|pop, ctx| pop.review_enrolment_for_tests(ctx, buyer));
    assert!(sim.people().enrolments.is_empty());
    let untaught = sim.people().untaught.clone();
    assert_eq!(
        untaught
            .iter()
            .map(|u| (u.household, usize::from(u.technique)))
            .collect::<Vec<_>>(),
        vec![(buyer, pottery)]
    );
    assert!(
        issue(&mut sim),
        "a household untaught, and a potter of the settlement"
    );
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == buyer {
            x.stores = held.clone();
        }
    }
    // The record saves and loads alike.
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves = SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = persist::save(&mut sim, &saves, SaveKind::Manual, "untaught").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loaded.people().untaught, untaught);
    // Someone proposes a school, its keeper one who knows pottery.
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
            Some(l) if l.kind == PolicyKind::KeepSchool => {
                proposed = Some(l);
                break;
            }
            Some(_) => continue,
            None => break,
        }
    }
    let law = proposed.expect("a school proposed");
    assert_eq!(law.issue, IssueKind::Untaught);
    let teacher = law.holder.expect("a schoolteacher named");
    assert!(
        sim.people()
            .person(teacher)
            .is_some_and(|p| p.knows(pottery) && p.household != buyer)
    );
    let teacher_home = sim.people().person(teacher).expect("alive").household;
    // Passed, with a law paying common work and coin in the store.
    let polity = sim.people().polities[pi].id;
    let pay_id = sim.allocate_id_for_tests();
    let today = sim.now().day_index();
    {
        let pop = sim.people_mut_for_tests();
        pop.every_adult_deliberates = false;
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
        assert!(pop.strike_for_tests(coin, polity, polity, today, 100.0, 8.0, 1.0));
    }
    assert_eq!(
        sim.people().polities[pi].schoolteacher().map(|(h, _)| h),
        Some(teacher)
    );
    let coin_held = |sim: &Sim| {
        sim.people()
            .household(teacher_home)
            .map_or(0.0, |x| x.stores.get(coin).copied().unwrap_or(0.0))
    };
    let before = coin_held(&sim);
    // The school takes the child without a fee, and the record is done with.
    sim.with_ctx_for_tests(|pop, ctx| pop.review_enrolment_for_tests(ctx, buyer));
    let e = *sim.people().enrolments.first().expect("enrolled");
    assert_eq!(
        (e.teacher, e.learner, e.technique as usize, e.payer),
        (teacher, child, pottery, buyer)
    );
    assert_eq!(e.fee_kcal, 0.0);
    assert!(sim.people().untaught.is_empty());
    // The child studies there, and the store pays for the hours taught.
    sim.advance_minutes(3 * 24 * 60).expect("advances");
    let hours = sim
        .people()
        .person(child)
        .expect("alive")
        .knows
        .iter()
        .find(|k| usize::from(k.technique) == pottery)
        .map_or(0.0, |k| f64::from(k.hours));
    assert!(hours > 0.0, "the child studied pottery: {hours} h");
    assert!(
        coin_held(&sim) > before,
        "the schoolteacher's household was paid: {} after {before}",
        coin_held(&sim)
    );
    // Its office in words.
    let words = sim.with_ctx_for_tests(|pop, ctx| {
        civ_agents::polity::office_words(
            &pop.polities[pi],
            &ctx.catalog.policies,
            &|id| pop.name_of(id),
            &|_| String::new(),
            ctx.now.day_index(),
        )
    });
    assert!(
        words.iter().any(|w| w.starts_with("Schoolteacher: ")),
        "{words:?}"
    );
}

/// Someone who can write, and alone of their settlement knows laying out, writes it down over
/// days until their household holds a record of it; a reader of the household reads it and learns
/// toward laying out from the page, from its writer (M10c slice EC, first part).
#[test]
fn a_sole_knower_who_can_write_writes_a_craft_down_and_a_reader_learns_it_from_the_page() {
    let mut sim = world(content(), 3);
    sim.advance_minutes(24 * 60).expect("advances");
    let catalog = &sim.rules().catalog;
    let index = |id: &str| catalog.technique_index(id).expect(id);
    let (reckoning, reading, writing, laying_out) = (
        index("core:technique/reckoning"),
        index("core:technique/reading"),
        index("core:technique/writing"),
        index("core:technique/laying_out"),
    );
    let now = sim.now();
    // A household with two grown members: the writer and a reader.
    let (home, writer, reader) = sim
        .people()
        .households
        .iter()
        .find_map(|(_, x)| {
            let grown: Vec<PermanentId> = x
                .members
                .iter()
                .copied()
                .filter(|&m| {
                    sim.people()
                        .person(m)
                        .is_some_and(|p| p.age_years(now) >= 16.0)
                })
                .collect();
            (grown.len() >= 2).then(|| (x.id, grown[0], grown[1]))
        })
        .expect("a household of two grown");
    let settlement = sim
        .people()
        .household(home)
        .and_then(|x| x.settlement)
        .expect("settled");
    let members: Vec<PermanentId> = sim
        .people()
        .households
        .iter()
        .filter(|(_, x)| x.settlement == Some(settlement))
        .flat_map(|(_, x)| x.members.clone())
        .collect();
    let pop = sim.people_mut_for_tests();
    for (_, p) in pop.people.iter_mut() {
        if !members.contains(&p.id) {
            continue;
        }
        if p.id == writer {
            // The writer knows only letters and laying out.
            p.knows.clear();
            for t in [reckoning, reading, writing, laying_out] {
                p.come_to_know(t, KnowSource::Observer, now);
            }
        } else {
            p.knows.retain(|k| usize::from(k.technique) != laying_out);
        }
        if p.id == reader {
            p.knows
                .retain(|k| ![reckoning, reading, writing].contains(&usize::from(k.technique)));
            for t in [reckoning, reading] {
                p.come_to_know(t, KnowSource::Observer, now);
            }
        }
    }
    let letters = |sim: &mut Sim, who: PermanentId| {
        sim.with_ctx_for_tests(|pop, ctx| pop.letters_for_tests(ctx, who))
    };
    assert_eq!(letters(&mut sim, writer).0, Some(laying_out as u16));
    assert_eq!(
        letters(&mut sim, reader),
        (None, None),
        "nothing to read yet"
    );
    // They write it down, two hours a day, until the record's twenty hours are written.
    let mut saved = false;
    for _ in 0..30 {
        sim.advance_minutes(24 * 60).expect("advances");
        if !saved && !sim.people().written.is_empty() {
            // Mid-way, the writing saves and loads alike.
            let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
            let saves = SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
            let at = persist::save(&mut sim, &saves, SaveKind::Manual, "writing").expect("saves");
            let loaded = persist::load(&at.path, content()).expect("loads");
            assert_eq!(loaded.people().written, sim.people().written);
            saved = true;
        }
        if !sim.people().writings.is_empty() {
            break;
        }
    }
    assert!(saved, "the writing was under way at a midnight");
    let record = *sim.people().writings.first().expect("a record written");
    assert_eq!(
        (record.technique as usize, record.holder, record.maker),
        (laying_out, home, writer)
    );
    assert!(sim.people().written.is_empty());
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves = SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let at = persist::save(&mut sim, &saves, SaveKind::Manual, "record").expect("saves");
    let loaded = persist::load(&at.path, content()).expect("loads");
    assert_eq!(loaded.people().writings, sim.people().writings);
    // Kept, the writer would write down something else only they know, writing itself, not laying
    // out again; the reader reads it and learns from the page.
    assert_eq!(letters(&mut sim, writer).0, Some(writing as u16));
    assert_eq!(letters(&mut sim, reader).1, Some(laying_out as u16));
    let mut toward = None;
    for _ in 0..10 {
        sim.advance_minutes(24 * 60).expect("advances");
        toward = sim
            .people()
            .person(reader)
            .and_then(|p| p.know(laying_out))
            .map(|k| (k.hours, k.source));
        if toward.is_some_and(|(h, _)| h > 0.0) {
            break;
        }
    }
    let (hours, source) = toward.expect("the reader learns toward laying out");
    assert!(hours > 0.0, "{hours} hours");
    assert_eq!(source, KnowSource::Read(writer));
    // The person pages say it (wire 1.106).
    let craft = sim.rules().catalog.techniques[laying_out]
        .name
        .to_lowercase();
    let wrote = civ_sim::frames::people::letters_words(&sim, writer);
    assert!(
        wrote.starts_with(&format!("Wrote a record of {craft}, kept by ")),
        "{wrote}"
    );
    let read = civ_sim::frames::people::letters_words(&sim, reader);
    assert!(
        read.starts_with(&format!("Reading {craft} from a record "))
            && read.ends_with(" of 80 hours"),
        "{read}"
    );
}

/// A record is kept in its household's building; a writer whose household holds the settlement's
/// only record of a craft copies it for close kin living apart who hold none; a household's
/// records pass to the household it joins (M10c slice EC, second part).
#[test]
fn a_writer_copies_the_settlements_only_record_for_kin_living_apart_and_records_pass_on() {
    let mut sim = world(content(), 3);
    sim.advance_minutes(24 * 60).expect("advances");
    let catalog = &sim.rules().catalog;
    let index = |id: &str| catalog.technique_index(id).expect(id);
    let (reckoning, reading, writing, laying_out) = (
        index("core:technique/reckoning"),
        index("core:technique/reading"),
        index("core:technique/writing"),
        index("core:technique/laying_out"),
    );
    let now = sim.now();
    let homes: Vec<PermanentId> = sim.people().households.iter().map(|(_, x)| x.id).collect();
    let (home, writer) = homes
        .iter()
        .find_map(|&h| {
            let m = sim
                .people()
                .household(h)?
                .members
                .iter()
                .copied()
                .find(|&m| {
                    sim.people()
                        .person(m)
                        .is_some_and(|p| p.age_years(now) >= 16.0)
                })?;
            Some((h, m))
        })
        .expect("a grown member");
    let (kin_home, mother) = homes
        .iter()
        .filter(|&&h| h != home)
        .find_map(|&h| {
            let m = *sim.people().household(h)?.members.first()?;
            Some((h, m))
        })
        .expect("another household");
    // The writer's household has a finished hut at home.
    let hut = {
        let catalog = &content().catalog;
        let def = &catalog.buildings[catalog.building_index("core:building/hut").expect("hut")];
        let at = sim.people().household(home).expect("home").home;
        let spec = civ_agents::build::design_shape(
            def,
            &catalog.goods,
            civ_agents::build::Shape::Round { radius: 310 },
            at,
            None,
        )
        .expect("a hut");
        sim.place_building_for_tests(home, spec, civ_grammar::Stage::ALL.len() as u8)
            .expect("placed")
    };
    let pop = sim.people_mut_for_tests();
    // The writer's mother lives apart, in another household.
    pop.records.get_mut(&writer).expect("their record").mother = Some(mother);
    for (_, p) in pop.people.iter_mut() {
        if p.id == writer {
            p.knows.clear();
            for t in [reckoning, reading, writing, laying_out] {
                p.come_to_know(t, KnowSource::Observer, now);
            }
        } else {
            p.knows.retain(|k| usize::from(k.technique) != laying_out);
        }
    }
    let write = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| {
            pop.finish_write_for_tests(ctx, writer, laying_out as u16, 20.0 * 60.0)
        })
    };
    let letters =
        |sim: &mut Sim| sim.with_ctx_for_tests(|pop, ctx| pop.letters_for_tests(ctx, writer));
    write(&mut sim);
    let first = *sim.people().writings.first().expect("written");
    assert_eq!((first.holder, first.building), (home, Some(hut)));
    // Its building's page says it is kept there (wire 1.106).
    let b = sim
        .land()
        .buildings
        .iter()
        .find(|b| b.id == hut)
        .expect("the hut");
    let words = civ_sim::frames::buildings::records_words(&sim, b);
    assert!(
        words.starts_with("Records: laying out") && words.contains(", by "),
        "{words}"
    );
    // The settlement's only record of it: copied for the writer's mother's household.
    assert_eq!(letters(&mut sim).0, Some(laying_out as u16));
    write(&mut sim);
    let copy = *sim.people().writings.last().expect("copied");
    assert_eq!(
        (sim.people().writings.len(), copy.holder, copy.maker),
        (2, kin_home, writer)
    );
    let words: Vec<String> = sim
        .people()
        .chronicle
        .iter()
        .filter(|e| e.kind == civ_agents::ChronicleKind::RecordWritten)
        .map(|e| e.name.clone())
        .collect();
    assert_eq!(words.len(), 2, "{words:?}");
    assert!(words[0].contains(" wrote down laying out"), "{}", words[0]);
    assert!(
        words[1].contains(" copied the record of laying out")
            && words[1].contains(" for the household of "),
        "{}",
        words[1]
    );
    // Its kin hold one now: nothing more to copy of it.
    assert_ne!(letters(&mut sim).0, Some(laying_out as u16));
    // Saved and loaded, each record keeps where it is kept.
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves = SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let at = persist::save(&mut sim, &saves, SaveKind::Manual, "records").expect("saves");
    let loaded = persist::load(&at.path, content()).expect("loads");
    assert_eq!(loaded.people().writings, sim.people().writings);
    // The kin household joins the writer's: its record passes with it.
    sim.with_ctx_for_tests(|pop, ctx| pop.merge_household_for_tests(ctx, kin_home, home));
    assert!(sim.people().writings.iter().all(|r| r.holder == home));
}
