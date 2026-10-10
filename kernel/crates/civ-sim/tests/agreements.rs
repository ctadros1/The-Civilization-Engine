//! Agreements between polities (M5c slice AU, ADR-0020 §6): someone whose household heard that
//! another polity claims places it works seeks terms with someone they know there; the two agree
//! on a package both expect to pass at home or part with none; each gathering decides it by its
//! own custom; and it is in force once each side has heard, from a traveller, of the other's
//! decision.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::agreements::{AgreementState, Clause, Failure};
use civ_agents::polity::{IssueKind, Law, LawStatus, Outcome, PolicyKind};
use civ_agents::ties::Act;
use civ_agents::uses::{HeardClaim, Place, Worked};
use civ_agents::views::{Domain, ViewAct};
use civ_agents::word::Wrong;
use civ_agents::{AgreementStep, ChronicleKind};
use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_sim::relations::{self, Standing};
use civ_sim::{NewWorld, Sim, persist};

const PRESET: &str = "core:worldgen/river_valley";
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

/// Two settlements of world `seed`, with a fixed identity so it lives the same life every run.
fn two_villages(seed: u64) -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Neighbours".to_owned(),
            seed,
            preset_id: PRESET.to_owned(),
            size_cells: 768,
            band_size: 30,
            neighbours: vec![30],
            neighbours_known: true,
            regime_id: String::new(),
        },
        content(),
        [7; 16],
    )
    .expect("generates")
}

/// The households living in settlement `s`, in id order.
fn households_of(sim: &Sim, s: PermanentId) -> Vec<PermanentId> {
    let mut out: Vec<PermanentId> = sim
        .people()
        .households
        .iter()
        .filter(|(_, h)| h.settlement == Some(s) && !h.members.is_empty())
        .map(|(_, h)| h.id)
        .collect();
    out.sort_unstable();
    out
}

/// The grown members of households `homes`, in id order.
fn adults_of(sim: &Sim, homes: &[PermanentId]) -> Vec<PermanentId> {
    let age = sim.rules().people.family.independent_age;
    let pop = sim.people();
    let mut out: Vec<PermanentId> = homes
        .iter()
        .filter_map(|&h| pop.household(h))
        .flat_map(|x| x.members.iter().copied())
        .filter(|&m| pop.person(m).is_some_and(|p| p.age_years(sim.now()) >= age))
        .collect();
    out.sort_unstable();
    out
}

/// What the setup made: the world, the two polities (A's first), the place each claims, and the
/// two settlements' households and adults.
struct Neighbours {
    sim: Sim,
    polities: [PermanentId; 2],
    places: [Place; 2],
    homes: [Vec<PermanentId>; 2],
    adults: [Vec<PermanentId>; 2],
}

