//! The wire (M11a, third part; ADR-0036 §3–§4): lines of current and hookups are saved with the
//! land and load exactly.

use std::path::Path;
use std::sync::OnceLock;

use civ_content::ContentRegistry;
use civ_core::PermanentId;
use civ_land::wire::{Cause, Hookup, Line};
use civ_sim::{NewWorld, Sim, persist};

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
            name: "Wires".to_owned(),
            seed: 3,
            preset_id: "core:worldgen/river_valley".to_owned(),
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

fn id(n: u64) -> PermanentId {
    PermanentId::from_raw(n).expect("id")
}

#[test]
fn lines_and_hookups_save_and_load_exactly() {
    let mut sim = world();
    let (station, home) = {
        let mut hh = sim.people().households.iter().map(|(_, h)| h.id);
        (hh.next().expect("a household"), hh.next().expect("another"))
    };
    let wires = &mut sim.land_mut_for_tests().wires;
    assert!(
        wires.lines.is_empty() && wires.hookups.is_empty(),
        "nobody lays any"
    );
    wires.lines.push(Line {
        id: id(9_100_001),
        station,
        parent: None,
        cells: vec![10, 11, 12],
        rating_kw: 6.0,
        length_m: 180.5,
        laid_m: 180.5,
        broken: false,
    });
    wires.lines.push(Line {
        id: id(9_100_002),
        station,
        parent: Some(id(9_100_001)),
        cells: vec![12, 780],
        rating_kw: 2.0,
        length_m: 75.0,
        laid_m: 12.25,
        broken: true,
    });
    wires.hookups.push(Hookup {
        building: id(9_100_010),
        household: home,
        line: id(9_100_002),
        ask_kw: 0.3,
        cut_off: true,
        lit_until: 1_234,
        cause: civ_land::wire::Cause::LineFull,
    });
    let before = sim.land().wires.clone();
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(&mut sim, &saves, commons_persist::SaveKind::Manual, "wires").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loaded.land().wires, before);
    let (a, b) = (
        persist::encode_sections(&sim),
        persist::encode_sections(&loaded),
    );
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(&b) {
        assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
    }
    assert!(
        a.iter().any(|s| s.tag.to_string() == "wires"),
        "a wires section"
    );
}

fn straight(width: u32, a: u32, b: u32) -> Vec<u32> {
    let w = i64::from(width);
    let (mut x, mut y) = (i64::from(a) % w, i64::from(a) / w);
    let (bx, by) = (i64::from(b) % w, i64::from(b) / w);
    let mut cells = vec![a];
    while (x, y) != (bx, by) {
        x += (bx - x).signum();
        y += (by - y).signum();
        cells.push((y * w + x) as u32);
    }
    cells
}

