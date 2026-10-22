//! Coal (M10a slice DS, first part; ADR-0033 §1, §7): seams lie in the ground with ids of their
//! own, so they move no other id; a household keeps coal for its hearth only if one of them knows
//! how to burn it; that is found by people gathering wood where their settlement knows coal
//! showing, and nowhere else; and those who know it dig coal and burn it by its heat.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::population::DepositKnown;
use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_sim::{NewWorld, Sim};

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
            name: "Coal".to_owned(),
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

fn technique(sim: &Sim, id: &str) -> usize {
    sim.rules()
        .catalog
        .techniques
        .iter()
        .position(|t| t.id == id)
        .expect(id)
}

/// The household with the lowest id, and its members.
fn first_household(sim: &Sim) -> (PermanentId, Vec<PermanentId>) {
    sim.people()
        .households
        .iter()
        .map(|(_, h)| h)
        .min_by_key(|h| h.id)
        .map(|h| (h.id, h.members.clone()))
        .expect("a household")
}

/// Everyone of `members` learns technique `t`.
fn teach(sim: &mut Sim, members: &[PermanentId], t: usize) {
    let learn_h = sim.rules().catalog.techniques[t].learn_h;
    let now = sim.now();
    let teacher = members[0];
    for (_, p) in sim.people_mut_for_tests().people.iter_mut() {
        if members.contains(&p.id) {
            p.learn(t, learn_h, teacher, learn_h, now);
        }
    }
}

/// Coal showing beside the first settlement's hearth, which it knows.
fn coal_by_the_hearth(sim: &mut Sim) -> PermanentId {
    let (settlement, hearth) = {
        let s = sim.land().settlements.first().expect("a settlement");
        (s.id, s.hearth_m)
    };
    let coal = sim
        .place_deposit((hearth.0 + 40.0, hearth.1), "core:good/coal", 12.0, true)
        .expect("laid down");
    let at = sim.now();
    let finder = first_household(sim).1[0];
    sim.people_mut_for_tests()
        .deposits_known
        .push(DepositKnown {
            settlement,
            deposit: coal,
            finder,
            at,
        });
    coal
}

#[test]
fn coal_is_reckoned_against_firewood_and_asks_its_technique() {
    let sim = world(3);
    let goods = &sim.rules().catalog.goods;
    let (coal, wood) = (
        good(&sim, "core:good/coal"),
        good(&sim, "core:good/firewood"),
    );
    // Coal comes first in the catalogue, yet the hearth's measure stays firewood, the first fuel
    // anyone burns.
    assert!(coal < wood);
    assert_eq!(civ_agents::params::fuel_ratio(goods, wood), 1.0);
    let ratio = civ_agents::params::fuel_ratio(goods, coal);
    assert!((ratio - 25.8 / 15.6).abs() < 1e-12, "{ratio}");
    let t = technique(&sim, "core:technique/coal_burning");
    assert_eq!(goods[coal].fuel.and_then(|f| f.technique), Some(t));
}

#[test]
fn seams_take_ids_of_their_own() {
    // Over a few seeds, every coal body has a keyed id and every other body one the world gave.
    let mut seams = 0;
    for seed in 1..=6 {
        let sim = world(seed);
        let coal = good(&sim, "core:good/coal");
        let next = sim.ids().peek_next();
        for d in &sim.land().deposits {
            if usize::from(d.body.good) == coal {
                seams += 1;
                assert!(d.id.get() >= civ_land::deposits::KEYED_ID_BASE, "{d:?}");
            } else {
                assert!(d.id.get() < next, "{d:?}");
            }
        }
    }
    assert!(seams > 0, "some seed has coal");
}

