//! Track laid by work (M10b slice DV, second part; ADR-0034 §2; the rail brief §1.1): a line is
//! begun as a way serving wagons alone, its bed graded to its ruling grade; its keeper's people lay
//! it hour by hour, each hour taking its share of the timber from their store, and it carries
//! wagons only once it is whole. It is saved and loaded with its way, and walkers' wear never
//! holds it.

use std::path::Path;
use std::sync::OnceLock;

use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_land::ways::{RAIL, Surface};
use civ_sim::{NewWorld, Sim, persist};
use civ_world::WATER_LAND;

const PRESET: &str = "core:worldgen/river_valley";

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
            name: "Rails".to_owned(),
            seed,
            preset_id: PRESET.to_owned(),
            size_cells: 768,
            band_size: 30,
            neighbours: Vec::new(),
            neighbours_known: true,
            regime_id: String::new(),
        },
        content(),
        [9; 16],
    )
    .expect("generates")
}

/// The hearth's cell and a dry cell `n` cells from it, east or west (whichever runs dry longer).
fn ends_from_hearth(sim: &Sim, n: i64) -> (u32, u32) {
    let map = sim.map();
    let (x, y) = sim.land().settlements[0].hearth_m;
    let (cx, cy) = ((x / map.cell_size_m) as i64, (y / map.cell_size_m) as i64);
    let dry = |step: i64| {
        (0..=n)
            .map(|k| cx + step * k)
            .take_while(|&c| c >= 0 && c < i64::from(map.width))
            .map(|c| (cy * i64::from(map.width) + c) as u32)
            .take_while(|&cell| map.water[cell as usize] == WATER_LAND)
            .count()
    };
    let step = if dry(1) >= dry(-1) { 1 } else { -1 };
    let reach = (dry(step) as i64 - 1).min(n);
    assert!(reach >= 8, "a run of dry land from the hearth");
    let from = (cy * i64::from(map.width) + cx) as u32;
    (from, (cy * i64::from(map.width) + cx + step * reach) as u32)
}

fn first_household(sim: &Sim) -> PermanentId {
    sim.people()
        .households
        .iter()
        .map(|(_, h)| h.id)
        .min()
        .expect("a household")
}

fn set_timber(sim: &mut Sim, household: PermanentId, kg: f64) {
    let timber = sim
        .rules()
        .catalog
        .good_index("core:good/timber")
        .expect("timber");
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == household {
            x.stores[timber] = kg;
        }
    }
}

fn timber_held(sim: &Sim, household: PermanentId) -> f64 {
    let timber = sim
        .rules()
        .catalog
        .good_index("core:good/timber")
        .expect("timber");
    sim.people().household(household).expect("household").stores[timber]
}

/// Saves `sim`, loads it, and checks every section of the loaded world is the same.
fn saves_alike(sim: &mut Sim) {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "rails").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    let (a, b) = (
        persist::encode_sections(sim),
        persist::encode_sections(&loaded),
    );
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(&b) {
        assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
    }
    assert_eq!(sim.land().ways, loaded.land().ways);
}

#[test]
fn a_line_is_laid_by_its_keepers_work_and_timber_and_carries_wagons_once_whole() {
    let mut sim = village(3);
    let household = first_household(&sim);
    let (from, to) = ends_from_hearth(&sim, 16);
    let ruling = 0.003;
    let id = sim
        .with_ctx_for_tests(|_, ctx| {
            civ_agents::population::Population::begin_track_for_tests(
                ctx, household, from, to, 1.0, ruling,
            )
        })
        .expect("a line begun");
    let way = sim
        .land()
        .ways
        .list
        .iter()
        .find(|w| w.id == id)
        .cloned()
        .expect("the line");
    // A way serving wagons alone, of timber track on a bed two metres wide, marked out only.
    assert_eq!(way.modes, RAIL);
    let s = &way.segments[0];
    assert_eq!(s.surface, Surface::Track);
    assert_eq!((s.cells[0], *s.cells.last().expect("cells")), (from, to));
    assert!((s.corridor_m - 2.0).abs() < 1e-6, "{s:?}");
    assert!(s.usable_m < 0.1 && s.built_m < 0.1);
    let track = way.track.clone().expect("its track");
    assert_eq!(track.bed.len(), s.cells.len());
    let map = sim.map();
    let runs = civ_land::rail::runs(&s.cells, map.width, f64::from(map.cell_size_m));
    for (pair, run) in track.bed.windows(2).zip(&runs[1..]) {
        assert!(
            f64::from((pair[1] - pair[0]).abs()) <= ruling * run + 1e-4,
            "the bed keeps its ruling grade: {:?}",
            track.bed
        );
    }
    let length: f64 = runs.iter().sum();
    assert!((f64::from(track.good_kg) - 50.0 * length).abs() < 1.0);
    assert!(f64::from(track.labour_h) >= 1.5 * length - 1e-3);
    // Walkers' wear never holds it.
    let day = sim.now().day_index();
    let params = sim.rules().land.paths;
    let held = sim.land().ways.holds(day, &params);
    assert!(
        s.cells
            .iter()
            .all(|c| held.binary_search_by_key(c, |h| h.0).is_err())
    );
    // With no timber, no hours lay any of it.
    set_timber(&mut sim, household, 0.0);
    let work = |sim: &mut Sim, hours: f64| {
        sim.with_ctx_for_tests(|pop, ctx| pop.way_work_for_tests(ctx, household, id, hours));
    };
    work(&mut sim, 10.0);
    let built = |sim: &Sim| {
        let w = sim
            .land()
            .ways
            .list
            .iter()
            .find(|w| w.id == id)
            .expect("line");
        (w.segments[0].built_m, w.segments[0].usable_m)
    };
    assert!(built(&sim).0 < 0.1, "nothing laid without timber");
    // Half its timber lays half of it, and is used up.
    let half = f64::from(track.good_kg) / 2.0;
    set_timber(&mut sim, household, half);
    work(&mut sim, f64::from(track.labour_h));
    let (b, u) = built(&sim);
    assert!((b - (0.05 + 1.95 / 2.0)).abs() < 0.01, "half laid: {b}");
    assert!(u < 0.1, "not yet usable while part laid: {u}");
    assert!(
        timber_held(&sim, household) < 1e-3,
        "{}",
        timber_held(&sim, household)
    );
    // Saved and loaded part laid, it is the same line.
    saves_alike(&mut sim);
    // The observer is shown it as a wagonway of its gauge and grade, half laid.
    let (gauge, grade, laid, words) = line_seen(&sim, id);
    assert!(
        (gauge - 1.0).abs() < 1e-6 && f64::from(grade) <= ruling + 1e-4,
        "{grade}"
    );
    assert!((laid - 0.5).abs() < 0.01, "{laid}");
    assert!(
        words.starts_with("Wagonway of ")
            && words.contains("gauge 1.0 m")
            && words.contains("% laid"),
        "{words}"
    );
    let laid_entries = |sim: &Sim| {
        sim.people()
            .chronicle
            .iter()
            .filter(|e| e.kind == civ_agents::ChronicleKind::LineLaid)
            .count()
    };
    assert_eq!(laid_entries(&sim), 0);
    // The rest of its timber and hours finish it, and then it carries wagons.
    set_timber(&mut sim, household, half + 100.0);
    work(&mut sim, f64::from(track.labour_h));
    let (b, u) = built(&sim);
    assert!((b - 2.0).abs() < 1e-3 && (u - 2.0).abs() < 1e-3, "{b} {u}");
    assert!((timber_held(&sim, household) - 100.0).abs() < 1.0);
    // Finished, it is chronicled once and shown laid.
    assert_eq!(laid_entries(&sim), 1);
    work(&mut sim, 5.0);
    assert_eq!(laid_entries(&sim), 1);
    let (_, _, laid, words) = line_seen(&sim, id);
    assert!(
        (laid - 1.0).abs() < 1e-6 && words.contains(", laid;"),
        "{words}"
    );
    saves_alike(&mut sim);
}

