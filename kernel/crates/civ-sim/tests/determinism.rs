//! A world lives the same life every time from the same state (plan §4.4: the kernel is
//! deterministic): loaded twice from one save and run for the same time, it saves to the same
//! bytes, section by section.

use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

use civ_content::ContentRegistry;
use civ_sim::persist::{self, SECTION_META};
use civ_sim::{NewWorld, Sim};
use commons_persist::{SaveDir, SaveKind};

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

fn new_world(seed: u64) -> Sim {
    Sim::create(
        &NewWorld {
            name: "Twice".to_owned(),
            seed,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 512,
            band_size: 0,
            neighbours: Vec::new(),
            regime_id: String::new(),
        },
        content(),
        &mut |_| {},
        &AtomicBool::new(false),
    )
    .expect("generates")
}

#[test]
fn a_world_lives_the_same_life_from_the_same_save() {
    // Two worlds made from one seed differ (each draws its own identity, which orders the
    // events of an instant); one world, loaded twice from the same save, does not.
    let mut sim = new_world(3);
    sim.advance_minutes(2 * 24 * 60).expect("advances");
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves = SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("save dir");
    let saved = persist::save(&mut sim, &saves, SaveKind::Manual, "twice").expect("saves");
    let lived = || {
        let mut sim = persist::load(&saved.path, content()).expect("loads");
        sim.advance_minutes(30 * 24 * 60).expect("advances");
        persist::encode_sections(&sim)
    };
    let (a, b) = (lived(), lived());
    assert_eq!(a.len(), b.len());
    let differ: Vec<String> = a
        .iter()
        .zip(&b)
        .filter(|(x, y)| x.tag != SECTION_META && x.bytes != y.bytes)
        .map(|(x, _)| format!("{} {}", x.tag, x.index))
        .collect();
    assert!(differ.is_empty(), "sections differ: {differ:?}");
}
