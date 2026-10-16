//! Limits and review (M7b slice BT, ADR-0025 §3-4): a limit in force forbids laws whose fields
//! meet its predicate; a law that breaches it stands until someone it binds and burdens challenges
//! it before a forum with review, which may strike it down. A law the limit does not bind, or a
//! polity whose forum may not review, sees no challenge. A challenge saves and loads exactly.

use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

use civ_agents::crime::CaseStage;
use civ_agents::polity::{IssueKind, Law, LawStatus, Outcome as Decided};
use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_sim::persist;
use civ_sim::{NewWorld, Sim};

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

/// The situation under test, not the tuning's balance: keeping grown members home in a curfew's
/// hours costs a household dear, and bringing a case costs nothing beyond regard.
fn nudge(sim: &mut Sim) {
    let people = &mut sim
        .rules_mut_for_tests()
        .expect("rules not yet shared")
        .people;
    people.crime.curfew_cost_days = 5.0;
    people.crime.report_cost = 0.0;
}

fn world(seed: u64) -> Sim {
    let mut sim = Sim::create(
        &NewWorld {
            name: "Review".to_owned(),
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
    nudge(&mut sim);
    sim
}

/// The settlement's grown members, in id order.
fn adults(sim: &Sim) -> Vec<PermanentId> {
    let (now, grown) = (sim.now(), sim.rules().people.family.independent_age);
    let pop = sim.people();
    let settlement = pop.polities[0].settlement;
    let mut out: Vec<PermanentId> = pop
        .households
        .iter()
        .filter(|(_, h)| h.settlement == Some(settlement))
        .flat_map(|(_, h)| h.members.clone())
        .filter(|&m| pop.person(m).is_some_and(|p| p.age_years(now) >= grown))
        .collect();
    out.sort_unstable();
    out
}

/// A law of template `template` in force at the first polity, with `hours`, known to all.
fn law(sim: &mut Sim, template: &str, hours: (u8, u8)) -> PermanentId {
    let policies = &sim.rules().catalog.policies;
    let policy = policies
        .iter()
        .position(|d| d.id == template)
        .expect("the core content has the template");
    let kind = policies[policy].kind;
    let id = sim.allocate_id_for_tests();
    let now = sim.now();
    let everyone = adults(sim);
    let pop = sim.people_mut_for_tests();
    let mut law = Law {
        id,
        policy: policy as u16,
        kind,
        levy_share: 0.0,
        holder: None,
        relief_days: 0.0,
        sanction: Default::default(),
        hours,
        status: LawStatus::InForce,
        sponsor: everyone[0],
        proposed: now,
        issue: IssueKind::Takings,
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

/// Its review is due at the next midnight.
fn review_due(sim: &mut Sim) {
    let (day, every) = (
        sim.now().day_index(),
        i64::from(sim.rules().people.polity.review_days),
    );
    sim.people_mut_for_tests().polities[0].reviewed = day - every;
}

/// Lives up to `days` days until a challenge is brought; returns its case id.
fn challenged(sim: &mut Sim, days: i64) -> Option<u32> {
    for _ in 0..days {
        sim.advance_minutes(DAY).expect("advances");
        if let Some(c) = sim
            .people()
            .order
            .cases
            .iter()
            .find(|c| c.challenge.is_some())
        {
            return Some(c.id);
        }
    }
    None
}

#[test]
fn a_curfew_over_the_morning_light_is_challenged_and_may_be_struck_down() {
    let mut sim = world(3);
    let curfew = law(&mut sim, "core:policy/curfew", (21, 5));
    let limit = law(&mut sim, "core:policy/limit_daylight_work", (0, 0));
    let forum = law(&mut sim, "core:policy/hear_by_elders", (0, 0));
    review_due(&mut sim);
    let id = challenged(&mut sim, 3).expect("someone the curfew keeps home challenges it");
    let c = sim.people().order.case(id).cloned().expect("the challenge");
    assert_eq!(
        (c.law, c.challenge, c.forum),
        (curfew, Some(limit), Some(forum))
    );
    assert_eq!(c.incident, 0, "a challenge names no incident");
    // Its forum hears it the next midnight.
    sim.advance_minutes(2 * DAY).expect("advances");
    let pop = sim.people();
    let c = pop.order.case(id).cloned().expect("kept");
    assert!(
        matches!(c.stage, CaseStage::Found | CaseStage::NotFound),
        "{c:?}"
    );
    assert!(!c.stances.is_empty());
    for r in &c.stances {
        assert!(
            r.household != c.accuser && r.household != c.accused_household,
            "a party sat: {r:?}"
        );
        // A clear breach read as one: the reading's points lean toward striking it.
        assert!(r.belief > 0.0, "{r:?}");
    }
    let status = pop.polities[0]
        .laws
        .iter()
        .find(|l| l.id == curfew)
        .expect("kept")
        .status;
    if c.stage == CaseStage::Found {
        assert_eq!(status, LawStatus::StruckDown);
        assert!(
            pop.chronicle
                .iter()
                .any(|e| e.name.contains("breaks the limit")),
            "the chronicle says so"
        );
    } else {
        assert_eq!(status, LawStatus::InForce);
    }
    // In this world the elders struck it down.
    assert_eq!(c.stage, CaseStage::Found, "{c:?}");
}

#[test]
fn no_challenge_without_a_breach_or_without_review() {
    // A curfew that covers no light the limit counts.
    let mut sim = world(3);
    law(&mut sim, "core:policy/curfew", (23, 4));
    law(&mut sim, "core:policy/limit_daylight_work", (0, 0));
    law(&mut sim, "core:policy/hear_by_elders", (0, 0));
    review_due(&mut sim);
    assert_eq!(challenged(&mut sim, 3), None, "nothing to challenge");
    // A breach, but a forum that may not review.
    let mut sim = world(3);
    law(&mut sim, "core:policy/curfew", (21, 5));
    law(&mut sim, "core:policy/limit_daylight_work", (0, 0));
    law(&mut sim, "core:policy/hear_by_panel", (0, 0));
    review_due(&mut sim);
    assert_eq!(challenged(&mut sim, 3), None, "no forum to hear it");
}

#[test]
fn a_challenge_saves_and_loads_exactly() {
    let mut sim = world(3);
    law(&mut sim, "core:policy/curfew", (21, 5));
    law(&mut sim, "core:policy/limit_daylight_work", (0, 0));
    law(&mut sim, "core:policy/hear_by_elders", (0, 0));
    review_due(&mut sim);
    challenged(&mut sim, 3).expect("a challenge");
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = persist::save(
        &mut sim,
        &saves,
        commons_persist::SaveKind::Manual,
        "review",
    )
    .expect("saves");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
    nudge(&mut loaded);
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
fn a_ruling_is_known_person_by_person() {
    use civ_agents::word::ClaimKind;
    let mut sim = world(3);
    let curfew = law(&mut sim, "core:policy/curfew", (21, 5));
    law(&mut sim, "core:policy/limit_daylight_work", (0, 0));
    law(&mut sim, "core:policy/hear_by_elders", (0, 0));
    review_due(&mut sim);
    let id = challenged(&mut sim, 3).expect("a challenge");
    sim.advance_minutes(DAY).expect("advances");
    let (today, remember) = (
        sim.now().day_index(),
        sim.rules().people.crime.remember_days,
    );
    let pop = sim.people();
    let law = pop.polities[0]
        .laws
        .iter()
        .find(|l| l.id == curfew)
        .expect("kept");
    assert_eq!(law.status, LawStatus::StruckDown);
    let heard = |p: PermanentId| {
        pop.word.heard_by(p).iter().any(|h| {
            pop.word
                .claim(h.claim)
                .is_some_and(|c| c.kind == ClaimKind::Finding && c.event == Some(id))
        })
    };
    let (knew, not): (Vec<PermanentId>, Vec<PermanentId>) =
        adults(&sim).into_iter().partition(|&p| heard(p));
    assert!(!knew.is_empty() && !not.is_empty(), "{knew:?} {not:?}");
    // Who heard the ruling no longer keeps the curfew; who knew the law and has not heard it does.
    for p in knew {
        assert!(!pop.still_keeps(p, law, today, remember), "{p}");
    }
    for p in not {
        assert!(pop.still_keeps(p, law, today, remember), "{p}");
    }
}