/// Line `id` as the observer's ways layer shows it: its gauge, grade, share laid and words.
fn line_seen(sim: &Sim, id: PermanentId) -> (f32, f32, f32, String) {
    let bytes = civ_sim::frames::ways::ways_response(sim);
    let response =
        civ_schema::flatbuffers::root::<civ_schema::wire::Response>(&bytes).expect("wire");
    let way = response
        .body_as_ways()
        .expect("ways")
        .ways()
        .expect("a list")
        .iter()
        .find(|w| w.id() == id.get())
        .expect("the line");
    assert_eq!(way.modes(), RAIL);
    (
        way.gauge_m(),
        way.ruling(),
        way.laid(),
        way.words().unwrap_or_default().to_owned(),
    )
}

/// The locomotive content's index.
fn engine_vehicle(sim: &Sim) -> usize {
    sim.rules()
        .catalog
        .vehicles
        .iter()
        .position(|v| v.id == "core:vehicle/locomotive")
        .expect("the locomotive")
}

#[test]
fn runs_on_a_laid_line_are_timed_by_its_bed_meet_at_its_loop_and_are_saved_where_they_are() {
    use civ_land::trains::{self, Run, RunState, Train};
    let mut sim = village(3);
    let household = first_household(&sim);
    let (from, to) = ends_from_hearth(&sim, 16);
    let id = sim
        .with_ctx_for_tests(|_, ctx| {
            civ_agents::population::Population::begin_track_for_tests(
                ctx, household, from, to, 1.0, 0.003,
            )
        })
        .expect("a line begun");
    let way = sim
        .land()
        .ways
        .list
        .iter()
        .find(|w| w.id == id)
        .cloned()
        .expect("the line");
    let track = way.track.clone().expect("its track");
    let cells = &way.segments[0].cells;
    let map = sim.map();
    let runs = civ_land::rail::runs(cells, map.width, f64::from(map.cell_size_m));
    // The core's locomotive with 10 t behind it.
    let def = sim.rules().catalog.vehicles[engine_vehicle(&sim)].clone();
    let engine = def.engine.expect("an engine");
    let train = Train {
        power_w: def.power_w,
        pull_n: def.pull_n,
        top_m_s: engine.top_speed_m_s,
        mass_kg: def.tare_kg + 10_000.0,
        c: def.rail_resistance,
    };
    // A loop halfway: two block sections.
    let half = (cells.len() / 2) as u32;
    sim.land_mut_for_tests().runs.add_loop(id, half);
    let sections = trains::sections(sim.land().runs.loops_of(id), cells.len());
    assert_eq!(sections.len(), 2);
    let out = trains::step_seconds(&track.bed, &runs, &train, false).expect("it runs out");
    let home = trains::step_seconds(&track.bed, &runs, &train, true).expect("and back");
    // On a bed within 0.3 %, the train runs the whole way at its top speed.
    let length: f64 = runs.iter().sum();
    assert!((out.iter().sum::<f64>() - length / 2.2).abs() < 1e-6);
    let start = sim.now().minutes();
    let make = |sim: &mut Sim, back: bool, secs: &[f64]| {
        let run_id = sim.allocate_id_for_tests();
        let run = Run {
            id: run_id,
            way: id,
            back,
            engine: sim.allocate_id_for_tests(),
            owner: household,
            cars: Vec::new(),
            legs: trains::legs(secs, &sections, back),
            leg: 0,
            state: RunState::Arrived { at: 0 },
            load_kg: 5_000.0,
        };
        sim.land_mut_for_tests().runs.depart(run, start);
        run_id
    };
    let a = make(&mut sim, false, &out);
    let b = make(&mut sim, true, &home);
    // Each half of so short a line takes them a minute: setting off, each holds the section at
    // its end; a minute on, they have met at the loop and each holds the other's.
    let legs: Vec<_> = sim
        .land()
        .runs
        .list
        .iter()
        .map(|r| r.legs.clone())
        .collect();
    assert_eq!(legs, vec![vec![(0, 1), (1, 1)], vec![(1, 1), (0, 1)]]);
    sim.land_mut_for_tests().runs.advance(start);
    assert_eq!(sim.land().runs.held_by(id, 0), Some(a));
    assert_eq!(sim.land().runs.held_by(id, 1), Some(b));
    sim.advance_minutes(1).expect("advances");
    let land = sim.land();
    assert_eq!(land.runs.held_by(id, 0), Some(b));
    assert_eq!(land.runs.held_by(id, 1), Some(a));
    // Saved and loaded while both are out, both are where they were.
    saves_alike(&mut sim);
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(&mut sim, &saves, commons_persist::SaveKind::Manual, "runs").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loaded.land().runs, sim.land().runs);
    // An hour on, they have met at the loop, passed one at a time, and arrived; neither is gone.
    sim.advance_minutes(60).expect("advances");
    let runs = &sim.land().runs;
    assert_eq!(runs.list.len(), 2);
    assert!(
        runs.list
            .iter()
            .all(|r| matches!(r.state, RunState::Arrived { .. })),
        "{:?}",
        runs.list
    );
}

/// The wagon content's index.
fn wagon_vehicle(sim: &Sim) -> usize {
    sim.rules()
        .catalog
        .vehicles
        .iter()
        .position(|v| v.id == "core:vehicle/wagon")
        .expect("the wagon")
}

