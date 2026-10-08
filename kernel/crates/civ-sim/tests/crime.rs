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
    let spell = sim.now().minutes();
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
        // Where the taker's household stood by food among its neighbours (the dashboard's crime
        // row): a measure, never an input.
        assert!(
            i.food_days >= 0.0 && (0.0..=1.0).contains(&i.richer),
            "{} days of food, {} of the neighbours richer",
            i.food_days,
            i.richer
        );
    }
    // While the other half still had food, those who went to take came from the half with none.
    let early: Vec<f32> = order
        .incidents
        .iter()
        .filter(|i| i.at.minutes() < spell + 5 * DAY)
        .map(|i| i.richer)
        .collect();
    if !early.is_empty() {
        let mean = early.iter().map(|&r| f64::from(r)).sum::<f64>() / early.len() as f64;
        assert!(mean > 0.5, "takers' neighbours richer: {early:?}");
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

/// Puts a law of kind `kind` in force at the first polity, as a gathering would have passed it,
/// known to every adult there: for a law against taking, with the template's harshest bundle (it
/// exiles); for an office, naming the first adult by id; for a curfew, the template's first hours. Returns the polity's id and the law's.
fn a_law(sim: &mut Sim, kind: civ_agents::polity::PolicyKind) -> (PermanentId, PermanentId) {
    use civ_agents::polity::{IssueKind, Law, LawStatus, Outcome as Decided};
    let policies = &sim.rules().catalog.policies;
    let policy = policies
        .iter()
        .position(|d| d.kind == kind)
        .expect("the core content has the template");
    let sanction = policies[policy].bundles.last().copied().unwrap_or_default();
    let hours = policies[policy].hours.first().copied().unwrap_or((0, 0));
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
    let office = matches!(
        kind,
        civ_agents::polity::PolicyKind::KeepStore | civ_agents::polity::PolicyKind::KeepWatch
    );
    let mut law = Law {
        id,
        policy: policy as u16,
        kind,
        levy_share: 0.0,
        holder: office.then_some(adults[0]),
        relief_days: 0.0,
        sanction,
        hours,
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
        watch: Default::default(),
        body: None,
    };
    for &a in &adults {
        law.learn(a, now.day_index());
    }
    pop.polities[0].laws.push(law);
    (pop.polities[0].id, id)
}

#[test]
fn a_watch_walks_its_rounds_at_night_within_the_nights_limit() {
    use civ_agents::polity::PolicyKind;
    let mut sim = world_with(content(), 7);
    sim.advance_minutes(3 * DAY).expect("advances");
    let (_, law) = a_law(&mut sim, PolicyKind::KeepWatch);
    let nights = 6;
    sim.advance_minutes(nights * DAY).expect("advances");
    let p = &sim.people().polities[0];
    let l = p.laws.iter().find(|l| l.id == law).expect("the law");
    let w = &l.watch;
    let rounds_per_night = sim.rules().people.crime.rounds_per_night;
    assert!(
        w.rounds > 0,
        "the watch walked no round in {nights} nights: {w:?}"
    );
    assert!(
        w.rounds <= rounds_per_night * (nights as u32 + 1),
        "more rounds than nights allow: {w:?}"
    );
    assert!(u32::from(w.tonight) <= rounds_per_night);
    // A round's stands take the activity's minutes each, and the watch counts them.
    let stand = sim
        .rules()
        .catalog
        .activities
        .iter()
        .find(|a| a.behavior == civ_agents::Behavior::Watch)
        .expect("the activity")
        .min_minutes;
    assert!(w.minutes >= f64::from(stand), "{w:?}");
    assert_eq!(p.watcher().map(|(h, _)| h), l.holder);
    saves_and_goes_on_alike(&mut sim, content(), DAY);
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
    let (polity, _) = a_law(&mut sim, civ_agents::polity::PolicyKind::AgainstTaking);
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
            // The harshest bundle exiles: the one found to have taken has left the valley, a
            // migration on their record, not a disappearance (ADR-0015 §5).
            assert!(c.exiled, "{c:?}");
            assert!(
                pop.person(c.accused).is_none(),
                "{} is still here",
                c.accused
            );
            let record = pop.records.get(&c.accused).expect("a record");
            assert!(record.left.is_some() && record.died.is_none(), "{record:?}");
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

#[test]
fn a_watcher_chooses_once_what_to_do_with_a_taking_they_saw() {
    use civ_agents::crime::Kept;
    use civ_agents::polity::PolicyKind;
    let mut sim = world_with(content(), 8);
    sim.advance_minutes(3 * DAY).expect("advances");
    // The situation under test: takings succeed and whoever is about sees them, and the watcher's
    // rounds pass every home, so the watcher sees some.
    let content_crime = sim.rules().people.crime.clone();
    {
        let crime = &mut sim
            .rules_mut_for_tests()
            .expect("rules not yet shared")
            .people
            .crime;
        crime.guardian_age = 200.0;
        crime.notice_chance = 0.0;
        crime.round_stops = 1000;
    }
    let (_, watch) = a_law(&mut sim, PolicyKind::KeepWatch);
    let (_, _) = a_law(&mut sim, PolicyKind::AgainstTaking);
    let officer = sim.people().polities[0]
        .watcher()
        .map(|(h, _)| h)
        .expect("the watcher");
    lean_spell(&mut sim);
    let held_before = sim.people().goods_held();
    let flows_before = sim.people().flows();
    for _ in 0..12 {
        if !sim.people().order.sightings.is_empty() {
            break;
        }
        sim.advance_minutes(5 * DAY).expect("advances");
    }
    sim.advance_minutes(2 * DAY).expect("advances");
    let pop = sim.people();
    let order = &pop.order;
    assert!(
        !order.sightings.is_empty(),
        "the watcher saw no taking in 60 days"
    );
    let mut seen = BTreeSet::new();
    for s in &order.sightings {
        // Once each, by the one who keeps the watch, of a taking they did see.
        assert!(seen.insert((s.officer, s.incident)), "{s:?} chosen twice");
        assert_eq!(s.officer, officer);
        let i = order.incident(s.incident).expect("its incident");
        assert!(i.seen_by.contains(&officer));
        match s.kept {
            // Kept quiet: nobody learnt it from them.
            Kept::LookedAway | Kept::Paid => assert!(
                !order
                    .beliefs
                    .iter()
                    .any(|b| b.incident == s.incident && b.from == Some(officer)),
                "{s:?} was told"
            ),
            Kept::Reported | Kept::Refused => assert!(
                order.cases.iter().any(|c| c.incident == s.incident),
                "{s:?} brought no case"
            ),
            Kept::Told => panic!("a law against taking was in force: {s:?}"),
        }
        assert_eq!(s.kept == Kept::Paid, s.kcal > 0.0, "{s:?}");
    }
    // The watch counts the cases it brought.
    let brought = order
        .sightings
        .iter()
        .filter(|s| matches!(s.kept, Kept::Reported | Kept::Refused))
        .filter(|s| {
            order
                .cases
                .iter()
                .any(|c| c.incident == s.incident && c.by == officer)
        })
        .count() as u32;
    let law = pop.polities[0]
        .laws
        .iter()
        .find(|l| l.id == watch)
        .expect("the watch");
    assert_eq!(law.watch.cases, brought);
    let goods = &sim.rules().catalog.goods;
    let gaps = population::unaccounted(
        goods.len(),
        (&held_before, &flows_before),
        (&pop.goods_held(), &pop.flows()),
    );
    assert!(gaps.is_empty(), "unaccounted for: {gaps:?}");
    sim.rules_mut_for_tests()
        .expect("rules not yet shared")
        .people
        .crime = content_crime;
    saves_and_goes_on_alike(&mut sim, content(), DAY);
}

#[test]
fn a_watcher_without_objection_asks_to_be_paid_and_the_payment_moves_on_its_channel() {
    use civ_agents::crime::Kept;
    use civ_agents::polity::PolicyKind;
    let mut sim = world_with(content(), 9);
    sim.advance_minutes(3 * DAY).expect("advances");
    // The situation under test: takings succeed and are seen, the watcher passes every home, the
    // watcher is one who would ask (no objection to taking, no sense of being found out, no pull
    // to report), and a taker's household can pay (test values).
    let content_crime = sim.rules().people.crime.clone();
    {
        let crime = &mut sim
            .rules_mut_for_tests()
            .expect("rules not yet shared")
            .people
            .crime;
        crime.guardian_age = 200.0;
        crime.notice_chance = 0.0;
        crime.round_stops = 1000;
        crime.w_watch_report = -20.0;
        // A household that took can pay from whatever it holds (a taker's household rarely has
        // food to spare beyond three days' need).
        crime.keep_days = 0.0;
        // Hearing of takings raises anyone's sense that takers are seen; here being found out
        // costs the watcher nothing. And they walk most of the night.
        crime.w_seen = 0.0;
        crime.w_watch = 30.0;
        crime.rounds_per_night = 12;
    }
    let (_, _) = a_law(&mut sim, PolicyKind::KeepWatch);
    let (_, against) = a_law(&mut sim, PolicyKind::AgainstTaking);
    if let Some(l) = sim.people_mut_for_tests().polities[0]
        .laws
        .iter_mut()
        .find(|l| l.id == against)
    {
        l.sanction.exile = false;
    }
    let officer = sim.people().polities[0]
        .watcher()
        .map(|(h, _)| h)
        .expect("the watcher");
    lean_spell(&mut sim);
    // The watcher's household has food. With no objection the watcher may still take once it is
    // given away to neighbours who ask; the law here does not exile, so a finding leaves them on
    // watch.
    let goods = sim.rules().catalog.goods.clone();
    let need = sim.rules().people.household.daily_kcal_per_person;
    let provisions = goods
        .iter()
        .position(|g| g.id == "core:good/provisions")
        .expect("provisions");
    let pop = sim.people_mut_for_tests();
    let home = pop.person(officer).map(|p| p.household).expect("a home");
    for (_, h) in pop.households.iter_mut() {
        let days = if h.id == home {
            60.0
        } else if h.stores[provisions] <= 0.0 {
            // Short, and so may take, but with something a watcher could be paid from.
            4.0
        } else {
            0.0
        };
        h.stores[provisions] +=
            days * need * h.members.len() as f64 / goods[provisions].kcal_per_kg;
    }
    for (_, p) in pop.people.iter_mut() {
        if p.id == officer {
            p.objection = 0.0;
            p.risk_seen = 0.0;
        }
    }
    let held_before = sim.people().goods_held();
    let flows_before = sim.people().flows();
    let bribes = |sim: &Sim| {
        (0..goods.len())
            .map(|g| sim.people().transfers.get(civ_agents::Channel::Bribe, g))
            .sum::<f64>()
    };
    let asked = |sim: &Sim| {
        sim.people()
            .order
            .sightings
            .iter()
            .any(|s| matches!(s.kept, Kept::Paid | Kept::Refused))
    };
    // Short households are kept at two days' food (the core keeps five): short enough to take,
    // and with something to pay a watcher from. The food given in is counted as given.
    let mut given = vec![0.0; goods.len()];
    for _ in 0..30 {
        if asked(&sim) {
            break;
        }
        let pop = sim.people_mut_for_tests();
        for (_, h) in pop.households.iter_mut() {
            let want = 2.0 * need * h.members.len() as f64 / goods[provisions].kcal_per_kg;
            if h.id != home && h.stores[provisions] < want {
                given[provisions] += want - h.stores[provisions];
                h.stores[provisions] = want;
            }
        }
        sim.advance_minutes(2 * DAY).expect("advances");
    }
    let pop = sim.people();
    let order = &pop.order;
    assert!(
        asked(&sim),
        "the watcher never asked: {:?}",
        order.sightings
    );
    let paid: f64 = order
        .sightings
        .iter()
        .filter(|s| s.kept == Kept::Paid)
        .map(|s| f64::from(s.kcal))
        .sum();
    // What was paid moved on the `bribe` channel, from the takers' households to the watcher's,
    // and nothing was made.
    let moved: f64 = (0..goods.len())
        .map(|g| pop.transfers.get(civ_agents::Channel::Bribe, g) * goods[g].kcal_per_kg)
        .sum();
    assert!(
        (moved - paid).abs() <= 1.0 + 1e-3 * paid,
        "{moved} moved, {paid} paid"
    );
    assert_eq!(bribes(&sim) > 0.0, paid > 0.0);
    for s in order.sightings.iter().filter(|s| s.kept == Kept::Refused) {
        assert!(
            order
                .cases
                .iter()
                .any(|c| c.incident == s.incident && c.by == officer)
        );
    }
    // Goods are conserved: what the test gave in is all that was made.
    let held: Vec<f64> = held_before.iter().zip(&given).map(|(h, g)| h + g).collect();
    let gaps = population::unaccounted(
        goods.len(),
        (&held, &flows_before),
        (&pop.goods_held(), &pop.flows()),
    );
    assert!(gaps.is_empty(), "unaccounted for: {gaps:?}");
    sim.rules_mut_for_tests()
        .expect("rules not yet shared")
        .people
        .crime = content_crime;
    saves_and_goes_on_alike(&mut sim, content(), DAY);
}

/// People away from their homes' plots (20 m) at each third of an hour of `nights` days' curfew
/// hours, `hours`, counted together, as `sim` lives them.
fn away_in_hours(sim: &mut Sim, days: i64, hours: (u8, u8)) -> u32 {
    let mut away = 0;
    for _ in 0..days {
        // The next time the hours begin, and when they end.
        let mut from = sim.now().day_index() * DAY + i64::from(hours.0) * 60;
        if from < sim.now().minutes() {
            from += DAY;
        }
        let long = (i64::from(hours.1) - i64::from(hours.0)).rem_euclid(24) * 60;
        let to = from + long;
        sim.advance_minutes(from - sim.now().minutes())
            .expect("advances");
        while sim.now().minutes() < to {
            let t = sim.now().minutes() as f64;
            let pop = sim.people();
            away += pop
                .people
                .iter()
                .filter(|(_, p)| {
                    pop.household(p.household).is_some_and(|h| {
                        let at = p.position_at(t);
                        (at.0 - h.home.0).hypot(at.1 - h.home.1) > 20.0
                    })
                })
                .count() as u32;
            sim.advance_minutes(20).expect("advances");
        }
    }
    away
}

/// A curfew (M4b slice AD): in its hours, a village whose people know of it has fewer of them away
/// from home than the same village without it; those who go out anyway break it, and the law
/// keeps count; and it saves and loads exactly. In the template's night hours nearly everyone is
/// asleep at home already, so to see whether keeping it weighs with people, this curfew runs
/// through the working day, when most of what they do takes them away.
#[test]
fn under_a_curfew_fewer_are_away_in_its_hours_and_breaches_are_counted() {
    use civ_agents::polity::PolicyKind;
    let mut sim = world_with(content(), 5);
    sim.advance_minutes(2 * DAY).expect("advances");
    // The same village twice: one under a curfew, one not.
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(&mut sim, &saves, commons_persist::SaveKind::Manual, "free").expect("saves");
    let mut free = persist::load(&saved.path, content()).expect("loads");
    let (_, law) = a_law(&mut sim, PolicyKind::Curfew);
    let hours = (9, 17);
    let l = sim.people_mut_for_tests().polities[0]
        .laws
        .iter_mut()
        .find(|l| l.id == law)
        .expect("the law");
    assert_ne!(l.hours, (0, 0), "the template sets hours");
    l.hours = hours;
    let nights = 3;
    let without = away_in_hours(&mut free, nights, hours);
    let with = away_in_hours(&mut sim, nights, hours);
    assert!(
        without > 0,
        "somebody is out in those hours without a curfew, or this test shows nothing"
    );
    assert!(
        with < without,
        "a curfew everyone knows keeps some in: {with} away under it, {without} without"
    );
    let l = sim.people().polities[0]
        .laws
        .iter()
        .find(|l| l.id == law)
        .expect("the law");
    // Everyone grown knows it: only children, who do not, break it unknowing.
    eprintln!(
        "curfew {hours:?}: {with} away under it, {without} without; broken {} knowing, {} not",
        l.compliance.broken, l.compliance.broken_unaware
    );
    assert!(
        with == 0 || l.compliance.broken + l.compliance.broken_unaware > 0,
        "those who went out are counted: {:?}",
        l.compliance
    );
    saves_and_goes_on_alike(&mut sim, content(), DAY);
}
