//! Water mills set by choice (M10a slice DR, third part; ADR-0033 §5-6): a household whose people
//! know the water mill and ground flour by hand last year weighs the hours a wheel would save
//! against digging a headrace and building the wheel, sets one where a race along a reach near
//! home leaves the wheel a fall, digs the race as a ditch is dug, and only then grinds cheaper;
//! the count it weighs from is saved; and a household that knows no water way never weighs one.

use civ_agents::person::{Flow, MillSeen};
use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_land::earth::EarthKind;
use civ_sim::{NewWorld, Sim, persist};
use commons_persist::{SaveDir, SaveKind};
use std::{path::Path, sync::OnceLock};

const DAY: i64 = 1440;

fn content() -> &'static ContentRegistry {
    static C: OnceLock<ContentRegistry> = OnceLock::new();
    C.get_or_init(|| {
        civ_content::load(
            &civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("root"),
        )
        .registry
        .expect("content")
    })
}

/// A lived village of seed 3, forty days on.
fn village() -> Sim {
    let mut sim = Sim::create_for_tests(
        &NewWorld {
            name: "Millrace".to_owned(),
            seed: 3,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 768,
            band_size: 30,
            neighbours: Vec::new(),
            neighbours_known: true,
            regime_id: String::new(),
        },
        content(),
        [7; 16],
    )
    .expect("generates");
    sim.advance_minutes(40 * DAY).expect("advances");
    sim
}

fn good(sim: &Sim, id: &str) -> usize {
    let id = format!("core:good/{id}");
    sim.rules()
        .catalog
        .goods
        .iter()
        .position(|g| g.id == id)
        .expect("a good")
}

/// Each household with a grown member, and that member.
fn grown(sim: &Sim) -> Vec<(PermanentId, PermanentId)> {
    let now = sim.now();
    sim.people()
        .households
        .iter()
        .filter_map(|(_, h)| {
            let who = h.members.iter().copied().find(|&m| {
                sim.people()
                    .person(m)
                    .is_some_and(|p| p.age_years(now) >= 20.0)
            })?;
            Some((h.id, who))
        })
        .collect()
}

#[test]
fn a_household_that_grinds_by_hand_sets_a_mill_where_a_race_falls_and_turns_it_once_dug() {
    let mut sim = village();
    let (flour, wheel) = (good(&sim, "flour"), good(&sim, "water_wheel"));
    let households = grown(&sim);
    assert!(!households.is_empty());
    // A household that knows no water way does not weigh one, nor count for it.
    let (h0, _) = households[0];
    sim.with_ctx_for_tests(|pop, ctx| pop.review_mill_for_tests(ctx, h0));
    assert!(sim.people().household(h0).expect("h").mill_seen.is_none());
    assert!(sim.land().installations.is_empty());
    // Every household's grown member learns the water mill; each ground 600 kg of flour by hand
    // in the year since its last count.
    for &(_, who) in &households {
        for t in [
            "core:technique/rotary_quern",
            "core:technique/jointed_frame",
            "core:technique/water_wheel",
        ] {
            sim.introduce_technique(who, t, false).expect("introduced");
        }
    }
    let today = sim.now().day_index();
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        let made = x.flows.get(Flow::Made, flour);
        x.flows.add(Flow::Made, flour, 600.0);
        x.mill_seen = Some(MillSeen {
            day: today - 365,
            kg: made,
        });
    }
    let before: Vec<_> = households
        .iter()
        .map(|&(h, _)| {
            sim.with_ctx_for_tests(|pop, ctx| pop.tool_way_for_tests(ctx, h, wheel).1[flour])
        })
        .collect();
    for &(h, _) in &households {
        sim.with_ctx_for_tests(|pop, ctx| pop.review_mill_for_tests(ctx, h));
    }
    let set = sim.land().installations.clone();
    assert!(!set.is_empty(), "no household found a site for a mill");
    // One to a household, each with its race: a ditch from its intake to its wheel that waters
    // no field, an undug earthwork of the household's, and a fall of its own at the wheel.
    let params = sim.rules().land.clone();
    for (i, x) in set.iter().enumerate() {
        assert_eq!(set.iter().filter(|y| y.owner == x.owner).count(), 1);
        assert!(x.head_m >= 0.5 && x.race_m3s > 0.0 && x.valid());
        let d = sim
            .land()
            .ditches
            .iter()
            .find(|d| d.work == x.id)
            .expect("a race");
        assert_eq!((d.field, d.b, d.reach), (x.id, x.at, x.reach));
        let e = sim
            .land()
            .earthworks
            .iter()
            .find(|e| e.id == x.id)
            .expect("its earthwork");
        assert_eq!(
            (e.kind, e.household, e.done),
            (EarthKind::Ditch, x.owner, 0.0)
        );
        let home = sim.people().household(x.owner).expect("owner").home;
        for p in [d.a, d.b] {
            let far = (p[0] - f64::from(home.0)).hypot(p[1] - f64::from(home.1));
            assert!(far <= 400.0, "{far} m from home");
        }
        // Undug, it turns nothing.
        assert_eq!(sim.land().installation_kw(&params, i), 0.0);
    }
    assert!(sim.land().problems(sim.map(), 5, u64::MAX).is_empty());
    // A second review sets no second mill.
    let (owner, digger) = households
        .iter()
        .copied()
        .find(|&(h, _)| h == set[0].owner)
        .expect("the owner");
    sim.with_ctx_for_tests(|pop, ctx| pop.review_mill_for_tests(ctx, owner));
    assert_eq!(sim.land().installations.len(), set.len());
    // Its people dig the race as a ditch; dug, the wheel's site turns and grinding by it is the
    // household's cheapest way to flour.
    let facts = sim
        .with_ctx_for_tests(|pop, ctx| pop.ditch_facts_for_tests(ctx, digger))
        .expect("the race to dig");
    assert_eq!(facts.ditch, set[0].id);
    assert!(facts.saves_per_hour > 0.0);
    sim.with_ctx_for_tests(|pop, ctx| {
        pop.ditch_work_by_for_tests(ctx, digger, set[0].id, 100_000.0)
    });
    let e = sim
        .land()
        .earthworks
        .iter()
        .find(|e| e.id == set[0].id)
        .expect("earthwork");
    assert_eq!(e.done, 1.0);
    assert!(sim.land().installation_ready(0));
    let q = f64::from(sim.land().reach_discharges[set[0].reach as usize]);
    assert_eq!(sim.land().installation_kw(&params, 0) > 0.0, q > 0.0);
    assert!(
        sim.people()
            .chronicle
            .iter()
            .any(|e| e.name.contains("finished digging a headrace"))
    );
    let k = households.iter().position(|&(h, _)| h == owner).expect("k");
    let after = sim.with_ctx_for_tests(|pop, ctx| pop.tool_way_for_tests(ctx, owner, wheel).1);
    let (was, now) = (
        before[k].expect("flour had a cost"),
        after[flour].expect("flour has a cost"),
    );
    assert!(now < was, "flour at {now} h/kg against {was} by hand");
    // Saved and loaded, the counts and the mills are as they were, to the byte.
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves = SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = persist::save(&mut sim, &saves, SaveKind::Manual, "mill").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loaded.land().installations, sim.land().installations);
    assert_eq!(
        loaded.people().household(owner).expect("h").mill_seen,
        sim.people().household(owner).expect("h").mill_seen
    );
    for (x, y) in persist::encode_sections(&sim)
        .iter()
        .zip(&persist::encode_sections(&loaded))
    {
        assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
    }
}
