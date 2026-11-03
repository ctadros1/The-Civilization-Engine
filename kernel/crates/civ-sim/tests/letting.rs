//! Letting rooms (ADR-0040): a household with no roof of its own lodges with a neighbour that has
//! spare sleeping floor when a month's rent costs it less than being roofless does, pays monthly
//! on the `Rent` channel with goods conserved, counts as sheltered with its goods kept under the
//! landlord's roof and its home there, and leaves once it has a roof of its own, or after a second
//! month it cannot pay. Asking rents move by a step with offers turned away and with vacancy; a
//! home's design counts the rent its spare floor would bring only where lets have been seen; lets
//! save and load exactly; and where nobody is roofless, letting changes nothing.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::build::{self, Shape};
use civ_agents::ledger::Channel;
use civ_agents::population;
use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_grammar::Stage;
use civ_sim::{NewWorld, Sim, persist};

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
            name: "Letting".to_owned(),
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

/// Gives household `h` a finished building of `shape` and `program` at its home.
fn give_home(sim: &mut Sim, h: PermanentId, program_id: &str, shape: Shape) -> PermanentId {
    let at = sim.people().household(h).expect("household").home;
    give_home_at(sim, h, program_id, shape, at)
}

/// Gives household `h` a finished building of `shape` and `program` at `at`.
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
    assert!(ids.len() >= 3, "{} households", ids.len());
    let provisions = good(&sim, "core:good/provisions");
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if ids.contains(&x.id) {
            x.stores[provisions] = kg;
        }
    }
    (sim, ids)
}

/// A village where its first household has a large finished home and the second none, and lets
/// it seek a room today: the village, the landlord and the lodger.
fn lodged() -> (Sim, PermanentId, PermanentId) {
    let (mut sim, ids) = village(5_000.0);
    let (landlord, lodger) = (ids[0], ids[1]);
    let big = Shape::Bays {
        bays: 8,
        storeys: 2,
        lofts: 0,
    };
    give_home(&mut sim, landlord, "core:building/longhouse", big);
    assert!(sim.people().household(landlord).expect("it").sheltered);
    assert!(!sim.people().household(lodger).expect("it").sheltered);
    sim.with_ctx_for_tests(|pop, ctx| pop.letting_day_for_tests(ctx, &[lodger]));
    (sim, landlord, lodger)
}

fn food_kcal(sim: &Sim, channel: Channel) -> f64 {
    let goods = &sim.rules().catalog.goods;
    let t = &sim.people().transfers;
    goods
        .iter()
        .enumerate()
        .map(|(g, d)| t.get(channel, g) * d.kcal_per_kg)
        .sum()
}

