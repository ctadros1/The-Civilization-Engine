//! Buildings a polity raises and owns (M8d slice DC; ADR-0031 §6): the losses each public program
//! answers are measured first, for nobody proposes one without a measured loss.
use civ_agents::polity::Gathering;
use civ_content::ContentRegistry;
use civ_sim::{NewWorld, Sim, persist};
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

fn village(seed: u64) -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Halling".to_owned(),
            seed,
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

fn reloaded(sim: &mut Sim) -> Sim {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "public").expect("saves");
    persist::load(&saved.path, content()).expect("loads")
}

#[test]
fn a_polity_measures_its_stores_spoiling_its_gatherings_in_the_wet_and_fires_that_reach_a_roof() {
    let mut sim = village(5);
    sim.advance_minutes(DAY).expect("advances");
    let (owner, settlement) = sim
        .people()
        .households
        .iter()
        .filter(|(_, x)| !x.members.is_empty())
        .filter_map(|(_, x)| Some((x.id, x.settlement?)))
        .min_by_key(|x| x.0)
        .expect("a settled household");
    let pi = sim
        .people()
        .polities
        .iter()
        .position(|p| p.settlement == settlement)
        .expect("a polity");
    let year = sim.now().date().year;
    let measured = |sim: &Sim| {
        sim.people().polities[pi]
            .measured
            .iter()
            .find(|m| m.year == year)
            .copied()
            .unwrap_or_default()
    };
    assert_eq!(measured(&sim).spoiled_kcal, 0.0);
    // Grain in the common store, left a month at the hearth under no roof, spoils.
    let grain = content()
        .catalog
        .goods
        .iter()
        .position(|g| g.kcal_per_kg > 0.0 && g.half_life_days > 0.0)
        .expect("a food that spoils");
    {
        let now = sim.now();
        let p = &mut sim.people_mut_for_tests().polities[pi];
        p.stores.resize(content().catalog.goods.len(), 0.0);
        p.stores[grain] += 500.0;
        p.stores_at = now;
    }
    sim.advance_minutes(30 * DAY).expect("advances");
    sim.with_ctx_for_tests(|pop, ctx| {
        pop.polities[pi].settle_stores(ctx.now, &ctx.catalog.goods, false);
    });
    let m = measured(&sim);
    assert!(m.spoiled_kcal > 0.0, "{m:?}");
    // A gathering held at the hearth is counted, and on a wet day counted wet.
    let wet = sim.land().weather.today.wet;
    let before = measured(&sim);
    sim.with_ctx_for_tests(|pop, ctx| {
        pop.polities[pi].gathering = Some(Gathering {
            law: None,
            day: ctx.now.day_index(),
            stakes: Vec::new(),
            present: Vec::new(),
            cases: Vec::new(),
        });
        pop.decide_gathering_for_tests(ctx, pi);
    });
    let after = measured(&sim);
    assert_eq!(after.gatherings, before.gatherings + 1);
    assert_eq!(after.wet_gatherings, before.wet_gatherings + u32::from(wet));
    // A fire that reaches a roof is counted against the settlement.
    let hut = {
        let catalog = &content().catalog;
        let def = &catalog.buildings[catalog.building_index("core:building/hut").expect("hut")];
        let at = sim.people().household(owner).expect("it").home;
        let spec = civ_agents::build::design_shape(
            def,
            &catalog.goods,
            civ_agents::build::Shape::Round { radius: 310 },
            (at.0 + 30.0, at.1),
            None,
        )
        .expect("a hut");
        sim.place_building_for_tests(owner, spec, civ_grammar::Stage::ALL.len() as u8)
            .expect("placed")
    };
    let fires = measured(&sim).roof_fires;
    sim.set_fire(hut).expect("set alight");
    assert_eq!(measured(&sim).roof_fires, fires + 1);
    // What was measured survives a save.
    let loaded = reloaded(&mut sim);
    assert_eq!(
        loaded.people().polities[pi].measured,
        sim.people().polities[pi].measured
    );
}
