//! Gas (M10b slice DY; ADR-0034 §8): a gasworks' retort makes gas from coal, with coke left over,
//! into its holder; gas made beyond the holder's room, or with no holder, is lost to the air.

use std::path::Path;
use std::sync::OnceLock;

use civ_agents::person::Flow;
use civ_content::ContentRegistry;
use civ_core::PermanentId;
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
            name: "Gasworks".to_owned(),
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

fn index(sim: &Sim, id: &str) -> usize {
    sim.rules()
        .catalog
        .goods
        .iter()
        .position(|g| g.id == id)
        .expect(id)
}

/// An adult of some household, by household and person.
fn adult(sim: &Sim) -> (PermanentId, PermanentId) {
    let now = sim.now();
    sim.people()
        .households
        .iter()
        .find_map(|(_, h)| {
            let p = h.members.iter().copied().find(|&m| {
                sim.people()
                    .person(m)
                    .is_some_and(|p| p.age_years(now) >= 20.0)
            })?;
            Some((h.id, p))
        })
        .expect("an adult")
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

fn saves_alike(sim: &mut Sim) {
    let dir = tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")).expect("temp dir");
    let saves =
        commons_persist::SaveDir::create(dir.path(), civ_schema::SAVE_EXTENSION).expect("dir");
    let saved =
        persist::save(sim, &saves, commons_persist::SaveKind::Manual, "gas").expect("saves");
    let loaded = persist::load(&saved.path, content()).expect("loads");
    let (a, b) = (
        persist::encode_sections(sim),
        persist::encode_sections(&loaded),
    );
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(&b) {
        assert!(x.bytes == y.bytes, "section `{}` differs", x.tag);
    }
}

#[test]
fn a_retort_makes_gas_into_its_holder_and_what_has_no_room_goes_to_the_air() {
    let mut sim = world();
    let (gas, coke, coal, retort, holder) = (
        index(&sim, "core:good/coal_gas"),
        index(&sim, "core:good/coke"),
        index(&sim, "core:good/coal"),
        index(&sim, "core:good/gas_retort"),
        index(&sim, "core:good/gas_holder"),
    );
    let make_gas = sim
        .rules()
        .catalog
        .activities
        .iter()
        .position(|a| a.id == "core:activity/make_gas")
        .expect("making gas");
    let (hh, who) = adult(&sim);
    teach(&mut sim, hh, "core:technique/gas_making");
    set(&mut sim, hh, coal, 10_000.0);
    set(&mut sim, hh, coke, 0.0);
    set(&mut sim, hh, gas, 0.0);
    set(&mut sim, hh, holder, 0.0);
    let stores = |sim: &Sim| {
        sim.people()
            .household(hh)
            .expect("household")
            .stores
            .clone()
    };
    let spoiled = |sim: &Sim| {
        sim.people()
            .household(hh)
            .expect("household")
            .flows
            .get(Flow::Spoiled, gas)
    };
    let work = |sim: &mut Sim, minutes: u32| {
        set(sim, hh, retort, 1.0);
        sim.with_ctx_for_tests(|pop, ctx| pop.make_work_for_tests(ctx, who, make_gas, minutes));
    };
    // With no holder, a session's gas goes to the air; its coke stays and its coal is spent.
    let before = stores(&sim);
    work(&mut sim, 240);
    let after = stores(&sim);
    let burned = before[coal] - after[coal];
    assert!(burned > 0.0, "coal charged into the retort");
    assert!(
        (after[coke] - 0.65 * burned).abs() < 1e-6,
        "{} coke",
        after[coke]
    );
    assert_eq!(after[gas], 0.0, "no holder keeps it");
    assert!(
        (spoiled(&sim) - 0.14 * burned).abs() < 1e-6,
        "all of it lost to the air"
    );
    // With a holder, a short session's gas (an hour of making past the retort's two of firing) is
    // kept, as much as its coal gives.
    set(&mut sim, hh, holder, 1.0);
    let before = stores(&sim);
    work(&mut sim, 180);
    let after = stores(&sim);
    let burned = before[coal] - after[coal];
    assert!(burned > 0.0);
    assert!(
        (after[gas] - 0.14 * burned).abs() < 1e-6,
        "{} gas",
        after[gas]
    );
    assert!(after[gas] < 150.0);
    // Long sessions fill it, and what has no room goes to the air.
    let lost_before = spoiled(&sim);
    for _ in 0..8 {
        work(&mut sim, 480);
    }
    let after = stores(&sim);
    assert!((after[gas] - 150.0).abs() < 1e-9, "{} gas", after[gas]);
    assert!(spoiled(&sim) > lost_before, "the rest is lost");
    // The holder's gas by its energy: 150 kg at 40 MJ a kilogram.
    let goods = &sim.rules().catalog.goods;
    let mj = civ_agents::make::holder_room(goods, &after, gas).expect("a holder keeps gas")
        * goods[gas].fuel.expect("a fuel").mj_per_kg;
    assert!((mj - 6000.0).abs() < 1e-6, "{mj} MJ");
    // Kept a day, the gas stays in its holder.
    sim.advance_minutes(1440).expect("advances");
    let x = sim.people().household(hh).expect("household");
    let now = sim.now();
    let settled = x.stores_at_time(now, &sim.rules().catalog.goods, &|_| 0.0);
    assert!((settled[gas] - 150.0).abs() < 1e-9, "{} gas", settled[gas]);
    saves_alike(&mut sim);
}

/// Minutes from now to `after` minutes past the next dusk.
fn to_dusk(sim: &Sim, after: i64) -> i64 {
    let now = sim.now();
    let doy = civ_land::day_of_year(now.day_index() + 1);
    let (_, set) = civ_agents::needs::daylight(sim.rules().people.latitude_deg, doy);
    (now.day_index() + 1) * 1_440 + set + after - now.minutes()
}

#[test]
fn a_main_lights_its_connections_at_dusk_from_its_works_gas_and_names_why_not() {
    use civ_land::gas::{Cause, Connection, Main};
    let mut sim = world();
    let at = to_dusk(&sim, 30);
    sim.advance_minutes(at).expect("advances");
    let (gas, holder, tallow, lamp, clay, wood) = (
        index(&sim, "core:good/coal_gas"),
        index(&sim, "core:good/gas_holder"),
        index(&sim, "core:good/tallow"),
        index(&sim, "core:good/fat_lamp"),
        index(&sim, "core:good/clay"),
        index(&sim, "core:good/firewood"),
    );
    let (works, _) = adult(&sim);
    // A household of another building, and an adult of it.
    let (home, consumer) = sim
        .land()
        .buildings
        .iter()
        .find(|b| b.household != works)
        .map(|b| (b.id, b.household))
        .expect("another household's building");
    let now = sim.now();
    let maker = sim
        .people()
        .household(consumer)
        .expect("household")
        .members
        .iter()
        .copied()
        .find(|&m| {
            sim.people()
                .person(m)
                .is_some_and(|p| p.age_years(now) >= 20.0)
        })
        .expect("an adult");
    set(&mut sim, works, holder, 1.0);
    set(&mut sim, works, gas, 100.0);
    // Coal enough to pay for its gas by the meter (the fourth part).
    let coal = index(&sim, "core:good/coal");
    set(&mut sim, consumer, coal, 1_000.0);
    let main = PermanentId::from_raw(990_001).expect("id");
    {
        let land = sim.land_mut_for_tests();
        land.gas.mains.push(Main {
            id: main,
            works,
            cells: Vec::new(),
            capacity_kg_h: 1.0,
            broken: false,
            length_m: 0.0,
            laid_m: 0.0,
            worth_h: 0.0,
        });
        land.gas
            .connections
            .push(Connection::new(home, consumer, main, 2));
    }
    let evening = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| pop.gas_evening_for_tests(ctx));
        sim.land().gas.connections[0].clone()
    };
    let held = |sim: &Sim| sim.people().household(works).expect("works").stores[gas];
    // Lit from now for four hours, two burners' gas drawn from the works.
    let c = evening(&mut sim);
    let t = sim.now().minutes();
    assert_eq!(
        (c.cause, c.lit_from, c.lit_until),
        (Cause::Served, t, t + 240)
    );
    assert!(
        (held(&sim) - (100.0 - 2.0 * 0.071 * 4.0)).abs() < 1e-9,
        "{}",
        held(&sim)
    );
    assert!(sim.land().gas.lit(consumer, t + 239) && !sim.land().gas.lit(works, t));
    // Work at home after dark by the burners burns none of the household's tallow, and is the
    // world's first light.
    set(&mut sim, consumer, lamp, 1.0);
    set(&mut sim, consumer, tallow, 1.0);
    set(&mut sim, consumer, clay, 100.0);
    set(&mut sim, consumer, wood, 100.0);
    let pots = sim
        .rules()
        .catalog
        .activities
        .iter()
        .position(|a| a.id == "core:activity/make_pots")
        .expect("pots");
    // Half an hour into the lit evening, 25 minutes of making ending now.
    sim.advance_minutes(30).expect("advances");
    sim.with_ctx_for_tests(|pop, ctx| pop.make_work_for_tests(ctx, maker, pots, 25));
    let x = sim.people().household(consumer).expect("household");
    assert_eq!(x.stores[tallow], 1.0, "no tallow burned under gas light");
    assert!(sim.people().order.lamp_first_lit.is_some());
    // Too little gas, too small a main, a broken main and a cut-off account each leave it dark,
    // by their cause.
    set(&mut sim, works, gas, 0.1);
    assert_eq!(evening(&mut sim).cause, Cause::NoGas);
    assert!(!sim.land().gas.lit(consumer, sim.now().minutes()));
    set(&mut sim, works, gas, 100.0);
    sim.land_mut_for_tests().gas.mains[0].capacity_kg_h = 0.1;
    assert_eq!(evening(&mut sim).cause, Cause::Overloaded);
    sim.land_mut_for_tests().gas.mains[0].capacity_kg_h = 1.0;
    sim.land_mut_for_tests().gas.mains[0].broken = true;
    assert_eq!(evening(&mut sim).cause, Cause::Broken);
    sim.land_mut_for_tests().gas.mains[0].broken = false;
    sim.land_mut_for_tests().gas.connections[0].cut_off = true;
    assert_eq!(evening(&mut sim).cause, Cause::CutOff);
    sim.land_mut_for_tests().gas.connections[0].cut_off = false;
    // Left to the clock, the next dusk lights it again.
    let at = to_dusk(&sim, 5);
    sim.advance_minutes(at).expect("advances");
    let c = sim.land().gas.connections[0].clone();
    assert_eq!((c.cause, c.evening), (Cause::Served, sim.now().day_index()));
    assert!(sim.land().gas.lit(consumer, sim.now().minutes()));
    // Saved and loaded, every section (the gas records among them) comes back alike.
    saves_alike(&mut sim);
}

