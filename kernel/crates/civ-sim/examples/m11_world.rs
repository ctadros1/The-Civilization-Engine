//! The M11 demo (plan §7, M11d; ADR-0036): a world of several settlements lived ahead at Max until
//! whoever will has turned a dynamo, strung a line of current, set a gas engine, kept a telegraph
//! office or driven a motor cart; what was built where and what the others did instead. Whether
//! any settlement electrifies is the people's.
//!
//! The observer acts only as the demo asks, each act logged: on the first day, in every other
//! settlement (the first, the third, ...), the eldest of its largest household comes to know the
//! M10 vocabulary that M11 stands on and the M11 vocabulary by the god tool (ADR-0008 §6,
//! `INTRODUCED`); the others are left as founded, the control. Resolution is by focus on the first
//! settlement, as the M9 and M10 demos have it. Nothing else is set and nothing is nudged.
//!
//! Each year it prints, settlement by settlement, what of the M11 vocabulary is in use there: who
//! knows any of it, dynamos held, stations, lines of current (strung of begun), homes hooked up
//! and lit on their last evening, gas engines, telegraph offices, telegraph wires (strung of
//! begun), telegrams waiting, motor carts, and the M11 law in force; and world-wide what has been
//! paid by the meter and in telegram fees; and for each taught settlement the way there: how many
//! households knowing any of it want a dynamo, a gas engine or a telegraph instrument, what they
//! hold of copper ore, copper, iron ore and iron (and keep of the last two) and coal gas, and the
//! water installations owned there. At the
//! end it prints the M11 laws proposed, saves,
//! loads, and says whether the control settlements took any of it up.
//!
//! ```sh
//! cargo run --release -p civ-sim --example m11_world -- <saves folder> [groups] [people] [years] [size]
//! ```

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use civ_agents::ledger::Channel;
use civ_agents::params::PowerSource;
use civ_agents::polity::{LawStatus, PolicyKind};
use civ_core::PermanentId;
use civ_core::time::DAYS_PER_YEAR;
use civ_sim::{NewWorld, SPEED_MAX, Sim, persist};
use commons_persist::{SaveDir, SaveKind};

/// Founding groups, unless given: three that learn and three that do not.
const GROUPS: u32 = 6;
/// People in each founding group, unless given.
const PEOPLE: u32 = 120;
/// Years lived, unless given: the time box.
const YEARS: i64 = 15;
/// The map's side in cells, unless given.
const SIZE: u32 = 2048;
const DAY: i64 = 24 * 60;
const NAME: &str = "Current and Wire";

/// What M11 stands on (M10's power, metals and the charcoal they are smelted and forged with, gas
/// and letters) and M11's own vocabulary (plan §7, M11; ADR-0036): the dynamo, the gas engine, the
/// road engine and telegraphy.
const VOCABULARY: [&str; 17] = [
    "core:technique/water_wheel",
    "core:technique/charcoal_burning",
    "core:technique/coal_burning",
    "core:technique/coking",
    "core:technique/bloomery",
    "core:technique/forging",
    "core:technique/copper_smelting",
    "core:technique/lamps",
    "core:technique/gas_making",
    "core:technique/reckoning",
    "core:technique/reading",
    "core:technique/writing",
    "core:technique/wheel_and_axle",
    "core:technique/dynamo",
    "core:technique/gas_engine",
    "core:technique/road_engine",
    "core:technique/telegraphy",
];

