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

#[test]
fn rain_keeps_people_from_a_gathering_in_the_open_until_a_hall_raised_together_shelters_it() {
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
    let people: Vec<PermanentId> = sim
        .people()
        .households
        .iter()
        .filter(|(_, x)| x.settlement == Some(settlement))
        .flat_map(|(_, x)| x.members.clone())
        .collect();
    let hall_template = content()
        .catalog
        .policies
        .iter()
        .position(|d| d.id == "core:policy/yourselves_roof_a_hall")
        .expect("a template to raise a hall") as u16;
    // Turns down whatever is before the gathering and reviews, up to 30 days, until someone puts
    // forward a template `want` accepts.
    let propose = |sim: &mut Sim, want: &dyn Fn(u16) -> bool| {
        sim.people_mut_for_tests().every_adult_deliberates = true;
        let mut found = None;
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
                && want(l.policy)
            {
                found = Some(l);
                break;
            }
            sim.advance_minutes(DAY).expect("advances");
        }
        sim.people_mut_for_tests().every_adult_deliberates = false;
        found
    };
    // A storehouse is put before a gathering.
    let store_template = content()
        .catalog
        .policies
        .iter()
        .position(|d| d.id == "core:policy/yourselves_raise_a_storehouse")
        .expect("a template to raise a storehouse") as u16;
    let put_forward = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| {
            pop.propose_for_tests(
                ctx,
                settlement,
                people[0],
                store_template,
                IssueKind::StoreSpoils,
            )
        })
        .expect("put forward")
    };
    let law = put_forward(&mut sim);
    // Rain takes the custom's own pull from a gathering held in the open.
    let someone = people[0];
    let points = |sim: &mut Sim, wet: bool| {
        sim.land_mut_for_tests().weather.today.wet = wet;
        sim.with_ctx_for_tests(|pop, ctx| pop.gathering_points_for_tests(ctx, someone))
            .expect("a gathering called")
    };
    let wet_attend = sim.with_ctx_for_tests(|_, ctx| ctx.params.polity.wet_attend);
    assert!(wet_attend > 0.0);
    let (dry, wet) = (points(&mut sim, false), points(&mut sim, true));
    assert!((dry - wet - wet_attend).abs() < 1e-9, "{dry} {wet}");
    // It sits on a wet day with only one come: no quorum, and the hour they gave is lost.
    let year = sim.now().date().year;
    let measured = |sim: &Sim| {
        sim.people().polities[pi]
            .measured
            .iter()
            .find(|m| m.year == year)
            .copied()
            .unwrap_or_default()
    };
    let before = measured(&sim);
    {
        let today = sim.now().day_index();
        let g = sim.people_mut_for_tests().polities[pi]
            .gathering
            .as_mut()
            .expect("called");
        g.day = today;
        g.present = vec![someone];
    }
    sim.land_mut_for_tests().weather.today.wet = true;
    sim.with_ctx_for_tests(|pop, ctx| pop.decide_gathering_for_tests(ctx, pi));
    let outcome = sim.people().polities[pi]
        .laws
        .iter()
        .find(|l| l.id == law)
        .and_then(|l| l.outcome);
    assert_eq!(outcome, Some(Outcome::NoQuorum));
    let minutes = sim.with_ctx_for_tests(|_, ctx| ctx.params.polity.gathering_minutes);
    let after = measured(&sim);
    assert_eq!(after.wet_gatherings, before.wet_gatherings + 1);
    assert!((after.wet_lost_h - before.wet_lost_h - f64::from(minutes) / 60.0).abs() < 1e-9);
    // One such gathering does not repay a hall: its work outweighs an hour lost.
    let gains =
        sim.with_ctx_for_tests(|pop, ctx| pop.rule_gains_for_tests(ctx, settlement, hall_template));
    assert!(gains.iter().all(|g| g.1 < 0.0), "{gains:?}");
    // A year of them, some 600 hours given for nothing, does.
    sim.people_mut_for_tests().polities[pi]
        .measured_mut(year)
        .wet_lost_h = 600.0;
    let gains =
        sim.with_ctx_for_tests(|pop, ctx| pop.rule_gains_for_tests(ctx, settlement, hall_template));
    assert!(gains.iter().any(|g| g.1 > 0.0), "{gains:?}");
    for &m in &people {
        sim.introduce_technique(m, "core:technique/jointed_frame", false)
            .expect("taught");
    }
    let hall_law = propose(&mut sim, &|p| p == hall_template).expect("a hall proposed");
    assert_eq!(hall_law.issue, IssueKind::RainedOff);
    assert_eq!(hall_law.kind, PolicyKind::RaiseBuilding);
    {
        let today = sim.now().day_index();
        let p = &mut sim.people_mut_for_tests().polities[pi];
        p.gathering = None;
        let l = p
            .laws
            .iter_mut()
            .find(|l| l.id == hall_law.id)
            .expect("the law");
        l.status = LawStatus::InForce;
        l.outcome = Some(Outcome::Passed);
        l.known = people.iter().map(|&a| (a, today)).collect();
    }
    sim.with_ctx_for_tests(|pop, ctx| pop.begin_public_raise_for_tests(ctx, pi, hall_law.id));
    let building = sim.people().polities[pi]
        .raise_site_of(hall_law.id)
        .and_then(|s| s.building)
        .expect("laid out");
    let worker = sim
        .people()
        .person(someone)
        .map(|p| p.household)
        .expect("a household");
    for _ in 0..4000 {
        if sim.people().polities[pi].hall.is_some() {
            break;
        }
        sim.with_ctx_for_tests(|pop, ctx| pop.raise_work_for_tests(ctx, worker, building, 4.0));
    }
    assert_eq!(sim.people().polities[pi].hall, Some(building));
    // Under its roof the rain takes nothing, and its gatherings are no longer held in the open.
    let law = put_forward(&mut sim);
    let (dry, wet) = (points(&mut sim, false), points(&mut sim, true));
    assert!((dry - wet).abs() < 1e-9, "{dry} {wet}");
    let before = measured(&sim);
    {
        let today = sim.now().day_index();
        let g = sim.people_mut_for_tests().polities[pi]
            .gathering
            .as_mut()
            .expect("called");
        g.day = today;
        g.present = vec![someone];
    }
    sim.land_mut_for_tests().weather.today.wet = true;
    sim.with_ctx_for_tests(|pop, ctx| pop.decide_gathering_for_tests(ctx, pi));
    assert!(sim.people().polities[pi].laws.iter().any(|l| l.id == law));
    let after = measured(&sim);
    assert_eq!(
        (after.gatherings, after.wet_lost_h),
        (before.gatherings, before.wet_lost_h)
    );
    // The hall and what was measured survive a save.
    let loaded = reloaded(&mut sim);
    assert_eq!(loaded.people().polities[pi].hall, Some(building));
    assert_eq!(
        loaded.people().polities[pi].measured,
        sim.people().polities[pi].measured
    );
}

