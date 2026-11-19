//! The polity (ADR-0013; plan §7 M4a slice Z): every settlement has one under the founding
//! custom; a village whose food will not last proposes, gathers and decides, and the whole of it
//! is kept; a common store in force takes its levy at threshing and answers asks; and all of it
//! saves and loads exactly.

use std::path::Path;
use std::sync::OnceLock;

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
    Sim::create_for_tests(
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
        [7; 16],
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

/// A stub model (ADR-0039 §3): answers each request with `pick` of its moves, and counts them.
struct Stub {
    pick: fn(usize) -> civ_agents::deliberation::ModelAnswer,
    asked: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

impl civ_agents::deliberation::ModelProvider for Stub {
    fn fingerprint(&self) -> civ_agents::deliberation::ProviderFingerprint {
        civ_agents::deliberation::ProviderFingerprint {
            kind: "stub".into(),
            model: "test".into(),
        }
    }
    fn answer(
        &mut self,
        request: &civ_agents::deliberation::DeliberationRequest,
    ) -> civ_agents::deliberation::ModelAnswer {
        self.asked
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        (self.pick)(request.moves.len())
    }
}

/// Every law proposed: polity, sponsor, day, template.
fn proposals(sim: &Sim) -> Vec<(usize, u64, i64, u16)> {
    sim.people()
        .polities
        .iter()
        .enumerate()
        .flat_map(|(i, p)| {
            p.laws
                .iter()
                .map(move |l| (i, l.sponsor.get(), l.proposed.day_index(), l.policy))
        })
        .collect()
}

/// Each law's "because" in the government panel's words.
fn issues(sim: &Sim) -> Vec<String> {
    let payload = civ_sim::frames::government::government_response(sim);
    let info = civ_schema::flatbuffers::root::<civ_schema::wire::Response>(&payload)
        .expect("a response")
        .body_as_government()
        .expect("the government");
    info.polities()
        .expect("polities")
        .iter()
        .flat_map(|p| p.laws().expect("laws").iter())
        .map(|l| l.issue().unwrap_or_default().to_owned())
        .collect()
}

fn sections(sim: &Sim) -> Vec<(String, Vec<u8>)> {
    persist::encode_sections(sim)
        .into_iter()
        .map(|s| (s.tag.to_string(), s.bytes))
        .collect()
}

#[test]
fn a_model_chooses_among_the_moves_and_a_recorded_world_relives_its_choices() {
    use civ_agents::deliberation::{ChosenBy, DeliberationMode, ModelAnswer};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    // A lean village about to weigh moves, saved.
    let mut sure = content().clone();
    sure.people.params.polity.temperature = 0.05;
    let mut sim = world_with(&sure, 3);
    sim.advance_minutes(95 * DAY).expect("advances");
    empty_food(&mut sim);
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let start = persist::save(&mut sim, &saves, commons_persist::SaveKind::Manual, "lean")
        .expect("saves")
        .path;
    let days = 21;
    let live = |mode: Option<DeliberationMode>| {
        let mut s = persist::load(&start, &sure).expect("loads");
        assert!(
            matches!(s.deliberation(), DeliberationMode::Rule),
            "a loaded world is rule-only"
        );
        if let Some(m) = mode {
            s.set_deliberation(m);
        }
        s.advance_minutes(days * DAY).expect("advances");
        s
    };
    let stub = |pick: fn(usize) -> ModelAnswer| {
        let asked = Arc::new(AtomicUsize::new(0));
        let mode = DeliberationMode::Model(Box::new(Stub {
            pick,
            asked: Arc::clone(&asked),
        }));
        (mode, asked)
    };
    // Rules alone.
    let rule = live(None);
    assert!(
        !proposals(&rule).is_empty(),
        "a lean village proposes within three weeks"
    );
    assert!(rule.saved_answers().answers.is_empty());
    assert!(issues(&rule).iter().all(|w| !w.contains("chose by")));

    // A model that never answers, or names no listed move: rules choose on the same draws, and
    // every section but the kept moves rules made is the rule-only world's.
    for pick in [
        (|_| ModelAnswer::Unanswered) as fn(usize) -> ModelAnswer,
        |n| ModelAnswer::Move(n),
        |_| ModelAnswer::Refused,
    ] {
        let (mode, asked) = stub(pick);
        let mut fallen = live(Some(mode));
        assert!(asked.load(Ordering::Relaxed) > 0, "the model was asked");
        assert_eq!(proposals(&fallen), proposals(&rule));
        let saved = fallen.saved_answers().clone();
        assert!(!saved.answers.is_empty(), "the moves rules made are kept");
        for (_, sponsor, day, _) in proposals(&fallen) {
            let by = saved.chosen_by(civ_core::PermanentId::from_raw(sponsor).expect("id"), day);
            assert_eq!(by, Some(ChosenBy::RuleFallback));
        }
        let said = issues(&fallen);
        assert!(
            said.iter()
                .all(|w| w.ends_with("; chose by rule (the model did not answer)")),
            "{said:?}"
        );
        fallen.people_mut_for_tests().deliberation.answers = Default::default();
        assert!(sections(&fallen) == sections(&rule), "{:?}", pick(1));
    }

    // A model that makes the last move offered: it makes it, and the receipt says so.
    let (mode, asked) = stub(|n| ModelAnswer::Move(n - 1));
    let mut chosen = live(Some(mode));
    assert!(asked.load(Ordering::Relaxed) > 0);
    let saved = chosen.saved_answers().clone();
    assert!(
        saved
            .answers
            .iter()
            .all(|a| a.by == ChosenBy::Model && a.chosen.is_some()),
        "every answer accepted"
    );
    let made = proposals(&chosen);
    assert!(!made.is_empty());
    for (_, sponsor, day, _) in &made {
        let by = saved.chosen_by(civ_core::PermanentId::from_raw(*sponsor).expect("id"), *day);
        assert_eq!(by, Some(ChosenBy::Model));
    }
    let said = issues(&chosen);
    assert!(
        said.iter().all(|w| w.ends_with("; chose by the model")),
        "{said:?}"
    );
    assert_eq!(saved.providers.len(), 1);
    assert_eq!(saved.providers[0].kind, "stub");

    // Saved and loaded, the answers are kept exactly; and the start, loaded and replaying them
    // without the model, lives the same weeks to the same bytes.
    let end = persist::save(
        &mut chosen,
        &saves,
        commons_persist::SaveKind::Manual,
        "model",
    )
    .expect("saves")
    .path;
    let loaded = persist::load(&end, &sure).expect("loads");
    assert_eq!(loaded.saved_answers(), &saved);
    assert!(sections(&loaded) == sections(&chosen), "saved exactly");
    let replayed = live(Some(DeliberationMode::Recorded(saved.clone())));
    assert_eq!(proposals(&replayed), made);
    assert!(
        sections(&replayed) == sections(&chosen),
        "replay relives it"
    );

    // A model that makes none: nothing is proposed by those it answered.
    let (mode, _) = stub(|_| ModelAnswer::None);
    let none = live(Some(mode));
    assert!(
        none.saved_answers()
            .answers
            .iter()
            .all(|a| a.chosen.is_none())
    );
    assert!(proposals(&none).is_empty(), "nobody proposes");
}

/// A watch kept by turns, drawn by lot (backlog D; research 12-04 §1.3: a rotating household
/// watch; 09-14 §2.2: a rotating office's term is three to twelve months): passed, a lot draws the
/// watcher from those who stood for it, less anyone keeping a watch already, for a season; it
/// lapses at the season's end, told as their term ending. All of it saves and loads.
#[test]
fn a_watch_kept_by_turns_is_drawn_by_lot_for_a_season() {
    use civ_agents::polity::{IssueKind, StanceRecord};
    let mut sim = world_with(content(), 3);
    sim.advance_minutes(DAY).expect("advances");
    let now = sim.now();
    let policies = sim.rules().catalog.policies.clone();
    let at = |id: &str| policies.iter().position(|p| p.id == id).expect("in core") as u16;
    let (turns, plain) = (
        at("core:policy/zestfully_take_the_watch_by_turns"),
        at("core:policy/keep_watch"),
    );
    let tenure = policies[usize::from(turns)].tenure.expect("held by turns");
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
    assert!(adults.len() >= 4);
    let next = sim.ids().peek_next();
    let law = |id: u64, policy: u16, holder, stances: Vec<StanceRecord>| Law {
        id: civ_core::PermanentId::from_raw(next + 1_000_000 + id).expect("non-zero"),
        policy,
        kind: PolicyKind::KeepWatch,
        levy_share: 0.0,
        holder,
        relief_days: 0.0,
        sanction: Default::default(),
        hours: (0, 0),
        status: LawStatus::InForce,
        sponsor: adults[0].0,
        proposed: now,
        issue: IssueKind::Takings,
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
    // The first adult keeps the plain watch already; three stood for the watch by turns, they
    // among them: whatever the key, the lot draws one of the other two.
    let stances: Vec<StanceRecord> = adults[..3]
        .iter()
        .map(|&(person, household)| StanceRecord {
            person,
            household,
            stance: Stance::Support,
            gain: 0.0,
            regard: 0.0,
            opinion: 0.0,
            values: 0.0,
            view: 0.0,
            owed: 0.0,
        })
        .collect();
    let first = law(0, turns, None, stances);
    let first_id = first.id;
    {
        let p = &mut sim.people_mut_for_tests().polities[0];
        p.laws.push(law(1, plain, Some(adults[0].0), Vec::new()));
        p.laws.push(first);
    }
    sim.with_ctx_for_tests(|pop, ctx| pop.fill_term_for_tests(ctx, 0, first_id));
    let p = &sim.people().polities[0];
    let watcher = p
        .laws
        .iter()
        .find(|l| l.id == first_id)
        .and_then(|l| l.holder)
        .expect("the lot drew a watcher");
    assert!(
        adults[1..3].iter().any(|a| a.0 == watcher),
        "drawn from those who stood for it, not one keeping a watch already"
    );
    let term = p.term_of(first_id).expect("held for a term");
    assert_eq!((term.pool, tenure.term_days), (2, 90));
    assert_eq!(term.until, now.day_index() + 90);
    assert!(
        sim.people()
            .chronicle
            .iter()
            .any(|e| e.kind == ChronicleKind::LotDrawn
                && e.people == vec![watcher]
                && e.name.contains("for the office of watch by turns")),
        "the chronicle tells the lot"
    );
    saves_and_goes_on_alike(&mut sim, content(), DAY);
    // Past its season, the office lapses: their term ended.
    sim.advance_minutes(91 * DAY).expect("advances");
    let p = &sim.people().polities[0];
    let ended = p.laws.iter().find(|l| l.id == first_id).expect("kept");
    assert_eq!(ended.status, LawStatus::Lapsed);
    assert!(
        sim.people()
            .chronicle
            .iter()
            .any(|e| e.kind == ChronicleKind::LawLapsed
                && e.people == vec![watcher]
                && e.name.contains("no longer keeps watch")
                && e.name.ends_with("their term ended.")),
        "the chronicle tells the term's end"
    );
}

/// What an office-term test needs (M7a slice BN): a world a day on with its keeper laws set
/// aside, its adults in id order, and a maker of laws in force under a template.
struct Offices {
    sim: Sim,
    adults: Vec<(civ_core::PermanentId, civ_core::PermanentId)>,
    next: u64,
}

impl Offices {
    fn new() -> Offices {
        Offices::after(1)
    }

    /// As [`Offices::new`], `days` on rather than one.
    fn after(days: i64) -> Offices {
        let mut sim = world_with(content(), 3);
        sim.advance_minutes(days * DAY).expect("advances");
        let now = sim.now();
        let adult = sim.rules().people.family.independent_age;
        let mut adults: Vec<(civ_core::PermanentId, civ_core::PermanentId)> = sim
            .people()
            .people
            .iter()
            .map(|(_, p)| p)
            .filter(|p| p.age_years(now) >= adult)
            .map(|p| (p.id, p.household))
            .collect();
        adults.sort_unstable();
        assert!(adults.len() >= 4);
        for l in &mut sim.people_mut_for_tests().polities[0].laws {
            if l.status == LawStatus::InForce
                && matches!(l.kind, PolicyKind::KeepStore | PolicyKind::KeepRegister)
            {
                l.status = LawStatus::Lapsed;
            }
        }
        let next = sim.ids().peek_next();
        Offices { sim, adults, next }
    }

    fn at(&self, id: &str) -> u16 {
        let policies = &self.sim.rules().catalog.policies;
        policies.iter().position(|p| p.id == id).expect("in core") as u16
    }

    /// Law number `k` under `policy`, in force, naming `holder`, with `support` standing for it;
    /// its term not yet filled.
    fn push(
        &mut self,
        k: u64,
        policy: u16,
        holder: Option<civ_core::PermanentId>,
        support: &[(civ_core::PermanentId, civ_core::PermanentId)],
    ) -> civ_core::PermanentId {
        use civ_agents::polity::{IssueKind, StanceRecord};
        let now = self.sim.now();
        let kind = self.sim.rules().catalog.policies[usize::from(policy)].kind;
        let id = civ_core::PermanentId::from_raw(self.next + 1_000_000 + k).expect("non-zero");
        let stances = support
            .iter()
            .map(|&(person, household)| StanceRecord {
                person,
                household,
                stance: Stance::Support,
                gain: 0.0,
                regard: 0.0,
                opinion: 0.0,
                values: 0.0,
                view: 0.0,
                owed: 0.0,
            })
            .collect();
        let law = Law {
            id,
            policy,
            kind,
            levy_share: 0.0,
            holder,
            relief_days: 5.0,
            sanction: Default::default(),
            hours: (0, 0),
            status: LawStatus::InForce,
            sponsor: self.adults[0].0,
            proposed: now,
            issue: IssueKind::StoreUnkept,
            meets_day: now.day_index(),
            decided: Some(now),
            outcome: Some(Outcome::Passed),
            eligible: self.adults.len() as u32,
            stances,
            known: Vec::new(),
            compliance: Default::default(),
            watch: Default::default(),
            body: None,
            ends: None,
            agreement: None,
            target: None,
        };
        self.sim.people_mut_for_tests().polities[0].laws.push(law);
        id
    }

    fn fill(&mut self, id: civ_core::PermanentId) -> &Law {
        self.sim
            .with_ctx_for_tests(|pop, ctx| pop.fill_term_for_tests(ctx, 0, id));
        self.law(id)
    }

    fn law(&self, id: civ_core::PermanentId) -> &Law {
        let laws = &self.sim.people().polities[0].laws;
        laws.iter().find(|l| l.id == id).expect("kept")
    }

    /// Law `id` and its term taken away again.
    fn undo(&mut self, id: civ_core::PermanentId) {
        let p = &mut self.sim.people_mut_for_tests().polities[0];
        p.laws.retain(|l| l.id != id);
        p.terms.retain(|t| t.0 != id);
    }

    fn told(&self, kind: ChronicleKind) -> usize {
        let chronicle = &self.sim.people().chronicle;
        chronicle.iter().filter(|e| e.kind == kind).count()
    }
}

/// An office the gathering names for a term (M7a slice BN; the core's storekeeper for a year):
/// filling its term keeps the one named, draws no lot and tells none, and counts nobody in; the
/// term runs the template's days from the day it passed; the office is in force on the term's
/// last day before its end and lapses at the midnight its end day begins, told as their term
/// ending. An office whose template takes no tenure is held for no term.
#[test]
fn an_office_named_for_a_term_holds_until_the_midnight_its_term_ends() {
    let mut o = Offices::new();
    let year = o.at("core:policy/your_year_keeping_the_store");
    let plain = o.at("core:policy/keep_store");
    let tenure = o.sim.rules().catalog.policies[usize::from(year)]
        .tenure
        .expect("held for a year");
    assert_eq!(
        (tenure.term_days, tenure.renewable, tenure.by_lot),
        (360, true, false)
    );
    assert!(
        o.sim.rules().catalog.policies[usize::from(plain)]
            .tenure
            .is_none()
    );
    let today = o.sim.now().day_index();
    let keeper = o.adults[1].0;
    // A template with no tenure: no term.
    let stood = o.adults[..3].to_vec();
    let untimed = o.push(0, plain, Some(keeper), &stood);
    let l = o.fill(untimed);
    assert_eq!((l.status, l.holder), (LawStatus::InForce, Some(keeper)));
    assert_eq!(o.sim.people().polities[0].term_of(untimed), None);
    o.undo(untimed);
    // Named for a year: the one named keeps it, whoever stood for it, and no lot is told.
    let lots = o.told(ChronicleKind::LotDrawn);
    let stood = o.adults[2..4].to_vec();
    let id = o.push(1, year, Some(keeper), &stood);
    let l = o.fill(id);
    assert_eq!((l.status, l.holder), (LawStatus::InForce, Some(keeper)));
    let term = o.sim.people().polities[0].term_of(id).expect("a term");
    assert_eq!((term.until, term.pool), (today + 360, 0));
    assert_eq!(o.told(ChronicleKind::LotDrawn), lots, "no lot is told");
    saves_and_goes_on_alike(&mut o.sim, content(), DAY);
    // Its end brought near: in force the day before, lapsed from the midnight of its end day.
    let today = o.sim.now().day_index();
    o.sim.people_mut_for_tests().polities[0].set_term(
        id,
        civ_agents::polity::Term {
            until: today + 2,
            pool: 0,
        },
    );
    o.sim.advance_minutes(DAY).expect("advances");
    assert_eq!(o.sim.now().day_index(), today + 1);
    assert_eq!(o.law(id).status, LawStatus::InForce, "its last day");
    let lapses = o.told(ChronicleKind::LawLapsed);
    o.sim.advance_minutes(DAY).expect("advances");
    assert_eq!(o.sim.now().day_index(), today + 2);
    assert_eq!(o.law(id).status, LawStatus::Lapsed, "its end day");
    assert_eq!(
        o.law(id).holder,
        Some(keeper),
        "the law still names who held it"
    );
    assert_eq!(o.told(ChronicleKind::LawLapsed), lapses + 1);
    let told = o
        .sim
        .people()
        .chronicle
        .iter()
        .rev()
        .find(|e| e.kind == ChronicleKind::LawLapsed);
    let told = told.expect("told");
    assert_eq!(told.people, vec![keeper]);
    assert!(
        told.name.contains(" no longer keeps the common store at ")
            && told.name.ends_with(": their term ended."),
        "{}",
        told.name
    );
    assert!(o.sim.people().polities[0].keeper().is_none());
}

/// A lot's pool (M7a slice BN): only those who stood for it who are still in the world are
/// counted in and drawn; one who stood and is no longer anyone is neither. The bar on a holder
/// whose template does not renew holds while their term runs as well as once it has ended: a lot
/// under the same template (a second law, as a petition's would be) never draws the one holding
/// it again. And where the office asks for a technique, only those who know it are counted in and
/// drawn, and with none who do, the law lapses.
#[test]
fn a_lot_draws_only_the_living_who_stood_and_never_the_holder_whose_term_does_not_renew() {
    let mut o = Offices::new();
    let turns = o.at("core:policy/your_turn_keeping_the_store");
    let tenure = o.sim.rules().catalog.policies[usize::from(turns)]
        .tenure
        .expect("held by turns");
    assert!(tenure.by_lot && !tenure.renewable);
    let (a, b, c) = (o.adults[0], o.adults[1], o.adults[2]);
    // One who stood is no longer anyone (their id belongs to nobody in the world).
    let gone = civ_core::PermanentId::from_raw(o.next + 2_000_000).expect("non-zero");
    assert!(o.sim.people().person(gone).is_none());
    for k in 0..8 {
        let id = o.push(k, turns, None, &[a, (gone, a.1), b]);
        let l = o.fill(id);
        let holder = l.holder.expect("drawn");
        assert!(holder == a.0 || holder == b.0, "draw {k}: {holder:?}");
        let term = o.sim.people().polities[0].term_of(id).expect("a term");
        assert_eq!(term.pool, 2, "draw {k}: the one gone is not counted in");
        o.undo(id);
    }
    // `a` holds it by turns, their term running.
    let first = o.push(10, turns, None, &[a]);
    assert_eq!(o.fill(first).holder, Some(a.0));
    let today = o.sim.now().day_index();
    assert_eq!(
        o.sim.people().polities[0].term_of(first).map(|t| t.until),
        Some(today + i64::from(tenure.term_days))
    );
    let draws = |o: &mut Offices| {
        (11..19)
            .map(|k| {
                let id = o.push(k, turns, None, &[a, b]);
                let holder = o.fill(id).holder.expect("drawn");
                o.undo(id);
                holder
            })
            .collect::<Vec<_>>()
    };
    // While their term runs, a second lot never draws them: 0 of 8.
    let running = draws(&mut o);
    let again = running.iter().filter(|&&h| h == a.0).count();
    assert_eq!(
        again, 0,
        "while the term runs, the one holding it is not drawn again: {running:?}"
    );
    assert!(running.iter().all(|&h| h == b.0));
    // Once their term has ended, never either: 0 of 8.
    o.sim.people_mut_for_tests().polities[0].set_term(
        first,
        civ_agents::polity::Term {
            until: today,
            pool: 1,
        },
    );
    assert!(draws(&mut o).iter().all(|&h| h == b.0));
    o.undo(first);
    // An office that asks for writing, drawn by lot: only one who knows it is counted in.
    let register = o.at("core:policy/yourselves_write_down_the_sick");
    let writing = o
        .sim
        .rules()
        .catalog
        .techniques
        .iter()
        .position(|t| t.id == "core:technique/writing")
        .expect("writing");
    assert_eq!(
        o.sim.rules().catalog.policies[usize::from(register)].holder_needs,
        Some(writing)
    );
    o.sim
        .rules_mut_for_tests()
        .expect("rules not yet shared")
        .catalog
        .policies[usize::from(register)]
    .tenure = Some(civ_agents::polity::Tenure {
        term_days: 360,
        renewable: true,
        by_lot: true,
    });
    let now = o.sim.now();
    for (_, p) in o.sim.people_mut_for_tests().people.iter_mut() {
        if [a.0, b.0, c.0].contains(&p.id) {
            p.knows.retain(|t| usize::from(t.technique) != writing);
            if p.id == c.0 {
                p.come_to_know(writing, civ_agents::person::KnowSource::Observer, now);
            }
        }
    }
    for k in 20..28 {
        let id = o.push(k, register, None, &[a, b, c]);
        let l = o.fill(id);
        assert_eq!(
            (l.status, l.holder),
            (LawStatus::InForce, Some(c.0)),
            "draw {k}"
        );
        assert_eq!(
            o.sim.people().polities[0].term_of(id).map(|t| t.pool),
            Some(1)
        );
        o.undo(id);
    }
    let id = o.push(30, register, None, &[a, b]);
    let l = o.fill(id);
    assert_eq!((l.status, l.holder), (LawStatus::Lapsed, None));
    assert_eq!(o.sim.people().polities[0].term_of(id), None);
}

/// How an office held for a term lapses when its holder goes before it ends (M7a slice BN; ADR-0013
/// §2): the chronicle tells how they went, not that their term ended, even where they go on the day
/// before its end, so that both fall at the same midnight: they died, they left the valley (exiled
/// beyond the map), or they no longer live there (their household gone from the settlement). A
/// holder who dies with most of their term to run is told the same.
#[test]
fn a_holder_gone_before_their_term_ends_is_told_how_they_went() {
    let cases: [(&str, i64); 4] = [("died", 1), ("died", 360), ("exiled", 1), ("moved", 1)];
    for (how, left) in cases {
        let mut o = Offices::new();
        let year = o.at("core:policy/your_year_keeping_the_store");
        let keeper = o.adults[1].0;
        let id = o.push(1, year, Some(keeper), &[]);
        o.fill(id);
        let today = o.sim.now().day_index();
        o.sim.people_mut_for_tests().polities[0].set_term(
            id,
            civ_agents::polity::Term {
                until: today + left,
                pool: 0,
            },
        );
        match how {
            "died" => o
                .sim
                .with_ctx_for_tests(|pop, ctx| pop.die_for_tests(ctx, keeper)),
            "exiled" => o
                .sim
                .with_ctx_for_tests(|pop, ctx| pop.exile_for_tests(ctx, keeper)),
            _ => {
                let pop = o.sim.people_mut_for_tests();
                let home = pop.person(keeper).expect("alive").household;
                for (_, x) in pop.households.iter_mut() {
                    if x.id == home {
                        x.settlement = None;
                    }
                }
            }
        }
        assert_eq!(
            o.law(id).status,
            LawStatus::InForce,
            "{how}: until midnight"
        );
        let lapses = o.told(ChronicleKind::LawLapsed);
        o.sim.advance_minutes(DAY).expect("advances");
        assert_eq!(o.sim.now().day_index(), today + 1);
        assert_eq!(o.law(id).status, LawStatus::Lapsed, "{how}, {left}");
        let pop = o.sim.people();
        let told: Vec<_> = pop
            .chronicle
            .iter()
            .filter(|e| e.kind == ChronicleKind::LawLapsed && e.people == vec![keeper])
            .collect();
        assert_eq!(told.len(), 1, "{how}, {left}: {told:?}");
        assert_eq!(o.told(ChronicleKind::LawLapsed), lapses + 1);
        let words = match how {
            "died" => ": they died.",
            "exiled" => ": they left the valley.",
            _ => ": they no longer live there.",
        };
        assert!(
            told[0]
                .name
                .contains(" no longer keeps the common store at ")
                && told[0].name.ends_with(words)
                && !told[0].name.contains("their term ended"),
            "{how}, {left}: {}",
            told[0].name
        );
        match how {
            "died" => assert!(pop.person(keeper).is_none()),
            "exiled" => assert!(pop.person(keeper).is_none(), "beyond the map"),
            _ => {}
        }
    }
}

/// A plan's keeper whose term ends (M8c slice CU; M7a slice BN): the content gives no plan a
/// tenure (`tenure` is for an office that keeps the store, the watch or hears cases), so a term
/// reaches a plan only as set here. At its end the keeper lets go of it as one who died or left
/// does: the plan stays in force with nobody to give its parcels, and nothing is told, then or
/// after; its term stays recorded.
#[test]
fn a_plan_s_keeper_whose_term_ends_leaves_the_plan_in_force_with_no_keeper() {
    let mut o = Offices::new();
    let plan = o.at("core:policy/yours_to_plan_the_ground");
    let def = &o.sim.rules().catalog.policies[usize::from(plan)];
    assert_eq!(def.kind, PolicyKind::Plan);
    assert!(def.tenure.is_none(), "the content gives a plan no tenure");
    assert!(
        o.sim.people().polities[0]
            .laws
            .iter()
            .all(|l| l.kind != PolicyKind::Plan),
        "no plan of the world's own"
    );
    let keeper = o.adults[1].0;
    let id = o.push(1, plan, Some(keeper), &[]);
    let today = o.sim.now().day_index();
    o.sim.people_mut_for_tests().polities[0].set_term(
        id,
        civ_agents::polity::Term {
            until: today + 1,
            pool: 0,
        },
    );
    let told = o.sim.people().chronicle.len();
    o.sim.advance_minutes(DAY).expect("advances");
    assert_eq!(o.sim.now().day_index(), today + 1);
    let l = o.law(id);
    assert_eq!((l.status, l.holder), (LawStatus::InForce, None));
    assert!(o.sim.people().person(keeper).is_some(), "they live on");
    let since: Vec<_> = o.sim.people().chronicle[told..]
        .iter()
        .filter(|e| e.people.contains(&keeper) || e.kind == ChronicleKind::LawLapsed)
        .collect();
    assert!(since.is_empty(), "nothing is told: {since:?}");
    o.sim.advance_minutes(DAY).expect("advances");
    let l = o.law(id);
    assert_eq!((l.status, l.holder), (LawStatus::InForce, None));
    assert_eq!(
        o.sim.people().polities[0].term_of(id).map(|t| t.until),
        Some(today + 1)
    );
}

/// A store kept for a term is brought up to date at the midnight the term ends (M7a slice BN;
/// M4a slice Z's keeper): its food spoils at the keeper's roof's rate until that midnight and at
/// the open rate from then, rather than the whole time since it was last brought up to date at
/// the open rate.
#[test]
fn a_store_whose_keeper_s_term_ends_is_settled_under_their_roof_at_that_midnight() {
    // Early November: the households are under their roofs.
    let mut o = Offices::after(250);
    let now = o.sim.now();
    let pop = o.sim.people();
    let keeper = o
        .adults
        .iter()
        .find(|a| pop.household(a.1).is_some_and(|x| x.sheltered))
        .map(|a| a.0)
        .expect("someone lives under a roof");
    assert!(pop.polities[0].storehouse.is_none(), "no storehouse's roof");
    let grain = o.sim.rules().catalog.crops[0].good;
    let g = o.sim.rules().catalog.goods[grain].clone();
    assert!(g.sheltered_half_life_days > 0.0 && g.sheltered_half_life_days > g.half_life_days);
    let store = o.at("core:policy/common_store");
    let year = o.at("core:policy/your_year_keeping_the_store");
    for l in &mut o.sim.people_mut_for_tests().polities[0].laws {
        if l.status == LawStatus::InForce && l.kind == PolicyKind::CommonStore {
            l.status = LawStatus::Lapsed;
        }
    }
    o.push(0, store, None, &[]);
    let id = o.push(1, year, Some(keeper), &[]);
    let today = now.day_index();
    {
        let p = &mut o.sim.people_mut_for_tests().polities[0];
        p.set_term(
            id,
            civ_agents::polity::Term {
                until: today + 2,
                pool: 0,
            },
        );
        p.stores.resize(grain + 1, 0.0);
        p.stores[grain] = 1000.0;
        p.stores_at = now;
    }
    o.sim.advance_minutes(DAY).expect("advances");
    assert_eq!(o.law(id).status, LawStatus::InForce);
    let p = &o.sim.people().polities[0];
    let (kg, at) = (p.stores[grain], p.stores_at);
    o.sim.advance_minutes(DAY).expect("advances");
    assert_eq!(o.law(id).status, LawStatus::Lapsed);
    let p = &o.sim.people().polities[0];
    let midnight = civ_core::SimTime::from_minutes((today + 2) * civ_core::time::MINUTES_PER_DAY);
    assert_eq!(
        p.stores_at, midnight,
        "brought up to date as the term ended"
    );
    let days = (midnight.minutes() - at.minutes()) as f64 / 1440.0;
    let roofed = kg * 0.5f64.powf(days / g.sheltered_half_life_days);
    let open = kg * 0.5f64.powf(days / g.half_life_days);
    assert!(
        (p.stores[grain] - roofed).abs() < 1e-9,
        "{} kg held, {roofed} kg roofed, {open} kg open, over {days} days from {kg} kg",
        p.stores[grain]
    );
    assert!(roofed > open);
}

/// Gleaning's wage test (§8 17, fourth, slice 2; research 08-22 §1, §4: the Islip and Wymeswold
/// bylaws barred from gleaning anyone who could earn the harvest wage): where some of the
/// settlement's people of an age to work for hire glean the stubble, a household short of food
/// whose others glean it, who would glean what the able take, forecasts a gain from keeping the
/// stubble for those who cannot earn, and proposes it; a household whose own able gleaned
/// forecasts a loss. Once in force, one who could earn and knows of it weighs keeping it when they
/// would glean; a child does not. What each household gleaned, and by whom, is kept across a save
/// and load.
#[test]
fn a_household_short_of_food_proposes_to_keep_the_stubble_for_those_who_cannot_earn() {
    let mut sim = world_with(content(), 3);
    sim.advance_minutes(30 * DAY).expect("advances");
    assert!(
        sim.rules()
            .catalog
            .policies
            .iter()
            .any(|p| p.kind == PolicyKind::KeepStubble),
        "the core content keeps the stubble for those who cannot earn"
    );
    let settlement = sim.people().polities[0].settlement;
    let now = sim.now();
    let age = |sim: &Sim, p: civ_core::PermanentId| {
        sim.people()
            .person(p)
            .map_or(0.0, |q| q.age_years(sim.now()))
    };
    let mut households: Vec<(civ_core::PermanentId, Vec<civ_core::PermanentId>)> = sim
        .people()
        .households
        .iter()
        .filter(|(_, x)| x.settlement == Some(settlement) && !x.members.is_empty())
        .map(|(_, x)| (x.id, x.members.clone()))
        .collect();
    households.sort_unstable();
    assert!(households.len() >= 2, "the settlement has households");
    // The poor household has a child among its gleaners; the other's able gleaned.
    let poor = households
        .iter()
        .find(|(_, m)| m.iter().any(|&p| age(&sim, p) < 14.0))
        .unwrap_or(&households[0])
        .clone();
    let able = households
        .iter()
        .find(|(h, _)| *h != poor.0)
        .expect("another household")
        .clone();
    let day = now.day_index();
    let set_record = |sim: &mut Sim| {
        let day = sim.now().day_index();
        let mut g = vec![
            civ_agents::crime::GleanedBy {
                household: poor.0,
                able_kg: 0.0,
                other_kg: 100.0,
                day,
            },
            civ_agents::crime::GleanedBy {
                household: able.0,
                able_kg: 150.0,
                other_kg: 0.0,
                day,
            },
        ];
        g.sort_by_key(|x| x.household);
        sim.people_mut_for_tests().order.gleaners = g;
    };
    // The poor household holds grain for ninety days, its harvest further off: its food will not
    // last to it.
    let starve = |sim: &mut Sim| {
        let goods = sim.rules().catalog.goods.clone();
        let grain = sim.rules().catalog.crops[sim.rules().people.farm.crop].good;
        let per_day = sim.rules().people.household.daily_kcal_per_person;
        let pop = sim.people_mut_for_tests();
        let x = pop
            .households
            .iter_mut()
            .find(|(_, x)| x.id == poor.0)
            .expect("the poor household")
            .1;
        let kg = x.members.len() as f64 * per_day * 90.0 / goods[grain].kcal_per_kg;
        for (i, (held, g)) in x.stores.iter_mut().zip(&goods).enumerate() {
            if g.purpose == civ_agents::params::GoodUse::Food {
                *held = if i == grain { kg } else { 0.0 };
            }
        }
    };
    set_record(&mut sim);
    starve(&mut sim);
    let fc = sim.with_ctx_for_tests(|pop, ctx| pop.stubble_forecasts_for_tests(ctx, settlement));
    let of = |h: civ_core::PermanentId| fc.iter().find(|f| f.0 == h).copied();
    let p = of(poor.0).expect("the poor household weighs it");
    let a = of(able.0).expect("the able household weighs it");
    assert!(
        p.1 > 0.0 && p.2 == 0.0,
        "the poor gain what the able gleaned: {p:?}"
    );
    assert!(
        a.1 == 0.0 && a.2 > 0.0,
        "the able give up their gleaning: {a:?}"
    );
    // The gathering hears it from someone whose household would gain: review day by day until
    // it is proposed, each other proposal taken as passed (a village that has its common store
    // and its lending already).
    let mut proposed = None;
    for _ in 0..60 {
        {
            let now = sim.now();
            let pop = sim.people_mut_for_tests();
            for l in pop.polities[0].laws.iter_mut() {
                if l.status == LawStatus::Proposed && l.kind != PolicyKind::KeepStubble {
                    l.status = LawStatus::InForce;
                    l.decided = Some(now);
                    l.outcome = Some(Outcome::Passed);
                }
            }
            pop.polities[0].gathering = None;
        }
        set_record(&mut sim);
        starve(&mut sim);
        sim.with_ctx_for_tests(|pop, ctx| pop.review_for_tests(ctx, settlement));
        proposed = sim.people().polities[0]
            .laws
            .iter()
            .find(|l| l.kind == PolicyKind::KeepStubble)
            .map(|l| (l.id, l.sponsor));
        if proposed.is_some() {
            break;
        }
        sim.advance_minutes(DAY).expect("advances");
    }
    let (law, sponsor) = proposed.expect("someone proposed keeping the stubble");
    let sponsor_household = sim
        .people()
        .person(sponsor)
        .map(|p| p.household)
        .expect("the sponsor lives");
    assert_eq!(
        sponsor_household, poor.0,
        "the household short of food, whose others glean, proposes it"
    );
    // In force and known to all, as if passed.
    let everyone: Vec<_> = {
        let mut k: Vec<_> = sim
            .people()
            .people
            .iter()
            .map(|(_, p)| (p.id, day))
            .collect();
        k.sort_unstable();
        k
    };
    {
        let now = sim.now();
        let pop = sim.people_mut_for_tests();
        let l = pop.polities[0]
            .laws
            .iter_mut()
            .find(|l| l.id == law)
            .expect("the law");
        l.status = LawStatus::InForce;
        l.decided = Some(now);
        l.outcome = Some(Outcome::Passed);
        l.known = everyone.clone();
        pop.polities[0].gathering = None;
    }
    let grown = able
        .1
        .iter()
        .copied()
        .find(|&p| (14.0..=70.0).contains(&age(&sim, p)))
        .expect("the able household has someone of an age to work for hire");
    let points = sim.with_ctx_for_tests(|pop, ctx| pop.stubble_points_for_tests(ctx, grown));
    assert!(
        points.is_some(),
        "one who could earn weighs keeping the stubble when they would glean"
    );
    if let Some(&child) = poor.1.iter().find(|&&p| age(&sim, p) < 14.0) {
        let points = sim.with_ctx_for_tests(|pop, ctx| pop.stubble_points_for_tests(ctx, child));
        assert!(points.is_none(), "a child gleans as before");
    }
    // Only a law they know of weighs with them (the ninth coverage pass, §8 18, third).
    let set_known = |sim: &mut Sim, known: Vec<(civ_core::PermanentId, i64)>| {
        let l = sim.people_mut_for_tests().polities[0]
            .laws
            .iter_mut()
            .find(|l| l.id == law)
            .expect("the law");
        l.known = known;
    };
    set_known(
        &mut sim,
        everyone.iter().copied().filter(|k| k.0 != grown).collect(),
    );
    let points = sim.with_ctx_for_tests(|pop, ctx| pop.stubble_points_for_tests(ctx, grown));
    assert!(
        points.is_none(),
        "a law they have not heard of weighs nothing"
    );
    set_known(&mut sim, everyone.clone());
    // Need over the gathering's word: while someone of their household goes hungry, it weighs
    // nothing, and they glean as before.
    let energy: Vec<(civ_core::PermanentId, f32, civ_core::SimTime)> = {
        let now = sim.now();
        let params = sim.rules().people.clone();
        let pop = sim.people_mut_for_tests();
        let members = pop
            .person(grown)
            .and_then(|p| pop.household(p.household))
            .map(|x| x.members.clone())
            .expect("their household");
        let mut was = Vec::new();
        for (_, p) in pop
            .people
            .iter_mut()
            .filter(|(_, p)| members.contains(&p.id))
        {
            was.push((p.id, p.energy_kcal, p.needs_at));
            let reserve = population::reserve_kcal(p, now, &params);
            p.energy_kcal = -(0.8 * reserve) as f32;
            p.needs_at = now;
        }
        was
    };
    let points = sim.with_ctx_for_tests(|pop, ctx| pop.stubble_points_for_tests(ctx, grown));
    assert!(points.is_none(), "a household going hungry still gleans");
    {
        let pop = sim.people_mut_for_tests();
        for (_, p) in pop.people.iter_mut() {
            if let Some(&(_, kcal, at)) = energy.iter().find(|e| e.0 == p.id) {
                (p.energy_kcal, p.needs_at) = (kcal, at);
            }
        }
    }
    let points = sim.with_ctx_for_tests(|pop, ctx| pop.stubble_points_for_tests(ctx, grown));
    assert!(
        points.is_some(),
        "fed again, the law weighs with them again"
    );
    // A law no longer in force (here repealed) weighs nothing.
    let set_status = |sim: &mut Sim, status: LawStatus| {
        sim.people_mut_for_tests().polities[0]
            .laws
            .iter_mut()
            .find(|l| l.id == law)
            .expect("the law")
            .status = status;
    };
    set_status(&mut sim, LawStatus::Superseded);
    let points = sim.with_ctx_for_tests(|pop, ctx| pop.stubble_points_for_tests(ctx, grown));
    assert!(points.is_none(), "a law superseded weighs nothing");
    set_status(&mut sim, LawStatus::InForce);
    // The record of who gleaned is kept across a save and load.
    let kept = sim.people().order.gleaners.clone();
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = persist::save(
        &mut sim,
        &saves,
        commons_persist::SaveKind::Manual,
        "stubble",
    )
    .expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loaded.people().order.gleaners, kept);
}