/// Cells in order from cell `a` to cell `b` on a map `width` cells wide, each one step of the
/// eight toward `b`.
fn line(width: u32, a: u32, b: u32) -> Vec<u32> {
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
fn a_main_is_laid_along_ways_with_leave_by_its_works_pipe_and_carries_gas_once_whole() {
    use civ_land::gas::{Cause, Connection};
    use civ_land::ways::{Keeper, Rank, Segment, Surface, WALK, Way};
    let mut sim = world();
    let at = to_dusk(&sim, 30);
    sim.advance_minutes(at).expect("advances");
    let (gas, holder, iron) = (
        index(&sim, "core:good/coal_gas"),
        index(&sim, "core:good/gas_holder"),
        index(&sim, "core:good/iron"),
    );
    let map = sim.map().clone();
    let cell = |at: (f32, f32)| civ_agents::population::cell_of(&map, at) as u32;
    let homes: Vec<(PermanentId, u32)> = sim
        .people()
        .households
        .iter()
        .map(|(_, h)| (h.id, cell(h.home)))
        .collect();
    let (works, from) = homes[0];
    // A household whose home lies some cells off, with a building of its own.
    let (consumer, to, home) = homes
        .iter()
        .find_map(|&(h, c)| {
            let w = i64::from(map.width);
            let d = (i64::from(c) % w - i64::from(from) % w)
                .abs()
                .max((i64::from(c) / w - i64::from(from) / w).abs());
            let b = sim.land().buildings.iter().find(|b| b.household == h)?;
            (h != works && d >= 3).then_some((h, c, b.id))
        })
        .expect("a household some cells off");
    let other = homes
        .iter()
        .map(|&(h, _)| h)
        .find(|&h| h != works && h != consumer)
        .expect("a third household");
    // A way between them, kept by a third household.
    let cells = line(map.width, from, to);
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
            keeper: Keeper::Household(other),
            kept: today,
        }],
        modes: WALK,
        trips: 0.0,
        trips_day: today,
        rank: Rank::Lane,
        track: None,
    });
    let begin =
        |sim: &mut Sim| sim.with_ctx_for_tests(|pop, ctx| pop.begin_main_for_tests(ctx, works, to));
    // Its keeper gives no leave: no main.
    assert_eq!(begin(&mut sim), None);
    // Kept by nobody, the main runs along it from the works to the building.
    sim.land_mut_for_tests()
        .ways
        .list
        .last_mut()
        .expect("the way")
        .segments[0]
        .keeper = Keeper::Nobody;
    let main = begin(&mut sim).expect("a main");
    let m = sim.land().gas.mains[0].clone();
    assert_eq!(m.id, main);
    assert_eq!((m.cells.first(), m.cells.last()), (Some(&from), Some(&to)));
    assert!(m.length_m > 0.0 && m.laid_m == 0.0 && !m.whole());
    assert_eq!(m.capacity_kg_h, 6.0, "the core's bore");
    let need = f64::from(m.length_m) * 25.0;
    sim.land_mut_for_tests()
        .gas
        .connections
        .push(Connection::new(home, consumer, main, 2));
    set(&mut sim, works, holder, 1.0);
    set(&mut sim, works, gas, 100.0);
    let work = |sim: &mut Sim, who: PermanentId| {
        sim.with_ctx_for_tests(|pop, ctx| pop.main_work_for_tests(ctx, who, main, 10_000.0));
        sim.land().gas.mains[0].laid_m
    };
    // Half the pipe lays half the main, and a main laid in part carries nothing.
    set(&mut sim, works, iron, need / 2.0);
    let laid = work(&mut sim, works);
    assert!(
        (f64::from(laid) - f64::from(m.length_m) / 2.0).abs() < 1e-2,
        "{laid}"
    );
    assert!(sim.people().household(works).expect("works").stores[iron] < 1e-6);
    sim.with_ctx_for_tests(|pop, ctx| pop.gas_evening_for_tests(ctx));
    assert_eq!(sim.land().gas.connections[0].evening, -1, "not yet checked");
    // Another household's work lays none of it.
    set(&mut sim, consumer, iron, need);
    assert_eq!(work(&mut sim, consumer), laid);
    // The rest of the pipe lays the rest; whole, it lights the building at dusk.
    set(&mut sim, works, iron, need);
    let laid = work(&mut sim, works);
    assert_eq!(laid, m.length_m);
    let left = sim.people().household(works).expect("works").stores[iron];
    assert!((left - need / 2.0).abs() < 1e-2, "{left}");
    sim.with_ctx_for_tests(|pop, ctx| pop.gas_evening_for_tests(ctx));
    assert_eq!(sim.land().gas.connections[0].cause, Cause::Served);
    assert!(sim.land().gas.lit(consumer, sim.now().minutes()));
    saves_alike(&mut sim);
}

