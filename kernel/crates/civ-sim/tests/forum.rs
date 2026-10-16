//! Forums (M7b slice BR, ADR-0025 §1): a *hear cases* law names who hears the cases brought under
//! the laws against taking and raiding. A case brought under one is heard the evening after by
//! the forum, not the gathering: a named hearer alone, a panel drawn by a keyed lot from the
//! body's members, or the elders of the households, never anyone of the two households. A forum
//! whose law has ended, or a hearer of one of the two households, leaves the case to the
//! gathering. Before a forum sits, a case may be settled or withdrawn. A case before a forum saves
//! and loads exactly.

use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

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
    let mut sim = Sim::create(
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
        &mut |_| {},
        &AtomicBool::new(false),
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