#[test]
fn a_fire_nobody_sees_in_time_is_answered_by_a_watch_tower_whose_watch_sees_further() {
    use civ_agents::polity::{IssueKind, LawStatus, Outcome, PolicyKind};
    use civ_core::PermanentId;
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
    let people: Vec<PermanentId> = sim
        .people()
        .households
        .iter()
        .filter(|(_, x)| x.settlement == Some(settlement))
        .flat_map(|(_, x)| x.members.clone())
        .collect();
    let template = |id: &str| {
        content()
            .catalog
            .policies
            .iter()
            .position(|d| d.id == id)
            .expect("the template") as u16
    };
    let tower_template = template("core:policy/yourselves_watch_from_a_tower");
    // A hut `east` metres east of the hearth, finished.
    let hut_at = |sim: &mut Sim, east: f32| {
        let catalog = &content().catalog;
        let def = &catalog.buildings[catalog.building_index("core:building/hut").expect("hut")];
        let hearth = sim
            .land()
            .settlements
            .iter()
            .find(|s| s.id == settlement)
            .expect("it")
            .hearth_m;
        let spec = civ_agents::build::design_shape(
            def,
            &catalog.goods,
            civ_agents::build::Shape::Round { radius: 310 },
            (hearth.0 + east, hearth.1),
            None,
        )
        .expect("a hut");
        sim.place_building_for_tests(owner, spec, civ_grammar::Stage::ALL.len() as u8)
            .expect("placed")
    };
    let year = sim.now().date().year;
    let late_h = |sim: &Sim| {
        sim.people().polities[pi]
            .measured
            .iter()
            .find(|m| m.year == year)
            .map_or(0.0, |m| m.late_fire_h)
    };
    // In the small hours, with everyone asleep, a hut catches and burns out unseen: the work it
    // takes to put right is measured.
    let to_one = (25 * 60 - sim.now().minutes().rem_euclid(DAY)) % DAY;
    sim.advance_minutes(to_one.max(1)).expect("advances");
    let first = hut_at(&mut sim, 60.0);
    for (_, p) in sim.people_mut_for_tests().people.iter_mut() {
        p.asleep = true;
    }
    assert_eq!(late_h(&sim), 0.0);
    sim.set_fire(first).expect("set alight");
    sim.advance_minutes(3 * 60).expect("advances");
    let fire = sim
        .people()
        .fires
        .list
        .iter()
        .find(|f| f.building == first)
        .cloned()
        .expect("its fire");
    assert!(fire.ended.is_some(), "{fire:?}");
    assert!(late_h(&sim) > 0.0);
    // With no watch to look out from it, a tower would bring nobody anything.
    let gains = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| pop.rule_gains_for_tests(ctx, settlement, tower_template))
    };
    let g = gains(&mut sim);
    assert!(g.iter().all(|g| g.1 == 0.0), "{g:?}");
    // Someone is named to keep watch.
    let watcher = people[0];
    let watch_law = sim
        .with_ctx_for_tests(|pop, ctx| {
            pop.propose_for_tests(
                ctx,
                settlement,
                watcher,
                template("core:policy/keep_watch"),
                IssueKind::Takings,
            )
        })
        .expect("put forward");
    {
        let p = &mut sim.people_mut_for_tests().polities[pi];
        p.gathering = None;
        let l = p
            .laws
            .iter_mut()
            .find(|l| l.id == watch_law)
            .expect("the law");
        l.status = LawStatus::InForce;
        l.outcome = Some(Outcome::Passed);
        l.holder = Some(watcher);
    }
    assert!(sim.people().polities[pi].watchers().next().is_some());
    // A year of such fires repays a tower; once its people know framing, one puts it forward.
    sim.people_mut_for_tests().polities[pi]
        .measured_mut(year)
        .late_fire_h = 3000.0;
    let g = gains(&mut sim);
    assert!(g.iter().any(|g| g.1 > 0.0), "{g:?}");
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
            && l.policy == tower_template
        {
            proposed = Some(l);
            break;
        }
        sim.advance_minutes(DAY).expect("advances");
    }
    sim.people_mut_for_tests().every_adult_deliberates = false;
    let law = proposed.expect("a tower proposed");
    assert_eq!(law.issue, IssueKind::FiresUnseen);
    assert_eq!(law.kind, PolicyKind::RaiseBuilding);
    {
        let today = sim.now().day_index();
        let p = &mut sim.people_mut_for_tests().polities[pi];
        p.gathering = None;
        let l = p.laws.iter_mut().find(|l| l.id == law.id).expect("the law");
        l.status = LawStatus::InForce;
        l.outcome = Some(Outcome::Passed);
        l.known = people.iter().map(|&a| (a, today)).collect();
    }
    sim.with_ctx_for_tests(|pop, ctx| pop.begin_public_raise_for_tests(ctx, pi, law.id));
    let tower = sim.people().polities[pi]
        .raise_site_of(law.id)
        .and_then(|s| s.building)
        .expect("laid out");
    for _ in 0..4000 {
        if sim.people().polities[pi].tower.is_some() {
            break;
        }
        sim.with_ctx_for_tests(|pop, ctx| pop.raise_work_for_tests(ctx, owner, tower, 4.0));
    }
    assert_eq!(sim.people().polities[pi].tower, Some(tower));
    // A watch round now begins at the tower.
    let stops = sim.with_ctx_for_tests(|pop, ctx| pop.watch_stops_for_tests(ctx, watcher));
    let tower_at = {
        let b = sim
            .land()
            .buildings
            .iter()
            .find(|b| b.id == tower)
            .expect("the tower");
        civ_agents::build::centre_m(&b.spec)
    };
    assert_eq!(stops.first().copied(), Some(tower_at), "{stops:?}");
    // The watcher, awake at the tower, sees a roof catch far beyond the sight of anyone on the
    // ground.
    let top = {
        let b = sim
            .land()
            .buildings
            .iter()
            .find(|b| b.id == tower)
            .expect("the tower");
        civ_agents::build::centre_m(&b.spec)
    };
    let far = hut_at(&mut sim, 300.0);
    let far_at = {
        let b = sim
            .land()
            .buildings
            .iter()
            .find(|b| b.id == far)
            .expect("the hut");
        civ_agents::build::centre_m(&b.spec)
    };
    let sight = sim.with_ctx_for_tests(|_, ctx| ctx.params.fire.sight_m);
    let gap = f64::from(far_at.0 - top.0).hypot(f64::from(far_at.1 - top.1));
    assert!(
        gap > sight + 20.0 && gap < civ_agents::population::TOWER_SIGHT_M - 20.0,
        "{gap}"
    );
    {
        let pop = sim.people_mut_for_tests();
        for (_, p) in pop.people.iter_mut() {
            p.asleep = true;
        }
        let w = pop
            .people
            .iter_mut()
            .find(|(_, p)| p.id == watcher)
            .map(|(_, p)| p)
            .expect("the watcher");
        w.asleep = false;
        w.trip = None;
        w.pos = top;
    }
    sim.set_fire(far).expect("set alight");
    let fire = sim
        .people()
        .fires
        .list
        .iter()
        .find(|f| f.building == far)
        .map(|f| f.id)
        .expect("its fire");
    let saw = sim
        .people()
        .fires
        .knew
        .iter()
        .any(|k| k.fire == fire && k.person == watcher);
    assert!(saw, "the watcher at the tower saw it ({gap:.0} m off)");
    // The tower and what was measured survive a save.
    let loaded = reloaded(&mut sim);
    assert_eq!(loaded.people().polities[pi].tower, Some(tower));
    assert_eq!(
        loaded.people().polities[pi].measured,
        sim.people().polities[pi].measured
    );
}

