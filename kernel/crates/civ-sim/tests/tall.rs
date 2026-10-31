//! Climbing, lifts, taller frames and wind (ADR-0042). Members living above the ground climb their
//! storeys on every trip, and that time enters a home's design hours for the storeys they would
//! sleep on and a lodger's weighing of a room for the storey its floor lies on, so an upper floor
//! is taken only where it costs less than a ground floor would.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::build::{self, Shape};
use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_grammar::Stage;
use civ_sim::{NewWorld, Sim};

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

fn world(seed: u64) -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Tall".to_owned(),
            seed,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 512,
            band_size: 30,
            neighbours: Vec::new(),
            neighbours_known: false,
            regime_id: String::new(),
        },
        content(),
        [8; 16],
    )
    .expect("generates")
}

fn good(sim: &Sim, id: &str) -> usize {
    sim.rules()
        .catalog
        .goods
        .iter()
        .position(|g| g.id == id)
        .expect(id)
}

fn program(id: &str) -> &'static civ_agents::params::BuildingDef {
    let catalog = &content().catalog;
    &catalog.buildings[catalog.building_index(id).expect(id)]
}

fn members(sim: &Sim, h: PermanentId) -> usize {
    sim.people().household(h).expect("household").members.len()
}

/// Gives household `h` a finished building of `shape` and `program` centred at `at`.
fn give_home_at(
    sim: &mut Sim,
    h: PermanentId,
    program_id: &str,
    shape: Shape,
    at: (f32, f32),
) -> PermanentId {
    let spec = build::design_shape(
        program(program_id),
        &content().catalog.goods,
        shape,
        at,
        None,
    )
    .expect("a design");
    sim.place_building_for_tests(h, spec, Stage::ALL.len() as u8)
        .expect("placed")
}

/// A world's households of one settlement, by id, a day in, each with `kg` of provisions.
fn village(kg: f64) -> (Sim, Vec<PermanentId>) {
    let mut sim = world(3);
    sim.advance_minutes(DAY).expect("lives");
    let settlement = sim
        .people()
        .households
        .iter()
        .map(|(_, x)| x)
        .filter(|x| !x.members.is_empty())
        .find_map(|x| x.settlement)
        .expect("a settlement");
    let mut ids: Vec<PermanentId> = sim
        .people()
        .households
        .iter()
        .map(|(_, x)| x)
        .filter(|x| !x.members.is_empty() && x.settlement == Some(settlement))
        .map(|x| x.id)
        .collect();
    ids.sort_unstable();
    assert!(ids.len() >= 4, "{} households", ids.len());
    let provisions = good(&sim, "core:good/provisions");
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if ids.contains(&x.id) {
            x.stores[provisions] = kg;
        }
    }
    (sim, ids)
}

/// The centre of the place about `at` that people go to, metres, and its id.
fn place_about(sim: &Sim, at: (f32, f32)) -> ((f32, f32), u32) {
    let (w, cs) = (sim.map().width, sim.map().cell_size_m);
    let cell = (at.1 / cs) as u32 * w + (at.0 / cs) as u32;
    let place = civ_agents::travel::going_place(cell, w);
    (civ_agents::travel::going_centre(place, w, cs), place)
}

/// Household `h`'s members each went `trips` times to the place about `at` from home, as of today.
fn trips_to(sim: &mut Sim, h: PermanentId, at: (f32, f32), trips: f32) {
    let (_, place) = place_about(sim, at);
    let today = sim.now().day_index();
    let people = sim.people().household(h).expect("it").members.clone();
    for m in people {
        sim.people_mut_for_tests().travel.goings.insert(
            m,
            civ_agents::travel::Goings {
                day: today,
                to: vec![(place, trips)],
            },
        );
    }
}

fn climb(sim: &mut Sim, h: PermanentId, shape: Shape) -> f64 {
    sim.with_ctx_for_tests(|pop, ctx| {
        pop.design_climb_for_tests(ctx, h, "core:building/longhouse", shape)
    })
}

