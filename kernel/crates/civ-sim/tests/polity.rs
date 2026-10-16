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
            neighbours: Vec::new(),
            neighbours_known: false,
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

#[test]
fn the_notables_tier_is_a_setting_a_loaded_world_has_on() {
    let mut sim = world_with(content(), 3);
    assert!(sim.notable_tier(), "a new world has the tier");
    sim.set_notable_tier(false);
    assert!(!sim.notable_tier());
    sim.advance_minutes(2 * DAY).expect("advances");
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(&mut sim, &saves, commons_persist::SaveKind::Manual, "tier").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    assert!(
        loaded.notable_tier(),
        "never saved: a loaded world has it on"
    );
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
    // Until a gathering has decided on a common store. Hunger can bring taking, and a watch may
    // be proposed and decided first (about one world in forty).
    for _ in 0..30 {
        sim.advance_minutes(DAY).expect("advances");
        if common_store_decided(&sim).is_some() {
            break;
        }
    }
    (sim, sure)
}

/// The first common store a gathering decided on, if any.
fn common_store_decided(sim: &Sim) -> Option<&Law> {
    sim.people()
        .polities
        .first()?
        .laws
        .iter()
        .find(|l| l.outcome.is_some() && l.kind == PolicyKind::CommonStore)
}

#[test]
fn a_village_whose_food_will_not_last_proposes_and_its_gathering_decides() {
    let (mut sim, sure) = lean_village();
    check_the_history(&sim);
    // And all of it saves and loads exactly, and goes on alike.
    saves_and_goes_on_alike(&mut sim, &sure, 3 * DAY);
    labels_name_it_and_change_nothing(&mut sim, &sure);
}

/// The village is labelled from its history (ADR-0013 §6), and a copy of it whose labels and
/// government panel are worked out every day lives exactly as one never labelled.
fn labels_name_it_and_change_nothing(sim: &mut Sim, content: &ContentRegistry) {
    let label = civ_sim::labels::label_of(sim, &sim.people().polities[0]);
    assert!(label.name.starts_with("Council community"), "{label:?}");
    assert!(
        label.why.iter().any(|w| w.contains("since it was founded")),
        "{label:?}"
    );
    assert!(label.confidence >= 0.2, "{label:?}");
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "labels").expect("saves");
    let mut plain = persist::load(&saved.path, content).expect("loads");
    let mut labelled = persist::load(&saved.path, content).expect("loads");
    for _ in 0..5 {
        plain.advance_minutes(DAY).expect("advances");
        labelled.advance_minutes(DAY).expect("advances");
        for p in &labelled.people().polities {
            let _ = civ_sim::labels::label_of(&labelled, p);
        }
        let _ = civ_sim::frames::government::government_response(&labelled);
    }
    let (a, b) = (
        persist::encode_sections(&plain),
        persist::encode_sections(&labelled),
    );
    for (x, y) in a.iter().zip(&b) {
        assert!(
            x.tag == persist::SECTION_META || x.bytes == y.bytes,
            "section `{}` differs once labelled",
            x.tag
        );
    }
}

