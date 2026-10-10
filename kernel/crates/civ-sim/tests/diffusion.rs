//! Diffusion through contact (M5b slice AR; research 11-02 §1.1, §5.5): people who visit
//! another settlement's hearth, or go to buy at a door there, see its new buildings, and their
//! household's next taste review meets those too, each once: a stranger's building is admired for
//! how well it was built and for what the household thinks of its owner, which for a stranger is
//! nothing. What was seen is let go at the review, and saves keep it until then.
//!
//! Step two (the M5 diffusion brief §1.2; research 07-02 §1.2, §5.4): work seen done there, or a
//! good bought there that only one technique makes, gives awareness of the technique and never
//! knowledge of it; a household that moves brings what it knows, which the settlement it comes
//! to records as brought from where it lived, and what only it knew is lost there, with where it
//! is still known among those left's kin and friends.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::history::Span;
use civ_agents::knowledge::KnowledgeEventKind;
use civ_agents::params::{BuildingDef, Taste};
use civ_agents::person::{KnowSource, Person, Step, Target};
use civ_agents::style;
use civ_agents::ties::Act;
use civ_content::ContentRegistry;
use civ_core::{PermanentId, SimTime};
use civ_grammar::hut_params;
use civ_land::Building;
use civ_sim::{NewWorld, Sim, persist};
use commons_persist::{SaveDir, SaveKind};

const DAY: i64 = 24 * 60;

/// The core content, but nothing new by chance: a building is built to its builders' taste.
fn content() -> &'static ContentRegistry {
    static CONTENT: OnceLock<ContentRegistry> = OnceLock::new();
    CONTENT.get_or_init(|| {
        let root = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
            .expect("content/ is above the crate");
        let mut c = civ_content::load(&root)
            .registry
            .expect("the core content loads");
        c.people.params.style.innovation = 0.0;
        c
    })
}

/// Settlements of `band` people each, one per entry of `neighbours` beside the first, that know
/// where each other camped; with a fixed identity, so the world lives the same life every run.
fn world(seed: u64, band: u32, neighbours: &[u32]) -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Diffusion".to_owned(),
            seed,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 768,
            band_size: band,
            neighbours: neighbours.to_vec(),
            neighbours_known: true,
            regime_id: String::new(),
        },
        content(),
        [7; 16],
    )
    .expect("generates")
}

fn def<'a>(sim: &'a Sim, b: &Building) -> &'a BuildingDef {
    let catalog = &sim.rules().catalog;
    &catalog.buildings[catalog
        .building_index(&b.spec.program)
        .expect("its program")]
}

fn taste(sim: &Sim, household: PermanentId) -> Taste {
    sim.people()
        .household(household)
        .expect("the household")
        .taste
}

/// The households of settlement `s` with anyone in them, in id order.
fn households_of(sim: &Sim, s: PermanentId) -> Vec<PermanentId> {
    let mut found: Vec<_> = sim
        .people()
        .households
        .iter()
        .filter(|(_, x)| x.settlement == Some(s) && !x.members.is_empty())
        .map(|(_, x)| x.id)
        .collect();
    found.sort();
    found
}

fn members(sim: &Sim, h: PermanentId) -> Vec<PermanentId> {
    let mut m = sim.people().household(h).expect("it").members.clone();
    m.sort();
    m
}

/// Where person `p` lives.
fn home_of(sim: &Sim, p: PermanentId) -> Option<PermanentId> {
    let pop = sim.people();
    pop.person(p)
        .and_then(|q| pop.household(q.household))
        .and_then(|h| h.settlement)
}

/// Lives `sim` until every household has a finished home, half a year at most.
fn until_housed(sim: &mut Sim) {
    for _ in 0..180 {
        sim.advance_minutes(DAY).expect("advances");
        let all = sim.people().households.iter().all(|(_, h)| {
            h.members.is_empty()
                || sim
                    .land()
                    .buildings
                    .iter()
                    .any(|b| b.household == h.id && b.finished())
        });
        if all {
            return;
        }
    }
    panic!("not every household was housed in half a year");
}

fn review(sim: &mut Sim) {
    let rules = sim.rules().clone();
    let (land, now) = (sim.land().clone(), sim.now());
    sim.people_mut_for_tests().review_tastes(
        &rules.catalog,
        &rules.people,
        &rules.land,
        &land,
        now,
    );
}

