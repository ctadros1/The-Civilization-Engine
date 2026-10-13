//! A building burns (M6b slice BC; ADR-0022 §4, §5, §13, §14): the observer sets a covering
//! alight, the building burns by stages, its heat reaches its neighbours by the gap and the wind,
//! their coverings catch or hold, timber chars, goods under the roof burn, sleepers caught inside
//! may die, and the fire's record and the chronicle tell of it, all kept exactly across a save.

use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;

use civ_agents::build::{self, Shape};
use civ_agents::fire::{Kindled, Stage};
use civ_agents::history::{Cause, ChronicleKind};
use civ_agents::influence::InfluenceKind;
use civ_agents::params::BuildingDef;
use civ_agents::person::Flow;
use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_grammar::{GroupKind, Stage as BuildStage};
use civ_land::GroupState;
use civ_sim::{NewWorld, Sim, persist};
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

fn world_spec(seed: u64) -> NewWorld {
    NewWorld {
        name: "Fire".to_owned(),
        seed,
        preset_id: "core:worldgen/river_valley".to_owned(),
        size_cells: 512,
        band_size: 0,
        neighbours: Vec::new(),
        neighbours_known: false,
        regime_id: String::new(),
    }
}

fn world(seed: u64) -> Sim {
    Sim::create(
        &world_spec(seed),
        content(),
        &mut |_| {},
        &AtomicBool::new(false),
    )
    .expect("generates")
}

fn hut_def() -> &'static BuildingDef {
    let catalog = &content().catalog;
    &catalog.buildings[catalog
        .building_index("core:building/hut")
        .expect("the hut")]
}

/// The first household of the world and where it lives.
fn first(sim: &Sim) -> (PermanentId, (f32, f32)) {
    let (_, h) = sim
        .people()
        .households
        .iter()
        .min_by_key(|(_, h)| h.id)
        .expect("a household");
    (h.id, h.home)
}

/// A finished hut of `household` at `at`.
fn hut(sim: &mut Sim, household: PermanentId, at: (f32, f32)) -> PermanentId {
    let spec = build::design_shape(
        hut_def(),
        &content().catalog.goods,
        Shape::Round { radius: 310 },
        at,
        None,
    )
    .expect("a hut");
    sim.place_building_for_tests(household, spec, BuildStage::ALL.len() as u8)
        .expect("placed")
}

/// The width of a hut's roof, metres: the box its plot covers.
fn hut_span_m() -> f32 {
    let spec = build::design_shape(
        hut_def(),
        &content().catalog.goods,
        Shape::Round { radius: 310 },
        (0.0, 0.0),
        None,
    )
    .expect("a hut");
    build::plot_rect(&spec, hut_def()).w as f32 / 100.0
}

/// World `seed` at noon of its first day, the day's wind set to `wind_ms` from the west and the
/// thatch holding `moisture` of its mass: a block of huts `rows` deep (west to east) and `cols`
/// across, `gap_m` apart roof to roof, east of the first household's home. The world and the
/// huts, west row first.
fn block(
    seed: u64,
    rows: usize,
    cols: usize,
    gap_m: f32,
    wind_ms: f32,
    moisture: f32,
) -> (Sim, Vec<PermanentId>) {
    let mut sim = world(seed);
    sim.advance_minutes(6 * 60).expect("advances");
    let (household, home) = first(&sim);
    let step = hut_span_m() + gap_m;
    let mut huts = Vec::new();
    for r in 0..rows {
        for c in 0..cols {
            let at = (home.0 + 60.0 + r as f32 * step, home.1 + c as f32 * step);
            huts.push(hut(&mut sim, household, at));
        }
    }
    set_day(&mut sim, wind_ms, moisture);
    (sim, huts)
}

/// Today's wind `wind_ms` from the west and the thatch holding `moisture` of its mass.
fn set_day(sim: &mut Sim, wind_ms: f32, moisture: f32) {
    let w = &mut sim.land_mut_for_tests().weather;
    w.today.wind_ms = wind_ms;
    w.today.wind_from_deg = 270.0;
    w.today.gust_ms = wind_ms * 1.7;
    w.today.fuel_moisture = moisture;
}

/// How many of `huts` burned.
fn burned(sim: &Sim, huts: &[PermanentId]) -> usize {
    huts.iter()
        .filter(|&&h| sim.people().fires.list.iter().any(|f| f.building == h))
        .count()
}

