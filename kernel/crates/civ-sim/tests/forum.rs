//! Forums (M7b slice BR, ADR-0025 §1): a *hear cases* law names who hears the cases brought under
//! the laws against taking and raiding. A case brought under one is heard the evening after by
//! the forum, not the gathering: a named hearer alone, a panel drawn by a keyed lot from the
//! body's members, or the elders of the households, never anyone of the two households. A forum
//! whose law has ended, or a hearer of one of the two households, leaves the case to the
//! gathering. Before a forum sits, a case may be settled or withdrawn. A case before a forum saves
//! and loads exactly.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::crime::{Case, CaseStage, Incident, Outcome as Ended};
use civ_agents::polity::{IssueKind, Law, LawStatus, Outcome as Decided, Stance};
use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_sim::persist;
use civ_sim::{NewWorld, Sim};

const DAY: i64 = 24 * 60;

type Household = (PermanentId, Vec<PermanentId>);

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
    let mut sim = Sim::create_for_tests(
        &NewWorld {
            name: "Forum".to_owned(),
            seed,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 512,
            band_size: 0,
            neighbours: Vec::new(),
            neighbours_known: false,
            regime_id: String::new(),
        },
        content(),
        [7; 16],
    )
    .expect("generates");
    sim.advance_minutes(2 * DAY).expect("advances");
    // The hearings under test, not the tuning: no accused offers to settle.
    sim.rules_mut_for_tests()
        .expect("rules not yet shared")
        .people
        .crime
        .comply_base = -100.0;
    sim
}

/// The settlement's households with grown members, each with its grown members, in id order.
fn households(sim: &Sim) -> Vec<Household> {
    let (now, grown) = (sim.now(), sim.rules().people.family.independent_age);
    let pop = sim.people();
    let settlement = pop.polities[0].settlement;
    let mut out: Vec<Household> = pop
        .households
        .iter()
        .filter(|(_, h)| h.settlement == Some(settlement))
        .map(|(_, h)| {
            let mut adults: Vec<PermanentId> = h
                .members
                .iter()
                .copied()
                .filter(|&m| pop.person(m).is_some_and(|p| p.age_years(now) >= grown))
                .collect();
            adults.sort_unstable();
            (h.id, adults)
        })
        .filter(|(_, a)| !a.is_empty())
        .collect();
    out.sort_unstable();
    out
}

/// A law of template `template` in force at the first polity, naming `holder`, known to all.
fn law(sim: &mut Sim, template: &str, holder: Option<PermanentId>) -> PermanentId {
    let policies = &sim.rules().catalog.policies;
    let policy = policies
        .iter()
        .position(|d| d.id == template)
        .expect("the core content has the template");
    let kind = policies[policy].kind;
    let sanction = policies[policy]
        .bundles
        .first()
        .copied()
        .unwrap_or_default();
    let id = sim.allocate_id_for_tests();
    let now = sim.now();
    let everyone: Vec<PermanentId> = households(sim).into_iter().flat_map(|h| h.1).collect();
    let pop = sim.people_mut_for_tests();
    let mut law = Law {
        id,
        policy: policy as u16,
        kind,
        levy_share: 0.0,
        holder,
        relief_days: 0.0,
        sanction,
        hours: (0, 0),
        status: LawStatus::InForce,
        sponsor: everyone[0],
        proposed: now,
        issue: IssueKind::CasesWaiting,
        meets_day: now.day_index(),
        decided: Some(now),
        outcome: Some(Decided::Passed),
        eligible: everyone.len() as u32,
        stances: Vec::new(),
        known: Vec::new(),
        compliance: Default::default(),
        watch: Default::default(),
        body: None,
        ends: None,
        agreement: None,
        target: None,
    };
    for &a in &everyone {
        law.learn(a, now.day_index());
    }
    pop.polities[0].laws.push(law);
    id
}

/// An open case brought today by the first adult of household `accuser` against the first adult
/// of household `accused`, under a law against taking now in force: its id.
fn case(sim: &mut Sim, accuser: &Household, accused: &Household) -> u32 {
    case_by(sim, accuser, accused, accused.1[0])
}

/// As [`case`], where it was `actor` who took (the accused may not have).
fn case_by(sim: &mut Sim, accuser: &Household, accused: &Household, actor: PermanentId) -> u32 {
    let against = law(sim, "core:policy/against_taking", None);
    let grain = sim
        .rules()
        .catalog
        .good_index("core:good/grain")
        .expect("in core");
    let now = sim.now();
    let day = now.day_index();
    let pop = sim.people_mut_for_tests();
    let actor_household = pop.person(actor).map_or(accused.0, |p| p.household);
    let settlement = pop.polities[0].settlement;
    // What it is about: a taking that household `accuser` says happened.
    let incident = pop.order.incidents.len() as u32 + 1;
    pop.order.incidents.push(Incident {
        id: incident,
        at: now,
        actor,
        actor_household,
        target: accuser.0,
        place: (0.0, 0.0),
        outcome: Ended::Taken,
        goods: vec![(grain as u16, 3.0)],
        kcal: 8_000.0,
        seen_by: vec![accuser.1[0]],
        noticed: true,
        food_days: f32::NAN,
        richer: f32::NAN,
        across: None,
    });
    let id = pop.order.cases.last().map_or(1, |c| c.id + 1);
    pop.order.cases.push(Case {
        id,
        incident,
        settlement,
        law: against,
        accuser: accuser.0,
        by: accuser.1[0],
        accused: accused.1[0],
        accused_household: accused.0,
        kcal: 8_000.0,
        leads: vec![accuser.1[0]],
        opened: day,
        stage: CaseStage::Open,
        heard: None,
        eligible: 0,
        stances: Vec::new(),
        exiled: false,
        forum: None,
        challenge: None,
        tags: 0,
        departed: None,
        loan: None,
        building: None,
    });
    id
}

