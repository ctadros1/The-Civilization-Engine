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

#[test]
fn flood_floor_loss_is_daily_and_extra_structural_checks_do_not_spoil_again() {
    let mut sim = world();
    let (owner, building) = weak_covering(&mut sim);
    for c in &mut sim
        .land_mut_for_tests()
        .buildings
        .iter_mut()
        .find(|b| b.id == building)
        .expect("building")
        .condition
    {
        c.quality = 1.0;
    }
    let now = sim.now();
    let grain = content()
        .catalog
        .goods
        .iter()
        .position(|g| g.kcal_per_kg > 0.0 && g.sheltered_half_life_days > 0.0)
        .expect("food");
    let cell = {
        let home = sim.people().household(owner).expect("household").home;
        (home.1 / sim.map().cell_size_m).floor() as usize * sim.map().width as usize
            + (home.0 / sim.map().cell_size_m).floor() as usize
    };
    {
        let hh = sim
            .people_mut_for_tests()
            .households
            .iter_mut()
            .find(|(_, h)| h.id == owner)
            .map(|(_, h)| h)
            .expect("household");
        hh.stores.fill(0.0);
        hh.stores[grain] = 100.0;
        hh.stores_at = now;
    }
    for _ in 0..3 {
        sim.land_mut_for_tests().flood_depths[cell] = 0.5;
        sim.with_ctx_for_tests(|p, c| p.storm_for_tests(c, owner, 0.0, 0));
    }
    assert_eq!(
        sim.people().household(owner).expect("household").stores[grain],
        100.0
    );
    sim.with_ctx_for_tests(|p, c| p.flood_loads_day_for_tests(c));
    let after = sim.people().household(owner).expect("household").stores[grain];
    assert!((after - 87.5).abs() < 1e-6, "{after}");
    sim.with_ctx_for_tests(|p, c| p.storm_for_tests(c, owner, 0.0, 0));
    assert_eq!(
        sim.people().household(owner).expect("household").stores[grain],
        after
    );
    assert!(
        sim.land()
            .buildings
            .iter()
            .find(|b| b.id == building)
            .expect("building")
            .standing()
    );
}

#[test]
fn repeated_river_commands_do_not_add_wet_days_before_midnight() {
    let mut sim = world();
    let (_, building) = weak_covering(&mut sim);
    let reach = sim
        .map()
        .reaches
        .iter()
        .position(|r| r.discharge_m3s > 2.0)
        .expect("river");
    let before = sim
        .land()
        .buildings
        .iter()
        .find(|b| b.id == building)
        .expect("building")
        .days_wet;
    for _ in 0..3 {
        sim.raise_river(reach, 100_000.0, 2).expect("raise");
        assert_eq!(
            sim.land()
                .buildings
                .iter()
                .find(|b| b.id == building)
                .expect("building")
                .days_wet,
            before
        );
    }
    let mut loaded = paired(&mut sim);
    assert_eq!(sim.land().flood_depths, loaded.land().flood_depths);
    same(&sim, &loaded);
    sim.advance_minutes(DAY).expect("advance");
    loaded.advance_minutes(DAY).expect("advance");
    assert_eq!(
        sim.land()
            .buildings
            .iter()
            .find(|b| b.id == building)
            .expect("building")
            .days_wet,
        loaded
            .land()
            .buildings
            .iter()
            .find(|b| b.id == building)
            .expect("building")
            .days_wet,
    );
    assert!(
        sim.land()
            .buildings
            .iter()
            .find(|b| b.id == building)
            .expect("building")
            .days_wet
            <= before + 1
    );
}

