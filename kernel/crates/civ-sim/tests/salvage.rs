//! What is thrown away (backlog C's first slice; ADR-0044 §1): a worn iron tool leaves half the
//! bar it took as scrap iron on its holder's heap, in proportion to the share worn, and a tool
//! that names no scrap leaves nothing; the heap's discards never decay, are not dung and lie with
//! the heap; carried to a tip they arrive whole and are recorded there; the destructor draws only
//! a tip's decaying share; and a world with discards on heaps and a tip saves and loads exactly.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::polity::{IssueKind, Law, LawStatus, Outcome as Decided, PolicyKind, Tip};
use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_sim::{NewWorld, Sim, persist};

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

/// A world of `seed` whose band has just settled, as the crime target's.
fn world_with(seed: u64) -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Tips".to_owned(),
            seed,
            preset_id: PRESET.to_owned(),
            size_cells: 512,
            band_size: 0,
            neighbours: Vec::new(),
            neighbours_known: false,
            regime_id: String::new(),
        },
        content(),
        [7; 16],
    )
    .expect("generates")
}

/// Saves `sim`, loads it, and checks every section of the loaded world is the same, and that the
/// two go on alike for `minutes`.
fn saves_and_goes_on_alike(sim: &mut Sim, minutes: i64) {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "tips").expect("saves");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
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

/// A law of `kind` in force in the first polity, every adult of its settlement knowing it, as the
/// crime target's: its id.
fn a_law(sim: &mut Sim, kind: PolicyKind) -> PermanentId {
    let policies = &sim.rules().catalog.policies;
    let policy = policies
        .iter()
        .position(|d| d.kind == kind)
        .expect("the core content has the template");
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
        kind,
        levy_share: 0.0,
        holder: None,
        relief_days: 0.0,
        sanction: Default::default(),
        hours: (0, 0),
        status: LawStatus::InForce,
        sponsor: adults[0],
        proposed: now,
        issue: IssueKind::HeapsNear,
        meets_day: now.day_index(),
        decided: Some(now),
        outcome: Some(Decided::Passed),
        eligible: adults.len() as u32,
        stances: Vec::new(),
        known: Vec::new(),
        compliance: Default::default(),
        watch: Default::default(),
        body: None,
        ends: None,
        agreement: None,
        target: None,
    };
    for &a in &adults {
        law.learn(a, now.day_index());
    }
    pop.polities[0].laws.push(law);
    id
}

/// The refuse law passed in the first polity, its tip opened as passing it opens one: the law and
/// the tip, if one opened.
fn the_law_passed(sim: &mut Sim) -> (PermanentId, Option<Tip>) {
    let law = a_law(sim, PolicyKind::RefuseToTip);
    sim.with_ctx_for_tests(|pop, ctx| pop.open_tip_for_tests(ctx, 0, law));
    let tip = sim.people().polities[0]
        .tips
        .iter()
        .find(|t| t.law == law)
        .cloned();
    (law, tip)
}

/// Households of the first polity's settlement with people, by id.
fn our_households(sim: &Sim) -> Vec<PermanentId> {
    let settlement = sim.people().polities[0].settlement;
    let mut out: Vec<PermanentId> = sim
        .people()
        .households
        .iter()
        .filter(|(_, x)| x.settlement == Some(settlement) && !x.members.is_empty())
        .map(|(_, x)| x.id)
        .collect();
    out.sort_unstable();
    out
}

/// The catalog index of good `id`.
fn good(sim: &Sim, id: &str) -> usize {
    sim.rules().catalog.good_index(id).expect("a core good")
}

/// Household `h`'s midden discards.
fn discards(sim: &Sim, h: PermanentId) -> Vec<(u16, f64)> {
    sim.people()
        .household(h)
        .expect("there")
        .midden
        .discards
        .clone()
}

/// Gives household `h` `n` of good `g`.
fn give(sim: &mut Sim, h: PermanentId, g: usize, n: f64) {
    let pop = sim.people_mut_for_tests();
    let (_, x) = pop
        .households
        .iter_mut()
        .find(|(_, x)| x.id == h)
        .expect("the household");
    x.stores.resize(x.stores.len().max(g + 1), 0.0);
    x.stores[g] = n;
}

