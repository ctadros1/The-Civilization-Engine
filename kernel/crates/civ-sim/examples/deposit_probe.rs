//! M10d slice EG's probe (ADR-0035 §9: the deposit-demand probe run again in towns; plan §7 CC2,
//! deferred 2026-10-17 when about 40,000 household-days in villages found no household holding
//! more than its roof kept): two settlements founded large on a river valley, as the M8 demo
//! founds them, lived at Max. Each midnight, for every household with members, it counts what of
//! its goods that keep better under a roof lies in the open beyond the room its roofed buildings
//! and pots give (`Keeping::shares`), what that loses a day, and the room it has to spare; and
//! each polity's store, with whether a storehouse of its stands. Nothing here changes what anyone
//! does; it prints the counts.
//!
//! Demand, decided before running: a housed household with at least `CARRY_KG` in the open on at
//! least 1 % of housed household-days, while another household of its settlement has room for
//! it. Else custody stays deferred.
//!
//! ```sh
//! cargo run --profile simcheck -p civ-sim --example deposit_probe -- [seed] [people] [days]
//! ```

use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

use civ_agents::population::stores_now;
use civ_core::PermanentId;
use civ_sim::{NewWorld, SPEED_MAX, Sim};

/// Minutes in a day.
const DAY: i64 = 1440;
/// People founding each of the two settlements, unless given.
const PEOPLE: u32 = 400;
/// Days lived, unless given.
const DAYS: i64 = 720;
/// Kilograms one person carries: what in the open would be worth carrying to a keeper.
const CARRY_KG: f64 = 50.0;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let seed = args.next().and_then(|s| s.parse().ok()).unwrap_or(1);
    let people = args.next().and_then(|s| s.parse().ok()).unwrap_or(PEOPLE);
    let days = args.next().and_then(|s| s.parse().ok()).unwrap_or(DAYS);
    match run(seed, people, days) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("deposit_probe: {e}");
            ExitCode::FAILURE
        }
    }
}

#[derive(Default)]
struct Tally {
    /// Household-days with members, under a roof of their own, and not yet.
    housed: u64,
    unhoused: u64,
    /// Housed household-days with any (>= 1 kg) and with a carry in the open.
    open_any: u64,
    open_carry: u64,
    /// Of those with a carry in the open, the days another household of the settlement had
    /// room for it.
    open_carry_room: u64,
    /// Kilogram-days in the open, kilograms lost to lying there, and the most in the open on a
    /// day.
    open_kg_days: f64,
    lost_kg: f64,
    open_max_kg: f64,
    /// Housed household-days with a carry of room to spare.
    spare_carry: u64,
    /// Polity-days, those with a storehouse standing, and the kilograms their stores held.
    store_days: u64,
    storehouse_days: u64,
    store_kg_days: f64,
    store_max_kg: f64,
    /// People at the end.
    people: usize,
}

fn run(seed: u64, people: u32, days: i64) -> Result<(), String> {
    let root = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
        .ok_or("content/ is not above the crate")?;
    let mut content = civ_content::load(&root)
        .registry
        .ok_or("the content does not load: run `civ-host content validate`")?;
    // Founded large: the content's band limits widened for this run only, as the M8 demo does.
    let band = &mut content.people.params.band;
    band.max_size = band.max_size.max(people);
    band.min_size = band.min_size.min(people);
    let mut sim = Sim::create_for_tests(
        &NewWorld {
            name: "Deposits".to_owned(),
            seed,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 1024,
            band_size: people,
            neighbours: vec![people],
            neighbours_known: true,
            regime_id: String::new(),
        },
        &content,
        [7; 16],
    )
    .map_err(|e| e.to_string())?;
    if let Some(problem) = sim.founding_problem() {
        return Err(format!("the founding groups could not settle: {problem}"));
    }
    sim.set_speed(SPEED_MAX).map_err(|e| e.to_string())?;
    sim.advance_to_midnight().map_err(|e| e.to_string())?;
    let started = Instant::now();
    let mut t = Tally::default();
    for day in 0..days {
        sim.advance_minutes(DAY).map_err(|e| e.to_string())?;
        count(&sim, &mut t);
        if (day + 1) % 90 == 0 {
            eprintln!(
                "day {}: {} housed household-days, {} with a carry in the open ({:.0} s)",
                day + 1,
                t.housed,
                t.open_carry,
                started.elapsed().as_secs_f64()
            );
        }
    }
    t.people = sim.people().people.len();
    let share = |n: u64| 100.0 * n as f64 / t.housed.max(1) as f64;
    println!("deposit probe: seed {seed}, two settlements of {people}, {days} days");
    println!(
        "household-days: {} housed, {} not yet under a roof of their own",
        t.housed, t.unhoused
    );
    println!(
        "in the open: any on {} ({:.2} %), a carry ({CARRY_KG} kg) on {} ({:.2} %), {} of them \
         with a neighbour's room for it",
        t.open_any,
        share(t.open_any),
        t.open_carry,
        share(t.open_carry),
        t.open_carry_room
    );
    println!(
        "kilogram-days in the open {:.0}, most on a day {:.1} kg, lost to lying there {:.1} kg",
        t.open_kg_days, t.open_max_kg, t.lost_kg
    );
    println!(
        "room to spare: a carry or more on {} housed household-days ({:.1} %)",
        t.spare_carry,
        share(t.spare_carry)
    );
    println!(
        "common stores: {} polity-days, {} under a storehouse, mean {:.0} kg, most {:.0} kg",
        t.store_days,
        t.storehouse_days,
        t.store_kg_days / t.store_days.max(1) as f64,
        t.store_max_kg
    );
    println!("people at the end: {}", t.people);
    let demand = share(t.open_carry_room) >= 1.0;
    println!(
        "demand (a carry in the open, a neighbour with room, on >= 1 % of housed \
         household-days): {}",
        if demand { "yes" } else { "no" }
    );
    Ok(())
}