/// Two villages of world 5 a day old. Each polity `s` in `claims` claims place `s` by a law in
/// force all its adults know. For the past sixty days the households of each village in
/// `workers[s]` worked place `s`, and each household of the other village heard of the claim on
/// it. A few adults of each village know a few of the other's, and some count one of the other
/// village as kin, so they visit.
fn neighbours(claims: &[usize], workers: [&[usize]; 2]) -> Neighbours {
    let mut sim = two_villages(5);
    sim.advance_minutes(DAY).expect("lives");
    let settlements: Vec<PermanentId> = sim.land().settlements.iter().map(|s| s.id).collect();
    assert_eq!(settlements.len(), 2);
    let homes = [
        households_of(&sim, settlements[0]),
        households_of(&sim, settlements[1]),
    ];
    let adults = [adults_of(&sim, &homes[0]), adults_of(&sim, &homes[1])];
    let polities = [0, 1].map(|s| {
        sim.people()
            .polities
            .iter()
            .find(|p| p.settlement == settlements[s])
            .expect("each settlement has a polity")
            .id
    });
    let places = [Place::Patch(4_242), Place::Patch(4_343)];
    let claim = sim
        .rules()
        .catalog
        .policies
        .iter()
        .position(|d| d.kind == PolicyKind::ClaimPlace)
        .expect("a claim template") as u16;
    let laws = [sim.allocate_id_for_tests(), sim.allocate_id_for_tests()];
    let tp = sim.rules().people.ties.clone();
    let now = sim.now();
    let today = now.day_index();
    let pop = sim.people_mut_for_tests();
    for &s in claims {
        let polity = pop
            .polities
            .iter_mut()
            .find(|p| p.id == polities[s])
            .expect("its polity");
        polity.laws.push(Law {
            id: laws[s],
            policy: claim,
            kind: PolicyKind::ClaimPlace,
            levy_share: 0.0,
            holder: None,
            relief_days: 0.0,
            sanction: Default::default(),
            hours: (0, 0),
            status: LawStatus::InForce,
            sponsor: adults[s][0],
            proposed: now,
            issue: IssueKind::Outsiders,
            meets_day: today,
            decided: Some(now),
            outcome: Some(Outcome::Passed),
            eligible: adults[s].len() as u32,
            stances: Vec::new(),
            known: adults[s].iter().map(|&p| (p, today)).collect(),
            compliance: Default::default(),
            watch: Default::default(),
            body: None,
            ends: None,
            agreement: None,
        });
        polity.claimed.push((laws[s], places[s]));
        for &h in &homes[1 - s] {
            pop.claims_heard.learn(
                h,
                HeardClaim {
                    place: places[s],
                    law: laws[s],
                    polity: polities[s],
                    day: today,
                    from: adults[s][0],
                },
            );
        }
    }
    for day in today - 60..today {
        for s in 0..2 {
            for &w in workers[s] {
                for &h in &homes[w] {
                    pop.uses.worked(Worked {
                        place: places[s],
                        day,
                        household: h,
                        settlement: settlements[w],
                        kcal: 2_500.0,
                        person: None,
                    });
                }
            }
        }
    }
    for s in 0..2 {
        for &x in &adults[s] {
            for &y in adults[1 - s].iter().take(3) {
                pop.ties.record(x, y, Act::Hearth, 2.0, 0.0, today, &tp);
            }
        }
        for (k, &c) in adults[1 - s].iter().rev().take(4).enumerate() {
            pop.records.get_mut(&c).expect("on record").mother = Some(adults[s][k]);
        }
    }
    Neighbours {
        sim,
        polities,
        places,
        homes,
        adults,
    }
}

/// The agreement law polity `polity` decides agreement `id` by.
fn law_of(sim: &Sim, polity: PermanentId, id: PermanentId) -> Option<Law> {
    sim.people()
        .polities
        .iter()
        .find(|p| p.id == polity)?
        .laws
        .iter()
        .find(|l| l.agreement == Some(id))
        .cloned()
}

/// The chronicle's agreement entries, in order.
fn agreement_entries(sim: &Sim) -> Vec<(AgreementStep, String)> {
    let steps = [
        AgreementStep::Agreed,
        AgreementStep::NoTerms,
        AgreementStep::InForce,
        AgreementStep::Failed,
        AgreementStep::Ended,
        AgreementStep::Delivered,
        AgreementStep::Missed,
    ];
    sim.people()
        .chronicle
        .iter()
        .filter(|e| e.kind == ChronicleKind::Agreement)
        .map(|e| (steps[e.number as usize], e.name.clone()))
        .collect()
}

