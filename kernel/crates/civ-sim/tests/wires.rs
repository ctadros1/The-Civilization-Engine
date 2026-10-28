//! The wire (M11a, third part; ADR-0036 §3–§4): lines of current and hookups are saved with the
//! land and load exactly.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::ledger::Channel;
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
        worth_h: 0.0,
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
        worth_h: 36.5,
    });
    wires.hookups.push(Hookup {
        building: id(9_100_010),
        household: home,
        line: id(9_100_002),
        ask_kw: 0.3,
        cut_off: true,
        lit_until: 1_234,
        cause: civ_land::wire::Cause::LineFull,
        asked: 0b1011,
        served: 0b0010,
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
    // Its home is lit, as a gas burner lights one, for work and sight after dark.
    assert!(sim.land().wires.lit(far, now) && !sim.land().wires.lit(far, now + 240));
    // Cut off, with nothing to pay with, it is dark again at the next evening, keeping why (one
    // that holds an evening's price is restored).
    let food: Vec<usize> = (0..sim.rules().catalog.goods.len())
        .filter(|&g| sim.rules().catalog.goods[g].purpose == civ_agents::params::GoodUse::Food)
        .collect();
    for g in food {
        set(&mut sim, far, g, 0.0);
    }
    sim.land_mut_for_tests().wires.hookups[0].cut_off = true;
    sim.with_ctx_for_tests(|pop, ctx| pop.wire_evening_for_tests(ctx));
    assert_eq!(sim.land().wires.hookups[0].cause, Cause::CutOff);
}

#[test]
fn a_household_whose_dynamo_turns_beside_its_installation_lights_its_own_home() {
    let mut sim = world();
    // Ten days for homes to be begun; one of them, finished.
    sim.advance_minutes(10 * 24 * 60).expect("lives");
    let (station, home_id) = {
        let land = sim.land();
        let b = land
            .buildings
            .iter()
            .find(|b| sim.rules().catalog.is_dwelling(&b.spec.program))
            .expect("a home");
        (b.household, b.id)
    };
    for b in &mut sim.land_mut_for_tests().buildings {
        if b.id == home_id {
            b.stage = civ_grammar::Stage::ALL.len() as _;
        }
    }
    let reach = (0..sim.land().reach_discharges.len())
        .max_by(|&a, &b| sim.land().reach_discharges[a].total_cmp(&sim.land().reach_discharges[b]))
        .expect("a reach") as u32;
    let at = sim.people().household(station).expect("station").home;
    let mill = sim.allocate_id_for_tests();
    sim.land_mut_for_tests()
        .installations
        .push(civ_land::power::Installation {
            id: mill,
            reach,
            at: [f64::from(at.0), f64::from(at.1)],
            head_m: 2.0,
            race_m3s: 0.5,
            efficiency: 0.3,
            rated_kw: 3.0,
            owner: station,
            building: None,
        });
    // Without a dynamo it hooks nothing up.
    sim.with_ctx_for_tests(|pop, ctx| pop.wire_reviews_for_tests(ctx));
    assert!(sim.land().wires.hookups.is_empty());
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
    // With one, its home is hooked up on a line of its own station, whole at once, and lit at the
    // evening; a second review adds nothing.
    sim.with_ctx_for_tests(|pop, ctx| pop.wire_reviews_for_tests(ctx));
    sim.with_ctx_for_tests(|pop, ctx| pop.wire_reviews_for_tests(ctx));
    let wires = sim.land().wires.clone();
    assert_eq!((wires.lines.len(), wires.hookups.len()), (1, 1));
    assert!(wires.lines[0].whole() && wires.lines[0].station == station);
    assert_eq!(
        (wires.hookups[0].building, wires.hookups[0].household),
        (home_id, station)
    );
    sim.with_ctx_for_tests(|pop, ctx| pop.wire_evening_for_tests(ctx));
    assert_eq!(sim.land().wires.hookups[0].cause, Cause::Served);
}

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

fn set(sim: &mut Sim, household: PermanentId, g: usize, kg: f64) {
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == household {
            x.stores[g] = kg;
        }
    }
}

/// The ways layer's lines of current as the observer reads them (wire 1.107): each line's
/// buildings hooked up, those lit now, its share strung and its words.
fn currents_seen(sim: &Sim) -> Vec<(u32, u32, f32, String)> {
    let bytes = civ_sim::frames::ways::ways_response(sim);
    let response =
        civ_schema::flatbuffers::root::<civ_schema::wire::Response>(&bytes).expect("wire");
    let ways = response.body_as_ways().expect("ways");
    ways.currents()
        .expect("lines of current")
        .iter()
        .map(|l| {
            (
                l.homes(),
                l.lit(),
                l.laid(),
                l.words().unwrap_or_default().to_owned(),
            )
        })
        .collect()
}