#[test]
fn a_block_of_huts_a_metre_and_a_half_apart_in_wind_burns_within_minutes() {
    // 12-03 §2.2: twenty dwellings 1-2 m apart in 4.2-6.9 m/s of wind, four lit together, were
    // all involved within about five minutes.
    let (mut sim, huts) = block(1, 5, 4, 1.5, 5.5, 0.12);
    let began = sim.now();
    for &h in &huts[..4] {
        let reached = sim.set_fire(h).expect("set alight");
        assert!(!reached.repeat);
    }
    sim.advance_minutes(15).expect("advances");
    assert_eq!(
        burned(&sim, &huts),
        20,
        "every hut caught within the quarter hour"
    );
    let fires = &sim.people().fires;
    let last = fires
        .list
        .iter()
        .map(|f| f.began.minutes() - began.minutes())
        .max()
        .expect("fires");
    assert!(last <= 12, "the last caught {last} minutes in");
    // The first four were the observer's; every other caught from a burning neighbour and is
    // part of a conflagration.
    for f in &fires.list {
        match f.kindled {
            Kindled::Observer { influence } => {
                assert!(huts[..4].contains(&f.building));
                let i = sim
                    .people()
                    .influences
                    .list
                    .iter()
                    .find(|i| i.id == influence)
                    .expect("its influence");
                assert_eq!(
                    (i.kind, i.target, i.subject),
                    (InfluenceKind::Fire, f.building, f.id)
                );
            }
            Kindled::Neighbour { fire, gap_m, .. } => {
                assert!(fire < f.id && (0.0..10.0).contains(&gap_m));
                assert!(f.episode.is_some());
            }
            Kindled::Ignition { .. } => panic!("a fire in use escaped in the quarter hour"),
        }
    }
    // Every fire burns out half an hour on, and the conflagrations end with them.
    sim.advance_minutes(60).expect("advances");
    let fires = &sim.people().fires;
    assert!(fires.list.iter().all(|f| f.ended.is_some()));
    assert!(!fires.episodes.is_empty() && fires.episodes.iter().all(|e| e.ended.is_some()));
    assert!(fires.exposure.is_empty());
    assert!(fires.problems().is_empty(), "{:?}", fires.problems());
    let told = sim
        .people()
        .chronicle
        .iter()
        .filter(|e| e.kind == ChronicleKind::Fire)
        .count();
    assert!(told >= 2, "the chronicle tells the spread and its end");
}

#[test]
fn wider_gaps_still_air_or_damp_thatch_lose_fewer_huts() {
    let lost = |gap: f32, wind: f32, moisture: f32| -> usize {
        let (mut sim, huts) = block(2, 5, 4, gap, wind, moisture);
        for &h in &huts[..4] {
            sim.set_fire(h).expect("set alight");
        }
        sim.advance_minutes(90).expect("advances");
        burned(&sim, &huts)
    };
    let close = lost(1.5, 5.5, 0.12);
    assert_eq!(close, 20);
    let wide = lost(8.0, 0.0, 0.12);
    assert!(wide < close, "8 m apart in a calm: {wide}");
    let damp = lost(3.0, 5.5, 0.34);
    assert!(damp < close, "damp thatch 3 m apart: {damp}");
}

#[test]
fn a_hut_burns_out_as_parts_its_goods_burn_and_what_is_left_is_weighed() {
    let mut sim = world(3);
    sim.advance_minutes(6 * 60).expect("advances");
    let (household, home) = first(&sim);
    let id = hut(&mut sim, household, home);
    set_day(&mut sim, 1.0, 0.12);
    let lost = |sim: &Sim| -> f64 {
        sim.people()
            .households
            .iter()
            .find(|(_, h)| h.id == household)
            .map_or(0.0, |(_, h)| {
                (0..content().catalog.goods.len())
                    .map(|g| h.flows.get(Flow::Lost, g))
                    .sum()
            })
    };
    sim.set_fire(id).expect("set alight");
    // Refused again while it burns, and for a building that is not there.
    assert!(sim.set_fire(id).is_err());
    assert!(
        sim.set_fire(PermanentId::from_raw(u64::MAX - 1).expect("an id"))
            .is_err()
    );
    let params = sim.rules().people.fire.clone();
    sim.advance_minutes(1).expect("advances");
    assert_eq!(
        params.stage_at(sim.people().fires.list[0].minutes_at(sim.now())),
        Stage::Involved
    );
    sim.advance_minutes(40).expect("advances");
    let fire = sim.people().fires.list[0].clone();
    assert!(fire.ended.is_some() && fire.episode.is_none());
    // What lay under its roof burned, counted as lost.
    assert!(fire.goods_kg > 0.0);
    let goods = f64::from(fire.goods_kg);
    assert!((lost(&sim) - goods).abs() < 1e-4 * goods.max(1.0));
    let b = sim
        .land()
        .buildings
        .iter()
        .find(|b| b.id == id)
        .expect("the hut")
        .clone();
    // The covering is gone, the timber charred, the wattle half burned.
    for c in &b.condition {
        match GroupKind::of_group(c.group) {
            Some(GroupKind::Covering) => {
                assert_eq!((c.loss, c.state), (1.0, GroupState::Failed));
            }
            Some(GroupKind::Posts | GroupKind::RoofFrame) => assert!(c.loss > 0.0),
            Some(GroupKind::Infill) => assert!(c.loss >= 0.5),
            _ => {}
        }
    }
    // A building whose covering is gone cannot be set alight again.
    assert!(sim.set_fire(id).is_err());
    // The chronicle: the observer's hand, and the hut burned out.
    let lines: Vec<_> = sim
        .people()
        .chronicle
        .iter()
        .filter(|e| {
            e.kind == ChronicleKind::Fire
                || (e.kind == ChronicleKind::Influence
                    && e.number == f64::from(InfluenceKind::Fire.code()))
        })
        .map(|e| e.name.clone())
        .collect();
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert!(
        lines[1].starts_with("Fire burned out the hut of "),
        "{}",
        lines[1]
    );
}

/// How many of `dead` died in a fire.
fn died_in(sim: &Sim, dead: &[PermanentId]) -> usize {
    dead.iter()
        .filter(|d| {
            sim.people()
                .records
                .get(d)
                .is_some_and(|r| matches!(r.died, Some((_, Cause::Burned))))
        })
        .count()
}

