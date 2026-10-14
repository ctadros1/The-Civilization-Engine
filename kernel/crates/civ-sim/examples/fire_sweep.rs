//! Activity-ignition sensitivity sweep (research 12-03 §2.5, §5.7).
//! Run with `--years 5 --seed 2 --people 40`; output is CSV.
//! Nominal frequencies scale core hazards relative to its 10/K household-year reference.
//! Actual frequencies follow fire use and moisture; no fires or outcomes are prescribed.

use std::path::Path;

use civ_agents::{Behavior, fire::Kindled};
use civ_core::time::MINUTES_PER_DAY;
use civ_sim::{NewWorld, SPEED_MAX, Sim};

fn main() -> Result<(), String> {
    let (mut years, mut seed, mut people) = (5_u32, 2_u64, 40_u32);
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| format!("missing value for {flag}"))?;
        match flag.as_str() {
            "--years" => years = value.parse().map_err(|_| "invalid years")?,
            "--seed" => seed = value.parse().map_err(|_| "invalid seed")?,
            "--people" => people = value.parse().map_err(|_| "invalid people")?,
            _ => return Err(format!("unknown option {flag}")),
        }
    }
    if !(1..=50).contains(&years) || !(10..=200).contains(&people) {
        return Err("years must be 1–50 and people 10–200".into());
    }
    let root = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
        .ok_or("content root missing")?;
    let content = civ_content::load(&root)
        .registry
        .ok_or("content failed to load")?;
    eprintln!(
        "Fire sweep: save schema {}, content API {}, wire 1.{}, base content {}; no observer inputs, daily household exposure.",
        civ_schema::SAVE_SCHEMA_VERSION,
        civ_content::KERNEL_CONTENT_API,
        civ_schema::WIRE_SCHEMA_MINOR,
        commons_persist::hex(&content.fingerprint)
    );
    println!(
        "seed,years,nominal_per_k,household_years,escapes,actual_per_k,put_out,roof_fires,spread,deaths,goods_kg,water_l,strip_records,salvage_records,pulled,response_work_h,noticed_roof_fires,unnoticed_roof_fires,first_notice_mean_min,living"
    );
    for nominal in [1.0, 3.0, 10.0, 30.0, 100.0] {
        let mut c = content.clone();
        // As in bench_village, only the harness permits an explicitly requested band size.
        c.people.params.band.min_size = c.people.params.band.min_size.min(people);
        c.people.params.band.max_size = c.people.params.band.max_size.max(people);
        let multiplier = nominal / 10.0;
        let f = &mut c.people.params.fire;
        f.hearth_lit_per_hour *= multiplier;
        f.hearth_banked_per_hour *= multiplier;
        f.worked_per_hour *= multiplier;
        let spec = NewWorld {
            name: "Fire sensitivity".into(),
            seed,
            preset_id: "core:worldgen/river_valley".into(),
            size_cells: 512,
            band_size: people,
            neighbours: Vec::new(),
            neighbours_known: false,
            regime_id: String::new(),
        };
        let mut identity = [7; 16];
        identity[..8].copy_from_slice(&seed.to_le_bytes());
        let mut sim = Sim::create_for_tests(&spec, &c, identity).map_err(|e| e.to_string())?;
        if let Some(problem) = sim.founding_problem() {
            return Err(problem.to_owned());
        }
        sim.set_speed(SPEED_MAX).map_err(|e| e.to_string())?;
        let mut household_days = 0_u64;
        for _ in 0..years * 365 {
            household_days += sim
                .people()
                .households
                .iter()
                .filter(|(_, h)| !h.members.is_empty())
                .count() as u64;
            sim.advance_minutes(MINUTES_PER_DAY)
                .map_err(|e| e.to_string())?;
        }
        let pop = sim.people();
        let fire = &pop.fires;
        let hhy = household_days as f64 / 365.0;
        let put_out = fire
            .ignitions
            .iter()
            .filter(|i| i.put_out_by.is_some())
            .count();
        let spread = fire
            .list
            .iter()
            .filter(|f| matches!(f.kindled, Kindled::Neighbour { .. }))
            .count();
        let deaths: usize = fire.list.iter().map(|f| f.deaths.len()).sum();
        let goods = fire.list.iter().map(|f| f.goods_kg).sum::<f32>().max(0.0);
        let water = fire.thrown.iter().map(|t| t.litres).sum::<f32>().max(0.0);
        let work = Behavior::ALL
            .iter()
            .zip(&pop.time_use.work)
            .filter(|(b, _)| {
                matches!(
                    b,
                    Behavior::Douse | Behavior::Strip | Behavior::Salvage | Behavior::Pull
                )
            })
            .map(|(_, m)| m)
            .sum::<f64>()
            / 60.0;
        let notices: Vec<_> = fire
            .list
            .iter()
            .filter_map(|f| {
                fire.knew
                    .iter()
                    .filter(|k| k.fire == f.id)
                    .map(|k| (k.at.minutes() - f.began.minutes()) as f64)
                    .min_by(f64::total_cmp)
            })
            .collect();
        let noticed = notices.len();
        let unnoticed = fire.list.len() - noticed;
        let notice = if notices.is_empty() {
            String::new()
        } else {
            format!("{:.2}", notices.iter().sum::<f64>() / notices.len() as f64)
        };
        println!(
            "{seed},{years},{nominal:.0},{hhy:.3},{},{:.3},{put_out},{},{spread},{deaths},{goods:.3},{water:.3},{},{},{},{work:.3},{noticed},{unnoticed},{notice},{}",
            fire.ignitions.len(),
            1000.0 * fire.ignitions.len() as f64 / hhy.max(f64::MIN_POSITIVE),
            fire.list.len(),
            fire.stripped.len(),
            fire.salvaged.len(),
            fire.pulled.len(),
            pop.living()
        );
    }
    Ok(())
}
