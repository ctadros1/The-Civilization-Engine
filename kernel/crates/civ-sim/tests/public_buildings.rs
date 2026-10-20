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

#[test]
fn a_common_store_that_spoils_is_answered_by_a_storehouse_raised_together_that_then_shelters_it() {
    use civ_agents::polity::{IssueKind, LawStatus, Outcome, PolicyKind};
    use civ_core::PermanentId;
    let mut sim = village(5);
    sim.advance_minutes(DAY).expect("advances");
    let settlement = sim
        .people()
        .households
        .iter()
        .filter(|(_, x)| !x.members.is_empty())
        .find_map(|(_, x)| x.settlement)
        .expect("a settlement");
    let pi = sim
        .people()
        .polities
        .iter()
        .position(|p| p.settlement == settlement)
        .expect("a polity");
    let template = content()
        .catalog
        .policies
        .iter()
        .position(|d| d.kind == PolicyKind::RaiseBuilding)
        .expect("a template to raise a building") as u16;
    let granary = content()
        .catalog
        .building_index("core:building/granary")
        .expect("granary");
    assert_eq!(
        content().catalog.policies[usize::from(template)]
            .raise
            .map(|r| r.program),
        Some(granary)
    );
    // Nothing measured, nothing to answer: a storehouse would bring nobody anything.
    let gains = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| pop.rule_gains_for_tests(ctx, settlement, template))
    };
    let g = gains(&mut sim);
    assert!(g.iter().all(|g| g.1 == 0.0), "{g:?}");
    // Grain in the common store, left a month under no roof, spoils, and is measured.
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
    assert!(
        sim.people().polities[pi]
            .measured
            .iter()
            .any(|m| m.spoiled_kcal > 0.0)
    );
    // Each household now weighs its share of the spoiling averted against its share of the work.
    let g = gains(&mut sim);
    assert!(g.iter().any(|g| g.1 > 0.0), "{g:?}");
    // Someone who knows how to frame a granary puts it forward, for the store spoils.
    let people: Vec<PermanentId> = sim
        .people()
        .households
        .iter()
        .filter(|(_, x)| x.settlement == Some(settlement))
        .flat_map(|(_, x)| x.members.clone())
        .collect();
    for &m in &people {
        sim.introduce_technique(m, "core:technique/jointed_frame", false)
            .expect("taught");
    }
    sim.people_mut_for_tests().every_adult_deliberates = true;
    let mut proposed = None;
    for _ in 0..30 {
        {
            let pop = sim.people_mut_for_tests();
            pop.polities[pi].gathering = None;
            for l in &mut pop.polities[pi].laws {
                if l.status == LawStatus::Proposed {
                    l.status = LawStatus::Rejected;
                }
            }
        }
        sim.with_ctx_for_tests(|pop, ctx| pop.review_for_tests(ctx, settlement));
        if let Some(l) = sim.people().polities[pi].agenda().cloned()
            && l.policy == template
        {
            proposed = Some(l);
            break;
        }
        sim.advance_minutes(DAY).expect("advances");
    }
    let law = proposed.expect("a storehouse proposed");
    assert_eq!(law.issue, IssueKind::StoreSpoils);
    let id = law.id;
    let site = sim.people().polities[pi]
        .raise_site_of(id)
        .expect("its site recorded");
    assert_eq!((usize::from(site.program), site.building), (granary, None));
    assert!(site.labour_h > 0.0);
    // Passed and known to all, it is laid out near the hearth, the polity's.
    {
        let today = sim.now().day_index();
        let pop = sim.people_mut_for_tests();
        pop.every_adult_deliberates = false;
        let p = &mut pop.polities[pi];
        p.gathering = None;
        let l = p.laws.iter_mut().find(|l| l.id == id).expect("the law");
        l.status = LawStatus::InForce;
        l.outcome = Some(Outcome::Passed);
        l.known = people.iter().map(|&a| (a, today)).collect();
    }
    sim.with_ctx_for_tests(|pop, ctx| pop.begin_public_raise_for_tests(ctx, pi, id));
    let polity = sim.people().polities[pi].id;
    let building = sim.people().polities[pi]
        .raise_site_of(id)
        .and_then(|s| s.building)
        .expect("laid out");
    {
        let b = sim
            .land()
            .buildings
            .iter()
            .find(|b| b.id == building)
            .expect("the building");
        assert_eq!(b.polity, Some(polity));
        assert_eq!(b.household, polity, "held in the polity's name");
        assert_eq!(
            content().catalog.building_index(&b.spec.program),
            Some(granary)
        );
        let hearth = sim
            .land()
            .settlements
            .iter()
            .find(|s| s.id == settlement)
            .expect("it")
            .hearth_m;
        let at = civ_agents::build::centre_m(&b.spec);
        assert!((at.0 - hearth.0).hypot(at.1 - hearth.1) <= 80.0 + 10.0);
    }
    // Words for the law say what is raised and the work it asks.
    let words = {
        let p = &sim.people().polities[pi];
        let law = p.laws.iter().find(|l| l.id == id).expect("the law").clone();
        p.words_of(&law, &sim.rules().catalog.policies, &|_| String::new())
    };
    assert!(words.contains("raised together near the hearth"), "{words}");
    // No second one while it is being raised: the issue is answered.
    let g = gains(&mut sim);
    assert!(g.iter().all(|g| g.1 == 0.0), "{g:?}");
    // Its store is not yet sheltered; the households give their hours until it stands.
    assert_eq!(sim.people().polities[pi].storehouse, None);
    let worker = sim
        .people()
        .households
        .iter()
        .find(|(_, x)| x.settlement == Some(settlement) && !x.members.is_empty())
        .map(|(_, x)| x.id)
        .expect("a household");
    let saved_partway = {
        sim.with_ctx_for_tests(|pop, ctx| pop.raise_work_for_tests(ctx, worker, building, 4.0));
        reloaded(&mut sim)
    };
    assert_eq!(
        saved_partway.people().polities[pi].raises,
        sim.people().polities[pi].raises
    );
    assert_eq!(
        saved_partway.people().polities[pi].raise_given,
        sim.people().polities[pi].raise_given
    );
    assert!(sim.people().polities[pi].raise_given_by(id, worker) > 0.0);
    for _ in 0..2000 {
        if sim.people().polities[pi].storehouse.is_some() {
            break;
        }
        sim.with_ctx_for_tests(|pop, ctx| pop.raise_work_for_tests(ctx, worker, building, 4.0));
    }
    assert_eq!(sim.people().polities[pi].storehouse, Some(building));
    assert!(
        sim.land()
            .buildings
            .iter()
            .any(|b| b.id == building && b.finished())
    );
    // Finished, it keeps the common store under its roof, and that survives a save.
    let loaded = reloaded(&mut sim);
    assert_eq!(loaded.people().polities[pi].storehouse, Some(building));
    let b = loaded
        .land()
        .buildings
        .iter()
        .find(|b| b.id == building)
        .expect("the building");
    assert_eq!(b.polity, Some(polity));
}
