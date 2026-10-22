//! Batches that run unattended: a charcoal burn (M10a slice DP; ADR-0033 §2).
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
            name: "Charwood".to_owned(),
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
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "burn").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    same(sim, &loaded);
    loaded
}

/// The household's stores as they stand now, and its batches.
fn stores(sim: &Sim, hh: PermanentId) -> (Vec<f64>, Vec<civ_agents::person::Batch>) {
    let x = sim.people().household(hh).expect("a household");
    let rules = sim.rules();
    (
        civ_agents::population::stores_now(x, sim.now(), &rules.people, &rules.catalog.goods),
        x.batches.clone(),
    )
}

#[test]
fn a_burn_takes_its_wood_when_stacked_and_gives_its_charcoal_when_opened_after_its_days() {
    let mut sim = village(3);
    let catalog = &sim.rules().catalog;
    let index = |list: Vec<&str>, id: &str| list.iter().position(|&x| x == id).expect(id);
    let act = index(
        catalog.activities.iter().map(|a| a.id.as_str()).collect(),
        "core:activity/burn_charcoal",
    );
    let goods: Vec<&str> = catalog.goods.iter().map(|g| g.id.as_str()).collect();
    let (wood, charcoal) = (
        index(goods.clone(), "core:good/firewood"),
        index(goods, "core:good/charcoal"),
    );
    let days = catalog.recipes[catalog.activities[act].recipe.expect("a recipe")].elapsed_days;
    // An adult of some household, with wood at home.
    let now = sim.now();
    let (hh, who) = sim
        .people()
        .households
        .iter()
        .find_map(|(_, h)| {
            let adult = h.members.iter().copied().find(|&m| {
                sim.people()
                    .person(m)
                    .is_some_and(|p| p.age_years(now) >= 20.0)
            })?;
            Some((h.id, adult))
        })
        .expect("a household with an adult");
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == hh {
            x.stores[wood] += 400.0;
        }
    }
    let (before, none) = stores(&sim, hh);
    assert!(none.is_empty());
    // Stacking a mound takes its wood, five kilograms a kilogram of charcoal, and gives nothing yet.
    sim.with_ctx_for_tests(|pop, ctx| pop.make_work_for_tests(ctx, who, act, 240));
    let (after, batches) = stores(&sim, hh);
    assert_eq!(batches.len(), 1, "one burn under way");
    let burn = batches[0].clone();
    assert!(burn.units > 0.0);
    assert_eq!(burn.ends_day, now.day_index() + days.ceil() as i64);
    assert!(
        (before[wood] - after[wood] - 5.0 * burn.units).abs() < 1e-6,
        "{} - {} for {} units",
        before[wood],
        after[wood],
        burn.units
    );
    assert_eq!(
        after[charcoal], before[charcoal],
        "nothing until it is opened"
    );
    // While it burns another session does nothing: no second mound, no wood taken.
    sim.with_ctx_for_tests(|pop, ctx| pop.make_work_for_tests(ctx, who, act, 240));
    let (again, still) = stores(&sim, hh);
    assert_eq!(still, batches);
    assert_eq!(again[wood], after[wood]);
    // It is saved as it is and the world goes on the same.
    let mut sim = reloaded(&mut sim);
    assert_eq!(stores(&sim, hh).1, batches);
    // Once its days are past, a session opens it: its charcoal, exactly, and a new mound.
    sim.advance_minutes(days.ceil() as i64 * DAY)
        .expect("advances");
    let (waiting, _) = stores(&sim, hh);
    sim.with_ctx_for_tests(|pop, ctx| pop.make_work_for_tests(ctx, who, act, 240));
    let (opened, next) = stores(&sim, hh);
    assert!(
        (opened[charcoal] - waiting[charcoal] - burn.units).abs() < 1e-9,
        "{} then {}, a burn of {}",
        waiting[charcoal],
        opened[charcoal],
        burn.units
    );
    assert!(next.iter().all(|b| b.ends_day > burn.ends_day));
}
