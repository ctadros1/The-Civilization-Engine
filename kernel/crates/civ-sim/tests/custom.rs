//! Amending the custom (plan §7 M4c slice AF; ADR-0013 §2, ADR-0017 §1): those a gathering
//! overruled weigh changing who decides, one change at a time, judged by how the new rule would
//! have decided what they saw; the present custom decides it; a passed amendment changes who
//! belongs to the body, who comes and who counts, and is kept as a version of the custom with the
//! law that made it; all of it saves and loads exactly.

use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

use civ_agents::ChronicleKind;
use civ_agents::polity::{
    Body, Gathering, IssueKind, Law, LawStatus, Membership, Outcome, PolicyKind, Stance,
    StanceRecord,
};
use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_schema::{flatbuffers, wire};
use civ_sim::persist;
use civ_sim::{NewWorld, Sim};

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

/// The core content with the deliberators' choice made all but certain (a low temperature), so
/// the test sees what they would most likely do.
fn sure() -> ContentRegistry {
    let mut c = content().clone();
    c.people.params.polity.temperature = 0.05;
    c
}

fn world_with(content: &ContentRegistry, seed: u64) -> Sim {
    Sim::create(
        &NewWorld {
            name: "Custom".to_owned(),
            seed,
            preset_id: PRESET.to_owned(),
            size_cells: 512,
            band_size: 0,
            regime_id: String::new(),
        },
        content,
        &mut |_| {},
        &AtomicBool::new(false),
    )
    .expect("generates")
}

/// Each household's elder (its first member of an age to keep one), with the household, in
/// household order.
fn elders(sim: &Sim) -> Vec<(PermanentId, PermanentId)> {
    let (now, adult) = (sim.now(), sim.rules().people.family.independent_age);
    let pop = sim.people();
    let mut homes: Vec<_> = pop
        .households
        .iter()
        .map(|(_, h)| h)
        .filter(|h| h.settlement.is_some())
        .collect();
    homes.sort_by_key(|h| h.id);
    homes
        .iter()
        .filter_map(|h| {
            h.members
                .iter()
                .copied()
                .find(|&m| pop.person(m).is_some_and(|p| p.age_years(now) >= adult))
                .map(|e| (e, h.id))
        })
        .collect()
}

fn policy_of(sim: &Sim, kind: PolicyKind) -> u16 {
    sim.rules()
        .catalog
        .policies
        .iter()
        .position(|p| p.kind == kind)
        .expect("in core") as u16
}

/// A blank law of template `policy`, by id `id`, sponsored by `sponsor` today.
fn law(sim: &Sim, id: u64, policy: u16, sponsor: PermanentId) -> Law {
    let now = sim.now();
    Law {
        id: PermanentId::from_raw(sim.ids().peek_next() + 1_000_000 + id).expect("non-zero"),
        policy,
        kind: sim.rules().catalog.policies[usize::from(policy)].kind,
        levy_share: 0.0,
        holder: None,
        relief_days: 0.0,
        sanction: Default::default(),
        hours: (0, 0),
        status: LawStatus::Proposed,
        sponsor,
        proposed: now,
        issue: IssueKind::FoodShort,
        meets_day: now.day_index() + 1,
        decided: None,
        outcome: None,
        eligible: 0,
        stances: Vec::new(),
        known: vec![(sponsor, now.day_index())],
        compliance: Default::default(),
        watch: Default::default(),
        body: None,
    }
}

/// Saves `sim`, loads it, and checks every section of the loaded world is the same, and that the
/// two go on alike for `minutes`.
fn saves_and_goes_on_alike(sim: &mut Sim, content: &ContentRegistry, minutes: i64) {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "custom").expect("saves");
    let mut loaded = persist::load(&saved.path, content).expect("loads");
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
fn those_a_thin_gathering_overruled_propose_to_amend_the_custom_one_change_at_a_time() {
    let sure = sure();
    let mut sim = world_with(&sure, 3);
    sim.advance_minutes(40 * DAY).expect("advances");
    let elders = elders(&sim);
    assert!(
        elders.len() >= 6,
        "a village of households: {}",
        elders.len()
    );
    // A common store three elders came to back, turned down because too few came: under the
    // founding custom a quarter of the adults must come, and only they did. By hand, as if it had
    // happened today; what they make of it is their own.
    let store = policy_of(&sim, PolicyKind::CommonStore);
    let backers: Vec<(PermanentId, PermanentId)> = elders.iter().copied().take(3).collect();
    let mut thin = law(&sim, 0, store, backers[0].0);
    let now = sim.now();
    thin.levy_share = 0.1;
    thin.relief_days = 5.0;
    thin.status = LawStatus::Rejected;
    thin.decided = Some(now);
    thin.outcome = Some(Outcome::NoQuorum);
    thin.stances = backers
        .iter()
        .map(|&(person, household)| StanceRecord {
            person,
            household,
            stance: Stance::Support,
            gain: 5.0,
            regard: 0.0,
            opinion: 0.0,
        })
        .collect();
    let pop = sim.people_mut_for_tests();
    let p = &mut pop.polities[0];
    assert!(
        p.laws.is_empty() && p.gathering.is_none(),
        "nothing before it yet"
    );
    thin.eligible = 40;
    p.laws.push(thin);
    p.reviewed = now.day_index() - 30;
    // Within the next review, one of them puts an amendment.
    let mut proposed = None;
    for _ in 0..8 {
        sim.advance_minutes(DAY).expect("advances");
        proposed = sim.people().polities[0]
            .laws
            .iter()
            .find(|l| l.kind == PolicyKind::AmendBody)
            .cloned();
        if proposed.is_some() {
            break;
        }
    }
    let amend = proposed.expect("someone the gathering overruled proposed an amendment");
    assert!(backers.iter().any(|b| b.0 == amend.sponsor), "{amend:?}");
    assert_eq!(amend.issue, IssueKind::Overruled);
    let founding = Body::gathering(&sure.people.params.polity);
    let body = amend.body.expect("the body it would make");
    let changes = u32::from(body.members != founding.members)
        + u32::from((body.quorum_share - founding.quorum_share).abs() > 1e-6)
        + u32::from(body.pass != founding.pass);
    assert_eq!(changes, 1, "one change at a time: {body:?}");
    // Under the elders the thin gathering would have passed it; under any other single change
    // it would not: that is the one worth their proposing.
    assert_eq!(body.members, Membership::Elders, "{body:?}");
    let pop = sim.people();
    assert!(
        pop.chronicle.iter().any(
            |e| e.kind == ChronicleKind::LawProposed && e.name.contains("change of the custom")
        )
    );
    // The present custom decides it, whichever way.
    sim.advance_minutes(3 * DAY).expect("advances");
    let pop = sim.people();
    let decided = pop.polities[0]
        .laws
        .iter()
        .find(|l| l.id == amend.id)
        .expect("kept");
    assert!(decided.outcome.is_some(), "the gathering sat: {decided:?}");
    let versions = &pop.polities[0].versions;
    if decided.outcome == Some(Outcome::Passed) {
        assert_eq!(versions.len(), 2);
        assert_eq!(versions[1].law, Some(amend.id));
        assert_eq!(pop.polities[0].body, body);
    } else {
        assert_eq!(versions.len(), 1, "a failed amendment changes nothing");
        assert_eq!(pop.polities[0].body, founding);
    }
    saves_and_goes_on_alike(&mut sim, &sure, DAY);
}

