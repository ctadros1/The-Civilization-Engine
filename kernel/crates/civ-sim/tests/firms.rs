//! Firms (slice J, ADR-0006 §5): a household that makes a tool others want, and can make it for
//! fewer hours than they could, sets up a workshop for it. The workshop holds what it makes, its
//! owners put in what the work uses, it sells through the ledger and keeps books; goods stay
//! accounted for, observers see it, it survives a save and load, and it is given up when it sells
//! nothing for long.

use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

use civ_agents::firm::{BookKind, Exit};
use civ_agents::history::ChronicleKind;
use civ_agents::market::Market;
use civ_agents::{Population, population};
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
            name: "Workshops".to_owned(),
            seed,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 512,
            band_size: 0,
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

/// Sets the level of a skill of everyone in a household.
fn set_skill(sim: &mut Sim, household: PermanentId, skill: usize, level: f64) {
    let pop = sim.people_mut_for_tests();
    for (_, p) in pop.people.iter_mut() {
        if p.household == household {
            p.set_skill(skill, level);
        }
    }
}

fn held(pop: &Population) -> (Vec<f64>, civ_agents::person::Flows) {
    (pop.goods_held(), pop.flows())
}

#[test]
fn a_skilled_household_sets_up_a_workshop_and_sells_what_it_makes() {
    let mut sim = new_world(3);
    sim.advance_minutes(24 * 60).expect("advances");
    let rules = sim.rules().clone();
    let goods = &rules.catalog.goods;
    let good = |id: &str| goods.iter().position(|d| d.id == id).expect(id);
    let sickle = good("core:good/sickle");
    let knapping = rules
        .catalog
        .skills
        .iter()
        .position(|k| k.id == "core:skill/knapping")
        .expect("knapping");
    let mut families: Vec<(usize, PermanentId)> = sim
        .people()
        .households
        .iter()
        .map(|(_, h)| (h.members.len(), h.id))
        .collect();
    families.sort_unstable_by(|a, b| b.cmp(a));
    let maker = families[0].1;
    // The maker: master knappers with flint and wood to spare and the sickles they need
    // themselves, no more.
    let now = sim.now();
    let ages: Vec<f64> = sim
        .people()
        .household(maker)
        .expect("maker")
        .members
        .iter()
        .filter_map(|m| sim.people().person(*m))
        .map(|p| p.age_years(now))
        .collect();
    let wants = civ_agents::make::tool_wants_for(
        &rules.catalog,
        &ages,
        rules.people.family.independent_age,
    );
    set(
        &mut sim,
        maker,
        &[
            (
                "core:good/sickle",
                wants[sickle] + civ_agents::make::SPARE_TOOL,
            ),
            ("core:good/toolstone", 10.0),
            ("core:good/timber", 20.0),
        ],
    );
    set_skill(&mut sim, maker, knapping, 1.0);
    // Everyone else: no sickle, nothing to make one from, and grain to pay with.
    for &(_, other) in &families[1..] {
        set(
            &mut sim,
            other,
            &[
                ("core:good/sickle", 0.0),
                ("core:good/toolstone", 0.0),
                ("core:good/timber", 0.0),
                ("core:good/grain", 3000.0),
            ],
        );
        set_skill(&mut sim, other, knapping, 0.0);
    }
    let start = held(sim.people());
    let sold = |sim: &Sim| {
        sim.people().firms.iter().any(|f| {
            f.books
                .months
                .iter()
                .any(|m| m.amount(BookKind::Sold, sickle as u16) > 0.0)
        })
    };
    // Buyers keep asking for sickles nobody offers: demand on record (ADR-0006 §4), kept up as
    // if they went on asking, whether or not they would have found the flint to make their own.
    let settlement = sim
        .people()
        .household(maker)
        .and_then(|h| h.settlement)
        .expect("settled");
    let ask = |sim: &mut Sim| {
        let day = sim.now().day_index();
        let goods = sim.rules().catalog.goods.len();
        let pop = sim.people_mut_for_tests();
        if !pop.markets.iter().any(|m| m.settlement == settlement) {
            pop.markets.push(Market::new(settlement, goods, day));
        }
        let m = pop
            .markets
            .iter_mut()
            .find(|m| m.settlement == settlement)
            .expect("a market");
        if m.unmet[sickle] < 2.0 {
            m.record_unmet(
                sickle,
                2.0 - m.unmet[sickle],
                40.0 * (2.0 - m.unmet[sickle]),
            );
        }
    };
    for _ in 0..60 {
        if sold(&sim) {
            break;
        }
        ask(&mut sim);
        sim.advance_minutes(24 * 60).expect("advances");
    }
    let pop = sim.people();
    let firm = pop
        .firms
        .iter()
        .find(|f| f.owner == maker)
        .unwrap_or_else(|| panic!("no workshop; firms {:?}", pop.firms));
    assert!(sold(&sim), "the workshop sold nothing: {:?}", firm.books);
    assert!(firm.is_open());
    assert_eq!(firm.lines, vec![sickle as u16]);
    let opened = pop
        .chronicle
        .iter()
        .find(|e| e.kind == ChronicleKind::WorkshopOpened)
        .expect("the chronicle notes it");
    assert_eq!(opened.firm, Some(firm.id));
    // Its books: what its owners put in and it used, what it made, sold and was paid.
    let total = |kind: BookKind, g: usize| -> f32 {
        firm.books
            .months
            .iter()
            .map(|m| m.amount(kind, g as u16))
            .sum()
    };
    let toolstone = good("core:good/toolstone");
    assert!(total(BookKind::Made, sickle) >= 1.0);
    assert!(total(BookKind::PutIn, toolstone) > 0.0);
    assert_eq!(
        total(BookKind::PutIn, toolstone),
        total(BookKind::Used, toolstone)
    );
    let paid: f32 = (0..goods.len()).map(|g| total(BookKind::Paid, g)).sum();
    assert!(paid > 0.0, "it was paid");
    assert!(firm.books.months.iter().any(|m| m.owner_h > 0.0));
    // The market remembers the workshop as the seller.
    assert_eq!(firm.settlement, Some(settlement));
    let market = pop.market(settlement).expect("a market");
    assert!(
        market
            .recent
            .iter()
            .any(|t| t.seller == firm.id && usize::from(t.good) == sickle)
    );
    // Every good is accounted for, the workshop's included.
    let end = held(pop);
    let gaps = population::unaccounted(goods.len(), (&start.0, &start.1), (&end.0, &end.1));
    assert!(gaps.is_empty(), "unaccounted for: {gaps:?}");

    // The observer sees it: in the list of workshops, on its page with its books, as the seller
    // in its market, and as a link in the chronicle.
    let rev = frames::firms::firms_rev(&sim);
    assert_ne!(rev, 0);
    let payload = frames::firms::firms_response(&sim);
    let list = flatbuffers::root::<wire::Response>(&payload)
        .expect("a response")
        .body_as_firms()
        .expect("workshops");
    assert_eq!(list.rev(), rev);
    let brief = list
        .firms()
        .expect("a list")
        .iter()
        .find(|b| b.id() == firm.id.get())
        .expect("listed");
    assert!(brief.open() && brief.owner() == maker.get());
    let name = brief.name().expect("a name");
    assert!(name.ends_with("'s sickle workshop"), "{name}");
    let record = brief.record().expect("its record");
    assert!(
        record.starts_with("Made ") && !record.ends_with("none."),
        "{record}"
    );
    assert_eq!(
        brief.lines().map(|l| l.iter().collect()),
        Some(vec![sickle as u16])
    );
    let payload = frames::firms::firm_response(&sim, firm.id.get()).expect("its page");
    let page = flatbuffers::root::<wire::Response>(&payload)
        .expect("a response")
        .body_as_firm_info()
        .expect("a page");
    assert_eq!(page.brief().map(|b| b.id()), Some(firm.id.get()));
    let entries = page.entries().expect("its books");
    assert_eq!(entries.len(), firm.books.entries.len());
    let sale = entries
        .iter()
        .find(|e| e.kind() == wire::BookKind::Sold)
        .expect("a sale in its books");
    let text = sale.text().expect("words");
    assert!(text.starts_with("Sold ") && text.contains(" to "), "{text}");
    assert_eq!(
        page.months().map(|m| m.len()),
        Some(firm.books.months.len())
    );
    assert!(frames::firms::firm_response(&sim, 0).is_err());
    let payload = frames::markets::markets_response(&sim);
    let info = flatbuffers::root::<wire::Response>(&payload)
        .expect("a response")
        .body_as_markets()
        .expect("markets")
        .markets()
        .expect("a list")
        .iter()
        .find(|m| m.settlement() == settlement.get())
        .expect("its market");
    let trade = info
        .recent()
        .expect("trades")
        .iter()
        .find(|t| t.seller() == firm.id.get())
        .expect("its sale");
    assert!(trade.seller_firm());
    let text = trade.text().expect("words");
    assert!(text.starts_with(name), "{text}");
    let payload = frames::people::chronicle_response(&sim, 0, 10_000);
    let linked = flatbuffers::root::<wire::Response>(&payload)
        .expect("a response")
        .body_as_chronicle()
        .expect("the chronicle")
        .entries()
        .expect("entries")
        .iter()
        .flat_map(|e| e.spans().into_iter().flatten())
        .any(|x| x.kind() == wire::SpanKind::Firm && x.id() == firm.id.get());
    assert!(linked, "the chronicle links the workshop");

    // It survives a save and load.
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves = SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("save dir");
    let saved = persist::save(&mut sim, &saves, SaveKind::Manual, "firms").expect("saves");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
    let strip = |s: &Sim| {
        s.people()
            .firms
            .iter()
            .map(|f| {
                let mut f = f.clone();
                f.flows = Default::default();
                f
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(strip(&loaded), strip(&sim));

    // Months without a sale and its owners give it up: what it holds goes back to them.
    let id = loaded.people().firms[0].id;
    let long_ago = loaded.now().plus_minutes(-200 * 24 * 60);
    {
        let pop = loaded.people_mut_for_tests();
        let f = pop.firms.iter_mut().find(|f| f.id == id).expect("firm");
        f.last_sale = Some(long_ago);
        f.founded = long_ago;
    }
    for _ in 0..8 {
        loaded.advance_minutes(24 * 60).expect("advances");
    }
    let f = loaded.people().firm(id).expect("kept on record");
    assert_eq!(f.closed.map(|(_, why)| why), Some(Exit::Idle));
    assert!(f.stores.iter().all(|&kg| kg == 0.0), "{:?}", f.stores);
    assert!(f.offers.is_empty());
    assert!(
        loaded
            .people()
            .chronicle
            .iter()
            .any(|e| e.kind == ChronicleKind::WorkshopClosed && e.firm == Some(id))
    );
}

#[test]
fn a_workshop_posts_a_wage_and_pays_those_who_take_the_work() {
    // ADR-0006 §5, research 08-10 §1.3: a workshop with more wanted of it than it holds posts a
    // wage on its own; people of a household short of food take work that pays well enough in
    // grain, and it pays them through the ledger for the time they worked.
    let mut sim = new_world(3);
    sim.advance_minutes(24 * 60).expect("advances");
    let rules = sim.rules().clone();
    let goods = &rules.catalog.goods;
    let good = |id: &str| goods.iter().position(|d| d.id == id).expect(id);
    let (sickle, grain) = (good("core:good/sickle"), good("core:good/grain"));
    let knapping = rules
        .catalog
        .skills
        .iter()
        .position(|k| k.id == "core:skill/knapping")
        .expect("knapping");
    let mut families: Vec<(usize, PermanentId)> = sim
        .people()
        .households
        .iter()
        .map(|(_, h)| (h.members.len(), h.id))
        .collect();
    families.sort_unstable_by(|a, b| b.cmp(a));
    let maker = families[0].1;
    let now = sim.now();
    let ages: Vec<f64> = sim
        .people()
        .household(maker)
        .expect("maker")
        .members
        .iter()
        .filter_map(|m| sim.people().person(*m))
        .map(|p| p.age_years(now))
        .collect();
    let wants = civ_agents::make::tool_wants_for(
        &rules.catalog,
        &ages,
        rules.people.family.independent_age,
    );
    // The workshop's household: master knappers with flint and wood, and grain to pay with.
    set(
        &mut sim,
        maker,
        &[
            (
                "core:good/sickle",
                wants[sickle] + civ_agents::make::SPARE_TOOL,
            ),
            ("core:good/toolstone", 30.0),
            ("core:good/timber", 50.0),
            ("core:good/grain", 4000.0),
        ],
    );
    set_skill(&mut sim, maker, knapping, 1.0);
    // Its neighbours: no sickle, nothing to make one from, and no food.
    for &(_, other) in &families[1..] {
        set(
            &mut sim,
            other,
            &[
                ("core:good/sickle", 0.0),
                ("core:good/toolstone", 0.0),
                ("core:good/timber", 0.0),
                ("core:good/grain", 0.0),
                ("core:good/provisions", 0.0),
            ],
        );
    }
    let settlement = sim
        .people()
        .household(maker)
        .and_then(|h| h.settlement)
        .expect("settled");
    let ask = |sim: &mut Sim| {
        let day = sim.now().day_index();
        let goods = sim.rules().catalog.goods.len();
        let pop = sim.people_mut_for_tests();
        if !pop.markets.iter().any(|m| m.settlement == settlement) {
            pop.markets.push(Market::new(settlement, goods, day));
        }
        let m = pop
            .markets
            .iter_mut()
            .find(|m| m.settlement == settlement)
            .expect("a market");
        if m.unmet[sickle] < 4.0 {
            m.record_unmet(
                sickle,
                4.0 - m.unmet[sickle],
                40.0 * (4.0 - m.unmet[sickle]),
            );
        }
    };
    // Until it posts a wage of its own.
    let posted = |sim: &Sim| {
        sim.people()
            .firms
            .iter()
            .find(|f| f.owner == maker)
            .and_then(|f| f.wage)
            .filter(|w| w.hours >= 1.0)
    };
    for _ in 0..40 {
        if posted(&sim).is_some() {
            break;
        }
        ask(&mut sim);
        sim.advance_minutes(24 * 60).expect("advances");
    }
    let first = posted(&sim).expect("the workshop posted a wage");
    assert!(first.per_hour > 0.0 && first.hour_h > 0.0);
    // Then, paying two kilograms of grain an hour, it finds hands.
    let generous = |sim: &mut Sim| {
        let pop = sim.people_mut_for_tests();
        if let Some(w) = pop
            .firms
            .iter_mut()
            .find(|f| f.owner == maker)
            .and_then(|f| f.wage.as_mut())
        {
            w.pay = grain as u16;
            w.per_hour = 2.0;
            w.hours = w.hours.max(w.taken + 6.0);
        }
    };
    let start = held(sim.people());
    let paid_out = |sim: &Sim| {
        sim.people()
            .firms
            .iter()
            .flat_map(|f| f.books.months.iter())
            .map(|m| m.amount(BookKind::Wages, grain as u16))
            .sum::<f32>()
    };
    for _ in 0..30 {
        if paid_out(&sim) > 0.0 {
            break;
        }
        ask(&mut sim);
        generous(&mut sim);
        sim.advance_minutes(24 * 60).expect("advances");
    }
    let pop = sim.people();
    let firm = pop
        .firms
        .iter()
        .find(|f| f.owner == maker)
        .expect("the workshop");
    let paid = paid_out(&sim);
    let hours: f32 = firm.books.months.iter().map(|m| m.hired_h).sum();
    assert!(
        paid > 0.0,
        "nobody took the work: wage {:?}, books {:?}",
        firm.wage,
        firm.books.months
    );
    assert!(
        (paid - 2.0 * hours).abs() <= 1e-3 * paid.max(1.0),
        "paid {paid} kg for {hours} h"
    );
    let worker = firm
        .books
        .entries
        .iter()
        .find(|e| e.kind == BookKind::Wages)
        .and_then(|e| e.other)
        .expect("a worker's household");
    assert_ne!(worker, maker);
    assert!(pop.transfers.get(civ_agents::Channel::Wage, grain) > 0.0);
    assert!(firm.is_open(), "it could pay what it owed");
    // The observer sees its wage and what it paid.
    let payload = frames::firms::firm_response(&sim, firm.id.get()).expect("its page");
    let page = flatbuffers::root::<wire::Response>(&payload)
        .expect("a response")
        .body_as_firm_info()
        .expect("a page");
    let wage = page.wage().expect("its wage");
    assert_eq!((wage.pay(), wage.per_hour()), (grain as u16, 2.0));
    let text = wage.text().expect("words");
    assert!(text.starts_with("Pays 2.0 kg of grain an hour"), "{text}");
    let wages = page
        .entries()
        .expect("its books")
        .iter()
        .find(|e| e.kind() == wire::BookKind::Wages)
        .expect("wages in its books");
    assert_eq!(wages.other(), worker.get());
    let text = wages.text().expect("words");
    assert!(
        text.starts_with("Paid ") && text.ends_with(" in wages."),
        "{text}"
    );
    assert!(
        page.months()
            .expect("months")
            .iter()
            .any(|m| m.hired_h() > 0.0)
    );
    let end = held(pop);
    let gaps = population::unaccounted(goods.len(), (&start.0, &start.1), (&end.0, &end.1));
    assert!(gaps.is_empty(), "unaccounted for: {gaps:?}");
}