/// Puts every household's taste and what it admired back as they were.
fn restore(sim: &mut Sim, saved: &[(PermanentId, Taste, Option<PermanentId>)]) {
    for (_, h) in sim.people_mut_for_tests().households.iter_mut() {
        if let Some(&(_, t, a)) = saved.iter().find(|x| x.0 == h.id) {
            (h.taste, h.admired) = (t, a);
        }
    }
}

fn close(a: &Taste, b: &Taste) -> bool {
    a.traits()
        .iter()
        .zip(b.traits())
        .all(|(x, y)| (x - y).abs() < 1e-3)
}

fn distance(a: &Taste, b: &Taste) -> f64 {
    a.traits()
        .iter()
        .zip(b.traits())
        .map(|(x, y)| f64::from(x - y).powi(2))
        .sum::<f64>()
        .sqrt()
}

#[test]
fn a_building_seen_in_another_settlement_moves_taste_as_a_stranger_s_or_a_friend_s() {
    let mut sim = world(3, 30, &[30]);
    until_housed(&mut sim);
    let (a, b) = (sim.land().settlements[0].id, sim.land().settlements[1].id);
    let now = sim.now();
    // One home of the second settlement, finished a month ago steep and tall; every other building
    // was finished long since, so it alone is new.
    let owner = households_of(&sim, b)[0];
    let month_ago = SimTime::from_minutes(now.minutes() - 30 * DAY);
    let long_ago = SimTime::from_minutes(now.minutes() - 400 * DAY);
    let mut seen = None;
    for x in sim.land_mut_for_tests().buildings.iter_mut() {
        if x.household == owner && x.finished() && seen.is_none() {
            x.spec.params[hut_params::PITCH_CENTIDEG] = 5_500;
            x.spec.params[hut_params::EAVE_CM] = 240;
            x.stage_since = month_ago;
            seen = Some(x.id);
        } else {
            x.stage_since = long_ago;
        }
    }
    let seen = seen.expect("the owner's home");
    let model = {
        let x = sim
            .land()
            .buildings
            .iter()
            .find(|x| x.id == seen)
            .expect("it");
        style::traits_of(&x.spec, def(&sim, x))
    };
    let visitors = households_of(&sim, a);
    let (x, y) = (visitors[0], visitors[1]);
    let saved: Vec<_> = sim
        .people()
        .households
        .iter()
        .map(|(_, h)| (h.id, h.taste, h.admired))
        .collect();
    let rules = sim.rules().people.clone();
    let most = rules.style.prestige_most.max(1.0);
    let was = taste(&sim, x);

    // Someone of the first settlement's first household saw it: a stranger's building, the only
    // new one of its settlement's year, so the best built, is admired half as much as could be.
    let one = members(&sim, x)[0];
    sim.people_mut_for_tests().seen_away.insert(one, vec![seen]);
    review(&mut sim);
    let stranger = taste(&sim, x);
    let expected = style::moved(
        &was,
        &model,
        rules.style.alpha * style::prestige(&rules.style, 0.5) / most,
    );
    assert!(
        close(&stranger, &expected),
        "{stranger:?} against {expected:?}"
    );
    assert_eq!(sim.people().household(x).expect("it").admired, Some(seen));
    // Nobody else of the first settlement saw it, and their settlement built nothing new.
    assert_eq!(
        taste(&sim, y),
        saved.iter().find(|s| s.0 == y).expect("y").1
    );
    // What was seen has been met.
    assert!(sim.people().seen_away.is_empty());
    // The observer says where the building admired stands, and where one built after it does.
    let there = sim.land().settlements[1].name.clone();
    let words = civ_sim::frames::people::taste_words(
        &sim,
        sim.people().household(x).expect("the household"),
    );
    assert!(words.ends_with(&format!(" at {there}")), "{words}");
    let mut home = sim
        .land()
        .buildings
        .iter()
        .find(|b| b.household == x && b.finished())
        .cloned()
        .expect("its home");
    home.style_from = Some(seen);
    let words = civ_sim::frames::buildings::style_words(&sim, &home, Some(def(&sim, &home)));
    assert!(words.ends_with(&format!(" at {there}")), "{words}");
    home.style_from = sim
        .land()
        .buildings
        .iter()
        .find(|b| b.household == y && b.finished())
        .map(|b| b.id);
    let words = civ_sim::frames::buildings::style_words(&sim, &home, Some(def(&sim, &home)));
    assert!(!words.contains(" at "), "{words}");

    // Seen by everyone of the household, it is still met once.
    restore(&mut sim, &saved);
    for m in members(&sim, x) {
        sim.people_mut_for_tests().seen_away.insert(m, vec![seen]);
    }
    review(&mut sim);
    assert!(close(&taste(&sim, x), &stranger), "met more than once");

    // Someone of the household esteems someone of the owner's: the patron's half of the
    // admiration is that esteem, e / (e + 1).
    restore(&mut sim, &saved);
    let friend = members(&sim, owner)[0];
    let day = sim.now().day_index();
    let ties = rules.ties.clone();
    sim.people_mut_for_tests()
        .ties
        .record(one, friend, Act::GiftReceived, 4.0, 0.0, day, &ties);
    let r: f64 = {
        let t = sim
            .people()
            .ties
            .tie(one, friend, day, &ties)
            .expect("a tie");
        civ_agents::ties::Domain::ALL
            .iter()
            .map(|&d| t.esteem(d, &ties))
            .sum()
    };
    assert!(r > 0.0, "a tie of esteem");
    sim.people_mut_for_tests().seen_away.insert(one, vec![seen]);
    review(&mut sim);
    let befriended = taste(&sim, x);
    let rank = (r / (r + 1.0) + 1.0) / 2.0;
    let expected = style::moved(
        &was,
        &model,
        rules.style.alpha * style::prestige(&rules.style, rank) / most,
    );
    assert!(
        close(&befriended, &expected),
        "{befriended:?} against {expected:?}"
    );
    assert!(distance(&befriended, &model) < distance(&stranger, &model));

    // A building seen that was finished over a year before the review is not met.
    restore(&mut sim, &saved);
    for x in sim.land_mut_for_tests().buildings.iter_mut() {
        if x.id == seen {
            x.stage_since = long_ago;
        }
    }
    sim.people_mut_for_tests().seen_away.insert(one, vec![seen]);
    review(&mut sim);
    assert_eq!(taste(&sim, x), was);
    assert!(sim.people().seen_away.is_empty());
}

