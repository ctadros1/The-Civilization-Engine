//! A duty's collector (M7c slice BY, ADR-0026 §4; research 09-09 §1.3): one named by a duty law
//! sees a purchase brought home and assesses it in full, or asks the buyer's household a payment
//! to assess none of it; the household pays, and nothing reaches the store, or refuses, pays the
//! duty and holds the asking against the office.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::ledger::Trade;
use civ_agents::polity::{IssueKind, Law, LawStatus, Outcome as Decided};
use civ_agents::reports::{PriceReport, ReportHow};
use civ_agents::word::{Blamed, Wrong};
use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_sim::{NewWorld, Sim};

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

fn good(sim: &Sim, id: &str) -> usize {
    sim.rules()
        .catalog
        .goods
        .iter()
        .position(|d| d.id == id)
        .expect(id)
}

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

/// Two settlements an hour and three quarters' walk apart (world 10, as `tests/fetch.rs` has
/// them): a household of the second offers sickles nobody of the first holds or can make, and
/// the largest household of the first has grain and provisions to pay with and holds a report
/// of the offer.
struct Scene {
    sim: Sim,
    a: PermanentId,
    b: PermanentId,
    buyer: PermanentId,
    sickle: usize,
}

fn scene() -> Scene {
    let mut sim = Sim::create_for_tests(
        &NewWorld {
            name: "Neighbours".to_owned(),
            seed: 10,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 768,
            band_size: 30,
            neighbours: vec![30],
            neighbours_known: true,
            regime_id: String::new(),
        },
        content(),
        [7; 16],
    )
    .expect("generates");
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
    // The purchase is the matter, not the walk (as `tests/fetch.rs` weighs it).
    sim.rules_mut_for_tests()
        .expect("rules of its own")
        .people
        .decision
        .w_walk_hour = 1.0;
    // Told of the seller's terms.
    let day = sim.now().day_index();
    let pop = sim.people_mut_for_tests();
    let s = pop.household(seller).expect("seller").clone();
    for o in s.offers.iter().filter(|o| usize::from(o.good) == sickle) {
        pop.reports.note(
            buyer,
            PriceReport {
                market: b,
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
    Scene {
        sim,
        a,
        b,
        buyer,
        sickle,
    }
}

/// A law of template `template` in force at settlement `at`'s polity since today, known to every
/// one of its people; against `target` for an embargo, at `share` for a duty.
fn law(
    sim: &mut Sim,
    at: PermanentId,
    template: &str,
    share: f32,
    target: Option<PermanentId>,
    holder: Option<PermanentId>,
) -> PermanentId {
    let policies = &sim.rules().catalog.policies;
    let policy = policies
        .iter()
        .position(|d| d.id == template)
        .expect("the core content has the template");
    let kind = policies[policy].kind;
    let id = sim.allocate_id_for_tests();
    let now = sim.now();
    let pop = sim.people_mut_for_tests();
    let members: Vec<PermanentId> = pop
        .households
        .iter()
        .filter(|(_, h)| h.settlement == Some(at))
        .flat_map(|(_, h)| h.members.clone())
        .collect();
    let pi = pop
        .polities
        .iter()
        .position(|p| p.settlement == at)
        .expect("a polity");
    let mut law = Law {
        id,
        policy: policy as u16,
        kind,
        levy_share: share,
        holder,
        relief_days: 0.0,
        sanction: Default::default(),
        hours: (0, 0),
        status: LawStatus::InForce,
        sponsor: members[0],
        proposed: now,
        issue: IssueKind::FetchedElsewhere,
        meets_day: now.day_index(),
        decided: Some(now),
        outcome: Some(Decided::Passed),
        eligible: members.len() as u32,
        stances: Vec::new(),
        known: Vec::new(),
        compliance: Default::default(),
        watch: Default::default(),
        body: None,
        ends: None,
        agreement: None,
        target,
    };
    for &m in &members {
        law.learn(m, now.day_index());
    }
    pop.polities[pi].laws.push(law);
    id
}

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

/// Lives up to `days` days half an hour at a time until the buyer has bought at the seller's
/// door.
fn until_bought(sc: &mut Scene, days: i64) -> Option<Trade> {
    for _ in 0..days * 48 {
        sc.sim.advance_minutes(30).expect("advances");
        if let Some(t) = bought_by(&sc.sim, sc.b, sc.a)
            .into_iter()
            .find(|t| t.buyer == sc.buyer)
        {
            return Some(t);
        }
    }
    None
}

fn the_law(sim: &Sim, id: PermanentId) -> Law {
    sim.people()
        .polities
        .iter()
        .flat_map(|p| &p.laws)
        .find(|l| l.id == id)
        .cloned()
        .expect("kept")
}

/// Person `p`'s objection to taking, set, and the chance they believe they run of being seen,
/// none.
fn object(sim: &mut Sim, p: PermanentId, objection: f32) {
    let pop = sim.people_mut_for_tests();
    let person = pop
        .people
        .iter_mut()
        .map(|(_, x)| x)
        .find(|x| x.id == p)
        .expect("person");
    person.objection = objection;
    person.risk_seen = 0.0;
}

/// The scene with a collected duty of a half on tools at the buyer's settlement, its collector
/// an adult of another household there who sees every purchase, with objection `collector`,
/// asking `ask_share` of what is owed; the buyer's household's members object `buyer`. Lives
/// until the buyer buys; returns the law and the collector.
fn collected(collector: f32, buyer: f32, ask_share: f64) -> (Scene, PermanentId, PermanentId) {
    let mut sc = scene();
    // Food weighs far above all else, so who asks and who pays turns on objection alone.
    let crime = &mut sc
        .sim
        .rules_mut_for_tests()
        .expect("rules of its own")
        .people
        .crime;
    crime.collector_sight = 1.0;
    crime.collector_ask_share = ask_share;
    crime.w_demand_loss = 1000.0;
    let other = households_of(&sc.sim, sc.a)
        .into_iter()
        .find(|&h| h != sc.buyer)
        .expect("another household");
    let c = sc.sim.people().household(other).expect("household").members[0];
    object(&mut sc.sim, c, collector);
    let members = sc
        .sim
        .people()
        .household(sc.buyer)
        .expect("buyer")
        .members
        .clone();
    for m in members {
        object(&mut sc.sim, m, buyer);
    }
    let duty = law(
        &mut sc.sim,
        sc.a,
        "core:policy/duty_on_tools_collected",
        0.5,
        None,
        Some(c),
    );
    let t = until_bought(&mut sc, 10).expect("the buyer fetched sickles");
    assert_eq!(usize::from(t.good), sc.sickle);
    (sc, duty, c)
}

/// Law `id`'s conduct in the government panel's words.
fn conduct_of(sim: &Sim, id: PermanentId) -> String {
    let payload = civ_sim::frames::government::government_response(sim);
    let info = civ_schema::flatbuffers::root::<civ_schema::wire::Response>(&payload)
        .expect("a response")
        .body_as_government()
        .expect("the government");
    info.polities()
        .expect("polities")
        .iter()
        .flat_map(|p| p.laws().expect("laws").iter())
        .find(|x| x.id() == id.get())
        .and_then(|x| x.conduct())
        .unwrap_or_default()
        .to_owned()
}

#[test]
fn a_collector_who_objects_assesses_the_purchase_in_full() {
    let filter = 0.8;
    let (sc, duty, _) = collected(filter, 0.0, 0.5);
    let c = the_law(&sc.sim, duty).compliance;
    assert_eq!((c.assessed, c.asked, c.bribed), (1, 0, 0), "{c:?}");
    let told = conduct_of(&sc.sim, duty);
    assert!(
        told.starts_with("its collector assessed 1 purchase in full and asked a payment 0 times"),
        "{told}"
    );
    assert!(c.levied_kg > 0.0);
}

#[test]
fn a_collector_asks_a_payment_and_one_who_will_not_pay_pays_the_duty_and_holds_it_against_the_office()
 {
    // A buyer's household that does not object pays the collector, and nothing is assessed.
    let (sc, duty, _) = collected(0.0, 0.0, 0.5);
    let c = the_law(&sc.sim, duty).compliance;
    assert_eq!((c.asked, c.bribed, c.assessed), (1, 1, 0), "{c:?}");
    assert!(c.bribe_kg > 0.0 && c.levied_kg == 0.0);
    // Asked all it owes, one that objects refuses: the duty is assessed and the asking held
    // against the office.
    let (sc, duty, _) = collected(0.0, 0.79, 1.0);
    let c = the_law(&sc.sim, duty).compliance;
    assert_eq!((c.asked, c.bribed, c.assessed), (1, 0, 1), "{c:?}");
    let members = sc
        .sim
        .people()
        .household(sc.buyer)
        .expect("buyer")
        .members
        .clone();
    assert!(
        members.iter().any(|&m| sc
            .sim
            .people()
            .word
            .grievances_of(m)
            .any(|g| g.wrong == Wrong::Extorted && g.blamed == Blamed::Office(duty))),
        "the asking is held against the office"
    );
}
