//! Buying from a neighbour by report (M5b slice AP, ADR-0019 §1–§4): a household knows another
//! settlement's offers only by price reports its members saw or were told, goes to a seller's
//! door by one, buys at the terms the seller posts there now, and the trade is tallied in the
//! seller's market with the buyer's settlement. And fetching to resell (M5b slice AQ, ADR-0019
//! §6): an errand planned at a household's review, run as making to sell is weighed, and the
//! goods offered at home.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::ledger::Trade;
use civ_agents::reports::{Missed, PriceReport, ReportHow};
use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_schema::{flatbuffers, wire};
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
    // World 10: its two camps are on one bank of the trunk river, which nobody can wade, about an
    // hour's walk apart.
    scene_in(10)
}

/// [`scene`] in world `seed`.
fn scene_in(seed: u64) -> Scene {
    let mut sim = world(seed, 30, &[30]);
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
    // The seller's market shows who is on the road to buy there today (wire 1.55): the buyer, on
    // the way home.
    let on_the_way = {
        let payload = civ_sim::frames::markets::markets_response(&sim);
        let response = flatbuffers::root::<wire::Response>(&payload).expect("a response");
        response
            .body_as_markets()
            .expect("markets")
            .markets()
            .expect("a list")
            .iter()
            .find(|m| m.settlement() == b.get())
            .expect("the seller's market")
            .on_the_way()
            .unwrap_or_default()
            .to_owned()
    };
    let from = sim
        .land()
        .settlements
        .iter()
        .find(|x| x.id == a)
        .expect("the buyer's settlement")
        .name
        .clone();
    assert_eq!(
        on_the_way,
        format!("On the road to buy here today: 1 person of {from}")
    );
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
    // The observer: the household's reports in the inspector, and the purchase in the seller's
    // market, named with the buyer's settlement (wire 1.54).
    let name = |s: PermanentId| {
        sim.land()
            .settlements
            .iter()
            .find(|x| x.id == s)
            .expect("a settlement")
            .name
            .clone()
    };
    let member = sim.people().household(buyer).expect("buyer").members[0];
    let payload = civ_sim::frames::people::person_response(&sim, member.get(), 0).expect("alive");
    let response = flatbuffers::root::<wire::Response>(&payload).expect("a response");
    let info = response.body_as_person_info().expect("a person");
    let lines: Vec<&str> = info.reports().expect("reports").iter().collect();
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with(&format!("At {}, ", name(b))) && l.contains("; seen ")),
        "{lines:?}"
    );
    let payload = civ_sim::frames::markets::markets_response(&sim);
    let response = flatbuffers::root::<wire::Response>(&payload).expect("a response");
    let markets = response.body_as_markets().expect("markets");
    let there = markets
        .markets()
        .expect("a list")
        .iter()
        .find(|m| m.settlement() == b.get())
        .expect("the seller's market");
    let outsiders = there.outsiders().unwrap_or_default();
    assert!(
        outsiders.contains(&format!("by people of {}", name(a))),
        "{outsiders}"
    );
    let shown = there
        .recent()
        .expect("trades")
        .iter()
        .find(|t| t.buyer() == buyer.get())
        .expect("the purchase is listed");
    assert_eq!(shown.from(), a.get());
    let text = shown.text().unwrap_or_default();
    assert!(text.contains(&format!(" of {} for ", name(a))), "{text}");
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
    let lines = civ_sim::frames::markets::reports_words(&sim, buyer);
    assert!(
        lines
            .iter()
            .any(|l| l.contains(": none left of the sickle it offered; ")),
        "{lines:?}"
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

#[test]
fn a_trip_to_a_seller_who_holds_less_than_its_terms_say_is_counted_and_not_made_again() {
    // ADR-0019 §2: the seller's terms still offer sickles, but it holds almost none (they wore,
    // or went, since its review). The buyer walks there, buys nothing, says why, and holds what
    // it saw: none to be had there.
    let Scene {
        mut sim,
        a,
        b,
        seller,
        buyer,
        sickle,
        ..
    } = scene();
    // Only the buyer's household has heard of the seller.
    sim.rules_mut_for_tests()
        .expect("rules of its own")
        .people
        .reports
        .share_told = 0.0;
    plant_reports(&mut sim, buyer, seller, sickle);
    set(&mut sim, seller, &[("core:good/sickle", 0.2)]);
    let mut missed = [0; Missed::COUNT];
    for _ in 0..10 * 48 {
        sim.advance_minutes(30).expect("advances");
        missed = contact(&sim, a, b).1;
        if missed.iter().any(|&n| n > 0) {
            break;
        }
        // The seller's review would post its terms afresh: it is kept from holding any again.
        set(&mut sim, seller, &[("core:good/sickle", 0.2)]);
    }
    assert_eq!(
        missed,
        [1, 0, 0, 0],
        "a trip for nothing, the seller had none to sell"
    );
    assert!(bought_by(&sim, b, a).is_empty());
    let held = held_of(&sim, buyer, seller, sickle);
    assert!(
        !held.is_empty()
            && held
                .iter()
                .all(|r| r.units == 0.0 && r.how == ReportHow::Seen),
        "{held:?}"
    );
    // Nobody of the household goes back for what is not there.
    sim.advance_minutes(3 * DAY).expect("advances");
    assert_eq!(contact(&sim, a, b).1, [1, 0, 0, 0]);
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
    let mut sim = world(10, 30, &[30]);
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

#[test]
fn an_ask_anchors_on_what_replacing_the_good_from_elsewhere_would_cost() {
    // Two worlds alike in all but one belief (ADR-0019 §5): a household of the first settlement
    // in one of them holds a fresh report of a seller in the second that offers what the
    // household sells, for next to nothing. At its next review of what it offers, its ask falls
    // toward what replacing the good from there would cost it; in the other world it does not.
    let mut worlds = [world(10, 30, &[30]), world(10, 30, &[30])];
    for sim in &mut worlds {
        sim.advance_minutes(2 * DAY).expect("advances");
    }
    let (a, b) = (
        worlds[0].land().settlements[0].id,
        worlds[0].land().settlements[1].id,
    );
    let (seller, offer) = households_of(&worlds[0], a)
        .into_iter()
        .find_map(|h| {
            let o = *worlds[0].people().household(h)?.offers.first()?;
            Some((h, o))
        })
        .expect("a household of the first settlement offers something");
    let elsewhere = households_of(&worlds[0], b)[0];
    let day = worlds[0].now().day_index();
    for (i, sim) in worlds.iter_mut().enumerate() {
        let pop = sim.people_mut_for_tests();
        if i == 0 {
            pop.reports.note(
                seller,
                PriceReport {
                    market: b,
                    seller: elsewhere,
                    firm: false,
                    good: offer.good,
                    payment: offer.payment,
                    price: offer.price * 0.01,
                    ask_h: offer.ask_h * 0.01,
                    units: 50.0,
                    day,
                    how: ReportHow::Seen,
                    from: None,
                },
            );
        }
    }
    let review = i64::from(worlds[0].rules().people.market.review_days);
    let ask = |sim: &Sim| {
        sim.people()
            .household(seller)
            .and_then(|h| h.offers.iter().find(|o| o.good == offer.good))
            .map(|o| o.ask_h)
    };
    for sim in &mut worlds {
        sim.advance_minutes((review + 1) * DAY).expect("advances");
    }
    let (believed, unaware) = (ask(&worlds[0]), ask(&worlds[1]));
    let (Some(believed), Some(unaware)) = (believed, unaware) else {
        panic!("the good is still offered in both: {believed:?}, {unaware:?}");
    };
    assert!(
        believed < unaware * 0.99,
        "asked {believed} h knowing it could be had elsewhere, {unaware} h not knowing"
    );
}

#[test]
fn a_month_of_trade_between_settlements_is_kept_on_record() {
    // ADR-0019 §7: at a month's end, the median asks of each good offered in both settlements and
    // what it fetched in each; through the month, the trips people made to the other's sellers,
    // their walking hours and what they carried home.
    let Scene {
        mut sim,
        a,
        b,
        seller,
        buyer,
        sickle,
        ..
    } = scene();
    plant_reports(&mut sim, buyer, seller, sickle);
    let mut trade = None;
    for _ in 0..10 {
        sim.advance_minutes(DAY).expect("advances");
        trade = bought_by(&sim, b, a).into_iter().find(|t| t.buyer == buyer);
        if trade.is_some() {
            break;
        }
    }
    let trade = trade.expect("a purchase");
    let month = civ_agents::market::month_of(trade.at);
    let carried = sim
        .people()
        .convergence
        .carried
        .get(&(month, a, b))
        .cloned()
        .expect("the month's trips from the first settlement to the second");
    assert!(carried.trips >= 1 && carried.walk_h > 1.0, "{carried:?}");
    assert!(
        carried
            .goods
            .iter()
            .any(|&(g, u)| usize::from(g) == sickle && u >= 1.0),
        "{carried:?}"
    );
    // On the next month's first day the month is closed.
    while civ_agents::market::month_of(sim.now()) == month {
        sim.advance_minutes(DAY).expect("advances");
    }
    sim.advance_minutes(DAY).expect("advances");
    let pair = (month, a.min(b), a.max(b));
    let gaps = sim
        .people()
        .convergence
        .gaps
        .get(&pair)
        .cloned()
        .expect("goods offered in both are on record");
    assert!(!gaps.is_empty());
    for g in &gaps {
        assert!(g.ask_h.iter().all(|&x| x > 0.0), "{g:?}");
        assert!(g.paid_h.iter().all(|&x| x == -1.0 || x > 0.0), "{g:?}");
    }
    assert!(sim.people().problems(u64::MAX, usize::MAX).is_empty());
    // The market panel says so (wire 1.55): in the first settlement's market, the asks there and
    // here of what both offered last month, and what people of here carried home from there.
    let name = |s: PermanentId| {
        sim.land()
            .settlements
            .iter()
            .find(|x| x.id == s)
            .expect("a settlement")
            .name
            .clone()
    };
    let payload = civ_sim::frames::markets::markets_response(&sim);
    let response = flatbuffers::root::<wire::Response>(&payload).expect("a response");
    let markets = response.body_as_markets().expect("markets");
    let here = markets
        .markets()
        .expect("a list")
        .iter()
        .find(|m| m.settlement() == a.get())
        .expect("the buyer's market");
    let lines: Vec<&str> = here.between().expect("lines").iter().collect();
    let line = lines
        .iter()
        .find(|l| l.starts_with(&format!("With {} last month: ", name(b))))
        .unwrap_or_else(|| panic!("{lines:?}"));
    assert!(line.contains(" points apart"), "{line}");
    assert!(
        line.contains("people of here carried home ") && line.contains(" from there in "),
        "{line}"
    );
    sim.rules_mut_for_tests()
        .expect("rules of its own")
        .people
        .decision
        .w_walk_hour = 2.0;
    saves_and_goes_on_alike(&mut sim, DAY / 2);
}

#[test]
fn the_twin_harness_stops_purchases_between_settlements() {
    // ADR-0019 §8: the demo's twin. Set in memory, never in content or a save: nobody buys from a
    // seller in another settlement, whatever they know of it.
    let Scene {
        mut sim,
        a,
        b,
        seller,
        buyer,
        sickle,
        ..
    } = scene();
    plant_reports(&mut sim, buyer, seller, sickle);
    sim.people_mut_for_tests().stop_trade_between = true;
    sim.advance_minutes(10 * DAY).expect("advances");
    assert!(bought_by(&sim, b, a).is_empty(), "nothing was bought");
    assert_eq!(contact(&sim, a, b).0, 0);
    // Nobody even set out: the trip is left out of the choice, not cut short at the door.
    assert!(sim.people().convergence.carried.is_empty(), "nobody went");
}

/// Advances to a minute past the next midnight on which `household` reviews its offers.
fn past_review(sim: &mut Sim, household: PermanentId) {
    let review_days = i64::from(sim.rules().people.market.review_days.max(1));
    for _ in 0..review_days {
        past_midnight(sim);
        if (household.get() as i64 + sim.now().day_index()).rem_euclid(review_days) == 0 {
            return;
        }
    }
    panic!("no review within {review_days} days");
}

/// Sets `household`'s sickles to `units` a minute before its next review (tools wear as they are
/// used), and advances to a minute past it.
fn review_holding(sim: &mut Sim, household: PermanentId, units: f64) {
    review_holding_with(sim, household, units, |_| {});
}

/// [`review_holding`], `before` done to the world in the same minute.
fn review_holding_with(
    sim: &mut Sim,
    household: PermanentId,
    units: f64,
    before: impl FnOnce(&mut Sim),
) {
    let review_days = i64::from(sim.rules().people.market.review_days.max(1));
    for _ in 0..=review_days {
        let into = sim.now().minutes().rem_euclid(DAY);
        let next = sim.now().day_index() + 1;
        if (household.get() as i64 + next).rem_euclid(review_days) == 0 {
            if into < DAY - 1 {
                sim.advance_minutes(DAY - 1 - into).expect("advances");
            }
            set(sim, household, &[("core:good/sickle", units)]);
            before(sim);
            sim.advance_minutes(2).expect("advances");
            return;
        }
        sim.advance_minutes(DAY - into + 1).expect("advances");
    }
    panic!("no review within {review_days} days");
}

/// `seller` offers `units` sickles again on the terms `household` holds reports of (its own
/// neighbours buy them too).
fn restock(sim: &mut Sim, seller: PermanentId, household: PermanentId, sickle: usize, units: f32) {
    let terms = held_of(sim, household, seller, sickle);
    set(sim, seller, &[("core:good/sickle", 2.0 * f64::from(units))]);
    let h = sim
        .people_mut_for_tests()
        .households
        .iter_mut()
        .map(|(_, h)| h)
        .find(|h| h.id == seller)
        .expect("seller");
    h.offers.retain(|o| usize::from(o.good) != sickle);
    for r in terms {
        h.offers.push(civ_agents::market::Offer {
            good: r.good,
            payment: r.payment,
            price: r.price,
            units,
            ask_h: r.ask_h,
        });
    }
}

/// Units of good `g` household `h` offers.
fn offered(sim: &Sim, h: PermanentId, g: usize) -> f32 {
    sim.people()
        .household(h)
        .expect("household")
        .offers
        .iter()
        .filter(|o| usize::from(o.good) == g)
        .map(|o| o.units)
        .fold(0.0, f32::max)
}

/// The scene's buyer as a household that would fetch sickles to sell at home (ADR-0019 §6): it
/// has heard of the seller's terms, and nobody it tells at the hearth passes them on; its
/// settlement's buyers are on record as wanting `wanted` sickles nobody offers, at `worth_h`
/// hours each. The walk is weighed at half a point an hour: the errand is the matter. What it
/// keeps of sickles, learnt from what it offers of plenty at a review (it cannot make one, so it
/// asks what replacing one from the seller would cost).
fn reseller(sc: &mut Scene, wanted: f64, worth_h: f64) -> f32 {
    let (a, buyer, seller, sickle) = (sc.a, sc.buyer, sc.seller, sc.sickle);
    let sim = &mut sc.sim;
    let rules = sim.rules_mut_for_tests().expect("rules of its own");
    rules.people.decision.w_walk_hour = 0.5;
    rules.people.reports.share_told = 0.0;
    plant_reports(sim, buyer, seller, sickle);
    let plenty = 20.0;
    review_holding(sim, buyer, plenty);
    let spare = offered(sim, buyer, sickle);
    assert!(spare > 1.0, "it offers what it holds beyond its keep");
    let keep = plenty as f32 - spare;
    let day = sim.now().day_index();
    let pop = sim.people_mut_for_tests();
    if let Some(h) = pop
        .households
        .iter_mut()
        .map(|(_, h)| h)
        .find(|h| h.id == buyer)
    {
        h.offers.retain(|o| usize::from(o.good) != sickle);
    }
    let goods = pop
        .markets
        .first()
        .map_or(0, |m| m.sold.len())
        .max(sickle + 1);
    if !pop.markets.iter().any(|m| m.settlement == a) {
        pop.markets
            .push(civ_agents::market::Market::new(a, goods, day));
    }
    let m = pop
        .markets
        .iter_mut()
        .find(|m| m.settlement == a)
        .expect("the buyer's market");
    m.record_unmet(sickle, wanted, wanted * worth_h);
    keep
}

#[test]
fn a_household_fetches_to_sell_at_home_what_its_neighbours_want_and_nobody_offers() {
    let mut sc = scene();
    let keep = reseller(&mut sc, 4.0, 60.0);
    let Scene {
        mut sim,
        a,
        b,
        seller,
        buyer,
        sickle,
        ..
    } = sc;
    // At its review, holding what it keeps of sickles and a little (too little to offer), it
    // weighs the trip: a few sickles, as many as its neighbours want and the seller offers,
    // worth carrying home.
    review_holding(&mut sim, buyer, f64::from(keep) + 0.3);
    // (Before, lacking any, it fetched sickles for itself.)
    let since = sim.now();
    let errand = *sim
        .people()
        .reports
        .errands
        .get(&buyer)
        .expect("an errand is planned");
    assert_eq!((errand.seller, errand.market), (seller, b));
    assert_eq!(usize::from(errand.good), sickle);
    assert!(
        errand.units >= 1.0 && errand.units <= 4.0 + 1e-3,
        "{errand:?}"
    );
    assert!(errand.share > 0.0 && errand.share <= 1.0, "{errand:?}");
    assert!(sim.people().problems(u64::MAX, usize::MAX).is_empty());
    // The inspector says what the household means to do (wire 1.55).
    let member = sim.people().household(buyer).expect("buyer").members[0];
    let payload = civ_sim::frames::people::person_response(&sim, member.get(), 0).expect("alive");
    let response = flatbuffers::root::<wire::Response>(&payload).expect("a response");
    let words = response
        .body_as_person_info()
        .expect("a person")
        .errand()
        .unwrap_or_default()
        .to_owned();
    let there = sim
        .land()
        .settlements
        .iter()
        .find(|x| x.id == b)
        .expect("the seller's settlement")
        .name
        .clone();
    assert!(
        words.starts_with("Their household means to fetch ")
            && words.contains(&format!(" at {there} to sell at home; planned today")),
        "{words}"
    );
    restock(&mut sim, seller, buyer, sickle, 20.0);
    // Saved and loaded, it is the same errand, and the world goes on alike.
    saves_and_goes_on_alike(&mut sim, 60);

    // Someone goes, buys more than one sickle and carries them home: the trade is tallied in the
    // seller's market with the buyer's settlement, as any purchase between them is.
    let mut trade = None;
    for _ in 0..7 * 48 {
        sim.advance_minutes(30).expect("advances");
        trade = bought_by(&sim, b, a)
            .into_iter()
            .find(|t| t.buyer == buyer && t.at > since);
        if trade.is_some() {
            break;
        }
    }
    let trade = trade.expect("the errand was run");
    assert_eq!((trade.seller, usize::from(trade.good)), (seller, sickle));
    assert!(trade.units > 1.0, "{trade:?}");
    assert!(
        !sim.people().reports.errands.contains_key(&buyer),
        "an errand run is done"
    );
    let held = sim.people().household(buyer).expect("buyer").stores[sickle];
    assert!(
        held > f64::from(keep) + 1.0,
        "{held} against a keep of {keep}"
    );
    let (purchases, _) = contact(&sim, a, b);
    assert!(purchases >= 1);

    // At its next review it offers at its door what it fetched beyond its keep.
    past_review(&mut sim, buyer);
    assert!(
        offered(&sim, buyer, sickle) >= 1.0,
        "the sickles fetched are offered at home"
    );
    assert!(sim.people().problems(u64::MAX, usize::MAX).is_empty());
}

#[test]
fn an_errand_to_a_seller_who_sold_out_buys_nothing_and_is_counted_with_why() {
    let mut sc = scene();
    let keep = reseller(&mut sc, 4.0, 60.0);
    let Scene {
        mut sim,
        a,
        b,
        seller,
        buyer,
        sickle,
        ..
    } = sc;
    review_holding(&mut sim, buyer, f64::from(keep) + 0.3);
    assert!(sim.people().reports.errands.contains_key(&buyer));
    let since = sim.now();
    // The seller has none left, which the buyer's household has not heard.
    set(&mut sim, seller, &[("core:good/sickle", 0.0)]);
    if let Some(h) = sim
        .people_mut_for_tests()
        .households
        .iter_mut()
        .map(|(_, h)| h)
        .find(|h| h.id == seller)
    {
        h.offers.retain(|o| usize::from(o.good) != sickle);
    }
    let mut missed = [0; Missed::COUNT];
    for _ in 0..7 * 48 {
        sim.advance_minutes(30).expect("advances");
        missed = contact(&sim, a, b).1;
        if missed.iter().any(|&n| n > 0) {
            break;
        }
    }
    assert_eq!(missed, [1, 0, 0, 0], "a trip for nothing, sold out");
    assert!(bought_by(&sim, b, a).iter().all(|t| t.at < since));
    assert!(!sim.people().reports.errands.contains_key(&buyer));
    // What it saw at the door it now holds: none left.
    let held = held_of(&sim, buyer, seller, sickle);
    assert!(
        !held.is_empty()
            && held
                .iter()
                .all(|r| r.units == 0.0 && r.how == ReportHow::Seen),
        "{held:?}"
    );
    // And with nothing reported to be had, it plans no errand there again.
    past_review(&mut sim, buyer);
    assert!(!sim.people().reports.errands.contains_key(&buyer));
}

#[test]
fn nobody_fetches_to_sell_what_their_neighbours_do_not_want_or_the_twin_stops() {
    // World 3 throughout: its walk to the seller is long.
    // Wanted for less than it would cost to fetch: no errand.
    let mut sc = scene_in(3);
    let keep = reseller(&mut sc, 4.0, 0.5);
    review_holding(&mut sc.sim, sc.buyer, f64::from(keep) + 0.3);
    assert!(!sc.sim.people().reports.errands.contains_key(&sc.buyer));

    // Wanted for more than the sickles and the trading cost, but not enough to pay for the walk
    // too (in world 3, whose way to the seller goes round the river, a unit costs it about 2.2
    // hours without the walk and more than 2.6 with it, its margin included): no errand.
    let mut sc = scene_in(3);
    let keep = reseller(&mut sc, 4.0, 2.6);
    review_holding(&mut sc.sim, sc.buyer, f64::from(keep) + 0.3);
    assert!(!sc.sim.people().reports.errands.contains_key(&sc.buyer));

    // Wanted, but a neighbour at home already offers as many as are wanted: no errand (research
    // 08-12 §1.6: depth, not price alone). The neighbour reviews on another day, so its offer
    // stands at the reseller's review.
    let mut sc = scene_in(3);
    let keep = reseller(&mut sc, 4.0, 60.0);
    let (buyer, sickle) = (sc.buyer, sc.sickle);
    let review_days = i64::from(sc.sim.rules().people.market.review_days.max(1));
    let neighbour = households_of(&sc.sim, sc.a)
        .into_iter()
        .find(|&h| h != buyer && (h.get() as i64 - buyer.get() as i64).rem_euclid(review_days) != 0)
        .expect("a neighbour who reviews on another day");
    review_holding_with(&mut sc.sim, buyer, f64::from(keep) + 0.3, |sim| {
        let h = sim
            .people_mut_for_tests()
            .households
            .iter_mut()
            .map(|(_, h)| h)
            .find(|h| h.id == neighbour)
            .expect("neighbour");
        h.offers.push(civ_agents::market::Offer {
            good: sickle as u16,
            payment: 0,
            price: 1.0,
            units: 6.0,
            ask_h: 4.0,
        });
    });
    assert!(!sc.sim.people().reports.errands.contains_key(&buyer));

    // Wanted, but under the demo's twin (ADR-0019 §8): planned, never run.
    let mut sc = scene_in(3);
    let keep = reseller(&mut sc, 4.0, 60.0);
    let Scene {
        mut sim,
        a,
        b,
        buyer,
        ..
    } = sc;
    sim.people_mut_for_tests().stop_trade_between = true;
    review_holding(&mut sim, buyer, f64::from(keep) + 0.3);
    assert!(sim.people().reports.errands.contains_key(&buyer));
    let since = sim.now();
    let went = |sim: &Sim| -> u32 {
        sim.people()
            .convergence
            .carried
            .iter()
            .filter(|((_, from, _), _)| *from == a)
            .map(|(_, c)| c.trips)
            .sum()
    };
    let trips = went(&sim);
    sim.advance_minutes(6 * DAY).expect("advances");
    assert!(
        bought_by(&sim, b, a).iter().all(|t| t.at < since),
        "nothing was bought"
    );
    assert_eq!(went(&sim), trips, "nobody went");
}
