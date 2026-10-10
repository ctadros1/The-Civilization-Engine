//! Word of mouth and grievances (ADR-0016 §2–§3; plan §7 M4c slice AE): a gathering is heard of
//! by contact and only those who heard come; a household short of food that knows of an empty
//! common store holds it against the gathering, relief redresses it, and a grievance held keenly
//! is told; all of it saves and loads exactly.

use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

use civ_agents::polity::{Law, LawStatus, Outcome, PolicyKind};
use civ_agents::word::{Blamed, ClaimKind, Grieved, Wrong};
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

fn world_with(content: &ContentRegistry, seed: u64) -> Sim {
    Sim::create(
        &NewWorld {
            name: "Word".to_owned(),
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

/// Everyone old enough to sit at a gathering, in id order.
fn adults(sim: &Sim) -> Vec<PermanentId> {
    let (now, adult) = (sim.now(), sim.rules().people.family.independent_age);
    let mut v: Vec<_> = sim
        .people()
        .people
        .iter()
        .map(|(_, p)| p)
        .filter(|p| p.age_years(now) >= adult)
        .map(|p| p.id)
        .collect();
    v.sort_unstable();
    v
}

/// A common store put in force by hand in the first polity, empty, known to `known`: what its
/// members make of it is what is tested. Returns the law's id.
fn a_store(sim: &mut Sim, known: &[PermanentId]) -> PermanentId {
    let now = sim.now();
    let policy = sim
        .rules()
        .catalog
        .policies
        .iter()
        .position(|p| p.kind == PolicyKind::CommonStore)
        .expect("the core content has a common store") as u16;
    let id = PermanentId::from_raw(sim.ids().peek_next() + 1_000_000).expect("non-zero");
    let pop = sim.people_mut_for_tests();
    let p = &mut pop.polities[0];
    assert!(p.laws.is_empty(), "the village has no law of its own yet");
    p.laws.push(Law {
        id,
        policy,
        kind: PolicyKind::CommonStore,
        levy_share: 0.1,
        holder: None,
        relief_days: 5.0,
        sanction: Default::default(),
        hours: (0, 0),
        status: LawStatus::InForce,
        sponsor: known[0],
        proposed: now,
        issue: civ_agents::polity::IssueKind::FoodShort,
        meets_day: now.day_index(),
        decided: Some(now),
        outcome: Some(Outcome::Passed),
        eligible: 0,
        stances: Vec::new(),
        known: known.iter().map(|&k| (k, 0)).collect(),
        compliance: Default::default(),
        watch: Default::default(),
        body: None,
        ends: None,
        agreement: None,
    });
    assert!(p.stores.iter().all(|kg| *kg == 0.0), "its store is empty");
    id
}

/// Saves `sim`, loads it, and checks every section of the loaded world is the same, and that the
/// two go on alike for `minutes`.
fn saves_and_goes_on_alike(sim: &mut Sim, content: &ContentRegistry, minutes: i64) {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "word").expect("saves");
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

/// The inspector shows the first holder's grievance in the kernel's words (wire 1.34).
fn inspector_says_it(sim: &Sim) {
    let holder = sim.people().word.grievances[0].holder;
    let payload = civ_sim::frames::people::person_response(sim, holder.get(), 0).expect("alive");
    let info = flatbuffers::root::<wire::Response>(&payload)
        .expect("a response")
        .body_as_person_info()
        .expect("a person");
    let lines = info.grievances().expect("their grievances");
    assert!(!lines.is_empty());
    let g = lines.get(0);
    assert_eq!(g.issue(), Grieved::Subsistence.code());
    assert_eq!(g.blamed(), Some("the gathering"));
    assert_eq!(g.over(), Some("food when their household was short"));
    assert_eq!(
        g.reason(),
        Some("the common store was empty when their household was short")
    );
    assert!(g.activation() > 0.0 && g.unresolved_days() > 0.0);
}

#[test]
fn only_those_who_heard_a_gathering_is_called_come_to_it() {
    // A village whose food will not last, with the deliberators' choice made all but certain (a
    // low temperature) so that a gathering is called: who hears of it is its own.
    let mut sure = content().clone();
    sure.people.params.polity.temperature = 0.05;
    let mut sim = world_with(&sure, 3);
    sim.advance_minutes(95 * DAY).expect("advances");
    empty_food(&mut sim);
    let (mut met, mut told) = (0, 0);
    for _ in 0..21 * 24 {
        sim.advance_minutes(60).expect("advances");
        let pop = sim.people();
        for p in &pop.polities {
            let Some(g) = &p.gathering else { continue };
            let claim = pop
                .word
                .gathering(p.settlement, g.day)
                .expect("a gathering called is a claim");
            if let Some(law) = g.law.and_then(|id| p.laws.iter().find(|l| l.id == id)) {
                assert!(pop.word.has_heard(law.sponsor, claim), "its sponsor knows");
            }
            for &m in &g.present {
                assert!(pop.word.has_heard(m, claim), "{m:?} came unheard");
            }
            met += g.present.len();
            told += pop
                .word
                .heard
                .iter()
                .filter(|h| h.claim == claim && h.from.is_some())
                .count();
        }
        if met > 0 && told > 0 {
            break;
        }
    }
    assert!(met > 0, "a gathering met");
    assert!(told > 0, "word of it was passed on");
    // The inspector says who told whom of it, in the kernel's words (wire 1.34).
    let pop = sim.people();
    let teller = pop
        .word
        .heard
        .iter()
        .find(|h| h.from.is_some())
        .expect("told");
    let payload =
        civ_sim::frames::people::person_response(&sim, teller.holder.get(), 0).expect("alive");
    let info = flatbuffers::root::<wire::Response>(&payload)
        .expect("a response")
        .body_as_person_info()
        .expect("a person");
    let line = info
        .heard()
        .expect("what they heard")
        .iter()
        .find(|l| l.from() == teller.from.map_or(0, PermanentId::get))
        .expect("the telling");
    assert_eq!(line.kind(), ClaimKind::Gathering.code());
    let what = line.what().expect("words");
    assert!(what.starts_with("a gathering meets on "), "{what}");
    assert!(line.from_name().is_some_and(|n| !n.is_empty()));
    let word = &sim.people().word;
    assert!(
        word.heard
            .windows(2)
            .all(|w| (w[0].holder, w[0].claim) < (w[1].holder, w[1].claim))
    );
    saves_and_goes_on_alike(&mut sim, &sure, 2 * DAY);
}

#[test]
fn an_empty_store_is_held_against_the_gathering_and_relief_redresses_it() {
    let mut sim = world_with(content(), 4);
    sim.advance_minutes(120 * DAY).expect("advances");
    // Everyone knows the store's law but the household with the lowest id until word of it
    // reaches them: without the expectation its terms give, the same harm makes no grievance
    // (ADR-0016 §2).
    let first_home = sim
        .people()
        .households
        .iter()
        .map(|(_, h)| h)
        .filter(|h| !h.members.is_empty())
        .map(|h| h.id)
        .min()
        .expect("a household");
    let unaware: Vec<PermanentId> = sim
        .people()
        .households
        .iter()
        .find(|(_, h)| h.id == first_home)
        .map(|(_, h)| h.members.clone())
        .unwrap_or_default();
    let known: Vec<PermanentId> = adults(&sim)
        .into_iter()
        .filter(|a| !unaware.contains(a))
        .collect();
    let law = a_store(&mut sim, &known);
    empty_food(&mut sim);
    sim.advance_minutes(2 * DAY).expect("advances");
    inspector_says_it(&sim);
    let pop = sim.people();
    let (polity, store) = (pop.polities[0].id, &pop.polities[0].laws[0]);
    let held = &pop.word.grievances;
    assert!(
        !held.is_empty(),
        "the short hold the empty store against it"
    );
    assert!(held.windows(2).all(|w| w[0].holder <= w[1].holder));
    for g in held {
        assert_eq!(g.issue, Grieved::Subsistence, "{g:?}");
        assert_eq!(g.blamed, Blamed::Body(polity), "no keeper: the gathering");
        assert_eq!((g.law, g.wrong), (law, Wrong::StoreEmpty), "{g:?}");
        assert!(store.knows(g.holder), "only those who know its terms");
        assert!(g.unresolved_days > 0.0 && g.unresolved_days <= 5.0, "{g:?}");
        assert!(g.activation > 0.0 && g.activation <= 1.0, "{g:?}");
    }
    let unresolved = |sim: &Sim| -> f64 {
        sim.people()
            .word
            .grievances
            .iter()
            .filter(|g| g.law == law)
            .map(|g| f64::from(g.unresolved_days))
            .sum()
    };
    let before = unresolved(&sim);
    // Grain comes to the store: it answers asks, and what it gives redresses the grievances.
    let grain = sim.rules().catalog.crops[0].good;
    let now = sim.now();
    let p = &mut sim.people_mut_for_tests().polities[0];
    p.stores.resize(p.stores.len().max(grain + 1), 0.0);
    p.stores[grain] = 5000.0;
    p.stores_at = now;
    sim.advance_minutes(10 * DAY).expect("advances");
    let c = sim.people().polities[0].laws[0].compliance;
    assert!(
        c.relieved > 0 && c.relief_kg > 0.0,
        "the store answered asks: {c:?}"
    );
    let after = unresolved(&sim);
    assert!(after < before, "relief redressed them: {before} → {after}");
    // A grievance held keenly was told to a companion.
    let word = &sim.people().word;
    let told = word.heard.iter().any(|h| {
        h.from.is_some()
            && word
                .claim(h.claim)
                .is_some_and(|c| c.kind == ClaimKind::Grievance)
    });
    assert!(told, "a grievance was told at the hearth");
    // What anyone who died or left held went with them.
    let pop = sim.people();
    assert!(
        pop.word
            .grievances
            .iter()
            .all(|g| pop.person(g.holder).is_some())
            && pop
                .word
                .heard
                .iter()
                .all(|h| pop.person(h.holder).is_some())
    );
    saves_and_goes_on_alike(&mut sim, content(), 2 * DAY);
}

#[test]
fn a_levy_is_held_against_the_gathering_in_a_lean_year_and_not_in_an_ordinary_one() {
    for lean in [false, true] {
        let mut sim = world_with(content(), 3);
        sim.advance_minutes(120 * DAY).expect("advances");
        let known = adults(&sim);
        let law = a_store(&mut sim, &known);
        // A heavy levy, so that some who can pay it are left short of a year's food; and, for the
        // lean year, a hungry spell before the harvest (its food runs short).
        sim.people_mut_for_tests().polities[0].laws[0].levy_share = 0.4;
        if lean {
            empty_food(&mut sim);
        }
        // Through the harvest and its threshing.
        sim.advance_minutes(100 * DAY).expect("advances");
        let pop = sim.people();
        let c = pop.polities[0].laws[0].compliance;
        assert!(c.complied > 0, "the threshers paid: {c:?}");
        let settlement = pop.polities[0].settlement;
        let ran_short = pop.chronicle.iter().any(|e| {
            e.kind == civ_agents::ChronicleKind::FoodRanShort && e.settlement == Some(settlement)
        });
        let polity = pop.polities[0].id;
        let levied: Vec<_> = pop
            .word
            .grievances
            .iter()
            .filter(|g| g.wrong == Wrong::LeanLevy)
            .collect();
        if lean {
            assert!(ran_short, "the hungry spell is a lean year");
            assert!(
                !levied.is_empty(),
                "a levy in a lean year left someone short: {c:?}"
            );
        }
        if !ran_short {
            assert!(
                levied.is_empty(),
                "an ordinary year's levy is the custom: {levied:?}"
            );
        }
        for g in levied {
            assert_eq!(g.issue, Grieved::Extraction, "{g:?}");
            assert_eq!((g.blamed, g.law), (Blamed::Body(polity), law), "{g:?}");
            assert!(g.harm_days > 0.0 && g.harm_days <= 0.4 * 365.0, "{g:?}");
            assert!(pop.polities[0].laws[0].knows(g.holder));
        }
        if lean {
            saves_and_goes_on_alike(&mut sim, content(), DAY);
        }
    }
}
