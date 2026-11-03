//! Observation must expose the actual world and leave its authoritative bytes unchanged.
use civ_sim::{
    NewWorld, Sim,
    observation::{self, Observation},
    persist,
};
use std::path::Path;

fn world() -> Sim {
    let root =
        civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("content");
    let content = civ_content::load(&root).registry.expect("loads");
    Sim::create_for_tests(
        &NewWorld {
            name: "Observation proof".into(),
            seed: 2,
            preset_id: "core:worldgen/river_valley".into(),
            size_cells: 256,
            band_size: 40,
            neighbours: vec![],
            neighbours_known: false,
            regime_id: String::new(),
        },
        &content,
        [3; 16],
    )
    .expect("world")
}

#[test]
fn actual_positions_terrain_and_atomic_frames_do_not_change_state() {
    let mut sim = world();
    sim.advance_minutes(2 * 1440).expect("lives");
    let before = persist::encode_sections(&sim);
    let dir = tempfile::tempdir().expect("directory");
    let mut publisher = Observation::in_directory(dir.path(), &sim);
    publisher.poll(&sim);
    let files: Vec<_> = std::fs::read_dir(dir.path())
        .expect("frames")
        .map(|p| p.expect("entry").path())
        .collect();
    assert_eq!(files.len(), 1);
    let bytes = std::fs::read(&files[0]).expect("frame");
    assert!(bytes.len() < 2_000_000);
    let frame: serde_json::Value = serde_json::from_slice(&bytes).expect("JSON");
    assert_eq!(frame["world_id"], sim.meta().world_id_hex());
    assert_eq!(frame["minute"], sim.now().minutes());
    assert_eq!(frame["totals"]["people"], sim.people().living());
    assert_eq!(frame["terrain"]["water"][0], sim.map().water[257]);
    let p = sim.people().people.iter().next().expect("person").1;
    let pos = p.position_at(sim.now().minutes() as f64);
    assert_eq!(frame["people"][0]["id"], p.id.get().to_string());
    assert!((frame["people"][0]["x"].as_f64().expect("x") - f64::from(pos.0)).abs() < 0.001);
    publisher.poll(&sim);
    assert_eq!(publisher.cost().0, 1, "rate limited");
    let after = persist::encode_sections(&sim);
    for (a, b) in before.iter().zip(&after) {
        assert_eq!(a.bytes, b.bytes, "section {}", a.index);
    }
    drop(publisher);
    assert!(
        !files[0].exists(),
        "completed publisher no longer looks live"
    );
}

#[test]
fn observation_does_not_change_future_events_or_advance_chunking() {
    let mut a = world();
    let root =
        civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("content");
    let content = civ_content::load(&root).registry.expect("loads");
    let dir = tempfile::tempdir().expect("directory");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("saves");
    let saved = persist::save(
        &mut a,
        &saves,
        commons_persist::SaveKind::Manual,
        "same start",
    )
    .expect("save");
    let mut b = persist::load(&saved.path, &content).expect("load");
    a.set_speed(civ_sim::SPEED_MAX).expect("speed");
    b.set_speed(civ_sim::SPEED_MAX).expect("speed");
    a.advance_to_midnight().expect("midnight");
    b.advance_to_midnight().expect("midnight");
    a.advance_minutes(30 * 1440).expect("lives");
    for _ in 0..30 {
        let _ = observation::snapshot(&b);
        b.advance_minutes(1440).expect("lives");
    }
    for (a, b) in persist::encode_sections(&a)
        .iter()
        .zip(persist::encode_sections(&b))
    {
        assert_eq!(
            a.bytes, b.bytes,
            "observed world diverged in section {}",
            a.index
        );
    }
}

#[test]
fn disabled_and_failed_publishers_do_not_stop_the_world() {
    let sim = world();
    let mut disabled = Observation::default();
    disabled.poll(&sim);
    assert_eq!(disabled.cost().0, 0);
    let file = tempfile::NamedTempFile::new().expect("file");
    let mut broken = Observation::in_directory(file.path(), &sim);
    broken.poll(&sim);
    assert!(!broken.enabled());
    assert_eq!(broken.cost().0, 0);
}