#[test]
fn a_worn_iron_tool_leaves_its_scrap_on_the_heap_in_proportion_to_its_wear() {
    let mut sim = world_with(5);
    sim.advance_minutes(2 * DAY).expect("advances");
    let (axe, stone_axe, scrap) = (
        good(&sim, "core:good/iron_axe"),
        good(&sim, "core:good/axe"),
        good(&sim, "core:good/scrap_iron"),
    );
    let life = sim.rules().catalog.goods[axe]
        .tool
        .as_ref()
        .expect("a tool")
        .life_h;
    let h = our_households(&sim)[0];
    assert!(discards(&sim, h).is_empty(), "nothing thrown on it yet");
    give(&mut sim, h, axe, 1.0);
    give(&mut sim, h, stone_axe, 0.0);
    // Half its life worn: half a tool gone, and half its scrap on the heap.
    sim.with_ctx_for_tests(|pop, ctx| pop.wear_tools_for_tests(ctx, h, &[axe], life / 2.0));
    let x = sim.people().household(h).expect("there");
    assert!((x.stores[axe] - 0.5).abs() < 1e-12, "{}", x.stores[axe]);
    let d = discards(&sim, h);
    assert_eq!(d.len(), 1);
    assert_eq!(usize::from(d[0].0), scrap);
    assert!((d[0].1 - 0.3).abs() < 1e-12, "{}", d[0].1);
    // Worn through: the tool is gone and 0.6 kg lies on the heap in all; more wear leaves no more.
    sim.with_ctx_for_tests(|pop, ctx| pop.wear_tools_for_tests(ctx, h, &[axe], life));
    let x = sim.people().household(h).expect("there");
    assert_eq!(x.stores[axe], 0.0);
    let d = discards(&sim, h);
    assert!((d[0].1 - 0.6).abs() < 1e-12, "{}", d[0].1);
    // A stone axe names no scrap: worn through, it leaves nothing.
    let other = our_households(&sim)[1];
    give(&mut sim, other, stone_axe, 1.0);
    give(&mut sim, other, axe, 0.0);
    let stone_life = sim.rules().catalog.goods[stone_axe]
        .tool
        .as_ref()
        .expect("a tool")
        .life_h;
    sim.with_ctx_for_tests(|pop, ctx| {
        pop.wear_tools_for_tests(ctx, other, &[stone_axe], 2.0 * stone_life)
    });
    assert_eq!(
        sim.people().household(other).expect("there").stores[stone_axe],
        0.0
    );
    assert!(discards(&sim, other).is_empty());
    // The heap's discards are inert: a year of the heap decaying leaves them as they were.
    sim.advance_minutes(30 * DAY).expect("advances");
    let d = discards(&sim, h);
    assert!((d[0].1 - 0.6).abs() < 1e-12, "{}", d[0].1);
}

#[test]
fn discards_carried_to_a_tip_arrive_whole_and_the_world_saves_and_loads_exactly() {
    let mut sim = world_with(5);
    sim.advance_minutes(2 * DAY).expect("advances");
    let (_, tip) = the_law_passed(&mut sim);
    let tip = tip.expect("a place qualifies in this valley, or this test shows nothing");
    let (axe, sickle, scrap) = (
        good(&sim, "core:good/iron_axe"),
        good(&sim, "core:good/iron_sickle"),
        good(&sim, "core:good/scrap_iron"),
    );
    let households = our_households(&sim);
    let (h, other) = (households[0], households[1]);
    // Two worn tools on one heap, one on another.
    give(&mut sim, h, axe, 1.0);
    give(&mut sim, h, sickle, 1.0);
    give(&mut sim, other, axe, 1.0);
    sim.with_ctx_for_tests(|pop, ctx| {
        pop.wear_tools_for_tests(ctx, h, &[axe, sickle], 1.0e6);
        pop.wear_tools_for_tests(ctx, other, &[axe], 1.0e6);
    });
    let before = discards(&sim, h);
    assert_eq!(before, vec![(scrap as u16, 0.6 + 0.2)]);
    let carrier = sim.people().household(h).expect("there").members[0];
    sim.with_ctx_for_tests(|pop, ctx| pop.carry_midden_for_tests(ctx, carrier, tip.place));
    // The heap is empty of them; the tip holds them whole.
    assert!(discards(&sim, h).is_empty());
    let t = &sim.people().polities[0].tips[0];
    assert_eq!(t.discards, before, "what left the heap lies at the tip");
    // The destructor's draw takes only a tip's decaying share, never its discards (ADR-0044 §1):
    // its draw reads `kg` alone, so a tip of discards and nothing else has nothing to burn.
    // Saved with discards on a heap and at a tip, the world loads exactly and goes on alike.
    assert_eq!(discards(&sim, other).len(), 1);
    saves_and_goes_on_alike(&mut sim, DAY);
}