#[test]
fn a_roofless_household_lodges_pays_monthly_and_leaves_when_its_own_roof_is_on() {
    let (mut sim, landlord, lodger) = lodged();
    let l = sim
        .people()
        .letting
        .lodging(lodger)
        .cloned()
        .expect("it lodges");
    assert_eq!(l.landlord, landlord);
    let per = 4.8;
    assert!((l.floor_m2 - per * members(&sim, lodger) as f64).abs() < 1e-9);
    // A day of its people's food a person a month, the first month paid as it began.
    let daily = sim.rules().people.household.daily_kcal_per_person;
    let rent_kcal = members(&sim, lodger) as f64 * daily;
    let kcal_per_kg = sim.rules().catalog.goods[l.good].kcal_per_kg;
    assert!((l.rent_kg * kcal_per_kg - rent_kcal).abs() < 1e-6);
    assert!((food_kcal(&sim, Channel::Rent) - rent_kcal).abs() < 1e-3);
    // It counts as sheltered, its goods kept under the landlord's roof, and it lives there.
    let building = sim
        .land()
        .buildings
        .iter()
        .find(|b| b.id == l.building)
        .expect("the home")
        .clone();
    let x = sim.people().household(lodger).expect("lodger");
    assert!(x.sheltered);
    assert!(
        x.keeping.roofed_kg + x.keeping.raised_kg > 100.0,
        "{:?}",
        x.keeping
    );
    assert_eq!(x.home, build::centre_m(&building.spec));
    let words = civ_sim::frames::people::lodging_words(&sim, lodger);
    assert_eq!(words.len(), 1);
    assert!(words[0].starts_with("lodges with ") && words[0].contains(" kg of food a month"));
    let words = civ_sim::frames::people::lodging_words(&sim, landlord);
    assert!(words[0].starts_with("lets ") && words[0].contains(" m² to "));
    let status = civ_sim::frames::buildings::status(&sim, &building);
    assert!(status.starts_with("finished; lets "), "{status}");
    // A month on it pays again, on `Rent`, goods conserved.
    let held_before = sim.people().goods_held();
    let flows_before = sim.people().flows();
    let day = sim.now().day_index();
    sim.people_mut_for_tests().letting.lets[0].due = day;
    sim.with_ctx_for_tests(|pop, ctx| pop.letting_day_for_tests(ctx, &[]));
    assert!((food_kcal(&sim, Channel::Rent) - 2.0 * rent_kcal).abs() < 1e-3);
    assert_eq!(sim.people().letting.lets[0].unpaid, 0);
    let gaps = population::unaccounted(
        sim.rules().catalog.goods.len(),
        (&held_before, &flows_before),
        (&sim.people().goods_held(), &sim.people().flows()),
    );
    assert!(gaps.is_empty(), "unaccounted for: {gaps:?}");
    // Its own roof on, it leaves at the let's review and lives there.
    let hut = give_home_at(
        &mut sim,
        lodger,
        "core:building/hut",
        Shape::Round { radius: 300 },
        l.own_home,
    );
    assert!(
        sim.people().letting.lodging(lodger).is_some(),
        "until its review"
    );
    sim.people_mut_for_tests().letting.lets[0].next_review = day;
    sim.with_ctx_for_tests(|pop, ctx| pop.letting_day_for_tests(ctx, &[]));
    assert!(sim.people().letting.lets.is_empty());
    let hut = sim
        .land()
        .buildings
        .iter()
        .find(|b| b.id == hut)
        .expect("its hut");
    let x = sim.people().household(lodger).expect("lodger");
    assert_eq!(x.home, build::centre_m(&hut.spec));
    assert!(x.sheltered);
    assert!(civ_sim::frames::people::lodging_words(&sim, lodger).is_empty());
}

#[test]
fn a_lodger_that_cannot_pay_leaves_after_two_months() {
    let (mut sim, _, lodger) = lodged();
    let own_home = sim.people().letting.lets[0].own_home;
    let edible: Vec<usize> = sim
        .rules()
        .catalog
        .goods
        .iter()
        .enumerate()
        .filter(|(_, d)| d.edible())
        .map(|(g, _)| g)
        .collect();
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == lodger {
            for &g in &edible {
                x.stores[g] = 0.0;
            }
        }
    }
    let day = sim.now().day_index();
    sim.people_mut_for_tests().letting.lets[0].due = day;
    sim.with_ctx_for_tests(|pop, ctx| pop.letting_day_for_tests(ctx, &[]));
    let l = sim.people().letting.lodging(lodger).expect("still lodges");
    assert_eq!(l.unpaid, 1, "a month behind");
    assert!(civ_sim::frames::people::lodging_words(&sim, lodger)[0].ends_with("a month behind"));
    sim.people_mut_for_tests().letting.lets[0].due = day;
    sim.with_ctx_for_tests(|pop, ctx| pop.letting_day_for_tests(ctx, &[]));
    assert!(
        sim.people().letting.lodging(lodger).is_none(),
        "a second month: gone"
    );
    let x = sim.people().household(lodger).expect("lodger");
    assert!(!x.sheltered);
    assert_eq!(x.home, own_home);
}