#[test]
fn a_visitor_sees_the_new_buildings_about_the_hearth_and_saves_keep_them() {
    // A woman of the first settlement is made the mother of an adult of the second, as if they had
    // married away: they visit, and see the homes newly built about each other's hearth.
    let mut sim = world(3, 30, &[30]);
    let (a, b) = (sim.land().settlements[0].id, sim.land().settlements[1].id);
    until_housed(&mut sim);
    let adult = |sim: &Sim, s: PermanentId, female: bool| {
        let pop = sim.people();
        let mut found: Vec<_> = pop
            .people
            .iter()
            .filter(|(_, p)| {
                p.age_years(sim.now()) >= 16.0
                    && (p.sex == civ_agents::Sex::Female) == female
                    && home_of(sim, p.id) == Some(s)
            })
            .map(|(_, p)| p.id)
            .collect();
        found.sort();
        found[0]
    };
    let (mother, son) = (adult(&sim, a, true), adult(&sim, b, false));
    sim.people_mut_for_tests()
        .records
        .get_mut(&son)
        .expect("on record")
        .mother = Some(mother);
    let mut found = false;
    for _ in 0..24 {
        sim.advance_minutes(5 * DAY).expect("advances");
        if !sim.people().seen_away.is_empty() {
            found = true;
            break;
        }
    }
    assert!(found, "a visitor saw a new building elsewhere");
    let (sight_m, seen_most) = {
        let s = &sim.rules().people.style;
        (s.sight_m as f32, s.seen_most)
    };
    let pop = sim.people();
    let year_ago = sim.now().minutes() - 360 * DAY;
    for (&p, list) in &pop.seen_away {
        assert!(!list.is_empty() && list.len() <= seen_most);
        let home = home_of(&sim, p).expect("they live somewhere");
        for id in list {
            let x = sim
                .land()
                .buildings
                .iter()
                .find(|x| x.id == *id)
                .expect("a building");
            let there = pop
                .household(x.household)
                .and_then(|h| h.settlement)
                .expect("of a settlement");
            assert_ne!(there, home, "{p} noted a building of their own settlement");
            assert!(x.finished(), "an unfinished building was seen");
            assert!(x.stage_since.minutes() >= year_ago - 150 * DAY);
            // Seen from the hearth on a visit, or from a door on a trip to buy.
            let hearth = sim
                .land()
                .settlements
                .iter()
                .find(|s| s.id == there)
                .expect("it")
                .hearth_m;
            let c = civ_agents::build::centre_m(&x.spec);
            let near_hearth = (c.0 - hearth.0).hypot(c.1 - hearth.1) <= sight_m;
            assert!(
                near_hearth
                    || pop
                        .reports
                        .held
                        .values()
                        .flatten()
                        .any(|r| r.market == there),
                "{p} saw a building out of sight"
            );
        }
    }
    assert!(pop.problems(u64::MAX, usize::MAX).is_empty());
    // A save keeps what was seen, until the review.
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        SaveDir::create(dir.path().join("saves"), civ_schema::SAVE_EXTENSION).expect("save dir");
    let saved = persist::save(&mut sim, &saves, SaveKind::Manual, "seen").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loaded.people().seen_away, sim.people().seen_away);
}