fn find(sim: &Sim, id: u32) -> Case {
    sim.people()
        .order
        .cases
        .iter()
        .find(|c| c.id == id)
        .cloned()
        .expect("the case")
}

fn before_the_gathering(sim: &Sim, id: u32) -> bool {
    sim.people().polities[0]
        .gathering
        .as_ref()
        .is_some_and(|g| g.cases.contains(&id))
}

#[test]
fn a_named_hearer_hears_a_case_alone_the_evening_after_it_is_brought() {
    let mut sim = world(3);
    let hh = households(&sim);
    assert!(hh.len() >= 3, "three households: {hh:?}");
    let hearer = hh[2].1[0];
    let forum = law(&mut sim, "core:policy/name_a_hearer", Some(hearer));
    let id = case(&mut sim, &hh[0], &hh[1]);
    // The midnight after it is brought, it is put before the forum; the next, heard.
    sim.advance_minutes(DAY).expect("advances");
    let c = find(&sim, id);
    assert_eq!(c.forum, Some(forum), "{c:?}");
    assert!(
        !before_the_gathering(&sim, id),
        "a case before a forum is not before the gathering"
    );
    sim.advance_minutes(DAY).expect("advances");
    let c = find(&sim, id);
    assert_ne!(c.stage, CaseStage::Open, "{c:?}");
    assert_eq!(c.eligible, 1);
    assert_eq!(c.stances.len(), 1, "the hearer alone: {:?}", c.stances);
    assert_eq!(c.stances[0].person, hearer);
    // One holder finds only by standing for a finding.
    assert_eq!(
        c.stage == CaseStage::Found,
        c.stances[0].stance == Stance::Support,
        "{c:?}"
    );
    assert!(
        sim.people()
            .chronicle
            .iter()
            .any(|e| e.name.starts_with("The hearer ")),
        "the chronicle names the hearer"
    );
}

#[test]
fn a_panel_is_drawn_by_lot_and_the_elders_sit_without_the_two_households() {
    for (template, panel) in [
        ("core:policy/hear_by_panel", true),
        ("core:policy/hear_by_elders", false),
    ] {
        let mut sim = world(4);
        let hh = households(&sim);
        let forum = law(&mut sim, template, None);
        let id = case(&mut sim, &hh[0], &hh[1]);
        sim.advance_minutes(2 * DAY).expect("advances");
        let c = find(&sim, id);
        assert_eq!(c.forum, Some(forum), "{template}: {c:?}");
        assert_ne!(c.stage, CaseStage::Open, "{template}: {c:?}");
        assert!(!c.stances.is_empty(), "{template}: somebody sat");
        assert_eq!(
            c.eligible as usize,
            c.stances.len(),
            "{template}: all who sit come"
        );
        for r in &c.stances {
            assert!(
                r.household != hh[0].0 && r.household != hh[1].0,
                "{template}: one of the two households sat: {r:?}"
            );
        }
        if panel {
            assert!(c.stances.len() <= 5, "{:?}", c.stances);
        } else {
            // One elder of each household sits.
            let mut seen: Vec<PermanentId> = c.stances.iter().map(|r| r.household).collect();
            seen.sort_unstable();
            seen.dedup();
            assert_eq!(
                seen.len(),
                c.stances.len(),
                "one of each household: {:?}",
                c.stances
            );
        }
    }
}

#[test]
fn without_its_forum_or_with_a_hearer_of_a_party_a_case_waits_for_the_gathering() {
    // The hearer is of the household that brought it.
    let mut sim = world(3);
    let hh = households(&sim);
    law(&mut sim, "core:policy/name_a_hearer", Some(hh[0].1[0]));
    let id = case(&mut sim, &hh[0], &hh[1]);
    sim.advance_minutes(DAY).expect("advances");
    assert_eq!(find(&sim, id).forum, None);
    assert!(
        before_the_gathering(&sim, id),
        "the gathering is called for it"
    );
    // The forum's law ends before it sits.
    let mut sim = world(3);
    let hh = households(&sim);
    let forum = law(&mut sim, "core:policy/name_a_hearer", Some(hh[2].1[0]));
    let id = case(&mut sim, &hh[0], &hh[1]);
    sim.advance_minutes(DAY).expect("advances");
    assert_eq!(find(&sim, id).forum, Some(forum));
    sim.people_mut_for_tests().polities[0]
        .laws
        .iter_mut()
        .find(|l| l.id == forum)
        .expect("the forum's law")
        .status = LawStatus::Lapsed;
    sim.advance_minutes(DAY).expect("advances");
    let c = find(&sim, id);
    assert_eq!(c.forum, None, "{c:?}");
    assert_eq!(c.stage, CaseStage::Open, "{c:?}");
    assert!(before_the_gathering(&sim, id), "it waits for the gathering");
}