#[test]
fn neighbours_each_claiming_what_the_other_works_agree_on_leave_and_it_is_in_force_once_each_hears()
{
    // Each village claims a place only the other's people work.
    let Neighbours {
        mut sim,
        polities,
        places,
        homes,
        adults,
    } = neighbours(&[0, 1], [&[1], &[0]]);

    // Within a few weeks someone of one village, whose household heard the other claims a place
    // it works, seeks terms with someone they know there, and the two agree.
    let mut made = None;
    for _ in 0..30 {
        sim.advance_minutes(DAY).expect("lives");
        if let Some(a) = sim.people().agreements.list.first() {
            made = Some(a.clone());
            break;
        }
    }
    let a = made.expect("someone sought terms");
    let entries = agreement_entries(&sim);
    assert_eq!(a.state, AgreementState::Offered, "{entries:?}");
    // Both give leave, each having something the other wants: the longest term the content
    // offers, since each gains by it.
    assert_eq!(
        a.clauses,
        vec![Clause::Leave { from: 0 }, Clause::Leave { from: 1 }]
    );
    assert_eq!(a.term_days, 1825);
    let seeker_side = polities
        .iter()
        .position(|&p| p == a.polities[0])
        .expect("a party");
    assert_ne!(a.polities[0], a.polities[1]);
    assert!(adults[seeker_side].contains(&a.negotiators[0]));
    assert!(adults[1 - seeker_side].contains(&a.negotiators[1]));
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].0, AgreementStep::Agreed);
    assert!(entries[0].1.contains("sought terms"), "{}", entries[0].1);
    // The seeker put it to their own gathering at once, under their household's issue.
    let law0 = law_of(&sim, a.polities[0], a.id).expect("the seeker sponsored it");
    assert_eq!(law0.sponsor, a.negotiators[0]);
    assert_eq!(law0.issue, IssueKind::ClaimedFromUs);
    assert_eq!(law0.kind, PolicyKind::Agreement);
    assert_eq!(a.laws[0], Some(law0.id));

    // Each gathering decides it by its own custom; the other side's negotiator put it to theirs.
    for _ in 0..20 {
        sim.advance_minutes(DAY).expect("lives");
        let x = &sim.people().agreements.list[0];
        if x.passed.iter().all(Option::is_some) || !x.open() {
            break;
        }
    }
    let x = sim.people().agreements.list[0].clone();
    assert!(
        x.passed.iter().all(Option::is_some),
        "both gatherings passed it: {:?} {:?}",
        x.state,
        agreement_entries(&sim)
    );
    let law1 = law_of(&sim, x.polities[1], x.id).expect("the other side sponsored it");
    assert_eq!(law1.sponsor, x.negotiators[1]);
    assert_eq!(law1.issue, IssueKind::TermsSought);
    for side in 0..2 {
        let l = law_of(&sim, x.polities[side], x.id).expect("still there");
        assert_eq!(l.outcome, Some(Outcome::Passed));
        assert_eq!(l.status, LawStatus::InForce);
        assert!(!l.stances.is_empty(), "every stance is kept");
    }
    // Neither side has heard of the other's decision yet: it is not in force.
    assert!(x.open());
    assert!(!sim.people().agreements.leave(polities[0], polities[1]));

    // Word crosses only with travellers: kin of each village visit the other's hearth.
    let mut in_force = false;
    for _ in 0..150 {
        sim.advance_minutes(DAY).expect("lives");
        let x = &sim.people().agreements.list[0];
        if !x.open() {
            in_force = x.in_force();
            break;
        }
    }
    let x = sim.people().agreements.list[0].clone();
    assert!(in_force, "{:?} {:?}", x.state, agreement_entries(&sim));
    let AgreementState::InForce { since } = x.state else {
        unreachable!()
    };
    assert!(x.heard.iter().all(|h| h.is_some_and(|d| d <= since)));
    assert!(sim.people().agreements.leave(polities[0], polities[1]));
    assert!(sim.people().agreements.leave(polities[1], polities[0]));
    assert!(
        agreement_entries(&sim)
            .iter()
            .any(|e| e.0 == AgreementStep::InForce && e.1.contains("came into force"))
    );
    assert!(sim.people().problems(u64::MAX, usize::MAX).is_empty());
    // The observer names each side's relation by it, and shows both law histories.
    let polity = |p: PermanentId| {
        sim.people()
            .polities
            .iter()
            .find(|x| x.id == p)
            .expect("a polity")
    };
    let (a_, b_) = (polity(polities[0]), polity(polities[1]));
    for (from, to) in [(a_, b_), (b_, a_)] {
        let label = relations::relation_of(&sim, from, to);
        assert_eq!(label.standing, Standing::UnderAgreement, "{:?}", label.why);
        assert!(
            label.why[0].starts_with("an agreement with "),
            "{:?}",
            label.why
        );
        let views = relations::agreements_between(&sim, from, to);
        assert_eq!(views.len(), 1);
        let v = &views[0];
        assert!(v.state.starts_with("in force since"), "{}", v.state);
        assert!(v.terms.contains("leave for"), "{}", v.terms);
        for side in [&v.ours, &v.theirs] {
            assert!(
                side.contains("; passed on")
                    && side.contains("that the other's gathering passed it"),
                "{side}"
            );
        }
    }

    // Under leave, people of the other village working a place one claims are not trespassing to
    // those who know their own law deciding it: B's households work A's place beside one of A's.
    let today = sim.now().day_index();
    let settlement_of = |sim: &Sim, p: PermanentId| {
        sim.people()
            .polities
            .iter()
            .find(|x| x.id == p)
            .expect("a polity")
            .settlement
    };
    let settlements = [
        settlement_of(&sim, polities[0]),
        settlement_of(&sim, polities[1]),
    ];
    let law_a = law_of(&sim, polities[0], x.id).expect("A's law");
    let witness = adults[0]
        .iter()
        .copied()
        .find(|&p| sim.people().person(p).is_some() && law_a.knows(p))
        .expect("someone of A knows A's law deciding it");
    let witness_home = sim.people().person(witness).expect("alive").household;
    let mut work = vec![(witness_home, settlements[0], Some(witness))];
    work.extend(homes[1].iter().map(|&h| (h, settlements[1], None)));
    let pop = sim.people_mut_for_tests();
    for (household, settlement, person) in work {
        pop.uses.worked(Worked {
            place: places[0],
            day: today,
            household,
            settlement,
            kcal: 2_500.0,
            person,
        });
    }
    sim.advance_minutes(DAY).expect("lives");
    assert!(
        !sim.people()
            .word
            .grievances_of(witness)
            .any(|g| g.wrong == Wrong::Trespass),
        "no trespass under leave"
    );

    // And the whole of it saves and loads.
    saves_and_goes_on_alike(&mut sim, DAY);
}