#[test]
fn a_home_along_a_main_connects_where_gas_light_costs_it_less_and_pays_by_the_meter() {
    use civ_agents::ledger::Channel;
    use civ_land::gas::Cause;
    use civ_land::ways::{Keeper, Rank, Segment, Surface, WALK, Way};
    let mut sim = world();
    let at = to_dusk(&sim, 30);
    sim.advance_minutes(at).expect("advances");
    let (gas, holder, burner, coal, lamp) = (
        index(&sim, "core:good/coal_gas"),
        index(&sim, "core:good/gas_holder"),
        index(&sim, "core:good/gas_burner"),
        index(&sim, "core:good/coal"),
        index(&sim, "core:good/fat_lamp"),
    );
    let map = sim.map().clone();
    let cell = |at: (f32, f32)| civ_agents::population::cell_of(&map, at) as u32;
    let homes: Vec<(PermanentId, u32)> = sim
        .people()
        .households
        .iter()
        .map(|(_, h)| (h.id, cell(h.home)))
        .collect();
    let (works, from) = homes[0];
    // A household whose home lies some cells off, and a way kept by nobody between them.
    let (consumer, to) = homes
        .iter()
        .copied()
        .find(|&(h, c)| {
            let w = i64::from(map.width);
            let d = (i64::from(c) % w - i64::from(from) % w)
                .abs()
                .max((i64::from(c) / w - i64::from(from) / w).abs());
            h != works && d >= 3 && sim.land().buildings.iter().any(|b| b.household == h)
        })
        .expect("a household some cells off");
    let cells = line(map.width, from, to);
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
    let main = sim
        .with_ctx_for_tests(|pop, ctx| pop.begin_main_for_tests(ctx, works, to))
        .expect("a main");
    {
        let m = &mut sim.land_mut_for_tests().gas.mains[0];
        m.laid_m = m.length_m;
    }
    // Only the consumer holds coal, and no lamp.
    let all: Vec<PermanentId> = homes.iter().map(|&(h, _)| h).collect();
    for &h in &all {
        set(&mut sim, h, coal, 0.0);
    }
    set(&mut sim, consumer, coal, 1_000.0);
    set(&mut sim, consumer, lamp, 0.0);
    set(&mut sim, works, holder, 1.0);
    set(&mut sim, works, gas, 100.0);
    set(&mut sim, works, burner, 0.0);
    let costs = sim.with_ctx_for_tests(|pop, ctx| pop.own_costs_for_tests(ctx, consumer));
    // It puts a figure on coal by its heat in firewood, and gas light at the meter costs it less
    // than an hour of its work.
    let wood = index(&sim, "core:good/firewood");
    let ratio = civ_agents::params::fuel_ratio(&sim.rules().catalog.goods, coal);
    let coal_h = costs[coal].unwrap_or(costs[wood].expect("firewood's cost") * ratio);
    assert!(0.071 / 0.14 * coal_h < 1.0, "{coal_h}");
    let evening = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| pop.gas_evening_for_tests(ctx));
        sim.land()
            .gas
            .connections
            .iter()
            .find(|c| c.household == consumer)
            .cloned()
    };
    let held = |sim: &Sim, h: PermanentId, g: usize| {
        sim.people().household(h).expect("household").stores[g]
    };
    // With no burner at the works to fit, nobody connects.
    assert!(evening(&mut sim).is_none());
    // A burner held whole, it is fitted at the consumer's home, lit, and paid for by the meter
    // in coal: four hours of a burner's gas at the coal a retort charges for it.
    set(&mut sim, works, burner, 1.0);
    let works_coal = held(&sim, works, coal);
    let c = evening(&mut sim).expect("connected");
    assert_eq!(
        (c.main, c.burners, c.cause, c.cut_off),
        (main, 1, Cause::Served, false)
    );
    assert!(held(&sim, works, burner) < 1e-9, "its burner fitted");
    let owed = 0.071 * 4.0 / 0.14;
    assert!(held(&sim, consumer, coal) <= 1_000.0 - owed + 1e-6);
    assert!((held(&sim, works, coal) - works_coal - owed).abs() < 1e-6);
    let metered = sim.people().transfers.get(Channel::Gas, coal);
    assert!((metered - owed).abs() < 1e-6, "{metered}");
    assert_eq!(
        sim.land()
            .gas
            .connections
            .iter()
            .filter(|c| c.household == consumer)
            .count(),
        1,
        "one connection"
    );
    // Short of coal, it is lit tonight, pays what it holds and is cut off; the next evening it
    // stays dark by that cause.
    set(&mut sim, consumer, coal, 1.0);
    let c = evening(&mut sim).expect("connected");
    assert!(c.cause == Cause::Served && c.cut_off);
    assert!(held(&sim, consumer, coal) < 1e-9);
    assert_eq!(evening(&mut sim).expect("connected").cause, Cause::CutOff);
    // Holding an evening's price again, it is restored.
    set(&mut sim, consumer, coal, 1_000.0);
    let c = evening(&mut sim).expect("connected");
    assert!(c.cause == Cause::Served && !c.cut_off);
    // The works of a main laid whole, its burner fitted, makes another to fit the next home (its
    // people smiths enough that a session's burner is a whole one).
    teach(&mut sim, works, "core:technique/gas_making");
    let smithing = sim
        .rules()
        .catalog
        .skills
        .iter()
        .position(|k| k.id == "core:skill/smithing")
        .expect("smithing");
    let members = sim
        .people()
        .household(works)
        .expect("works")
        .members
        .clone();
    for (_, p) in sim.people_mut_for_tests().people.iter_mut() {
        if members.contains(&p.id) {
            p.set_skill(smithing, 0.5);
        }
    }
    let (iron, charcoal) = (
        index(&sim, "core:good/iron"),
        index(&sim, "core:good/charcoal"),
    );
    set(&mut sim, works, iron, 10.0);
    set(&mut sim, works, charcoal, 10.0);
    let mut made = false;
    for _ in 0..4 {
        sim.advance_minutes(1_440).expect("advances");
        made |= held(&sim, works, burner) >= 1.0;
    }
    assert!(made, "the works made a burner");
    saves_alike(&mut sim);
}