#[test]
fn a_homes_design_counts_the_climb_of_members_sleeping_above_the_ground() {
    let (mut sim, ids) = village(5_000.0);
    let h = ids[0];
    let n = members(&sim, h);
    assert!(n >= 2, "{n} members");
    let home = sim.people().household(h).expect("it").home;
    let one = Shape::Bays {
        bays: 2,
        storeys: 1,
        lofts: 0,
    };
    let narrow = Shape::Bays {
        bays: 1,
        storeys: 2,
        lofts: 0,
    };
    let wide = Shape::Bays {
        bays: 8,
        storeys: 2,
        lofts: 0,
    };
    // Nobody has gone anywhere: no climbing counted.
    trips_to(&mut sim, h, home, 0.0);
    assert_eq!(climb(&mut sim, h, narrow), 0.0);
    trips_to(&mut sim, h, home, 50.0);
    // One storey: nobody climbs.
    assert_eq!(climb(&mut sim, h, one), 0.0);
    // Two storeys whose first holds them all: nobody climbs.
    assert_eq!(climb(&mut sim, h, wide), 0.0);
    // One bay of two storeys: those whose floor lies above the first storey climb on each trip.
    let def = program("core:building/longhouse");
    let cost = def
        .shapes
        .iter()
        .find(|c| c.shape == narrow)
        .expect("the narrow shape");
    let (base, per) = (6.0, 4.8);
    let storeys = civ_agents::climb::members_storeys(&cost.living_by_storey, base, per, n);
    let up = storeys.iter().filter(|&&t| t > 0).count();
    assert!(up > 0 && up < n, "{storeys:?}");
    let h50 = climb(&mut sim, h, narrow);
    assert!(h50 > 0.0);
    // In proportion to the trips and to those upstairs, at 15 s up and 8 s down a storey.
    trips_to(&mut sim, h, home, 100.0);
    let h100 = climb(&mut sim, h, narrow);
    assert!((h100 - 2.0 * h50).abs() < 1e-9 * h100, "{h50} {h100}");
    let trips_year = h100 * 3600.0 / 23.0 * n as f64 / up as f64;
    assert!(
        (civ_agents::climb::climb_h_year(&storeys, trips_year, |t| 23.0 * t as f64) - h100).abs()
            < 1e-9
    );
}

/// A village where `a` and `b` each have a longhouse of eight bays and two storeys, 40 metres
/// either side of `lodger`'s home, `b` already letting all of its upper storey to `c`; `lodger`'s
/// members have gone 500 times each (about two trips a day) to the place about its home.
fn two_landlords() -> (Sim, [PermanentId; 4]) {
    let (mut sim, ids) = village(5_000.0);
    let (a, b, c, lodger) = (ids[0], ids[1], ids[2], ids[3]);
    // The lodger lives at the middle of the place its people go to, the two homes either side.
    let (home, _) = place_about(&sim, sim.people().household(lodger).expect("it").home);
    for (_, h) in sim.people_mut_for_tests().households.iter_mut() {
        if h.id == lodger {
            h.home = home;
        }
    }
    let tall = Shape::Bays {
        bays: 8,
        storeys: 2,
        lofts: 0,
    };
    let x = 40.0;
    let ba = give_home_at(
        &mut sim,
        a,
        "core:building/longhouse",
        tall,
        (home.0, home.1 - x),
    );
    let bb = give_home_at(
        &mut sim,
        b,
        "core:building/longhouse",
        tall,
        (home.0, home.1 + x),
    );
    trips_to(&mut sim, lodger, home, 500.0);
    // `b`'s upper storey is all let to `c`.
    let def = program("core:building/longhouse");
    let upper = def
        .shapes
        .iter()
        .find(|s| s.shape == tall)
        .expect("tall")
        .living_by_storey[1];
    let _ = ba;
    let day = sim.now().day_index();
    let provisions = good(&sim, "core:good/provisions");
    let own_home = sim.people().household(c).expect("c").home;
    sim.people_mut_for_tests()
        .letting
        .lets
        .push(civ_agents::letting::Let {
            landlord: b,
            lodger: c,
            building: bb,
            floor_m2: upper,
            good: provisions,
            rent_kg: 1.0,
            begun: day,
            due: day + 30,
            next_review: day + 30,
            unpaid: 0,
            own_home,
        });
    (sim, [a, b, c, lodger])
}

