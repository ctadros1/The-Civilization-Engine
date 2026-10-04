//! Knowledge carried by people (M3b slice M, ADR-0008): founders arrive knowing their work and
//! children learn it at its age; work needs its technique, and someone who does not know it learns
//! by working beside someone who does; a technique is lost with its last knower there; the
//! observer can introduce one; and older saves load with what founders know.

use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

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
    Sim::create(
        &NewWorld {
            name: "Knowledge".to_owned(),
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
    assert_eq!(brought, founders.len(), "one record each");
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