#[test]
fn in_a_world_of_one_settlement_nobody_sees_a_building_elsewhere() {
    let mut sim = world(3, 30, &[]);
    until_housed(&mut sim);
    sim.advance_minutes(30 * DAY).expect("advances");
    assert!(sim.people().seen_away.is_empty());
}

#[test]
fn what_someone_saw_elsewhere_goes_with_them_when_they_die_or_are_exiled() {
    // Found by the M5b demo: someone who died between seeing and the review left their sights
    // behind, and the save of that year-end would not load.
    let mut sim = world(3, 30, &[30]);
    until_housed(&mut sim);
    let owner = households_of(&sim, sim.land().settlements[1].id)[0];
    let seen = sim
        .land()
        .buildings
        .iter()
        .find(|x| x.household == owner && x.finished())
        .expect("a building of the second settlement")
        .id;
    let a = sim.land().settlements[0].id;
    let visitors = households_of(&sim, a);
    let (dead, exiled, stays) = (
        members(&sim, visitors[0])[0],
        members(&sim, visitors[1])[0],
        members(&sim, visitors[2])[0],
    );
    for p in [dead, exiled, stays] {
        sim.people_mut_for_tests().seen_away.insert(p, vec![seen]);
    }
    sim.die_for_tests(dead);
    sim.exile_for_tests(exiled);
    let pop = sim.people();
    assert!(pop.person(dead).is_none() && pop.person(exiled).is_none());
    assert_eq!(pop.seen_away.keys().copied().collect::<Vec<_>>(), [stays]);
    assert!(pop.problems(u64::MAX, usize::MAX).is_empty());
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        SaveDir::create(dir.path().join("saves"), civ_schema::SAVE_EXTENSION).expect("save dir");
    let saved = persist::save(&mut sim, &saves, SaveKind::Manual, "gone").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loaded.people().seen_away, sim.people().seen_away);
}

fn person(sim: &mut Sim, id: PermanentId) -> &mut Person {
    sim.people_mut_for_tests()
        .people
        .iter_mut()
        .map(|(_, p)| p)
        .find(|p| p.id == id)
        .expect("the person")
}

fn technique(sim: &Sim, id: &str) -> usize {
    sim.rules().catalog.technique_index(id).expect(id)
}

fn good(sim: &Sim, id: &str) -> usize {
    sim.rules().catalog.good_index(id).expect(id)
}

fn hearth(sim: &Sim, s: PermanentId) -> (f32, f32) {
    sim.land()
        .settlements
        .iter()
        .find(|x| x.id == s)
        .expect("the settlement")
        .hearth_m
}

/// `id` stands at `at` doing the work of activity `def` (step: working, not walking).
fn at_work(sim: &mut Sim, id: PermanentId, def: usize, at: (f32, f32), working: bool) {
    let now = sim.now();
    let p = person(sim, id);
    p.trip = None;
    p.pos = at;
    p.act.def = def as u16;
    p.act.target = Target::None;
    p.act.steps = vec![if working {
        Step::Work { minutes: 120 }
    } else {
        Step::Walk { to: at }
    }];
    p.act.step = 0;
    p.act.started = now;
    p.act.step_started = now;
}