#[test]
fn those_asleep_when_their_hut_is_fully_involved_get_out_or_die_and_none_sleep_on_in_it() {
    // Several worlds, each with its first household asleep in a hut at its home at night.
    let (mut caught, mut died) = (0, 0);
    for seed in 4..12 {
        let mut sim = world(seed);
        let (household, home) = first(&sim);
        let id = hut(&mut sim, household, home);
        // Two in the morning of the second day (a world begins at six): the household is abed.
        sim.advance_minutes(20 * 60).expect("advances");
        set_day(&mut sim, 1.0, 0.15);
        let at_home = |sim: &Sim| -> Vec<PermanentId> {
            let t = sim.now().minutes() as f64;
            sim.people()
                .people
                .iter()
                .filter(|(_, p)| {
                    p.household == household && p.asleep && p.trip.is_none() && {
                        let at = p.position_at(t);
                        (at.0 - home.0).hypot(at.1 - home.1) < 2.0
                    }
                })
                .map(|(_, p)| p.id)
                .collect()
        };
        let asleep = at_home(&sim);
        sim.set_fire(id).expect("set alight");
        sim.advance_minutes(2).expect("advances");
        let fire = sim.people().fires.list[0].clone();
        for &who in &asleep {
            caught += 1;
            let record = sim.people().records.get(&who).expect("a record");
            match record.died {
                Some((_, cause)) => {
                    died += 1;
                    assert_eq!(cause, Cause::Burned);
                    assert!(fire.deaths.contains(&who));
                }
                None => {
                    let p = sim.people().person(who).expect("living");
                    assert!(!p.asleep, "woken by the fire");
                }
            }
        }
        assert_eq!(fire.deaths.len(), died_in(&sim, &fire.deaths));
        // While it burns, none of the household sleeps on in it or comes back to.
        for _ in 0..28 {
            sim.advance_minutes(1).expect("advances");
            assert!(
                at_home(&sim).is_empty(),
                "seed {seed}: someone slept in a burning hut"
            );
        }
    }
    assert!(caught > 0, "nobody was asleep at home");
    assert!(
        died > 0 && died < caught,
        "{died} of {caught} caught asleep died"
    );
}

#[test]
fn a_world_saved_mid_fire_burns_on_as_if_never_saved() {
    let (mut sim, huts) = block(5, 3, 3, 2.0, 4.0, 0.12);
    for &h in &huts[..3] {
        sim.set_fire(h).expect("set alight");
    }
    sim.advance_minutes(4).expect("advances");
    assert!(sim.people().fires.any_burning());
    let dir = tempfile::tempdir().expect("tempdir");
    let saves = SaveDir::create(dir.path().join("saves"), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = persist::save(&mut sim, &saves, SaveKind::Manual, "fire").expect("saves");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loaded.people().fires, sim.people().fires);
    for s in [&mut sim, &mut loaded] {
        s.advance_minutes(60).expect("advances");
    }
    assert_eq!(loaded.people().fires, sim.people().fires);
    let conditions = |s: &Sim| -> Vec<_> {
        s.land()
            .buildings
            .iter()
            .map(|b| (b.id, b.condition.clone(), b.state))
            .collect()
    };
    assert_eq!(conditions(&loaded), conditions(&sim));
    assert_eq!(loaded.people().chronicle, sim.people().chronicle);
}

#[test]
fn fire_caution_survives_a_save_after_buildings_burn() {
    let (mut sim, _, huts) = block_near_home(5, 3.0, 5.5);
    sim.set_fire(huts[0]).expect("set alight");
    sim.advance_minutes(120).expect("advances");
    assert!(sim.people().fire_trust.iter().any(|t| t.losses > 0.0));
    let before = sim.people().fire_trust.clone();
    let dir = tempfile::tempdir().expect("tempdir");
    let saves = SaveDir::create(dir.path().join("saves"), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = persist::save(&mut sim, &saves, SaveKind::Manual, "fire caution").expect("saves");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loaded.people().fire_trust, before);
    let resaved = persist::save(&mut loaded, &saves, SaveKind::Manual, "fire caution again")
        .expect("saves again");
    let digests = |path: &Path| {
        let reader =
            commons_persist::SnapshotReader::open_file(path, commons_persist::Limits::default())
                .expect("opens");
        reader
            .chunks()
            .iter()
            .map(|c| (c.tag, c.index, c.raw_digest))
            .collect::<Vec<_>>()
    };
    assert_eq!(digests(&saved.path), digests(&resaved.path));
}

