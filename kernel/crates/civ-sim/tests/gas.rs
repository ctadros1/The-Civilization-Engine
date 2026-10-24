//! Gas (M10b slice DY; ADR-0034 §8): a gasworks' retort makes gas from coal, with coke left over,
//! into its holder; gas made beyond the holder's room, or with no holder, is lost to the air.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::person::Flow;
use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_sim::{NewWorld, Sim, persist};

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

fn world() -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Gasworks".to_owned(),
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
    .expect("generates")
}

fn index(sim: &Sim, id: &str) -> usize {
    sim.rules()
        .catalog
        .goods
        .iter()
        .position(|g| g.id == id)
        .expect(id)
}

/// An adult of some household, by household and person.
fn adult(sim: &Sim) -> (PermanentId, PermanentId) {
    let now = sim.now();
    sim.people()
        .households
        .iter()
        .find_map(|(_, h)| {
            let p = h.members.iter().copied().find(|&m| {
                sim.people()
                    .person(m)
                    .is_some_and(|p| p.age_years(now) >= 20.0)
            })?;
            Some((h.id, p))
        })
        .expect("an adult")
}

fn teach(sim: &mut Sim, household: PermanentId, id: &str) {
    let t = sim
        .rules()
        .catalog
        .techniques
        .iter()
        .position(|t| t.id == id)
        .expect("a technique");
    let learn_h = sim.rules().catalog.techniques[t].learn_h;
    let members = sim
        .people()
        .household(household)
        .expect("household")
        .members
        .clone();
    let now = sim.now();
    for (_, p) in sim.people_mut_for_tests().people.iter_mut() {
        if members.contains(&p.id) {
            p.learn(t, learn_h, members[0], learn_h, now);
        }
    }
}

fn set(sim: &mut Sim, household: PermanentId, g: usize, kg: f64) {
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == household {
            x.stores[g] = kg;
        }
    }
}

fn saves_alike(sim: &mut Sim) {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "gas").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    let (a, b) = (
        persist::encode_sections(sim),
        persist::encode_sections(&loaded),
    );
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(&b) {
        assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
    }
}

#[test]
fn a_retort_makes_gas_into_its_holder_and_what_has_no_room_goes_to_the_air() {
    let mut sim = world();
    let (gas, coke, coal, retort, holder) = (
        index(&sim, "core:good/coal_gas"),
        index(&sim, "core:good/coke"),
        index(&sim, "core:good/coal"),
        index(&sim, "core:good/gas_retort"),
        index(&sim, "core:good/gas_holder"),
    );
    let make_gas = sim
        .rules()
        .catalog
        .activities
        .iter()
        .position(|a| a.id == "core:activity/make_gas")
        .expect("making gas");
    let (hh, who) = adult(&sim);
    teach(&mut sim, hh, "core:technique/gas_making");
    set(&mut sim, hh, coal, 10_000.0);
    set(&mut sim, hh, coke, 0.0);
    set(&mut sim, hh, gas, 0.0);
    set(&mut sim, hh, holder, 0.0);
    let stores = |sim: &Sim| {
        sim.people()
            .household(hh)
            .expect("household")
            .stores
            .clone()
    };
    let spoiled = |sim: &Sim| {
        sim.people()
            .household(hh)
            .expect("household")
            .flows
            .get(Flow::Spoiled, gas)
    };
    let work = |sim: &mut Sim, minutes: u32| {
        set(sim, hh, retort, 1.0);
        sim.with_ctx_for_tests(|pop, ctx| pop.make_work_for_tests(ctx, who, make_gas, minutes));
    };
    // With no holder, a session's gas goes to the air; its coke stays and its coal is spent.
    let before = stores(&sim);
    work(&mut sim, 240);
    let after = stores(&sim);
    let burned = before[coal] - after[coal];
    assert!(burned > 0.0, "coal charged into the retort");
    assert!(
        (after[coke] - 0.65 * burned).abs() < 1e-6,
        "{} coke",
        after[coke]
    );
    assert_eq!(after[gas], 0.0, "no holder keeps it");
    assert!(
        (spoiled(&sim) - 0.14 * burned).abs() < 1e-6,
        "all of it lost to the air"
    );
    // With a holder, a short session's gas (an hour of making past the retort's two of firing) is
    // kept, as much as its coal gives.
    set(&mut sim, hh, holder, 1.0);
    let before = stores(&sim);
    work(&mut sim, 180);
    let after = stores(&sim);
    let burned = before[coal] - after[coal];
    assert!(burned > 0.0);
    assert!(
        (after[gas] - 0.14 * burned).abs() < 1e-6,
        "{} gas",
        after[gas]
    );
    assert!(after[gas] < 150.0);
    // Long sessions fill it, and what has no room goes to the air.
    let lost_before = spoiled(&sim);
    for _ in 0..8 {
        work(&mut sim, 480);
    }
    let after = stores(&sim);
    assert!((after[gas] - 150.0).abs() < 1e-9, "{} gas", after[gas]);
    assert!(spoiled(&sim) > lost_before, "the rest is lost");
    // The holder's gas by its energy: 150 kg at 40 MJ a kilogram.
    let goods = &sim.rules().catalog.goods;
    let mj = civ_agents::make::holder_room(goods, &after, gas).expect("a holder keeps gas")
        * goods[gas].fuel.expect("a fuel").mj_per_kg;
    assert!((mj - 6000.0).abs() < 1e-6, "{mj} MJ");
    // Kept a day, the gas stays in its holder.
    sim.advance_minutes(1440).expect("advances");
    let x = sim.people().household(hh).expect("household");
    let now = sim.now();
    let settled = x.stores_at_time(now, &sim.rules().catalog.goods, &|_| 0.0);
    assert!((settled[gas] - 150.0).abs() < 1e-9, "{} gas", settled[gas]);
    saves_alike(&mut sim);
}