#[test]
fn a_lodger_weighs_the_climb_to_its_rooms_storey_so_upper_floors_let_for_less() {
    let (mut sim, [a, b, _, lodger]) = two_landlords();
    assert!(a < b);
    // The two rooms are alike in all but their storey: `a`'s lies on its upper storey, `b`'s on
    // its first, its upper storey let. Equally near, and asking the same, the first is taken.
    sim.with_ctx_for_tests(|pop, ctx| pop.letting_day_for_tests(ctx, &[lodger]));
    let l = sim
        .people()
        .letting
        .lodging(lodger)
        .cloned()
        .expect("it lodges");
    assert_eq!(l.landlord, b, "the room on the first storey");
    // Asking a fiftieth of a food-day less, the upper room still costs more than the first
    // storey's by its climb; asking two fifths, it is taken: an upper floor lets for less.
    let prior = sim
        .rules()
        .people
        .letting
        .asking_rent_food_days_per_person_month;
    for (asking, taken) in [(prior - 0.02, b), (prior * 0.4, a)] {
        let (mut sim, [a2, _, _, lodger]) = two_landlords();
        assert_eq!(a2, a);
        sim.people_mut_for_tests().letting.asking.insert(a, asking);
        sim.with_ctx_for_tests(|pop, ctx| pop.letting_day_for_tests(ctx, &[lodger]));
        let l = sim
            .people()
            .letting
            .lodging(lodger)
            .cloned()
            .expect("it lodges");
        assert_eq!(l.landlord, taken, "asking {asking}");
    }
}

/// The core content with the workshop's program allowing a steel skeleton up to twelve storeys
/// (ADR-0042 §3: content narrows per program; the core's workshop has one).
fn tall_content() -> &'static ContentRegistry {
    static CONTENT: OnceLock<ContentRegistry> = OnceLock::new();
    CONTENT.get_or_init(|| {
        let mut c = content().clone();
        let i = c
            .catalog
            .building_index("core:building/workshop")
            .expect("a workshop");
        if let civ_grammar::ProgramRules::Frame(r) = &mut c.catalog.buildings[i].rules {
            r.storeys = (1, civ_grammar::MAX_STOREYS_V4);
        }
        c
    })
}

/// A world of the tall content, a day in.
fn tall_world() -> Sim {
    let mut sim = Sim::create_for_tests(
        &NewWorld {
            name: "Tall".to_owned(),
            seed: 3,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 512,
            band_size: 30,
            neighbours: Vec::new(),
            neighbours_known: false,
            regime_id: String::new(),
        },
        tall_content(),
        [8; 16],
    )
    .expect("generates");
    sim.advance_minutes(DAY).expect("lives");
    sim
}

/// The workshop of two bays and `storeys` storeys its builders would design knowing the steel
/// skeleton, at `at`.
fn tall_design(storeys: u8, at: (f32, f32)) -> civ_grammar::BuildingSpec {
    let catalog = &tall_content().catalog;
    let def = &catalog.buildings[catalog
        .building_index("core:building/workshop")
        .expect("a workshop")];
    build::design_cautious(
        def,
        &catalog.goods,
        Shape::Bays {
            bays: 2,
            storeys,
            lofts: 0,
        },
        at,
        None,
        1.0,
        None,
        Some(build::FireDesign {
            knows_systems: [false, false, false, true],
            ..Default::default()
        }),
    )
    .expect("a design")
}