#[test]
fn the_observer_sees_each_fire_its_conflagration_what_held_and_the_chronicle_s_links() {
    use civ_schema::{flatbuffers, wire};
    use civ_sim::frames::buildings::buildings_response;
    use civ_sim::frames::incidents::{self, CONFLAGRATION, FIRE};
    let building = |sim: &Sim, id: PermanentId| -> (u8, u32, String) {
        let bytes = buildings_response(sim);
        let response = flatbuffers::root::<wire::Response>(&bytes).expect("decodes");
        let b = response
            .body_as_buildings()
            .and_then(|b| b.buildings())
            .expect("buildings")
            .iter()
            .find(|b| b.id() == id.get())
            .expect("the hut");
        (
            b.fire(),
            b.fire_id(),
            b.fire_words().unwrap_or("").to_owned(),
        )
    };
    // Six huts close in wind: the first set alight, the fire spreads.
    let (mut sim, huts) = block(6, 2, 3, 1.5, 5.5, 0.12);
    assert_eq!(building(&sim, huts[0]).0, 0, "no fire yet");
    sim.set_fire(huts[0]).expect("set alight");
    let (stage, id, words) = building(&sim, huts[0]);
    assert_eq!((stage, id), (1, 1));
    assert!(words.contains("set alight by the observer"), "{words}");
    sim.advance_minutes(3).expect("advances");
    assert_eq!(building(&sim, huts[0]).0, 2, "fully involved");
    let bytes = incidents::incidents_response(&sim, None, FIRE, None, 0);
    let response = flatbuffers::root::<wire::Response>(&bytes).expect("decodes");
    let list = response.body_as_incidents().expect("incidents");
    let rows: Vec<_> = list.rows().expect("rows").iter().collect();
    let fires = sim.people().fires.list.len();
    assert!(fires > 1, "it spread");
    assert_eq!(rows.len(), fires);
    assert!(
        rows.iter()
            .all(|r| r.kind() == FIRE && r.open() && r.observer() > 0)
    );
    let total = list
        .totals()
        .expect("totals")
        .iter()
        .find(|t| t.kind() == FIRE)
        .expect("fires counted");
    assert_eq!(
        (total.all(), total.observer()),
        (fires as u32, fires as u32)
    );
    assert_eq!(list.conflagrations().expect("conflagrations").len(), 1);
    // Burnt out, each says so on the map, and its page tells its stages and its cause.
    sim.advance_minutes(60).expect("advances");
    let (stage, _, words) = building(&sim, huts[0]);
    assert_eq!(stage, 3);
    assert!(words.starts_with("burnt out at "), "{words}");
    let page = |sim: &Sim, id: u32| -> Vec<u8> {
        incidents::incident_response(sim, FIRE, id).expect("a page")
    };
    let bytes = page(&sim, 1);
    let response = flatbuffers::root::<wire::Response>(&bytes).expect("decodes");
    let info = response.body_as_incident_info().expect("a fire's page");
    let steps: Vec<(&str, u8)> = info
        .steps()
        .expect("steps")
        .iter()
        .map(|s| (s.label().unwrap_or(""), s.state()))
        .collect();
    assert_eq!(
        steps,
        vec![("Caught", 0), ("Fully involved", 0), ("Burnt out", 0)]
    );
    let links: Vec<(u8, u8)> = info
        .links()
        .expect("links")
        .iter()
        .map(|l| (l.relation(), l.target()))
        .collect();
    assert_eq!(
        links,
        vec![(0, 0), (1, 8)],
        "the observer's, part of a conflagration"
    );
    assert!(
        info.row()
            .and_then(|r| r.who_name())
            .is_some_and(|n| n.starts_with("The hut of ")),
        "named by its building"
    );
    let bytes = page(&sim, 2);
    let response = flatbuffers::root::<wire::Response>(&bytes).expect("decodes");
    let caught = response
        .body_as_incident_info()
        .and_then(|i| i.links())
        .expect("links")
        .get(0);
    assert_eq!(
        (caught.relation(), caught.target()),
        (0, 7),
        "caught from a fire"
    );
    assert!(caught.id() < 2 && caught.words().is_some_and(|w| w.contains(" m away")));
    // The conflagration's page: its fires in the order they caught, its cause and its weather.
    let bytes = incidents::conflagration_response(&sim, 1).expect("its page");
    let response = flatbuffers::root::<wire::Response>(&bytes).expect("decodes");
    let c = response
        .body_as_conflagration_info()
        .expect("a conflagration");
    let row = c.row().expect("its row");
    let fires = sim.people().fires.list.len();
    assert_eq!(row.burned() as usize, fires);
    assert!(row.ended() > row.began());
    assert_eq!(c.fires().expect("fires").len(), fires);
    assert!(
        c.cause()
            .is_some_and(|w| w.contains("set alight by the observer"))
    );
    assert!(
        c.weather()
            .is_some_and(|w| w.starts_with("When it first spread"))
    );
    assert!(incidents::conflagration_response(&sim, 9).is_none());
    assert!(incidents::incident_response(&sim, FIRE, 999).is_none());
    // Every fire entry of the chronicle, and the observer's hand, opens what it tells of.
    let linked: Vec<_> = sim
        .people()
        .chronicle
        .iter()
        .filter(|e| {
            e.kind == ChronicleKind::Fire
                || (e.kind == ChronicleKind::Influence
                    && e.number == f64::from(InfluenceKind::Fire.code()))
        })
        .map(|e| incidents::chronicle_link(&sim, e))
        .collect();
    assert_eq!(
        linked,
        vec![
            Some((FIRE, 1)),
            Some((CONFLAGRATION, 1)),
            Some((CONFLAGRATION, 1))
        ]
    );

    // A lone hut in still air: its page tells what its heat reached and what held.
    let (mut sim, huts) = block(7, 1, 2, 4.0, 0.0, 0.12);
    sim.set_fire(huts[0]).expect("set alight");
    sim.advance_minutes(40).expect("advances");
    assert_eq!(sim.people().fires.list.len(), 1, "it stayed alone");
    let bytes = page(&sim, 1);
    let response = flatbuffers::root::<wire::Response>(&bytes).expect("decodes");
    let truth: Vec<&str> = response
        .body_as_incident_info()
        .and_then(|i| i.truth())
        .expect("truth")
        .iter()
        .collect();
    let at = truth
        .iter()
        .position(|l| *l == "Its heat reached these, and they held:")
        .expect("what held");
    assert!(
        truth[at + 1].starts_with("The hut of ") && truth[at + 1].contains(" its covering needed"),
        "{truth:?}"
    );
    let linked: Vec<_> = sim
        .people()
        .chronicle
        .iter()
        .filter(|e| e.kind == ChronicleKind::Fire)
        .map(|e| incidents::chronicle_link(&sim, e))
        .collect();
    assert_eq!(linked, vec![Some((FIRE, 1))]);
}