#[test]
fn an_amendment_passed_changes_who_decides_and_is_kept_as_a_version() {
    let mut sim = world_with(content(), 4);
    sim.advance_minutes(40 * DAY).expect("advances");
    let elders = elders(&sim);
    let amend_policy = policy_of(&sim, PolicyKind::AmendBody);
    let mut amend = law(&sim, 1, amend_policy, elders[0].0);
    let founding = sim.people().polities[0].body;
    let body = Body {
        members: Membership::Elders,
        ..founding
    };
    amend.issue = IssueKind::Overruled;
    amend.body = Some(body);
    // Every household stands to gain by it: they back it at the gathering tomorrow, and every
    // adult has heard it is called.
    let mut stakes: Vec<(PermanentId, f32)> = sim
        .people()
        .households
        .iter()
        .map(|(_, h)| (h.id, 5.0))
        .collect();
    stakes.sort_by_key(|s| s.0);
    let (id, meets, now) = (amend.id, amend.meets_day, sim.now());
    let params = sim.rules().people.clone();
    let pop = sim.people_mut_for_tests();
    let p = &mut pop.polities[0];
    assert!(
        p.laws.is_empty() && p.gathering.is_none(),
        "nothing before it yet"
    );
    p.laws.push(amend);
    p.gathering = Some(Gathering {
        law: Some(id),
        day: meets,
        stakes,
        present: Vec::new(),
        cases: Vec::new(),
    });
    pop.hear_of_gatherings_called(now, &params);
    sim.advance_minutes(2 * DAY).expect("advances");
    let pop = sim.people();
    let polity = &pop.polities[0];
    let passed = polity.laws.iter().find(|l| l.id == id).expect("kept");
    assert_eq!(passed.outcome, Some(Outcome::Passed), "{passed:?}");
    assert_eq!(passed.status, LawStatus::InForce);
    // The custom changed by its own procedure, and both versions are kept.
    assert_eq!(polity.body, body);
    assert_eq!(polity.versions.len(), 2);
    assert_eq!(polity.versions[0].body, founding);
    assert_eq!(polity.versions[0].law, None);
    assert_eq!(polity.versions[1].law, Some(id));
    assert!(
        pop.chronicle
            .iter()
            .any(|e| e.kind == ChronicleKind::CustomAmended && e.name.contains("the elders"))
    );
    // Now only the elders belong: one to a household.
    let now = sim.now();
    let members = pop.body_members(&sim.land().fields, 0, now, &sim.rules().people);
    let elders_now = self::elders(&sim);
    assert_eq!(members.len(), elders_now.len());
    assert!(
        members.iter().all(|m| elders_now.contains(m)),
        "{members:?}"
    );
    // The Government panel says so, and the label reads the share the body admits.
    let payload = civ_sim::frames::government::government_response(&sim);
    let info = flatbuffers::root::<wire::Response>(&payload)
        .expect("a response")
        .body_as_government()
        .expect("the government");
    let line = info.polities().expect("polities").get(0);
    assert_eq!(line.body_members() as usize, members.len());
    assert!(
        line.members() as usize > members.len(),
        "fewer decide than live there"
    );
    let history: Vec<&str> = line.custom_history().expect("versions").iter().collect();
    assert_eq!(history.len(), 2);
    assert!(history[1].contains("by the amendment"), "{history:?}");
    assert!(line.custom().is_some_and(|c| c.starts_with("The elders")));
    saves_and_goes_on_alike(&mut sim, content(), 2 * DAY);
}
