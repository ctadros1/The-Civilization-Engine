//! The polity (ADR-0013; plan §7 M4a slice Z): every settlement has one under the founding
//! custom; a village whose food will not last proposes, gathers and decides, and the whole of it
//! is kept; a common store in force takes its levy at threshing and answers asks; and all of it
//! saves and loads exactly.

use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

use civ_agents::ChronicleKind;
use civ_agents::polity::{Body, Law, LawStatus, Outcome, PolicyKind, Stance};
use civ_agents::population;
use civ_content::ContentRegistry;
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
            name: "Polity".to_owned(),
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

/// Takes every household's food away (a test's lean season): nobody can spare any.
fn empty_food(sim: &mut Sim) {
    let goods = sim.rules().catalog.goods.clone();
    for (_, h) in sim.people_mut_for_tests().households.iter_mut() {
        for (kg, g) in h.stores.iter_mut().zip(&goods) {
            if g.purpose == civ_agents::params::GoodUse::Food {
                *kg = 0.0;
            }
        }
    }
}

/// Saves `sim`, loads it, and checks every section of the loaded world is the same, and that the
/// two go on alike for `minutes`.
fn saves_and_goes_on_alike(sim: &mut Sim, content: &ContentRegistry, minutes: i64) {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "polity").expect("saves");
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
fn every_settlement_has_one_polity_under_the_founding_custom() {
    let mut sim = world_with(content(), 3);
    sim.advance_minutes(2 * DAY).expect("advances");
    let pop = sim.people();
    let params = &sim.rules().people.polity;
    assert_eq!(pop.polities.len(), sim.land().settlements.len());
    for (p, s) in pop.polities.iter().zip(&sim.land().settlements) {
        assert_eq!(p.settlement, s.id);
        assert_ne!(p.id, s.id, "a polity is not its settlement");
        assert_eq!(
            p.body,
            Body::gathering(params),
            "every world starts from one custom"
        );
        assert!(p.laws.is_empty() && p.gathering.is_none());
        assert!(
            p.stores.iter().all(|kg| *kg == 0.0),
            "its store starts empty"
        );
    }
}

/// A village whose food will not last, from early June, with the deliberators' choice made all
/// but certain (a low temperature) so the test sees a proposal: what it does with it is its
/// own. Lived until a gathering has decided, at most three weeks.
fn lean_village() -> (Sim, ContentRegistry) {
    let mut sure = content().clone();
    sure.people.params.polity.temperature = 0.05;
    let mut sim = world_with(&sure, 3);
    sim.advance_minutes(95 * DAY).expect("advances");
    empty_food(&mut sim);
    for _ in 0..21 {
        sim.advance_minutes(DAY).expect("advances");
        let decided = sim
            .people()
            .polities
            .iter()
            .any(|p| p.laws.iter().any(|l| l.outcome.is_some()));
        if decided {
            break;
        }
    }
    (sim, sure)
}

#[test]
fn a_village_whose_food_will_not_last_proposes_and_its_gathering_decides() {
    let (mut sim, sure) = lean_village();
    check_the_history(&sim);
    // And all of it saves and loads exactly, and goes on alike.
    saves_and_goes_on_alike(&mut sim, &sure, 3 * DAY);
}