#[test]
fn no_spare_floor_or_a_rent_too_dear_lets_nothing_and_asking_rents_move_by_a_step() {
    // A hut too small for anyone else: no let.
    let (mut sim, ids) = village(5_000.0);
    let step = sim.rules().people.letting.step;
    let prior = sim
        .rules()
        .people
        .letting
        .asking_rent_food_days_per_person_month;
    give_home(
        &mut sim,
        ids[0],
        "core:building/hut",
        Shape::Round { radius: 150 },
    );
    sim.with_ctx_for_tests(|pop, ctx| pop.letting_day_for_tests(ctx, &[ids[1]]));
    assert!(sim.people().letting.lets.is_empty());
    // A home with room, but asking more than being roofless costs: no let.
    let (mut sim, ids) = village(5_000.0);
    let big = Shape::Bays {
        bays: 8,
        storeys: 2,
        lofts: 0,
    };
    give_home(&mut sim, ids[0], "core:building/longhouse", big);
    sim.people_mut_for_tests()
        .letting
        .asking
        .insert(ids[0], 300.0);
    sim.with_ctx_for_tests(|pop, ctx| pop.letting_day_for_tests(ctx, &[ids[1]]));
    assert!(sim.people().letting.lets.is_empty(), "too dear");
    // Its review with spare floor, no lodger and roofless neighbours: a step less.
    sim.with_ctx_for_tests(|pop, ctx| pop.letting_day_for_tests(ctx, &[ids[0]]));
    let asking = sim.people().letting.asking.get(&ids[0]).copied();
    assert!((asking.expect("moved") - (300.0 - step)).abs() < 1e-9);
    // Room for one of two seekers: the first lodges, the second is turned away and its landlord
    // asks a step more.
    let (mut sim, ids) = village(5_000.0);
    let (a, b) = (ids[1], ids[2]);
    let need = |h| 4.8 * members(&sim, h) as f64;
    let (least, most) = (need(a).max(need(b)), need(a) + need(b));
    let own = 6.0 + 4.8 * members(&sim, ids[0]) as f64;
    let def = program("core:building/longhouse");
    let shape = def
        .shapes
        .iter()
        .filter(|c| c.living_m2 - own >= least + 0.5 && c.living_m2 - own < most - 0.5)
        .min_by(|x, y| x.living_m2.total_cmp(&y.living_m2))
        .map(|c| c.shape)
        .expect("a home with room for one of them");
    give_home(&mut sim, ids[0], "core:building/longhouse", shape);
    sim.with_ctx_for_tests(|pop, ctx| pop.letting_day_for_tests(ctx, &[a, b]));
    assert_eq!(sim.people().letting.lets.len(), 1);
    assert_eq!(sim.people().letting.lets[0].lodger, a);
    let asking = sim.people().letting.asking.get(&ids[0]).copied();
    assert!(
        (asking.expect("moved") - (prior + step)).abs() < 1e-9,
        "{asking:?}"
    );
}

