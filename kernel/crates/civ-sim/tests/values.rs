//! Values (plan §7 M4c slice AG, ADR-0016 §4; research 06-04 §1.1): everyone holds each value
//! content names, drawn their own way; at a gathering, what a law does to what someone holds dear
//! weighs beside their household's lot, and the record says so; all of it saves and loads
//! exactly.

use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

use civ_agents::polity::{Gathering, IssueKind, Law, LawStatus, PolicyKind, Stance};
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
            name: "Values".to_owned(),
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

/// Saves `sim`, loads it, and checks every section of the loaded world is the same, and that the
/// two go on alike for `minutes`.
fn saves_and_goes_on_alike(sim: &mut Sim, content: &ContentRegistry, minutes: i64) {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "values").expect("saves");
    let mut loaded = persist::load(&saved.path, content).expect("loads");
    let same = |a: &Sim, b: &Sim| {
        let (a, b) = (persist::encode_sections(a), persist::encode_sections(b));
        assert_eq!(a.len(), b.len());
        let differ: Vec<String> = a
            .iter()
            .zip(&b)
            .filter(|(x, y)| x.bytes != y.bytes)
            .map(|(x, _)| x.tag.to_string())
            .collect();
        assert!(differ.is_empty(), "sections differ: {differ:?}");
    };
    same(sim, &loaded);
    sim.advance_minutes(minutes).expect("advances");
    loaded.advance_minutes(minutes).expect("advances");
    same(sim, &loaded);
}

fn value_index(sim: &Sim, id: &str) -> u16 {
    sim.rules()
        .catalog
        .values
        .iter()
        .position(|v| v.id == id)
        .unwrap_or_else(|| panic!("{id} in core")) as u16
}

#[test]
fn everyone_holds_each_value_their_own_way() {
    let mut sim = world_with(content(), 3);
    sim.advance_minutes(40 * DAY).expect("advances");
    let pop = sim.people();
    let defs = &sim.rules().catalog.values;
    assert!(defs.len() >= 3, "the core content names values");
    assert_eq!(
        pop.values.held.len(),
        pop.people.len() * defs.len(),
        "everyone here on the first of the month holds each"
    );
    for k in 0..defs.len() as u16 {
        let held: Vec<f32> = pop
            .values
            .held
            .iter()
            .filter(|h| h.value == k)
            .map(|h| h.v)
            .collect();
        assert!(held.iter().all(|v| v.abs() < 1.0));
        assert!(
            held.iter().any(|&v| v > 0.4) && held.iter().any(|&v| v < -0.4),
            "some hold it dearer than most and some less: {held:?}"
        );
    }
    // The common store bears on safety from want and harm.
    let security = value_index(&sim, "core:value/security");
    let store = sim
        .rules()
        .catalog
        .policies
        .iter()
        .find(|p| p.kind == PolicyKind::CommonStore)
        .expect("in core");
    assert!(store.bears.iter().any(|&(k, b)| k == security && b > 0.0));
    saves_and_goes_on_alike(&mut sim, content(), DAY);
}

#[test]
fn what_a_law_does_to_what_they_hold_dear_weighs_in_their_stance() {
    // A store proposed, nobody's household standing to gain or lose by it, before a village that
    // holds safety from want and harm dearer than most and a household's say over its own as
    // most do; then before the same village holding it less than most.
    let stances_with = |security_v: f32| {
        let mut sim = world_with(content(), 4);
        // To 10 April: everyone holds their values, and no first of the month comes before the
        // gathering.
        sim.advance_minutes(40 * DAY).expect("advances");
        let security = value_index(&sim, "core:value/security");
        let store = sim
            .rules()
            .catalog
            .policies
            .iter()
            .position(|p| p.kind == PolicyKind::CommonStore)
            .expect("in core") as u16;
        let now = sim.now();
        let id = PermanentId::from_raw(sim.ids().peek_next() + 1_000_000).expect("non-zero");
        let params = sim.rules().people.clone();
        let pop = sim.people_mut_for_tests();
        for h in &mut pop.values.held {
            h.v = if h.value == security { security_v } else { 0.0 };
        }
        // Nobody holds an ideology, so what they hold dear alone tilts them (step four).
        pop.ideologies.held.clear();
        // Talk has moved nobody from their household's lot.
        for p in &mut pop.opinion.positions {
            p.x = p.anchor;
        }
        let mut adults: Vec<PermanentId> = pop
            .people
            .iter()
            .map(|(_, p)| p)
            .filter(|p| p.age_years(now) >= params.family.independent_age)
            .map(|p| p.id)
            .collect();
        adults.sort_unstable();
        let sponsor = adults[0];
        let mut stakes: Vec<(PermanentId, f32)> =
            pop.households.iter().map(|(_, h)| (h.id, 0.0)).collect();
        stakes.sort_by_key(|s| s.0);
        let p = &mut pop.polities[0];
        assert!(p.laws.is_empty() && p.gathering.is_none());
        p.laws.push(Law {
            id,
            policy: store,
            kind: PolicyKind::CommonStore,
            levy_share: 0.1,
            holder: None,
            relief_days: 5.0,
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
            ends: None,
            agreement: None,
            body: None,
        });
        p.gathering = Some(Gathering {
            law: Some(id),
            day: now.day_index() + 1,
            stakes,
            present: Vec::new(),
            cases: Vec::new(),
        });
        pop.hear_of_gatherings_called(now, &params);
        sim.advance_minutes(2 * DAY).expect("advances");
        let law = sim.people().polities[0]
            .laws
            .iter()
            .find(|l| l.id == id)
            .expect("kept")
            .clone();
        assert!(law.outcome.is_some(), "the gathering sat: {law:?}");
        (sim, law, sponsor, params.polity.stance_margin)
    };
    let (mut sim, law, sponsor, margin) = stances_with(0.8);
    let others: Vec<_> = law.stances.iter().filter(|r| r.person != sponsor).collect();
    assert!(!others.is_empty());
    for r in &others {
        // Bears fully on it, at a point's weight: 0.8 points each.
        assert!((r.values - 0.8).abs() < 1e-3, "{r:?}");
        assert_eq!(r.stance, Stance::Support, "{r:?}");
        let why = civ_agents::polity::stance_words(r, margin, sponsor);
        assert!(
            why.contains("what they hold dear drew them toward it"),
            "{why}"
        );
    }
    saves_and_goes_on_alike(&mut sim, content(), DAY);
    let (_, law, sponsor, margin) = stances_with(-0.8);
    for r in law.stances.iter().filter(|r| r.person != sponsor) {
        assert!((r.values + 0.8).abs() < 1e-3, "{r:?}");
        assert_ne!(r.stance, Stance::Support, "{r:?}");
        let why = civ_agents::polity::stance_words(r, margin, sponsor);
        assert!(
            why.contains("what they hold dear turned them against it"),
            "{why}"
        );
    }
}
