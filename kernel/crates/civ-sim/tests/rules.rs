//! Rules for building (M6b slice BH; ADR-0022 §12): a fire remembered, a rule proposed by someone
//! who knows how to make what it asks, and a rule kept or broken by the buildings begun under it.
use civ_agents::polity::{IssueKind, LawStatus, PolicyKind};
use civ_content::ContentRegistry;
use civ_core::PermanentId;
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
            name: "Thatchford".to_owned(),
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

fn same(a: &Sim, b: &Sim) {
    let (a, b) = (persist::encode_sections(a), persist::encode_sections(b));
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(&b) {
        assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
    }
}

fn reloaded(sim: &mut Sim) -> Sim {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "rule").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    same(sim, &loaded);
    loaded
}

/// The least gap between the plot `spec` would take and every standing building's plot, metres.
fn nearest_m(sim: &Sim, spec: &civ_grammar::BuildingSpec) -> f64 {
    let catalog = &sim.rules().catalog;
    let def = catalog
        .building_index(&spec.program)
        .map(|i| &catalog.buildings[i])
        .expect("its program");
    let rect = civ_agents::build::plot_rect(spec, def);
    sim.land()
        .buildings
        .iter()
        .filter(|b| b.standing())
        .filter_map(|b| sim.land().plots.iter().find(|p| p.id == b.plot))
        .map(|p| civ_agents::rules::gap_m(&rect, &p.rect))
        .fold(f64::INFINITY, f64::min)
}