#[test]
fn a_line_is_begun_along_ways_with_leave_and_strung_by_work_and_copper() {
    use civ_land::ways::{Keeper, Rank, Segment, Surface, WALK, Way};
    let mut sim = world();
    let map = sim.map().clone();
    let cell = |at: (f32, f32)| civ_agents::population::cell_of(&map, at) as u32;
    let homes: Vec<(PermanentId, u32)> = sim
        .people()
        .households
        .iter()
        .map(|(_, h)| (h.id, cell(h.home)))
        .collect();
    let (station, from) = homes[0];
    let (far, to) = homes
        .iter()
        .copied()
        .find(|&(h, c)| {
            let w = i64::from(map.width);
            let d = (i64::from(c) % w - i64::from(from) % w)
                .abs()
                .max((i64::from(c) / w - i64::from(from) / w).abs());
            h != station && d >= 3
        })
        .expect("a home some cells off");
    // No way between them: no route, no line.
    let begin = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| pop.begin_line(ctx, station, from, to, None))
    };
    assert!(begin(&mut sim).is_none());
    let cells = straight(map.width, from, to);
    let way = sim.allocate_id_for_tests();
    let today = sim.now().day_index();
    sim.land_mut_for_tests().ways.add(Way {
        id: way,
        begun: today,
        segments: vec![Segment {
            cells: cells[1..cells.len() - 1].to_vec(),
            corridor_m: 4.0,
            built_m: 4.0,
            usable_m: 4.0,
            surface: Surface::Cleared,
            keeper: Keeper::Nobody,
            kept: today,
        }],
        modes: WALK,
        trips: 0.0,
        trips_day: today,
        rank: Rank::Lane,
        track: None,
    });
    let line = begin(&mut sim).expect("a route along the way");
    let params = sim.rules().land.wire.expect("the core lays lines");
    let l = sim.land().wires.lines[0].clone();
    assert_eq!((l.id, l.station, l.parent), (line, station, None));
    assert_eq!((l.cells.first(), l.cells.last()), (Some(&from), Some(&to)));
    assert!(l.length_m > 0.0 && l.laid_m == 0.0 && !l.whole());
    assert_eq!(l.rating_kw, params.rating_kw);
    // Without copper no metre is strung; with copper for ten metres, ten hours string ten metres.
    let copper = params.good;
    let set_copper = |sim: &mut Sim, kg: f64| {
        for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
            if x.id == station {
                x.stores[copper] = kg;
            }
        }
    };
    set_copper(&mut sim, 0.0);
    sim.with_ctx_for_tests(|pop, ctx| pop.line_work_for_tests(ctx, station, line, 10.0));
    assert_eq!(sim.land().wires.lines[0].laid_m, 0.0);
    set_copper(&mut sim, 10.0 * params.good_kg_m);
    sim.with_ctx_for_tests(|pop, ctx| pop.line_work_for_tests(ctx, station, line, 100.0));
    let laid = sim.land().wires.lines[0].laid_m;
    assert!((laid - 10.0).abs() < 1e-9, "the copper bounds it: {laid}");
    // With copper enough, the rest is strung and the line is whole.
    set_copper(&mut sim, 1e6);
    sim.with_ctx_for_tests(|pop, ctx| pop.line_work_for_tests(ctx, station, line, 1e6));
    assert!(sim.land().wires.lines[0].whole());
    // The far home hooks up, asking 0.3 kW. The station has no drive: it is short.
    let building = sim.allocate_id_for_tests();
    sim.with_ctx_for_tests(|pop, ctx| pop.begin_hookup(ctx, building, far, line, 0.3));
    sim.with_ctx_for_tests(|pop, ctx| pop.wire_evening_for_tests(ctx));
    let h = sim.land().wires.hookups[0].clone();
    assert_eq!((h.cause, h.lit_until), (Cause::StationShort, -1));
    // A water installation at the station's home on the reach that carries most, and a dynamo
    // beside it: the far home is lit for the evening.
    let reach = (0..sim.land().reach_discharges.len())
        .max_by(|&a, &b| sim.land().reach_discharges[a].total_cmp(&sim.land().reach_discharges[b]))
        .expect("a reach") as u32;
    let home = sim.people().household(station).expect("station").home;
    let mill = sim.allocate_id_for_tests();
    sim.land_mut_for_tests()
        .installations
        .push(civ_land::power::Installation {
            id: mill,
            reach,
            at: [f64::from(home.0), f64::from(home.1)],
            head_m: 2.0,
            race_m3s: 0.5,
            efficiency: 0.3,
            rated_kw: 3.0,
            owner: station,
            building: None,
        });
    let dynamo = sim
        .rules()
        .catalog
        .goods
        .iter()
        .position(|g| g.id == "core:good/dynamo")
        .expect("the dynamo");
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == station {
            x.stores[dynamo] = 1.0;
        }
    }
    let current = sim.with_ctx_for_tests(|pop, ctx| pop.electric_kw(ctx, station));
    assert!(current > 0.3, "the dynamo gives {current} kW");
    let now = sim.now().minutes();
    sim.with_ctx_for_tests(|pop, ctx| pop.wire_evening_for_tests(ctx));
    let h = sim.land().wires.hookups[0].clone();
    assert_eq!((h.cause, h.lit_until), (Cause::Served, now + 240));
    assert_eq!(sim.land().wires.lit_until(far, now), Some(now + 240));
    // Cut off, it is dark again at the next evening, keeping why.
    sim.land_mut_for_tests().wires.hookups[0].cut_off = true;
    sim.with_ctx_for_tests(|pop, ctx| pop.wire_evening_for_tests(ctx));
    assert_eq!(sim.land().wires.hookups[0].cause, Cause::CutOff);
}