#[test]
fn a_works_begins_a_main_where_the_homes_along_it_repay_it_and_its_people_lay_it() {
    use civ_land::ways::{Keeper, Rank, Segment, Surface, WALK, Way};
    let mut sim = world();
    let at = to_dusk(&sim, 30);
    sim.advance_minutes(at).expect("advances");
    let (retort, holder, iron, coal, lamp) = (
        index(&sim, "core:good/gas_retort"),
        index(&sim, "core:good/gas_holder"),
        index(&sim, "core:good/iron"),
        index(&sim, "core:good/coal"),
        index(&sim, "core:good/fat_lamp"),
    );
    let map = sim.map().clone();
    let cell = |at: (f32, f32)| civ_agents::population::cell_of(&map, at) as u32;
    let homes: Vec<(PermanentId, u32)> = sim
        .people()
        .households
        .iter()
        .map(|(_, h)| (h.id, cell(h.home)))
        .collect();
    // Two households of one settlement whose homes lie some cells apart, the first the works,
    // with a way kept by nobody between them.
    let settled = |h: PermanentId| sim.people().household(h).expect("household").settlement;
    let built = |h: PermanentId| sim.land().buildings.iter().any(|b| b.household == h);
    let w = i64::from(map.width);
    let apart = |a: u32, b: u32| {
        (i64::from(a) % w - i64::from(b) % w)
            .abs()
            .max((i64::from(a) / w - i64::from(b) / w).abs())
    };
    let ((works, from), (consumer, to)) = homes
        .iter()
        .flat_map(|&a| homes.iter().map(move |&b| (a, b)))
        .find(|&((a, ca), (b, cb))| {
            a != b && settled(a) == settled(b) && apart(ca, cb) >= 3 && built(b)
        })
        .expect("two homes of one settlement some cells apart");
    let cells = line(map.width, from, to);
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
    // The works holds a retort and a holder, and iron enough for a main; the consumer coal and
    // no lamp.
    set(&mut sim, works, retort, 1.0);
    set(&mut sim, works, holder, 1.0);
    set(&mut sim, works, iron, 5_000.0);
    set(&mut sim, consumer, coal, 1_000.0);
    set(&mut sim, consumer, lamp, 0.0);
    let reckon = |sim: &mut Sim| {
        sim.with_ctx_for_tests(|pop, ctx| pop.main_reckonings_for_tests(ctx, works))
    };
    // Not knowing gas making, it weighs no main, and none is begun.
    assert!(reckon(&mut sim).is_empty());
    sim.with_ctx_for_tests(|pop, ctx| pop.review_main_for_tests(ctx, works));
    assert!(sim.land().gas.mains.is_empty());
    // Knowing it, the main to the consumer's home is reckoned: its home along it would repay
    // its evenings' gain, against its laying.
    teach(&mut sim, works, "core:technique/gas_making");
    let r = reckon(&mut sim)
        .into_iter()
        .find(|r| r.to == to)
        .expect("a main to the consumer reckoned");
    assert!(r.homes >= 1 && r.worth_h_year > 0.0, "{r:?}");
    assert!(
        (r.cost_h - r.length_m * 3.0).abs() < 1e-6,
        "its pipe in hand, the cost is its laying: {r:?}"
    );
    assert!(r.worth_h_year * 50.0 > r.cost_h, "{r:?}");
    // At its review it begins the best, marked out with what it reckoned.
    let best = reckon(&mut sim).into_iter().next().expect("a best");
    sim.with_ctx_for_tests(|pop, ctx| pop.review_main_for_tests(ctx, works));
    let m = sim.land().gas.mains[0].clone();
    assert_eq!(m.works, works);
    assert_eq!(m.cells.last(), Some(&best.to));
    assert!((f64::from(m.worth_h) - best.worth_h_year).abs() < 1e-3);
    assert!(m.laid_m == 0.0 && !m.whole());
    // A second review begins nothing while it is being laid.
    sim.with_ctx_for_tests(|pop, ctx| pop.review_main_for_tests(ctx, works));
    assert_eq!(sim.land().gas.mains.len(), 1);
    // Left to the clock, its people lay it as way work, a metre of pipe at a time.
    for _ in 0..6 {
        sim.advance_minutes(1_440).expect("advances");
    }
    let laid = sim.land().gas.mains[0].laid_m;
    assert!(laid > 0.0, "{laid}");
    let left = sim.people().household(works).expect("works").stores[iron];
    assert!(
        (5_000.0 - left - f64::from(laid) * 25.0).abs() < 1e-2,
        "{left} {laid}"
    );
    saves_alike(&mut sim);
}