#[test]
fn a_wagon_on_a_line_from_the_pit_brings_home_what_the_work_digs() {
    let mut sim = village(3);
    let household = first_household(&sim);
    let home = sim.people().household(household).expect("household").home;
    let carry = sim.rules().people.household.carry_kg;
    let seam = contour_seam(&mut sim, home);
    // A session without a wagon opens the pit and brings what one person carries.
    let dig = |sim: &mut Sim, minutes: f64| {
        sim.with_ctx_for_tests(|pop, ctx| {
            pop.dig_and_bring_for_tests(ctx, household, seam, minutes)
        })
    };
    let mut by_hand = 0.0;
    for _ in 0..20 {
        by_hand = dig(&mut sim, 180.0);
        if by_hand > 0.0 {
            break;
        }
    }
    assert!(by_hand > 0.0 && by_hand <= carry + 1e-9, "{by_hand}");
    let pit = sim
        .land()
        .earthworks
        .iter()
        .find(|e| e.kind == civ_land::earth::EarthKind::Pit && e.deposit == Some(seam))
        .map(|e| e.rect.centre_m())
        .expect("a pit");
    // A whole line from beside the pit to beside home, and a wagon at its pit end.
    let cell_of = |at: (f32, f32)| civ_agents::population::cell_of(sim.map(), at) as u32;
    let (pit_cell, home_cell) = (cell_of(pit), cell_of(home));
    let id = sim
        .with_ctx_for_tests(|_, ctx| {
            civ_agents::population::Population::begin_track_for_tests(
                ctx, household, pit_cell, home_cell, 1.0, 0.003,
            )
        })
        .expect("a line from the pit home");
    let need = sim
        .land()
        .ways
        .list
        .iter()
        .find(|x| x.id == id)
        .and_then(|x| x.track.clone())
        .expect("its track");
    set_timber(&mut sim, household, f64::from(need.good_kg) + 1.0);
    sim.with_ctx_for_tests(|pop, ctx| {
        pop.way_work_for_tests(ctx, household, id, f64::from(need.labour_h) + 1.0)
    });
    let vehicle = wagon_vehicle(&sim);
    let wagon = sim.with_ctx_for_tests(|_, ctx| {
        civ_agents::population::Population::add_wagon_for_tests(ctx, household, vehicle, pit_cell)
    });
    // The observer is shown the wagon, of its own mode, standing at the pit end.
    let bytes = civ_sim::frames::vehicles::vehicles_response(&sim);
    let response =
        civ_schema::flatbuffers::root::<civ_schema::wire::Response>(&bytes).expect("wire");
    let list = response.body_as_vehicles().expect("vehicles");
    assert_eq!(list.rev(), civ_sim::frames::vehicles::vehicles_rev(&sim));
    let seen = list
        .vehicles()
        .expect("a list")
        .iter()
        .find(|v| v.id() == wagon.get())
        .expect("the wagon");
    assert_eq!((seen.mode(), seen.out_with()), (2, 0));
    let centre = civ_agents::population::cell_centre(sim.map(), pit_cell as usize);
    assert_eq!((seen.x(), seen.y()), centre);
    assert!(
        seen.words().unwrap_or_default().contains("'s wagon"),
        "{:?}",
        seen.words()
    );
    let before = civ_sim::frames::vehicles::vehicles_rev(&sim);
    // Now a session brings what its work digs, far more than a person carries, and the wagon
    // is left at the home end.
    let by_wagon = dig(&mut sim, 180.0);
    assert!(by_wagon > 2.0 * carry, "{by_wagon} against {carry} by hand");
    let at = sim
        .land()
        .wagons
        .list
        .iter()
        .find(|x| x.id == wagon)
        .expect("the wagon")
        .at;
    assert_eq!(at, home_cell);
    assert_ne!(civ_sim::frames::vehicles::vehicles_rev(&sim), before);
    // From the home end it goes out empty with the next digger and comes back loaded.
    assert!(dig(&mut sim, 180.0) > 2.0 * carry);
    // Saved and loaded, the wagon stands where it was left.
    saves_alike(&mut sim);
    // A wagon of another gauge carries nothing on this line: one person's load again.
    for x in &mut sim.land_mut_for_tests().ways.list {
        if let Some(t) = x.track.as_mut() {
            t.gauge_m = 1.5;
        }
    }
    let again = dig(&mut sim, 180.0);
    assert!(again <= carry + 1e-9, "{again}");
}

fn stores_of(sim: &Sim, household: PermanentId, id: &str) -> f64 {
    let g = sim.rules().catalog.good_index(id).expect("a good");
    sim.people().household(household).expect("household").stores[g]
}

fn set_store(sim: &mut Sim, household: PermanentId, id: &str, kg: f64) {
    let g = sim.rules().catalog.good_index(id).expect("a good");
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == household {
            x.stores[g] = kg;
        }
    }
}

#[test]
fn an_engine_brings_a_pit_s_digging_home_by_train_burning_coal_by_its_work() {
    use civ_land::trains::RunState;
    let mut sim = village(3);
    let household = first_household(&sim);
    let home = sim.people().household(household).expect("household").home;
    let carry = sim.rules().people.household.carry_kg;
    let seam = contour_seam(&mut sim, home);
    let dig = |sim: &mut Sim, minutes: f64| {
        sim.with_ctx_for_tests(|pop, ctx| {
            pop.dig_and_bring_for_tests(ctx, household, seam, minutes)
        })
    };
    for _ in 0..20 {
        if dig(&mut sim, 180.0) > 0.0 {
            break;
        }
    }
    let pit = sim
        .land()
        .earthworks
        .iter()
        .find(|e| e.kind == civ_land::earth::EarthKind::Pit && e.deposit == Some(seam))
        .map(|e| e.rect.centre_m())
        .expect("a pit");
    // A whole line from beside the pit to beside home, with an engine and a wagon at the pit.
    let cell_of = |at: (f32, f32)| civ_agents::population::cell_of(sim.map(), at) as u32;
    let (pit_cell, home_cell) = (cell_of(pit), cell_of(home));
    let id = sim
        .with_ctx_for_tests(|_, ctx| {
            civ_agents::population::Population::begin_track_for_tests(
                ctx, household, pit_cell, home_cell, 1.0, 0.003,
            )
        })
        .expect("a line from the pit home");
    let need = sim
        .land()
        .ways
        .list
        .iter()
        .find(|x| x.id == id)
        .and_then(|x| x.track.clone())
        .expect("its track");
    set_timber(&mut sim, household, f64::from(need.good_kg) + 1.0);
    sim.with_ctx_for_tests(|pop, ctx| {
        pop.way_work_for_tests(ctx, household, id, f64::from(need.labour_h) + 1.0)
    });
    let (wagon_v, engine_v) = (wagon_vehicle(&sim), engine_vehicle(&sim));
    let add = |sim: &mut Sim, v: usize| {
        sim.with_ctx_for_tests(|_, ctx| {
            civ_agents::population::Population::add_wagon_for_tests(ctx, household, v, pit_cell)
        })
    };
    let wagon = add(&mut sim, wagon_v);
    let engine = add(&mut sim, engine_v);
    // A session's digging goes onto the wagon, more than a person carries, and nobody carries
    // it home.
    assert_eq!(dig(&mut sim, 180.0), 0.0, "nothing carried");
    let aboard = |sim: &Sim| {
        sim.people()
            .consignments
            .iter()
            .filter(|c| c.boat == wagon)
            .map(|c| c.stores.iter().sum::<f64>())
            .sum::<f64>()
    };
    let loaded = aboard(&sim);
    assert!(
        loaded > 2.0 * carry,
        "{loaded} aboard against {carry} by hand"
    );
    // With no coal the engine stays.
    set_store(&mut sim, household, "core:good/coal", 0.0);
    let day = sim.now().day_index();
    let account = |sim: &mut Sim, day: i64| {
        sim.with_ctx_for_tests(|pop, ctx| pop.engine_runs_for_tests(ctx, household, day));
    };
    account(&mut sim, day);
    assert!(sim.land().runs.list.is_empty(), "no coal, no run");
    // With coal it sets off home at six, burning what its work takes: the train's weight up
    // each rise of the bed homeward and its rolling resistance, at 3 % of coal's 25.8 MJ/kg.
    set_store(&mut sim, household, "core:good/coal", 50.0);
    account(&mut sim, day);
    let run = sim.land().runs.list.first().cloned().expect("a run home");
    assert_eq!((run.engine, run.cars.clone()), (engine, vec![wagon]));
    assert!((f64::from(run.load_kg) - loaded).abs() < 1e-3);
    let way = sim
        .land()
        .ways
        .list
        .iter()
        .find(|w| w.id == id)
        .cloned()
        .expect("line");
    let cells: Vec<u32> = way.segments.iter().flat_map(|s| s.cells.clone()).collect();
    let map = sim.map();
    let runs = civ_land::rail::runs(&cells, map.width, f64::from(map.cell_size_m));
    let defs = &sim.rules().catalog.vehicles;
    let mass = defs[engine_v].tare_kg + defs[wagon_v].tare_kg + loaded;
    let mut steps: Vec<f64> = runs[1..].to_vec();
    if run.back {
        steps.reverse();
    }
    let work: f64 = civ_land::rail::step_grades(&need.bed, &runs, run.back)
        .iter()
        .zip(&steps)
        .map(|(g, r)| (mass * 9.81 * (0.005 + g)).max(0.0) * r)
        .sum();
    let burned = 50.0 - stores_of(&sim, household, "core:good/coal");
    assert!(
        (burned - work / 0.03 / 25.8e6).abs() < 1e-9,
        "{burned} burned against {work} J of work"
    );
    // Saved and loaded with the run waiting to go, it is the same run.
    saves_alike(&mut sim);
    // By an hour after six it has arrived home; the next accounting unloads the wagon into the
    // store and sends the engine back out light.
    let now = sim.now().minutes();
    let after = (day * 1_440 + 7 * 60 - now).max(1);
    sim.advance_minutes(after).expect("advances");
    assert!(matches!(
        sim.land().runs.list.first().map(|r| r.state),
        Some(RunState::Arrived { .. })
    ));
    let coal_before = stores_of(&sim, household, "core:good/coal");
    account(&mut sim, day);
    let at = |sim: &Sim, id| {
        sim.land()
            .wagons
            .list
            .iter()
            .find(|w| w.id == id)
            .expect("there")
            .at
    };
    assert_eq!((at(&sim, engine), at(&sim, wagon)), (home_cell, home_cell));
    assert!(aboard(&sim) < 1e-9, "unloaded");
    let coal_after = stores_of(&sim, household, "core:good/coal");
    assert!(
        coal_after > coal_before + loaded - 5.0,
        "the load came into the store: {coal_before} -> {coal_after}"
    );
    let out = sim
        .land()
        .runs
        .list
        .first()
        .cloned()
        .expect("a run back out");
    assert!(out.back != run.back && out.load_kg == 0.0);
    // Out at the pit end again, it waits there for the next digging.
    sim.advance_minutes(60).expect("advances");
    account(&mut sim, day);
    assert_eq!((at(&sim, engine), at(&sim, wagon)), (pit_cell, pit_cell));
    assert!(sim.land().runs.list.is_empty(), "nothing to bring");
}