#[test]
fn people_who_believe_the_other_village_harms_them_weigh_it_against_an_agreement() {
    // As the first test, but everyone of B believes A's people harm theirs: each saw them
    // trespass on ten days lately (ADR-0020 §3: a view weighs as regard for a sponsor does).
    let Neighbours {
        mut sim,
        polities,
        adults,
        ..
    } = neighbours(&[0, 1], [&[1], &[0]]);
    let rp = sim.rules().people.relations.clone();
    let today = sim.now().day_index();
    let pop = sim.people_mut_for_tests();
    for &p in &adults[1] {
        pop.polity_views
            .record(p, polities[0], ViewAct::SawTrespass, 10.0, today, &rp);
    }
    let warmth = pop
        .polity_views
        .of(adults[1][0], polities[0], today, &rp)
        .expect("a view")
        .warmth();
    assert!(warmth < -0.4, "{warmth}");
    let mut made = None;
    for _ in 0..60 {
        sim.advance_minutes(DAY).expect("lives");
        let x = sim.people().agreements.list.first();
        if let Some(a) = x.filter(|a| !a.open() || a.passed.iter().all(Option::is_some)) {
            made = Some(a.clone());
            break;
        }
    }
    let a = made.unwrap_or_else(|| panic!("{:?}", agreement_entries(&sim)));
    // B's side holds it against A: either B's negotiator saw no package worth sponsoring, or B's
    // gathering turned it down, every stance there keeping what the view added.
    let b_side = a.side_of(polities[1]).expect("B is a party");
    match a.state {
        AgreementState::Failed {
            why: Failure::NoTerms,
            ..
        } => {}
        AgreementState::Failed {
            why: Failure::TurnedDown(s),
            ..
        } => {
            assert_eq!(s, b_side);
            let law = law_of(&sim, polities[1], a.id).expect("B's law");
            let viewed: Vec<f32> = law
                .stances
                .iter()
                .filter(|r| r.person != law.sponsor)
                .map(|r| r.view)
                .collect();
            assert!(
                !viewed.is_empty() && viewed.iter().all(|&v| v < 0.0),
                "{viewed:?}"
            );
        }
        other => panic!(
            "B would not have it: {other:?} {:?}",
            agreement_entries(&sim)
        ),
    }
    saves_and_goes_on_alike(&mut sim, DAY);
}