/// World `seed` with a hut at each of its first `homes` households' homes, saved; two copies
/// loaded from that save, each with its homes' fires in use escaping at `per_hour` an hour, lit or
/// banked (no worked fires).
fn hearth_worlds(seed: u64, homes: usize, per_hour: f64) -> (Sim, Sim, Vec<PermanentId>) {
    let mut sim = world(seed);
    let mut households: Vec<(PermanentId, (f32, f32))> = sim
        .people()
        .households
        .iter()
        .map(|(_, h)| (h.id, h.home))
        .collect();
    households.sort_by_key(|h| h.0);
    let huts: Vec<PermanentId> = households
        .iter()
        .take(homes)
        .map(|&(h, at)| hut(&mut sim, h, at))
        .collect();
    let dir = tempfile::tempdir().expect("tempdir");
    let saves = SaveDir::create(dir.path().join("saves"), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = persist::save(&mut sim, &saves, SaveKind::Manual, "homes").expect("saves");
    let load = || {
        let mut s = persist::load(&saved.path, content()).expect("loads");
        let fire = &mut s
            .rules_mut_for_tests()
            .expect("rules of its own")
            .people
            .fire;
        fire.hearth_lit_per_hour = per_hour;
        fire.hearth_banked_per_hour = per_hour;
        fire.worked_per_hour = 0.0;
        s
    };
    (load(), load(), huts)
}

#[test]
fn homes_catch_fire_from_their_hearths_at_the_same_minutes_whatever_the_speed() {
    use civ_agents::fire::Hearth;
    let (mut a, mut b, huts) = hearth_worlds(9, 6, 0.02);
    // One lives five days at once, the other in uneven pieces.
    a.advance_minutes(5 * 1440).expect("advances");
    let mut left = 5 * 1440;
    for step in [7, 13, 101, 389, 1440].iter().cycle() {
        let n = (*step).min(left);
        b.advance_minutes(n).expect("advances");
        left -= n;
        if left == 0 {
            break;
        }
    }
    // The same escapes, fires and knowledge of them, whatever the speed.
    let fires = &a.people().fires;
    assert_eq!(fires, &b.people().fires);
    assert!(fires.problems().is_empty(), "{:?}", fires.problems());
    let ignitions = &fires.ignitions;
    assert!(ignitions.len() >= 5, "{} ignitions", ignitions.len());
    let [lo, hi] = a.rules().people.fire.grow_minutes;
    // Only homes ignite; a hearth lit has someone awake at home, who puts it out at once.
    for g in ignitions {
        assert!(huts.contains(&g.building));
        if g.hearth == Hearth::Lit {
            assert_eq!(g.ended, Some(g.at), "a lit hearth's ignition escaped");
            assert!(g.put_out_by.is_some());
        }
        assert_ne!(g.hearth, Hearth::Worked);
        if g.ended == Some(g.at) {
            assert!(
                g.put_out_by.is_some(),
                "only one put out at once ends at once"
            );
            continue;
        }
        // Left alone, it grew for its minutes before it could reach the roof.
        assert!(
            f64::from(g.grows_min) >= lo && f64::from(g.grows_min) <= hi,
            "grew {} minutes",
            g.grows_min
        );
        if let Some(f) = g.fire {
            let fire = fires.get(f).expect("its fire");
            assert_eq!(fire.kindled, Kindled::Ignition { ignition: g.id });
            assert_eq!((fire.building, Some(fire.began)), (g.building, g.ended));
            let grew = fire.began.minutes() - g.at.minutes();
            assert!(grew as f32 >= g.grows_min, "caught after {grew} minutes");
        }
    }
    // Most are put out, at once or while they grow; some, with nobody to find them, catch.
    let at_once = ignitions
        .iter()
        .filter(|g| g.put_out_by.is_some() && g.ended == Some(g.at))
        .count();
    let growing = ignitions
        .iter()
        .filter(|g| g.put_out_by.is_some() && g.ended > Some(g.at))
        .count();
    let caught = ignitions.iter().filter(|g| g.fire.is_some()).count();
    assert!(
        at_once > 0 && growing > 0 && caught > 0,
        "{at_once} put out at once, {growing} while growing, {caught} caught"
    );
}

/// The people awake or asleep, and where, at this minute.
fn placed(sim: &Sim) -> Vec<(PermanentId, (f32, f32), bool)> {
    let t = sim.now().minutes() as f64;
    sim.people()
        .people
        .iter()
        .map(|(_, p)| (p.id, p.position_at(t), p.asleep))
        .collect()
}

fn apart(a: (f32, f32), b: (f32, f32)) -> f64 {
    f64::from(a.0 - b.0).hypot(f64::from(a.1 - b.1))
}

#[test]
fn a_hut_alight_at_noon_is_seen_within_sight_and_its_alarm_heard_within_earshot() {
    use civ_agents::fire::Noticed;
    use civ_schema::{flatbuffers, wire};
    use civ_sim::frames::incidents::{self, FIRE};
    use std::collections::BTreeSet;
    let (mut sim, huts) = block(4, 1, 1, 3.0, 1.0, 0.15);
    // Sight shorter than a shout carries, so that the alarm reaches some who did not see it.
    sim.rules_mut_for_tests()
        .expect("rules")
        .people
        .fire
        .sight_m = 60.0;
    let (sight, earshot) = {
        let p = &sim.rules().people.fire;
        (p.sight_m, p.earshot_m)
    };
    let at = sim
        .land()
        .buildings
        .iter()
        .find(|b| b.id == huts[0])
        .map(|b| build::centre_m(&b.spec))
        .expect("the hut");
    let r = f64::from(hut_span_m()) / 2.0;
    let before = placed(&sim);
    let where_ = |id: PermanentId| before.iter().find(|x| x.0 == id).map(|x| x.1);
    sim.set_fire(huts[0]).expect("set alight");
    let now = sim.now();
    let knew = sim.people().fires.knew.clone();
    // Everyone awake within sight of its roof saw it at once, and nobody else did.
    let seen: BTreeSet<PermanentId> = before
        .iter()
        .filter(|&&(_, p, asleep)| !asleep && (apart(p, at) - r).max(0.0) <= sight)
        .map(|x| x.0)
        .collect();
    let saw: BTreeSet<PermanentId> = knew
        .iter()
        .filter(|k| k.how == Noticed::Saw)
        .map(|k| k.person)
        .collect();
    assert!(!saw.is_empty(), "nobody saw it");
    assert_eq!(saw, seen);
    // Each who saw it shouted at once: everyone else within earshot of one of them heard it,
    // awake or asleep, and nobody further.
    let in_earshot: BTreeSet<PermanentId> = before
        .iter()
        .filter(|&&(id, p, _)| {
            !saw.contains(&id)
                && saw
                    .iter()
                    .any(|&s| where_(s).is_some_and(|q| apart(q, p) <= earshot))
        })
        .map(|x| x.0)
        .collect();
    let heard: BTreeSet<PermanentId> = knew
        .iter()
        .filter(|k| k.how == Noticed::Heard)
        .map(|k| k.person)
        .collect();
    assert!(!heard.is_empty(), "nobody heard the alarm");
    assert_eq!(heard, in_earshot);
    assert_eq!(knew.len(), saw.len() + heard.len(), "one record each");
    assert!(
        knew.iter()
            .all(|k| k.at == now && k.shouted == (k.how != Noticed::Heard))
    );
    // Those who came to know of it are awake and stopped to decide again, unless carrying a load
    // home.
    for k in &knew {
        let p = sim.people().person(k.person).expect("living");
        assert!(!p.asleep, "woken by the shout");
        let laden = p.carrying.kg > 0.0 || p.carrying.water_l > 0.0;
        assert!(
            laden || p.act.steps == vec![civ_agents::person::Step::Wait { minutes: 1 }],
            "stopped"
        );
    }
    // Its household keeps from home only those who saw it (none was inside).
    let (household, _) = first(&sim);
    let members = sim
        .people()
        .household(household)
        .expect("the household")
        .members
        .clone();
    for m in members {
        assert_eq!(
            sim.people()
                .knows_home_afire(m, household, now, &sim.rules().people),
            saw.contains(&m),
            "only those who saw it know their hut burns"
        );
    }
    // A minute on, those who heard shout in turn: each new hearer is within earshot of one.
    sim.advance_minutes(1).expect("advances");
    let later = placed(&sim);
    let knew = sim.people().fires.knew.clone();
    let shouting: Vec<(f32, f32)> = knew
        .iter()
        .filter(|k| k.at == now && k.how == Noticed::Heard)
        .chain(
            knew.iter()
                .filter(|k| k.at > now && k.how != Noticed::Heard),
        )
        .filter_map(|k| later.iter().find(|x| x.0 == k.person).map(|x| x.1))
        .collect();
    for k in knew
        .iter()
        .filter(|k| k.at > now && k.how == Noticed::Heard)
    {
        let p = later.iter().find(|x| x.0 == k.person).expect("placed").1;
        assert!(
            shouting.iter().any(|&q| apart(p, q) <= earshot),
            "heard from beyond earshot"
        );
    }
    // Burnt out, its page says who knew of it, first, and that they saw its roof catch.
    sim.advance_minutes(45).expect("advances");
    assert!(sim.people().fires.problems().is_empty());
    let bytes = incidents::incident_response(&sim, FIRE, 1).expect("a page");
    let response = flatbuffers::root::<wire::Response>(&bytes).expect("decodes");
    let info = response.body_as_incident_info().expect("a fire's page");
    let lines: Vec<&str> = info.knowledge().expect("what people know").iter().collect();
    let seen_by = sim
        .people()
        .fires
        .knew
        .iter()
        .filter(|k| k.how == Noticed::Saw)
        .count();
    assert!(seen_by >= saw.len());
    assert!(
        lines[0].starts_with(&format!("Seen burning by {seen_by}")),
        "{lines:?}"
    );
    assert!(lines.iter().any(|l| l.starts_with("First to know: ")));
    assert!(
        lines
            .iter()
            .any(|l| l.ends_with(&format!("{} others saw its roof catch", saw.len() - 2))),
        "{lines:?}"
    );
    assert!(lines.iter().any(|l| l.starts_with("The alarm reached ")));
}

#[test]
fn at_night_those_woken_by_their_burning_hut_wake_their_neighbours() {
    use civ_agents::fire::Noticed;
    let (mut woke, mut roused) = (0, 0);
    for seed in 4..8 {
        let mut sim = world(seed);
        let (household, home) = first(&sim);
        let id = hut(&mut sim, household, home);
        // Two in the morning of the second day: the village is abed.
        sim.advance_minutes(20 * 60).expect("advances");
        set_day(&mut sim, 1.0, 0.15);
        let before = placed(&sim);
        sim.set_fire(id).expect("set alight");
        sim.advance_minutes(3).expect("advances");
        let fire = sim.people().fires.list[0].clone();
        let knew = &sim.people().fires.knew;
        // Those of the household asleep inside who got out woke to it when it was fully
        // involved; nobody else did.
        for k in knew.iter().filter(|k| k.how == Noticed::Woke) {
            woke += 1;
            assert_eq!(k.fire, fire.id);
            assert!(!fire.deaths.contains(&k.person));
            let p = sim.people().person(k.person).expect("living");
            assert_eq!(p.household, household);
            assert!(
                apart(
                    before.iter().find(|x| x.0 == k.person).expect("placed").1,
                    home
                ) < 3.0
            );
        }
        // Their shout woke others asleep near.
        roused += knew
            .iter()
            .filter(|k| {
                k.how == Noticed::Heard
                    && before.iter().any(|x| x.0 == k.person && x.2)
                    && sim
                        .people()
                        .person(k.person)
                        .is_some_and(|p| p.household != household)
            })
            .count();
        assert!(sim.people().fires.problems().is_empty());
    }
    assert!(woke > 0, "nobody woke to a burning hut");
    assert!(roused > 0, "no neighbour was woken by the alarm");
}

#[test]
fn a_world_saved_with_ignitions_pending_lives_on_as_if_never_saved() {
    let (mut a, mut b, _) = hearth_worlds(10, 6, 0.02);
    for s in [&mut a, &mut b] {
        s.advance_minutes(1440 + 10 * 60).expect("advances");
    }
    // B is saved mid-morning with the day's candidates pending, and loaded.
    let dir = tempfile::tempdir().expect("tempdir");
    let saves = SaveDir::create(dir.path().join("saves"), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = persist::save(&mut b, &saves, SaveKind::Manual, "mid").expect("saves");
    let mut b = persist::load(&saved.path, content()).expect("loads");
    let fire = &mut b.rules_mut_for_tests().expect("rules").people.fire;
    fire.hearth_lit_per_hour = 0.02;
    fire.hearth_banked_per_hour = 0.02;
    fire.worked_per_hour = 0.0;
    for s in [&mut a, &mut b] {
        s.advance_minutes(2 * 1440).expect("advances");
    }
    assert!(!a.people().fires.ignitions.is_empty());
    assert_eq!(a.people().fires, b.people().fires);
}

/// World `seed` at noon of its first day, the day's wind `wind_ms` from the west and thatch at
/// 0.12: a block of huts two deep and three across, `gap_m` apart, of the first household, 15 m
/// east of its home and 12 m south, saved and loaded twice so that both worlds are one world
/// (M6b slice BD, step three).
fn block_near_home(seed: u64, gap_m: f32, wind_ms: f32) -> (Sim, Sim, Vec<PermanentId>) {
    // Hold scheduling tie-breaks fixed: this is a controlled response fixture, not a sample
    // of worlds with random identities. Ordinary simulations remain free to vary.
    let mut sim = Sim::create_for_tests(&world_spec(seed), content(), [7; 16]).expect("generates");
    sim.advance_minutes(6 * 60).expect("advances");
    let (household, home) = first(&sim);
    let step = hut_span_m() + gap_m;
    let mut huts = Vec::new();
    for r in 0..2 {
        for c in 0..3 {
            let at = (
                home.0 + 15.0 + r as f32 * step,
                home.1 + 12.0 + c as f32 * step,
            );
            huts.push(hut(&mut sim, household, at));
        }
    }
    set_day(&mut sim, wind_ms, 0.12);
    let dir = tempfile::tempdir().expect("tempdir");
    let saves = SaveDir::create(dir.path().join("saves"), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = persist::save(&mut sim, &saves, SaveKind::Manual, "block").expect("saves");
    let load = || persist::load(&saved.path, content()).expect("loads");
    (load(), load(), huts)
}

#[test]
fn people_carry_water_to_roofs_a_fire_threatens_and_save_some() {
    use civ_agents::person::FROM_STORE;
    let (mut lost_dry, mut lost_wet, mut loads) = (0, 0, 0);
    for seed in 4..7 {
        let (mut dry, mut wet, huts) = block_near_home(seed, 3.0, 5.5);
        let carry = wet.rules().people.household.carry_water_l as f32;
        // Nobody cares to carry water in one; in the other, as people do.
        dry.rules_mut_for_tests().expect("rules").people.fire.w_save = 0.0;
        for s in [&mut dry, &mut wet] {
            s.set_fire(huts[0]).expect("set alight");
            s.advance_minutes(120).expect("advances");
            assert!(
                s.people().fires.problems().is_empty(),
                "{:?}",
                s.people().fires.problems()
            );
        }
        assert!(dry.people().fires.thrown.is_empty());
        lost_dry += burned(&dry, &huts);
        lost_wet += burned(&wet, &huts);
        let fires = &wet.people().fires;
        loads += fires.thrown.len();
        for t in &fires.thrown {
            // A load at most, thrown against the fire burning then, on a roof of the block.
            assert!(t.litres > 0.0 && t.litres <= carry, "{} L", t.litres);
            assert_eq!(fires.root(t.fire), t.fire);
            assert!(huts.contains(&t.building));
            if t.from == FROM_STORE {
                continue;
            }
            assert!(t.from < wet.map().width * wet.map().height);
        }
        // Burnt out, nothing stands wet.
        assert!(fires.wet.is_empty());
        // The conflagration's page says what was carried against it.
        if let Some(c) = fires
            .episodes
            .iter()
            .find(|c| fires.thrown.iter().any(|t| t.fire == c.first))
        {
            use civ_schema::{flatbuffers, wire};
            let bytes =
                civ_sim::frames::incidents::conflagration_response(&wet, c.id).expect("page");
            let response = flatbuffers::root::<wire::Response>(&bytes).expect("decodes");
            let info = response
                .body_as_conflagration_info()
                .expect("a conflagration");
            let lines: Vec<&str> = info.held().expect("held").iter().collect();
            assert!(
                lines
                    .iter()
                    .any(|l| l.starts_with("Water was carried against it: ")),
                "{lines:?}"
            );
        }
    }
    assert!(loads > 0, "nobody carried water");
    assert!(
        lost_wet < lost_dry,
        "{lost_wet} burned with water carried, {lost_dry} without"
    );
}

#[test]
fn a_wet_roof_boils_its_water_off_before_it_can_catch() {
    // Two huts 1.5 m apart in wind, nobody to carry water: the downwind one catches within
    // minutes, unless its thatch holds water, which the heat must boil off first.
    let caught_after = |litres: f64| -> Option<i64> {
        let (mut sim, huts) = block(4, 2, 1, 1.5, 5.5, 0.12);
        sim.rules_mut_for_tests().expect("rules").people.fire.w_save = 0.0;
        if litres > 0.0 {
            sim.people_mut_for_tests().fires.wet.push((huts[1], litres));
        }
        sim.set_fire(huts[0]).expect("set alight");
        let began = sim.now().minutes();
        sim.advance_minutes(30).expect("advances");
        let fires = &sim.people().fires;
        fires
            .list
            .iter()
            .find(|f| f.building == huts[1])
            .map(|f| f.began.minutes() - began)
    };
    let dry = caught_after(0.0).expect("a dry roof catches");
    assert!(dry <= 5, "caught after {dry} minutes");
    match caught_after(400.0) {
        None => {}
        Some(wet) => assert!(wet > dry + 5, "wet caught after {wet}, dry after {dry}"),
    }
}

#[test]
fn carrying_water_is_the_same_whatever_the_speed_and_across_a_save() {
    let (mut a, mut b, huts) = block_near_home(4, 3.0, 5.5);
    for s in [&mut a, &mut b] {
        s.set_fire(huts[0]).expect("set alight");
    }
    // One lives the fire at once; the other in uneven steps, saved and loaded midway.
    a.advance_minutes(90).expect("advances");
    let mut left = 90;
    let mut saved_once = false;
    for step in [1, 3, 7, 2, 11].iter().cycle() {
        let n = (*step).min(left);
        b.advance_minutes(n).expect("advances");
        left -= n;
        if !saved_once && left < 60 {
            saved_once = true;
            let dir = tempfile::tempdir().expect("tempdir");
            let saves =
                SaveDir::create(dir.path().join("saves"), civ_schema::SAVE_EXTENSION).expect("dir");
            let saved = persist::save(&mut b, &saves, SaveKind::Manual, "mid").expect("saves");
            b = persist::load(&saved.path, content()).expect("loads");
        }
        if left == 0 {
            break;
        }
    }
    assert!(!a.people().fires.thrown.is_empty(), "nobody carried water");
    assert_eq!(a.people().fires, b.people().fires);
}

#[test]
fn a_household_strips_its_threatened_roofs_and_carries_its_goods_out_of_the_way() {
    let (mut strips, mut loads) = (0, 0);
    for seed in 4..8 {
        let (mut idle, mut busy, huts) = block_near_home(seed, 3.0, 5.5);
        // In one, stripping costs as much as burning and goods are worth nothing to save. In the
        // other, one of the two is made cheap, by turns, so that each is exercised whichever way
        // the tuning leans (carrying water, or the other, would otherwise often be the better
        // choice, and the scheduler's tie-break differs between worlds).
        for (s, strip_cost, salvage_share) in [
            (&mut idle, 1.0, 0.0),
            (
                &mut busy,
                if seed % 2 == 0 { 0.0 } else { 1.0 },
                if seed % 2 == 0 { 0.0 } else { 2.0 },
            ),
        ] {
            let fire = &mut s.rules_mut_for_tests().expect("rules").people.fire;
            fire.strip_cost = strip_cost;
            fire.salvage_share = salvage_share;
        }
        for s in [&mut idle, &mut busy] {
            s.set_fire(huts[0]).expect("set alight");
            s.advance_minutes(120).expect("advances");
            let fires = &s.people().fires;
            assert!(fires.problems().is_empty(), "{:?}", fires.problems());
        }
        let quiet = &idle.people().fires;
        assert!(quiet.stripped.is_empty() && quiet.salvaged.is_empty());
        let fires = &busy.people().fires;
        let (household, _) = first(&busy);
        for s in &fires.stripped {
            // A roof of the block, stripped by its own household or with leave, against the fire
            // burning then; its covering is the poorer for it.
            assert!(huts.contains(&s.building));
            assert_eq!(fires.root(s.fire), s.fire);
            let b = busy
                .land()
                .buildings
                .iter()
                .find(|b| b.id == s.building)
                .expect("still stands");
            assert!(
                b.condition
                    .iter()
                    .filter(|c| GroupKind::of_group(c.group) == Some(GroupKind::Covering))
                    .all(|c| c.loss > 0.0)
            );
        }
        let carry = busy.rules().people.household.carry_kg as f32;
        for s in &fires.salvaged {
            // Only the household's own goods, a load at most each time.
            assert!(huts.contains(&s.building));
            let p = busy.people().person(s.person);
            assert!(p.is_none_or(|p| p.household == household));
            assert!(s.kg <= carry + 1e-3);
        }
        strips += fires.stripped.len();
        loads += fires.salvaged.len();
    }
    eprintln!("stripped {strips}, loads {loads}");
    assert!(strips > 0, "nobody stripped a roof");
    assert!(loads > 0, "nobody carried goods out");
}
