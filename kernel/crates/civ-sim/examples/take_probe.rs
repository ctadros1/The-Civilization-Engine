//! M6c slice BI's probe (ADR-0023 §2: measure before targets widen): three villages founded
//! together, lived three years at Max. On the first of each month, for every household short of
//! its food target, it compares the best store it could take from at home with the best among the
//! homes of settlements its members have seen, as `take_target` ranks them. Nothing here changes
//! what anyone does; it prints the counts.
//!
//! ```sh
//! cargo run --release -p civ-sim --example take_probe -- [seed] [preset] [band]
//! ```

use std::process::ExitCode;
use std::sync::atomic::AtomicBool;

use civ_sim::{NewWorld, SPEED_MAX, Sim};

/// Minutes in a day.
const DAY: i64 = 1440;
/// Days lived.
const DAYS: i64 = 3 * 365;
/// Days between probes.
const EVERY: i64 = 30;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let seed = args.next().and_then(|s| s.parse().ok()).unwrap_or(1);
    let preset = args
        .next()
        .unwrap_or_else(|| "core:worldgen/river_valley".to_owned());
    let band = args.next().and_then(|s| s.parse().ok()).unwrap_or(100);
    match run(seed, &preset, band) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("take_probe: {e}");
            ExitCode::FAILURE
        }
    }
}

#[derive(Default)]
struct Tally {
    wanting: usize,
    seen_any: usize,
    home: usize,
    away: usize,
    away_more_food: usize,
    away_ranked_first: usize,
    away_only: usize,
    unreachable: usize,
    away_walks: Vec<f64>,
}

fn run(seed: u64, preset: &str, band: u32) -> Result<(), String> {
    let root = civ_content::find_content_root(std::path::Path::new(env!("CARGO_MANIFEST_DIR")))
        .ok_or("content/ is not above the crate")?;
    let content = civ_content::load(&root)
        .registry
        .ok_or("the content does not load")?;
    let mut sim = Sim::create(
        &NewWorld {
            name: "Probe".to_owned(),
            seed,
            preset_id: preset.to_owned(),
            size_cells: 768,
            band_size: band,
            neighbours: vec![band, band],
            neighbours_known: true,
            regime_id: String::new(),
        },
        &content,
        &mut |_| {},
        &AtomicBool::new(false),
    )
    .map_err(|e| e.to_string())?;
    sim.set_speed(SPEED_MAX).map_err(|e| e.to_string())?;
    sim.advance_to_midnight().map_err(|e| e.to_string())?;
    let hearths: Vec<String> = sim
        .land()
        .settlements
        .iter()
        .map(|s| format!("{} at ({:.0}, {:.0})", s.name, s.hearth_m.0, s.hearth_m.1))
        .collect();
    println!("seed {seed} {preset} band {band}: {}", hearths.join("; "));
    let mut total = Tally::default();
    let mut day = 0;
    while day < DAYS {
        sim.advance_minutes(EVERY * DAY)
            .map_err(|e| e.to_string())?;
        day += EVERY;
        let probes = sim.with_ctx_for_tests(|pop, ctx| pop.take_probe_for_tests(ctx));
        let mut t = Tally::default();
        for p in &probes {
            t.wanting += 1;
            t.seen_any += usize::from(p.seen > 0);
            t.home += usize::from(p.home.is_some());
            t.away += usize::from(p.away.is_some());
            t.unreachable += p.away_unreachable;
            if let Some((ak, ac, _)) = p.away {
                t.away_walks.push(ac);
                match p.home {
                    None => {
                        t.away_only += 1;
                        t.away_ranked_first += 1;
                    }
                    Some((hk, hc)) => {
                        if ak > hk + 1e-6 {
                            t.away_more_food += 1;
                            t.away_ranked_first += 1;
                        } else if ak >= hk - 1e-6 && ac < hc {
                            t.away_ranked_first += 1;
                        }
                    }
                }
            }
        }
        let people = sim.people().people.iter().count();
        let households = sim.people().households.iter().count();
        println!(
            "day {day:4}: {people:3} people in {households:2} households; {:2} short of food, \
             {:2} have seen homes elsewhere, {:2} have a store at home, {:2} one elsewhere, {:2} \
             would rank elsewhere first ({} only elsewhere, {} more food there); {} stores \
             elsewhere out of reach",
            t.wanting,
            t.seen_any,
            t.home,
            t.away,
            t.away_ranked_first,
            t.away_only,
            t.away_more_food,
            t.unreachable,
        );
        total.wanting += t.wanting;
        total.seen_any += t.seen_any;
        total.home += t.home;
        total.away += t.away;
        total.away_ranked_first += t.away_ranked_first;
        total.away_only += t.away_only;
        total.away_more_food += t.away_more_food;
        total.unreachable += t.unreachable;
        total.away_walks.extend(t.away_walks);
    }
    let mut walks = total.away_walks.clone();
    walks.sort_by(f64::total_cmp);
    // A taker walks at most the take activity's limit, so a store beyond it is never weighed.
    let take_max = sim
        .rules()
        .catalog
        .activities
        .iter()
        .find(|a| a.id == "core:activity/take_food")
        .map_or(f64::INFINITY, |a| f64::from(a.max_walk_minutes));
    let within = walks.iter().filter(|&&w| w <= take_max).count();
    println!("stores elsewhere within the {take_max:.0} minutes a taker walks: {within}");
    let median = walks.get(walks.len() / 2).copied().unwrap_or(f64::NAN);
    let incidents = sim.people().order.incidents.len();
    println!(
        "total: {} household-probes short of food; {} had seen homes elsewhere; {} had a store \
         at home, {} one elsewhere; {} would rank elsewhere first ({} only elsewhere, {} more \
         food there); median cost elsewhere {median:.0} min; takings at home over the run: \
         {incidents}",
        total.wanting,
        total.seen_any,
        total.home,
        total.away,
        total.away_ranked_first,
        total.away_only,
        total.away_more_food,
    );
    Ok(())
}
