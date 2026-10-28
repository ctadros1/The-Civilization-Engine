//! Water mills set by choice (M10a slice DR, third part; ADR-0033 §5-6): a household whose people
//! know the water mill and ground flour by hand last year weighs the hours a wheel would save
//! against digging a headrace and building the wheel, sets one where a race along a reach near
//! home leaves the wheel a fall, digs the race as a ditch is dug, and only then grinds cheaper;
//! the count it weighs from is saved; and a household that knows no water way never weighs one.
//! A household near a neighbour's working mill carries its grain there and grinds it for a toll
//! (the fourth part), and weighs a mill of its own against that.

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

#[test]
fn a_household_grinds_at_a_neighbours_mill_for_its_toll_and_builds_none_of_its_own() {
    let mut sim = village();
    let (grain, flour, wheel) = (
        good(&sim, "grain"),
        good(&sim, "flour"),
        good(&sim, "water_wheel"),
    );
    let households = grown(&sim);
    assert!(households.len() >= 2);
    let (owner, _) = households[0];
    let (customer, carrier) = households
        .iter()
        .copied()
        .find(|&(h, _)| {
            h != owner
                && sim.people().household(h).expect("h").settlement
                    == sim.people().household(owner).expect("o").settlement
        })
        .expect("a neighbour of the same settlement");
    // The owner's mill: a ready installation (no race to dig) at its home on the reach that
    // carries most today, its wheel in the owner's store.
    let reach = (0..sim.land().reach_discharges.len())
        .max_by(|&a, &b| sim.land().reach_discharges[a].total_cmp(&sim.land().reach_discharges[b]))
        .expect("a reach") as u32;
    let home = sim.people().household(owner).expect("o").home;
    let mill = PermanentId::from_raw(9_000_002).expect("id");
    sim.land_mut_for_tests()
        .installations
        .push(civ_land::power::Installation {
            id: mill,
            reach,
            at: [f64::from(home.0), f64::from(home.1)],
            head_m: 2.0,
            race_m3s: 0.5,
            efficiency: 0.3,
            rated_kw: 3.0,
            owner,
            building: None,
        });
    let n = sim.rules().catalog.goods.len();
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        x.stores.resize(x.stores.len().max(n), 0.0);
        if x.id == owner {
            x.stores[wheel] = 1.0;
        }
        if x.id == customer {
            x.stores[grain] = 60.0;
            x.stores[flour] = 0.0;
        }
    }
    let params = sim.rules().land.clone();
    let kw = sim.land().installation_kw(&params, 0);
    assert!(kw > 0.0, "the river turns the wheel");
    // M11a, second part (ADR-0036 §2): the wheel's shaft gives no current without a dynamo, and
    // 80 % of its power, up to 10 kW, with one whole dynamo set beside it.
    assert_eq!(
        sim.with_ctx_for_tests(|pop, ctx| pop.electric_kw(ctx, owner)),
        0.0
    );
    let dynamo = sim
        .rules()
        .catalog
        .goods
        .iter()
        .position(|g| g.id == "core:good/dynamo")
        .expect("the dynamo");
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == owner {
            x.stores[dynamo] = 1.0;
        }
    }
    let current = sim.with_ctx_for_tests(|pop, ctx| pop.electric_kw(ctx, owner));
    assert!(
        (current - (0.8 * kw).min(10.0)).abs() < 1e-9,
        "{current} of {kw}"
    );
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == owner {
            x.stores[dynamo] = 0.0;
        }
    }
    // The observer sees the mill on the water layer, its power today and whose it is (wire 1.97).
    assert_ne!(civ_sim::frames::water::water_rev(&sim), 0);
    let payload = civ_sim::frames::water::water_response(&sim);
    let water = civ_schema::flatbuffers::root::<civ_schema::wire::Response>(&payload)
        .expect("a response")
        .body_as_water()
        .expect("water");
    let seen = water.mills().expect("mills");
    assert_eq!(seen.len(), 1);
    let m = seen.get(0);
    assert_eq!(
        (m.id(), m.household(), m.dug()),
        (mill.get(), owner.get(), true)
    );
    assert!((f64::from(m.kw()) - kw).abs() < 1e-3);
    let words = m.words().expect("words");
    assert!(
        words.contains("water mill") && words.contains("kW today"),
        "{words}"
    );
    // The neighbour sees the mill: a load's grinding, and hours spared.
    let facts = sim
        .with_ctx_for_tests(|pop, ctx| pop.mill_facts_for_tests(ctx, carrier))
        .expect("a mill to grind at");
    assert_eq!(facts.mill, mill);
    assert!(facts.left_h > 0.0 && facts.saves_per_hour > 0.0);
    // The owner grinds at home: it sees no neighbour's mill.
    let owner_person = households[0].1;
    assert!(
        sim.with_ctx_for_tests(|pop, ctx| pop.mill_facts_for_tests(ctx, owner_person))
            .is_none()
    );
    let stores = |sim: &Sim, h: PermanentId| {
        let x = sim.people().household(h).expect("household");
        civ_agents::population::stores_now(
            x,
            sim.now(),
            &sim.rules().people,
            &sim.rules().catalog.goods,
        )
    };
    let (c0, o0) = (stores(&sim, customer), stores(&sim, owner));
    sim.with_ctx_for_tests(|pop, ctx| pop.mill_grind_for_tests(ctx, carrier, mill, 120));
    let (c1, o1) = (stores(&sim, customer), stores(&sim, owner));
    let ground = c0[grain] - c1[grain];
    assert!(ground > 0.0, "grain was ground");
    assert!(
        ground <= sim.rules().people.household.carry_kg + 1e-9,
        "{ground} kg is no more than a load"
    );
    // The flour is the customer's but a sixteenth, which is the owner's toll.
    let made = ground * 0.98;
    assert!((c1[flour] - c0[flour] - made * 15.0 / 16.0).abs() < 1e-6);
    assert!((o1[flour] - o0[flour] - made / 16.0).abs() < 1e-6);
    // The owner's wheel wore by the hours it turned.
    assert!(o1[wheel] < o0[wheel]);
    // A customer that knows the mill and grinds 600 kg a year weighs its own against the toll
    // and, with a working mill this near, sets none.
    for t in [
        "core:technique/rotary_quern",
        "core:technique/jointed_frame",
        "core:technique/water_wheel",
    ] {
        sim.introduce_technique(carrier, t, false)
            .expect("introduced");
    }
    let today = sim.now().day_index();
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == customer {
            let made = x.flows.get(Flow::Made, flour);
            x.flows.add(Flow::Made, flour, 600.0);
            x.mill_seen = Some(MillSeen {
                day: today - 365,
                kg: made,
            });
        }
    }
    sim.with_ctx_for_tests(|pop, ctx| pop.review_mill_for_tests(ctx, customer));
    assert_eq!(sim.land().installations.len(), 1, "no second mill");
}
