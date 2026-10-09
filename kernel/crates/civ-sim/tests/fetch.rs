//! Buying from a neighbour by report (M5b slice AP, ADR-0019 §1–§4): a household knows another
//! settlement's offers only by price reports its members saw or were told, goes to a seller's
//! door by one, buys at the terms the seller posts there now, and the trade is tallied in the
//! seller's market with the buyer's settlement.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::ledger::Trade;
use civ_agents::reports::{Missed, PriceReport, ReportHow};
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

/// Settlements of `band` people each, one per entry of `neighbours` beside the first, that know
/// where each other camped; with a fixed identity, so the world lives the same life every run.
fn world(seed: u64, band: u32, neighbours: &[u32]) -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Neighbours".to_owned(),
            seed,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 768,
            band_size: band,
            neighbours: neighbours.to_vec(),
            neighbours_known: true,
            regime_id: String::new(),
        },
        content(),
        [7; 16],
    )
    .expect("generates")
}

fn good(sim: &Sim, id: &str) -> usize {
    sim.rules()
        .catalog
        .goods
        .iter()
        .position(|d| d.id == id)
        .expect(id)
}

/// Sets a household's stores of the goods given by content id.
fn set(sim: &mut Sim, household: PermanentId, kg: &[(&str, f64)]) {
    let goods: Vec<usize> = kg.iter().map(|&(id, _)| good(sim, id)).collect();
    let h = sim
        .people_mut_for_tests()
        .households
        .iter_mut()
        .map(|(_, h)| h)
        .find(|h| h.id == household)
        .expect("household");
    for (&g, &(_, v)) in goods.iter().zip(kg) {
        h.stores[g] = v;
    }
}

/// The households of settlement `s`, largest first.
fn households_of(sim: &Sim, s: PermanentId) -> Vec<PermanentId> {
    let mut found: Vec<_> = sim
        .people()
        .households
        .iter()
        .filter(|(_, x)| x.settlement == Some(s) && !x.members.is_empty())
        .map(|(_, x)| (std::cmp::Reverse(x.members.len()), x.id))
        .collect();
    found.sort();
    found.into_iter().map(|x| x.1).collect()
}

/// Where person `p` lives.
fn home_of(sim: &Sim, p: PermanentId) -> Option<PermanentId> {
    let pop = sim.people();
    pop.person(p)
        .and_then(|q| pop.household(q.household))
        .and_then(|h| h.settlement)
}

/// Two settlements; a household of the second offers sickles, which nobody of the first knows how
/// to make or holds, and the largest household of the first has grain and provisions to pay
/// with.
struct Scene {
    sim: Sim,
    a: PermanentId,
    b: PermanentId,
    seller: PermanentId,
    buyer: PermanentId,
    sickle: usize,
    /// The content's weight of an hour's walk.
    walk_hour: f64,
}

fn scene() -> Scene {
    let mut sim = world(3, 30, &[30]);
    sim.advance_minutes(DAY).expect("advances");
    let (a, b) = (sim.land().settlements[0].id, sim.land().settlements[1].id);
    let sickle = good(&sim, "core:good/sickle");
    let seller = households_of(&sim, b)[0];
    set(
        &mut sim,
        seller,
        &[
            ("core:good/sickle", 14.0),
            ("core:good/grain", 0.0),
            ("core:good/provisions", 40.0),
        ],
    );
    let offers = |sim: &Sim| {
        sim.people()
            .household(seller)
            .is_some_and(|h| h.offers.iter().any(|o| usize::from(o.good) == sickle))
    };
    for _ in 0..8 {
        if offers(&sim) {
            break;
        }
        sim.advance_minutes(DAY).expect("advances");
    }
    assert!(offers(&sim), "the seller offers its spare sickles");
    for h in households_of(&sim, a) {
        set(
            &mut sim,
            h,
            &[
                ("core:good/sickle", 0.0),
                ("core:good/toolstone", 0.0),
                ("core:good/timber", 0.0),
            ],
        );
    }
    let buyer = households_of(&sim, a)[0];
    set(
        &mut sim,
        buyer,
        &[("core:good/grain", 3000.0), ("core:good/provisions", 60.0)],
    );
    let knapping = sim
        .rules()
        .catalog
        .technique_index("core:technique/knapping")
        .expect("knapping");
    let in_a: Vec<PermanentId> = sim
        .people()
        .households
        .iter()
        .filter(|(_, h)| h.settlement == Some(a))
        .flat_map(|(_, h)| h.members.clone())
        .collect();
    for (_, p) in sim.people_mut_for_tests().people.iter_mut() {
        if in_a.contains(&p.id) {
            p.knows.retain(|k| usize::from(k.technique) != knapping);
        }
    }
    // The hearths are an hour and three quarters' walk apart. At the content's two points an
    // hour, a tool the household lacks (four points) is worth an hour's walk each way and no
    // more, so here the walk is weighed at one point an hour: the purchase is the matter.
    let rules = sim.rules_mut_for_tests().expect("rules of its own");
    let walk_hour = rules.people.decision.w_walk_hour;
    rules.people.decision.w_walk_hour = 1.0;
    Scene {
        sim,
        a,
        b,
        seller,
        buyer,
        sickle,
        walk_hour,
    }
}

