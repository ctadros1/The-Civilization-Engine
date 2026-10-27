//! Acceptance of a money (M10d slice EF; ADR-0035 §8, §12): what a money is worth to a household
//! from its use as money is kept in a table built at midnight, not scanned in each decision, and
//! its onward worth falls with the sellers of what it wants who do not take it.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::market::{Market, Offer};
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
            name: "Money".to_owned(),
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

/// A household with a settlement, another household, and the settlement.
fn pair(sim: &Sim) -> (PermanentId, PermanentId, PermanentId) {
    let (hh, settlement) = sim
        .people()
        .households
        .iter()
        .map(|(_, x)| x)
        .find_map(|x| x.settlement.map(|s| (x.id, s)))
        .expect("a household");
    let other = sim
        .people()
        .households
        .iter()
        .map(|(_, x)| x)
        .find(|x| x.id != hh && x.settlement == Some(settlement))
        .map(|x| x.id)
        .expect("a neighbour");
    (hh, other, settlement)
}

/// `hh` remembers paying 2 kg of copper for 100 kg of grain from `seller` now.
fn bought_with_copper(
    sim: &mut Sim,
    hh: PermanentId,
    seller: PermanentId,
    settlement: PermanentId,
) {
    let [copper, grain] = ["copper", "grain"].map(|g| good(sim, &format!("core:good/{g}")));
    let n = sim.rules().catalog.goods.len();
    let now = sim.now();
    let pop = sim.people_mut_for_tests();
    if !pop.markets.iter().any(|m| m.settlement == settlement) {
        pop.markets
            .push(Market::new(settlement, n, now.day_index()));
    }
    let market = pop
        .markets
        .iter_mut()
        .find(|m| m.settlement == settlement)
        .expect("market");
    market.recent.push_back(civ_agents::ledger::Trade {
        at: now,
        seller,
        buyer: hh,
        good: grain as u16,
        units: 100.0,
        payment: copper as u16,
        paid: 2.0,
        channel: civ_agents::ledger::Channel::Sale,
        from: None,
    });
}

fn worth(sim: &mut Sim, hh: PermanentId, p: usize, held: f64) -> Option<f64> {
    sim.with_ctx_for_tests(|pop, ctx| pop.pay_worth_for_tests(ctx, hh, p, held))
}

#[test]
fn what_a_money_bought_counts_from_the_next_midnight() {
    let mut sim = world(3);
    let copper = good(&sim, "core:good/copper");
    let grain = good(&sim, "core:good/grain");
    let (hh, seller, settlement) = pair(&sim);
    bought_with_copper(&mut sim, hh, seller, settlement);
    // Bought with today, it is not yet in the table: worth nothing to one who cannot use it.
    assert_eq!(worth(&mut sim, hh, copper, 0.0), None);
    assert!(sim.people().money.is_empty());
    // At the next midnight the table holds the purchase, a day faded.
    let m = sim.now().minutes();
    sim.advance_minutes(DAY - m.rem_euclid(DAY))
        .expect("advances");
    let b = sim
        .people()
        .money
        .iter()
        .find(|b| b.household == hh && usize::from(b.good) == copper)
        .copied()
        .expect("a belief of copper");
    let memory = sim.rules().people.market.memory_days;
    assert!(
        (b.paid - 2.0 * 0.5f64.powf(1.0 / memory)).abs() < 1e-9,
        "{} paid",
        b.paid
    );
    assert_eq!((b.taken, b.refused), (0.0, 0.0));
    let grain_h = worth(&mut sim, hh, grain, 0.0).expect("grain has a cost");
    // A kilogram bought 50 kg of grain, less the tenth of an hour of weighing it.
    let w = worth(&mut sim, hh, copper, 0.0).expect("worth what it bought");
    assert!(
        (w - (grain_h * 50.0 - 0.1)).abs() < 1e-6 * grain_h.max(1.0) * 50.0,
        "{w} against {grain_h}"
    );
}

#[test]
fn sellers_who_do_not_take_a_money_lower_its_worth_and_beliefs_save_exactly() {
    let mut sim = world(3);
    let [copper, axe, copper_axe, firewood] =
        ["copper", "axe", "copper_axe", "firewood"].map(|g| good(&sim, &format!("core:good/{g}")));
    let (hh, seller, settlement) = pair(&sim);
    let other = sim
        .people()
        .households
        .iter()
        .map(|(_, x)| x)
        .find(|x| x.id != hh && x.id != seller && x.settlement == Some(settlement))
        .map(|x| x.id)
        .expect("a third household");
    bought_with_copper(&mut sim, hh, seller, settlement);
    sim.with_ctx_for_tests(|pop, ctx| pop.money_day_for_tests(ctx));
    let before = worth(&mut sim, hh, copper, 0.0).expect("worth what it bought");
    // It holds copper it has no use for and has lost its axes; of the two neighbours selling an
    // axe, one takes copper and one does not.
    let row = |payment: usize| Offer {
        good: axe as u16,
        payment: payment as u16,
        price: 1.0,
        units: 100.0,
        ask_h: 1.0,
    };
    let pop = sim.people_mut_for_tests();
    for f in &mut pop.firms {
        f.offers.clear();
    }
    for (_, x) in pop.households.iter_mut() {
        let id = x.id;
        x.offers = if id == seller {
            vec![row(copper)]
        } else if id == other {
            vec![row(firewood)]
        } else {
            Vec::new()
        };
        if id == hh {
            x.stores[axe] = 0.0;
            x.stores[copper_axe] = 0.0;
            x.stores[copper] = 5.0;
        }
    }
    sim.with_ctx_for_tests(|pop, ctx| pop.money_review_for_tests(ctx, hh));
    let b = sim
        .people()
        .money
        .iter()
        .find(|b| b.household == hh && usize::from(b.good) == copper)
        .copied()
        .expect("a belief of copper");
    assert_eq!((b.taken, b.refused), (1.0, 1.0), "one took it, one did not");
    assert!((b.acceptance() - 0.5).abs() < 1e-12);
    let after = worth(&mut sim, hh, copper, 0.0).expect("still worth something");
    assert!(
        (after - ((before + 0.1) * 0.5 - 0.1)).abs() < 1e-9,
        "{after} against {before}"
    );
    // A household that holds none counts nothing.
    sim.with_ctx_for_tests(|pop, ctx| pop.money_review_for_tests(ctx, other));
    assert!(sim.people().money.iter().all(|b| b.household != other));
    // Saved and loaded, the beliefs are the same, and so is the world after.
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(&mut sim, &saves, commons_persist::SaveKind::Manual, "money").expect("saves");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loaded.people().money, sim.people().money);
    let same = |x: &Sim, y: &Sim| {
        let (x, y) = (persist::encode_sections(x), persist::encode_sections(y));
        assert_eq!(x.len(), y.len());
        for (p, q) in x.iter().zip(&y) {
            assert!(p.bytes == q.bytes, "section `{}` differs", p.tag);
        }
    };
    same(&sim, &loaded);
    sim.advance_minutes(2 * DAY).expect("lives");
    loaded.advance_minutes(2 * DAY).expect("lives");
    same(&sim, &loaded);
}
