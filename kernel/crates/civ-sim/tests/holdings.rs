//! A polity over several settlements (M6c slice BM, ADR-0023 §9): one polity takes another in;
//! the other's deciding closes and its record is kept; the settlement's residents become members
//! of the polity that took it; its store becomes that polity's; its agreements end; and all of it
//! saves and loads.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::ChronicleKind;
use civ_agents::agreements::{Agreement, AgreementState, Clause};
use civ_agents::polity::{HeldHow, LawStatus};
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

/// Two villages of `band`, founded apart and unknown to each other, with a fixed identity, a day
/// old.
fn two_villages(seed: u64, band: u32) -> Sim {
    let mut sim = Sim::create_for_tests(
        &NewWorld {
            name: "Holdings".to_owned(),
            seed,
            preset_id: PRESET.to_owned(),
            size_cells: 768,
            band_size: band,
            neighbours: vec![band],
            neighbours_known: false,
            regime_id: String::new(),
        },
        content(),
        [9; 16],
    )
    .expect("generates");
    sim.advance_minutes(DAY).expect("advances");
    sim
}

/// The index of the polity seated at settlement `s`.
fn seated(sim: &Sim, s: PermanentId) -> usize {
    sim.people()
        .polities
        .iter()
        .position(|p| p.settlement == s)
        .expect("its polity")
}

/// Saves `sim`, loads it, and checks every section is the same, and that the two go on alike for
/// `minutes`.
fn saves_and_goes_on_alike(sim: &mut Sim, minutes: i64) {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "held").expect("saves");
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
fn a_polity_taken_in_decides_no_more_and_its_people_are_the_takers_members() {
    let mut sim = two_villages(4, 30);
    let (a, b) = (sim.land().settlements[0].id, sim.land().settlements[1].id);
    let (pi, qi) = (seated(&sim, a), seated(&sim, b));
    let (taker, taken) = (sim.people().polities[pi].id, sim.people().polities[qi].id);
    let grain = sim.rules().catalog.crops[0].good;
    let goods = sim.rules().catalog.goods.len();
    // The village to be taken in keeps a store, and has an agreement with the other.
    let agreement = sim.allocate_id_for_tests();
    let now = sim.now();
    {
        let pop = sim.people_mut_for_tests();
        let q = &mut pop.polities[qi];
        q.stores.resize(goods, 0.0);
        q.stores[grain] = 300.0;
        q.stores_at = now;
        let negotiators = [pop.polities[pi].id, pop.polities[qi].id];
        pop.agreements.list.push(Agreement {
            id: agreement,
            polities: [taker, taken],
            negotiators,
            clauses: vec![Clause::NoRaids { from: 0 }],
            term_days: 365,
            made: now.day_index(),
            laws: [None, None],
            passed: [None, None],
            heard: [None, None],
            state: AgreementState::Offered,
        });
    }
    let (fields, params) = (sim.land().fields.clone(), sim.rules().people.clone());
    let before = sim.people().body_members(&fields, pi, now, &params).len();
    let theirs = sim.people().body_members(&fields, qi, now, &params).len();
    assert!(before > 0 && theirs > 0);
    let held_before = sim.people().polities[pi]
        .stores
        .get(grain)
        .copied()
        .unwrap_or(0.0);
    sim.with_ctx_for_tests(|pop, ctx| pop.take_in_for_tests(ctx, pi, qi, HeldHow::Annexed));

    let pop = sim.people();
    let (p, q) = (&pop.polities[pi], &pop.polities[qi]);
    // The one taken in decides no more, and its record names who took it.
    let end = q.ended.expect("its deciding closed");
    assert_eq!((end.by, end.how), (taker, HeldHow::Annexed));
    assert!(q.gathering.is_none());
    assert!(q.laws.iter().all(|l| l.status != LawStatus::InForce));
    assert!(!q.holds(Some(b)));
    // The taker holds both, and the far village's adults are its members.
    assert!(p.holds(Some(a)) && p.holds(Some(b)));
    assert_eq!(p.settlements(), vec![a, b]);
    assert_eq!(p.held[0].from, taken);
    assert_eq!(
        pop.body_members(&fields, pi, now, &params).len(),
        before + theirs,
        "membership is residence in any of its settlements"
    );
    // Its store is the taker's now.
    assert!(
        (p.stores[grain] - held_before - 300.0).abs() < 1.0,
        "{}",
        p.stores[grain]
    );
    assert!(q.stores.iter().all(|&kg| kg == 0.0));
    // Its agreements ended.
    let x = pop.agreements.get(agreement).expect("kept");
    assert!(matches!(x.state, AgreementState::Ended { by: Some(1), .. }));
    // The chronicle tells it.
    assert!(
        pop.chronicle
            .iter()
            .any(|e| e.kind == ChronicleKind::PolityJoined && e.settlement == Some(b)),
        "the taking in is chronicled"
    );
    assert!(pop.problems(u64::MAX, usize::MAX).is_empty());

    // Days on, no polity is founded for the settlement taken in, and only the taker reviews.
    let laws_before = sim.people().polities[qi].laws.len();
    sim.advance_minutes(20 * DAY).expect("advances");
    let pop = sim.people();
    assert_eq!(
        pop.polities.len(),
        2,
        "no new polity for the settlement taken in"
    );
    assert_eq!(
        pop.polities[qi].laws.len(),
        laws_before,
        "it proposes nothing"
    );
    assert!(pop.polities[qi].gathering.is_none());
    assert!(pop.polities[pi].holds(Some(b)));
    saves_and_goes_on_alike(&mut sim, DAY);
}