/// Building `id`'s current in words as the observer reads them (wire 1.107).
fn wire_words_seen(sim: &Sim, id: PermanentId) -> String {
    let bytes = civ_sim::frames::buildings::buildings_response(sim);
    let response =
        civ_schema::flatbuffers::root::<civ_schema::wire::Response>(&bytes).expect("wire");
    response
        .body_as_buildings()
        .expect("buildings")
        .buildings()
        .expect("a list")
        .iter()
        .find(|b| b.id() == id.get())
        .expect("the building")
        .wire()
        .unwrap_or_default()
        .to_owned()
}

fn paid_by_meter(sim: &Sim) -> f64 {
    let goods = &sim.rules().catalog.goods;
    (0..goods.len())
        .map(|g| sim.people().transfers.get(Channel::Wire, g) * goods[g].kcal_per_kg)
        .sum()
}

#[test]
fn a_station_that_knows_the_dynamo_strings_a_line_to_homes_that_pay_by_the_meter() {
    use civ_land::ways::{Keeper, Rank, Segment, Surface, WALK, Way};
    let mut sim = world();
    // Ten days for homes to be begun; every one, finished.
    sim.advance_minutes(10 * 24 * 60).expect("lives");
    let dwellings: Vec<PermanentId> = sim
        .land()
        .buildings
        .iter()
        .filter(|b| sim.rules().catalog.is_dwelling(&b.spec.program))
        .map(|b| b.id)
        .collect();
    for b in &mut sim.land_mut_for_tests().buildings {
        if dwellings.contains(&b.id) {
            b.stage = civ_grammar::Stage::ALL.len() as _;
        }
    }
    let map = sim.map().clone();
    let cell = |at: (f32, f32)| civ_agents::population::cell_of(&map, at) as u32;
    let homes: Vec<(PermanentId, u32)> = sim
        .land()
        .buildings
        .iter()
        .filter(|b| dwellings.contains(&b.id))
        .filter_map(|b| {
            let h = sim.people().household(b.household)?;
            Some((h.id, cell(h.home)))
        })
        .collect();
    let (station, from) = homes[0];
    let w = i64::from(map.width);
    let (far, to) = homes
        .iter()
        .copied()
        .find(|&(h, c)| {
            let d = (i64::from(c) % w - i64::from(from) % w)
                .abs()
                .max((i64::from(c) / w - i64::from(from) / w).abs());
            h != station && d >= 3
        })
        .expect("a home some cells off");
    // A water installation at the station's home and a dynamo beside it.
    let reach = (0..sim.land().reach_discharges.len())
        .max_by(|&a, &b| sim.land().reach_discharges[a].total_cmp(&sim.land().reach_discharges[b]))
        .expect("a reach") as u32;
    let at = sim.people().household(station).expect("station").home;
    let mill = sim.allocate_id_for_tests();
    sim.land_mut_for_tests()
        .installations
        .push(civ_land::power::Installation {
            id: mill,
            reach,
            at: [f64::from(at.0), f64::from(at.1)],
            head_m: 4.0,
            race_m3s: 1.0,
            efficiency: 0.5,
            rated_kw: 6.0,
            owner: station,
            building: None,
        });
    let goods = sim.rules().catalog.goods.clone();
    let dynamo = goods
        .iter()
        .position(|g| g.id == "core:good/dynamo")
        .expect("the dynamo");
    let params = sim.rules().land.wire.expect("the core lays lines");
    set(&mut sim, station, dynamo, 1.0);
    set(&mut sim, station, params.good, 1e6);
    // Its own home first, on a line of no length.
    sim.with_ctx_for_tests(|pop, ctx| pop.wire_reviews_for_tests(ctx));
    assert_eq!(sim.land().wires.hookups.len(), 1);
    let reckon = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| pop.wire_line_reckonings_for_tests(ctx, station))
    };
    // Not knowing how a dynamo is made, it weighs no line; knowing, none reaches without a way.
    assert!(reckon(&mut sim).is_empty());
    teach(&mut sim, station, "core:technique/dynamo");
    assert!(reckon(&mut sim).iter().all(|r| r.to != to));
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
    // Along the way the far home would repay a line over its life: the best is begun.
    let reckoned = reckon(&mut sim);
    let r = reckoned
        .iter()
        .find(|r| r.to == to)
        .expect("a line to the far home");
    assert!(r.homes >= 1 && r.worth_h_year > 0.0, "{r:?}");
    let best = reckoned[0].clone();
    assert!(
        best.worth_h_year * params.life_years > best.cost_h,
        "{best:?}"
    );
    sim.with_ctx_for_tests(|pop, ctx| pop.review_wire_line_for_tests(ctx, station));
    let l = sim.land().wires.lines.last().expect("a line").clone();
    assert_eq!(
        (l.station, l.parent, l.cells.last()),
        (station, None, Some(&best.to))
    );
    assert!(!l.whole() && (l.worth_h - best.worth_h_year).abs() < 1e-9);
    // The observer sees it marked out, nothing strung.
    let seen = currents_seen(&sim);
    assert_eq!(seen.len(), 2);
    assert!(seen[1].3.ends_with(", 0% strung."), "{}", seen[1].3);
    assert_eq!(seen[1].2, 0.0);
    // A second review begins none while it is being strung.
    sim.with_ctx_for_tests(|pop, ctx| pop.review_wire_line_for_tests(ctx, station));
    assert_eq!(sim.land().wires.lines.len(), 2);
    // Strung whole, the homes along it hook up at the evening, are lit, and pay by the meter.
    sim.with_ctx_for_tests(|pop, ctx| pop.line_work_for_tests(ctx, station, l.id, 1e6));
    assert!(sim.land().wires.lines[1].whole());
    sim.with_ctx_for_tests(|pop, ctx| pop.wire_evening_for_tests(ctx));
    let theirs: Vec<Hookup> = sim
        .land()
        .wires
        .hookups
        .iter()
        .filter(|h| h.line == l.id)
        .cloned()
        .collect();
    assert!(!theirs.is_empty(), "homes along the line hooked up");
    assert!(
        theirs
            .iter()
            .all(|h| h.cause == Cause::Served && h.household != station)
    );
    // The observer sees the line strung and its homes lit, each building's current, the station,
    // and a member's light.
    let seen = currents_seen(&sim);
    let n = theirs.len() as u32;
    assert_eq!((seen[1].0, seen[1].1, seen[1].2), (n, n, 1.0));
    let homes = if n == 1 {
        "1 home hooked up".to_owned()
    } else {
        format!("{n} homes hooked up")
    };
    assert!(
        seen[1].3.ends_with(&format!(
            ", strung, passing up to 6.0 kW; {homes}, {n} lit now."
        )),
        "{}",
        seen[1].3
    );
    let until = theirs[0].lit_until;
    let clock = format!("{:02}:{:02}", (until % 1440) / 60, until % 60);
    let words = wire_words_seen(&sim, theirs[0].building);
    assert!(
        words.starts_with("Current: 0.2 kW from ")
            && words.ends_with(&format!(
                "; lit until {clock}; served 1 of 1 evening asked in the last 30"
            )),
        "{words}"
    );
    let station_first = sim
        .land()
        .buildings
        .iter()
        .find(|b| b.household == station)
        .expect("the station's building")
        .id;
    let words = wire_words_seen(&sim, station_first);
    assert!(
        words.contains("Station: a dynamo beside its own water installation; feeds 2 lines, "),
        "{words}"
    );
    let member = sim
        .people()
        .household(theirs[0].household)
        .expect("home")
        .members[0];
    let light = civ_sim::frames::people::light_words(&sim, member);
    assert!(
        light == "Working by electric light at home"
            || light == format!("Their home is lit by the wire until {clock}"),
        "{light}"
    );
    let evening = 0.2 * 4.0 * 0.1 * sim.rules().people.household.daily_kcal_per_person;
    let paid = paid_by_meter(&sim);
    assert!(
        (paid - evening * theirs.len() as f64).abs() < 1e-3,
        "{paid} for {} homes",
        theirs.len()
    );
    // One with no food left is served but cannot pay: it is cut off, and dark at the next
    // evening; with food again it is restored.
    let poor = theirs[0].household;
    let food: Vec<usize> = (0..goods.len())
        .filter(|&g| goods[g].purpose == civ_agents::params::GoodUse::Food)
        .collect();
    let held: Vec<f64> = food
        .iter()
        .map(|&g| sim.people().household(poor).expect("home").stores[g])
        .collect();
    for &g in &food {
        set(&mut sim, poor, g, 0.0);
    }
    let of = |sim: &Sim| {
        sim.land()
            .wires
            .hookups
            .iter()
            .find(|h| h.household == poor)
            .cloned()
            .expect("hooked up")
    };
    sim.with_ctx_for_tests(|pop, ctx| pop.wire_evening_for_tests(ctx));
    assert!(of(&sim).cut_off);
    sim.with_ctx_for_tests(|pop, ctx| pop.wire_evening_for_tests(ctx));
    assert_eq!(of(&sim).cause, Cause::CutOff);
    let words = wire_words_seen(&sim, of(&sim).building);
    assert!(
        words.contains("; cut off: its account went unpaid; served 2 of 2 evenings asked"),
        "{words}"
    );
    for (&g, &kg) in food.iter().zip(&held) {
        set(&mut sim, poor, g, kg);
    }
    sim.with_ctx_for_tests(|pop, ctx| pop.wire_evening_for_tests(ctx));
    let h = of(&sim);
    assert!(!h.cut_off && h.cause == Cause::Served, "{h:?}");
    // Its record: four evenings, three asked (not the one cut off), all three served.
    assert_eq!(h.record_of(30), (3, 3));
    let words = wire_words_seen(&sim, h.building);
    assert!(
        words.ends_with("; served 3 of 3 evenings asked in the last 30"),
        "{words}"
    );
    let _ = far;
}