#[test]
fn a_case_before_a_forum_saves_and_loads_exactly() {
    let mut sim = world(3);
    let hh = households(&sim);
    law(&mut sim, "core:policy/hear_by_panel", None);
    case(&mut sim, &hh[0], &hh[1]);
    sim.advance_minutes(DAY).expect("advances");
    assert!(sim.people().order.cases.iter().any(|c| c.forum.is_some()));
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(&mut sim, &saves, commons_persist::SaveKind::Manual, "forum").expect("saves");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
    // The nudge lives only in the run: the loaded world takes it again.
    loaded
        .rules_mut_for_tests()
        .expect("rules not yet shared")
        .people
        .crime
        .comply_base = -100.0;
    let same = |a: &Sim, b: &Sim| {
        let (a, b) = (persist::encode_sections(a), persist::encode_sections(b));
        assert_eq!(a.len(), b.len());
        for (x, y) in a.iter().zip(&b) {
            assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
        }
    };
    same(&sim, &loaded);
    sim.advance_minutes(2 * DAY).expect("advances");
    loaded.advance_minutes(2 * DAY).expect("advances");
    same(&sim, &loaded);
}

#[test]
fn before_a_forum_sits_a_case_may_be_settled_or_withdrawn() {
    use civ_agents::crime::{Obligation, Owed, Standing};
    // The accused's household offers back what was taken, and the one who brought it, with no
    // compensation to hope for and one account behind it, accepts.
    let mut sim = world(3);
    {
        let crime = &mut sim.rules_mut_for_tests().expect("rules").people.crime;
        crime.comply_base = 100.0;
    }
    let hh = households(&sim);
    law(&mut sim, "core:policy/hear_by_panel", None);
    let id = case(&mut sim, &hh[0], &hh[1]);
    let against = find(&sim, id).law;
    for l in &mut sim.people_mut_for_tests().polities[0].laws {
        if l.id == against {
            l.sanction.compensation_days = 0.0;
        }
    }
    sim.advance_minutes(2 * DAY).expect("advances");
    let c = find(&sim, id);
    assert_eq!(c.stage, CaseStage::Settled, "{c:?}");
    assert!(c.stances.is_empty(), "nobody heard it");
    assert!(c.closure_days().is_some_and(|d| d >= 1), "{c:?}");
    let owed: Vec<&Obligation> = sim
        .people()
        .order
        .obligations
        .iter()
        .filter(|o| o.case == Some(id))
        .collect();
    assert_eq!(owed.len(), 1, "{owed:?}");
    assert_eq!(owed[0].kind, Owed::Restitution);
    assert_eq!((owed[0].debtor, owed[0].beneficiary), (hh[1].0, hh[0].0));
    // What was taken was given back on demand before the sitting: the case is withdrawn.
    let mut sim = world(3);
    let hh = households(&sim);
    law(&mut sim, "core:policy/hear_by_panel", None);
    let id = case(&mut sim, &hh[0], &hh[1]);
    let incident = find(&sim, id).incident;
    let day = sim.now().day_index();
    let order = &mut sim.people_mut_for_tests().order;
    let oid = order.obligations.last().map_or(1, |o| o.id + 1);
    order.obligations.push(Obligation {
        id: oid,
        incident,
        kind: Owed::Demanded,
        case: None,
        debtor: hh[1].0,
        beneficiary: hh[0].0,
        kcal: 8_000.0,
        paid_kcal: 8_000.0,
        made: day,
        due: day + 7,
        standing: Standing::Met,
        answer: None,
    });
    sim.advance_minutes(2 * DAY).expect("advances");
    let c = find(&sim, id);
    assert_eq!(c.stage, CaseStage::Withdrawn, "{c:?}");
    assert!(
        sim.people()
            .chronicle
            .iter()
            .any(|e| e.name.contains(" withdrew the case ")),
        "the chronicle says it was withdrawn"
    );
}

/// `holder` believes the accused of case `id` took, having seen it.
fn believe(sim: &mut Sim, holder: PermanentId, id: u32) {
    let taker = find(sim, id).accused;
    believe_took(sim, holder, id, taker);
}

/// `holder` believes `taker` took in the incident of case `id`, having seen it.
fn believe_took(sim: &mut Sim, holder: PermanentId, id: u32, taker: PermanentId) {
    use civ_agents::crime::{Belief, Source};
    let c = find(sim, id);
    let day = sim.now().day_index();
    let order = &mut sim.people_mut_for_tests().order;
    let at = order
        .beliefs
        .partition_point(|b| (b.holder, b.incident) < (holder, c.incident));
    order.beliefs.insert(
        at,
        Belief {
            holder,
            incident: c.incident,
            taker: Some(taker),
            source: Source::Saw,
            from: None,
            origin: None,
            day,
        },
    );
}