#[test]
fn a_homes_design_counts_rent_only_where_lets_were_seen() {
    let (mut sim, ids) = village(5_000.0);
    let builder = ids[2];
    let credit = |sim: &mut Sim, m2: f64| {
        sim.with_ctx_for_tests(|pop, ctx| pop.rent_credit_for_tests(ctx, builder, m2))
    };
    // No let seen: nothing counted.
    assert_eq!(credit(&mut sim, 10.0), 0.0);
    let big = Shape::Bays {
        bays: 8,
        storeys: 2,
        lofts: 0,
    };
    give_home(&mut sim, ids[0], "core:building/longhouse", big);
    sim.with_ctx_for_tests(|pop, ctx| pop.letting_day_for_tests(ctx, &[ids[1]]));
    let l = sim.people().letting.lets[0].clone();
    let settlement = sim.people().household(ids[0]).and_then(|x| x.settlement);
    let rent = sim
        .people()
        .letting
        .recent_rent(settlement.expect("settled"), sim.now().day_index())
        .expect("a let seen");
    let kcal = l.rent_kg * sim.rules().catalog.goods[l.good].kcal_per_kg;
    assert!((rent - kcal / l.floor_m2).abs() < 1e-9);
    // Lets seen and roofless neighbours: spare floor counts, in proportion.
    let ten = credit(&mut sim, 10.0);
    assert!(ten > 0.0);
    assert!((credit(&mut sim, 20.0) - 2.0 * ten).abs() < 1e-9 * ten);
    assert_eq!(credit(&mut sim, 0.0), 0.0);
    // The share of the settlement's households roofless scales it: here, all but the landlord
    // and its lodger.
    let all = sim
        .people()
        .households
        .iter()
        .filter(|(_, x)| !x.members.is_empty() && x.settlement == settlement)
        .count();
    let roofless = sim
        .people()
        .households
        .iter()
        .filter(|(_, x)| !x.members.is_empty() && x.settlement == settlement && !x.sheltered)
        .count();
    assert_eq!(roofless, all - 2);
    let hour_kcal = rent * 10.0 * 12.0 * (roofless as f64 / all as f64) / ten;
    assert!(
        (civ_agents::letting::rent_credit_h_year(
            Some(rent),
            10.0,
            roofless as f64 / all as f64,
            hour_kcal
        ) - ten)
            .abs()
            < 1e-9
    );
    // Nobody roofless: nothing counted, lets seen or not.
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        x.sheltered = true;
    }
    assert_eq!(credit(&mut sim, 10.0), 0.0);
    // A year on, the let is no longer recent.
    sim.people_mut_for_tests().letting.begun[0].day -= 365;
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        x.sheltered = false;
    }
    assert_eq!(credit(&mut sim, 10.0), 0.0);
}

#[test]
fn a_world_with_lets_saves_and_loads_exactly() {
    let (mut sim, landlord, _) = lodged();
    assert_eq!(sim.people().letting.lets.len(), 1);
    sim.people_mut_for_tests()
        .letting
        .asking
        .insert(landlord, 1.3);
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(&mut sim, &saves, commons_persist::SaveKind::Manual, "let").expect("saves");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loaded.people().letting, sim.people().letting);
    let same = |x: &Sim, y: &Sim, when: &str| {
        let (x, y) = (persist::encode_sections(x), persist::encode_sections(y));
        assert_eq!(x.len(), y.len());
        for (p, q) in x.iter().zip(&y) {
            assert!(p.bytes == q.bytes, "section `{}` differs {when}", p.tag);
        }
    };
    same(&sim, &loaded, "on loading");
    // The lodger's shelter is derived on loading as it was.
    for (_, x) in sim.people().households.iter() {
        let y = loaded.people().household(x.id).expect("loaded");
        assert_eq!(
            (x.sheltered, x.keeping),
            (y.sheltered, y.keeping),
            "{}",
            x.id
        );
    }
    sim.advance_minutes(2 * DAY).expect("lives");
    loaded.advance_minutes(2 * DAY).expect("lives");
    same(&sim, &loaded, "two days on");
}

#[test]
fn where_nobody_is_roofless_letting_changes_nothing() {
    let (mut sim, ids) = village(5_000.0);
    let big = Shape::Bays {
        bays: 8,
        storeys: 2,
        lofts: 0,
    };
    let all: Vec<PermanentId> = sim
        .people()
        .households
        .iter()
        .filter(|(_, x)| !x.members.is_empty())
        .map(|(_, x)| x.id)
        .collect();
    for &h in &all {
        if h == ids[0] {
            give_home(&mut sim, h, "core:building/longhouse", big);
        } else {
            give_home(
                &mut sim,
                h,
                "core:building/hut",
                Shape::Round { radius: 300 },
            );
        }
    }
    assert!(
        sim.people()
            .households
            .iter()
            .all(|(_, x)| x.members.is_empty() || x.sheltered)
    );
    let before = persist::encode_sections(&sim);
    sim.with_ctx_for_tests(|pop, ctx| pop.letting_day_for_tests(ctx, &all));
    let after = persist::encode_sections(&sim);
    assert_eq!(before.len(), after.len());
    for (p, q) in before.iter().zip(&after) {
        assert!(p.bytes == q.bytes, "section `{}` differs", p.tag);
    }
    assert!(sim.people().letting.lets.is_empty() && sim.people().letting.asking.is_empty());
}