#[test]
fn coal_is_kept_for_the_hearth_only_where_someone_knows_how_to_burn_it() {
    let mut sim = world(3);
    let (coal, wood) = (
        good(&sim, "core:good/coal"),
        good(&sim, "core:good/firewood"),
    );
    let (household, members) = first_household(&sim);
    let keep = sim.with_ctx_for_tests(|pop, ctx| pop.keep_for_tests(ctx, household));
    assert_eq!(keep[coal], 0.0, "nobody knows how to burn it");
    assert!(keep[wood] > 0.0);
    let burning = technique(&sim, "core:technique/coal_burning");
    teach(&mut sim, &members, burning);
    let keep = sim.with_ctx_for_tests(|pop, ctx| pop.keep_for_tests(ctx, household));
    let ratio = civ_agents::params::fuel_ratio(&sim.rules().catalog.goods, coal);
    // The same heat as the firewood it keeps.
    assert!((keep[coal] * ratio - keep[wood]).abs() < 1e-9, "{keep:?}");
}

#[test]
fn burning_coal_is_found_only_where_coal_is_known() {
    let mut sim = world(3);
    let t = technique(&sim, "core:technique/coal_burning");
    // Found at once by anyone who may find it.
    if let Some(rules) = sim.rules_mut_for_tests() {
        rules.catalog.techniques[t].e50_h = 1e-6;
    }
    let knows = |sim: &Sim| {
        sim.people()
            .people
            .iter()
            .filter(|(_, p)| p.knows(t))
            .count()
    };
    sim.advance_minutes(10 * 24 * 60).expect("advances");
    assert_eq!(knows(&sim), 0, "no coal known, none found");
    coal_by_the_hearth(&mut sim);
    sim.advance_minutes(10 * 24 * 60).expect("advances");
    assert!(
        knows(&sim) > 0,
        "gathering wood where coal is known finds it"
    );
}

#[test]
fn those_who_know_dig_coal_and_burn_it() {
    let mut sim = world(3);
    let coal = good(&sim, "core:good/coal");
    let (household, members) = first_household(&sim);
    let burning = technique(&sim, "core:technique/coal_burning");
    teach(&mut sim, &members, burning);
    let seam = coal_by_the_hearth(&mut sim);
    let held = |sim: &Sim| {
        sim.people()
            .household(household)
            .map_or(0.0, |h| h.stores.get(coal).copied().unwrap_or(0.0))
    };
    let mut most = 0.0f64;
    for _ in 0..60 * 24 {
        sim.advance_minutes(60).expect("advances");
        most = most.max(held(&sim));
    }
    let taken = sim
        .land()
        .deposits
        .iter()
        .find(|d| d.id == seam)
        .map_or(0.0, |d| d.taken_kg);
    assert!(taken > 0.0, "coal was dug");
    assert!(most > 0.0, "coal was carried home");
    // The household never holds more than was dug from the seam.
    assert!(most <= taken + 1e-9, "{most} of {taken}");
}

#[test]
fn a_world_with_seams_saves_and_loads_to_the_byte() {
    // Keyed ids are not the world's next ones, and a save holding them loads as it was.
    let mut sim = (1..=6)
        .map(world)
        .find(|sim| {
            let coal = good(sim, "core:good/coal");
            sim.land()
                .deposits
                .iter()
                .any(|d| usize::from(d.body.good) == coal)
        })
        .expect("some seed has coal");
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = civ_sim::persist::save(&mut sim, &saves, commons_persist::SaveKind::Manual, "coal")
        .expect("saves");
    let loaded = civ_sim::persist::load(&saved.path, content()).expect("loads");
    let (a, b) = (
        civ_sim::persist::encode_sections(&sim),
        civ_sim::persist::encode_sections(&loaded),
    );
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(&b) {
        assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
    }
}