/// Puts a law of `kind` in force in polity `pi`, known to all its adults, naming `holder` and
/// against `target`: its id.
fn a_law(
    sim: &mut Sim,
    pi: usize,
    kind: civ_agents::polity::PolicyKind,
    holder: Option<PermanentId>,
    target: Option<PermanentId>,
) -> PermanentId {
    use civ_agents::polity::{IssueKind, Law, Outcome as Decided};
    let policy = sim
        .rules()
        .catalog
        .policies
        .iter()
        .position(|d| d.kind == kind)
        .expect("the content has the policy");
    let id = sim.allocate_id_for_tests();
    let (now, grown) = (sim.now(), sim.rules().people.family.independent_age);
    let pop = sim.people_mut_for_tests();
    let mut adults: Vec<PermanentId> = pop
        .households
        .iter()
        .filter(|(_, h)| pop.polities[pi].holds(h.settlement))
        .flat_map(|(_, h)| h.members.clone())
        .filter(|&m| pop.person(m).is_some_and(|p| p.age_years(now) >= grown))
        .collect();
    adults.sort_unstable();
    let mut law = Law {
        id,
        policy: policy as u16,
        kind,
        levy_share: 0.0,
        holder,
        relief_days: 0.0,
        sanction: Default::default(),
        hours: (0, 0),
        status: LawStatus::InForce,
        sponsor: adults[0],
        proposed: now,
        issue: IssueKind::Raided,
        meets_day: now.day_index(),
        decided: Some(now),
        outcome: Some(Decided::Passed),
        eligible: adults.len() as u32,
        stances: Vec::new(),
        known: Vec::new(),
        compliance: Default::default(),
        watch: Default::default(),
        body: None,
        ends: None,
        agreement: None,
        target,
    };
    for &a in &adults {
        law.learn(a, now.day_index());
    }
    pop.polities[pi].laws.push(law);
    id
}