/// Coal showing about 100 m from `home` along ground a pushed wagonway can keep to: the first
/// direction round from east in which a line at a 0.3 % ruling grade joins the two.
fn contour_seam(sim: &mut Sim, home: (f32, f32)) -> PermanentId {
    let params = sim.rules().land.track.expect("the core lays track");
    let along = {
        let map = sim.map();
        let decks = sim.land().cart_decks(map);
        let t = sim.land().cart_terrain(map, &decks);
        let home_cell = civ_agents::population::cell_of(map, home) as u32;
        (0..16)
            .map(|k| f32::from(k as u8) * std::f32::consts::PI / 8.0)
            .map(|a| (home.0 + 100.0 * a.cos(), home.1 + 100.0 * a.sin()))
            .find(|&at| {
                let cell = civ_agents::population::cell_of(map, at) as u32;
                t.plan_track(home_cell, cell, 1.0, 0.003, &params, 200_000)
                    .is_some()
            })
            .expect("a direction a line can keep to")
    };
    sim.place_deposit(
        (
            home.0 + (along.0 - home.0) * 1.12,
            home.1 + (along.1 - home.1) * 1.12,
        ),
        "core:good/coal",
        12.0,
        true,
    )
    .expect("laid down")
}

#[test]
fn a_household_that_digs_often_weighs_a_line_home_and_begins_it_and_its_wagon() {
    let mut sim = village(3);
    let household = first_household(&sim);
    let home = sim.people().household(household).expect("household").home;
    let seam = contour_seam(&mut sim, home);
    // A session opens the pit.
    for _ in 0..20 {
        let kg = sim.with_ctx_for_tests(|pop, ctx| {
            pop.dig_and_bring_for_tests(ctx, household, seam, 180.0)
        });
        if kg > 0.0 {
            break;
        }
    }
    let reckon = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| pop.line_reckoning_for_tests(ctx, household))
    };
    // Digging there no day yet: nothing to weigh.
    assert_eq!(reckon(&mut sim), None);
    // It has dug there about a day a week, as it holds its dig days (fading).
    let half = sim.rules().people.places.use_half_life_days;
    let held = |per_year: f64| (per_year * half / (365.0 * std::f64::consts::LN_2)) as f32;
    let set_days = |sim: &mut Sim, per_year: f64| {
        let day = sim.now().day_index();
        let place = civ_agents::uses::Place::Deposit(seam);
        let list = sim
            .people_mut_for_tests()
            .uses
            .households
            .entry(household)
            .or_default();
        list.retain(|u| u.place != place);
        list.push(civ_agents::uses::PlaceUse {
            place,
            days: held(per_year),
            kcal: 0.0,
            outsiders: Vec::new(),
            seen: Vec::new(),
            day,
            recent: 0,
        });
        list.sort_by_key(|a| a.place);
    };
    set_days(&mut sim, 52.0);
    let r = reckon(&mut sim).expect("a line weighed");
    assert_eq!(r.deposit, seam);
    // Each dig day spares most of its session and walks: one person carries 20 kg, a wagon what
    // the work digs.
    assert!(r.saves_h_year > 52.0, "{r:?}");
    assert!(r.cost_h > f64::from(r.track.labour_h), "{r:?}");
    // Not knowing how to make a wagon, it begins nothing.
    sim.with_ctx_for_tests(|pop, ctx| pop.review_line_for_tests(ctx, household));
    assert!(sim.land().ways.list.is_empty());
    assert!(sim.land().wagons.list.is_empty());
    // Knowing wooden railways and holding the wagon's timber, it marks out the line and begins
    // the wagon at its home end, when the line repays itself over the wagon's life.
    let railway = sim
        .rules()
        .catalog
        .techniques
        .iter()
        .position(|t| t.id == "core:technique/wooden_railway")
        .expect("wooden railways");
    let learn_h = sim.rules().catalog.techniques[railway].learn_h;
    let members = sim
        .people()
        .household(household)
        .expect("household")
        .members
        .clone();
    let now = sim.now();
    for (_, p) in sim.people_mut_for_tests().people.iter_mut() {
        if members.contains(&p.id) {
            p.learn(railway, learn_h, members[0], learn_h, now);
        }
    }
    set_timber(&mut sim, household, 1_000.0);
    let life = sim.rules().catalog.vehicles[wagon_vehicle(&sim)].life_years;
    // A day a week does not repay a line's 1,900 hours and a wagon's 120 over its 15 years.
    assert!(r.saves_h_year * life < r.cost_h, "{r:?}");
    sim.with_ctx_for_tests(|pop, ctx| pop.review_line_for_tests(ctx, household));
    assert!(sim.land().ways.list.is_empty());
    // Two days a week does.
    set_days(&mut sim, 104.0);
    let r = reckon(&mut sim).expect("a line weighed");
    assert!(r.saves_h_year * life > r.cost_h, "{r:?}");
    sim.with_ctx_for_tests(|pop, ctx| pop.review_line_for_tests(ctx, household));
    let line = sim.land().ways.list.first().cloned().expect("a line begun");
    assert_eq!(line.modes, RAIL);
    let (_, home_end) = line.ends().expect("ends");
    assert_eq!(home_end, *r.cells.last().expect("cells"));
    let wagon = sim
        .land()
        .wagons
        .list
        .first()
        .cloned()
        .expect("a wagon begun");
    assert_eq!((wagon.owner, wagon.at), (household, home_end));
    assert!(!wagon.made() && wagon.labour_h > 0.0);
    // Its timber came out of the store.
    assert!((timber_held(&sim, household) - 700.0).abs() < 1e-6);
    // Weighed again, nothing more is begun.
    sim.with_ctx_for_tests(|pop, ctx| pop.review_line_for_tests(ctx, household));
    assert_eq!(
        (sim.land().ways.list.len(), sim.land().wagons.list.len()),
        (1, 1)
    );
    saves_alike(&mut sim);
}