#[test]
fn a_package_the_other_side_expects_to_lose_by_is_not_agreed_and_the_meeting_is_recorded() {
    // Only A claims a place, which B's people have worked these two months. Over a year before,
    // both villages' people worked it side by side, and A's got much there: A's households still
    // hold what B's took then, which leave would give back to them. (Had they met there within
    // the year, B's people would have claimed it too, and held it as their own.)
    let Neighbours {
        mut sim,
        polities,
        places,
        homes,
        ..
    } = neighbours(&[0], [&[1], &[]]);
    let today = sim.now().day_index();
    let settlements = [0, 1].map(|s| {
        sim.people()
            .polities
            .iter()
            .find(|p| p.id == polities[s])
            .expect("a polity")
            .settlement
    });
    let pop = sim.people_mut_for_tests();
    for day in today - 460..today - 400 {
        for s in 0..2 {
            for &h in &homes[s] {
                pop.uses.worked(Worked {
                    place: places[0],
                    day,
                    household: h,
                    settlement: settlements[s],
                    kcal: if s == 0 { 40_000.0 } else { 2_500.0 },
                    person: None,
                });
            }
        }
    }
    let mut met = None;
    for _ in 0..30 {
        sim.advance_minutes(DAY).expect("lives");
        if let Some(a) = sim.people().agreements.list.first() {
            met = Some(a.clone());
            break;
        }
    }
    let laws: Vec<String> = sim
        .people()
        .chronicle
        .iter()
        .filter(|e| {
            matches!(
                e.kind,
                ChronicleKind::LawProposed | ChronicleKind::LawDecided
            )
        })
        .map(|e| e.name.clone())
        .collect();
    let a = met.unwrap_or_else(|| panic!("someone of B sought terms; laws: {laws:#?}"));
    assert_eq!(a.polities, [polities[1], polities[0]], "B sought them");
    assert_eq!(
        a.state,
        AgreementState::Failed {
            day: a.made,
            why: Failure::NoTerms
        },
        "{:?}",
        agreement_entries(&sim)
    );
    assert!(a.clauses.is_empty() && a.laws == [None, None]);
    let entries = agreement_entries(&sim);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].0, AgreementStep::NoTerms);
    assert!(
        entries[0].1.contains("parted with none"),
        "{}",
        entries[0].1
    );
    // Nobody seeks terms again between the two while the failure is remembered.
    sim.advance_minutes(30 * DAY).expect("lives");
    assert_eq!(sim.people().agreements.list.len(), 1);
    saves_and_goes_on_alike(&mut sim, DAY);
}

/// Polity `polity` keeps a common store holding `kg` of grain by a law in force, with neither levy
/// nor relief, and `keeper`, when given, keeps it by another. Returns grain's good index.
fn keep_store(sim: &mut Sim, polity: PermanentId, keeper: Option<PermanentId>, kg: f64) -> usize {
    let policies = sim.rules().catalog.policies.clone();
    let of = |kind: PolicyKind| {
        policies
            .iter()
            .position(|p| p.kind == kind)
            .expect("in core") as u16
    };
    let grain = sim.rules().catalog.crops[0].good;
    let goods = sim.rules().catalog.goods.len();
    let ids = [sim.allocate_id_for_tests(), sim.allocate_id_for_tests()];
    let now = sim.now();
    let today = now.day_index();
    let pop = sim.people_mut_for_tests();
    let sponsor =
        keeper.unwrap_or_else(|| pop.people.iter().map(|(_, p)| p.id).min().expect("people"));
    let law = |id: PermanentId, kind: PolicyKind, holder: Option<PermanentId>| Law {
        id,
        policy: of(kind),
        kind,
        levy_share: 0.0,
        holder,
        relief_days: 0.0,
        sanction: Default::default(),
        hours: (0, 0),
        status: LawStatus::InForce,
        sponsor,
        proposed: now,
        issue: IssueKind::FoodShort,
        meets_day: today,
        decided: Some(now),
        outcome: Some(Outcome::Passed),
        eligible: 0,
        stances: Vec::new(),
        known: Vec::new(),
        compliance: Default::default(),
        watch: Default::default(),
        body: None,
        ends: None,
        agreement: None,
    };
    let p = pop
        .polities
        .iter_mut()
        .find(|p| p.id == polity)
        .expect("a polity");
    p.laws.push(law(ids[0], PolicyKind::CommonStore, None));
    if keeper.is_some() {
        p.laws.push(law(ids[1], PolicyKind::KeepStore, keeper));
    }
    p.stores.resize(goods, 0.0);
    p.stores[grain] = kg;
    p.stores_at = now;
    grain
}

