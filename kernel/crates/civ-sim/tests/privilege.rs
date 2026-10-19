//! Privileges (M7c slice BX, ADR-0026 §3): a licence binds households none of whose people belong
//! to the organization it names, each keeping it or breaking it as it reviews what it offers; a
//! member brings a breach seen at a door as a case, which the gathering hears; a workshop whose
//! owners keep a licence over what it makes, and so sells nothing, is given up as barred; a world
//! with a privilege saves and loads exactly.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::crime::CaseStage;
use civ_agents::faction::{Admit, Decides, Faction, Member, Stake, Why};
use civ_agents::firm::{Exit, Firm};
use civ_agents::params::GoodUse;
use civ_agents::polity::{IssueKind, Law, LawStatus, Outcome as Decided, Sanction};
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

fn offers_sickles(sim: &Sim, h: PermanentId, sickle: usize) -> bool {
    sim.people()
        .household(h)
        .is_some_and(|x| x.offers.iter().any(|o| usize::from(o.good) == sickle))
}

/// A village of world 10's first settlement: its largest household's adult organizes those who
/// sell tools, and the next household, outside it, offers spare sickles at its door.
struct Scene {
    sim: Sim,
    a: PermanentId,
    seller: PermanentId,
    org: PermanentId,
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
    let a = sim.land().settlements[0].id;
    let sickle = good(&sim, "core:good/sickle");
    let hs = households_of(&sim, a);
    let (member, seller) = (hs[0], hs[1]);
    set(
        &mut sim,
        seller,
        &[("core:good/sickle", 14.0), ("core:good/provisions", 40.0)],
    );
    for _ in 0..8 {
        if offers_sickles(&sim, seller, sickle) {
            break;
        }
        sim.advance_minutes(DAY).expect("advances");
    }
    assert!(offers_sickles(&sim, seller, sickle), "it offers sickles");
    let founder = adult_of(&sim, member);
    let org = sim.allocate_id_for_tests();
    let (now, day) = (sim.now(), sim.now().day_index());
    let pop = sim.people_mut_for_tests();
    pop.factions.list.push(Faction::organize(
        org,
        a,
        founder,
        Stake {
            purpose: GoodUse::Tool,
            admit: Admit::Open,
            decides: Decides::Organizer,
            dues_share: 0.05,
        },
        now,
    ));
    pop.factions.join(Member {
        person: founder,
        faction: org,
        since: day,
        why: Why::default(),
    });
    // Its founder stays in it whatever belonging is worth to them: the organization, not their
    // weighing of it, is the matter here.
    sim.rules_mut_for_tests()
        .expect("rules of its own")
        .people
        .faction
        .leave_margin = 1e9;
    Scene {
        sim,
        a,
        seller,
        org,
        sickle,
    }
}

/// The licence on tools in force at `at`'s polity since today, for organization `org`, with a
/// finding imposing `sanction`; known to every one of its people when `known`.
fn licence(
    sim: &mut Sim,
    at: PermanentId,
    org: PermanentId,
    sanction: Sanction,
    known: bool,
) -> PermanentId {
    let policies = &sim.rules().catalog.policies;
    let policy = policies
        .iter()
        .position(|d| d.id == "core:policy/licence_tools")
        .expect("the core content has the licence");
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
        levy_share: 0.0,
        holder: None,
        relief_days: 0.0,
        sanction,
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
        target: Some(org),
    };
    if known {
        for &m in &members {
            law.learn(m, now.day_index());
        }
    }
    pop.polities[pi].laws.push(law);
    id
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

const FINE: Sanction = Sanction {
    compensation_days: 3.0,
    fine_days: 2.0,
    exile: false,
    remove: false,
    yearly_days: 0.0,
};

#[test]
fn a_licence_binds_those_outside_its_organization_who_keep_it_or_break_it() {
    // Known, the seller weighs it at its next review: kept, it offers no sickles; broken, it
    // does.
    let mut sc = scene();
    let law = licence(&mut sc.sim, sc.a, sc.org, FINE, true);
    // Until the seller has weighed it: others who come to offer sickles weigh it too.
    for _ in 0..10 {
        let c = the_law(&sc.sim, law).compliance;
        let offers = offers_sickles(&sc.sim, sc.seller, sc.sickle);
        if c.broken > 0 || (c.complied > 0 && !offers) {
            break;
        }
        sc.sim.advance_minutes(DAY).expect("advances");
    }
    let c = the_law(&sc.sim, law).compliance;
    assert!(c.complied + c.broken > 0, "weighed at a review: {c:?}");
    assert_eq!(c.broken_unaware, 0);
    let offers = offers_sickles(&sc.sim, sc.seller, sc.sickle);
    if c.complied > 0 && c.broken == 0 {
        assert!(!offers, "kept: its sickles are withdrawn");
    }
    if c.broken > 0 && c.complied == 0 {
        assert!(offers, "broken: they stand at its door");
    }
    // Unknown, it breaks it unawares and keeps offering.
    let mut sc = scene();
    let law = licence(&mut sc.sim, sc.a, sc.org, FINE, false);
    for _ in 0..10 {
        if the_law(&sc.sim, law).compliance.broken_unaware > 0 {
            break;
        }
        sc.sim.advance_minutes(DAY).expect("advances");
    }
    let c = the_law(&sc.sim, law).compliance;
    assert!(c.broken_unaware > 0 && c.complied == 0, "{c:?}");
    assert!(offers_sickles(&sc.sim, sc.seller, sc.sickle));
}