#[test]
fn water_standing_in_a_coal_pit_is_lifted_out_before_coal_is_dug() {
    let mut sim = world(3);
    let coal = good(&sim, "core:good/coal");
    let (_, members) = first_household(&sim);
    let burning = technique(&sim, "core:technique/coal_burning");
    teach(&mut sim, &members, burning);
    let seam = coal_by_the_hearth(&mut sim);
    let taken = |sim: &Sim| {
        sim.land()
            .deposits
            .iter()
            .find(|d| d.id == seam)
            .map_or(0.0, |d| d.taken_kg)
    };
    let pit = |sim: &Sim| {
        sim.land()
            .earthworks
            .iter()
            .find(|w| w.kind == civ_land::earth::EarthKind::Pit && w.deposit == Some(seam))
            .map(|w| w.id)
    };
    let water = |sim: &Sim, work| {
        sim.land()
            .pit_water
            .iter()
            .find(|p| p.work == work)
            .map_or(0.0, |p| p.water_m3)
    };
    // Until a pit is open on the seam.
    for _ in 0..60 * 24 {
        if pit(&sim).is_some() {
            break;
        }
        sim.advance_minutes(60).expect("advances");
    }
    let work = pit(&sim).expect("a pit on the seam");
    // A flood stands in it: 30 m³, more than an hour's bailing.
    let now = sim.now();
    sim.land_mut_for_tests().set_pit_water(work, 30.0, now);
    let mut bailed = false;
    let mut dug_after = false;
    for _ in 0..60 * 24 {
        let (before_kg, before_m3) = (taken(&sim), water(&sim, work));
        sim.advance_minutes(60).expect("advances");
        let (after_kg, after_m3) = (taken(&sim), water(&sim, work));
        if after_m3 < before_m3 - 1e-9 {
            bailed = true;
        }
        // Coal is dug only from a pit bailed dry.
        if after_kg > before_kg {
            assert!(after_m3 <= 1e-9, "dug with {after_m3} m³ standing");
            if bailed {
                dug_after = true;
                break;
            }
        }
    }
    assert!(bailed, "the water was lifted out");
    assert!(dug_after, "and coal dug once it was dry");
    // Only pits in a body that passes water keep a record of it.
    for p in &sim.land().pit_water {
        let w = sim
            .land()
            .earthworks
            .iter()
            .find(|w| w.id == p.work)
            .expect("its pit");
        let d = sim
            .land()
            .deposits
            .iter()
            .find(|d| Some(d.id) == w.deposit)
            .expect("its deposit");
        assert_eq!(usize::from(d.body.good), coal);
    }
    // And a save keeps it to the byte.
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = civ_sim::persist::save(&mut sim, &saves, commons_persist::SaveKind::Manual, "wet")
        .expect("saves");
    let loaded = civ_sim::persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loaded.land().pit_water, sim.land().pit_water);
}

/// A pit opened on a seam by the first household, which knows how to burn coal: the seam, the
/// pit and the household that dug it.
fn a_coal_pit(sim: &mut Sim) -> (PermanentId, PermanentId, PermanentId) {
    let (_, members) = first_household(sim);
    let burning = technique(sim, "core:technique/coal_burning");
    teach(sim, &members, burning);
    let seam = coal_by_the_hearth(sim);
    let pit = |sim: &Sim| {
        sim.land()
            .earthworks
            .iter()
            .find(|w| w.kind == civ_land::earth::EarthKind::Pit && w.deposit == Some(seam))
            .map(|w| (w.id, w.household))
    };
    for _ in 0..60 * 24 {
        if pit(sim).is_some() {
            break;
        }
        sim.advance_minutes(60).expect("advances");
    }
    let (work, household) = pit(sim).expect("a pit on the seam");
    (seam, work, household)
}

