//! Ideologies (plan §7 M4c slice AG, ADR-0016 §4; research 06-04 §1.4, §6.1): some founders bring
//! each, and a founding child may take up a parent's; at the hearth a holder speaks of it and a
//! listener takes it up by trust and fit, from whom the record says; a holder weighs the laws its
//! program names when the problem it explains is before the village, and a law so proposed keeps
//! its creed; all of it saves and loads exactly.

use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

use civ_agents::ideology::Holding;
use civ_agents::polity::PolicyKind;
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
            name: "Ideologies".to_owned(),
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
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "ideas").expect("saves");
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

fn ideology(sim: &Sim, id: &str) -> u16 {
    sim.rules()
        .catalog
        .ideologies
        .iter()
        .position(|d| d.id == id)
        .unwrap_or_else(|| panic!("{id} in core")) as u16
}

#[test]
fn founders_bring_ideologies_and_companions_take_them_up_from_each_other() {
    let mut sim = world_with(content(), 3);
    // Through the first midnight: everyone takes their start.
    sim.advance_minutes(DAY + 60).expect("advances");
    {
        let pop = sim.people();
        let ideas = &pop.ideologies;
        assert!(!ideas.held.is_empty(), "some founders bring one");
        assert!(ideas.held.len() < pop.people.len(), "a few, not everyone");
        for h in &ideas.held {
            let p = pop.person(h.holder).expect("here");
            match h.from {
                // Brought by one with no parents recorded.
                None => assert!(p.mother.is_none() && p.father.is_none(), "{h:?}"),
                // Or taken up from a parent who holds it.
                Some(q) => {
                    assert!(p.mother == Some(q) || p.father == Some(q), "{h:?}");
                    assert!(ideas.holds(q, h.ideology));
                }
            }
        }
    }
    // Holders speak of it at the hearth often, and a listener who trusts them takes it up
    // readily: by hand, to see the path in a month.
    let rules = sim.rules_mut_for_tests().expect("not yet shared");
    let content_values: Vec<(f64, f64)> = rules
        .catalog
        .ideologies
        .iter()
        .map(|d| (d.share, d.adopt))
        .collect();
    for d in &mut rules.catalog.ideologies {
        d.share = 0.05;
        d.adopt = 1.0;
    }
    sim.advance_minutes(30 * DAY).expect("advances");
    let pop = sim.people();
    let ideas = &pop.ideologies;
    assert!(
        ideas.told > 0 && ideas.taken > 0,
        "{} told, {} taken",
        ideas.told,
        ideas.taken
    );
    let learnt: Vec<&Holding> = ideas
        .held
        .iter()
        .filter(|h| {
            h.from.is_some_and(|q| {
                let p = pop.person(h.holder).expect("here");
                p.mother != Some(q) && p.father != Some(q)
            })
        })
        .collect();
    assert!(
        !learnt.is_empty(),
        "taken up from a companion at the hearth"
    );
    for h in &learnt {
        let q = h.from.expect("told");
        let (hh, qh) = (
            pop.person(h.holder).map(|p| p.household),
            pop.person(q).map(|p| p.household),
        );
        assert_ne!(hh, qh, "companions at the hearth are of other households");
    }
    // The content's own rates again, as a loaded world has them.
    let rules = sim.rules_mut_for_tests().expect("not yet shared");
    for (d, &(share, adopt)) in rules.catalog.ideologies.iter_mut().zip(&content_values) {
        (d.share, d.adopt) = (share, adopt);
    }
    saves_and_goes_on_alike(&mut sim, content(), DAY);
}

#[test]
fn a_holder_proposes_what_their_ideology_proposes_and_the_law_keeps_its_creed() {
    let mut sim = world_with(content(), 3);
    sim.advance_minutes(DAY + 60).expect("advances");
    let provision = ideology(&sim, "core:ideology/common_provision");
    // Common provision weighs heavily with its holders, and every adult holds it: by hand.
    let rules = sim.rules_mut_for_tests().expect("not yet shared");
    rules.catalog.ideologies[usize::from(provision)].w_program = 40.0;
    let now = sim.now();
    let adult = sim.rules().people.family.independent_age;
    let pop = sim.people_mut_for_tests();
    let adults: Vec<PermanentId> = pop
        .people
        .iter()
        .map(|(_, p)| p)
        .filter(|p| p.age_years(now) >= adult)
        .map(|p| p.id)
        .collect();
    for a in adults {
        pop.ideologies.take_up(Holding {
            holder: a,
            ideology: provision,
            since: now.day_index(),
            from: None,
        });
    }
    // With nobody's food in store, the problem it explains is before the village: weeks of
    // reviews.
    let goods = sim.rules().catalog.goods.clone();
    for (_, h) in sim.people_mut_for_tests().households.iter_mut() {
        for (kg, g) in h.stores.iter_mut().zip(&goods) {
            if g.purpose == civ_agents::params::GoodUse::Food {
                *kg = 0.0;
            }
        }
    }
    sim.advance_minutes(30 * DAY).expect("advances");
    let pop = sim.people();
    let laws = &pop.polities[0].laws;
    let store = laws
        .iter()
        .find(|l| l.kind == PolicyKind::CommonStore)
        .expect("a holder proposed a common store");
    assert_eq!(
        pop.ideologies.creed_of(store.id),
        Some(provision),
        "{store:?}"
    );
    let rules = sim.rules_mut_for_tests().expect("not yet shared");
    rules.catalog.ideologies[usize::from(provision)].w_program = 1.0;
    saves_and_goes_on_alike(&mut sim, content(), DAY);
}