/// Reports `household` holds of `seller`'s offers of good `g`, told today by one of its people.
fn plant_reports(sim: &mut Sim, household: PermanentId, seller: PermanentId, g: usize) {
    let day = sim.now().day_index();
    let pop = sim.people_mut_for_tests();
    let s = pop.household(seller).expect("seller").clone();
    let offers: Vec<_> = s
        .offers
        .iter()
        .filter(|o| usize::from(o.good) == g)
        .copied()
        .collect();
    assert!(!offers.is_empty(), "offered");
    for o in offers {
        pop.reports.note(
            household,
            PriceReport {
                market: s.settlement.expect("settled"),
                seller,
                firm: false,
                good: o.good,
                payment: o.payment,
                price: o.price,
                ask_h: o.ask_h,
                units: o.units,
                day,
                how: ReportHow::Told,
                from: Some(s.members[0]),
            },
        );
    }
}

/// Trades settled in `market` by buyers of `from`.
fn bought_by(sim: &Sim, market: PermanentId, from: PermanentId) -> Vec<Trade> {
    sim.people()
        .market(market)
        .map(|m| {
            m.recent
                .iter()
                .filter(|t| t.from == Some(from))
                .copied()
                .collect()
        })
        .unwrap_or_default()
}

/// Purchases and trips that bought nothing, by why, of people of `from` at `to`, all years.
fn contact(sim: &Sim, from: PermanentId, to: PermanentId) -> (u32, [u32; Missed::COUNT]) {
    sim.people()
        .contacts
        .years
        .iter()
        .filter(|((_, f, t), _)| *f == from && *t == to)
        .fold((0, [0; Missed::COUNT]), |(b, mut m), (_, c)| {
            for (x, n) in m.iter_mut().zip(c.missed) {
                *x += n;
            }
            (b + c.bought, m)
        })
}

/// What `household` holds of `seller`'s offers of good `g`.
fn held_of(sim: &Sim, household: PermanentId, seller: PermanentId, g: usize) -> Vec<PriceReport> {
    sim.people()
        .reports
        .of(household)
        .iter()
        .filter(|r| r.seller == seller && usize::from(r.good) == g)
        .copied()
        .collect()
}

/// Advances to a minute past the next midnight, when households have reviewed their offers.
fn past_midnight(sim: &mut Sim) {
    let into = sim.now().minutes().rem_euclid(DAY);
    sim.advance_minutes(DAY - into + 1).expect("advances");
}

fn saves_and_goes_on_alike(sim: &mut Sim, minutes: i64) {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "fetch").expect("saves");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
    let same = |a: &Sim, b: &Sim| {
        let (a, b) = (persist::encode_sections(a), persist::encode_sections(b));
        assert_eq!(a.len(), b.len());
        for (x, y) in a.iter().zip(&b) {
            assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
        }
    };
    same(sim, &loaded);
    sim.advance_minutes(minutes).expect("advances");
    loaded.advance_minutes(minutes).expect("advances");
    same(sim, &loaded);
}