#[test]
fn a_fire_remembered_opens_a_rule_that_only_one_who_fires_tile_proposes_and_new_roofs_keep() {
    let mut sim = village(5);
    sim.advance_minutes(DAY).expect("advances");
    // Four households' huts in a row, a metre and a half apart roof to roof, east of the first
    // household's home; the second is set alight and burns out.
    let homes = huts_in_a_row(&mut sim, 4, 1.5);
    let (target, owner) = homes[1];
    let settlement = sim
        .people()
        .household(owner)
        .and_then(|x| x.settlement)
        .expect("settled");
    sim.set_fire(target).expect("set alight");
    for _ in 0..3 {
        sim.advance_minutes(DAY).expect("advances");
    }
    assert!(
        sim.people().fires.list.iter().all(|f| !f.burning()),
        "the fire is out"
    );
    // Its household remembers what it lost, and whoever saw a fire remembers that one.
    let own = sim
        .people()
        .household(owner)
        .expect("the household")
        .fire_losses
        .iter()
        .find(|l| l.building == target)
        .copied()
        .expect("remembered");
    assert!(own.own && own.burnable && own.hours > 0.0, "{own:?}");
    for (_, x) in sim.people().households.iter() {
        for l in &x.fire_losses {
            let fire = sim
                .people()
                .fires
                .list
                .iter()
                .find(|f| f.building == l.building)
                .expect("a fire there");
            assert_eq!(l.own, fire.household == x.id, "{l:?}");
        }
    }
    // A household that remembers a burnt roof expects a rule to bring it something.
    let gains = sim.with_ctx_for_tests(|pop, ctx| {
        pop.site_gains_for_tests(ctx, settlement, PolicyKind::BuildingRule, 0)
    });
    assert!(gains.iter().any(|g| g.1 > 0.0), "{gains:?}");
    // At the polity's review, nobody proposes it while nobody can fire tile.
    sim.people_mut_for_tests().every_adult_deliberates = true;
    let pi = sim
        .people()
        .polities
        .iter()
        .position(|p| p.settlement == settlement)
        .expect("a polity");
    let firing = sim
        .rules()
        .catalog
        .technique_index("core:technique/firing")
        .expect("firing");
    let roofs = sim
        .rules()
        .catalog
        .policies
        .iter()
        .position(|d| d.id == "core:policy/works_wrought_roofs")
        .expect("the roofs rule") as u16;
    // The rule for roofs, put before the gathering, if anyone puts it there within a month of
    // reviews (any other proposal is turned down, so each review is open to it again).
    let review = |sim: &mut Sim| -> Option<civ_agents::polity::Law> {
        // Each sponsor's draw is of the day: try on successive days.
        for _ in 0..30 {
            sim.advance_minutes(DAY).expect("advances");
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
                && l.policy == roofs
            {
                return Some(l);
            }
        }
        None
    };
    if !sim.people().people.iter().any(|(_, p)| p.knows(firing)) {
        assert!(
            review(&mut sim).is_none(),
            "nobody fires tile: no roofs rule"
        );
    }
    // Once the people of the settlement know how, one of them proposes it.
    let people: Vec<PermanentId> = sim
        .people()
        .households
        .iter()
        .filter(|(_, x)| x.settlement == Some(settlement))
        .flat_map(|(_, x)| x.members.clone())
        .collect();
    for m in people {
        sim.introduce_technique(m, "core:technique/firing", false)
            .expect("taught");
    }
    let law = review(&mut sim).expect("a rule proposed");
    assert_eq!(law.issue, IssueKind::Burned);
    assert!(
        sim.people()
            .person(law.sponsor)
            .is_some_and(|p| p.knows(firing)),
        "proposed by one who fires tile"
    );
    let what = civ_sim::frames::word::law_text(&sim, &sim.people().polities[pi], &law);
    assert_eq!(
        what,
        "a rule for building: a new roof within 4 m of another building must not burn, of what \
         is begun from its day"
    );
    // Passed and known to all: a household that keeps it lays out a home whose roof does not
    // burn, or that stands apart from every other building.
    let now = sim.now();
    {
        let pop = sim.people_mut_for_tests();
        let l = pop.polities[pi].law_mut(law.id).expect("the law");
        l.status = LawStatus::InForce;
        l.decided = Some(now);
    }
    let everyone: Vec<PermanentId> = sim.people().people.iter().map(|(_, p)| p.id).collect();
    for p in everyone {
        sim.people_mut_for_tests().polities[pi]
            .law_mut(law.id)
            .expect("the law")
            .learn(p, 0);
    }
    let keeper = homes
        .iter()
        .map(|h| h.1)
        .find(|&h| {
            sim.with_ctx_for_tests(|pop, ctx| pop.rules_kept_for_tests(ctx, h))
                .roof_within
                .is_some()
        })
        .expect("someone keeps it");
    let spec = sim
        .with_ctx_for_tests(|pop, ctx| pop.first_home_for_tests(ctx, keeper))
        .expect("a home laid out");
    assert!(
        civ_agents::rules::roof_unburnt(&spec) || nearest_m(&sim, &spec) >= 4.0,
        "kept: {} m from the nearest",
        nearest_m(&sim, &spec)
    );
    // A building begun among others with a roof that burns is counted as breaking it, by one
    // who knew of it.
    let begun = homes
        .iter()
        .find(|h| h.1 == keeper)
        .map(|h| h.0)
        .expect("its home");
    sim.with_ctx_for_tests(|pop, ctx| pop.count_rule_keeping_for_tests(ctx, keeper, begun));
    let c = sim.people().polities[pi]
        .laws
        .iter()
        .find(|l| l.id == law.id)
        .expect("the law")
        .compliance;
    assert_eq!((c.complied, c.broken_unaware), (0, 0), "{c:?}");
    assert!(c.broken <= 1, "{c:?}");
    // The world saves and loads exactly, remembered fires and all.
    let loaded = reloaded(&mut sim);
    assert_eq!(
        loaded
            .people()
            .household(owner)
            .expect("the household")
            .fire_losses,
        sim.people()
            .household(owner)
            .expect("the household")
            .fire_losses
    );
}

/// `n` households' finished huts in a row, `gap_m` apart roof to roof, 20 m east of the first
/// household's home: each hut and its household.
fn huts_in_a_row(sim: &mut Sim, n: usize, gap_m: f32) -> Vec<(PermanentId, PermanentId)> {
    let catalog = &content().catalog;
    let hut_def = &catalog.buildings[catalog.building_index("core:building/hut").expect("hut")];
    let design = |at: (f32, f32)| {
        civ_agents::build::design_shape(
            hut_def,
            &catalog.goods,
            civ_agents::build::Shape::Round { radius: 310 },
            at,
            None,
        )
        .expect("a hut")
    };
    let span = civ_agents::build::plot_rect(&design((0.0, 0.0)), hut_def).w as f32 / 100.0;
    let mut owners: Vec<PermanentId> = sim
        .people()
        .households
        .iter()
        .filter(|(_, h)| !h.members.is_empty())
        .map(|(_, h)| h.id)
        .collect();
    owners.sort_unstable();
    let start = sim.people().household(owners[0]).expect("a household").home;
    owners
        .iter()
        .take(n)
        .enumerate()
        .map(|(k, &h)| {
            let at = (start.0 + 20.0 + k as f32 * (span + gap_m), start.1);
            let id = sim
                .place_building_for_tests(h, design(at), civ_grammar::Stage::ALL.len() as u8)
                .expect("placed");
            (id, h)
        })
        .collect()
}