#[test]
fn work_seen_or_a_good_bought_elsewhere_gives_awareness_and_never_knowledge() {
    let mut sim = world(3, 30, &[30]);
    sim.advance_minutes(DAY).expect("advances");
    let (a, b) = (sim.land().settlements[0].id, sim.land().settlements[1].id);
    let (drying, rotary, quern) = (
        technique(&sim, "core:technique/drying"),
        technique(&sim, "core:technique/rotary_quern"),
        technique(&sim, "core:technique/quern_grinding"),
    );
    let dry_meat = {
        let catalog = &sim.rules().catalog;
        let r = catalog
            .recipes
            .iter()
            .position(|r| r.id == "core:recipe/dry_meat")
            .expect("the recipe");
        catalog
            .activities
            .iter()
            .position(|x| x.recipe == Some(r))
            .expect("its activity")
    };
    let here = hearth(&sim, b);
    // Someone of the second settlement dries meat by its hearth; nobody of the first knows how.
    let worker = members(&sim, households_of(&sim, b)[0])[0];
    let now = sim.now();
    person(&mut sim, worker).come_to_know(drying, KnowSource::Found, now);
    at_work(&mut sim, worker, dry_meat, (here.0 + 30.0, here.1), true);
    let firsts: Vec<PermanentId> = households_of(&sim, a)
        .into_iter()
        .take(4)
        .map(|h| members(&sim, h)[0])
        .collect();
    let (near, far, buyer, flour) = (firsts[0], firsts[1], firsts[2], firsts[3]);
    let watch_m = sim.rules().people.knowledge.watch_m as f32;
    let (near_at, far_at) = (here, (here.0 + 30.0 + watch_m + 20.0, here.1));
    assert!(watch_m > 30.0);
    sim.with_ctx_for_tests(|pop, ctx| {
        pop.watch_work_for_tests(ctx, near, b, near_at);
        pop.watch_work_for_tests(ctx, far, b, far_at);
    });
    let know = |sim: &Sim, p: PermanentId, t: usize| {
        sim.people().person(p).and_then(|q| q.know(t)).copied()
    };
    // One within sight of the work knows of it, seen at the second settlement; not how to do it.
    let k = know(&sim, near, drying).expect("aware of drying");
    assert!(!k.known && k.hours == 0.0, "{k:?}");
    assert_eq!(k.source, KnowSource::Seen(b));
    assert!(!sim.people().person(near).expect("them").knows(drying));
    // One farther off saw nothing of it.
    assert!(know(&sim, far, drying).is_none());
    // Nor does anyone of the second settlement learn of it by watching at home, nor from one
    // walking rather than working.
    let neighbour = members(&sim, households_of(&sim, b)[1])[0];
    at_work(&mut sim, worker, dry_meat, (here.0 + 30.0, here.1), false);
    sim.with_ctx_for_tests(|pop, ctx| {
        pop.watch_work_for_tests(ctx, neighbour, b, near_at);
        pop.watch_work_for_tests(ctx, far, b, near_at);
    });
    assert!(know(&sim, neighbour, drying).is_none());
    assert!(know(&sim, far, drying).is_none());

    // Dried meat bought there shows drying, the only way it is made; a rotary quern shows its
    // craft. Flour shows nothing: it is ground at either quern.
    let (dried, rquern, meal) = (
        good(&sim, "core:good/dried_meat"),
        good(&sim, "core:good/rotary_quern"),
        good(&sim, "core:good/flour"),
    );
    // The one buying flour has never ground at either quern.
    person(&mut sim, flour)
        .knows
        .retain(|k| ![quern, rotary].contains(&usize::from(k.technique)));
    let before = know(&sim, near, drying);
    sim.with_ctx_for_tests(|pop, ctx| {
        pop.bought_for_tests(ctx, buyer, b, dried);
        pop.bought_for_tests(ctx, buyer, b, rquern);
        pop.bought_for_tests(ctx, flour, b, meal);
        pop.bought_for_tests(ctx, near, b, dried);
    });
    assert_eq!(
        know(&sim, buyer, drying).map(|k| (k.known, k.source)),
        Some((false, KnowSource::Seen(b)))
    );
    assert_eq!(
        know(&sim, buyer, rotary).map(|k| (k.known, k.source)),
        Some((false, KnowSource::Seen(b)))
    );
    assert!(know(&sim, flour, rotary).is_none());
    assert!(know(&sim, flour, quern).is_none());
    // What they knew of already is as it was.
    assert_eq!(know(&sim, near, drying), before);

    // The inspector says where they saw it, and a save keeps it.
    let words = civ_sim::frames::knowledge::source_words(&sim, KnowSource::Seen(b));
    let name = sim.land().settlements[1].name.clone();
    assert_eq!(words, format!("saw it at {name}"));
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        SaveDir::create(dir.path().join("saves"), civ_schema::SAVE_EXTENSION).expect("save dir");
    let saved = persist::save(&mut sim, &saves, SaveKind::Manual, "seen").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    for p in [near, buyer] {
        assert_eq!(
            loaded.people().person(p).expect("them").knows,
            sim.people().person(p).expect("them").knows
        );
    }
}

