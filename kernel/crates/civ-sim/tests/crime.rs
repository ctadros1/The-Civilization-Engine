//! Taking (ADR-0015; plan §7 M4b slice AA): a well-fed village takes nothing; where some
//! households are out of food and nobody can spare any, those whose objection lets them weigh it
//! may take, and nobody above the moral filter does; goods stay accounted for; what people
//! believe traces to those who saw; and all of it saves and loads exactly.

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

use civ_agents::Channel;
use civ_agents::crime::{Obligation, Outcome, Owed, Source, Standing};
use civ_agents::params::GoodUse;
use civ_agents::population;
use civ_content::ContentRegistry;
use civ_core::PermanentId;
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

fn world_with(content: &ContentRegistry, seed: u64) -> Sim {
    Sim::create(
        &NewWorld {
            name: "Order".to_owned(),
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

/// Saves `sim`, loads it, and checks every section of the loaded world is the same, and that the
/// two go on alike for `minutes`.
fn saves_and_goes_on_alike(sim: &mut Sim, content: &ContentRegistry, minutes: i64) {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "order").expect("saves");
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
fn a_well_fed_village_takes_nothing() {
    // A band carries food for eighteen months: in its first two months nobody is short.
    let mut sim = world_with(content(), 3);
    sim.advance_minutes(60 * DAY).expect("advances");
    let order = &sim.people().order;
    assert!(
        order.incidents.is_empty(),
        "nobody short of food, nobody takes: {:?}",
        order.incidents
    );
}

/// A lean spell some households live through with nothing: half of them lose all their food, the
/// rest keep six days' worth, too little to spare (a test's shared shortage). In the households
/// with nothing, every other grown member can weigh taking (an objection of 0.3) and the others
/// cannot (0.95, above the filter). Returns those who could and those who could not.
fn lean_spell(sim: &mut Sim) -> (BTreeSet<PermanentId>, BTreeSet<PermanentId>) {
    let goods = sim.rules().catalog.goods.clone();
    let params = sim.rules().people.clone();
    let provisions = goods
        .iter()
        .position(|g| g.id == "core:good/provisions")
        .expect("provisions");
    let now = sim.now();
    let pop = sim.people_mut_for_tests();
    let mut ids: Vec<PermanentId> = pop.households.iter().map(|(_, h)| h.id).collect();
    ids.sort_unstable();
    let hungry: BTreeSet<PermanentId> = ids.iter().copied().step_by(2).collect();
    let (mut may, mut may_not) = (BTreeSet::new(), BTreeSet::new());
    for (_, h) in pop.households.iter_mut() {
        for (kg, g) in h.stores.iter_mut().zip(&goods) {
            if g.purpose == GoodUse::Food {
                *kg = 0.0;
            }
        }
        if !hungry.contains(&h.id) {
            let need = h.members.len() as f64 * params.household.daily_kcal_per_person;
            h.stores[provisions] = 6.0 * need / goods[provisions].kcal_per_kg;
        }
    }
    let mut hungry_members: Vec<PermanentId> = pop
        .households
        .iter()
        .filter(|(_, h)| hungry.contains(&h.id))
        .flat_map(|(_, h)| h.members.clone())
        .collect();
    hungry_members.sort_unstable();
    for (k, m) in hungry_members.into_iter().enumerate() {
        let Some(p) = pop.people.iter_mut().map(|(_, p)| p).find(|p| p.id == m) else {
            continue;
        };
        if p.age_years(now) < params.family.independent_age {
            continue;
        }
        if k % 2 == 0 {
            p.objection = 0.3;
            may.insert(m);
        } else {
            p.objection = 0.95;
            may_not.insert(m);
        }
    }
    (may, may_not)
}

#[test]
fn the_hungry_take_from_those_with_food_and_word_follows_what_was_seen() {
    let mut sim = world_with(content(), 4);
    sim.advance_minutes(3 * DAY).expect("advances");
    // The situation under test: a taker notices nobody about, so those about see them rather than
    // turn them back (with the content's attentive neighbours about 1 lean spell in 24 sees no
    // taking in 60 days). A member at home still turns them back.
    let content_crime = sim.rules().people.crime.clone();
    sim.rules_mut_for_tests()
        .expect("rules not yet shared")
        .people
        .crime
        .notice_chance = 0.0;
    let (may, may_not) = lean_spell(&mut sim);
    assert!(!may.is_empty() && !may_not.is_empty());
    let held_before = sim.people().goods_held();
    let flows_before = sim.people().flows();
    // Most who go to take turn back: someone is home or about (guardianship, settled on
    // arrival). Live on until someone has taken, and a few days more for word to go round.
    let taken = |sim: &Sim| {
        sim.people()
            .order
            .incidents
            .iter()
            .any(|i| i.outcome == Outcome::Taken)
    };
    for _ in 0..12 {
        if taken(&sim) {
            break;
        }
        sim.advance_minutes(5 * DAY).expect("advances");
    }
    assert!(
        taken(&sim),
        "someone with nothing and an objection that lets them weigh it took food in 60 days: {:?}",
        sim.people().order.incidents
    );
    sim.advance_minutes(3 * DAY).expect("advances");
    let pop = sim.people();
    let order = &pop.order;
    assert!(
        order
            .incidents
            .iter()
            .filter(|i| i.outcome == Outcome::TurnedBack)
            .all(|i| i.seen_by.is_empty() && i.kcal == 0.0),
        "nobody sees a taker who turned back, and they carry nothing"
    );
    // The moral filter: nobody above it took, or went to take (research 04-09 §5.3).
    for i in &order.incidents {
        assert!(
            !may_not.contains(&i.actor),
            "{} is above the filter and went to take",
            i.actor
        );
        assert!(i.goods.is_empty() == (i.outcome != Outcome::Taken));
    }
    // Takings move goods and never make them (ADR-0015 §2).
    let goods = &sim.rules().catalog.goods;
    let gaps = population::unaccounted(
        goods.len(),
        (&held_before, &flows_before),
        (&pop.goods_held(), &pop.flows()),
    );
    assert!(gaps.is_empty(), "unaccounted for: {gaps:?}");
    // Every belief that names a taker traces to someone who saw it or to the taker themself
    // (ADR-0015 §1), and names the one who did it; a loss found without a witness names nobody.
    for b in &order.beliefs {
        let i = order.incident(b.incident).expect("its incident");
        match b.taker {
            Some(t) => {
                assert_eq!(t, i.actor, "a belief names someone who did not do it");
                let origin = b.origin.expect("an account has an origin");
                assert!(
                    origin == i.actor || i.seen_by.contains(&origin),
                    "{b:?} traces to nobody who saw it"
                );
                if matches!(b.source, Source::Saw | Source::Did) {
                    assert_eq!(origin, b.holder);
                }
                if b.source == Source::Did {
                    assert_eq!(b.holder, i.actor, "only the taker knows by doing");
                }
            }
            None => assert_eq!(b.source, Source::Noticed),
        }
    }
    // Every taking was found by the household taken from by the next midnight.
    let today = sim.now().day_index();
    for i in order
        .incidents
        .iter()
        .filter(|i| i.outcome == Outcome::Taken)
    {
        assert!(
            i.noticed || i.at.day_index() == today,
            "{} was never found",
            i.id
        );
    }
    a_debt_meant_to_be_paid_is_paid_from_what_can_be_spared(&mut sim);
    // A loaded world runs by the content's rules: so must this one, to go on alike.
    sim.rules_mut_for_tests()
        .expect("rules not yet shared")
        .people
        .crime = content_crime;
    saves_and_goes_on_alike(&mut sim, content(), 2 * DAY);
}

/// The paying of a demand (ADR-0015 §5), which the lean spell rarely reaches (a household that took
/// has nothing to spare): the household of the first taker still here owes the food back and has
/// said it will pay. Given a month's food just before a midnight (given earlier, it would go to
/// hungry neighbours who ask), it gives back what it owes from what it can spare at that midnight,
/// on its own channel, and every good stays accounted for.
fn a_debt_meant_to_be_paid_is_paid_from_what_can_be_spared(sim: &mut Sim) {
    let goods = sim.rules().catalog.goods.clone();
    let params = sim.rules().people.clone();
    let provisions = goods
        .iter()
        .position(|g| g.id == "core:good/provisions")
        .expect("provisions");
    let to_midnight = DAY - sim.now().minutes().rem_euclid(DAY);
    sim.advance_minutes(to_midnight - 10).expect("advances");
    let day = sim.now().day_index();
    let pop = sim.people_mut_for_tests();
    let Some((incident, debtor, beneficiary, kcal)) = pop
        .order
        .incidents
        .iter()
        .filter(|i| i.outcome == Outcome::Taken)
        .filter_map(|i| {
            let debtor = pop.person(i.actor)?.household;
            (pop.household(debtor).is_some() && pop.household(i.target).is_some())
                .then_some((i.id, debtor, i.target, i.kcal))
        })
        .next()
    else {
        panic!("a taker and the household they took from are still here");
    };
    let id = pop.order.obligations.last().map_or(1, |o| o.id + 1);
    pop.order.obligations.push(Obligation {
        id,
        incident,
        kind: Owed::Demanded,
        case: None,
        debtor,
        beneficiary,
        kcal,
        paid_kcal: 0.0,
        made: day,
        due: day + 30,
        standing: Standing::Open,
        answer: Some((true, 0.0)),
    });
    for (_, h) in pop.households.iter_mut() {
        if h.id == debtor {
            let need = h.members.len() as f64 * params.household.daily_kcal_per_person;
            h.stores[provisions] += 30.0 * need / goods[provisions].kcal_per_kg;
        }
    }
    let held_before = sim.people().goods_held();
    let flows_before = sim.people().flows();
    let moved_before = sim.people().transfers.get(Channel::Restitution, provisions);
    sim.advance_minutes(20).expect("advances");
    let pop = sim.people();
    let o = pop
        .order
        .obligations
        .iter()
        .find(|o| o.id == id)
        .expect("the obligation");
    assert_eq!(o.standing, Standing::Met, "{o:?}");
    assert!((o.paid_kcal - o.kcal).abs() <= 1.0 + 1e-3 * o.kcal, "{o:?}");
    assert!(pop.transfers.get(Channel::Restitution, provisions) > moved_before);
    let gaps = population::unaccounted(
        goods.len(),
        (&held_before, &flows_before),
        (&pop.goods_held(), &pop.flows()),
    );
    assert!(gaps.is_empty(), "unaccounted for: {gaps:?}");
    assert!(
        pop.chronicle
            .iter()
            .any(|e| e.kind == civ_agents::ChronicleKind::Restitution
                && e.name.contains("gave back")),
        "the chronicle tells it"
    );
}

/// Puts a law against taking in force at the first polity, as a gathering would have passed it,
/// with the template's harshest bundle, known to every adult there. Returns the polity's id.
fn a_law_against_taking(sim: &mut Sim) -> PermanentId {
    use civ_agents::polity::{IssueKind, Law, LawStatus, Outcome as Decided, PolicyKind};
    let policies = &sim.rules().catalog.policies;
    let policy = policies
        .iter()
        .position(|d| d.kind == PolicyKind::AgainstTaking)
        .expect("the core content has a law against taking");
    let sanction = *policies[policy].bundles.last().expect("a bundle");
    let id = sim.allocate_id_for_tests();
    let (now, grown) = (sim.now(), sim.rules().people.family.independent_age);
    let pop = sim.people_mut_for_tests();
    let settlement = pop.polities[0].settlement;
    let mut adults: Vec<PermanentId> = pop
        .households
        .iter()
        .filter(|(_, h)| h.settlement == Some(settlement))
        .flat_map(|(_, h)| h.members.clone())
        .filter(|&m| pop.person(m).is_some_and(|p| p.age_years(now) >= grown))
        .collect();
    adults.sort_unstable();
    let mut law = Law {
        id,
        policy: policy as u16,
        levy_share: 0.0,
        holder: None,
        relief_days: 0.0,
        sanction,
        status: LawStatus::InForce,
        sponsor: adults[0],
        proposed: now,
        issue: IssueKind::Takings,
        meets_day: now.day_index(),
        decided: Some(now),
        outcome: Some(Decided::Passed),
        eligible: adults.len() as u32,
        stances: Vec::new(),
        known: Vec::new(),
        compliance: Default::default(),
    };
    for &a in &adults {
        law.learn(a, now.day_index());
    }
    pop.polities[0].laws.push(law);
    pop.polities[0].id
}

#[test]
fn under_a_law_against_taking_cases_are_brought_heard_and_their_findings_owed() {
    use civ_agents::crime::{CaseStage, Choice};
    use civ_agents::polity::Stance;
    let mut sim = world_with(content(), 6);
    sim.advance_minutes(3 * DAY).expect("advances");
    // The situation under test, not the tuning's balance: nobody is a capable guardian and a
    // taker notices nobody, so a taking succeeds and whoever is about sees it; and a household
    // that learns who took from it brings a case rather than letting it go or demanding the food
    // back.
    let content_crime = sim.rules().people.crime.clone();
    {
        let crime = &mut sim
            .rules_mut_for_tests()
            .expect("rules not yet shared")
            .people
            .crime;
        crime.guardian_age = 200.0;
        crime.notice_chance = 0.0;
        crime.report_cost = -20.0;
        crime.demand_base = -20.0;
    }
    let polity = a_law_against_taking(&mut sim);
    let (may, _) = lean_spell(&mut sim);
    assert!(!may.is_empty());
    let held_before = sim.people().goods_held();
    let flows_before = sim.people().flows();
    let heard = |sim: &Sim| {
        sim.people()
            .order
            .cases
            .iter()
            .any(|c| c.stage != CaseStage::Open)
    };
    for _ in 0..12 {
        if heard(&sim) {
            break;
        }
        sim.advance_minutes(5 * DAY).expect("advances");
    }
    // A few days more for what a finding imposes to be answered and paid.
    sim.advance_minutes(3 * DAY).expect("advances");
    let pop = sim.people();
    let order = &pop.order;
    assert!(
        heard(&sim),
        "a case was heard within 60 days: {:?}",
        order.cases
    );
    assert!(
        order
            .responses
            .iter()
            .any(|r| r.choice == Choice::Report && r.report_points.is_some())
    );
    for c in &order.cases {
        // What the gathering was told traces to what happened: the household taken from
        // accuses the one who took, on the word of those who saw (ADR-0015 §1, §4).
        let i = order.incident(c.incident).expect("its incident");
        assert_eq!(c.accuser, i.target, "{c:?}");
        assert_eq!(c.accused, i.actor, "{c:?}");
        assert!(!c.leads.is_empty(), "a case with no account: {c:?}");
        assert!(c.leads.iter().all(|w| i.seen_by.contains(w)), "{c:?}");
        if c.stage == CaseStage::Open {
            continue;
        }
        assert!(c.heard.is_some());
        assert!(c.stances.windows(2).all(|w| w[0].person < w[1].person));
        for r in &c.stances {
            if r.person == c.by {
                assert_eq!(r.stance, Stance::Support);
            }
            if r.person == c.accused {
                assert_eq!(r.stance, Stance::Oppose);
            }
        }
        let owed: Vec<_> = order
            .obligations
            .iter()
            .filter(|o| o.case == Some(c.id))
            .collect();
        if c.stage == CaseStage::Found {
            // The bundle: restitution if anything is still owed, compensation, and a fine to the
            // polity's store; all owed by the accused's household.
            assert!(
                owed.iter().any(|o| o.kind == Owed::Compensation),
                "{owed:?}"
            );
            assert!(
                owed.iter()
                    .any(|o| o.kind == Owed::Fine && o.beneficiary == polity),
                "{owed:?}"
            );
            assert!(
                owed.iter()
                    .all(|o| o.kind == Owed::Fine || o.beneficiary == c.accuser)
            );
            // What a finding imposes is answered once, whole.
            let answers: BTreeSet<_> = owed.iter().filter_map(|o| o.answer.map(|a| a.0)).collect();
            assert!(answers.len() <= 1, "{owed:?}");
        } else {
            assert!(
                owed.is_empty(),
                "nothing is owed without a finding: {owed:?}"
            );
        }
    }
    // Cases move goods only as obligations are paid, and never make them (ADR-0015 §5).
    let goods = &sim.rules().catalog.goods;
    let gaps = population::unaccounted(
        goods.len(),
        (&held_before, &flows_before),
        (&pop.goods_held(), &pop.flows()),
    );
    assert!(gaps.is_empty(), "unaccounted for: {gaps:?}");
    let told = |k: civ_agents::ChronicleKind| pop.chronicle.iter().any(|e| e.kind == k);
    assert!(told(civ_agents::ChronicleKind::CaseBrought));
    assert!(told(civ_agents::ChronicleKind::CaseHeard));
    // A loaded world runs by the content's rules: so must this one, to go on alike.
    sim.rules_mut_for_tests()
        .expect("rules not yet shared")
        .people
        .crime = content_crime;
    saves_and_goes_on_alike(&mut sim, content(), 2 * DAY);
}
