//! Diffusion through contact (M5b slice AR; research 11-02 §1.1, §5.5): people who visit
//! another settlement's hearth, or go to buy at a door there, see its new buildings, and their
//! household's next taste review meets those too, each once: a stranger's building is admired for
//! how well it was built and for what the household thinks of its owner, which for a stranger is
//! nothing. What was seen is let go at the review, and saves keep it until then.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::params::{BuildingDef, Taste};
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