#[test]
fn a_steel_skeleton_of_five_to_twelve_storeys_expands_saves_and_loads() {
    let mut sim = tall_world();
    let catalog = &tall_content().catalog;
    let def = &catalog.buildings[catalog
        .building_index("core:building/workshop")
        .expect("a workshop")];
    let h = sim
        .people()
        .households
        .iter()
        .map(|(_, x)| x)
        .find(|x| !x.members.is_empty())
        .map(|x| x.id)
        .expect("a household");
    let home = sim.people().household(h).expect("it").home;
    let mut placed = Vec::new();
    for (i, storeys) in [5_u8, 8, 12].into_iter().enumerate() {
        let at = (home.0 + 30.0 * (i as f32 + 1.0), home.1 + 30.0);
        let spec = tall_design(storeys, at);
        assert_eq!(spec.version, 4, "a skeleton");
        assert_eq!(spec.storeys, storeys);
        let e = civ_grammar::expand(&spec, &def.rules).expect("it expands");
        // One floor a storey, each of its own level, the top one's code in every group above.
        let levels: Vec<civ_grammar::Level> = e
            .spaces
            .iter()
            .map(|s| s.level)
            .filter(|l| l.storey_index().is_some())
            .collect();
        assert_eq!(levels.len(), usize::from(storeys));
        let top = civ_grammar::Level::storey(usize::from(storeys) - 1).expect("a storey");
        assert_eq!(top.ordinal(), Some(["fifth", "eighth", "twelfth"][i]));
        assert!(e.groups.iter().any(
            |g| g.id >> 24 == u32::from(top.code()) && g.kind == civ_grammar::GroupKind::Posts
        ));
        // With the core's own workshop (one storey), the same design is refused.
        let core = &content().catalog;
        let core_def = &core.buildings[core.building_index("core:building/workshop").expect("it")];
        assert!(civ_grammar::expand(&spec, &core_def.rules).is_err());
        placed.push(
            sim.place_building_for_tests(h, spec, Stage::ALL.len() as u8)
                .expect("placed"),
        );
    }
    // Every group of every storey has a condition, and the buildings stand.
    for &b in &placed {
        let b = sim.land().buildings.iter().find(|x| x.id == b).expect("it");
        assert!(
            b.condition.len() > 20 && b.finished(),
            "{}",
            b.condition.len()
        );
    }
    sim.advance_minutes(DAY).expect("lives a day with them");
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = civ_sim::persist::save(&mut sim, &saves, commons_persist::SaveKind::Manual, "tall")
        .expect("saves");
    let mut loaded = civ_sim::persist::load(&saved.path, tall_content()).expect("loads");
    let same = |x: &Sim, y: &Sim, when: &str| {
        let (x, y) = (
            civ_sim::persist::encode_sections(x),
            civ_sim::persist::encode_sections(y),
        );
        assert_eq!(x.len(), y.len());
        for (p, q) in x.iter().zip(&y) {
            assert!(p.bytes == q.bytes, "section `{}` differs {when}", p.tag);
        }
    };
    same(&sim, &loaded, "on loading");
    for &b in &placed {
        let b = loaded
            .land()
            .buildings
            .iter()
            .find(|x| x.id == b)
            .expect("loaded");
        assert!(b.spec.storeys >= 5 && civ_grammar::expand(&b.spec, &def.rules).is_ok());
    }
    sim.advance_minutes(DAY).expect("lives");
    loaded.advance_minutes(DAY).expect("lives");
    same(&sim, &loaded, "a day on");
}

/// `tall_design(storeys)` as a steel skeleton however low, with posts `post_cm` across and
/// storeys `eave_cm` high.
fn frame_of(storeys: u8, post_cm: i32, eave_cm: i32, at: (f32, f32)) -> civ_grammar::BuildingSpec {
    use civ_grammar::frame_params as fp;
    let mut spec = tall_design(storeys, at);
    if spec.version < 4 {
        spec.version = 4;
        spec.params[fp::POST_SYSTEM] = 2;
        spec.materials.push("core:good/steel_section".to_owned());
    }
    spec.params[fp::POST_CM] = post_cm;
    spec.params[fp::EAVE_CM] = eave_cm;
    spec
}