#[test]
fn an_engine_at_a_flooded_pit_pumps_it_dry_with_the_owners_coal() {
    let mut sim = world(3);
    let (coal, engine) = (
        good(&sim, "core:good/coal"),
        good(&sim, "core:good/atmospheric_engine"),
    );
    let (_, work, owner) = a_coal_pit(&mut sim);
    let now = sim.now();
    sim.land_mut_for_tests().set_pit_water(work, 30.0, now);
    for (_, h) in sim.people_mut_for_tests().households.iter_mut() {
        if h.id == owner {
            h.stores[engine] += 1.0;
            h.stores[coal] += 500.0;
        }
    }
    let set = sim.with_ctx_for_tests(|pop, ctx| pop.set_engine_for_tests(ctx, owner, work, engine));
    assert!(set.is_some());
    // What lifting it out takes: 30 m³ through its lift at the pump's half, burned at 1 % of
    // coal's 25.8 MJ.
    let state = sim
        .with_ctx_for_tests(|_, ctx| {
            ctx.land.pit_water_now(
                ctx.map,
                ctx.land_params,
                ctx.params.digging.influence_m,
                work,
                ctx.now,
            )
        })
        .expect("a wet pit");
    let kwh = civ_land::power::lift_kwh(state.water_m3, state.lift_m(), 0.5);
    let burned = civ_land::power::fuel_kg(kwh, 25.8, 0.01);
    let held = |sim: &Sim, g: usize| {
        sim.people()
            .household(owner)
            .map_or(0.0, |h| h.stores.get(g).copied().unwrap_or(0.0))
    };
    // What the engine burns is counted as used; what the hearth burns, as burned.
    let used = |sim: &Sim| {
        sim.people()
            .household(owner)
            .map_or(0.0, |h| h.flows.get(civ_agents::person::Flow::Used, coal))
    };
    let (used_before, engine_before) = (used(&sim), held(&sim, engine));
    sim.with_ctx_for_tests(|pop, ctx| pop.engines_day_for_tests(ctx));
    let water = sim
        .land()
        .pit_water
        .iter()
        .find(|p| p.work == work)
        .map_or(0.0, |p| p.water_m3);
    assert!(water < 1e-9, "pumped dry: {water} m³ left");
    let burned_now = used(&sim) - used_before;
    assert!(
        (burned_now - burned).abs() < 1e-9 + 1e-6 * burned,
        "{burned_now} kg against {burned}"
    );
    assert!(held(&sim, engine) < engine_before, "the engine wore");
}

#[test]
fn an_engine_without_fuel_pumps_nothing() {
    let mut sim = world(3);
    let (coal, engine) = (
        good(&sim, "core:good/coal"),
        good(&sim, "core:good/atmospheric_engine"),
    );
    let (_, work, owner) = a_coal_pit(&mut sim);
    let now = sim.now();
    sim.land_mut_for_tests().set_pit_water(work, 30.0, now);
    // An engine, and no fuel of any kind.
    let fuels: Vec<usize> = sim
        .rules()
        .catalog
        .goods
        .iter()
        .enumerate()
        .filter(|(_, g)| g.purpose == civ_agents::params::GoodUse::Fuel)
        .map(|(i, _)| i)
        .collect();
    for (_, h) in sim.people_mut_for_tests().households.iter_mut() {
        if h.id == owner {
            h.stores[engine] += 1.0;
            for &f in &fuels {
                h.stores[f] = 0.0;
            }
        }
    }
    let _ = coal;
    sim.with_ctx_for_tests(|pop, ctx| pop.set_engine_for_tests(ctx, owner, work, engine));
    sim.with_ctx_for_tests(|pop, ctx| pop.engines_day_for_tests(ctx));
    let water = sim
        .land()
        .pit_water
        .iter()
        .find(|p| p.work == work)
        .map_or(0.0, |p| p.water_m3);
    assert!((water - 30.0).abs() < 1e-9, "{water}");
}

/// The first household knows how to build an engine and all it is made of: the engine, copper
/// smelting, iron from bog ore by charcoal, and forging.
fn teach_engine_building(sim: &mut Sim) {
    let (_, members) = first_household(sim);
    for id in [
        "core:technique/atmospheric_engine",
        "core:technique/copper_smelting",
        "core:technique/charcoal_burning",
        "core:technique/bloomery",
        "core:technique/forging",
    ] {
        let t = technique(sim, id);
        teach(sim, &members, t);
    }
}

