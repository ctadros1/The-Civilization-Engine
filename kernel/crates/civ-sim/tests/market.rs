//! Exchange (slice I, ADR-0006 §3-4): a household offers what it can spare, and a neighbour that
//! needs it and holds what the seller wants buys it at the posted terms, through the ledger; the
//! market remembers it across a save and load.

use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

use civ_agents::ledger::Channel;
use civ_agents::market::{Market, Offer};
use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_schema::{flatbuffers, wire};
use civ_sim::{NewWorld, Sim, frames, persist};
use commons_persist::{SaveDir, SaveKind};

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

fn new_world(seed: u64) -> Sim {
    Sim::create(
        &NewWorld {
            name: "Market".to_owned(),
            seed,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 512,
            band_size: 0,
            neighbours: Vec::new(),
            neighbours_known: false,
            regime_id: String::new(),
        },
        content(),
        &mut |_| {},
        &AtomicBool::new(false),
    )
    .expect("generates")
}

/// Sets a household's stores of the goods given by content id.
fn set(sim: &mut Sim, household: PermanentId, kg: &[(&str, f64)]) {
    let goods = sim.rules().catalog.goods.clone();
    let pop = sim.people_mut_for_tests();
    let h = pop
        .households
        .iter_mut()
        .map(|(_, h)| h)
        .find(|h| h.id == household)
        .expect("household");
    for &(id, v) in kg {
        let g = goods.iter().position(|d| d.id == id).expect(id);
        h.stores[g] = v;
    }
}

#[test]
fn a_household_short_of_a_tool_buys_one_from_a_neighbour_that_can_spare_it() {
    let mut sim = new_world(3);
    sim.advance_minutes(24 * 60).expect("advances");
    let goods = sim.rules().catalog.goods.clone();
    let good = |id: &str| goods.iter().position(|d| d.id == id).expect(id);
    let (sickle, grain) = (good("core:good/sickle"), good("core:good/grain"));
    let mut families: Vec<(usize, PermanentId)> = sim
        .people()
        .households
        .iter()
        .map(|(_, h)| (h.members.len(), h.id))
        .collect();
    families.sort_unstable_by(|a, b| b.cmp(a));
    let (seller, buyer) = (families[0].1, families[1].1);
    // The seller has sickles to spare and little food. Once it has reviewed what it offers, the
    // buyer has no sickle, nothing to make one from, and grain to spare.
    set(
        &mut sim,
        seller,
        &[
            ("core:good/sickle", 14.0),
            ("core:good/grain", 0.0),
            ("core:good/provisions", 40.0),
        ],
    );
    let offers_sickles = |sim: &Sim| {
        sim.people()
            .household(seller)
            .is_some_and(|h| h.offers.iter().any(|o| usize::from(o.good) == sickle))
    };
    for _ in 0..8 {
        if offers_sickles(&sim) {
            break;
        }
        sim.advance_minutes(24 * 60).expect("advances");
    }
    assert!(offers_sickles(&sim), "the seller offers its spare sickles");
    let terms = sim
        .people()
        .household(seller)
        .expect("seller")
        .offers
        .clone();
    assert!(
        terms
            .iter()
            .any(|o| usize::from(o.good) == sickle && usize::from(o.payment) == grain),
        "for grain, which it is short of: {terms:?}"
    );
    set(
        &mut sim,
        buyer,
        &[
            ("core:good/sickle", 0.0),
            ("core:good/toolstone", 0.0),
            ("core:good/timber", 0.0),
            ("core:good/grain", 3000.0),
        ],
    );
    let mut traded = false;
    for _ in 0..10 {
        sim.advance_minutes(24 * 60).expect("advances");
        let sold = sim.people().markets.iter().any(|m| {
            m.recent
                .iter()
                .any(|t| usize::from(t.good) == sickle && t.buyer == buyer)
        });
        if sold && sim.people().transfers.get(Channel::Barter, sickle) > 0.0 {
            traded = true;
            break;
        }
    }
    assert!(traded, "no sickle changed hands");
    let pop = sim.people();
    let settlement = pop
        .household(seller)
        .and_then(|h| h.settlement)
        .expect("settled");
    let market = pop.market(settlement).expect("a market");
    // Other households may buy a sickle too; this is the one the buyer made.
    let trade = market
        .recent
        .iter()
        .find(|t| usize::from(t.good) == sickle && t.buyer == buyer)
        .expect("the trade is remembered");
    assert_eq!((trade.seller, trade.buyer), (seller, buyer));
    // Paid in one of the goods the seller asked for (grain, or something else it is short of
    // that costs the buyer less).
    assert!(
        terms
            .iter()
            .any(|o| usize::from(o.good) == sickle && o.payment == trade.payment),
        "paid in what the seller wants: {trade:?}, terms {terms:?}"
    );
    assert!(trade.paid > 0.0 && trade.units >= 0.5);
    let bought = pop.household(buyer).expect("buyer").stores[sickle];
    assert!(bought > 0.0, "the buyer holds what it bought");
    assert!(
        pop.household(seller).expect("seller").stores[usize::from(trade.payment)] > 0.0,
        "the seller holds the payment"
    );

    // The observer sees the trade, the terms and the price history.
    let payload = frames::markets::markets_response(&sim);
    let response = flatbuffers::root::<wire::Response>(&payload).expect("a response");
    let answer = response.body_as_markets().expect("markets");
    assert_eq!(answer.rev(), frames::markets::markets_rev(&sim));
    assert_ne!(answer.rev(), 0);
    let info = answer
        .markets()
        .expect("a list")
        .iter()
        .find(|m| m.settlement() == settlement.get())
        .expect("the settlement's market");
    assert!(!info.summary().expect("a summary").is_empty());
    let shown = info
        .recent()
        .expect("trades")
        .iter()
        .find(|t| usize::from(t.good()) == sickle && t.buyer() == buyer.get())
        .expect("the trade is listed");
    let text = shown.text().expect("words");
    assert!(text.contains("sickle") && text.contains(" sold "), "{text}");
    let line = info
        .goods()
        .expect("goods")
        .iter()
        .find(|g| usize::from(g.good()) == sickle)
        .expect("a line for sickles");
    assert!(line.sold() > 0.0);
    // The last sickle sold (perhaps by another household since) and what it was paid in.
    assert!(line.last_payment() >= 0 && line.last_price() > 0.0);
    assert!(
        info.history()
            .expect("history")
            .iter()
            .any(|h| usize::from(h.good()) == sickle && h.trades() >= 1)
    );

    // The market and the terms posted survive a save and load.
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves = SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("save dir");
    let saved = persist::save(&mut sim, &saves, SaveKind::Manual, "market").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(markets(&loaded), markets(&sim));
    assert_eq!(offers(&loaded), offers(&sim));
    assert!(offers(&sim).iter().any(|(_, o)| !o.is_empty()));
}

fn markets(sim: &Sim) -> Vec<Market> {
    let mut out = sim.people().markets.clone();
    out.sort_by_key(|m| m.settlement);
    out
}

fn offers(sim: &Sim) -> Vec<(PermanentId, Vec<Offer>)> {
    let mut out: Vec<_> = sim
        .people()
        .households
        .iter()
        .map(|(_, h)| (h.id, h.offers.clone()))
        .collect();
    out.sort_by_key(|o| o.0);
    out
}