fn workshop_def() -> &'static civ_agents::params::BuildingDef {
    let catalog = &tall_content().catalog;
    &catalog.buildings[catalog
        .building_index("core:building/workshop")
        .expect("a workshop")]
}

/// The states of building `b`'s posts, by storey.
fn posts_states(sim: &Sim, b: PermanentId) -> Vec<(usize, civ_land::GroupState)> {
    let b = sim.land().buildings.iter().find(|x| x.id == b).expect("it");
    b.condition
        .iter()
        .filter(|c| {
            civ_grammar::GroupKind::of_group(c.group) == Some(civ_grammar::GroupKind::Posts)
        })
        .map(|c| {
            let t = civ_grammar::Level::from_code((c.group >> 24) as u8)
                .and_then(civ_grammar::Level::storey_index)
                .expect("a storey");
            (t, c.state)
        })
        .collect()
}

/// The least wind margin of building `b`'s posts under a gust of `q_pa`, and its sway over its
/// height with the storey swaying most.
fn wind_on(sim: &Sim, b: PermanentId, q_pa: f64) -> (f64, Option<(f64, usize)>) {
    use civ_agents::structure;
    let def = workshop_def();
    let goods = &tall_content().catalog.goods;
    let b = sim.land().buildings.iter().find(|x| x.id == b).expect("it");
    let e = civ_grammar::expand(&b.spec, &def.rules).expect("expands");
    let least = e
        .groups
        .iter()
        .filter(|g| g.kind == civ_grammar::GroupKind::Posts)
        .map(|g| {
            let c = b.group(g.id).expect("its condition");
            let t = structure::group_timber(goods, def, g).expect("steel");
            structure::wind_margin(g, c, &e, &t, q_pa).strength
        })
        .fold(f64::INFINITY, f64::min);
    let drift = structure::wind_drift(&e, q_pa, |g| {
        structure::group_timber(goods, def, g).map(|t| t.stiffness_pa)
    });
    (least, drift)
}

fn storm(sim: &mut Sim, h: PermanentId, q_pa: f64) {
    sim.with_ctx_for_tests(|pop, ctx| pop.storm_for_tests(ctx, h, q_pa, 0));
}