/// A whole line laid from beside a pit at `seam`, opened by a session's digging, to beside
/// household `household`'s home at `home`: its id and its pit and home cells.
fn laid_line(
    sim: &mut Sim,
    household: PermanentId,
    home: (f32, f32),
    seam: PermanentId,
) -> (PermanentId, u32, u32) {
    for _ in 0..20 {
        let kg = sim.with_ctx_for_tests(|pop, ctx| {
            pop.dig_and_bring_for_tests(ctx, household, seam, 180.0)
        });
        if kg > 0.0 {
            break;
        }
    }
    let pit = sim
        .land()
        .earthworks
        .iter()
        .find(|e| e.kind == civ_land::earth::EarthKind::Pit && e.deposit == Some(seam))
        .map(|e| e.rect.centre_m())
        .expect("a pit");
    let cell_of = |at: (f32, f32)| civ_agents::population::cell_of(sim.map(), at) as u32;
    let (pit_cell, home_cell) = (cell_of(pit), cell_of(home));
    let id = sim
        .with_ctx_for_tests(|_, ctx| {
            civ_agents::population::Population::begin_track_for_tests(
                ctx, household, pit_cell, home_cell, 1.0, 0.003,
            )
        })
        .expect("a line from the pit home");
    let need = sim
        .land()
        .ways
        .list
        .iter()
        .find(|x| x.id == id)
        .and_then(|x| x.track.clone())
        .expect("its track");
    set_timber(sim, household, f64::from(need.good_kg) + 1.0);
    sim.with_ctx_for_tests(|pop, ctx| {
        pop.way_work_for_tests(ctx, household, id, f64::from(need.labour_h) + 1.0)
    });
    (id, pit_cell, home_cell)
}

/// Household `household`'s dig days at deposit `seam` set as if it dug there `per_year` days a
/// year (they fade, so it holds fewer).
fn set_dig_days(sim: &mut Sim, household: PermanentId, seam: PermanentId, per_year: f64) {
    let half = sim.rules().people.places.use_half_life_days;
    let held = (per_year * half / (365.0 * std::f64::consts::LN_2)) as f32;
    let day = sim.now().day_index();
    let place = civ_agents::uses::Place::Deposit(seam);
    let list = sim
        .people_mut_for_tests()
        .uses
        .households
        .entry(household)
        .or_default();
    list.retain(|u| u.place != place);
    list.push(civ_agents::uses::PlaceUse {
        place,
        days: held,
        kcal: 0.0,
        outsiders: Vec::new(),
        seen: Vec::new(),
        day,
        recent: 0,
    });
    list.sort_by_key(|a| a.place);
}

/// Every member of household `household` learns technique `id`.
fn teach(sim: &mut Sim, household: PermanentId, id: &str) {
    let t = sim
        .rules()
        .catalog
        .techniques
        .iter()
        .position(|t| t.id == id)
        .expect("a technique");
    let learn_h = sim.rules().catalog.techniques[t].learn_h;
    let members = sim
        .people()
        .household(household)
        .expect("household")
        .members
        .clone();
    let now = sim.now();
    for (_, p) in sim.people_mut_for_tests().people.iter_mut() {
        if members.contains(&p.id) {
            p.learn(t, learn_h, members[0], learn_h, now);
        }
    }
}

#[test]
fn a_household_whose_pushed_loads_fall_short_of_its_digging_weighs_an_engine_and_begins_one() {
    let mut sim = village(3);
    let household = first_household(&sim);
    let home = sim.people().household(household).expect("household").home;
    let seam = contour_seam(&mut sim, home);
    let (_, pit_cell, _) = laid_line(&mut sim, household, home, seam);
    let (wagon_v, engine_v) = (wagon_vehicle(&sim), engine_vehicle(&sim));
    let add = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|_, ctx| {
            civ_agents::population::Population::add_wagon_for_tests(
                ctx, household, wagon_v, pit_cell,
            )
        })
    };
    for _ in 0..4 {
        add(&mut sim);
    }
    set_store(&mut sim, household, "core:good/iron", 3_200.0);
    set_dig_days(&mut sim, household, seam, 200.0);
    let reckon = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| pop.engine_reckoning_for_tests(ctx, household))
    };
    // The core's wagon holds a ton or two, more than a session digs, so one person pushes home
    // all the work digs and an engine would spare nothing.
    assert_eq!(reckon(&mut sim), None);
    // With tubs of 100 kg, a session digs more than one is pushed home with: a train of four
    // brings it all, sparing the share of each dig day's hours the push leaves undug.
    sim.rules_mut_for_tests().expect("rules").catalog.vehicles[wagon_v].payload_kg = 100.0;
    let r = reckon(&mut sim).expect("an engine weighed");
    assert!(r.saves_h_year > 0.0 && r.coal_h_year > 0.0, "{r:?}");
    // Its coal is dug at the pit it serves, so costs a trifle; it holds the iron and cannot
    // make it, so the engine costs its labour.
    assert!(r.coal_h_year < 0.01 * r.saves_h_year, "{r:?}");
    let def = sim.rules().catalog.vehicles[engine_v].clone();
    assert!((r.cost_h - def.labour_h).abs() < 1e-9, "{r:?}");
    assert_eq!(r.pit_end, pit_cell);
    // The hours it spares go with the dig days.
    set_dig_days(&mut sim, household, seam, 100.0);
    let half = reckon(&mut sim).expect("weighed");
    assert!(
        (half.saves_h_year - r.saves_h_year / 2.0).abs() < 0.01 * r.saves_h_year,
        "{half:?} against {r:?}"
    );
    // Knowing wooden railways but not the locomotive, its line review begins nothing.
    teach(&mut sim, household, "core:technique/wooden_railway");
    let review = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| pop.review_line_for_tests(ctx, household));
    };
    let engines = |sim: &Sim| {
        sim.land()
            .wagons
            .list
            .iter()
            .filter(|w| usize::from(w.vehicle) == engine_v)
            .cloned()
            .collect::<Vec<_>>()
    };
    set_dig_days(&mut sim, household, seam, 200.0);
    review(&mut sim);
    assert!(engines(&sim).is_empty());
    // Knowing it, a day a week does not repay its 3,000 hours over its 20 years.
    teach(&mut sim, household, "core:technique/locomotive");
    set_dig_days(&mut sim, household, seam, 52.0);
    let few = reckon(&mut sim).expect("weighed");
    assert!(
        (few.saves_h_year - few.coal_h_year) * def.life_years < few.cost_h,
        "{few:?}"
    );
    review(&mut sim);
    assert!(engines(&sim).is_empty());
    // Two hundred do: it begins an engine at the pit end of its line, its iron from the store.
    set_dig_days(&mut sim, household, seam, 200.0);
    let r = reckon(&mut sim).expect("weighed");
    assert!((r.saves_h_year - r.coal_h_year) * def.life_years > r.cost_h);
    review(&mut sim);
    let begun = engines(&sim);
    assert_eq!(begun.len(), 1, "an engine begun");
    let e = &begun[0];
    assert_eq!((e.owner, e.at), (household, pit_cell));
    assert!(!e.made() && (f64::from(e.labour_h) - def.labour_h).abs() < 1e-3);
    assert!((f64::from(e.saves_h_year) - r.saves_h_year).abs() < 1e-3);
    assert!((stores_of(&sim, household, "core:good/iron") - 200.0).abs() < 1e-6);
    // Weighed again, nothing more is begun.
    review(&mut sim);
    assert_eq!(engines(&sim).len(), 1);
    // Saved and loaded while it is being made, it stands as it was.
    saves_alike(&mut sim);
    // Its people work on it as on a wagon; once its labour is done it is made where it stands,
    // and the chronicle says so.
    let id = e.id;
    let work = |sim: &mut Sim, hours: f64| {
        sim.with_ctx_for_tests(|pop, ctx| pop.vehicle_work_for_tests(ctx, household, id, hours));
    };
    work(&mut sim, def.labour_h / 2.0);
    assert!(!engines(&sim)[0].made());
    work(&mut sim, def.labour_h);
    let made = engines(&sim)[0].clone();
    assert!(made.made() && made.at == pit_cell, "{made:?}");
    assert!(
        sim.people().chronicle.iter().any(|c| {
            c.kind == civ_agents::ChronicleKind::CartMade && c.name.contains("pit end of its line")
        }),
        "the chronicle tells it"
    );
}

