//! Observer hazards keep their requested physical forcing through daily steps and saves.
use civ_content::ContentRegistry;
use civ_sim::{NewWorld, Sim, persist};
use std::{path::Path, sync::OnceLock};
const DAY: i64 = 1440;
fn content() -> &'static ContentRegistry {
    static C: OnceLock<ContentRegistry> = OnceLock::new();
    C.get_or_init(|| {
        civ_content::load(
            &civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("root"),
        )
        .registry
        .expect("content")
    })
}
fn world() -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Hazards".into(),
            seed: 2,
            preset_id: "core:worldgen/river_valley".into(),
            size_cells: 512,
            band_size: 30,
            neighbours: vec![],
            neighbours_known: true,
            regime_id: String::new(),
        },
        content(),
        [5; 16],
    )
    .expect("world")
}
fn paired(sim: &mut Sim) -> Sim {
    let d = tempfile::tempdir().expect("tempdir");
    let saves =
        commons_persist::SaveDir::create(d.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "hazard").expect("save");
    persist::load(&saved.path, content()).expect("load")
}
fn same(a: &Sim, b: &Sim) {
    for (a, b) in persist::encode_sections(a)
        .iter()
        .zip(persist::encode_sections(b))
    {
        assert_eq!(a.bytes, b.bytes, "section {}", a.tag);
    }
}
#[test]
fn raised_runoff_routes_downstream_holds_its_days_and_resumes_exactly() {
    let mut sim = world();
    let reach = sim
        .map()
        .reaches
        .iter()
        .position(|r| r.downstream.is_some())
        .expect("tributary");
    let downstream = sim.map().reaches[reach].downstream.expect("downstream") as usize;
    let natural = sim
        .land()
        .river_flows(sim.map(), &sim.rules().land, sim.now().day_index());
    let target = natural[reach] + 100.0;
    let reached = sim.raise_river(reach, target, 3).expect("raise");
    assert_eq!(sim.land().reach_discharges[reach], target);
    assert!((sim.land().reach_discharges[downstream] - natural[downstream] - 100.0).abs() < 0.001);
    assert_eq!(sim.land().river_forcings[0].influence, reached.id);
    let mut loaded = paired(&mut sim);
    same(&sim, &loaded);
    for _ in 0..2 {
        sim.advance_minutes(DAY).expect("advance");
        loaded.advance_minutes(DAY).expect("advance");
        assert!(sim.land().reach_discharges[reach] >= target);
        same(&sim, &loaded);
    }
    sim.advance_minutes(DAY).expect("expires");
    loaded.advance_minutes(DAY).expect("expires");
    assert!(sim.land().river_forcings.is_empty());
    assert_eq!(
        sim.land().reach_discharges,
        sim.land()
            .river_flows(sim.map(), &sim.rules().land, sim.now().day_index())
    );
    same(&sim, &loaded);
}
#[test]
fn a_forced_storm_changes_todays_wind_without_changing_the_natural_series() {
    let mut sim = world();
    let mut natural = paired(&mut sim);
    let latent = sim.land().weather.wind;
    sim.send_storm(40.0, 270.0).expect("storm");
    assert_eq!(sim.land().weather.today.wind_from_deg, 270.0);
    assert_eq!(sim.land().weather.today.wind_ms, 40.0);
    assert_eq!(sim.land().weather.today.gust_ms, 40.0);
    assert_eq!(sim.land().weather.wind, latent);
    let mut loaded = paired(&mut sim);
    same(&sim, &loaded);
    sim.advance_minutes(DAY).expect("advance");
    loaded.advance_minutes(DAY).expect("advance");
    natural.advance_minutes(DAY).expect("natural");
    same(&sim, &loaded);
    assert_eq!(sim.land().weather.today, natural.land().weather.today);
    assert_eq!(sim.land().weather.wind, natural.land().weather.wind);
}
#[test]
fn invalid_hazard_inputs_leave_the_snapshot_unchanged() {
    let mut sim = world();
    let before = persist::encode_sections(&sim);
    for value in [f32::NAN, f32::INFINITY, -1.0, 0.0] {
        assert!(sim.raise_river(0, value, 2).is_err());
        assert!(sim.send_storm(value, 0.0).is_err());
    }
    for direction in [f32::NAN, f32::INFINITY, -1.0, 360.0] {
        assert!(sim.send_storm(40.0, direction).is_err());
    }
    assert!(sim.raise_river(0, 100.0, 0).is_err());
    assert!(sim.raise_river(usize::MAX, 100.0, 2).is_err());
    for (a, b) in before.iter().zip(persist::encode_sections(&sim)) {
        assert_eq!(a.bytes, b.bytes, "section {}", a.tag);
    }
}

#[test]
fn malformed_river_snapshots_are_refused() {
    let mut sim = world();
    let n = sim.map().reaches.len();
    let dir = tempfile::tempdir().expect("tempdir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    for values in [vec![0.0; n - 1], vec![f32::NAN; n], vec![-1.0; n]] {
        sim.land_mut_for_tests().reach_discharges = values;
        let saved = persist::save(
            &mut sim,
            &saves,
            commons_persist::SaveKind::Manual,
            "malformed flow",
        )
        .expect("writes deliberately malformed test state");
        assert!(persist::load(&saved.path, content()).is_err());
    }
}
