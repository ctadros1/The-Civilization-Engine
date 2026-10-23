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