/// The vehicles layer's entry for vehicle `id`: its mode, run, place and words.
fn seen_vehicle(sim: &Sim, id: PermanentId) -> (u8, u8, (f32, f32), String) {
    let bytes = civ_sim::frames::vehicles::vehicles_response(sim);
    let response =
        civ_schema::flatbuffers::root::<civ_schema::wire::Response>(&bytes).expect("wire");
    let list = response.body_as_vehicles().expect("vehicles");
    let v = list
        .vehicles()
        .expect("a list")
        .iter()
        .find(|v| v.id() == id.get())
        .expect("the vehicle");
    (
        v.mode(),
        v.run(),
        (v.x(), v.y()),
        v.words().unwrap_or_default().to_owned(),
    )
}

#[test]
fn the_observer_shows_an_engine_as_its_own_mode_and_its_train_moving_along_the_line() {
    use civ_land::trains::{Run, RunState};
    let mut sim = village(3);
    let household = first_household(&sim);
    let (from, to) = ends_from_hearth(&sim, 16);
    let id = sim
        .with_ctx_for_tests(|_, ctx| {
            civ_agents::population::Population::begin_track_for_tests(
                ctx, household, from, to, 1.0, 0.003,
            )
        })
        .expect("a line begun");
    let cells: Vec<u32> = sim
        .land()
        .ways
        .list
        .iter()
        .find(|w| w.id == id)
        .expect("the line")
        .segments
        .iter()
        .flat_map(|s| s.cells.clone())
        .collect();
    let (first, last) = (cells[0], *cells.last().expect("cells"));
    let (wagon_v, engine_v) = (wagon_vehicle(&sim), engine_vehicle(&sim));
    let add = |sim: &mut Sim, v: usize| {
        sim.with_ctx_for_tests(|_, ctx| {
            civ_agents::population::Population::add_wagon_for_tests(ctx, household, v, first)
        })
    };
    let wagon = add(&mut sim, wagon_v);
    let engine = add(&mut sim, engine_v);
    let centre = |cell: u32| civ_agents::population::cell_centre(sim.map(), cell as usize);
    let (start, end) = (centre(first), centre(last));
    // Standing at the line's end, an engine is shown as an engine, a wagon as a wagon, neither on
    // a run.
    let (mode, run, at, words) = seen_vehicle(&sim, engine);
    assert_eq!((mode, run, at), (3, 0, start), "{words}");
    assert!(words.contains("'s locomotive"), "{words}");
    assert_eq!(seen_vehicle(&sim, wagon).0, 2);
    let before = civ_sim::frames::vehicles::vehicles_rev(&sim);
    // A run of ten minutes along the whole line, with the wagon behind.
    let now = sim.now().minutes();
    let run_id = sim.allocate_id_for_tests();
    sim.land_mut_for_tests().runs.depart(
        Run {
            id: run_id,
            way: id,
            back: false,
            engine,
            owner: household,
            cars: vec![wagon],
            legs: vec![(0, 10)],
            leg: 0,
            state: RunState::Arrived { at: 0 },
            load_kg: 380.0,
        },
        now,
    );
    sim.land_mut_for_tests().runs.advance(now);
    let (_, run, at, words) = seen_vehicle(&sim, engine);
    assert_eq!((run, at), (2, start), "{words}");
    assert!(
        words.ends_with("; running home with a wagon, 380 kg aboard"),
        "{words}"
    );
    assert_ne!(civ_sim::frames::vehicles::vehicles_rev(&sim), before);
    // Halfway through its leg it is about halfway along, the wagon a wagon's length behind.
    let moving = civ_sim::frames::vehicles::vehicles_rev(&sim);
    sim.advance_minutes(5).expect("advances");
    let (_, run, (x, y), _) = seen_vehicle(&sim, engine);
    assert_eq!(run, 2);
    let from_start = f64::from((x - start.0).hypot(y - start.1));
    let from_end = f64::from((x - end.0).hypot(y - end.1));
    assert!(
        (from_start - from_end).abs() < 0.2 * (from_start + from_end),
        "{from_start} m from the start and {from_end} from the end"
    );
    let (_, car_run, (cx, cy), car_words) = seen_vehicle(&sim, wagon);
    assert_eq!(car_run, 2);
    assert!(
        car_words.contains("; in a train running home"),
        "{car_words}"
    );
    let gap = f64::from((cx - x).hypot(cy - y));
    assert!(gap > 2.0 && gap <= 4.0 + 1e-3, "{gap}");
    assert!(f64::from((cx - start.0).hypot(cy - start.1)) < from_start);
    assert_ne!(civ_sim::frames::vehicles::vehicles_rev(&sim), moving);
    // Its leg run, it has arrived at the line's far end.
    sim.advance_minutes(6).expect("advances");
    let (_, run, at, words) = seen_vehicle(&sim, engine);
    assert_eq!((run, at), (3, end), "{words}");
    assert!(words.ends_with("; arrived home"), "{words}");
}