#[test]
fn a_household_buys_from_a_neighbour_by_report_and_the_trade_is_tallied_where_it_settles() {
    let Scene {
        mut sim,
        a,
        b,
        seller,
        buyer,
        sickle,
        walk_hour,
    } = scene();
    // Nobody of the first settlement has heard what the second's people offer: nobody goes.
    assert!(sim.people().reports.of(buyer).is_empty());
    sim.advance_minutes(2 * DAY).expect("advances");
    assert!(
        bought_by(&sim, b, a).is_empty(),
        "nobody buys from a seller they hold no report of"
    );
    assert_eq!(
        sim.people().household(buyer).expect("buyer").stores[sickle],
        0.0
    );

    // Told of the seller's terms, someone of the buyer's household walks to its door and buys.
    plant_reports(&mut sim, buyer, seller, sickle);
    // One of the household goes at a time: what another is on the way to buy is not home yet.
    let fetch = sim
        .rules()
        .catalog
        .activities
        .iter()
        .position(|x| x.id == "core:activity/fetch")
        .expect("fetch") as u16;
    let (mut trade, mut went) = (None, 0);
    for _ in 0..10 * 48 {
        sim.advance_minutes(30).expect("advances");
        let pop = sim.people();
        let going = pop
            .household(buyer)
            .expect("buyer")
            .members
            .iter()
            .filter(|&&m| pop.person(m).is_some_and(|p| p.act.def == fetch))
            .count();
        assert!(going <= 1, "{going} of the household on the way at once");
        went = went.max(going);
        trade = bought_by(&sim, b, a).into_iter().find(|t| t.buyer == buyer);
        if trade.is_some() {
            break;
        }
    }
    assert_eq!(went, 1, "someone was seen on the way");
    let trade = trade.expect("the buyer fetched a sickle from the seller's door");
    assert_eq!((trade.seller, usize::from(trade.good)), (seller, sickle));
    assert!(trade.units >= 1.0 - 1e-6 && trade.paid > 0.0);
    let pop = sim.people();
    assert!(pop.household(buyer).expect("buyer").stores[sickle] > 0.0);
    // Tallied in the seller's market, with the buyer's settlement; never in the buyer's own.
    assert!(
        pop.market(a)
            .is_none_or(|m| m.recent.iter().all(|t| t.seller != seller)),
        "the first settlement's market saw none of it"
    );
    let (purchases, missed) = contact(&sim, a, b);
    assert!(purchases >= 1, "the purchase is counted between the two");
    assert_eq!(missed, [0; Missed::COUNT]);
    let (here, there) = (
        civ_sim::frames::people::contacts_words(&sim, a),
        civ_sim::frames::people::contacts_words(&sim, b),
    );
    assert!(here.contains(" at "), "{here}");
    assert!(here.contains("purchase"), "{here}");
    assert!(there.contains("here by people of"), "{there}");
    // What the buyer saw at the door is what it now holds of the seller's terms.
    let seen = trade.at.day_index();
    let held = held_of(&sim, buyer, seller, sickle);
    assert!(!held.is_empty());
    assert!(
        held.iter()
            .all(|r| r.how == ReportHow::Seen && r.day == seen && r.from.is_none()),
        "{held:?}"
    );
    assert!(pop.problems(u64::MAX, usize::MAX).is_empty());

    // With the walk weighed as the content has it, nobody walks so far for a sickle again. Word
    // goes round the first settlement's hearths as old as it is: a report only the buyer's
    // household holds, of the seller's grain as seen twenty days ago, reaches others with that
    // day (ADR-0019 §1).
    sim.rules_mut_for_tests()
        .expect("rules of its own")
        .people
        .decision
        .w_walk_hour = walk_hour;
    let grain = good(&sim, "core:good/grain") as u16;
    let long_ago = sim.now().day_index() - 20;
    let rumour = PriceReport {
        good: grain,
        payment: sickle as u16,
        units: 0.0,
        day: long_ago,
        how: ReportHow::Seen,
        from: None,
        ..held[0]
    };
    sim.people_mut_for_tests().reports.note(buyer, rumour);
    sim.advance_minutes(6 * DAY).expect("advances");
    let pop = sim.people();
    let others: Vec<(PermanentId, PriceReport)> = pop
        .reports
        .held
        .iter()
        .filter(|(h, _)| **h != buyer)
        .flat_map(|(&h, list)| list.iter().map(move |r| (h, *r)))
        .collect();
    let told_from_a = |r: &PriceReport| {
        r.how == ReportHow::Told && r.from.and_then(|p| home_of(&sim, p)) == Some(a)
    };
    assert!(
        others
            .iter()
            .any(|(_, r)| r.good == trade.good && told_from_a(r)),
        "word of the seller's sickles went round the first settlement: {others:?}"
    );
    let passed: Vec<_> = others
        .iter()
        .filter(|(_, r)| r.key() == rumour.key())
        .collect();
    assert!(!passed.is_empty(), "the old report was passed on");
    for (h, _) in &others {
        assert_eq!(pop.household(*h).and_then(|x| x.settlement), Some(a));
    }
    // Believed the less the older it is: half as much a half-life on.
    let (today, half_life) = (
        sim.now().day_index(),
        sim.rules().people.reports.half_life_days,
    );
    for (_, r) in passed {
        assert!(told_from_a(r));
        assert_eq!(r.day, long_ago, "passed on as old as it was");
        let w = 0.5f64.powf((today - long_ago) as f64 / half_life);
        assert!((r.weight(today, half_life) - w).abs() < 1e-12 && w < 0.6);
    }

    // A report older than the content keeps one is let go at midnight; a newer one stays.
    let max_age = sim.rules().people.reports.max_age_days;
    let old = PriceReport {
        good: grain,
        day: sim.now().day_index() - max_age - 1,
        ..held[0]
    };
    sim.people_mut_for_tests().reports.note(buyer, old);
    past_midnight(&mut sim);
    let pop = sim.people();
    assert!(pop.reports.of(buyer).iter().all(|r| r.key() != old.key()));
    assert!(
        pop.reports
            .of(buyer)
            .iter()
            .any(|r| r.key() == rumour.key())
    );
    assert!(!held_of(&sim, buyer, seller, sickle).is_empty());

    // The reports, the trade's settlement and the counts are saved and go on alike.
    saves_and_goes_on_alike(&mut sim, DAY / 2);
}