/// An agreement between `polities` with `clauses`, put in force by hand on day `since` by a law
/// of each side passed that day and known to all its `adults`, the first adult of each side
/// having agreed on it.
fn in_force(
    sim: &mut Sim,
    polities: [PermanentId; 2],
    adults: &[Vec<PermanentId>; 2],
    clauses: Vec<Clause>,
    since: i64,
) -> PermanentId {
    let id = sim.allocate_id_for_tests();
    let laws = [sim.allocate_id_for_tests(), sim.allocate_id_for_tests()];
    let policy = sim
        .rules()
        .catalog
        .policies
        .iter()
        .position(|d| d.kind == PolicyKind::Agreement)
        .expect("an agreement template") as u16;
    let now = sim.now();
    let negotiators = [adults[0][0], adults[1][0]];
    let pop = sim.people_mut_for_tests();
    for s in 0..2 {
        let polity = pop
            .polities
            .iter_mut()
            .find(|p| p.id == polities[s])
            .expect("its polity");
        polity.laws.push(Law {
            id: laws[s],
            policy,
            kind: PolicyKind::Agreement,
            levy_share: 0.0,
            holder: None,
            relief_days: 0.0,
            sanction: Default::default(),
            hours: (0, 0),
            status: LawStatus::InForce,
            sponsor: negotiators[s],
            proposed: now,
            issue: if s == 0 {
                IssueKind::ClaimedFromUs
            } else {
                IssueKind::TermsSought
            },
            meets_day: since,
            decided: Some(now),
            outcome: Some(Outcome::Passed),
            eligible: adults[s].len() as u32,
            stances: Vec::new(),
            known: adults[s].iter().map(|&p| (p, since)).collect(),
            compliance: Default::default(),
            watch: Default::default(),
            body: None,
            ends: None,
            agreement: Some(id),
        });
    }
    pop.agreements.list.push(civ_agents::agreements::Agreement {
        id,
        polities,
        negotiators,
        clauses,
        term_days: 1825,
        made: since,
        laws: [Some(laws[0]), Some(laws[1])],
        passed: [Some(since); 2],
        heard: [Some(since); 2],
        state: AgreementState::InForce { since },
    });
    id
}

/// The store of polity `p`, kilograms of good `g`.
fn store_of(sim: &Sim, p: PermanentId, g: usize) -> f64 {
    sim.people()
        .polities
        .iter()
        .find(|x| x.id == p)
        .and_then(|x| x.stores.get(g).copied())
        .unwrap_or(0.0)
}