/// The history of the first law a gathering decided, stage by stage.
fn check_the_history(sim: &Sim) {
    let pop = sim.people();
    let pp = &sim.rules().people.polity;
    let polity = &pop.polities[0];
    let law: &Law = polity
        .laws
        .iter()
        .find(|l| l.outcome.is_some())
        .expect("someone proposed a common store and a gathering decided on it");
    // Stage 1: a sponsor, an issue, and a level the template allows.
    let def = &sim.rules().catalog.policies[usize::from(law.policy)];
    assert_eq!(def.kind, PolicyKind::CommonStore);
    assert!(
        def.levy_shares
            .iter()
            .any(|&s| (s - f64::from(law.levy_share)).abs() < 1e-6)
    );
    assert!(pop.records.contains_key(&law.sponsor));
    assert!(
        law.known.iter().any(|k| k.0 == law.sponsor),
        "its sponsor knows it"
    );
    // Stages 2-3: the stances of those present, and the body's rule applied to them.
    let (present, support, oppose) = law.counts();
    assert!(present <= law.eligible);
    assert!(law.stances.windows(2).all(|w| w[0].person < w[1].person));
    let outcome = law.outcome.expect("decided");
    assert_eq!(
        outcome,
        polity.body.decide(law.eligible, present, support, oppose)
    );
    match outcome {
        Outcome::Passed => {
            assert_eq!(law.status, LawStatus::InForce);
            for r in &law.stances {
                assert!(law.knows(r.person), "those present know a law they passed");
            }
        }
        _ => assert_eq!(law.status, LawStatus::Rejected),
    }
    // Each stance is the forecast and the regard for the sponsor, against the margin.
    for r in law.stances.iter().filter(|r| r.person != law.sponsor) {
        let s = f64::from(r.gain) + f64::from(r.regard);
        let want = if s > pp.stance_margin {
            Stance::Support
        } else if s < -pp.stance_margin {
            Stance::Oppose
        } else {
            Stance::Abstain
        };
        assert!(
            r.stance == want || (s.abs() - pp.stance_margin).abs() < 1e-4,
            "{r:?}"
        );
    }
    // The chronicle has the proposal and the decision, in words.
    let entries = |kind| {
        pop.chronicle
            .iter()
            .filter(move |e| e.kind == kind && e.people.first() == Some(&law.sponsor))
    };
    assert!(entries(ChronicleKind::LawProposed).count() >= 1);
    let decided: Vec<_> = entries(ChronicleKind::LawDecided).collect();
    assert!(!decided.is_empty());
    assert!(decided.iter().any(|e| e.name.contains("gathering")));
}

#[test]
fn a_common_store_in_force_takes_its_levy_at_threshing_and_answers_asks() {
    let mut sim = world_with(content(), 3);
    sim.advance_minutes(120 * DAY).expect("advances");
    let now = sim.now();
    let policy = sim
        .rules()
        .catalog
        .policies
        .iter()
        .position(|p| p.kind == PolicyKind::CommonStore)
        .expect("the core content has a common store") as u16;
    let id = sim.ids().peek_next();
    // A law put in force by hand, known to everyone: what it asks and gives is what is tested.
    let pop = sim.people_mut_for_tests();
    let mut known: Vec<_> = pop.people.iter().map(|(_, p)| (p.id, 0)).collect();
    known.sort_unstable();
    let sponsor = known[0].0;
    pop.polities[0].laws.push(Law {
        id: civ_core::PermanentId::from_raw(id + 1_000_000).expect("non-zero"),
        policy,
        levy_share: 0.2,
        holder: None,
        relief_days: 5.0,
        status: LawStatus::InForce,
        sponsor,
        proposed: now,
        issue: civ_agents::polity::IssueKind::FoodShort,
        meets_day: now.day_index(),
        decided: Some(now),
        outcome: Some(Outcome::Passed),
        eligible: 0,
        stances: Vec::new(),
        known,
        compliance: Default::default(),
    });
    let held_before = sim.people().goods_held();
    let flows_before = sim.people().flows();
    // Through the harvest and its threshing.
    sim.advance_minutes(100 * DAY).expect("advances");
    let pop = sim.people();
    let goods = &sim.rules().catalog.goods;
    let c = pop.polities[0].laws[0].compliance;
    assert!(
        c.complied + c.could_not + c.evaded > 0,
        "the threshers met the levy: {c:?}"
    );
    assert!(c.complied > 0, "most pay a levy known to all: {c:?}");
    let grain = sim.rules().catalog.crops[0].good;
    assert!(c.levied_kg > 0.0);
    assert!(pop.polities[0].stores[grain] > 0.0, "the store holds grain");
    // Every good is accounted for, the store's included (ADR-0006 §3, ADR-0013 §4).
    let gaps = population::unaccounted(
        goods.len(),
        (&held_before, &flows_before),
        (&pop.goods_held(), &pop.flows()),
    );
    assert!(gaps.is_empty(), "unaccounted for: {gaps:?}");
    // A lean spell after the harvest: with nobody able to spare food, the store answers asks.
    empty_food(&mut sim);
    sim.advance_minutes(5 * DAY).expect("advances");
    let c = sim.people().polities[0].laws[0].compliance;
    assert!(c.relieved > 0, "the store answered asks: {c:?}");
    assert!(c.relief_kg > 0.0);
    saves_and_goes_on_alike(&mut sim, content(), 2 * DAY);
}