/// The template with content id `id`, by index.
fn policy(id: &str) -> u16 {
    content()
        .catalog
        .policies
        .iter()
        .position(|d| d.id == id)
        .expect("the template") as u16
}

/// Polity `pi` of settlement `settlement` reviews on each of up to 30 days until someone puts
/// template `k` before its gathering; any other proposal is turned down, so each review is open
/// to it again.
fn proposed(
    sim: &mut Sim,
    pi: usize,
    settlement: PermanentId,
    k: u16,
) -> Option<civ_agents::polity::Law> {
    for _ in 0..30 {
        sim.advance_minutes(DAY).expect("advances");
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
            && l.policy == k
        {
            return Some(l);
        }
    }
    None
}

/// Puts law `law` of polity `pi` in force today, known to everyone.
fn in_force(sim: &mut Sim, pi: usize, law: PermanentId) {
    let now = sim.now();
    {
        let l = sim.people_mut_for_tests().polities[pi]
            .law_mut(law)
            .expect("the law");
        l.status = LawStatus::InForce;
        l.decided = Some(now);
    }
    let everyone: Vec<PermanentId> = sim.people().people.iter().map(|(_, p)| p.id).collect();
    for p in everyone {
        sim.people_mut_for_tests().polities[pi]
            .law_mut(law)
            .expect("the law")
            .learn(p, 0);
    }
}

#[test]
fn a_building_heard_to_give_way_opens_a_walls_rule_and_a_fire_a_gap_rule_both_kept_or_broken() {
    let mut sim = village(5);
    sim.advance_minutes(DAY).expect("advances");
    let homes = huts_in_a_row(&mut sim, 3, 6.0);
    let (weak, owner) = homes[0];
    let settlement = sim
        .people()
        .household(owner)
        .and_then(|x| x.settlement)
        .expect("settled");
    // Its first hut is so poorly made that its own weight brings it down.
    for b in sim.land_mut_for_tests().buildings.iter_mut() {
        if b.id == weak {
            for c in &mut b.condition {
                c.quality = 0.01;
            }
        }
    }
    sim.advance_minutes(DAY).expect("advances");
    let heard: Vec<(PermanentId, civ_agents::person::GaveWay)> = sim
        .people()
        .households
        .iter()
        .filter(|(_, x)| x.settlement == Some(settlement))
        .flat_map(|(_, x)| x.gave_way.iter().map(move |g| (x.id, *g)))
        .collect();
    assert!(!heard.is_empty(), "it gave way and was heard of");
    for (h, g) in &heard {
        assert_eq!(g.building, weak);
        assert_eq!(g.own, *h == owner);
        assert_eq!(g.wall_kind, 0, "a hut's walls are wattle and daub");
    }
    // The walls rule needs rubble walling, which the founders bring: proposed against it.
    let pi = sim
        .people()
        .polities
        .iter()
        .position(|p| p.settlement == settlement)
        .expect("a polity");
    sim.people_mut_for_tests().every_adult_deliberates = true;
    let walls = proposed(
        &mut sim,
        pi,
        settlement,
        policy("core:policy/works_wrought_walls"),
    )
    .expect("a walls rule proposed");
    assert_eq!(walls.issue, IssueKind::GaveWay);
    assert_eq!(
        civ_sim::frames::word::law_text(&sim, &sim.people().polities[pi], &walls),
        "a rule for building: new walls must be of rubble, mud brick or fired brick, of what is \
         begun from its day"
    );
    // Nothing burned: nobody proposes the gap rule, which answers fire alone.
    let gap = policy("core:policy/works_wrought_spacing");
    assert!(proposed(&mut sim, pi, settlement, gap).is_none());
    // A fire later, and someone does.
    sim.set_fire(homes[2].0).expect("set alight");
    for _ in 0..3 {
        sim.advance_minutes(DAY).expect("advances");
    }
    let spacing = proposed(&mut sim, pi, settlement, gap).expect("a gap rule proposed");
    assert_eq!(spacing.issue, IssueKind::Burned);
    // Both in force: a hut breaks the walls rule, its walls being wattle; one 6 m from every
    // other keeps the gap rule.
    in_force(&mut sim, pi, walls.id);
    in_force(&mut sim, pi, spacing.id);
    let kept = sim.with_ctx_for_tests(|pop, ctx| pop.rules_kept_for_tests(ctx, owner));
    let near = homes[1];
    sim.with_ctx_for_tests(|pop, ctx| pop.count_rule_keeping_for_tests(ctx, near.1, near.0));
    let laws = &sim.people().polities[pi].laws;
    let c = |id: PermanentId| {
        laws.iter()
            .find(|l| l.id == id)
            .expect("the law")
            .compliance
    };
    let (cw, cg) = (c(walls.id), c(spacing.id));
    assert_eq!(cw.complied, 0, "{cw:?}");
    assert_eq!(cw.broken + cw.broken_unaware, 1, "{cw:?}");
    // The second hut stands 6 m from its neighbours: apart, as the gap asks.
    assert_eq!(
        (cg.complied, cg.broken, cg.broken_unaware),
        (1, 0, 0),
        "{cg:?}"
    );
    assert!(kept.gap.is_none_or(|g| g == 3.0), "{kept:?}");
    let reloaded = reloaded(&mut sim);
    assert_eq!(
        reloaded
            .people()
            .household(owner)
            .expect("the household")
            .gave_way,
        sim.people()
            .household(owner)
            .expect("the household")
            .gave_way
    );
}

