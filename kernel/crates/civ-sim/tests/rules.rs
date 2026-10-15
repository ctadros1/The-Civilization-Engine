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
    let mut homes: Vec<(PermanentId, PermanentId)> = Vec::new();
    for (k, &h) in owners.iter().take(4).enumerate() {
        let at = (start.0 + 20.0 + k as f32 * (span + 1.5), start.1);
        let id = sim
            .place_building_for_tests(h, design(at), civ_grammar::Stage::ALL.len() as u8)
            .expect("placed");
        homes.push((id, h));
    }
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
                && l.kind == PolicyKind::BuildingRule
            {
                return Some(l);
            }
        }
        None
    };
    if !sim.people().people.iter().any(|(_, p)| p.knows(firing)) {
        assert!(review(&mut sim).is_none(), "nobody fires tile: no rule");
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
            sim.with_ctx_for_tests(|pop, ctx| pop.roof_rule_kept_for_tests(ctx, h))
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