#[test]
fn a_trip_by_a_report_gone_stale_buys_nothing_and_is_counted_with_why() {
    let Scene {
        mut sim,
        a,
        b,
        seller,
        buyer,
        sickle,
        ..
    } = scene();
    past_midnight(&mut sim);
    plant_reports(&mut sim, buyer, seller, sickle);
    assert!(held_of(&sim, buyer, seller, sickle).len() >= 2);
    // The seller now asks more for a sickle than the buyer holds of anything.
    let dear = |sim: &mut Sim| {
        let pop = sim.people_mut_for_tests();
        let (_, h) = pop
            .households
            .iter_mut()
            .find(|(_, h)| h.id == seller)
            .expect("seller");
        for o in h
            .offers
            .iter_mut()
            .filter(|o| usize::from(o.good) == sickle)
        {
            o.price = 1e6;
        }
    };
    for _ in 0..10 {
        dear(&mut sim);
        sim.advance_minutes(DAY).expect("advances");
        if contact(&sim, a, b).1[Missed::Payment as usize] > 0 {
            break;
        }
    }
    let mine = |sim: &Sim| {
        bought_by(sim, b, a)
            .iter()
            .filter(|t| t.buyer == buyer)
            .count()
    };
    assert_eq!(mine(&sim), 0, "nothing was bought");
    assert_eq!(
        contact(&sim, a, b).1,
        [0, 0, 1, 0],
        "a trip that found the seller taking nothing they could spare"
    );
    // The buyer holds what it saw, and no longer the terms it went by.
    let held = held_of(&sim, buyer, seller, sickle);
    assert!(!held.is_empty());
    assert!(
        held.iter()
            .all(|r| r.how == ReportHow::Seen && r.price >= 1e6),
        "{held:?}"
    );

    // Told again of the terms it went by, it walks to a seller who has sold out.
    past_midnight(&mut sim);
    plant_reports_as_they_were(&mut sim, buyer, seller, sickle);
    let sold_out = |sim: &mut Sim| {
        let pop = sim.people_mut_for_tests();
        let (_, h) = pop
            .households
            .iter_mut()
            .find(|(_, h)| h.id == seller)
            .expect("seller");
        h.offers.retain(|o| usize::from(o.good) != sickle);
        h.stores[sickle] = 0.0;
    };
    for _ in 0..10 {
        sold_out(&mut sim);
        sim.advance_minutes(DAY).expect("advances");
        if contact(&sim, a, b).1[Missed::SoldOut as usize] > 0 {
            break;
        }
    }
    assert!(contact(&sim, a, b).1[Missed::SoldOut as usize] >= 1);
    assert_eq!(mine(&sim), 0, "nothing was bought");
    let words = civ_sim::frames::people::contacts_words(&sim, a);
    assert!(
        words.contains("that bought nothing") && words.contains("the seller had sold out"),
        "{words}"
    );
    let held = held_of(&sim, buyer, seller, sickle);
    assert!(
        !held.is_empty() && held.iter().all(|r| r.units == 0.0),
        "it knows the seller has none left, seen or told, whatever it hears of before: {held:?}"
    );
    assert_eq!(
        sim.people().household(buyer).expect("buyer").stores[sickle],
        0.0
    );
    assert!(sim.people().problems(u64::MAX, usize::MAX).is_empty());
}