#[test]
fn a_taking_unseen_in_the_dark_puts_streets_lit_by_a_lamplighter_before_the_gathering() {
    use civ_agents::crime::{Incident, KnownAmount, Outcome};
    use civ_agents::polity::{IssueKind, LawStatus, PolicyKind};
    use civ_land::ways::{Keeper, Rank, Segment, Surface, WALK, Way};
    let mut sim = world();
    // The settlement's polity is founded at its first midnight.
    sim.advance_minutes(24 * 60).expect("advances");
    let (victim, _) = adult(&sim);
    let settlement = sim
        .people()
        .household(victim)
        .and_then(|x| x.settlement)
        .expect("a settlement");
    let pi = sim
        .people()
        .polities
        .iter()
        .position(|p| p.holds(Some(settlement)))
        .expect("its polity");
    // A way kept by nobody passing beside each home of the settlement, a cell east of it.
    let map = sim.map().clone();
    let mut beside: Vec<u32> = sim
        .people()
        .households
        .iter()
        .filter(|(_, x)| x.settlement == Some(settlement) && !x.members.is_empty())
        .map(|(_, x)| civ_agents::population::cell_of(&map, x.home) as u32 + 1)
        .collect();
    beside.sort_unstable();
    beside.dedup();
    let way = sim.allocate_id_for_tests();
    let today = sim.now().day_index();
    sim.land_mut_for_tests().ways.add(Way {
        id: way,
        begun: today,
        segments: vec![Segment {
            cells: beside.clone(),
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
    let issue =
        |sim: &mut Sim| sim.with_ctx_for_tests(|pop, ctx| pop.dark_takings_for_tests(ctx, pi));
    // Nothing taken: no issue.
    assert!(!issue(&mut sim));
    // Grain taken from the victim's store at one in the morning, nobody seeing who took it, and
    // the loss found.
    let (taker, _) = sim
        .people()
        .households
        .iter()
        .find_map(|(_, x)| (x.id != victim).then(|| (x.members[0], x.id)))
        .expect("another household");
    let night = civ_core::SimTime::from_minutes(today * 1440 + 60);
    let home = sim.people().household(victim).expect("victim").home;
    let taker_household = sim.people().person(taker).expect("taker").household;
    {
        let order = &mut sim.people_mut_for_tests().order;
        let id = order.incidents.len() as u32 + 1;
        order.incidents.push(Incident {
            id,
            at: night,
            actor: taker,
            actor_household: taker_household,
            target: victim,
            place: home,
            outcome: Outcome::Taken,
            goods: Vec::new(),
            kcal: 2_000.0,
            seen_by: Vec::new(),
            noticed: true,
            food_days: f32::NAN,
            richer: f32::NAN,
            across: None,
        });
        order.amounts.push(KnownAmount {
            household: victim,
            incident: id,
            kcal: 2_000.0,
            lost: true,
            day: today,
        });
    }
    // Nobody of the settlement knows how to make a light: still no issue. Taught lamps, there is.
    let knows = sim.people().households.iter().any(|(_, x)| {
        x.settlement == Some(settlement) && {
            let t = sim
                .rules()
                .catalog
                .techniques
                .iter()
                .position(|t| t.id == "core:technique/lamps")
                .expect("lamps");
            x.members
                .iter()
                .any(|&m| sim.people().person(m).is_some_and(|p| p.knows(t)))
        }
    });
    if !knows {
        assert!(!issue(&mut sim), "nobody knows how to make a light");
    }
    teach(&mut sim, victim, "core:technique/lamps");
    assert!(issue(&mut sim));
    // Its lamps would stand beside its homes, on the way.
    let sites = sim.with_ctx_for_tests(|pop, ctx| pop.street_lamp_sites_for_tests(ctx, pi));
    assert!(
        !sites.is_empty() && sites.iter().all(|c| beside.contains(c)),
        "{sites:?}"
    );
    // Someone proposes that a lamplighter light the streets, against takings in the dark.
    sim.people_mut_for_tests().every_adult_deliberates = true;
    let mut proposed = None;
    for _ in 0..24 {
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
            Some(l) if l.kind == PolicyKind::LightStreets => {
                proposed = Some(l);
                break;
            }
            Some(_) => continue,
            None => break,
        }
    }
    let law = proposed.expect("streets lit proposed");
    assert_eq!(law.issue, IssueKind::DarkTakings);
    let lamplighter = law.holder.expect("a lamplighter named");
    // Passed, the polity sets its lamps beside the homes at once, unlit, and the one named holds
    // the office.
    {
        let pop = sim.people_mut_for_tests();
        pop.every_adult_deliberates = false;
        let p = &mut pop.polities[pi];
        p.gathering = None;
        let l = p.laws.iter_mut().find(|l| l.id == law.id).expect("the law");
        l.status = LawStatus::InForce;
    }
    sim.with_ctx_for_tests(|pop, ctx| pop.begin_street_lamps_for_tests(ctx, pi, law.id));
    let lamps = sim.land().gas.lamps.clone();
    assert_eq!(lamps.iter().map(|l| l.cell).collect::<Vec<_>>(), sites);
    let polity = sim.people().polities[pi].id;
    assert!(
        lamps
            .iter()
            .all(|l| l.polity == polity && l.law == law.id && l.lit_until == -1)
    );
    assert_eq!(
        sim.people().polities[pi].lamplighter().map(|(h, _)| h),
        Some(lamplighter)
    );
    // Its lamps stand where it would set them, so no more are wanted.
    let again = sim.with_ctx_for_tests(|pop, ctx| pop.street_lamp_sites_for_tests(ctx, pi));
    assert!(again.is_empty(), "{again:?}");
    saves_alike(&mut sim);
}
