//! Opinion (plan §7 M4c slice AG, ADR-0016 §4): every adult holds a position on each question
//! content names, anchored in what their household's own lot makes of it on the first of each
//! month and moved a little by what trusted companions say at the hearth; at a gathering, how far
//! talk has moved a member from their household's lot weighs in their stance, and the record says
//! so; all of it saves and loads exactly.

use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

use civ_agents::polity::{Gathering, IssueKind, Law, LawStatus, Outcome, PolicyKind, Stance};
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
            name: "Opinion".to_owned(),
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

fn questions(sim: &Sim) -> Vec<u16> {
    sim.rules()
        .catalog
        .policies
        .iter()
        .enumerate()
        .filter(|(_, d)| d.question.is_some())
        .map(|(k, _)| k as u16)
        .collect()
}

/// Saves `sim`, loads it, and checks every section of the loaded world is the same, and that the
/// two go on alike for `minutes`.
fn saves_and_goes_on_alike(sim: &mut Sim, content: &ContentRegistry, minutes: i64) {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "opinion").expect("saves");
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
fn positions_are_anchored_in_each_households_lot_and_moved_by_talk() {
    let mut sim = world_with(content(), 3);
    // From 1 March through two firsts of the month (1 April and 1 May) and a month of talk.
    sim.advance_minutes(65 * DAY).expect("advances");
    let questions = questions(&sim);
    assert!(questions.len() >= 2, "the core content asks questions");
    let adults = adults(&sim);
    let pop = sim.people();
    let op = &pop.opinion;
    // Every adult who was here on the first of May holds a position on each question.
    let holding = adults
        .iter()
        .filter(|&&a| questions.iter().all(|&k| op.position(a, k).is_some()))
        .count();
    assert!(
        holding * 10 >= adults.len() * 9,
        "{holding} of {} adults hold positions",
        adults.len()
    );
    for p in &op.positions {
        assert!(
            (0.0..=1.0).contains(&p.x) && (0.0..=1.0).contains(&p.anchor),
            "{p:?}"
        );
        assert!(questions.contains(&p.policy));
    }
    // Members of one household share its lot: their anchors agree.
    let mut by_home: Vec<(PermanentId, u16, f32)> = op
        .positions
        .iter()
        .filter_map(|p| {
            let h = pop.person(p.holder)?.household;
            Some((h, p.policy, p.anchor))
        })
        .collect();
    by_home.sort_by_key(|a| (a.0, a.1));
    assert!(
        by_home
            .windows(2)
            .filter(|w| (w[0].0, w[0].1) == (w[1].0, w[1].1))
            .all(|w| (w[0].2 - w[1].2).abs() < 1e-6),
        "one household, one lot"
    );
    // Companions said where they stood, and some of it was taken in; talk moves a little.
    assert!(
        op.told > 0 && op.taken > 0,
        "{} told, {} taken",
        op.told,
        op.taken
    );
    assert!(op.positions.iter().any(|p| p.heard > 0));
    let drift = op
        .positions
        .iter()
        .map(|p| f64::from((p.x - p.anchor).abs()))
        .fold(0.0, f64::max);
    assert!(drift < 0.5, "talk moves a little, not everything: {drift}");
    saves_and_goes_on_alike(&mut sim, content(), 2 * DAY);
}

#[test]
fn how_far_talk_moved_someone_weighs_in_their_stance_and_the_record_says_so() {
    let mut sim = world_with(content(), 4);
    // To 10 April: positions were anchored on 1 April, and no first of the month comes before
    // the gathering.
    sim.advance_minutes(40 * DAY).expect("advances");
    let store = sim
        .rules()
        .catalog
        .policies
        .iter()
        .position(|p| p.kind == PolicyKind::CommonStore)
        .expect("in core") as u16;
    let adults = adults(&sim);
    let now = sim.now();
    let id = PermanentId::from_raw(sim.ids().peek_next() + 1_000_000).expect("non-zero");
    let params = sim.rules().people.clone();
    let w_position = params.opinion.w_position as f32;
    let pop = sim.people_mut_for_tests();
    // Talk has drawn everyone toward a common store, well past what their household's lot makes
    // of it: by hand, as if a season of talk had done it.
    for p in pop
        .opinion
        .positions
        .iter_mut()
        .filter(|p| p.policy == store)
    {
        p.x = (p.anchor + 0.4).min(1.0);
    }
    // A store proposed, nobody's household standing to gain or lose by it, to sit tomorrow.
    let sponsor = adults[0];
    let mut stakes: Vec<(PermanentId, f32)> =
        pop.households.iter().map(|(_, h)| (h.id, 0.0)).collect();
    stakes.sort_by_key(|s| s.0);
    let p = &mut pop.polities[0];
    assert!(
        p.laws.is_empty() && p.gathering.is_none(),
        "nothing before it yet"
    );
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
    let pop = sim.people();
    let law = pop.polities[0]
        .laws
        .iter()
        .find(|l| l.id == id)
        .expect("kept");
    assert!(law.outcome.is_some(), "the gathering sat: {law:?}");
    let moved: Vec<_> = law
        .stances
        .iter()
        .filter(|r| r.person != sponsor && r.opinion > 0.1)
        .collect();
    assert!(
        !moved.is_empty(),
        "talk weighed with those who came: {law:?}"
    );
    for r in &moved {
        // A little more than was set: talk in the day before moved them on.
        assert!(r.opinion <= w_position * 0.45, "{r:?}");
        assert_eq!(
            r.stance,
            Stance::Support,
            "drawn toward it, at no cost: {r:?}"
        );
        let why = civ_agents::polity::stance_words(r, params.polity.stance_margin, sponsor);
        assert!(
            why.contains("talk at the hearth had drawn them toward it"),
            "{why}"
        );
    }
    assert_eq!(law.outcome, Some(Outcome::Passed));
    saves_and_goes_on_alike(&mut sim, content(), DAY);
}