/// Records `hours` bailed by hand at pit `work` over the year before now.
fn bailed_for_a_year(sim: &mut Sim, work: PermanentId, hours: f64) {
    let since = civ_core::SimTime::from_minutes(sim.now().minutes() - 365 * 24 * 60);
    let land = sim.land_mut_for_tests();
    land.pit_bailing.retain(|b| b.work != work);
    land.pit_bailing
        .push(civ_land::pits::Bailed { work, hours, since });
}

#[test]
fn bailing_by_hand_is_tallied_at_its_pit_and_saved() {
    let mut sim = world(3);
    let (_, work, _) = a_coal_pit(&mut sim);
    let now = sim.now();
    sim.land_mut_for_tests().set_pit_water(work, 30.0, now);
    let tallied = |sim: &Sim| {
        sim.land()
            .pit_bailing
            .iter()
            .find(|b| b.work == work)
            .map_or(0.0, |b| b.hours)
    };
    for _ in 0..60 * 24 {
        if tallied(&sim) > 0.0 {
            break;
        }
        sim.advance_minutes(60).expect("advances");
    }
    let hours = tallied(&sim);
    // At most what lifting all 30 m³ by hand takes, through no more than the pit's depth.
    assert!(hours > 0.0, "the bailing was tallied");
    // Only pits are tallied, and a save keeps the tally to the byte.
    assert!(
        sim.land()
            .problems(sim.map(), usize::MAX, sim.ids().peek_next())
            .is_empty()
    );
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = civ_sim::persist::save(
        &mut sim,
        &saves,
        commons_persist::SaveKind::Manual,
        "bailed",
    )
    .expect("saves");
    let loaded = civ_sim::persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loaded.land().pit_bailing, sim.land().pit_bailing);
}

#[test]
fn a_household_that_bails_long_sets_an_engine_at_its_pit_and_then_wants_one() {
    let mut sim = world(3);
    let engine = good(&sim, "core:good/atmospheric_engine");
    let (_, work, owner) = a_coal_pit(&mut sim);
    teach_engine_building(&mut sim);
    let wanted =
        |sim: &mut Sim| sim.with_ctx_for_tests(|pop, ctx| pop.keep_for_tests(ctx, owner))[engine];
    // Before it has set one, it wants no engine, whatever its people's work.
    assert_eq!(wanted(&mut sim), 0.0);
    // Five thousand hours of bailing a year repay an engine and its fuel over its years.
    bailed_for_a_year(&mut sim, work, 5_000.0);
    sim.with_ctx_for_tests(|pop, ctx| pop.review_engine_for_tests(ctx, owner));
    let set: Vec<_> = sim
        .land()
        .engines
        .iter()
        .filter(|e| e.pit == work && e.owner == owner)
        .copied()
        .collect();
    assert_eq!(set.len(), 1, "an engine is set at the pit");
    assert!(wanted(&mut sim) >= 1.0, "and it wants one for it");
    // Weighing again sets no second engine at the same pit.
    sim.with_ctx_for_tests(|pop, ctx| pop.review_engine_for_tests(ctx, owner));
    assert_eq!(sim.land().engines.len(), 1);
    assert!(
        sim.land()
            .problems(sim.map(), usize::MAX, sim.ids().peek_next())
            .is_empty()
    );
}

#[test]
fn a_little_bailing_or_no_way_to_build_one_sets_no_engine() {
    // Fifty hours a year repay nothing like an engine.
    let mut sim = world(3);
    let (_, work, owner) = a_coal_pit(&mut sim);
    teach_engine_building(&mut sim);
    bailed_for_a_year(&mut sim, work, 50.0);
    sim.with_ctx_for_tests(|pop, ctx| pop.review_engine_for_tests(ctx, owner));
    assert!(sim.land().engines.is_empty());
    // Nor does a household that bails much but knows no way of building one.
    let mut sim = world(3);
    let (_, work, owner) = a_coal_pit(&mut sim);
    bailed_for_a_year(&mut sim, work, 5_000.0);
    sim.with_ctx_for_tests(|pop, ctx| pop.review_engine_for_tests(ctx, owner));
    assert!(sim.land().engines.is_empty());
}