#[test]
fn findings_are_remembered_and_recalled_as_far_as_the_forum_s_authority_goes() {
    use civ_agents::word::ClaimKind;
    for (template, moves) in [
        ("core:policy/name_a_hearer", false),
        ("core:policy/hearer_by_precedent", true),
    ] {
        let mut sim = world(3);
        let hh = households(&sim);
        let hearer = hh[2].1[0];
        law(&mut sim, template, Some(hearer));
        let first = case(&mut sim, &hh[0], &hh[1]);
        sim.advance_minutes(2 * DAY).expect("advances");
        let a = find(&sim, first);
        let dir = match a.stage {
            CaseStage::Found => 1.0,
            CaseStage::NotFound => -1.0,
            _ => panic!("{template}: heard and decided: {a:?}"),
        };
        // The hearer and the two parties hold the finding (ADR-0025 §2).
        let pop = sim.people();
        let held: Vec<PermanentId> = pop
            .word
            .heard
            .iter()
            .filter(|h| {
                pop.word
                    .claim(h.claim)
                    .is_some_and(|c| c.kind == ClaimKind::Finding && c.event == Some(first))
            })
            .map(|h| h.holder)
            .collect();
        for p in [hearer, hh[0].1[0], hh[1].1[0]] {
            assert!(held.contains(&p), "{template}: {p} holds it: {held:?}");
        }
        // A second of the same facts, which the hearer saw.
        let second = case(&mut sim, &hh[0], &hh[1]);
        believe(&mut sim, hearer, second);
        sim.advance_minutes(2 * DAY).expect("advances");
        let b = find(&sim, second);
        assert_eq!(b.stances.len(), 1, "{b:?}");
        let r = &b.stances[0];
        if moves {
            assert_eq!(
                f64::from(r.precedent).signum(),
                dir,
                "{template}: the remembered finding pulls: {r:?}"
            );
        } else {
            assert_eq!(r.precedent, 0.0, "{template}: informative: {r:?}");
        }
        if a.stage == CaseStage::NotFound {
            assert_eq!(b.tags, a.tags, "the same facts");
        }
        let departs = b.tags == a.tags
            && matches!(b.stage, CaseStage::Found | CaseStage::NotFound)
            && b.stage != a.stage;
        assert_eq!(
            b.departed,
            departs.then_some(first),
            "{template}: {a:?} {b:?}"
        );
        // In this world the first was not found. The hearer who saw the second finds it when
        // informative, departing from the first; bound by the first, they keep to it.
        assert_eq!(a.stage, CaseStage::NotFound);
        if moves {
            assert_eq!(b.stage, CaseStage::NotFound, "{b:?}");
        } else {
            assert_eq!((b.stage, b.departed), (CaseStage::Found, Some(first)));
        }
    }
}

#[test]
fn a_finding_grieved_blames_the_forum_and_findings_are_a_norm_of_their_own() {
    use civ_agents::word::Blamed;
    let mut sim = world(3);
    let hh = households(&sim);
    let j = 3;
    let forum = law(&mut sim, "core:policy/name_a_hearer", Some(hh[2].1[0]));
    // It was someone of a third household who took, not the accused.
    let id = case_by(&mut sim, &hh[0], &hh[j], hh[1].1[0]);
    // The one who brought it believes the accused took; the hearer saw who did.
    believe(&mut sim, hh[0].1[0], id);
    believe_took(&mut sim, hh[2].1[0], id, hh[1].1[0]);
    sim.advance_minutes(2 * DAY).expect("advances");
    let c = find(&sim, id);
    assert_eq!(c.stage, CaseStage::NotFound, "{c:?}");
    let pop = sim.people();
    // The one it wronged holds it against the hearer's forum, not the gathering (ADR-0025 §5).
    let theirs: Vec<_> = pop.word.grievances_of(hh[0].1[0]).collect();
    assert!(
        theirs.iter().any(|g| g.blamed == Blamed::Forum(forum)),
        "{c:?} {theirs:?}"
    );
    assert!(
        !pop.word
            .grievances
            .iter()
            .any(|g| g.blamed == Blamed::Body(pop.polities[0].id) && g.law == c.law),
        "nobody blames the gathering for it"
    );
    // Everyone holds the norm that findings bind apart from the gathering's.
    let k = sim
        .rules()
        .catalog
        .norms
        .iter()
        .position(|n| n.id == "core:norm/findings_bind")
        .expect("in core") as u16;
    for p in households(&sim).iter().flat_map(|h| h.1.clone()) {
        assert!(pop.norms.state(p, k).is_some(), "{p} holds it");
    }
}

fn reloaded(sim: &mut Sim) -> Sim {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "forum").expect("saves");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
    loaded
        .rules_mut_for_tests()
        .expect("rules not yet shared")
        .people
        .crime
        .comply_base = -100.0;
    let (a, b) = (
        persist::encode_sections(sim),
        persist::encode_sections(&loaded),
    );
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(&b) {
        assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
    }
    loaded
}

fn offices(sim: &Sim) -> Vec<String> {
    let payload = civ_sim::frames::government::government_response(sim);
    let info = civ_schema::flatbuffers::root::<civ_schema::wire::Response>(&payload)
        .expect("a response")
        .body_as_government()
        .expect("the government");
    info.polities()
        .expect("polities")
        .get(0)
        .offices()
        .expect("offices")
        .iter()
        .map(str::to_owned)
        .collect()
}