/// Text of a chronicle entry, links as their names.
fn text(sim: &Sim, e: &civ_agents::history::ChronicleEvent) -> String {
    let name = |id: PermanentId| {
        sim.people()
            .records
            .get(&id)
            .map_or_else(|| "someone".to_owned(), |r| r.given.clone())
    };
    civ_agents::history::render(e, &name)
        .into_iter()
        .map(|s| match s {
            Span::Text(t) | Span::Person(_, t) | Span::Settlement(_, t) | Span::Firm(_, t) => t,
        })
        .collect()
}

#[test]
fn a_household_that_moves_brings_what_it_knows_and_its_loss_says_where_it_is_still_known() {
    let mut sim = world(3, 30, &[30]);
    sim.advance_minutes(DAY).expect("advances");
    let (a, b) = (sim.land().settlements[0].id, sim.land().settlements[1].id);
    let drying = technique(&sim, "core:technique/drying");
    // One household of the first settlement alone knows drying there, as its record says.
    let movers = households_of(&sim, a)[0];
    let people = members(&sim, movers);
    let now = sim.now();
    for &m in &people {
        person(&mut sim, m).come_to_know(drying, KnowSource::Found, now);
    }
    sim.people_mut_for_tests()
        .knowledge
        .push(civ_agents::knowledge::KnowledgeEvent {
            at: now,
            settlement: a,
            technique: drying as u16,
            person: people[0],
            kind: KnowledgeEventKind::Known(KnowSource::Found),
            elsewhere: None,
        });
    // Someone who stays behind thinks well of one of them.
    let stays = members(&sim, households_of(&sim, a)[1])[0];
    let day = now.day_index();
    let ties = sim.rules().people.ties.clone();
    sim.people_mut_for_tests().ties.record(
        stays,
        people[0],
        Act::GiftReceived,
        2.0,
        0.0,
        day,
        &ties,
    );
    let entries = sim.people().chronicle.len();
    sim.with_ctx_for_tests(|pop, ctx| pop.relocate_for_tests(ctx, movers, b));
    let record: Vec<_> = sim
        .people()
        .knowledge
        .iter()
        .filter(|e| usize::from(e.technique) == drying)
        .cloned()
        .collect();
    // Lost where they lived, still known where one left behind has a friend; brought to where
    // they went, from where they lived.
    let lost = record
        .iter()
        .find(|e| e.settlement == a && e.kind == KnowledgeEventKind::Lost)
        .expect("lost where they lived");
    assert_eq!(lost.elsewhere, Some(b));
    let brought = record
        .iter()
        .find(|e| e.settlement == b && matches!(e.kind, KnowledgeEventKind::Known(_)))
        .expect("known where they went");
    assert_eq!(brought.elsewhere, Some(a));
    assert!(people.contains(&brought.person));
    // The chronicle says so; techniques everyone knows were neither lost nor brought.
    let words: Vec<String> = sim.people().chronicle[entries..]
        .iter()
        .filter(|e| e.kind == civ_agents::history::ChronicleKind::TechniqueLost)
        .map(|e| text(&sim, e))
        .collect();
    assert_eq!(words.len(), 1, "{words:?}");
    assert!(
        words[0].contains("still known where some here have kin or friends"),
        "{}",
        words[0]
    );
    let new_known = sim
        .people()
        .knowledge
        .iter()
        .filter(|e| e.settlement == b && e.elsewhere == Some(a))
        .count();
    assert_eq!(new_known, 1, "only drying was new to the second settlement");
    // A save keeps the record as it is.
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        SaveDir::create(dir.path().join("saves"), civ_schema::SAVE_EXTENSION).expect("save dir");
    let saved = persist::save(&mut sim, &saves, SaveKind::Manual, "moved").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loaded.people().knowledge, sim.people().knowledge);
    assert!(sim.people().problems(u64::MAX, usize::MAX).is_empty());
}