#[test]
fn a_neighbour_s_digging_goes_home_on_another_household_s_train_for_a_tenth_in_freight() {
    use civ_land::trains::RunState;
    let mut sim = village(3);
    let keeper = first_household(&sim);
    let (home, settlement) = {
        let x = sim.people().household(keeper).expect("household");
        (x.home, x.settlement)
    };
    // A neighbour of the same settlement, living at the keeper's door.
    let neighbour = sim
        .people()
        .households
        .iter()
        .map(|(_, x)| x)
        .find(|x| x.id != keeper && x.settlement == settlement && !x.members.is_empty())
        .map(|x| x.id)
        .expect("a neighbour");
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == neighbour {
            x.home = home;
        }
    }
    let carry = sim.rules().people.household.carry_kg;
    let seam = contour_seam(&mut sim, home);
    let (_, pit_cell, home_cell) = laid_line(&mut sim, keeper, home, seam);
    let dig = |sim: &mut Sim, who: PermanentId| {
        sim.with_ctx_for_tests(|pop, ctx| pop.dig_and_bring_for_tests(ctx, who, seam, 180.0))
    };
    // With no engine on the line, the neighbour carries what one person carries.
    let by_hand = dig(&mut sim, neighbour);
    assert!(by_hand > 0.0 && by_hand <= carry + 1e-9, "{by_hand}");
    // With the keeper's engine and wagon at the pit end, the neighbour's digging goes onto the
    // wagon as its own goods, far more than a carry, and nothing is carried home.
    let (wagon_v, engine_v) = (wagon_vehicle(&sim), engine_vehicle(&sim));
    let add = |sim: &mut Sim, v: usize| {
        sim.with_ctx_for_tests(|_, ctx| {
            civ_agents::population::Population::add_wagon_for_tests(ctx, keeper, v, pit_cell)
        })
    };
    let wagon = add(&mut sim, wagon_v);
    let engine = add(&mut sim, engine_v);
    assert_eq!(dig(&mut sim, neighbour), 0.0, "nothing carried");
    let theirs = |sim: &Sim| {
        sim.people()
            .consignments
            .iter()
            .filter(|c| c.boat == wagon && c.owner == neighbour)
            .map(|c| c.stores.iter().sum::<f64>())
            .sum::<f64>()
    };
    let loaded = theirs(&sim);
    assert!(
        loaded * 0.9 > 2.0 * carry,
        "{loaded} aboard against {carry} by hand"
    );
    // The keeper's train takes it home with its own coal; at home the neighbour's goods go into
    // its store less a tenth, which the keeper keeps as freight.
    set_store(&mut sim, keeper, "core:good/coal", 50.0);
    set_store(&mut sim, neighbour, "core:good/coal", 0.0);
    let day = sim.now().day_index();
    let account = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| pop.engine_runs_for_tests(ctx, keeper, day));
    };
    account(&mut sim);
    let run = sim.land().runs.list.first().cloned().expect("a run home");
    assert_eq!((run.engine, run.cars.clone()), (engine, vec![wagon]));
    assert!((f64::from(run.load_kg) - loaded).abs() < 1e-3);
    let now = sim.now().minutes();
    sim.advance_minutes((day * 1_440 + 7 * 60 - now).max(1))
        .expect("advances");
    assert!(matches!(
        sim.land().runs.list.first().map(|r| r.state),
        Some(RunState::Arrived { .. })
    ));
    let keeper_before = stores_of(&sim, keeper, "core:good/coal");
    let neighbour_before = stores_of(&sim, neighbour, "core:good/coal");
    account(&mut sim);
    assert!(theirs(&sim) < 1e-9, "unloaded");
    let got = stores_of(&sim, neighbour, "core:good/coal") - neighbour_before;
    let kept = stores_of(&sim, keeper, "core:good/coal") - keeper_before;
    assert!((got - 0.9 * loaded).abs() < 1.0, "{got} of {loaded}");
    // The keeper's store gains the freight, less what it burns sending the engine back out.
    assert!(
        kept > 0.1 * loaded - 1.0 && kept < 0.1 * loaded + 1e-6,
        "{kept}"
    );
    // It is counted as freight, apart from the goods carried home.
    let coal = sim
        .rules()
        .catalog
        .good_index("core:good/coal")
        .expect("coal");
    let moved = |c| sim.people().transfers.get(c, coal);
    let freight = moved(civ_agents::ledger::Channel::Freight);
    assert!((freight - 0.1 * loaded).abs() < 1e-6, "{freight}");
    assert!(moved(civ_agents::ledger::Channel::Carried) >= 0.9 * loaded - 1e-6);
    let at = sim
        .land()
        .wagons
        .list
        .iter()
        .find(|w| w.id == wagon)
        .expect("the wagon")
        .at;
    assert_eq!(at, home_cell);
}

#[test]
fn a_line_opens_stations_with_its_first_run_keeps_goods_waiting_and_runs_by_its_timetable() {
    use civ_land::trains::RunState;
    let mut sim = village(3);
    let household = first_household(&sim);
    let home = sim.people().household(household).expect("household").home;
    let carry = sim.rules().people.household.carry_kg;
    let seam = contour_seam(&mut sim, home);
    let (way, pit_cell, home_cell) = laid_line(&mut sim, household, home, seam);
    let (wagon_v, engine_v) = (wagon_vehicle(&sim), engine_vehicle(&sim));
    let add = |sim: &mut Sim, v: usize| {
        sim.with_ctx_for_tests(|_, ctx| {
            civ_agents::population::Population::add_wagon_for_tests(ctx, household, v, pit_cell)
        })
    };
    let wagon = add(&mut sim, wagon_v);
    let engine = add(&mut sim, engine_v);
    let dig = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| pop.dig_and_bring_for_tests(ctx, household, seam, 180.0))
    };
    let goods_on = |sim: &Sim, boat: PermanentId| {
        sim.people()
            .consignments
            .iter()
            .filter(|c| c.boat == boat)
            .map(|c| c.stores.iter().sum::<f64>())
            .sum::<f64>()
    };
    // No station before the first run: digging goes onto the wagon at the pit.
    assert_eq!(dig(&mut sim), 0.0);
    let first_load = goods_on(&sim, wagon);
    assert!(first_load > 2.0 * carry);
    assert!(sim.land().runs.line(way).is_none_or(|l| !l.stations));
    set_store(&mut sim, household, "core:good/coal", 50.0);
    let day = sim.now().day_index();
    sim.with_ctx_for_tests(|pop, ctx| pop.engine_runs_for_tests(ctx, household, day));
    // The first run opens the line's ends as stations, and the chronicle says so.
    let line = sim.land().runs.line(way).cloned().expect("a line record");
    assert!(line.stations && line.runs_a_day == 1, "{line:?}");
    assert!(
        sim.people()
            .chronicle
            .iter()
            .any(|c| c.kind == civ_agents::ChronicleKind::StationsOpened),
        "the chronicle tells it"
    );
    // With the train away, a session's digging waits at the pit station, nothing carried.
    assert_eq!(dig(&mut sim), 0.0);
    let waiting = goods_on(&sim, way);
    assert!(waiting > 2.0 * carry, "{waiting}");
    // Home, the train is unloaded and what it brought is tallied on the line; it goes back out
    // light, and at the pit end what waited at the station goes aboard for the next run home.
    let now = sim.now().minutes();
    sim.advance_minutes((day * 1_440 + 7 * 60 - now).max(1))
        .expect("advances");
    sim.with_ctx_for_tests(|pop, ctx| pop.engine_runs_for_tests(ctx, household, day));
    let tallied = sim.land().runs.line(way).expect("line").freight_kg;
    assert!((f64::from(tallied) - first_load).abs() < 1e-2, "{tallied}");
    sim.advance_minutes(60).expect("advances");
    sim.with_ctx_for_tests(|pop, ctx| pop.engine_runs_for_tests(ctx, household, day));
    let run = sim
        .land()
        .runs
        .list
        .iter()
        .find(|r| r.engine == engine)
        .cloned()
        .expect("a run home");
    assert!((f64::from(run.load_kg) - waiting).abs() < 1e-3, "{run:?}");
    assert!(goods_on(&sim, way) < 1e-9, "nothing left waiting");
    // At its review the timetable is set from the freight brought: one run a day for a little,
    // the most for a great deal.
    let review = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| pop.review_timetable_for_tests(ctx, household));
    };
    review(&mut sim);
    let line = sim.land().runs.line(way).cloned().expect("line");
    assert_eq!((line.runs_a_day, line.freight_kg), (1, 0.0));
    sim.land_mut_for_tests().runs.line_mut(way).freight_kg = 1.0e7;
    review(&mut sim);
    assert_eq!(sim.land().runs.line(way).expect("line").runs_a_day, 4);
    // Saved and loaded, the line's stations and timetable are as they were.
    saves_alike(&mut sim);
    // Left to the clock, its trains run by their timetable: by the next evening the run home
    // has been brought to account and the waiting goods are in the store.
    let coal_before = stores_of(&sim, household, "core:good/coal");
    let next = (day + 1) * 1_440 + 20 * 60;
    let now = sim.now().minutes();
    sim.advance_minutes(next - now).expect("advances");
    let coal_after = stores_of(&sim, household, "core:good/coal");
    assert!(
        coal_after > coal_before + waiting - 10.0,
        "{coal_before} -> {coal_after} of {waiting}"
    );
    let at = sim
        .land()
        .wagons
        .list
        .iter()
        .find(|w| w.id == engine)
        .expect("the engine")
        .at;
    assert!(at == pit_cell || at == home_cell);
    assert!(
        sim.land().runs.list.iter().all(|r| r.engine != engine
            || !matches!(r.state, RunState::Arrived { .. })
            || r.load_kg == 0.0),
        "no loaded run left standing"
    );
}