#[test]
fn a_store_sets_aside_what_an_agreement_owes_and_its_keeper_carries_it_to_the_other_hearth() {
    // No claims: nobody seeks terms. A keeps a store of 600 kg of grain with a keeper; B keeps
    // one too. An agreement in force from today owes B a gift of 400 kg and 100 kg a year.
    let Neighbours {
        mut sim,
        polities,
        adults,
        ..
    } = neighbours(&[], [&[], &[]]);
    let keeper = adults[0][1];
    let grain = keep_store(&mut sim, polities[0], Some(keeper), 600.0);
    keep_store(&mut sim, polities[1], None, 0.0);
    let today = sim.now().day_index();
    let id = in_force(
        &mut sim,
        polities,
        &adults,
        vec![
            Clause::Leave { from: 1 },
            Clause::Gift {
                from: 0,
                good: grain as u16,
                kg: 400,
            },
            Clause::Transfer {
                from: 0,
                good: grain as u16,
                kg: 100,
                every_days: 365,
            },
        ],
        today,
    );
    let held_before = sim.people().goods_held();
    let flows_before = sim.people().flows();

    // At midnight both payments fall due, and the store sets aside all of each.
    sim.advance_minutes(DAY).expect("lives");
    let dues = sim.people().agreements.dues.clone();
    assert_eq!(dues.len(), 2, "{dues:?}");
    for (d, (clause, kg)) in dues.iter().zip([(1u8, 400.0), (2, 100.0)]) {
        assert_eq!(
            (d.agreement, d.clause, d.from, d.to),
            (id, clause, polities[0], polities[1])
        );
        assert_eq!(usize::from(d.good), grain);
        assert_eq!(d.owed_kg, kg);
        assert!((d.set_aside_kg - kg).abs() < 1e-6, "{d:?}");
        assert_eq!(d.carrier, Some(keeper), "the keeper is to carry it");
    }
    assert!(store_of(&sim, polities[0], grain) < 100.0 + 1e-6);

    // Within the month the keeper walks it over, a payment a day at most, and hands it over at
    // the other hearth.
    let mut met = false;
    for _ in 0..30 {
        sim.advance_minutes(DAY).expect("lives");
        let dues = &sim.people().agreements.dues;
        if dues
            .iter()
            .all(|d| d.state != civ_agents::agreements::DueState::Open)
        {
            met = true;
            break;
        }
    }
    let dues = sim.people().agreements.dues.clone();
    let entries = agreement_entries(&sim);
    assert!(met, "{dues:?} {entries:?}");
    let mut arrived = 0.0;
    for d in &dues {
        assert!(
            matches!(d.state, civ_agents::agreements::DueState::Met { .. }),
            "{d:?} {entries:?}"
        );
        // Grain keeps: what arrived is nearly all that was set aside.
        assert!(d.arrived_kg > 0.98 * d.set_aside_kg && d.arrived_kg <= d.set_aside_kg);
        arrived += d.arrived_kg;
    }
    assert!((store_of(&sim, polities[1], grain) - arrived).abs() < 1.0);
    let carried: Vec<&String> = entries
        .iter()
        .filter(|e| e.0 == AgreementStep::Delivered)
        .map(|e| &e.1)
        .collect();
    assert_eq!(carried.len(), 2, "{entries:?}");
    assert!(
        carried[0].contains("kg of grain from") && carried[0].contains("as their agreement asks"),
        "{}",
        carried[0]
    );
    // Every kilogram is accounted for, the payments' included.
    let gaps = civ_agents::population::unaccounted(
        sim.rules().catalog.goods.len(),
        (&held_before, &flows_before),
        (&sim.people().goods_held(), &sim.people().flows()),
    );
    assert!(gaps.is_empty(), "unaccounted for: {gaps:?}");
    assert!(sim.people().problems(u64::MAX, usize::MAX).is_empty());
    // The observer words the terms by what they give.
    let views = relations::agreements_between(
        &sim,
        sim.people()
            .polities
            .iter()
            .find(|p| p.id == polities[0])
            .expect("A"),
        sim.people()
            .polities
            .iter()
            .find(|p| p.id == polities[1])
            .expect("B"),
    );
    assert!(
        views[0].terms.contains("a gift of 400 kg of grain"),
        "{}",
        views[0].terms
    );
    assert!(
        views[0].terms.contains("100 kg of grain"),
        "{}",
        views[0].terms
    );
    // And each payment, with what was set aside and what arrived (wire 1.59).
    assert_eq!(views[0].payments.len(), 2, "{:?}", views[0].payments);
    for line in &views[0].payments {
        assert!(
            line.contains("kg of grain from")
                && line.contains("set aside")
                && line.contains("; met on"),
            "{line}"
        );
    }

    // Those of B who know B's law deciding it take each payment met as evidence that A keeps its
    // word, and the gift that A helps (M5c slice AV, step two); A's people take nothing.
    let rp = sim.rules().people.relations.clone();
    let day = sim.now().day_index();
    let b_views: Vec<_> = adults[1]
        .iter()
        .filter_map(|&p| sim.people().polity_views.of(p, polities[0], day, &rp))
        .collect();
    assert!(!b_views.is_empty());
    for v in &b_views {
        assert!(v.lean(Domain::KeepsWord) > 0.7, "{v:?}");
        assert!(v.lean(Domain::HelpsUs) > 0.6, "{v:?}");
        assert!(v.warmth() > 0.0, "{v:?}");
    }
    assert!(adults[0].iter().all(|&p| {
        sim.people()
            .polity_views
            .of(p, polities[1], day, &rp)
            .is_none()
    }));
    // A pays a transfer in force: it is tributary to B, its burden given both ways (research
    // 13-02 §2.1); B is under agreement, and says what it receives.
    let polity = |p: PermanentId| {
        sim.people()
            .polities
            .iter()
            .find(|x| x.id == p)
            .expect("a polity")
    };
    let (a_, b_) = (polity(polities[0]), polity(polities[1]));
    let label = relations::relation_of(&sim, a_, b_);
    assert_eq!(label.standing, Standing::Tributary, "{:?}", label.why);
    assert!(
        label.why[0].contains("100 kg of grain a year: ")
            && label.why[0].contains("of what its households' fields bring in an ordinary year"),
        "{:?}",
        label.why
    );
    let label = relations::relation_of(&sim, b_, a_);
    assert_eq!(label.standing, Standing::UnderAgreement, "{:?}", label.why);
    assert!(
        label.why[0].ends_with("pays it 100 kg of grain a year"),
        "{:?}",
        label.why
    );
    let burden = sim
        .people()
        .burden(
            sim.land(),
            &sim.rules().catalog,
            &sim.rules().people,
            sim.now(),
            polities[0],
            polities[1],
        )
        .expect("A pays B");
    assert_eq!(burden.goods, vec![(grain as u16, 100.0)]);
    assert!(burden.paid > 0.0 && burden.need > 0.0);
    saves_and_goes_on_alike(&mut sim, DAY);
}

