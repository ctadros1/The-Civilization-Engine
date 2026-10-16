//! Organizations by stake (M7c slice BW, ADR-0026 §2): one whose household sells goods at home that
//! a duty would serve organizes others who sell them, when they know one; an applicant is admitted
//! by its rules or turned away and holds it against it, each recorded; members may live in
//! another settlement; a polity's recognition spares its members' households the polity's
//! duties on its goods; a world with one saves and loads exactly.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::faction::{Admit, Decides, Faction, FactionEventKind, Member, Stake, Why};
use civ_agents::params::GoodUse;
use civ_agents::polity::{IssueKind, Law, LawStatus, Outcome as Decided};
use civ_agents::reports::{PriceReport, ReportHow};
use civ_agents::ties::Act;
use civ_agents::word::{Blamed, Wrong};
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

fn bought_by(sim: &Sim, market: PermanentId, from: PermanentId) -> Vec<civ_agents::ledger::Trade> {
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

fn until_bought(sc: &mut Scene, days: i64) -> bool {
    for _ in 0..days * 48 {
        sc.sim.advance_minutes(30).expect("advances");
        if bought_by(&sc.sim, sc.b, sc.a)
            .iter()
            .any(|t| t.buyer == sc.buyer)
        {
            return true;
        }
    }
    false
}

/// An adult of household `h`.
fn adult_of(sim: &Sim, h: PermanentId) -> PermanentId {
    let now = sim.now();
    let x = sim.people().household(h).expect("household");
    x.members
        .iter()
        .copied()
        .find(|&m| {
            sim.people()
                .person(m)
                .is_some_and(|p| p.age_years(now) >= 16.0)
        })
        .expect("an adult")
}

/// `a` and `b` keep company at the hearth: each knows and regards the other.
fn befriend(sim: &mut Sim, a: PermanentId, b: PermanentId) {
    let day = sim.now().day_index();
    let tp = sim.rules().people.ties.clone();
    let pop = sim.people_mut_for_tests();
    for _ in 0..40 {
        pop.ties.record(a, b, Act::Hearth, 4.0, 0.0, day, &tp);
        pop.ties.record(b, a, Act::Hearth, 4.0, 0.0, day, &tp);
    }
}

/// A household of the first settlement other than the buyer's that offers sickles at home.
fn home_seller(sc: &mut Scene) -> PermanentId {
    let h = households_of(&sc.sim, sc.a)
        .into_iter()
        .find(|&h| h != sc.buyer)
        .expect("another household");
    set(
        &mut sc.sim,
        h,
        &[("core:good/sickle", 14.0), ("core:good/provisions", 40.0)],
    );
    // Its people know knapping again, as the seller's at the other door do: sickles are worth to
    // them what making them costs.
    let knapping = sc
        .sim
        .rules()
        .catalog
        .technique_index("core:technique/knapping")
        .expect("knapping");
    let known = sc
        .sim
        .people()
        .people
        .iter()
        .find_map(|(_, p)| {
            p.knows
                .iter()
                .find(|k| usize::from(k.technique) == knapping)
                .copied()
        })
        .expect("someone knows knapping");
    let members = sc
        .sim
        .people()
        .household(h)
        .expect("household")
        .members
        .clone();
    for (_, p) in sc.sim.people_mut_for_tests().people.iter_mut() {
        if members.contains(&p.id) && !p.knows.iter().any(|k| usize::from(k.technique) == knapping)
        {
            p.knows.push(known);
            p.knows.sort_by_key(|k| k.technique);
        }
    }
    let sickle = sc.sickle;
    for _ in 0..8 {
        if sc
            .sim
            .people()
            .household(h)
            .is_some_and(|x| x.offers.iter().any(|o| usize::from(o.good) == sickle))
        {
            break;
        }
        sc.sim.advance_minutes(DAY).expect("advances");
    }
    h
}

#[test]
fn one_whose_duty_would_pay_organizes_those_who_sell_the_same_goods() {
    let mut sc = scene();
    assert!(until_bought(&mut sc, 10), "a purchase away");
    let seller_home = home_seller(&mut sc);
    let founder = adult_of(&sc.sim, seller_home);
    // What a duty on tools would bring the founder's household, against what organizing costs.
    let points = sc
        .sim
        .with_ctx_for_tests(|pop, ctx| pop.stake_points_for_tests(ctx, founder, GoodUse::Tool));
    assert!(points > 0.0, "a duty would bring something: {points}");
    // Too dear to organize, what it would bring is not worth it: nothing is founded.
    let rules = sc.sim.rules_mut_for_tests().expect("rules of its own");
    rules.people.faction.organize_days = 1e6;
    sc.sim
        .with_ctx_for_tests(|pop, ctx| pop.review_belonging_for_tests(ctx, founder));
    assert!(
        sc.sim
            .people()
            .factions
            .list
            .iter()
            .all(|f| f.stake.is_none()),
        "not worth organizing"
    );
    // Cheap to organize, and knowing the seller at the other settlement's door as well as any
    // here, they organize.
    let rules = sc.sim.rules_mut_for_tests().expect("rules of its own");
    rules.people.faction.organize_days = 1e-3;
    let other = adult_of(&sc.sim, households_of(&sc.sim, sc.b)[0]);
    befriend(&mut sc.sim, founder, other);
    sc.sim
        .with_ctx_for_tests(|pop, ctx| pop.review_belonging_for_tests(ctx, founder));
    let f = sc
        .sim
        .people()
        .factions
        .list
        .iter()
        .find(|f| f.stake.is_some())
        .cloned()
        .expect("an organization by stake");
    let st = f.stake.expect("its stake");
    assert_eq!(st.purpose, GoodUse::Tool);
    assert_eq!(f.founder, founder);
    assert_eq!(f.settlement, sc.a);
    let core = &sc.sim.rules().people.faction;
    assert_eq!(
        (st.admit, st.decides),
        (core.stake_admit, core.stake_decides)
    );
    assert_eq!(
        sc.sim
            .people()
            .factions
            .membership(founder)
            .map(|m| m.faction),
        Some(f.id)
    );
}

/// An organization of tool sellers founded at the first settlement by `founder`, with `admit`
/// and `decides` its rules.
fn organization(
    sc: &mut Scene,
    founder: PermanentId,
    admit: Admit,
    decides: Decides,
) -> PermanentId {
    let id = sc.sim.allocate_id_for_tests();
    let (now, day, a) = (sc.sim.now(), sc.sim.now().day_index(), sc.a);
    let pop = sc.sim.people_mut_for_tests();
    pop.factions.list.push(Faction::organize(
        id,
        a,
        founder,
        Stake {
            purpose: GoodUse::Tool,
            admit,
            decides,
            dues_share: 0.05,
        },
        now,
    ));
    pop.factions.join(Member {
        person: founder,
        faction: id,
        since: day,
        why: Why::default(),
    });
    id
}

#[test]
fn an_applicant_is_admitted_by_its_rules_or_turned_away_and_holds_it_against_it() {
    let mut sc = scene();
    let home = home_seller(&mut sc);
    let founder = adult_of(&sc.sim, home);
    let applicant = adult_of(&sc.sim, households_of(&sc.sim, sc.b)[0]);
    // Admitted by sponsorship only: nobody in it regards them, so they are turned away.
    let closed = organization(&mut sc, founder, Admit::Sponsor, Decides::Organizer);
    let admitted = sc
        .sim
        .with_ctx_for_tests(|pop, ctx| pop.apply_for_tests(ctx, applicant, closed));
    assert!(!admitted);
    let f = sc.sim.people().factions.get(closed).cloned().expect("kept");
    assert!(
        f.history
            .iter()
            .any(|e| e.kind == FactionEventKind::TurnedAway && e.who == Some(applicant))
    );
    assert!(
        sc.sim
            .people()
            .word
            .grievances_of(applicant)
            .any(|g| g.blamed == Blamed::Organization(closed) && g.wrong == Wrong::TurnedAway),
        "they hold it against it"
    );
    // Once its member knows and regards them, they are spoken for and admitted.
    befriend(&mut sc.sim, founder, applicant);
    let admitted = sc
        .sim
        .with_ctx_for_tests(|pop, ctx| pop.apply_for_tests(ctx, applicant, closed));
    assert!(admitted);
    let f = sc.sim.people().factions.get(closed).cloned().expect("kept");
    assert!(
        f.history
            .iter()
            .any(|e| e.kind == FactionEventKind::Admitted && e.who == Some(applicant))
    );
    // A member of another settlement stays a member.
    sc.sim.advance_minutes(2 * DAY).expect("advances");
    assert_eq!(
        sc.sim
            .people()
            .factions
            .membership(applicant)
            .map(|m| m.faction),
        Some(closed)
    );
}

#[test]
fn an_organization_open_to_anyone_who_sells_its_goods_admits_a_stranger() {
    let mut sc = scene();
    let home = home_seller(&mut sc);
    let founder = adult_of(&sc.sim, home);
    let applicant = adult_of(&sc.sim, households_of(&sc.sim, sc.b)[0]);
    let open = organization(&mut sc, founder, Admit::Open, Decides::Organizer);
    assert!(
        sc.sim
            .with_ctx_for_tests(|pop, ctx| pop.apply_for_tests(ctx, applicant, open))
    );
}

#[test]
fn a_world_with_an_organization_saves_and_loads_exactly() {
    let mut sc = scene();
    let home = home_seller(&mut sc);
    let founder = adult_of(&sc.sim, home);
    let applicant = adult_of(&sc.sim, households_of(&sc.sim, sc.b)[0]);
    let id = organization(&mut sc, founder, Admit::Sponsor, Decides::Members);
    sc.sim
        .with_ctx_for_tests(|pop, ctx| pop.apply_for_tests(ctx, applicant, id));
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = persist::save(
        &mut sc.sim,
        &saves,
        commons_persist::SaveKind::Manual,
        "guild",
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
    let f = loaded.people().factions.get(id).cloned().expect("loaded");
    assert_eq!(f.stake.map(|s| s.decides), Some(Decides::Members));
    sc.sim.advance_minutes(2 * DAY).expect("advances");
    loaded.advance_minutes(2 * DAY).expect("advances");
    same(&sc.sim, &loaded);
}

/// A law of template `template` in force at settlement `at`'s polity since today, known to every
/// one of its people, at `share`, naming `target` (as `tests/trade_law.rs` lays one).
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

fn compliance_of(sim: &Sim, id: PermanentId) -> civ_agents::polity::Compliance {
    sim.people()
        .polities
        .iter()
        .flat_map(|p| &p.laws)
        .find(|l| l.id == id)
        .map(|l| l.compliance)
        .expect("kept")
}

#[test]
fn a_recognized_organization_s_members_owe_no_duty_on_its_goods() {
    // Unrecognized, the buyer's household owes the duty on the sickles it fetches.
    let mut sc = scene();
    let buyer = adult_of(&sc.sim, sc.buyer);
    organization(&mut sc, buyer, Admit::Open, Decides::Organizer);
    let duty = law(&mut sc.sim, sc.a, "core:policy/duty_on_tools", 0.5, None);
    assert!(until_bought(&mut sc, 10), "a purchase away");
    let c = compliance_of(&sc.sim, duty);
    assert_eq!(c.complied + c.evaded, 1, "{c:?}");
    // Recognized by its polity, it owes none.
    let mut sc = scene();
    let buyer = adult_of(&sc.sim, sc.buyer);
    let org = organization(&mut sc, buyer, Admit::Open, Decides::Organizer);
    let duty = law(&mut sc.sim, sc.a, "core:policy/duty_on_tools", 0.5, None);
    law(
        &mut sc.sim,
        sc.a,
        "core:policy/recognize_organization",
        0.0,
        Some(org),
    );
    assert!(until_bought(&mut sc, 10), "a purchase away");
    let c = compliance_of(&sc.sim, duty);
    assert_eq!(c.complied + c.evaded + c.unaware, 0, "{c:?}");
    // The government panel says what it is and who recognizes it.
    let lines = civ_sim::frames::government::faction_words(&sc.sim, sc.a);
    let name = sc
        .sim
        .land()
        .settlements
        .iter()
        .find(|s| s.id == sc.a)
        .map(|s| s.name.clone())
        .expect("named");
    assert!(
        lines.iter().any(|l| l.contains("of those who sell tools")
            && l.ends_with(&format!("; {name}'s gathering recognizes it"))),
        "{lines:?}"
    );
    // Recognition by another polity spares nobody here.
    let mut sc = scene();
    let buyer = adult_of(&sc.sim, sc.buyer);
    let org = organization(&mut sc, buyer, Admit::Open, Decides::Organizer);
    let duty = law(&mut sc.sim, sc.a, "core:policy/duty_on_tools", 0.5, None);
    law(
        &mut sc.sim,
        sc.b,
        "core:policy/recognize_organization",
        0.0,
        Some(org),
    );
    assert!(until_bought(&mut sc, 10), "a purchase away");
    let c = compliance_of(&sc.sim, duty);
    assert_eq!(c.complied + c.evaded, 1, "{c:?}");
}