/// A hearer held for a term (backlog D, content API 226; research 09-07 §1.3: rotation and
/// lottery are selection primitives of their own): proposed, a hearer by lot names nobody; passed
/// by the gathering, a lot draws the hearer from those who stood for it, who hears the cases
/// brought alone until the term's end; then the office lapses, cases go to the gathering again,
/// and the one who heard them is not drawn, nor named under a template that does not renew, for
/// the next term. All of it saves and loads exactly.
#[test]
fn a_hearer_drawn_by_lot_hears_for_a_term_and_is_not_drawn_again_at_once() {
    use civ_agents::ChronicleKind;
    use civ_agents::polity::Term;
    let mut sim = world(3);
    let hh = households(&sim);
    assert!(hh.len() >= 4, "four households: {hh:?}");
    let policies = sim.rules().catalog.policies.clone();
    let at = |id: &str| policies.iter().position(|d| d.id == id).expect("in core") as u16;
    let lot = at("core:policy/zestfully_draw_a_hearer_by_lot");
    let named = at("core:policy/zestfully_name_a_hearer_for_a_year");
    let tenure = policies[usize::from(lot)].tenure.expect("held for a term");
    assert!(tenure.by_lot && !tenure.renewable);
    let (pi, settlement) = (0, sim.people().polities[0].settlement);
    let nominee = |sim: &mut Sim, policy: u16, d: PermanentId| {
        sim.with_ctx_for_tests(|pop, ctx| pop.hearer_nominee_for_tests(ctx, settlement, policy, d))
    };
    // Proposed, a hearer by lot names nobody; one named names whom the proposer regards most.
    let sponsor = hh[0].1[0];
    assert_eq!(nominee(&mut sim, lot, sponsor), Some(None));
    assert_eq!(
        nominee(&mut sim, named, sponsor),
        nominee(&mut sim, at("core:policy/name_a_hearer"), sponsor)
    );
    let id = sim
        .with_ctx_for_tests(|pop, ctx| {
            pop.propose_for_tests(ctx, settlement, sponsor, lot, IssueKind::CasesWaiting)
        })
        .expect("proposed");
    let everyone: Vec<PermanentId> = hh.iter().flat_map(|h| h.1.clone()).collect();
    let today = sim.now().day_index();
    let sit = |sim: &mut Sim, present: Vec<PermanentId>| {
        let g = sim.people_mut_for_tests().polities[pi]
            .gathering
            .as_mut()
            .expect("called");
        g.day = today;
        g.present = present;
        sim.with_ctx_for_tests(|pop, ctx| pop.decide_gathering_for_tests(ctx, pi));
        sim.people().polities[pi]
            .laws
            .iter()
            .find(|l| l.id == id)
            .expect("the law")
            .clone()
    };
    let mut again = reloaded(&mut sim);
    let mut decided = sit(&mut sim, everyone.clone());
    if decided.outcome != Some(Decided::Passed) {
        // The same gathering, in the world as it was before it sat, with only those who stood
        // for it there and any number enough.
        let present: Vec<PermanentId> = decided
            .stances
            .iter()
            .filter(|r| r.stance == Stance::Support)
            .map(|r| r.person)
            .collect();
        again.people_mut_for_tests().polities[pi].body.quorum_share = 0.0;
        decided = sit(&mut again, present);
        sim = again;
    }
    assert_eq!(decided.outcome, Some(Decided::Passed), "{decided:?}");
    assert_eq!(decided.status, LawStatus::InForce);
    // The lot drew one of those who stood for it, and holds them for its term.
    let stood: Vec<PermanentId> = decided
        .stances
        .iter()
        .filter(|r| r.stance == Stance::Support)
        .map(|r| r.person)
        .collect();
    let hearer = decided.holder.expect("the lot drew a hearer");
    assert!(stood.contains(&hearer), "{hearer:?} of {stood:?}");
    let term = sim.people().polities[pi].term_of(id).expect("a term");
    assert_eq!(term.until, today + i64::from(tenure.term_days));
    assert_eq!(term.pool as usize, stood.len());
    assert!(
        sim.people()
            .chronicle
            .iter()
            .any(|e| e.kind == ChronicleKind::LotDrawn
                && e.people == vec![hearer]
                && e.name.contains("for the office of hearer by lot")),
        "the chronicle tells the lot"
    );
    let line = |sim: &Sim| {
        offices(sim)
            .into_iter()
            .find(|o| o.starts_with("Hearer by lot: "))
            .expect("the office listed")
    };
    let held = line(&sim);
    assert!(
        held.contains(", until ") && held.contains("drawn by lot"),
        "{held}"
    );
    // They hear a case between two other households alone.
    let theirs = sim.people().person(hearer).expect("alive").household;
    let others: Vec<Household> = hh.iter().filter(|h| h.0 != theirs).cloned().collect();
    let case_id = case(&mut sim, &others[0], &others[1]);
    sim.advance_minutes(DAY).expect("advances");
    assert_eq!(find(&sim, case_id).forum, Some(id));
    sim.advance_minutes(DAY).expect("advances");
    let c = find(&sim, case_id);
    assert_ne!(c.stage, CaseStage::Open, "{c:?}");
    assert_eq!(
        c.stances.iter().map(|r| r.person).collect::<Vec<_>>(),
        vec![hearer]
    );
    let mut sim = reloaded(&mut sim);
    // The term ends: the office lapses, and says so.
    {
        let p = &mut sim.people_mut_for_tests().polities[pi];
        let t = p.terms.iter_mut().find(|t| t.0 == id).expect("its term");
        t.1.until = today + 3;
    }
    sim.advance_minutes(2 * DAY).expect("advances");
    let lapsed = |sim: &Sim| {
        sim.people().polities[pi]
            .laws
            .iter()
            .find(|l| l.id == id)
            .expect("kept")
            .status
    };
    assert_eq!(lapsed(&sim), LawStatus::Lapsed);
    assert!(
        sim.people()
            .chronicle
            .iter()
            .any(|e| e.kind == ChronicleKind::LawLapsed
                && e.people == vec![hearer]
                && e.name.contains("no longer hears cases")
                && e.name.ends_with("their term ended.")),
        "the chronicle tells the term's end"
    );
    assert!(line(&sim).contains("vacant, since "), "{}", line(&sim));
    // A case brought now waits for the gathering.
    let case_id = case(&mut sim, &others[0], &others[1]);
    sim.advance_minutes(DAY).expect("advances");
    assert_eq!(find(&sim, case_id).forum, None);
    assert!(before_the_gathering(&sim, case_id));
    // The next lot cannot fall to the last hearer: with them and one other in, it draws the
    // other, whatever the key.
    let other = others[0].1[0];
    for k in 0..4 {
        let next = law(&mut sim, "core:policy/zestfully_draw_a_hearer_by_lot", None);
        let stances = [hearer, other]
            .iter()
            .map(|&person| civ_agents::polity::StanceRecord {
                person,
                household: sim.people().person(person).expect("alive").household,
                stance: Stance::Support,
                gain: 0.0,
                regard: 0.0,
                opinion: 0.0,
                values: 0.0,
                view: 0.0,
                owed: 0.0,
            })
            .collect();
        sim.people_mut_for_tests().polities[pi]
            .law_mut(next)
            .expect("the law")
            .stances = stances;
        sim.with_ctx_for_tests(|pop, ctx| pop.fill_term_for_tests(ctx, pi, next));
        let p = &mut sim.people_mut_for_tests().polities[pi];
        assert_eq!(
            p.law_mut(next).expect("the law").holder,
            Some(other),
            "draw {k}"
        );
        p.laws.retain(|l| l.id != next);
        p.terms.retain(|t| t.0 != next);
    }
    // Named for a year, one whose term has ended may be named again; under a template that
    // does not renew, not at once.
    let first = law(
        &mut sim,
        "core:policy/zestfully_name_a_hearer_for_a_year",
        Some(hearer),
    );
    {
        let p = &mut sim.people_mut_for_tests().polities[pi];
        p.law_mut(first).expect("the law").status = LawStatus::Lapsed;
        p.terms.push((
            first,
            Term {
                until: today,
                pool: 0,
            },
        ));
    }
    assert_eq!(nominee(&mut sim, named, hearer), Some(Some(hearer)));
    sim.rules_mut_for_tests()
        .expect("rules not yet shared")
        .catalog
        .policies[usize::from(named)]
    .tenure = Some(civ_agents::polity::Tenure {
        term_days: 360,
        renewable: false,
        by_lot: false,
    });
    let next = nominee(&mut sim, named, hearer)
        .expect("someone")
        .expect("named");
    assert_ne!(next, hearer);
}