#[test]
fn a_band_a_law_sent_holds_the_village_it_took_the_field_at() {
    use civ_agents::places::PlaceHow;
    use civ_agents::polity::PolicyKind;
    use civ_agents::war::BandStage;
    let mut sim = two_villages(12, 30);
    let (a, b) = (sim.land().settlements[0].id, sim.land().settlements[1].id);
    let pi = seated(&sim, a);
    let now = sim.now();
    let adult = sim.rules().people.bands.min_age;
    let leader = {
        let pop = sim.people();
        let mut grown: Vec<PermanentId> = pop
            .people
            .iter()
            .filter(|(_, p)| p.age_years(now) >= adult && p.age_years(now) <= 50.0)
            .filter(|(_, p)| pop.household(p.household).and_then(|x| x.settlement) == Some(a))
            .map(|(_, p)| p.id)
            .collect();
        grown.sort_unstable();
        grown[0]
    };
    {
        let day = now.day_index();
        let pop = sim.people_mut_for_tests();
        let hh = pop.person(leader).expect("lives").household;
        pop.known_places.learn(hh, b, day, PlaceHow::Seen, None);
    }
    let law = a_law(&mut sim, pi, PolicyKind::SendBand, Some(leader), Some(b));
    // It is called, goes, takes the field and stays to hold the village; its people stay the
    // night there, asleep where they keep it.
    let mut holding_since = None;
    let mut slept_there = false;
    for _ in 0..(10 * 24) {
        sim.advance_minutes(60).expect("advances");
        let pop = sim.people();
        let Some(x) = pop.order.bands.iter().find(|x| x.law == Some(law)) else {
            continue;
        };
        if x.stage == BandStage::Holding {
            holding_since.get_or_insert(sim.now().minutes());
            slept_there |= x
                .arrived
                .iter()
                .filter(|m| !x.home.contains(m))
                .any(|&m| pop.person(m).is_some_and(|p| p.asleep));
        }
        if x.stage.over() {
            break;
        }
    }
    let since = holding_since.expect("the band held the field and stayed to hold the village");
    assert!(slept_there, "those holding it slept where they kept it");
    let pop = sim.people();
    let x = pop
        .order
        .bands
        .iter()
        .find(|x| x.law == Some(law))
        .expect("the band");
    // With the camp's weights its people go home one by one before the hold stands (a finding),
    // but not before the days the law's service owes.
    let service = (sim.rules().people.bands.service_days * DAY as f64) as i64;
    assert!(x.stage.over(), "{:?}", x.stage);
    assert!(
        x.since - since >= service,
        "they stayed at least the days service owes"
    );
    assert!(x.held.is_none());
    assert!(pop.problems(u64::MAX, usize::MAX).is_empty());
}

#[test]
fn a_hold_kept_its_days_is_a_holding_issue_and_the_annex_law_takes_the_village_in() {
    use civ_agents::polity::{IssueKind, PolicyKind};
    use civ_agents::war::{Band, BandPurpose, BandStage};
    let mut sim = two_villages(4, 30);
    let (a, b) = (sim.land().settlements[0].id, sim.land().settlements[1].id);
    let (pi, qi) = (seated(&sim, a), seated(&sim, b));
    let send = a_law(&mut sim, pi, PolicyKind::SendBand, None, Some(b));
    // A band of its law has kept a home of the other village eight days.
    let now = sim.now();
    let hold_days = i64::from(sim.rules().people.bands.hold_days);
    {
        let pop = sim.people_mut_for_tests();
        let mut ours: Vec<PermanentId> = pop
            .people
            .iter()
            .filter(|(_, p)| pop.household(p.household).and_then(|x| x.settlement) == Some(a))
            .filter(|(_, p)| p.age_years(now) >= 18.0)
            .map(|(_, p)| p.id)
            .collect();
        ours.sort_unstable();
        ours.truncate(4);
        let mut theirs: Vec<PermanentId> = pop
            .households
            .iter()
            .filter(|(_, h)| h.settlement == Some(b) && !h.members.is_empty())
            .map(|(_, h)| h.id)
            .collect();
        theirs.sort_unstable();
        let target = theirs[0];
        let place = pop.household(target).expect("household").home;
        pop.order.bands.push(Band {
            id: 1,
            organizer: ours[0],
            settlement: a,
            purpose: BandPurpose::Take,
            target,
            place,
            called: now.minutes() - (hold_days + 2) * DAY,
            muster: now.minutes() - (hold_days + 1) * DAY,
            members: ours.clone(),
            asked: Vec::new(),
            mustered: ours.clone(),
            arrived: ours.clone(),
            home: Vec::new(),
            stage: BandStage::Holding,
            since: now.minutes() - (hold_days + 1) * DAY,
            clash: None,
            incident: None,
            law: Some(send),
            taken: Vec::new(),
            taken_kcal: 0.0,
            approach: None,
            held: None,
        });
    }
    sim.with_ctx_for_tests(|pop, ctx| pop.holds_day_for_tests(ctx));
    let pop = sim.people();
    assert!(
        pop.order.bands[0].held.is_some(),
        "kept its days, it holds the village"
    );
    assert!(
        pop.chronicle
            .iter()
            .any(|e| e.kind == ChronicleKind::Raid && e.name.contains("has held")),
        "the hold is told"
    );
    assert_eq!(pop.held_by_for_tests(pi), Some(b));
    // Its gathering may weigh taking the village in: the issue is theirs, and a template answers
    // it.
    assert!(
        sim.rules()
            .catalog
            .policies
            .iter()
            .any(|d| d.kind == PolicyKind::Annex && d.answers.contains(&IssueKind::Holding))
    );
    // Passed, the village is taken in and the band goes home.
    let annex = a_law(&mut sim, pi, PolicyKind::Annex, None, Some(b));
    sim.with_ctx_for_tests(|pop, ctx| pop.annex_passed_for_tests(ctx, pi, annex, b));
    let pop = sim.people();
    assert!(pop.polities[pi].holds(Some(b)));
    assert!(pop.polities[qi].ended.is_some());
    assert_eq!(pop.held_by_for_tests(pi), None, "nothing left to take in");
    let l = pop.polities[pi]
        .laws
        .iter()
        .find(|l| l.id == annex)
        .expect("the law");
    assert_eq!(l.status, LawStatus::Lapsed, "it has run its course");
    sim.advance_minutes(2 * DAY).expect("advances");
    let pop = sim.people();
    assert!(pop.order.bands[0].stage.over(), "the band went home");
    assert!(pop.problems(u64::MAX, usize::MAX).is_empty());
    saves_and_goes_on_alike(&mut sim, DAY);
}