/// A report `household` is told today of `seller`'s sickles at the terms it took for provisions
/// as the scene began.
fn plant_reports_as_they_were(
    sim: &mut Sim,
    household: PermanentId,
    seller: PermanentId,
    sickle: usize,
) {
    let day = sim.now().day_index();
    let held = held_of(sim, household, seller, sickle)[0];
    let provisions = good(sim, "core:good/provisions") as u16;
    sim.people_mut_for_tests().reports.note(
        household,
        PriceReport {
            payment: provisions,
            price: 3.0,
            day,
            how: ReportHow::Told,
            ..held
        },
    );
}

#[test]
fn a_visitor_hears_what_those_they_keep_company_with_offer() {
    // A woman of the first settlement is made the mother of an adult of the second, as if they
    // had married away: they visit, and at the hearth each hears what the other's household
    // offers (ADR-0019 §1).
    let mut sim = world(3, 30, &[30]);
    let (a, b) = (sim.land().settlements[0].id, sim.land().settlements[1].id);
    let adult = |sim: &Sim, s: PermanentId, female: bool| {
        let pop = sim.people();
        let mut found: Vec<_> = pop
            .people
            .iter()
            .filter(|(_, p)| {
                p.age_years(sim.now()) >= 16.0
                    && (p.sex == civ_agents::Sex::Female) == female
                    && home_of(sim, p.id) == Some(s)
            })
            .map(|(_, p)| p.id)
            .collect();
        found.sort();
        found[0]
    };
    let (mother, son) = (adult(&sim, a, true), adult(&sim, b, false));
    sim.people_mut_for_tests()
        .records
        .get_mut(&son)
        .expect("on record")
        .mother = Some(mother);
    let mut heard = Vec::new();
    for _ in 0..12 {
        sim.advance_minutes(5 * DAY).expect("advances");
        let pop = sim.people();
        heard = pop
            .reports
            .held
            .iter()
            .flat_map(|(&h, list)| list.iter().map(move |r| (h, *r)))
            .filter(|(_, r)| {
                r.how == ReportHow::Told && r.from.and_then(|p| home_of(&sim, p)) == Some(r.market)
            })
            .collect();
        if !heard.is_empty() {
            break;
        }
    }
    assert!(!heard.is_empty(), "a visitor heard offers at a hearth");
    let pop = sim.people();
    for (h, list) in &pop.reports.held {
        let home = pop.household(*h).and_then(|x| x.settlement);
        for r in list {
            assert_ne!(
                Some(r.market),
                home,
                "a household's own market is never a report"
            );
            assert!(r.market == a || r.market == b);
        }
    }
    assert!(pop.problems(u64::MAX, usize::MAX).is_empty());
}

#[test]
fn in_a_world_of_one_settlement_nobody_holds_a_report_or_buys_elsewhere() {
    let mut sim = world(3, 30, &[]);
    sim.advance_minutes(20 * DAY).expect("advances");
    let pop = sim.people();
    assert!(pop.reports.held.is_empty());
    assert!(
        pop.markets
            .iter()
            .flat_map(|m| m.recent.iter())
            .all(|t| t.from.is_none())
    );
    assert!(pop.contacts.years.is_empty());
}
