//! Closure and capture (M7c slice BZ, ADR-0026 §5): a body closed to the members of the
//! organizations its polity recognizes, or of those it grants a privilege, admits their households'
//! adults alone, and nobody while there is none; the core's amendment that closes the custom offers
//! both; who an organization has come to hold is said in sentences, never a percentage; a world
//! with a closed custom saves and loads exactly.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::faction::{Admit, Decides, Faction, Member, Stake, Why};
use civ_agents::params::GoodUse;
use civ_agents::polity::{
    IssueKind, Law, LawStatus, Membership, Outcome as Decided, Sanction, Stance, StanceRecord,
};
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

/// A law of template `template` in force at `at`'s polity since today, for organization `org`,
/// sponsored by `sponsor`, known to every one of its people, its holder `holder`.
fn law(
    sim: &mut Sim,
    at: PermanentId,
    template: &str,
    org: Option<PermanentId>,
    sponsor: PermanentId,
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
        levy_share: 0.0,
        holder,
        relief_days: 0.0,
        sanction: Sanction::default(),
        hours: (0, 0),
        status: LawStatus::InForce,
        sponsor,
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
        target: org,
    };
    for &m in &members {
        law.learn(m, now.day_index());
    }
    pop.polities[pi].laws.push(law);
    id
}

/// The adults of polity 0's body now.
fn body(sim: &Sim) -> Vec<PermanentId> {
    let now = sim.now();
    let params = sim.rules().people.clone();
    sim.people()
        .body_members(&sim.land().fields, 0, now, &params)
        .into_iter()
        .map(|m| m.0)
        .collect()
}

/// The adults of household `h`, as the polity counts them.
fn adults_of(sim: &mut Sim, h: PermanentId) -> Vec<PermanentId> {
    let was = sim.people().polities[0].body.members;
    sim.people_mut_for_tests().polities[0].body.members = Membership::Adults;
    let mut v: Vec<PermanentId> = body_of(sim, h);
    sim.people_mut_for_tests().polities[0].body.members = was;
    v.sort_unstable();
    v
}

/// Those of polity 0's body now of household `h`.
fn body_of(sim: &Sim, h: PermanentId) -> Vec<PermanentId> {
    let now = sim.now();
    let params = sim.rules().people.clone();
    sim.people()
        .body_members(&sim.land().fields, 0, now, &params)
        .into_iter()
        .filter(|m| m.1 == h)
        .map(|m| m.0)
        .collect()
}

/// Polity 0's custom closed to membership `m`, as an amendment would leave it: its body and its
/// latest version.
fn close(sim: &mut Sim, m: Membership) {
    let p = &mut sim.people_mut_for_tests().polities[0];
    p.body.members = m;
    if let Some(v) = p.versions.last_mut() {
        v.body.members = m;
    }
}

fn founder_home(sc: &Scene) -> PermanentId {
    let founder = sc
        .sim
        .people()
        .factions
        .members_of(sc.org)
        .next()
        .expect("a member")
        .person;
    sc.sim.people().person(founder).expect("founder").household
}

#[test]
fn a_body_closed_to_recognized_organizations_admits_their_members_alone() {
    let mut sc = scene();
    assert_eq!(sc.sim.people().polities[0].settlement, sc.a);
    let home = founder_home(&sc);
    let founder = adult_of(&sc.sim, home);
    close(&mut sc.sim, Membership::Organized);
    assert!(body(&sc.sim).is_empty(), "it recognizes none: nobody");
    law(
        &mut sc.sim,
        sc.a,
        "core:policy/recognize_organization",
        Some(sc.org),
        founder,
        None,
    );
    assert_eq!(body(&sc.sim), adults_of(&mut sc.sim, home));
    // Closed to those it grants a privilege: nobody until it grants one.
    close(&mut sc.sim, Membership::Privileged);
    assert!(body(&sc.sim).is_empty());
    law(
        &mut sc.sim,
        sc.a,
        "core:policy/licence_tools",
        Some(sc.org),
        founder,
        None,
    );
    assert_eq!(body(&sc.sim), adults_of(&mut sc.sim, home));
    let words = sc.sim.people().polities[0].body.words();
    assert!(
        words.starts_with("The adults of households in an organization it grants a privilege"),
        "{words}"
    );
    let _ = (sc.seller, sc.sickle);
}

