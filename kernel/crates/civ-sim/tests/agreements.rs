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
use civ_agents::word::Wrong;
use civ_agents::{AgreementStep, ChronicleKind};
use civ_content::ContentRegistry;
use civ_core::PermanentId;
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