#[test]
fn each_settlement_s_way_of_building_is_shown_on_three_clocks_and_chains_name_where_they_cross() {
    let mut sim = world(3, 30, &[30]);
    until_housed(&mut sim);
    let (a, b) = (sim.land().settlements[0].id, sim.land().settlements[1].id);
    // Each founding band's way is its drawn one, and a save keeps it.
    let seed = sim.meta().seed;
    for s in [a, b] {
        let drawn = style::band_way(&sim.rules().people.style, seed, s);
        assert_eq!(sim.people().founding_ways.get(&s), Some(&drawn));
    }
    // The clocks: every household's taste, and every finished building, new this first year.
    let (catalog, land, now) = (sim.rules().catalog.clone(), sim.land().clone(), sim.now());
    let clocks = sim.people().style_clocks(&catalog, &land, now, a);
    assert_eq!(clocks.taste.n, households_of(&sim, a).len());
    let homes: Vec<PermanentId> = land
        .buildings
        .iter()
        .filter(|x| x.finished() && sim.people().building_settlement(&land, x) == Some(a))
        .map(|x| x.id)
        .collect();
    assert!(homes.len() >= 2);
    assert_eq!((clocks.stock.n, clocks.new.n), (homes.len(), homes.len()));
    assert_eq!(clocks.new_after_elsewhere, 0);
    let way = clocks.way.expect("its founding way");
    assert!((clocks.taste.mean[0] - f64::from(way.pitch_centideg)).abs() < 300.0);
    // A home here that followed one here that followed one there: its chain crosses.
    let there = land
        .buildings
        .iter()
        .find(|x| x.finished() && sim.people().building_settlement(&land, x) == Some(b))
        .map(|x| x.id)
        .expect("a home there");
    for x in sim.land_mut_for_tests().buildings.iter_mut() {
        if x.id == homes[1] {
            x.style_from = Some(there);
        } else if x.id == homes[0] {
            x.style_from = Some(homes[1]);
        }
    }
    let land = sim.land().clone();
    let first = land
        .buildings
        .iter()
        .find(|x| x.id == homes[0])
        .expect("it");
    assert_eq!(sim.people().crossing_of(&land, first), Some(there));
    let clocks = sim.people().style_clocks(&catalog, &land, now, a);
    assert_eq!(clocks.new_after_elsewhere, 2);
    // The observer's words: the settlement's clocks, the chain back to where it crossed, and what
    // someone saw elsewhere.
    let name = sim.land().settlements[1].name.clone();
    let words = civ_sim::frames::people::style_clock_words(&sim, a);
    assert!(words.starts_with("founded to build "), "{words}");
    assert!(words.contains(" households would build "), "{words}");
    assert!(words.contains(", 2 after a building elsewhere"), "{words}");
    assert!(words.contains(" standing buildings: "), "{words}");
    let readout = civ_sim::frames::buildings::style_words(&sim, first, Some(def(&sim, first)));
    assert!(
        readout.contains(", which goes back to ") && readout.ends_with(&format!(" at {name}")),
        "{readout}"
    );
    let viewer = members(&sim, households_of(&sim, a)[0])[0];
    sim.people_mut_for_tests()
        .seen_away
        .insert(viewer, vec![there]);
    let seen = civ_sim::frames::people::seen_away_words(&sim, viewer);
    assert!(seen.starts_with(&format!("At {name}: ")), "{seen}");
    // A save keeps the founding ways.
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        SaveDir::create(dir.path().join("saves"), civ_schema::SAVE_EXTENSION).expect("save dir");
    let saved = persist::save(&mut sim, &saves, SaveKind::Manual, "ways").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loaded.people().founding_ways, sim.people().founding_ways);
}