#[test]
fn a_polity_strings_a_line_of_current_together_to_homes_beyond_a_stations_lines() {
    use civ_agents::polity::{IssueKind, LawStatus, Outcome, PolicyKind};
    use civ_land::ways::{Keeper, Rank, Segment, Surface, WALK, Way};
    let mut sim = world();
    sim.advance_minutes(10 * 24 * 60).expect("lives");
    let dwellings: Vec<PermanentId> = sim
        .land()
        .buildings
        .iter()
        .filter(|b| sim.rules().catalog.is_dwelling(&b.spec.program))
        .map(|b| b.id)
        .collect();
    for b in &mut sim.land_mut_for_tests().buildings {
        if dwellings.contains(&b.id) {
            b.stage = civ_grammar::Stage::ALL.len() as _;
        }
    }
    let map = sim.map().clone();
    let cell = |at: (f32, f32)| civ_agents::population::cell_of(&map, at) as u32;
    let homes: Vec<(PermanentId, u32)> = sim
        .land()
        .buildings
        .iter()
        .filter(|b| dwellings.contains(&b.id))
        .filter_map(|b| {
            let h = sim.people().household(b.household)?;
            Some((h.id, cell(h.home)))
        })
        .collect();
    let (station, from) = homes[0];
    let settlement = sim
        .people()
        .household(station)
        .and_then(|x| x.settlement)
        .expect("of a settlement");
    let w = i64::from(map.width);
    let apart = |a: u32, b: u32| {
        (i64::from(a) % w - i64::from(b) % w)
            .abs()
            .max((i64::from(a) / w - i64::from(b) / w).abs())
    };
    let (far, to) = homes
        .iter()
        .copied()
        .find(|&(h, c)| h != station && apart(c, from) >= 3)
        .expect("a home some cells off");
    // A second home beside the far one, beyond the station's reach too.
    let beside = homes
        .iter()
        .map(|h| h.0)
        .find(|&h| h != station && h != far)
        .expect("another home");
    let far_home = sim.people().household(far).expect("far").home;
    for (_, x) in sim.people_mut_for_tests().households.iter_mut() {
        if x.id == beside {
            x.home = far_home;
        }
    }
    // A station: a water installation and a dynamo beside it, its own home hooked up.
    let reach = (0..sim.land().reach_discharges.len())
        .max_by(|&a, &b| sim.land().reach_discharges[a].total_cmp(&sim.land().reach_discharges[b]))
        .expect("a reach") as u32;
    let at = sim.people().household(station).expect("station").home;
    let mill = sim.allocate_id_for_tests();
    sim.land_mut_for_tests()
        .installations
        .push(civ_land::power::Installation {
            id: mill,
            reach,
            at: [f64::from(at.0), f64::from(at.1)],
            head_m: 4.0,
            race_m3s: 1.0,
            efficiency: 0.5,
            rated_kw: 6.0,
            owner: station,
            building: None,
        });
    let goods = sim.rules().catalog.goods.clone();
    let dynamo = goods
        .iter()
        .position(|g| g.id == "core:good/dynamo")
        .expect("the dynamo");
    let params = sim.rules().land.wire.expect("the core lays lines");
    set(&mut sim, station, dynamo, 1.0);
    for (h, _) in &homes {
        set(&mut sim, *h, params.good, 100.0);
    }
    sim.with_ctx_for_tests(|pop, ctx| pop.wire_reviews_for_tests(ctx));
    let public = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| pop.public_current_for_tests(ctx, settlement))
    };
    // No way runs to the far homes: no line to string together reaches them.
    assert!(public(&mut sim).is_none_or(|p| p.1 != to));
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
    // Along the way the two far homes would be lit by a line strung together from the station.
    let (st, end, labour_h, saves) = public(&mut sim).expect("a line to string together");
    // It leaves the station; the home it runs to is the one whose line serves most for least.
    assert_eq!(st, station);
    assert!(labour_h > 0.0, "{labour_h}");
    assert!(
        saves.iter().any(|s| s.0 == far) && saves.iter().any(|s| s.0 == beside),
        "{saves:?}"
    );
    // Someone proposes that the polity string it, for their homes lie dark beyond the wire.
    let pi = sim
        .people()
        .polities
        .iter()
        .position(|p| p.settlement == settlement)
        .expect("a polity");
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
            Some(l) if l.kind == PolicyKind::LayCurrent => {
                proposed = Some(l);
                break;
            }
            Some(_) => continue,
            None => break,
        }
    }
    let law = proposed.expect("a line of current proposed");
    assert_eq!(
        (law.issue, law.target),
        (IssueKind::HomesUnlit, Some(station))
    );
    let id = law.id;
    let site = sim.people().polities[pi]
        .way_site_of(id)
        .expect("its site recorded");
    assert_eq!((site.cell, site.work), (end, None));
    assert!(sim.people().polities[pi].way_saves_of(id, far) > 0.0);
    // Passed and known, the polity marks the line out from the station's home.
    let adults: Vec<PermanentId> = sim
        .people()
        .people
        .iter()
        .filter(|(_, p)| p.age_years(sim.now()) >= 16.0)
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
    assert!(words.contains("line of current strung together"), "{words}");
    sim.with_ctx_for_tests(|pop, ctx| pop.begin_public_current_for_tests(ctx, pi, id));
    let line = sim.people().polities[pi]
        .way_site_of(id)
        .and_then(|s| s.work)
        .expect("the line marked out");
    let l = sim
        .land()
        .wires
        .lines
        .iter()
        .find(|l| l.id == line)
        .cloned()
        .expect("the line");
    assert_eq!(
        (l.station, l.cells.first(), l.cells.last()),
        (station, Some(&from), Some(&end))
    );
    assert!(!l.whole());
    // Saved and loaded while it is marked out, the line is kept.
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves_dir =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved = persist::save(
        &mut sim,
        &saves_dir,
        commons_persist::SaveKind::Manual,
        "law",
    )
    .expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    assert_eq!(loaded.land().wires, sim.land().wires);
    assert_eq!(
        loaded.people().polities[pi].way_site_of(id),
        sim.people().polities[pi].way_site_of(id)
    );
    // The far households' shares string it, each from its own copper, counted toward its share.
    let half = l.length_m * params.lay_h_m / 2.0 + 1.0;
    for h in [far, beside] {
        sim.with_ctx_for_tests(|pop, ctx| pop.way_work_for_tests(ctx, h, line, half));
    }
    let strung = sim
        .land()
        .wires
        .lines
        .iter()
        .find(|l| l.id == line)
        .cloned()
        .expect("the line");
    assert!(strung.whole(), "{strung:?}");
    for h in [far, beside] {
        assert!(sim.people().polities[pi].way_given_by(id, h) > 0.0);
        let copper = sim.people().household(h).expect("home").stores[params.good];
        assert!(copper < 100.0, "its own copper strung: {copper}");
    }
    // At the evening the far homes hook up to it, are lit, and pay the station by the meter.
    sim.with_ctx_for_tests(|pop, ctx| pop.wire_evening_for_tests(ctx));
    for h in [far, beside] {
        let k = sim
            .land()
            .wires
            .hookups
            .iter()
            .find(|k| k.household == h)
            .cloned()
            .expect("hooked up");
        assert_eq!((k.line, k.cause), (line, Cause::Served));
    }
    assert!(paid_by_meter(&sim) > 0.0);
}