#[test]
fn the_core_amendment_that_closes_the_custom_offers_both_closed_bodies() {
    let sc = scene();
    let def = sc
        .sim
        .rules()
        .catalog
        .policies
        .iter()
        .find(|d| d.id == "core:policy/close_the_custom")
        .expect("in core");
    let mut members: Vec<Membership> = def.bodies.iter().map(|b| b.members).collect();
    members.sort_unstable_by_key(|m| m.code());
    members.dedup();
    assert_eq!(members, [Membership::Organized, Membership::Privileged]);
    for m in [Membership::Organized, Membership::Privileged] {
        assert_eq!(Membership::from_name(m.name()), Some(m));
        assert_eq!(Membership::from_code(m.code()), Some(m));
    }
}

#[test]
fn who_an_organization_has_come_to_hold_is_said_in_sentences() {
    let mut sc = scene();
    let home = founder_home(&sc);
    let founder = adult_of(&sc.sim, home);
    let others: Vec<PermanentId> = households_of(&sc.sim, sc.a)
        .into_iter()
        .filter(|&h| h != home)
        .take(2)
        .map(|h| adult_of(&sc.sim, h))
        .collect();
    // Its founder keeps the store, proposed its licence, and a neighbour who owed them stood for it.
    law(
        &mut sc.sim,
        sc.a,
        "core:policy/keep_store",
        None,
        others[0],
        Some(founder),
    );
    let licence = law(
        &mut sc.sim,
        sc.a,
        "core:policy/licence_tools",
        Some(sc.org),
        founder,
        None,
    );
    {
        let pop = sc.sim.people_mut_for_tests();
        let hh = |p: PermanentId| pop.person(p).expect("person").household;
        let stood = |person: PermanentId, stance, owed: f32| StanceRecord {
            person,
            household: hh(person),
            stance,
            gain: 0.0,
            regard: 0.0,
            opinion: 0.0,
            values: 0.0,
            view: 0.0,
            owed,
        };
        let stances = vec![
            stood(founder, Stance::Support, 0.0),
            stood(others[0], Stance::Support, 0.4),
            stood(others[1], Stance::Oppose, 0.0),
        ];
        pop.polities[0].law_mut(licence).expect("law").stances = stances;
    }
    let polity = sc.sim.people().polities[0].clone();
    let said = civ_sim::frames::government::capture_words(&sc.sim, &polity);
    assert_eq!(said.len(), 1, "{said:?}");
    let s = &said[0];
    assert!(
        s.contains("1 of the 2 laws passed since it was founded were proposed by its members"),
        "{s}"
    );
    assert!(
        s.contains("2 stood for them, 1 of them its members and 1 who owed its members for gifts"),
        "{s}"
    );
    assert!(s.contains("its members hold 1 of its 1 offices"), "{s}");
    assert!(!s.contains('%'), "{s}");
    // An organization with nothing to say is left out.
    let pop = sc.sim.people_mut_for_tests();
    pop.polities[0].laws.clear();
    let polity = sc.sim.people().polities[0].clone();
    assert!(civ_sim::frames::government::capture_words(&sc.sim, &polity).is_empty());
}

#[test]
fn a_world_with_a_closed_custom_saves_and_loads_exactly() {
    let mut sc = scene();
    let home = founder_home(&sc);
    let founder = adult_of(&sc.sim, home);
    law(
        &mut sc.sim,
        sc.a,
        "core:policy/recognize_organization",
        Some(sc.org),
        founder,
        None,
    );
    close(&mut sc.sim, Membership::Organized);
    sc.sim.advance_minutes(2 * DAY).expect("advances");
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = persist::save(
        &mut sc.sim,
        &saves,
        commons_persist::SaveKind::Manual,
        "closed",
    )
    .expect("saves");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(
        loaded.people().polities[0].body.members,
        Membership::Organized
    );
    let same = |a: &Sim, b: &Sim| {
        let (a, b) = (persist::encode_sections(a), persist::encode_sections(b));
        assert_eq!(a.len(), b.len());
        for (x, y) in a.iter().zip(&b) {
            assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
        }
    };
    same(&sc.sim, &loaded);
    // The scene's rules are its own, not saved: the loaded world lives under the same.
    loaded
        .rules_mut_for_tests()
        .expect("rules of its own")
        .people
        .faction
        .leave_margin = 1e9;
    sc.sim.advance_minutes(2 * DAY).expect("advances");
    loaded.advance_minutes(2 * DAY).expect("advances");
    same(&sc.sim, &loaded);
}