/// A hearer's term ends while a case waits before them (backlog D; research 02 §3.6: a term's
/// expiry is a procedure of its own; 09-14 §2.2: a vacancy calls for the decisions it needs at
/// once; 09-07 §4.2: a case waits until it is closed, never dropped): the hearer, out of office,
/// does not hear it; it goes to the gathering, called for it, which hears it; or, where a
/// successor holds the forum by that day, before the successor, who hears it the evening after.
#[test]
fn a_case_waiting_before_a_hearer_whose_term_ends_goes_to_the_gathering_or_their_successor() {
    use civ_agents::ChronicleKind;
    use civ_agents::polity::Term;
    for successor in [false, true] {
        let mut sim = world(3);
        let hh = households(&sim);
        assert!(hh.len() >= 4, "four households: {hh:?}");
        let (pi, today) = (0, sim.now().day_index());
        let hearer = hh[2].1[0];
        let forum = law(
            &mut sim,
            "core:policy/zestfully_name_a_hearer_for_a_year",
            Some(hearer),
        );
        // Its term ends the midnight the case would be heard.
        sim.people_mut_for_tests().polities[pi].set_term(
            forum,
            Term {
                until: today + 2,
                pool: 0,
            },
        );
        let id = case(&mut sim, &hh[0], &hh[1]);
        sim.advance_minutes(DAY).expect("advances");
        assert_eq!(
            find(&sim, id).forum,
            Some(forum),
            "it waits before the hearer"
        );
        let next = successor.then(|| {
            let next = law(
                &mut sim,
                "core:policy/zestfully_name_a_hearer_for_a_year",
                Some(hh[3].1[0]),
            );
            sim.people_mut_for_tests().polities[pi].set_term(
                next,
                Term {
                    until: today + 362,
                    pool: 0,
                },
            );
            next
        });
        sim.advance_minutes(DAY).expect("advances");
        let pop = sim.people();
        let status = pop.polities[pi]
            .laws
            .iter()
            .find(|l| l.id == forum)
            .expect("kept")
            .status;
        assert_eq!(status, LawStatus::Lapsed);
        assert!(
            pop.chronicle
                .iter()
                .any(|e| e.kind == ChronicleKind::LawLapsed
                    && e.people == vec![hearer]
                    && e.name.ends_with("their term ended.")),
            "the chronicle tells the term's end"
        );
        // The hearer whose term ended did not hear it.
        let c = find(&sim, id);
        assert_eq!(c.stage, CaseStage::Open, "{c:?}");
        assert!(c.stances.is_empty(), "{c:?}");
        let mut sim = reloaded(&mut sim);
        match next {
            None => {
                assert_eq!(c.forum, None, "{c:?}");
                assert!(
                    before_the_gathering(&sim, id),
                    "the gathering is called for it"
                );
                // The gathering hears it when it sits, its members coming as to any.
                let meets = sim.people().polities[pi]
                    .gathering
                    .as_ref()
                    .expect("called")
                    .day;
                sim.advance_minutes((meets + 1 - sim.now().day_index()) * DAY)
                    .expect("advances");
                let c = find(&sim, id);
                assert_ne!(c.stage, CaseStage::Open, "{c:?}");
                assert_eq!(c.forum, None, "{c:?}");
                assert!(c.eligible > 1, "the gathering's members: {c:?}");
            }
            Some(next) => {
                assert_eq!(c.forum, Some(next), "{c:?}");
                assert!(!before_the_gathering(&sim, id));
                sim.advance_minutes(DAY).expect("advances");
                let c = find(&sim, id);
                assert_ne!(c.stage, CaseStage::Open, "{c:?}");
                assert_eq!(
                    c.stances.iter().map(|r| r.person).collect::<Vec<_>>(),
                    vec![hh[3].1[0]],
                    "the successor alone"
                );
            }
        }
    }
}