#[test]
fn a_keeper_keeps_the_store_under_a_roof_until_they_are_gone() {
    let mut sim = world_with(content(), 3);
    // Early November: the households are under their roofs.
    sim.advance_minutes(250 * DAY).expect("advances");
    let now = sim.now();
    let policies = sim.rules().catalog.policies.clone();
    let of = |kind: PolicyKind| {
        policies
            .iter()
            .position(|p| p.kind == kind)
            .expect("in core") as u16
    };
    let (store, keep) = (of(PolicyKind::CommonStore), of(PolicyKind::KeepStore));
    let grain = sim.rules().catalog.crops[0].good;
    let adult = sim.rules().people.family.independent_age;
    let next = sim.ids().peek_next();
    let pop = sim.people_mut_for_tests();
    let keeper = pop
        .people
        .iter()
        .map(|(_, p)| p)
        .filter(|p| p.age_years(now) >= adult)
        .filter(|p| pop.household(p.household).is_some_and(|x| x.sheltered))
        .map(|p| p.id)
        .min()
        .expect("someone lives under a roof");
    let mut known: Vec<_> = pop.people.iter().map(|(_, p)| (p.id, 0)).collect();
    known.sort_unstable();
    let law = |id: u64, policy: u16, holder: Option<civ_core::PermanentId>| Law {
        id: civ_core::PermanentId::from_raw(next + 1_000_000 + id).expect("non-zero"),
        policy,
        levy_share: if holder.is_some() { 0.0 } else { 0.1 },
        holder,
        relief_days: 5.0,
        status: LawStatus::InForce,
        sponsor: keeper,
        proposed: now,
        issue: civ_agents::polity::IssueKind::FoodShort,
        meets_day: now.day_index(),
        decided: Some(now),
        outcome: Some(Outcome::Passed),
        eligible: 0,
        stances: Vec::new(),
        known: known.clone(),
        compliance: Default::default(),
    };
    let p = &mut pop.polities[0];
    p.laws.push(law(0, store, None));
    p.laws.push(law(1, keep, Some(keeper)));
    p.stores.resize(grain + 1, 0.0);
    p.stores[grain] = 1000.0;
    p.stores_at = now;
    assert_eq!(p.keeper().map(|k| k.0), Some(keeper));
    // A hundred days under their roof: grain loses about 1.4 % (4.3 % in the open).
    sim.advance_minutes(100 * DAY).expect("advances");
    let spoiled = sim.people().polities[0]
        .flows
        .get(civ_agents::person::Flow::Spoiled, grain);
    assert!(spoiled < 25.0, "kept under a roof: {spoiled} kg spoiled");
    // Their household moves away: the law lapses, and the chronicle says so.
    let pop = sim.people_mut_for_tests();
    let home = pop.person(keeper).expect("alive").household;
    for (_, x) in pop.households.iter_mut() {
        if x.id == home {
            x.settlement = None;
        }
    }
    sim.advance_minutes(DAY).expect("advances");
    let pop = sim.people();
    let kept = pop.polities[0]
        .laws
        .iter()
        .find(|l| l.holder == Some(keeper))
        .expect("the law");
    assert_eq!(kept.status, LawStatus::Lapsed);
    assert!(pop.polities[0].keeper().is_none());
    assert!(
        pop.chronicle
            .iter()
            .any(|e| e.kind == ChronicleKind::LawLapsed && e.people == vec![keeper])
    );
}
