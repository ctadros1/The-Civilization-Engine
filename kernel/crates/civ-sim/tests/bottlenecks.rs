//! Bottlenecks for walkers (M8b slice CN; ADR-0029 §4): a village inside a whole palisade passes
//! its open gates first come; at their own pace (a walker every 0.7 s through each way's files of
//! a 2.5 m gate) a village of 30 waits a fraction of a second a passage and never a minute, and
//! made slow enough to queue, its walkers wait at
//! the approach, their trips are re-timed as new revisions, their households remember the waits,
//! and everyone still arrives. It all saves and goes on alike.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::traffic::Facility;
use civ_content::ContentRegistry;
use civ_land::enclosures::{Enclosure, ring};
use civ_schema::{flatbuffers, wire};
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

/// A village of world `seed`, with a fixed identity so it lives the same life every run.
fn village(seed: u64) -> Sim {
    Sim::create_for_tests(
        &NewWorld {
            name: "Gates".to_owned(),
            seed,
            preset_id: PRESET.to_owned(),
            size_cells: 768,
            band_size: 30,
            neighbours: Vec::new(),
            neighbours_known: true,
            regime_id: String::new(),
        },
        content(),
        [7; 16],
    )
    .expect("generates")
}

/// A whole palisade 80 m round the first settlement's hearth with the content's gates, put in
/// the world and laid on the walking grid.
fn enclose(sim: &mut Sim) -> Enclosure {
    let s = &sim.land().settlements[0];
    let (settlement, hearth) = (s.id, s.hearth_m);
    let ep = sim.rules().people.enclosures.clone();
    let sectors = ring(
        hearth,
        80.0,
        ep.sector_m as f32,
        ep.gates as usize,
        0.0,
        ep.palisade_h_per_m as f32,
        sim.map().cell_size_m,
        sim.map().width,
        sim.map().height,
    );
    let id = sim.allocate_id_for_tests();
    let mut e = Enclosure {
        id,
        owner: settlement,
        law: settlement,
        settlement,
        centre: hearth,
        radius_m: 80.0,
        sectors,
        work_h: 0.0,
        shares: Vec::new(),
        crew: (0, 0),
        begun: sim.now(),
        finished: Some(sim.now().day_index()),
        upkeep_h: 0.0,
    };
    e.work_h = e.labour_h();
    let land = sim.land_mut_for_tests();
    land.enclosures.list = vec![e.clone()];
    land.enclosures.changed();
    sim.lay_decks_for_tests();
    e
}

/// Passages and minutes waited at every facility, as remembered today.
fn tallied(sim: &Sim) -> (f32, f32) {
    let day = sim.now().day_index();
    sim.people()
        .traffic
        .tallies
        .iter()
        .map(|t| t.faded(day).totals())
        .fold((0.0, 0.0), |a, b| (a.0 + b.0, a.1 + b.1))
}

/// Saves `sim`, loads it, and checks every section is the same and that the two go on alike for
/// `minutes`, with the same test headway (which is not saved) on both.
fn saves_and_goes_on_alike(sim: &mut Sim, minutes: i64) {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "gates").expect("saves");
    let mut loaded = persist::load(&saved.path, content()).expect("loads");
    loaded.people_mut_for_tests().traffic.test_headway_s = sim.people().traffic.test_headway_s;
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
fn a_village_passes_its_gates_without_waiting_and_made_slow_its_walkers_queue_and_arrive() {
    let mut sim = village(1);
    let e = enclose(&mut sim);
    assert!(
        e.sectors.iter().any(|s| s.gate.is_some()),
        "the line has gates"
    );
    // At the gates' own pace.
    sim.advance_minutes(3 * DAY).expect("lives");
    let (passed, waited) = tallied(&sim);
    assert!(
        passed > 20.0,
        "people went out and in by the gates: {passed}"
    );
    assert!(
        sim.people()
            .traffic
            .tallies
            .iter()
            .all(|t| matches!(t.facility, Facility::Gate { enclosure, .. } if enclosure == e.id)),
        "only the gates are facilities here"
    );
    // Those who set off together step through one behind another, a fraction of a second each;
    // nobody waits a minute, so no trip is re-timed and no household holds a wait.
    assert!(
        waited * 60.0 / passed < 1.0,
        "a village of 30 hardly waits at a 2.5 m gate: {waited} minutes over {passed} passages"
    );
    assert!(sim.people().traffic.waits.is_empty());

    // Made slow: a walker every five minutes each way.
    let living = sim.people().living();
    sim.people_mut_for_tests().traffic.test_headway_s = Some(300.0);
    let mut revised = 0;
    for _ in 0..(2 * 24) {
        sim.advance_minutes(60).expect("lives");
        revised += sim
            .people()
            .people
            .iter()
            .filter(|(_, p)| p.trip.as_ref().is_some_and(|t| t.rev > 0))
            .count();
    }
    let (_, waited) = tallied(&sim);
    assert!(
        waited > 30.0,
        "walkers waited at the gates: {waited} minutes"
    );
    assert!(revised > 0, "waiting walkers' trips were re-timed");
    let day = sim.now().day_index();
    let half = sim.rules().people.places.use_half_life_days;
    let traffic = &sim.people().traffic;
    let held: f32 = traffic
        .waits
        .keys()
        .flat_map(|&h| traffic.held(h, day, half))
        .map(|w| w.minutes)
        .sum();
    assert!(held > 30.0, "households remember their waits: {held}");
    // Nobody vanished in the queue, and nobody is stuck in it.
    assert_eq!(sim.people().living(), living);
    let now = sim.now();
    assert!(
        sim.people()
            .people
            .iter()
            .all(|(_, p)| p.act.step_ends >= now),
        "every walk still ends"
    );
    assert!(sim.people().traffic.problems().is_empty());
    // Waits of a quarter of an hour or more are told as incidents.
    let long = &sim.people().traffic.long_waits;
    assert!(!long.is_empty(), "some waited a quarter of an hour");
    assert!(long.iter().all(|w| w.minutes >= 15.0));
    let payload = civ_sim::frames::incidents::incidents_response(
        &sim,
        None,
        civ_sim::frames::incidents::WAIT,
        None,
        0,
    );
    let response = flatbuffers::root::<wire::Response>(&payload).expect("decodes");
    let rows = response
        .body_as_incidents()
        .and_then(|i| i.rows())
        .expect("rows");
    assert!(!rows.is_empty());
    let words = rows.get(0).happened().unwrap_or_default().to_owned();
    assert!(
        words.starts_with("waited ") && words.contains("a gate of"),
        "{words}"
    );
    // The Traffic lens: the gates, busiest first, with their passages and waits by hour.
    let payload = civ_sim::frames::traffic::lens_response(&sim, None);
    let response = flatbuffers::root::<wire::Response>(&payload).expect("decodes");
    let lens = response.body_as_coverage().expect("coverage");
    assert_eq!(lens.lens(), civ_sim::frames::traffic::TRAFFIC);
    let places = lens.places().expect("places");
    assert!(!places.is_empty());
    let first = places.get(0);
    assert_eq!(first.hour_passages().map_or(0, |v| v.len()), 24);
    assert!(first.passages() > 0.0 && first.waited_min() > 0.0);
    assert!(
        places
            .iter()
            .zip(places.iter().skip(1))
            .all(|(a, b)| a.passages() >= b.passages())
    );
    let summary = lens.summary().expect("summary").get(0);
    assert!(summary.contains("crossings and gates passed"), "{summary}");
    saves_and_goes_on_alike(&mut sim, DAY / 2);
}