/// A hearer removed before their term (backlog D; research 09-02 §8.6: tenure ends by removal as
/// well as by its term; 09-14 §1.2, §1.5: who may dismiss an officeholder is a body and a
/// procedure; 09-07 §4.3: what follows a finding against those who can act is part of a court's
/// independence): those who hold the hearer's findings against their forum found a faction, which
/// petitions the gathering for another hearer in place of the one hearing cases, a year's term
/// not half run. Each who blames the forum weighs it against their regard for the hearer, as
/// against a keeper; passed, the old forum gives way, the new hearer holds a term of their own,
/// and a case waiting before the old hearer goes before the new one. All of it saves and loads
/// exactly.
#[test]
fn a_faction_against_a_hearer_petitions_for_another_before_their_term_is_out() {
    use civ_agents::polity::{Outcome as Decided, Term};
    use civ_agents::ties::Tie;
    use civ_agents::word::{Blamed, Claim, ClaimKind, Grievance, Grieved, Wrong};
    let mut sim = world(3);
    sim.advance_minutes(20 * DAY).expect("advances");
    let hh = households(&sim);
    assert!(hh.len() >= 4, "four households: {hh:?}");
    let (pi, today) = (0, sim.now().day_index());
    let hearer = hh[2].1[0];
    let forum = law(
        &mut sim,
        "core:policy/zestfully_name_a_hearer_for_a_year",
        Some(hearer),
    );
    sim.people_mut_for_tests().polities[pi].set_term(
        forum,
        Term {
            until: today + 360,
            pool: 0,
        },
    );
    // Every adult but the hearer holds a keen grievance against the forum over a finding, has
    // heard that every other does, and regards the others well.
    let params = sim.rules().people.clone();
    let adults: Vec<PermanentId> = hh.iter().flat_map(|h| h.1.clone()).collect();
    let aggrieved: Vec<PermanentId> = adults.iter().copied().filter(|&a| a != hearer).collect();
    let pop = sim.people_mut_for_tests();
    let settlement = pop.polities[pi].settlement;
    for &a in &aggrieved {
        pop.word.grievances.push(Grievance {
            holder: a,
            issue: Grieved::Collective,
            blamed: Blamed::Forum(forum),
            law: forum,
            harm_days: 10.0,
            unresolved_days: 10.0,
            activation: 1.0,
            raised: today,
            made: today,
            wrong: Wrong::NotFound,
        });
        let claim = pop.word.make(Claim {
            fire: None,
            event: None,
            id: 0,
            kind: ClaimKind::Grievance,
            settlement,
            day: today,
            subject: Some(a),
            grievance: Some((Blamed::Forum(forum), Grieved::Collective)),
            suspected: None,
        });
        for &b in &aggrieved {
            pop.word
                .hear(b, claim, today, (a != b).then_some(a), Some(a));
        }
        let ties: Vec<Tie> = adults
            .iter()
            .filter(|&&b| b != a)
            .map(|&b| Tie {
                familiarity: 1.0,
                warmth: 1.0,
                ..Tie::new(b, today, &params.ties)
            })
            .collect();
        pop.ties.restore(a, ties);
    }
    // The petition under test, not when people review where they belong: each does so daily.
    sim.rules_mut_for_tests()
        .expect("rules not yet shared")
        .people
        .faction
        .review_days = 1;
    // A faction is founded against the forum, and its organizer petitions for another hearer.
    let mut called = None;
    for _ in 0..80 {
        sim.advance_minutes(DAY).expect("advances");
        if let Some(p) = sim.people().factions.petitions.first() {
            called = Some(p.clone());
            break;
        }
    }
    let called = called.expect("a petition was called");
    let pop = sim.people();
    let f = pop.factions.get(called.faction).expect("its faction");
    assert_eq!(f.against, Blamed::Forum(forum));
    assert_eq!(called.ends, forum, "{called:?}");
    let nominee = called.nominee.expect("someone named");
    assert_ne!(nominee, hearer);
    assert_eq!(
        sim.rules().catalog.policies[usize::from(called.policy)].id,
        "core:policy/zestfully_name_a_hearer_for_a_year"
    );
    let words = civ_sim::frames::word::petition_demand_words(&sim, &called);
    assert_eq!(
        words,
        format!(
            "{} to hear cases in place of {}",
            pop.name_of(nominee),
            pop.name_of(hearer)
        )
    );
    // It sits, and goes before the gathering.
    let mut proposed = None;
    for _ in 0..12 {
        sim.advance_minutes(DAY).expect("advances");
        if let Some(l) = sim.people().factions.petitions[0].law {
            proposed = Some(l);
            break;
        }
    }
    let proposed = proposed.expect("it went before the gathering");
    let meets = sim.people().polities[pi]
        .laws
        .iter()
        .find(|l| l.id == proposed)
        .expect("kept")
        .meets_day;
    // A case between two households neither hearer is of, brought the day before the gathering
    // sits on it, waits before the hearer in office when the gathering decides.
    let theirs = |sim: &Sim, p: PermanentId| sim.people().person(p).expect("alive").household;
    let (h1, h2) = (theirs(&sim, hearer), theirs(&sim, nominee));
    let parties: Vec<Household> = hh
        .iter()
        .filter(|h| h.0 != h1 && h.0 != h2)
        .cloned()
        .collect();
    assert!(parties.len() >= 2, "{parties:?}");
    assert!(sim.now().day_index() < meets, "it has not sat yet");
    while sim.now().day_index() < meets - 1 {
        sim.advance_minutes(DAY).expect("advances");
    }
    let id = case(&mut sim, &parties[0], &parties[1]);
    sim.advance_minutes(DAY).expect("advances");
    assert_eq!(
        find(&sim, id).forum,
        Some(forum),
        "before the hearer in office"
    );
    let mut sim = reloaded(&mut sim);
    sim.advance_minutes(DAY).expect("advances");
    let pop = sim.people();
    let l = pop.polities[pi]
        .laws
        .iter()
        .find(|l| l.id == proposed)
        .expect("kept")
        .clone();
    assert_eq!(l.ends, Some(forum));
    assert_eq!(l.holder, Some(nominee));
    assert_eq!(l.outcome, Some(Decided::Passed), "{l:?}");
    // Those who blame the forum weighed it against their regard for the hearer: the hearer, who
    // came, did not back being replaced.
    for r in &l.stances {
        if r.person == hearer {
            assert!(r.regard <= 0.0, "{r:?}");
            assert_ne!(r.stance, Stance::Support, "{r:?}");
        } else if r.person != l.sponsor {
            assert!(r.regard > 0.0, "{r:?}");
        }
    }
    let old = pop.polities[pi]
        .laws
        .iter()
        .find(|l| l.id == forum)
        .expect("kept");
    assert_eq!(old.status, LawStatus::Superseded);
    let term = pop.polities[pi]
        .term_of(proposed)
        .expect("a term of its own");
    assert_eq!(term.until, l.decided.expect("decided").day_index() + 360);
    // The case waiting before the hearer removed goes before the new one, who hears it.
    let c = find(&sim, id);
    assert_eq!(c.forum, Some(proposed), "{c:?}");
    assert_eq!(c.stage, CaseStage::Open, "{c:?}");
    sim.advance_minutes(DAY).expect("advances");
    let c = find(&sim, id);
    assert_ne!(c.stage, CaseStage::Open, "{c:?}");
    assert_eq!(
        c.stances.iter().map(|r| r.person).collect::<Vec<_>>(),
        vec![nominee]
    );
    let lines = civ_sim::frames::government::petition_words(&sim, settlement);
    assert!(
        lines
            .iter()
            .any(|w| w.contains("to hear cases in place of") && w.contains("granted it")),
        "{lines:?}"
    );
    reloaded(&mut sim);
}

