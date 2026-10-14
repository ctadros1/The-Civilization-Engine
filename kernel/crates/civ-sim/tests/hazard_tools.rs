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

fn weak_covering(sim: &mut Sim) -> (civ_core::PermanentId, civ_core::PermanentId) {
    use civ_agents::build::{self, Shape};
    let (owner, home) = sim
        .people()
        .households
        .iter()
        .map(|(_, h)| (h.id, h.home))
        .next()
        .expect("household");
    let def = &content().catalog.buildings[content()
        .catalog
        .building_index("core:building/hut")
        .expect("hut")];
    let spec = build::design_shape(
        def,
        &content().catalog.goods,
        Shape::Round { radius: 180 },
        home,
        None,
    )
    .expect("design");
    let building = sim
        .place_building_for_tests(owner, spec, civ_grammar::Stage::ALL.len() as u8)
        .expect("place");
    let installed = sim.now();
    let b = sim
        .land_mut_for_tests()
        .buildings
        .iter_mut()
        .find(|b| b.id == building)
        .expect("building");
    for c in &mut b.condition {
        c.quality = if civ_grammar::GroupKind::of_group(c.group)
            == Some(civ_grammar::GroupKind::Covering)
        {
            0.1
        } else {
            1.0
        };
        c.installed = installed;
        c.repaired = installed;
    }
    (owner, building)
}
fn covering_loss(sim: &Sim, building: civ_core::PermanentId) -> Vec<f32> {
    sim.land()
        .buildings
        .iter()
        .find(|b| b.id == building)
        .expect("building")
        .condition
        .iter()
        .filter(|c| {
            civ_grammar::GroupKind::of_group(c.group) == Some(civ_grammar::GroupKind::Covering)
        })
        .map(|c| c.loss)
        .collect()
}
#[test]
fn extra_load_checks_do_not_repeat_a_storm_and_its_marker_survives_a_save() {
    let mut sim = world();
    let (owner, building) = weak_covering(&mut sim);
    let pressure = 700.0;
    let speed = (2.0_f64 * pressure / 1.225).sqrt() as f32;
    let input = sim.send_storm(speed, 270.0).expect("storm");
    let losses = covering_loss(&sim, building);
    assert!(!losses.is_empty());
    assert!(losses.iter().all(|x| (*x - 0.4).abs() < 1e-6), "{losses:?}");
    assert_eq!(
        sim.people()
            .floods
            .storm_damages
            .iter()
            .filter(|d| d.building == building)
            .count(),
        1
    );
    let mut loaded = paired(&mut sim);
    same(&sim, &loaded);
    for s in [&mut sim, &mut loaded] {
        for _ in 0..10 {
            s.with_ctx_for_tests(|pop, ctx| pop.storm_for_tests(ctx, owner, pressure, input.id));
        }
        assert_eq!(covering_loss(s, building), losses);
        assert_eq!(
            s.people()
                .floods
                .storm_damages
                .iter()
                .filter(|d| d.building == building)
                .count(),
            1
        );
    }
    same(&sim, &loaded);
}
#[test]
fn a_later_natural_storm_is_not_attributed_to_an_earlier_observer() {
    let mut sim = world();
    weak_covering(&mut sim);
    let speed = (2.0_f64 * 700.0 / 1.225).sqrt() as f32;
    sim.send_storm(speed, 270.0).expect("observer storm");
    let midnight = DAY - sim.now().minutes().rem_euclid(DAY);
    sim.advance_minutes(midnight).expect("next day");
    let (owner, building) = weak_covering(&mut sim);
    sim.with_ctx_for_tests(|pop, ctx| pop.storm_for_tests(ctx, owner, 700.0, 0));
    let damage = sim
        .people()
        .floods
        .storm_damages
        .iter()
        .find(|d| d.building == building)
        .expect("damage");
    assert!(damage.origin.natural());
    let episode = sim
        .people()
        .floods
        .storm_episodes
        .iter()
        .find(|e| Some(e.id) == damage.episode)
        .expect("episode");
    assert!(episode.origin.natural());
    let bytes = civ_sim::frames::incidents::storm_response(&sim, episode.id).expect("page");
    let response =
        civ_schema::flatbuffers::root::<civ_schema::wire::Response>(&bytes).expect("response");
    let info = response.body_as_storm_info().expect("storm info");
    assert_eq!(info.row().expect("row").observer(), 0);
    assert!(info.cause_link().is_none());
    let loaded = paired(&mut sim);
    same(&sim, &loaded);
}