#[test]
fn a_storm_fails_a_weak_tall_frame_sways_a_flexible_one_and_leaves_lower_frames_be() {
    use civ_agents::structure;
    use civ_land::GroupState;
    let mut sim = tall_world();
    let mut ids: Vec<PermanentId> = sim
        .people()
        .households
        .iter()
        .map(|(_, x)| x)
        .filter(|x| !x.members.is_empty())
        .map(|x| x.id)
        .collect();
    ids.sort_unstable();
    let (h1, h2) = (ids[0], ids[1]);
    let home = sim.people().household(h1).expect("it").home;
    let finished = Stage::ALL.len() as u8;
    // Eight storeys of posts 12 cm across, 3 m a storey; four of the same; and, another's, six.
    let weak = sim
        .place_building_for_tests(
            h1,
            frame_of(8, 12, 300, (home.0 + 30.0, home.1 + 30.0)),
            finished,
        )
        .expect("placed");
    let low = sim
        .place_building_for_tests(
            h1,
            frame_of(4, 12, 300, (home.0 + 60.0, home.1 + 30.0)),
            finished,
        )
        .expect("placed");
    let swaying = sim
        .place_building_for_tests(
            h2,
            frame_of(6, 12, 300, (home.0 + 90.0, home.1 + 30.0)),
            finished,
        )
        .expect("placed");
    // A calm day: each stands under its own weight.
    for h in [h1, h2] {
        storm(&mut sim, h, 0.0);
    }
    for b in [weak, low, swaying] {
        assert!(
            posts_states(&sim, b)
                .iter()
                .all(|&(_, s)| s == GroupState::Sound),
            "{:?}",
            posts_states(&sim, b)
        );
    }
    // The wind's shear on the six storeys' first: the gust's q times 1.3 on its broad face (two
    // bays of 3 m and its walls' thickness), 3 m a storey, six storeys high.
    let e = {
        let b = sim
            .land()
            .buildings
            .iter()
            .find(|x| x.id == swaying)
            .expect("it");
        civ_grammar::expand(&b.spec, &workshop_def().rules).expect("expands")
    };
    let face = {
        let o = &e.outline;
        (0..o.len())
            .map(|i| {
                let (a, b) = (o[i], o[(i + 1) % o.len()]);
                f64::from(b.0 - a.0).hypot(f64::from(b.1 - a.1)) / 100.0
            })
            .fold(0.0, f64::max)
    };
    assert!((6.0..6.5).contains(&face), "{face}");
    let shears = structure::wind_shears(&e, 400.0);
    assert_eq!(shears.len(), 6);
    assert!(
        (shears[0] - 400.0 * 1.3 * face * 3.0 * 6.0).abs() < 1e-6,
        "{shears:?}"
    );
    assert!((shears[5] - 400.0 * 1.3 * face * 3.0).abs() < 1e-6);
    // Four storeys are not weighed against the wind at all.
    assert_eq!(wind_on(&sim, low, 400.0), (f64::INFINITY, None));
    // The least gust that sways the six storeys past H/500 leaves every post of it beyond showing
    // the strain.
    let q = (1..400)
        .map(|i| f64::from(i) * 5.0)
        .find(|&q| {
            wind_on(&sim, swaying, q)
                .1
                .is_some_and(|(d, _)| d > structure::DRIFT_SHOWS)
        })
        .expect("a gust that sways it");
    let (least, drift) = wind_on(&sim, swaying, q);
    assert!(least > structure::STRAIN_SHOWS, "{least} at {q} Pa");
    let (_, most) = drift.expect("it sways");
    assert_eq!(most, 0, "the first storey sways most");
    // The eight storeys give way under that gust.
    assert!(
        wind_on(&sim, weak, q).0 < 1.0,
        "{:?}",
        wind_on(&sim, weak, q)
    );
    storm(&mut sim, h2, q);
    let states = posts_states(&sim, swaying);
    assert!(states.iter().all(|&(_, s)| s != GroupState::Failed));
    assert!(
        states
            .iter()
            .all(|&(t, s)| (s == GroupState::Symptom) == (t == most)),
        "{states:?}"
    );
    let b = sim
        .land()
        .buildings
        .iter()
        .find(|x| x.id == swaying)
        .expect("it");
    assert!(b.standing());
    let words = civ_agents::condition::symptoms(b, &workshop_def().upkeep);
    assert!(
        words.contains("the posts lean on the first storey"),
        "{words}"
    );
    // The same storm on the eight storeys and the four: the eight's posts give way, the four's
    // stand as on a calm day.
    storm(&mut sim, h1, q);
    assert!(
        posts_states(&sim, weak)
            .iter()
            .any(|&(_, s)| s == GroupState::Failed),
        "{:?}",
        posts_states(&sim, weak)
    );
    let b = sim
        .land()
        .buildings
        .iter()
        .find(|x| x.id == weak)
        .expect("it");
    assert!(!b.standing() || b.condition.iter().any(|c| c.state == GroupState::Failed));
    assert!(
        posts_states(&sim, low)
            .iter()
            .all(|&(_, s)| s == GroupState::Sound),
        "{:?}",
        posts_states(&sim, low)
    );
    // A calm day after: the sway no longer shows.
    storm(&mut sim, h2, 0.0);
    assert!(
        posts_states(&sim, swaying)
            .iter()
            .all(|&(_, s)| s == GroupState::Sound)
    );
}