/// Homelab run 64 (the notables' gate, river_valley seed 1, every adult, stream 5): one case a
/// thin gathering left unheard before the polity had a forum kept *cases waiting* before it for
/// all of its two years' memory, and every other forum a template offers was put forward in place
/// of the one in force week after week, 38 laws passed in two years. A case the gathering left
/// before the forum in force is answered by it: only a case left unheard or not found since it
/// came into force, or a grievance against it, keeps the issue open.
#[test]
fn a_case_left_before_the_forum_in_force_keeps_no_case_waiting() {
    let mut sim = world(3);
    sim.set_notable_tier(false);
    let hh = households(&sim);
    let settlement = sim.people().polities[0].settlement;
    let waiting = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| pop.cases_waiting_for_tests(ctx, settlement))
    };
    // Yesterday a thin gathering left a case unheard: cases wait, with no forum.
    let id = case(&mut sim, &hh[0], &hh[1]);
    let yesterday = civ_core::SimTime::from_minutes(sim.now().minutes() - DAY);
    {
        let pop = sim.people_mut_for_tests();
        let c = pop
            .order
            .cases
            .iter_mut()
            .find(|c| c.id == id)
            .expect("the case");
        c.opened = yesterday.day_index();
        c.stage = CaseStage::Unheard;
        c.heard = Some(yesterday);
    }
    assert!(waiting(&mut sim), "a case unheard and no forum");
    // Today a forum came into force: it answers that case.
    let forum = law(&mut sim, "core:policy/hear_by_elders", None);
    assert!(!waiting(&mut sim), "the forum in force answers it");
    // Nobody puts another forum in place of it for that case, week after week.
    let policies = sim.rules().catalog.policies.clone();
    for _ in 0..8 {
        sim.with_ctx_for_tests(|pop, ctx| pop.review_for_tests(ctx, settlement));
        let pop = sim.people_mut_for_tests();
        let proposed: Vec<String> = pop.polities[0]
            .laws
            .iter()
            .filter(|l| l.status == LawStatus::Proposed)
            .map(|l| policies[usize::from(l.policy)].id.clone())
            .collect();
        assert!(!proposed.iter().any(|p| p.contains("hear")), "{proposed:?}");
        for l in &mut pop.polities[0].laws {
            if l.status == LawStatus::Proposed {
                l.status = LawStatus::Rejected;
            }
        }
        pop.polities[0].gathering = None;
        sim.advance_minutes(DAY).expect("advances");
    }
    // A case the forum in force does not find keeps cases waiting again.
    let id = case(&mut sim, &hh[0], &hh[1]);
    {
        let now = sim.now();
        let pop = sim.people_mut_for_tests();
        let c = pop
            .order
            .cases
            .iter_mut()
            .find(|c| c.id == id)
            .expect("the case");
        c.stage = CaseStage::NotFound;
        c.heard = Some(now);
        c.forum = Some(forum);
    }
    assert!(waiting(&mut sim), "a case the forum in force did not find");
}
