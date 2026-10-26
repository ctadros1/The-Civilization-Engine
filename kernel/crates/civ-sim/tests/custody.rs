//! Custody (M10d slice EG, second part; ADR-0027 §4): a household with no roof of its own leaves
//! what keeps better under one with a neighbour who has room, keeps some days of its food in hand,
//! pays a keeper who is not close kin a share each month, and takes it all back once it has a
//! roof; the goods are the depositor's throughout and are conserved.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::ledger::Channel;
use civ_content::ContentRegistry;
use civ_core::PermanentId;
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
            name: "Custody".to_owned(),
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

/// A world where household `d` has no roof of its own and 600 kg of provisions, and `k`, of its
/// settlement and not its close kin, alone has a roof with room, on `d`'s review day.
fn scene() -> (Sim, PermanentId, PermanentId) {
    let mut sim = world(3);
    sim.advance_minutes(DAY).expect("lives");
    let (d, settlement) = sim
        .people()
        .households
        .iter()
        .map(|(_, x)| x)
        .filter(|x| !x.members.is_empty())
        .find_map(|x| x.settlement.map(|s| (x.id, s)))
        .expect("a household");
    let kin = sim.people().kin_households_for_tests(d);
    let k = sim
        .people()
        .households
        .iter()
        .map(|(_, x)| x)
        .find(|x| {
            x.id != d
                && !x.members.is_empty()
                && x.settlement == Some(settlement)
                && !kin.contains(&x.id)
        })
        .map(|x| x.id)
        .expect("a neighbour who is not close kin");
    let review = i64::from(sim.rules().people.market.review_days.max(1));
    while (d.get() as i64 + sim.now().day_index()).rem_euclid(review) != 0 {
        sim.advance_minutes(DAY).expect("lives");
    }
    let provisions = good(&sim, "core:good/provisions");
    let factor = sim.rules().people.household.raised_store_factor;
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.settlement != Some(settlement) {
            continue;
        }
        x.sheltered = x.id == k;
        x.keeping = civ_agents::person::Keeping {
            raised_kg: 0.0,
            roofed_kg: if x.id == k { 100_000.0 } else { 0.0 },
            raised_factor: factor,
        };
        if x.id == d {
            x.stores[provisions] = 600.0;
        }
    }
    (sim, d, k)
}

#[test]
fn a_household_with_no_roof_leaves_its_goods_with_a_neighbour_and_takes_them_back() {
    let (mut sim, d, k) = scene();
    let provisions = good(&sim, "core:good/provisions");
    // Its provisions as they are now, spoiled to this minute.
    let before = {
        let x = sim.people().household(d).expect("household");
        let rules = sim.rules();
        civ_agents::population::stores_now(x, sim.now(), &rules.people, &rules.catalog.goods)
            [provisions]
    };
    let keep_back = {
        let x = sim.people().household(d).expect("household");
        let need = x.members.len() as f64 * sim.rules().people.household.daily_kcal_per_person;
        sim.rules().people.market.custody_keep_days * need
    };
    sim.with_ctx_for_tests(|pop, ctx| pop.custody_day_for_tests(ctx));
    let (left, _) = sim.people().custody_for_tests(d);
    let (_, keeps) = sim.people().custody_for_tests(k);
    assert!(left > 100.0, "{left} kg left with a neighbour");
    assert!((left - keeps).abs() < 1e-9, "{left} against {keeps}");
    assert!(sim.people().transfers.get(Channel::Custody, provisions) > 100.0);
    // It kept its days of food in hand.
    let x = sim.people().household(d).expect("household");
    let food = civ_agents::person::stock_kcal(&x.stores, &sim.rules().catalog.goods);
    assert!(
        (food - keep_back).abs() <= 0.01 * keep_back,
        "{food} kcal in hand against {keep_back}"
    );
    // The goods are the same goods, held apart.
    let mine = |sim: &Sim| sim.people().household(d).expect("household").stores[provisions];
    let kept = sim.people().kept[0].stores[provisions];
    assert!(
        (mine(&sim) + kept - before).abs() < 1e-6,
        "{} + {kept} against {before}",
        mine(&sim)
    );
    // A month on, the keeper, not close kin, takes its share.
    let day = sim.now().day_index();
    sim.people_mut_for_tests().kept[0].fee_day = day - 30;
    // The keeper's provisions as they are now, spoiled to this minute.
    let held = |sim: &Sim| {
        let x = sim.people().household(k).expect("keeper");
        let rules = sim.rules();
        civ_agents::population::stores_now(x, sim.now(), &rules.people, &rules.catalog.goods)
            [provisions]
    };
    let keeper_before = held(&sim);
    sim.with_ctx_for_tests(|pop, ctx| pop.custody_day_for_tests(ctx));
    let fee = sim.rules().people.market.custody_fee_share;
    assert!(
        (held(&sim) - keeper_before - kept * fee).abs() < 1e-6,
        "{} against {}",
        held(&sim) - keeper_before,
        kept * fee
    );
    // Under a roof of its own with no room for them, it leaves them where they are.
    let factor = sim.rules().people.household.raised_store_factor;
    let roof = |sim: &mut Sim, room: f64| {
        for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
            if x.id == d {
                x.sheltered = true;
                x.keeping = civ_agents::person::Keeping {
                    raised_kg: 0.0,
                    roofed_kg: room,
                    raised_factor: factor,
                };
            }
        }
    };
    roof(&mut sim, 0.0);
    sim.with_ctx_for_tests(|pop, ctx| pop.custody_day_for_tests(ctx));
    assert!(sim.people().custody_for_tests(d).0 > 100.0, "still kept");
    // With room for them all, it takes everything back.
    roof(&mut sim, 100_000.0);
    sim.with_ctx_for_tests(|pop, ctx| pop.custody_day_for_tests(ctx));
    assert_eq!(sim.people().custody_for_tests(d), (0.0, 0.0));
    assert!(sim.people().kept.is_empty());
    assert!(
        (mine(&sim) - (before - kept * fee)).abs() < 1e-6,
        "{} against {}",
        mine(&sim),
        before - kept * fee
    );
}

#[test]
fn goods_kept_for_another_save_and_load_exactly() {
    let (mut sim, d, _) = scene();
    sim.with_ctx_for_tests(|pop, ctx| pop.custody_day_for_tests(ctx));
    assert!(sim.people().custody_for_tests(d).0 > 100.0);
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(&mut sim, &saves, commons_persist::SaveKind::Manual, "kept").expect("saves");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
    // As saved: its counters are not (as a load in transit's are not).
    let saved_of = |sim: &Sim| {
        sim.people()
            .kept
            .iter()
            .map(|k| {
                (
                    k.id,
                    k.depositor,
                    k.keeper,
                    k.since,
                    k.fee_day,
                    k.stores.clone(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(saved_of(&loaded), saved_of(&sim));
    let same = |x: &Sim, y: &Sim, when: &str| {
        let (x, y) = (persist::encode_sections(x), persist::encode_sections(y));
        assert_eq!(x.len(), y.len());
        for (p, q) in x.iter().zip(&y) {
            assert!(p.bytes == q.bytes, "section `{}` differs {when}", p.tag);
        }
    };
    same(&sim, &loaded, "on loading");
    // The shelter the scene set by hand is derived, not saved: both as the land has it.
    sim.with_ctx_for_tests(|pop, ctx| pop.derive_shelter(ctx.land, ctx.catalog, ctx.params));
    sim.advance_minutes(2 * DAY).expect("lives");
    loaded.advance_minutes(2 * DAY).expect("lives");
    same(&sim, &loaded, "two days on");
}