#[test]
fn a_rule_keeps_a_home_where_fire_is_worked_from_others_homes() {
    // M8c slice CW (ADR-0030 §6): a rule for building over `fire.rs`'s worked fires. In force, it
    // binds only a household that works fire, holding a kiln: its builders keep its next home ten
    // metres from others' homes, and its hut six metres from its neighbours' breaks it; a
    // household that works no fire is not bound.
    let mut sim = village(5);
    sim.advance_minutes(DAY).expect("advances");
    let homes = huts_in_a_row(&mut sim, 3, 6.0);
    let (hut, potter) = homes[0];
    let (other_hut, other) = homes[1];
    let kiln = content()
        .catalog
        .goods
        .iter()
        .position(|g| g.id == "core:good/kiln")
        .expect("a kiln");
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == potter {
            if x.stores.len() <= kiln {
                x.stores.resize(kiln + 1, 0.0);
            }
            x.stores[kiln] = 1.0;
        }
    }
    let settlement = sim
        .people()
        .household(potter)
        .and_then(|x| x.settlement)
        .expect("settled");
    let pi = sim
        .people()
        .polities
        .iter()
        .position(|p| p.settlement == settlement)
        .expect("a polity");
    let sponsor = sim.people().household(other).expect("a household").members[0];
    let k = policy("core:policy/works_wrought_worked_fires");
    let law = sim.with_ctx_for_tests(|pop, ctx| pop.enact_rule_for_tests(ctx, pi, k, sponsor));
    let kept = sim.with_ctx_for_tests(|pop, ctx| pop.rules_kept_for_tests(ctx, potter));
    assert_eq!(kept.fire_from, Some(10.0), "{kept:?}");
    let unbound = sim.with_ctx_for_tests(|pop, ctx| pop.rules_kept_for_tests(ctx, other));
    assert_eq!(unbound.fire_from, None, "no fire worked: {unbound:?}");
    // Its next home stands ten metres from every other household's home.
    let spec = sim
        .with_ctx_for_tests(|pop, ctx| pop.first_home_for_tests(ctx, potter))
        .expect("a home laid out");
    let catalog = &content().catalog;
    let def = &catalog.buildings[catalog.building_index(&spec.program).expect("its program")];
    let rect = civ_agents::build::plot_rect(&spec, def);
    let nearest = sim
        .land()
        .buildings
        .iter()
        .filter(|b| b.household != potter && b.standing())
        .filter(|b| catalog.is_dwelling(&b.spec.program))
        .filter_map(|b| sim.land().plots.iter().find(|p| p.id == b.plot))
        .map(|p| civ_agents::rules::gap_m(&rect, &p.rect))
        .fold(f64::INFINITY, f64::min);
    assert!(nearest >= 10.0, "{nearest} m from another's home");
    // Its hut six metres from its neighbour's breaks it; the neighbour's binds nobody.
    sim.with_ctx_for_tests(|pop, ctx| pop.count_rule_keeping_for_tests(ctx, potter, hut));
    sim.with_ctx_for_tests(|pop, ctx| pop.count_rule_keeping_for_tests(ctx, other, other_hut));
    let c = sim.people().polities[pi]
        .laws
        .iter()
        .find(|l| l.id == law)
        .expect("the law")
        .compliance;
    assert_eq!((c.complied, c.broken + c.broken_unaware), (0, 1), "{c:?}");
}