#[test]
fn one_storm_records_one_building_incident_for_several_covering_groups() {
    use civ_agents::build::{self, Shape};
    let mut sim = world();
    let (owner, home) = sim
        .people()
        .households
        .iter()
        .map(|(_, h)| (h.id, h.home))
        .next()
        .expect("household");
    let def = &content().catalog.buildings[content()
        .catalog
        .building_index("core:building/longhouse")
        .expect("longhouse")];
    let spec = build::design_shape(
        def,
        &content().catalog.goods,
        Shape::Bays {
            bays: 2,
            storeys: 1,
            lofts: 0,
        },
        home,
        None,
    )
    .expect("design");
    let building = sim
        .place_building_for_tests(owner, spec, civ_grammar::Stage::ALL.len() as u8)
        .expect("place");
    let b = sim
        .land_mut_for_tests()
        .buildings
        .iter_mut()
        .find(|b| b.id == building)
        .expect("building");
    for c in &mut b.condition {
        c.quality = if civ_grammar::GroupKind::of_group(c.group)
            == Some(civ_grammar::GroupKind::Covering)
        {
            0.1
        } else {
            1.0
        };
    }
    assert!(covering_loss(&sim, building).len() > 1);
    sim.send_storm((2.0_f64 * 700.0 / 1.225).sqrt() as f32, 270.0)
        .expect("storm");
    assert_eq!(
        sim.people()
            .floods
            .storm_damages
            .iter()
            .filter(|d| d.building == building)
            .count(),
        1
    );
    let checks = sim
        .land()
        .storm_checks
        .iter()
        .filter(|c| c.building == building)
        .count();
    assert_eq!(checks, covering_loss(&sim, building).len());
    let loaded = paired(&mut sim);
    same(&sim, &loaded);
}

#[test]
fn bad_hazard_links_origins_and_physical_markers_are_refused() {
    let mut sim = world();
    weak_covering(&mut sim);
    sim.send_storm((2.0_f64 * 700.0 / 1.225).sqrt() as f32, 270.0)
        .expect("storm");
    let flood = sim
        .raise_river(0, 0.001, 1)
        .expect("a minimum below natural flow")
        .id;
    let records = sim.people().floods.clone();
    let checks = sim.land().storm_checks.clone();
    let now = sim.now();
    let dir = tempfile::tempdir().expect("dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("saves");
    for case in 0..7 {
        sim.people_mut_for_tests().floods = records.clone();
        sim.land_mut_for_tests().storm_checks = checks.clone();
        match case {
            0 => sim.people_mut_for_tests().floods.storm_damages[0].episode = Some(999),
            1 => sim.people_mut_for_tests().floods.storm_episodes[0]
                .damages
                .clear(),
            2 => sim.people_mut_for_tests().floods.storm_damages[0].uplift_pa = f32::NAN,
            3 => {
                sim.people_mut_for_tests().floods.storm_episodes[0]
                    .origin
                    .observers = vec![flood]
            }
            4 => {
                sim.people_mut_for_tests().floods.storm_episodes[0].ended =
                    Some(now.plus_minutes(1))
            }
            5 => sim.land_mut_for_tests().storm_checks[0].observer = flood,
            _ => sim
                .land_mut_for_tests()
                .storm_checks
                .push(checks[0].clone()),
        }
        let saved = persist::save(
            &mut sim,
            &saves,
            commons_persist::SaveKind::Manual,
            "invalid hazard state",
        )
        .expect("writes deliberately bad test state");
        assert!(
            persist::load(&saved.path, content()).is_err(),
            "case {case}"
        );
    }
}

#[test]
fn schema_95_hazards_keep_damage_but_do_not_invent_provenance() {
    let mut sim = world();
    let (_, building) = weak_covering(&mut sim);
    sim.send_storm((2.0_f64 * 700.0 / 1.225).sqrt() as f32, 270.0)
        .expect("storm");
    let dir = tempfile::tempdir().expect("dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("saves");
    let saved = persist::save(
        &mut sim,
        &saves,
        commons_persist::SaveKind::Manual,
        "legacy provenance",
    )
    .expect("save");
    let reader =
        commons_persist::SnapshotReader::open_file(&saved.path, commons_persist::Limits::default())
            .expect("reader");
    let mut info = reader.info().clone();
    info.schema_version = persist::SCHEMA_V95;
    let legacy = commons_persist::publish_file(
        dir.path(),
        "legacy.tcesave",
        &info,
        &persist::encode_sections(&sim),
        commons_persist::DEFAULT_ZSTD_LEVEL,
    )
    .expect("legacy");
    let mut loaded = persist::load(&legacy.path, content()).expect("loads schema 95");
    assert_eq!(
        covering_loss(&sim, building),
        covering_loss(&loaded, building)
    );
    assert!(!loaded.people().floods.storm_episodes[0].origin.known);
    assert!(
        loaded.people().floods.storm_episodes[0]
            .origin
            .observers
            .is_empty()
    );
    let bytes = civ_sim::frames::incidents::storm_response(&loaded, 1).expect("page");
    let response =
        civ_schema::flatbuffers::root::<civ_schema::wire::Response>(&bytes).expect("response");
    assert!(
        response
            .body_as_storm_info()
            .expect("storm")
            .cause()
            .expect("cause")
            .contains("not recorded")
    );
    let resaved = paired(&mut loaded);
    same(&loaded, &resaved);
}