#[test]
fn a_member_brings_a_breach_seen_at_a_door_and_it_is_heard() {
    let mut sc = scene();
    // Nobody heard of it, so the seller keeps offering.
    let law = licence(&mut sc.sim, sc.a, sc.org, FINE, false);
    // The case against the seller: others who come to offer sickles may be brought too.
    let seller = sc.seller;
    let case = |sim: &Sim| {
        sim.people()
            .order
            .cases
            .iter()
            .find(|c| c.law == law && c.accused_household == seller)
            .cloned()
    };
    for _ in 0..40 {
        if case(&sc.sim).is_some() {
            break;
        }
        sc.sim.advance_minutes(DAY).expect("advances");
    }
    let c = case(&sc.sim).expect("a member brought the breach");
    assert_eq!(c.incident, 0);
    assert!(
        sc.sim
            .people()
            .chronicle
            .iter()
            .any(|e| e.name.contains("for breaking") && e.name.contains("organization's privilege")),
        "the chronicle says it was brought"
    );
    for _ in 0..40 {
        if case(&sc.sim).is_some_and(|c| c.stage != CaseStage::Open) {
            break;
        }
        sc.sim.advance_minutes(DAY).expect("advances");
    }
    let c = case(&sc.sim).expect("kept");
    assert!(
        matches!(
            c.stage,
            CaseStage::Found | CaseStage::NotFound | CaseStage::Unheard
        ),
        "{:?}",
        c.stage
    );
    if c.stage == CaseStage::Found {
        assert!(
            sc.sim
                .people()
                .order
                .obligations
                .iter()
                .any(|o| o.case == Some(c.id) && o.debtor == sc.seller)
        );
        // What a breach's finding imposed names no incident: it is answered all the same, and
        // a world holding it saves and loads.
        sc.sim.advance_minutes(3 * DAY).expect("advances");
        assert!(
            sc.sim
                .people()
                .order
                .obligations
                .iter()
                .filter(|o| o.case == Some(c.id))
                .all(|o| o.answer.is_some()),
            "answered: {:?}",
            sc.sim.people().order.obligations
        );
        let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
        let saves =
            commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
        let saved = persist::save(
            &mut sc.sim,
            &saves,
            commons_persist::SaveKind::Manual,
            "breach",
        )
        .expect("saves");
        persist::load(&saved.path, content()).expect("loads");
    }
}

#[test]
fn a_workshop_whose_owners_keep_a_licence_over_what_it_makes_is_given_up_as_barred() {
    let mut sc = scene();
    let law = licence(&mut sc.sim, sc.a, sc.org, FINE, true);
    let id = sc.sim.allocate_id_for_tests();
    let (now, goods) = (sc.sim.now(), sc.sim.rules().catalog.goods.len());
    let founder = adult_of(&sc.sim, sc.seller);
    let long_ago = now.plus_minutes(-200 * DAY);
    let mut f = Firm::new(
        id,
        sc.seller,
        founder,
        Some(sc.a),
        sc.sickle as u16,
        goods,
        long_ago,
    );
    f.last_sale = Some(long_ago);
    // It holds sickles to sell, which its owners withdraw as they keep the licence.
    f.stores[sc.sickle] = 6.0;
    sc.sim.people_mut_for_tests().firms.push(f);
    for _ in 0..10 {
        if sc.sim.people().firm(id).is_some_and(|f| !f.is_open()) {
            break;
        }
        sc.sim.advance_minutes(DAY).expect("advances");
    }
    let f = sc.sim.people().firm(id).expect("kept on record");
    let why = f.closed.map(|(_, why)| why).expect("given up");
    // It held sickles and offered none when it closed, and the licence binds its owners.
    assert!(the_law(&sc.sim, law).status == LawStatus::InForce);
    assert_eq!(why, Exit::Barred, "{}", why.text());
}

#[test]
fn a_world_with_a_privilege_saves_and_loads_exactly() {
    let mut sc = scene();
    licence(&mut sc.sim, sc.a, sc.org, FINE, true);
    sc.sim.advance_minutes(3 * DAY).expect("advances");
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = persist::save(
        &mut sc.sim,
        &saves,
        commons_persist::SaveKind::Manual,
        "licence",
    )
    .expect("saves");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
    let same = |a: &Sim, b: &Sim| {
        let (a, b) = (persist::encode_sections(a), persist::encode_sections(b));
        assert_eq!(a.len(), b.len());
        for (x, y) in a.iter().zip(&b) {
            assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
        }
    };
    same(&sc.sim, &loaded);
    sc.sim.advance_minutes(3 * DAY).expect("advances");
    loaded.advance_minutes(3 * DAY).expect("advances");
    same(&sc.sim, &loaded);
}
