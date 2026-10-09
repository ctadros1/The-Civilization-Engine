//! Norms (plan §7 M4c slice AG, ADR-0016 §4; research 06-05): everyone holds their own state of
//! the norm that what the gathering decides binds, drawn by a key; what each believes of others
//! is moved only by what companions tell of their own households' last levy; and what someone
//! believes of others weighs in whether they pay. All of it saves and loads exactly.

use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

use civ_agents::polity::{Law, LawStatus, Outcome, PolicyKind};
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
            name: "Norms".to_owned(),
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
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "norms").expect("saves");
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

/// A common store at a fifth put in force by hand, known to everyone: what it asks is what is
/// tested, not how it passed.
fn a_store_in_force(sim: &mut Sim) {
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
    let mut known: Vec<_> = pop.people.iter().map(|(_, p)| (p.id, 0)).collect();
    known.sort_unstable();
    let sponsor = known[0].0;
    pop.polities[0].laws.push(Law {
        id,
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
        ends: None,
        body: None,
    });
}

#[test]
fn everyone_holds_the_norm_and_learns_what_households_did_from_companions() {
    let mut sim = world_with(content(), 3);
    // To the end of April: states taken on 1 April, before any levy.
    sim.advance_minutes(60 * DAY).expect("advances");
    let binds = sim
        .rules()
        .catalog
        .norms
        .iter()
        .position(|d| d.id == "core:norm/gathering_binds")
        .expect("in core") as u16;
    let prior = sim.rules().catalog.norms[usize::from(binds)].expect_prior as f32;
    {
        let pop = sim.people();
        let n = &pop.norms;
        assert_eq!(
            n.states.len(),
            pop.people.len(),
            "everyone here on the first of the month holds one"
        );
        assert!(n.states.iter().all(|s| s.norm == binds));
        assert!(
            n.states.iter().all(|s| (s.expect - prior).abs() < 1e-6),
            "nobody has been told anything yet"
        );
        let endorse: Vec<f32> = n.states.iter().map(|s| s.endorse).collect();
        let (lo, hi) = endorse
            .iter()
            .fold((1.0f32, 0.0f32), |(l, h), &e| (l.min(e), h.max(e)));
        assert!(hi - lo > 0.4, "held each their own way: {lo}..{hi}");
        assert!(n.states.iter().all(|s| (0.2..=0.9).contains(&s.threshold)));
        assert!(n.acts.is_empty() && n.told == 0);
    }
    a_store_in_force(&mut sim);
    // Through the harvest, its threshing and the talk after it.
    sim.advance_minutes(150 * DAY).expect("advances");
    let pop = sim.people();
    let n = &pop.norms;
    let c = pop.polities[0].laws[0].compliance;
    assert!(c.complied > 0, "{c:?}");
    assert!(!n.acts.is_empty(), "households' levies are theirs to tell");
    for a in &n.acts {
        assert!(a.paid + a.kept > 0 && pop.household(a.household).is_some());
    }
    assert!(
        n.told > 0 && n.taken > 0,
        "{} told, {} taken",
        n.told,
        n.taken
    );
    let heard: Vec<_> = n.states.iter().filter(|s| s.heard > 0).collect();
    assert!(!heard.is_empty());
    assert!(
        heard.iter().any(|s| (s.expect - prior).abs() > 1e-3),
        "what they were told moved what they believe"
    );
    saves_and_goes_on_alike(&mut sim, content(), 2 * DAY);
}

#[test]
fn those_who_believe_others_keep_back_pay_less_often() {
    // The same village twice, its store in force by hand; in one everyone believes every
    // household pays and holds the norm firmly, in the other nobody believes anyone pays and
    // nobody holds it.
    let share_paid = |endorse: f32, expect: f32| {
        let mut sim = world_with(content(), 3);
        sim.advance_minutes(120 * DAY).expect("advances");
        a_store_in_force(&mut sim);
        for s in &mut sim.people_mut_for_tests().norms.states {
            (s.endorse, s.expect) = (endorse, expect);
        }
        sim.advance_minutes(100 * DAY).expect("advances");
        let c = sim.people().polities[0].laws[0].compliance;
        assert!(c.complied + c.evaded >= 10, "enough threshed: {c:?}");
        f64::from(c.complied) / f64::from(c.complied + c.evaded)
    };
    let bound = share_paid(1.0, 1.0);
    let unbound = share_paid(0.0, 0.0);
    assert!(
        bound > unbound + 0.15,
        "paid {bound:.2} where the norm holds, {unbound:.2} where it does not"
    );
}
