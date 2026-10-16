//! Trade laws (M7c slice BV, ADR-0026 §1): a duty in force is owed on goods a member's household
//! buys at another settlement's door and paid at home in kind to the store, or kept back; another
//! polity's duty binds nobody else; an embargo a member keeps turns their trip to its doors away;
//! a world with them saves and loads exactly.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::ledger::Trade;
use civ_agents::polity::{IssueKind, Law, LawStatus, Outcome as Decided};
use civ_agents::reports::{PriceReport, ReportHow};
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
        holder: None,
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

#[test]
fn a_duty_in_force_is_owed_on_tools_bought_elsewhere_and_paid_to_the_store_or_kept_back() {
    let mut sc = scene();
    let duty = law(&mut sc.sim, sc.a, "core:policy/duty_on_tools", 0.5, None);
    let pi = sc
        .sim
        .people()
        .polities
        .iter()
        .position(|p| p.settlement == sc.a)
        .expect("polity");
    let before = sc.sim.people().polities[pi]
        .stores
        .get(sc.sickle)
        .copied()
        .unwrap_or(0.0);
    let trade =
        until_bought(&mut sc, 10).expect("the buyer fetched sickles from the seller's door");
    let c = the_law(&sc.sim, duty).compliance;
    // The purchase was liable: paid, or kept back.
    assert_eq!(c.complied + c.evaded, 1, "{c:?}");
    let owed = f64::from(trade.units) * 0.5;
    let store = sc.sim.people().polities[pi]
        .stores
        .get(sc.sickle)
        .copied()
        .unwrap_or(0.0)
        - before;
    if c.complied == 1 {
        assert!(
            (store - owed).abs() < 1e-6,
            "{store} into the store, {owed} owed"
        );
        assert!((c.levied_kg - owed).abs() < 1e-6);
    } else {
        assert!(store.abs() < 1e-9, "nothing paid");
        assert!((c.withheld_kg - owed).abs() < 1e-6);
    }
}

#[test]
fn another_polity_s_duty_binds_nobody_of_this_one() {
    let mut sc = scene();
    let duty = law(&mut sc.sim, sc.b, "core:policy/duty_on_tools", 0.5, None);
    until_bought(&mut sc, 10).expect("a purchase");
    let c = the_law(&sc.sim, duty).compliance;
    assert_eq!(c.complied + c.evaded + c.unaware, 0, "{c:?}");
}

#[test]
fn an_embargo_kept_turns_the_trip_to_its_doors_away() {
    let mut free = scene();
    assert!(until_bought(&mut free, 4).is_some(), "without it, they buy");
    let mut sc = scene();
    let embargo = law(&mut sc.sim, sc.a, "core:policy/embargo", 0.0, Some(sc.b));
    let bought = until_bought(&mut sc, 4);
    let l = the_law(&sc.sim, embargo);
    // Every one of them knows it and keeps it (the norms they hold weigh toward keeping a law):
    // nobody goes, and none broke it.
    assert!(bought.is_none(), "{:?}", l.compliance);
    assert_eq!(l.compliance.broken, 0);
}

#[test]
fn a_world_with_trade_laws_saves_and_loads_exactly() {
    let mut sc = scene();
    law(&mut sc.sim, sc.a, "core:policy/duty_on_tools", 0.25, None);
    until_bought(&mut sc, 10).expect("a purchase");
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = persist::save(
        &mut sc.sim,
        &saves,
        commons_persist::SaveKind::Manual,
        "duty",
    )
    .expect("saves");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
    loaded
        .rules_mut_for_tests()
        .expect("rules of its own")
        .people
        .decision
        .w_walk_hour = 1.0;
    let same = |a: &Sim, b: &Sim| {
        let (a, b) = (persist::encode_sections(a), persist::encode_sections(b));
        assert_eq!(a.len(), b.len());
        for (x, y) in a.iter().zip(&b) {
            assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
        }
    };
    same(&sc.sim, &loaded);
    sc.sim.advance_minutes(2 * DAY).expect("advances");
    loaded.advance_minutes(2 * DAY).expect("advances");
    same(&sc.sim, &loaded);
}