#[test]
fn a_payment_the_store_cannot_meet_is_missed_and_its_cause_kept_and_a_transfer_falls_due_again() {
    // A keeps an empty store with no keeper. An agreement in force from today owes B 100 kg of
    // grain every five days.
    let Neighbours {
        mut sim,
        polities,
        adults,
        ..
    } = neighbours(&[], [&[], &[]]);
    let grain = keep_store(&mut sim, polities[0], None, 0.0);
    let today = sim.now().day_index();
    in_force(
        &mut sim,
        polities,
        &adults,
        vec![Clause::Transfer {
            from: 0,
            good: grain as u16,
            kg: 100,
            every_days: 5,
        }],
        today,
    );
    sim.advance_minutes(DAY).expect("lives");
    assert_eq!(sim.people().agreements.dues.len(), 1);
    sim.advance_minutes(5 * DAY).expect("lives");
    let dues = sim.people().agreements.dues.clone();
    assert_eq!(dues.len(), 2, "the transfer fell due again: {dues:?}");
    assert_eq!(
        dues[1].made,
        today + 5,
        "five days after it came into force"
    );
    assert!(
        dues.iter()
            .all(|d| d.set_aside_kg == 0.0 && d.carrier.is_none())
    );
    // A month after it fell due the first is missed: the store held none.
    let deliver = sim.rules().people.relations.deliver_days;
    sim.advance_minutes((deliver - 4) * DAY).expect("lives");
    let d = sim.people().agreements.dues[0].clone();
    assert!(
        matches!(
            d.state,
            civ_agents::agreements::DueState::Missed {
                why: civ_agents::agreements::Miss::EmptyStore,
                ..
            }
        ),
        "{d:?}"
    );
    let missed: Vec<String> = agreement_entries(&sim)
        .into_iter()
        .filter(|e| e.0 == AgreementStep::Missed)
        .map(|e| e.1)
        .collect();
    assert_eq!(
        missed.len(),
        1,
        "only the first is past its day: {missed:?}"
    );
    assert!(
        missed[0].contains("was missed: the store held none of it"),
        "{}",
        missed[0]
    );
    // Those of B who know the agreement take the miss as evidence that A does not keep its word.
    let rp = sim.rules().people.relations.clone();
    let day = sim.now().day_index();
    for &p in &adults[1] {
        let v = sim
            .people()
            .polity_views
            .of(p, polities[0], day, &rp)
            .expect("a view of A");
        assert!(v.lean(Domain::KeepsWord) < 0.5, "{v:?}");
        assert_eq!(
            v.reason.map(|r| r.act),
            Some(civ_agents::views::ViewAct::PaymentMissed)
        );
    }
    let views = relations::agreements_between(
        &sim,
        sim.people()
            .polities
            .iter()
            .find(|p| p.id == polities[1])
            .expect("B"),
        sim.people()
            .polities
            .iter()
            .find(|p| p.id == polities[0])
            .expect("A"),
    );
    assert!(
        views[0].payments[0].contains("nothing set aside; missed on")
            && views[0].payments[0].ends_with("the store held none of it"),
        "{:?}",
        views[0].payments
    );
    assert!(sim.people().problems(u64::MAX, usize::MAX).is_empty());
    saves_and_goes_on_alike(&mut sim, DAY);
}

/// Saves `sim`, loads it, and checks every section of the loaded world is the same, and that the
/// two go on alike for `minutes`.
fn saves_and_goes_on_alike(sim: &mut Sim, minutes: i64) {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "terms").expect("saves");
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