#[test]
fn a_building_raised_together_keeps_its_dates_and_its_users_see_who_raised_it_or_let_it_stall() {
    use civ_agents::polity::{IssueKind, LawStatus, Outcome};
    use civ_agents::ties::Domain;
    use civ_core::{PermanentId, SimTime};
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
    let mut households: Vec<(PermanentId, Vec<PermanentId>)> = sim
        .people()
        .households
        .iter()
        .filter(|(_, x)| x.settlement == Some(settlement) && !x.members.is_empty())
        .map(|(_, x)| (x.id, x.members.clone()))
        .collect();
    households.sort_unstable();
    assert!(households.len() >= 3);
    let template = |id: &str| {
        content()
            .catalog
            .policies
            .iter()
            .position(|d| d.id == id)
            .expect("the template") as u16
    };
    let sponsor = households[0].1[0];
    // Passes a raising of template `t` sponsored by `sponsor` and lays it out.
    let raise = |sim: &mut Sim, t: u16| {
        let law = sim
            .with_ctx_for_tests(|pop, ctx| {
                pop.propose_for_tests(ctx, settlement, sponsor, t, IssueKind::StoreSpoils)
            })
            .expect("put forward");
        {
            let p = &mut sim.people_mut_for_tests().polities[pi];
            p.gathering = None;
            let l = p.laws.iter_mut().find(|l| l.id == law).expect("the law");
            l.status = LawStatus::InForce;
            l.outcome = Some(Outcome::Passed);
        }
        sim.with_ctx_for_tests(|pop, ctx| pop.begin_public_raise_for_tests(ctx, pi, law));
        law
    };
    let site = |sim: &Sim, law: PermanentId| {
        sim.people().polities[pi]
            .raise_site_of(law)
            .expect("its site")
    };
    // What `holder` makes of `to` in `domain`: good and bad evidence.
    let seen = |sim: &Sim, holder: PermanentId, to: PermanentId, domain: Domain| {
        sim.people()
            .ties
            .of(holder)
            .iter()
            .find(|t| t.to == to)
            .map_or((0.0, 0.0), |t| {
                (t.good[domain.index()], t.bad[domain.index()])
            })
    };
    let user = households[2].1[0];
    // A storehouse is proposed, laid out and begun: its dates are kept.
    let store = raise(
        &mut sim,
        template("core:policy/yourselves_raise_a_storehouse"),
    );
    let s = site(&sim, store);
    assert_eq!(s.begun, Some(sim.now()));
    assert_eq!(
        (s.last_work, s.first_use, s.finished, s.stalled),
        (None, None, None, None)
    );
    let building = s.building.expect("laid out");
    // Two households give their shares; the store moves in the day it is finished.
    let (a, b) = (households[0].0, households[1].0);
    let before = (
        seen(&sim, user, sponsor, Domain::Counsel),
        seen(&sim, user, households[1].1[0], Domain::Provision),
        seen(&sim, user, households[1].1[0], Domain::Craft),
    );
    for turn in 0..4000 {
        if site(&sim, store).finished.is_some() {
            break;
        }
        let who = if turn % 2 == 0 { a } else { b };
        sim.with_ctx_for_tests(|pop, ctx| pop.raise_work_for_tests(ctx, who, building, 4.0));
    }
    let s = site(&sim, store);
    assert_eq!(s.finished, Some(sim.now()));
    assert_eq!(s.first_use, s.finished);
    assert_eq!(s.last_work, Some(sim.now()));
    // Its users see its sponsor's counsel borne out, and who gave their share and built it.
    let elder_b = sim
        .people()
        .household(b)
        .and_then(|x| x.members.first().copied())
        .expect("b's elder");
    let after = (
        seen(&sim, user, sponsor, Domain::Counsel),
        seen(&sim, user, elder_b, Domain::Provision),
        seen(&sim, user, elder_b, Domain::Craft),
    );
    assert!(after.0.0 > before.0.0, "{before:?} {after:?}");
    assert!(after.1.0 > before.1.0, "{before:?} {after:?}");
    assert!(after.2.0 > before.2.0, "{before:?} {after:?}");
    let words = {
        let p = &sim.people().polities[pi];
        let law = p
            .laws
            .iter()
            .find(|l| l.id == store)
            .expect("the law")
            .clone();
        p.words_of(&law, &sim.rules().catalog.policies, &|_| String::new())
    };
    assert!(words.ends_with("; finished"), "{words}");
    // A hall is begun, worked on once, then left: after a season it stands stalled, and its users
    // see its sponsor's counsel come to nothing.
    let hall = raise(&mut sim, template("core:policy/yourselves_roof_a_hall"));
    let hall_building = site(&sim, hall).building.expect("laid out");
    sim.with_ctx_for_tests(|pop, ctx| pop.raise_work_for_tests(ctx, a, hall_building, 4.0));
    let counsel = seen(&sim, user, sponsor, Domain::Counsel);
    sim.with_ctx_for_tests(|pop, ctx| pop.look_over_raisings_for_tests(ctx, pi));
    assert_eq!(site(&sim, hall).stalled, None, "worked on today");
    // Its last work set back to the world's first day, and nobody knowing of the law to work on
    // it, the monthly look over finds it stalled once a season has passed.
    {
        let p = &mut sim.people_mut_for_tests().polities[pi];
        let s = p.raises.iter_mut().find(|s| s.0 == hall).expect("its site");
        s.1.last_work = Some(SimTime::from_minutes(0));
        let l = p.laws.iter_mut().find(|l| l.id == hall).expect("the law");
        l.known.clear();
    }
    let days = 121 - sim.now().day_index();
    sim.advance_minutes((days + 31) * DAY).expect("advances");
    let s = site(&sim, hall);
    assert!(
        s.stalled
            .is_some_and(|t| t.day_index() >= civ_agents::population::STALL_DAYS),
        "{s:?}"
    );
    let stalled = seen(&sim, user, sponsor, Domain::Counsel);
    assert!(stalled.1 > counsel.1, "{counsel:?} {stalled:?}");
    // The observer reads each raising as a sentence, newest first: why it was raised, who put it
    // forward, the hours given and by how many households, and how it ended.
    let projects = civ_sim::frames::government::project_words(&sim, &sim.people().polities[pi]);
    assert_eq!(projects.len(), 2, "{projects:?}");
    assert!(
        projects[0].starts_with("A hall raised together because"),
        "{projects:?}"
    );
    assert!(projects[0].contains("stalled since"), "{projects:?}");
    assert!(projects[0].contains("given by 1 household"), "{projects:?}");
    assert!(
        projects[1].starts_with("A granary raised together because"),
        "{projects:?}"
    );
    assert!(
        projects[1].contains("given by 2 households"),
        "{projects:?}"
    );
    assert!(projects[1].contains("; finished on "), "{projects:?}");
    assert!(
        projects
            .iter()
            .all(|w| w.contains("proposed by ") && w.ends_with('.'))
    );
    // The dates survive a save.
    let loaded = reloaded(&mut sim);
    assert_eq!(
        loaded.people().polities[pi].raises,
        sim.people().polities[pi].raises
    );
}