/// The law kinds M11 brought.
const LAWS: [PolicyKind; 1] = [PolicyKind::LayCurrent];

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(saves) = args.next().map(PathBuf::from) else {
        eprintln!("usage: m11_world <saves folder> [groups] [people] [years] [size]");
        return ExitCode::FAILURE;
    };
    let groups: u32 = args.next().and_then(|s| s.parse().ok()).unwrap_or(GROUPS);
    let people: u32 = args.next().and_then(|s| s.parse().ok()).unwrap_or(PEOPLE);
    let years: i64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(YEARS);
    let size: u32 = args.next().and_then(|s| s.parse().ok()).unwrap_or(SIZE);
    match make(&saves, groups, people, years, size) {
        // Each line was printed as it came, so a run cut short still says how far it got.
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("the demo failed: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Prints a line of the demo's record as it comes.
fn say(line: String) {
    use std::io::Write;
    println!("{line}");
    let _ = std::io::stdout().flush();
}

/// The eldest living member of settlement `s`'s largest household.
fn eldest_of_largest(sim: &Sim, s: PermanentId) -> Option<PermanentId> {
    let now = sim.now();
    let pop = sim.people();
    pop.households
        .iter()
        .map(|(_, h)| h)
        .filter(|h| h.settlement == Some(s) && !h.members.is_empty())
        .max_by_key(|h| (h.members.len(), std::cmp::Reverse(h.id)))
        .and_then(|h| {
            h.members
                .iter()
                .filter_map(|&m| pop.person(m))
                .max_by(|a, b| a.age_years(now).total_cmp(&b.age_years(now)))
                .map(|p| p.id)
        })
}

fn settlement_name(sim: &Sim, s: PermanentId) -> String {
    sim.land()
        .settlements
        .iter()
        .find(|x| x.id == s)
        .map_or_else(|| "another settlement".to_owned(), |x| x.name.clone())
}

/// What of the M11 vocabulary settlement `s` uses now, in words, and whether it uses any.
fn in_use(sim: &Sim, s: PermanentId, known: &[usize]) -> (String, bool) {
    let pop = sim.people();
    let land = sim.land();
    let goods = &sim.rules().catalog.goods;
    let households: Vec<_> = pop
        .households
        .iter()
        .map(|(_, h)| h)
        .filter(|h| h.settlement == Some(s))
        .collect();
    let here = |id: PermanentId| households.iter().any(|h| h.id == id);
    let residents: usize = households.iter().map(|h| h.members.len()).sum();
    let knowers = households
        .iter()
        .flat_map(|h| h.members.iter())
        .filter(|&&m| {
            pop.person(m)
                .is_some_and(|p| known.iter().any(|&t| p.knows(t)))
        })
        .count();
    let held = |pick: &dyn Fn(&civ_agents::params::GoodDef) -> bool| -> f64 {
        households
            .iter()
            .map(|h| {
                goods
                    .iter()
                    .enumerate()
                    .filter(|(_, d)| pick(d))
                    .map(|(g, _)| h.stores.get(g).copied().unwrap_or(0.0).floor())
                    .sum::<f64>()
            })
            .sum()
    };
    let dynamos = held(&|d| d.power.is_some_and(|p| p.source == PowerSource::Shaft));
    let gas_engines = held(&|d| {
        d.power
            .is_some_and(|p| p.source == PowerSource::Fuel && p.fuel.is_some())
    });
    let wires = &land.wires;
    let stations: BTreeSet<PermanentId> = wires
        .lines
        .iter()
        .filter(|l| here(l.station))
        .map(|l| l.station)
        .collect();
    let lines: Vec<_> = wires
        .lines
        .iter()
        .filter(|l| here(l.station) && l.length_m >= 1e-3)
        .collect();
    let strung = lines.iter().filter(|l| l.whole()).count();
    let hookups: Vec<_> = wires.hookups.iter().filter(|k| here(k.household)).collect();
    let lit = hookups
        .iter()
        .filter(|k| k.lit_until >= 0 && k.cause == civ_land::wire::Cause::Served)
        .count();
    let offices = households
        .iter()
        .filter(|h| {
            pop.office_of(&sim.rules().catalog, &sim.rules().people, sim.now(), h.id)
                .is_some()
        })
        .count();
    let telegraphs: Vec<_> = wires.telegraphs.iter().filter(|t| here(t.a)).collect();
    let wired = telegraphs.iter().filter(|t| t.whole()).count();
    let waiting = pop.reports.telegrams.keys().filter(|&&h| here(h)).count();
    let road_engine = sim
        .rules()
        .catalog
        .vehicle_index("core:vehicle/road_engine");
    let motor_carts = land
        .carts
        .list
        .iter()
        .filter(|c| here(c.owner) && c.made() && road_engine == Some(usize::from(c.vehicle)))
        .count();
    let laws: Vec<&str> = pop
        .polities
        .iter()
        .filter(|p| p.settlement == s)
        .flat_map(|p| p.laws.iter())
        .filter(|l| l.status == LawStatus::InForce && LAWS.contains(&l.kind))
        .map(|l| l.kind.name())
        .collect();
    let any = dynamos + gas_engines > 0.0
        || !lines.is_empty()
        || !hookups.is_empty()
        || offices > 0
        || !telegraphs.is_empty()
        || motor_carts > 0
        || !laws.is_empty();
    (
        format!(
            "{residents} people, {knowers} knowing any of it; {dynamos:.0} dynamos, {} stations, \
             {strung} of {} lines of current strung, {} homes hooked up ({lit} lit on their last \
             evening), {gas_engines:.0} gas engines; {offices} telegraph offices, {wired} of {} \
             telegraph wires strung, {waiting} telegrams waiting; {motor_carts} motor carts; \
             laws: {}",
            stations.len(),
            lines.len(),
            hookups.len(),
            telegraphs.len(),
            if laws.is_empty() {
                "none of M11's".to_owned()
            } else {
                laws.join(", ")
            }
        ),
        any,
    )
}

/// The way to the M11 vocabulary in taught settlement `s`, in words: the households whose people
/// know any of it, how many of them want a dynamo, a gas engine or a telegraph instrument (what
/// they keep of it, from their stores now), what they hold of what those are made of, and the
/// water installations its households own, where a dynamo could be set. Why none is built yet, if
/// none is.
fn way_there(sim: &mut Sim, s: PermanentId, known: &[usize]) -> String {
    let index = |id: &str| sim.rules().catalog.good_index(id);
    let [dynamo, engine, instrument, ore, copper, iron_ore, iron, gas] = [
        "core:good/dynamo",
        "core:good/gas_engine",
        "core:good/telegraph_instrument",
        "core:good/copper_ore",
        "core:good/copper",
        "core:good/iron_ore",
        "core:good/iron",
        "core:good/coal_gas",
    ]
    .map(index);
    let homes: Vec<PermanentId> = sim
        .people()
        .households
        .iter()
        .map(|(_, h)| h)
        .filter(|h| h.settlement == Some(s))
        .filter(|h| {
            h.members.iter().any(|&m| {
                sim.people()
                    .person(m)
                    .is_some_and(|p| known.iter().any(|&t| p.knows(t)))
            })
        })
        .map(|h| h.id)
        .collect();
    let keeps: Vec<Vec<f64>> = homes
        .iter()
        .map(|&h| sim.with_ctx_for_tests(|pop, ctx| pop.keep_for_tests(ctx, h)))
        .collect();
    let wanting = |g: Option<usize>| {
        g.map_or(0, |g| {
            keeps
                .iter()
                .filter(|k| k.get(g).is_some_and(|&x| x > 0.0))
                .count()
        })
    };
    let kept = |g: Option<usize>| -> f64 {
        g.map_or(0.0, |g| {
            keeps.iter().map(|k| k.get(g).copied().unwrap_or(0.0)).sum()
        })
    };
    let pop = sim.people();
    let held = |g: Option<usize>| -> f64 {
        g.map_or(0.0, |g| {
            homes
                .iter()
                .filter_map(|&h| pop.household(h))
                .map(|h| h.stores.get(g).copied().unwrap_or(0.0))
                .sum()
        })
    };
    let ours: BTreeSet<PermanentId> = pop
        .households
        .iter()
        .filter(|(_, h)| h.settlement == Some(s))
        .map(|(_, h)| h.id)
        .collect();
    let installations = sim
        .land()
        .installations
        .iter()
        .filter(|i| ours.contains(&i.owner))
        .count();
    format!(
        "the way there: {} households knowing any of it; wanting a dynamo {}, a gas engine {}, a \
         telegraph instrument {}; holding copper ore {:.0} kg, copper {:.0} kg, iron ore {:.0} \
         of {:.0} kg kept, iron {:.0} of {:.0} kg kept, coal gas {:.0}; {installations} water \
         installations owned here",
        homes.len(),
        wanting(dynamo),
        wanting(engine),
        wanting(instrument),
        held(ore),
        held(copper),
        held(iron_ore),
        kept(iron_ore),
        held(iron),
        kept(iron),
        held(gas),
    )
}

/// Where the copper lies from settlement `s`'s hearth, in words: the nearest body of copper ore,
/// how far and whether it shows at the surface (where a walker passing within
/// `civ_agents::population::deposits::FIND_M` finds it), and how many lie on the map. Copper
/// is also picked where it weathers out on upland ground, which this does not measure.
fn copper_near(sim: &Sim, s: PermanentId) -> String {
    let Some(copper) = sim.rules().catalog.good_index("core:good/copper_ore") else {
        return "no copper ore in the content".to_owned();
    };
    let Some(hearth) = sim
        .land()
        .settlements
        .iter()
        .find(|x| x.id == s)
        .map(|x| x.hearth_m)
    else {
        return "no hearth".to_owned();
    };
    let bodies: Vec<_> = sim
        .land()
        .deposits
        .iter()
        .filter(|d| usize::from(d.body.good) == copper)
        .collect();
    let nearest = bodies
        .iter()
        .map(|d| {
            let (x, y) = (d.body.at_cm.0 as f64 / 100.0, d.body.at_cm.1 as f64 / 100.0);
            (
                (x - f64::from(hearth.0)).hypot(y - f64::from(hearth.1)),
                d.body.exposed,
            )
        })
        .min_by(|a, b| a.0.total_cmp(&b.0));
    match nearest {
        Some((m, exposed)) => format!(
            "copper: the nearest body of its ore {:.1} km from the hearth, {}; {} on the map",
            m / 1000.0,
            if exposed {
                "showing at the surface"
            } else {
                "buried"
            },
            bodies.len()
        ),
        None => "copper: no body of its ore on the map".to_owned(),
    }
}

/// What has been paid on `channel`, kcal of food, world-wide.
fn paid(sim: &Sim, channel: Channel) -> f64 {
    let goods = &sim.rules().catalog.goods;
    (0..goods.len())
        .map(|g| sim.people().transfers.get(channel, g) * goods[g].kcal_per_kg)
        .sum()
}

fn make(saves: &Path, groups: u32, people: u32, years: i64, size: u32) -> Result<(), String> {
    let root = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
        .ok_or("content/ is not above the crate")?;
    let mut content = civ_content::load(&root)
        .registry
        .ok_or("the content does not load: run `civ-host content validate`")?;
    // Founded large: the content's band limits widened for this run only, as the bench does.
    let band = &mut content.people.params.band;
    band.max_size = band.max_size.max(people);
    band.min_size = band.min_size.min(people);
    let made = Instant::now();
    let mut sim = Sim::create_for_tests(
        &NewWorld {
            name: NAME.to_owned(),
            seed: 1,
            preset_id: content.default_preset().id.clone(),
            size_cells: size,
            band_size: people,
            neighbours: vec![people; groups.saturating_sub(1) as usize],
            neighbours_known: true,
            regime_id: String::new(),
        },
        &content,
        // A fixed identity, so the demo lives the same life every run.
        [11; 16],
    )
    .map_err(|e| e.to_string())?;
    if let Some(problem) = sim.founding_problem() {
        return Err(format!("the founding groups could not settle: {problem}"));
    }
    let settlements: Vec<PermanentId> = sim.land().settlements.iter().map(|s| s.id).collect();
    say(format!(
        "world (seed 1, {size} cells): {} people in {} settlements, founded in {:.1} s",
        sim.people().living(),
        settlements.len(),
        made.elapsed().as_secs_f64()
    ));
    let first = *settlements.first().ok_or("nothing was founded")?;
    sim.set_speed(SPEED_MAX).map_err(|e| e.to_string())?;
    sim.set_resolution_by_focus(true);
    sim.set_focus(Some(first));
    // The observer: the vocabulary to one person of every other settlement.
    let known: Vec<usize> = VOCABULARY
        .iter()
        .filter_map(|t| sim.rules().catalog.technique_index(t))
        .collect();
    if known.len() != VOCABULARY.len() {
        return Err("the content lacks a technique of the demo's vocabulary".to_owned());
    }
    let mut taught: BTreeSet<PermanentId> = BTreeSet::new();
    for (i, &s) in settlements.iter().enumerate() {
        let name = settlement_name(&sim, s);
        if i % 2 == 1 {
            say(format!("{name}: the control, as founded"));
            continue;
        }
        let Some(p) = eldest_of_largest(&sim, s) else {
            continue;
        };
        for t in VOCABULARY {
            sim.introduce_technique(p, t, false)?;
        }
        taught.insert(s);
        say(format!(
            "day {}: INTRODUCED (observer): {} of {name} comes to know the M11 vocabulary and \
             what it stands on ({} techniques)",
            sim.now().day_index(),
            sim.people().name_of(p),
            VOCABULARY.len()
        ));
        say(format!("  {}", copper_near(&sim, s)));
    }
    let started = Instant::now();
    sim.advance_to_midnight().map_err(|e| e.to_string())?;
    say(format!(
        "the founding day lived to midnight in {:.1} s",
        started.elapsed().as_secs_f64()
    ));
    let world_dir = saves.join(persist::world_dir_name(sim.meta()));
    let dir = SaveDir::create(world_dir, civ_schema::SAVE_EXTENSION).map_err(|e| e.to_string())?;
    let days = years * DAYS_PER_YEAR;
    let mut walls: Vec<f64> = Vec::with_capacity(days as usize);
    for day in 1..=days {
        let t = Instant::now();
        sim.advance_minutes(DAY).map_err(|e| e.to_string())?;
        walls.push(t.elapsed().as_secs_f64());
        if sim.people().living() == 0 {
            say(format!("day {}: the land is empty", sim.now().day_index()));
            break;
        }
        if day % DAYS_PER_YEAR == 0 {
            let recent = &walls[walls.len().saturating_sub(DAYS_PER_YEAR as usize)..];
            say(format!(
                "year {}: {} people in {} settlements; {:.2} s a day; paid by the meter {:.0} \
                 kcal, in telegram fees {:.0} kcal",
                day / DAYS_PER_YEAR,
                sim.people().living(),
                sim.land().settlements.len(),
                recent.iter().sum::<f64>() / recent.len().max(1) as f64,
                paid(&sim, Channel::Wire),
                paid(&sim, Channel::Telegraph)
            ));
            let settlements: Vec<(PermanentId, String)> = sim
                .land()
                .settlements
                .iter()
                .map(|s| (s.id, s.name.clone()))
                .collect();
            for (id, name) in settlements {
                let (words, _) = in_use(&sim, id, &known);
                let mark = if taught.contains(&id) {
                    "taught"
                } else {
                    "control"
                };
                say(format!("  {name} ({mark}): {words}"));
                if taught.contains(&id) {
                    say(format!("    {}", way_there(&mut sim, id, &known)));
                }
            }
        }
    }
    say(format!(
        "lived {} days, to day {}; {} people",
        walls.len(),
        sim.now().day_index(),
        sim.people().living()
    ));
    // The M11 laws put to a gathering, whoever proposed them and how they stand.
    say("the M11 laws proposed:".to_owned());
    let mut proposed = 0usize;
    for p in &sim.people().polities {
        for l in p.laws.iter().filter(|l| LAWS.contains(&l.kind)) {
            say(format!(
                "  day {} at {}: {} proposed by {}: {:?}",
                l.proposed.day_index(),
                settlement_name(&sim, p.settlement),
                l.kind.name(),
                sim.people().name_of(l.sponsor),
                l.status
            ));
            proposed += 1;
        }
    }
    if proposed == 0 {
        say("  none".to_owned());
    }
    // The control: whether any of it reached where nobody was taught (ADR-0036: a world without
    // them lives as before), as people may carry it there.
    for s in &sim.land().settlements {
        if taught.contains(&s.id) {
            continue;
        }
        let (words, any) = in_use(&sim, s.id, &known);
        say(format!(
            "control {}: {}: {words}",
            s.name,
            if any { "took some up" } else { "as before" }
        ));
    }
    let mut sorted = walls.clone();
    sorted.sort_by(f64::total_cmp);
    let mean = walls.iter().sum::<f64>() / walls.len().max(1) as f64;
    let at = |q: f64| sorted[((sorted.len() as f64 - 1.0) * q).round() as usize];
    say(format!(
        "speed: {mean:.2} s a day on average (p50 {:.2} s, p95 {:.2} s), {:.3} simulated years a \
         real minute",
        at(0.5),
        at(0.95),
        60.0 / (mean * DAYS_PER_YEAR as f64)
    ));
    let saved = persist::save(&mut sim, &dir, SaveKind::Manual, "After the years")
        .map_err(|e| e.to_string())?;
    let bytes = std::fs::metadata(&saved.path).map_or(0, |m| m.len());
    let loaded = persist::load(&saved.path, &content).map_err(|e| e.to_string())?;
    say(format!(
        "saved ({:.1} MB) and loaded: {} people against {}",
        bytes as f64 / 1e6,
        loaded.people().living(),
        sim.people().living()
    ));
    Ok(())
}