#[test]
fn water_carried_back_is_delivered_at_the_households_flood_refuge() {
    let mut sim = world();
    sim.advance_minutes(1).expect("initial cadence");
    let (owner, home, refuge, actor) = sim
        .people()
        .households
        .iter()
        .find_map(|(_, h)| {
            let refuge = h
                .settlement
                .and_then(|id| sim.land().settlements.iter().find(|s| s.id == id))?
                .hearth_m;
            let home_cell = civ_agents::population::cell_of(sim.map(), h.home);
            let refuge_cell = civ_agents::population::cell_of(sim.map(), refuge);
            (home_cell != refuge_cell).then_some((h.id, h.home, refuge, h.members[0]))
        })
        .expect("a home away from its hearth");
    let home_cell = civ_agents::population::cell_of(sim.map(), home);
    let refuge_cell = civ_agents::population::cell_of(sim.map(), refuge);
    sim.land_mut_for_tests().flood_depths[home_cell] = 0.5;
    sim.land_mut_for_tests().flood_depths[refuge_cell] = 0.0;
    let now = sim.now();
    {
        let hh = sim
            .people_mut_for_tests()
            .households
            .iter_mut()
            .find(|(_, h)| h.id == owner)
            .map(|(_, h)| h)
            .expect("household");
        hh.water_l = 0.0;
        hh.water_at = now;
    }
    {
        let p = sim
            .people_mut_for_tests()
            .people
            .iter_mut()
            .find(|(_, p)| p.id == actor)
            .map(|(_, p)| p)
            .expect("actor");
        p.pos = refuge;
        p.trip = None;
        p.asleep = false;
        p.carrying.water_l = 15.0;
    }
    sim.with_ctx_for_tests(|pop, ctx| pop.reconsider_for_tests(ctx, actor));
    sim.advance_minutes(2).expect("arrival");
    let hh = sim.people().household(owner).expect("household");
    assert!(
        hh.water_l > 14.0,
        "water {}, actor {:?}",
        hh.water_l,
        sim.people().person(actor).expect("actor").act
    );
    assert_eq!(sim.people().person(actor).expect("actor").pos, refuge);
    assert_eq!(hh.home, home, "refuge never moves the household's home");
    assert_eq!(
        sim.people().person(actor).expect("actor").carrying.water_l,
        0.0
    );
}
#[test]
fn shaking_checks_each_standing_building_once_and_its_record_resumes_exactly() {
    use civ_agents::influence::InfluenceKind;
    use civ_agents::quake::{Outcome, RESPONSE_SPREAD};
    let mut sim = world();
    let (_, hut) = weak_covering(&mut sim);
    // Out of the tool's range, or a settlement that is not, nothing changes.
    let before = paired(&mut sim);
    for strength in [0.01, 0.7, f64::NAN] {
        assert!(sim.shake(strength, None).is_err(), "{strength}");
    }
    let nowhere = civ_core::PermanentId::from_raw(999_999).expect("id");
    assert!(sim.shake(0.2, Some(nowhere)).is_err());
    same(&sim, &before);
    let standing: Vec<_> = sim
        .land()
        .buildings
        .iter()
        .filter(|b| !b.condition.is_empty() && b.standing())
        .map(|b| b.id)
        .collect();
    assert!(standing.contains(&hut));
    let reached = sim.shake(0.05, None).expect("shake");
    let q = sim.people().quakes.list.last().expect("a record").clone();
    assert_eq!((q.id, q.influence), (1, reached.id));
    assert!(
        sim.people()
            .influences
            .list
            .iter()
            .any(|i| i.id == q.influence && i.kind == InfluenceKind::Earthquake)
    );
    // Each standing building once, its response the event's strength by its own draw.
    let mut seen: Vec<_> = q.shaken.iter().map(|s| s.building).collect();
    seen.sort_unstable();
    let mut want = standing.clone();
    want.sort_unstable();
    assert_eq!(seen, want);
    for s in &q.shaken {
        let cs = 0.05 * (RESPONSE_SPREAD * f64::from(s.z)).exp();
        assert!((f64::from(s.cs) - cs).abs() < 1e-6, "{} {cs}", s.cs);
        assert_eq!(s.outcome == Outcome::Cracked, !s.cracked.is_empty());
    }
    // The record and what it did survive a save, and the two go on alike.
    let mut loaded = paired(&mut sim);
    same(&sim, &loaded);
    assert_eq!(loaded.people().quakes, sim.people().quakes);
    sim.advance_minutes(DAY).expect("day");
    loaded.advance_minutes(DAY).expect("day");
    same(&sim, &loaded);
    // The strongest shaking: what fell is failed and told as shaken down, and the dead counted.
    let deaths_before = sim
        .people()
        .chronicle
        .iter()
        .filter(|e| e.kind == civ_agents::history::ChronicleKind::BuildingFailed)
        .count();
    sim.shake(0.6, None).expect("shake");
    let q = sim.people().quakes.list.last().expect("a record").clone();
    assert_eq!(q.id, 2);
    let fell: Vec<_> = q
        .shaken
        .iter()
        .filter(|s| s.outcome == Outcome::Fell)
        .collect();
    let told: Vec<_> = sim
        .people()
        .chronicle
        .iter()
        .filter(|e| e.kind == civ_agents::history::ChronicleKind::BuildingFailed)
        .skip(deaths_before)
        .collect();
    assert_eq!(told.len(), fell.len());
    assert!(
        told.iter()
            .all(|e| e.name.contains("when the ground shook"))
    );
    let dead: u32 = fell.iter().map(|s| u32::from(s.deaths)).sum();
    assert_eq!(dead, told.iter().map(|e| e.number as u32).sum::<u32>());
    for s in &fell {
        let b = sim
            .land()
            .buildings
            .iter()
            .find(|b| b.id == s.building)
            .expect("building");
        assert!(
            b.condition
                .iter()
                .any(|c| c.state == civ_land::GroupState::Failed)
        );
    }
    eprintln!(
        "strongest: {} fell, {} cracked, {} stood of {}",
        fell.len(),
        q.shaken
            .iter()
            .filter(|s| s.outcome == Outcome::Cracked)
            .count(),
        q.shaken
            .iter()
            .filter(|s| s.outcome == Outcome::Stood)
            .count(),
        q.shaken.len()
    );
    let loaded = paired(&mut sim);
    same(&sim, &loaded);
}