/// The history of the first common store a gathering decided, stage by stage.
fn check_the_history(sim: &Sim) {
    let pop = sim.people();
    let pp = &sim.rules().people.polity;
    let polity = &pop.polities[0];
    let law: &Law = common_store_decided(sim)
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
    // Each stance is the forecast, the regard for the sponsor, what talk had moved them and what
    // the law does to what they hold dear (M4c slice AG), against the margin.
    for r in law.stances.iter().filter(|r| r.person != law.sponsor) {
        let s =
            f64::from(r.gain) + f64::from(r.regard) + f64::from(r.opinion) + f64::from(r.values);
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
        kind: PolicyKind::CommonStore,
        levy_share: 0.2,
        holder: None,
        relief_days: 5.0,
        sanction: Default::default(),
        hours: (0, 0),
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
        watch: Default::default(),
        body: None,
        ends: None,
        agreement: None,
        target: None,
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
        kind: policies[usize::from(policy)].kind,
        levy_share: if holder.is_some() { 0.0 } else { 0.1 },
        holder,
        relief_days: 5.0,
        sanction: Default::default(),
        hours: (0, 0),
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
        watch: Default::default(),
        body: None,
        ends: None,
        agreement: None,
        target: None,
    };
    let p = &mut pop.polities[0];
    // The world's own store and keeper laws, if its gathering passed any in these 250 days (a
    // world of random identity sometimes does), give way to the test's.
    for l in &mut p.laws {
        if l.status == LawStatus::InForce
            && matches!(l.kind, PolicyKind::CommonStore | PolicyKind::KeepStore)
        {
            l.status = LawStatus::Lapsed;
        }
    }
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

/// An office held by turns, drawn by lot (M7a slice BN, ADR-0024 §2): a lot draws its keeper from
/// those who stood for it, its term ends on its day and the office lapses, the one who kept it
/// cannot be drawn for the next turn, a lot with nobody to draw lapses the law, and all of it
/// saves and loads.
#[test]
fn a_keeper_drawn_by_lot_keeps_for_a_term_and_cannot_be_drawn_again_at_once() {
    use civ_agents::polity::{IssueKind, StanceRecord};
    let mut sim = world_with(content(), 3);
    sim.advance_minutes(DAY).expect("advances");
    let now = sim.now();
    let policies = sim.rules().catalog.policies.clone();
    let turns = policies
        .iter()
        .position(|p| p.id == "core:policy/your_turn_keeping_the_store")
        .expect("in core") as u16;
    let tenure = policies[usize::from(turns)].tenure.expect("held by turns");
    assert!(tenure.by_lot && !tenure.renewable);
    let adult = sim.rules().people.family.independent_age;
    let pop = sim.people();
    let mut adults: Vec<(civ_core::PermanentId, civ_core::PermanentId)> = pop
        .people
        .iter()
        .map(|(_, p)| p)
        .filter(|p| p.age_years(now) >= adult)
        .map(|p| (p.id, p.household))
        .collect();
    adults.sort_unstable();
    assert!(adults.len() >= 5);
    let next = sim.ids().peek_next();
    let stood = |who: &[(civ_core::PermanentId, civ_core::PermanentId)], stance| {
        who.iter()
            .map(|&(person, household)| StanceRecord {
                person,
                household,
                stance,
                gain: 0.0,
                regard: 0.0,
                opinion: 0.0,
                values: 0.0,
                view: 0.0,
                owed: 0.0,
            })
            .collect::<Vec<_>>()
    };
    let law = |id: u64, stances: Vec<StanceRecord>| Law {
        id: civ_core::PermanentId::from_raw(next + 1_000_000 + id).expect("non-zero"),
        policy: turns,
        kind: PolicyKind::KeepStore,
        levy_share: 0.0,
        holder: None,
        relief_days: 5.0,
        sanction: Default::default(),
        hours: (0, 0),
        status: LawStatus::InForce,
        sponsor: adults[0].0,
        proposed: now,
        issue: IssueKind::StoreUnkept,
        meets_day: now.day_index(),
        decided: Some(now),
        outcome: Some(Outcome::Passed),
        eligible: adults.len() as u32,
        stances,
        known: Vec::new(),
        compliance: Default::default(),
        watch: Default::default(),
        body: None,
        ends: None,
        agreement: None,
        target: None,
    };
    // Three stood for it, one against: the lot draws among the three.
    let mut stances = stood(&adults[..3], Stance::Support);
    stances.extend(stood(&adults[3..4], Stance::Oppose));
    let first = law(0, stances);
    let first_id = first.id;
    {
        let p = &mut sim.people_mut_for_tests().polities[0];
        for l in &mut p.laws {
            if l.status == LawStatus::InForce && l.kind == PolicyKind::KeepStore {
                l.status = LawStatus::Lapsed;
            }
        }
        p.laws.push(first);
    }
    sim.with_ctx_for_tests(|pop, ctx| pop.fill_term_for_tests(ctx, 0, first_id));
    let p = &sim.people().polities[0];
    let keeper = p
        .laws
        .iter()
        .find(|l| l.id == first_id)
        .and_then(|l| l.holder);
    let keeper = keeper.expect("the lot drew a keeper");
    assert!(
        adults[..3].iter().any(|a| a.0 == keeper),
        "drawn from those who stood for it"
    );
    let term = p.term_of(first_id).expect("held for a term");
    assert_eq!(term.pool, 3);
    assert_eq!(term.until, now.day_index() + i64::from(tenure.term_days));
    assert!(
        sim.people()
            .chronicle
            .iter()
            .any(|e| e.kind == ChronicleKind::LotDrawn && e.people == vec![keeper]),
        "the chronicle tells the lot"
    );
    // The Government panel says how it was filled and until when.
    let payload = civ_sim::frames::government::government_response(&sim);
    let info = civ_schema::flatbuffers::root::<civ_schema::wire::Response>(&payload)
        .expect("a response")
        .body_as_government()
        .expect("the government");
    let offices: Vec<&str> = info
        .polities()
        .expect("polities")
        .get(0)
        .offices()
        .expect("offices")
        .iter()
        .collect();
    assert!(
        offices
            .iter()
            .any(|o| o.contains("drawn by lot from 3 who put their names in")),
        "{offices:?}"
    );
    saves_and_goes_on_alike(&mut sim, content(), DAY);

    // Past its term, the office lapses: their term ended.
    sim.advance_minutes((i64::from(tenure.term_days) + 1) * DAY)
        .expect("advances");
    let p = &sim.people().polities[0];
    let ended = p.laws.iter().find(|l| l.id == first_id).expect("kept");
    assert_eq!(ended.status, LawStatus::Lapsed);
    assert!(
        sim.people()
            .chronicle
            .iter()
            .any(|e| e.kind == ChronicleKind::LawLapsed && e.name.contains("their term ended")),
        "the chronicle tells the term's end"
    );
    // The next lot cannot fall to the last keeper: with them and one other in, it draws the
    // other, whatever the key.
    let other = adults
        .iter()
        .copied()
        .find(|a| a.0 != keeper && sim.people().person(a.0).is_some());
    let other = other.expect("another adult");
    let again = adults
        .iter()
        .copied()
        .find(|a| a.0 == keeper)
        .expect("the keeper");
    for k in 1..=4u64 {
        let second = law(k, stood(&[again, other], Stance::Support));
        let second_id = second.id;
        sim.people_mut_for_tests().polities[0].laws.push(second);
        sim.with_ctx_for_tests(|pop, ctx| pop.fill_term_for_tests(ctx, 0, second_id));
        let l = sim.people().polities[0]
            .laws
            .iter()
            .find(|l| l.id == second_id)
            .expect("kept");
        assert_eq!(
            l.holder,
            Some(other.0),
            "the last keeper may not be drawn at once"
        );
        // Undone, so that the next draw is again the turn after the first keeper's.
        let p = &mut sim.people_mut_for_tests().polities[0];
        p.laws.retain(|l| l.id != second_id);
        p.terms.retain(|t| t.0 != second_id);
    }
    // With nobody who stood for it, the lot draws no one and the law lapses.
    let empty = law(9, stood(&adults[3..4], Stance::Oppose));
    let empty_id = empty.id;
    sim.people_mut_for_tests().polities[0].laws.push(empty);
    sim.with_ctx_for_tests(|pop, ctx| pop.fill_term_for_tests(ctx, 0, empty_id));
    let l = sim.people().polities[0]
        .laws
        .iter()
        .find(|l| l.id == empty_id)
        .expect("kept");
    assert_eq!((l.status, l.holder), (LawStatus::Lapsed, None));
    saves_and_goes_on_alike(&mut sim, content(), DAY);
}