#[test]
fn an_agreement_of_one_body_in_force_makes_one_polity_of_two() {
    use civ_agents::polity::PolicyKind;
    let mut sim = two_villages(4, 30);
    let (a, b) = (sim.land().settlements[0].id, sim.land().settlements[1].id);
    let (pi, qi) = (seated(&sim, a), seated(&sim, b));
    let laws = [
        a_law(&mut sim, pi, PolicyKind::Agreement, None, None),
        a_law(&mut sim, qi, PolicyKind::Agreement, None, None),
    ];
    let id = sim.allocate_id_for_tests();
    let day = sim.now().day_index();
    {
        let pop = sim.people_mut_for_tests();
        let polities = [pop.polities[pi].id, pop.polities[qi].id];
        for (k, &i) in [pi, qi].iter().enumerate() {
            pop.polities[i].law_mut(laws[k]).expect("the law").agreement = Some(id);
        }
        let negotiators = [pi, qi].map(|i| pop.polities[i].laws.last().expect("law").sponsor);
        // Passed by both and heard of by each: it comes into force at the next midnight.
        pop.agreements.list.push(Agreement {
            id,
            polities,
            negotiators,
            clauses: vec![Clause::OneBody { custom_of: 0 }],
            term_days: 365,
            made: day,
            laws: [Some(laws[0]), Some(laws[1])],
            passed: [Some(day); 2],
            heard: [Some(day); 2],
            state: AgreementState::Offered,
        });
    }
    sim.advance_minutes(DAY).expect("advances");
    let pop = sim.people();
    let p = &pop.polities[pi];
    assert!(
        p.holds(Some(a)) && p.holds(Some(b)),
        "one body under the first's custom"
    );
    let end = pop.polities[qi].ended.expect("the second decides no more");
    assert_eq!(end.how, HeldHow::Merged);
    assert_eq!(p.held[0].how, HeldHow::Merged);
    assert!(
        pop.chronicle
            .iter()
            .any(|e| e.kind == ChronicleKind::PolityJoined && e.name.contains("one body")),
        "the merger is told"
    );
    assert!(pop.problems(u64::MAX, usize::MAX).is_empty());
    saves_and_goes_on_alike(&mut sim, DAY);
}