#[test]
fn a_line_the_gathering_asks_for_is_laid_by_the_households_shares_from_a_pit_to_the_hearth() {
    use civ_agents::polity::{IssueKind, LawStatus, Outcome, PolicyKind};
    let mut sim = village(3);
    // The settlement's polity is founded at its first midnight.
    sim.advance_minutes(24 * 60).expect("advances");
    let first = first_household(&sim);
    let settlement = sim
        .people()
        .household(first)
        .and_then(|x| x.settlement)
        .expect("a settlement");
    let hearth = sim
        .land()
        .settlements
        .iter()
        .find(|s| s.id == settlement)
        .expect("the settlement")
        .hearth_m;
    let pi = sim
        .people()
        .polities
        .iter()
        .position(|p| p.holds(Some(settlement)))
        .expect("its polity");
    // Coal shows where a line from the hearth can keep its grade, and two households dig it
    // most days.
    let seam = contour_seam(&mut sim, hearth);
    let second = sim
        .people()
        .households
        .iter()
        .map(|(_, x)| x)
        .find(|x| x.id != first && x.settlement == Some(settlement) && !x.members.is_empty())
        .map(|x| x.id)
        .expect("a second household");
    for _ in 0..20 {
        let kg =
            sim.with_ctx_for_tests(|pop, ctx| pop.dig_and_bring_for_tests(ctx, first, seam, 180.0));
        if kg > 0.0 {
            break;
        }
    }
    for h in [first, second] {
        set_dig_days(&mut sim, h, seam, 200.0);
        set_timber(&mut sim, h, 5_000.0);
    }
    let public = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| pop.public_line_for_tests(ctx, settlement))
    };
    // Nobody knows how to make a wagon: no line is weighed.
    assert_eq!(public(&mut sim), None);
    teach(&mut sim, first, "core:technique/wooden_railway");
    // Now the settlement's people could lay one from the pit to the hearth, sparing both.
    let (cell, labour_h, saves) = public(&mut sim).expect("a line to lay together");
    let pit = sim
        .land()
        .earthworks
        .iter()
        .find(|e| e.kind == civ_land::earth::EarthKind::Pit && e.deposit == Some(seam))
        .map(|e| e.rect.centre_m())
        .expect("a pit");
    assert_eq!(cell, civ_agents::population::cell_of(sim.map(), pit) as u32);
    assert!(labour_h > 100.0, "{labour_h}");
    let mut diggers: Vec<PermanentId> = saves.iter().map(|s| s.0).collect();
    diggers.sort_unstable();
    let mut both = vec![first, second];
    both.sort_unstable();
    assert_eq!(diggers, both, "{saves:?}");
    // A law to lay it would bring the diggers more than a household that does not dig there.
    let gains = sim.with_ctx_for_tests(|pop, ctx| {
        pop.site_gains_for_tests(ctx, settlement, PolicyKind::BuildLine, cell)
    });
    let gain = |h: PermanentId| gains.iter().find(|g| g.0 == h).map_or(0.0, |g| g.1);
    assert!(
        gains
            .iter()
            .filter(|g| g.0 != first && g.0 != second)
            .all(|g| g.1 < gain(first)),
        "{gains:?}"
    );
    // Someone proposes that the polity lay it, for they carry home what they dig there.
    sim.people_mut_for_tests().every_adult_deliberates = true;
    let mut proposed = None;
    for _ in 0..12 {
        {
            let pop = sim.people_mut_for_tests();
            pop.polities[pi].gathering = None;
            for l in &mut pop.polities[pi].laws {
                if l.status == LawStatus::Proposed {
                    l.status = LawStatus::Rejected;
                }
            }
        }
        sim.with_ctx_for_tests(|pop, ctx| pop.review_for_tests(ctx, settlement));
        match sim.people().polities[pi].agenda().cloned() {
            Some(l) if l.kind == PolicyKind::BuildLine => {
                proposed = Some(l);
                break;
            }
            Some(_) => continue,
            None => break,
        }
    }
    let law = proposed.expect("a line proposed");
    assert_eq!(law.issue, IssueKind::LoadsCarried);
    let id = law.id;
    let site = sim.people().polities[pi]
        .way_site_of(id)
        .expect("its site recorded");
    assert_eq!((site.cell, site.work), (cell, None));
    assert!(sim.people().polities[pi].way_saves_of(id, second) > 0.0);
    // Passed and known, the polity marks the line out from the pit to the hearth.
    let today = sim.now().day_index();
    let adults: Vec<PermanentId> = sim
        .people()
        .people
        .iter()
        .filter(|(_, p)| {
            p.age_years(sim.now()) >= 16.0
                && sim
                    .people()
                    .household(p.household)
                    .is_some_and(|x| x.settlement == Some(settlement))
        })
        .map(|(_, p)| p.id)
        .collect();
    {
        let pop = sim.people_mut_for_tests();
        pop.every_adult_deliberates = false;
        let p = &mut pop.polities[pi];
        p.gathering = None;
        let l = p.laws.iter_mut().find(|l| l.id == id).expect("the law");
        l.status = LawStatus::InForce;
        l.outcome = Some(Outcome::Passed);
        l.known = adults.iter().map(|&a| (a, today)).collect();
    }
    let words = sim.people().polities[pi].words_of(
        &sim.people().polities[pi]
            .laws
            .iter()
            .find(|l| l.id == id)
            .expect("the law")
            .clone(),
        &sim.rules().catalog.policies,
        &|_| String::new(),
    );
    assert!(words.contains("line of track laid together"), "{words}");
    sim.with_ctx_for_tests(|pop, ctx| pop.begin_public_line_for_tests(ctx, pi, id));
    let polity = sim.people().polities[pi].id;
    let line = sim
        .land()
        .ways
        .list
        .iter()
        .find(|w| {
            w.modes & RAIL != 0
                && w.segments
                    .iter()
                    .all(|s| s.keeper == civ_land::ways::Keeper::Polity(polity))
        })
        .cloned()
        .expect("the polity's line marked out");
    let hearth_cell = civ_agents::population::cell_of(sim.map(), hearth) as u32;
    assert_eq!(line.ends(), Some((cell, hearth_cell)));
    assert_eq!(
        sim.people().polities[pi]
            .way_site_of(id)
            .and_then(|s| s.work),
        Some(line.id)
    );
    // Saved and loaded while it is marked out, the world goes on alike.
    saves_alike(&mut sim);
    // The households' shares lay it, each from its own timber, and the last length is
    // chronicled; what each gave is counted toward its share.
    let track = line.track.clone().expect("its track");
    for h in [first, second] {
        sim.with_ctx_for_tests(|pop, ctx| {
            pop.way_work_for_tests(ctx, h, line.id, f64::from(track.labour_h) / 2.0 + 1.0)
        });
    }
    let laid = sim
        .land()
        .ways
        .list
        .iter()
        .find(|w| w.id == line.id)
        .cloned()
        .expect("the line");
    assert!(
        laid.segments
            .iter()
            .all(|s| s.built_m + 1e-3 >= s.corridor_m),
        "{laid:?}"
    );
    let given = |h| sim.people().polities[pi].way_given_by(id, h);
    assert!(given(first) > 0.0 && given(second) > 0.0);
    assert!(
        sim.people().chronicle.iter().any(|c| {
            c.kind == civ_agents::ChronicleKind::LineLaid && c.name.contains("laid together")
        }),
        "the chronicle tells it"
    );
    // With it laid, no other line is weighed there.
    assert_eq!(public(&mut sim), None);
}