/// The day's counts.
fn count(sim: &Sim, t: &mut Tally) {
    let rules = sim.rules();
    let goods = &rules.catalog.goods;
    let now = sim.now();
    let rate = |h: f64| {
        if h > 0.0 {
            std::f64::consts::LN_2 / h
        } else {
            0.0
        }
    };
    // Per household: its settlement, kilograms in the open, and room to spare.
    let mut rows: Vec<(PermanentId, f64, f64)> = Vec::new();
    for (_, x) in sim.people().households.iter() {
        let Some(settlement) = x.settlement else {
            continue;
        };
        if x.members.is_empty() {
            continue;
        }
        if !x.sheltered {
            t.unhoused += 1;
            continue;
        }
        t.housed += 1;
        let stores = stores_now(x, now, &rules.people, goods);
        let keeping = x.keeping.with_stores(&stores, goods);
        let shares = keeping.shares(&stores, goods);
        let (mut open, mut lost, mut kept) = (0.0, 0.0, 0.0);
        for (g, d) in goods.iter().enumerate() {
            if d.sheltered_half_life_days <= 0.0 {
                continue;
            }
            let kg = stores.get(g).copied().unwrap_or(0.0).max(0.0);
            let (r, s) = shares.get(g).copied().unwrap_or((0.0, 0.0));
            let out = kg * (1.0 - r - s).max(0.0);
            open += out;
            kept += kg - out;
            lost += out * (rate(d.half_life_days) - rate(d.sheltered_half_life_days)).max(0.0);
        }
        let room = keeping.raised_kg.max(0.0) + keeping.roofed_kg.max(0.0);
        let spare = if room.is_finite() {
            (room - kept).max(0.0)
        } else {
            f64::INFINITY
        };
        rows.push((settlement, open, spare));
        t.open_kg_days += open;
        t.lost_kg += lost;
        t.open_max_kg = t.open_max_kg.max(open);
        if open >= 1.0 {
            t.open_any += 1;
        }
        if spare >= CARRY_KG {
            t.spare_carry += 1;
        }
    }
    for (i, &(s, open, _)) in rows.iter().enumerate() {
        if open < CARRY_KG {
            continue;
        }
        t.open_carry += 1;
        if rows
            .iter()
            .enumerate()
            .any(|(j, &(s2, _, spare))| j != i && s2 == s && spare >= open)
        {
            t.open_carry_room += 1;
        }
    }
    for p in &sim.people().polities {
        let kg: f64 = p
            .stores
            .iter()
            .zip(goods)
            .filter(|(_, d)| d.sheltered_half_life_days > 0.0)
            .map(|(kg, _)| kg.max(0.0))
            .sum::<f64>()
            + 0.0;
        t.store_days += 1;
        t.store_kg_days += kg;
        t.store_max_kg = t.store_max_kg.max(kg);
        if p.storehouse.is_some() {
            t.storehouse_days += 1;
        }
    }
}
