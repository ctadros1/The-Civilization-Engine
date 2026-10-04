//! `civ-host run`: lives one world for some years and reports, at each year's end, how its people
//! live: how many there are, what they hold, their tools and skills, and how adults spend their
//! days. A tool for calibration (plan §4.7), not a test: it checks nothing.

use std::collections::BTreeMap;
use std::io::Write;
use std::sync::atomic::AtomicBool;

use civ_agents::ledger::{Channel, Transfers};
use civ_agents::params::GoodUse;
use civ_agents::person::{Flow, Flows, Step};
use civ_content::ContentRegistry;
use civ_core::time::MINUTES_PER_YEAR;
use civ_sim::{NewWorld, Sim};

/// What to run.
#[derive(Clone, Debug)]
pub struct RunOptions {
    /// The world-generation preset, or the default one.
    pub preset: Option<String>,
    /// The world seed.
    pub seed: u64,
    /// Map side, cells.
    pub size: u32,
    /// Founding band size (0: the content's default).
    pub band: u32,
    /// Years to live.
    pub years: u32,
}

/// Lives the world and writes a report to `out`.
pub fn run(
    content: &ContentRegistry,
    options: &RunOptions,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    let preset = options
        .preset
        .clone()
        .unwrap_or_else(|| content.default_preset().id.clone());
    let mut sim = Sim::create(
        &NewWorld {
            name: format!("Run {}", options.seed),
            seed: options.seed,
            preset_id: preset.clone(),
            size_cells: options.size,
            band_size: options.band,
        },
        content,
        &mut |_| {},
        &AtomicBool::new(false),
    )
    .map_err(|e| anyhow::anyhow!("generation failed: {e}"))?;
    if let Some(problem) = sim.founding_problem() {
        anyhow::bail!("the band could not settle: {problem}");
    }
    writeln!(
        out,
        "{preset} seed {} on a {}-cell map: {} people, {}",
        options.seed,
        options.size,
        sim.people().living(),
        sim.date()
    )?;
    let start = sim.now().minutes();
    let mut flows_before = sim.people().flows();
    let mut moved_before = sim.people().transfers.clone();
    for year in 1..=options.years {
        let mut hours: BTreeMap<u16, f64> = BTreeMap::new();
        let mut adult_hours = 0.0;
        let end = start + i64::from(year) * MINUTES_PER_YEAR;
        while sim.now().minutes() < end {
            let step = (end - sim.now().minutes()).min(60);
            sim.advance_minutes(step)
                .map_err(|e| anyhow::anyhow!("the world stopped: {e}"))?;
            let now = sim.now();
            for (_, p) in sim.people().people.iter() {
                if p.age_years(now) < 15.0 {
                    continue;
                }
                let h = step as f64 / 60.0;
                adult_hours += h;
                let doing = match p.act.steps.get(p.act.step as usize) {
                    Some(Step::Work { .. } | Step::Walk { .. } | Step::Deposit) => p.act.def,
                    _ => u16::MAX,
                };
                *hours.entry(doing).or_default() += h;
            }
        }
        report_year(&sim, year, start, &hours, adult_hours, out)?;
        let flows = sim.people().flows();
        report_food(&sim, &flows_before, &flows, out)?;
        flows_before = flows;
        let moved = sim.people().transfers.clone();
        report_moved(&sim, &moved_before, &moved, out)?;
        moved_before = moved;
    }
    Ok(())
}

/// What became of food over the year, millions of kilocalories: gathered and harvested, made and
/// put into recipes, eaten, spoiled, sown, and taken away by households that left.
fn report_food(
    sim: &Sim,
    before: &Flows,
    after: &Flows,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    let goods = &sim.rules().catalog.goods;
    let mcal = |flow: Flow| {
        goods
            .iter()
            .enumerate()
            .filter(|(_, g)| g.purpose == GoodUse::Food)
            .map(|(i, g)| (after.get(flow, i) - before.get(flow, i)) * g.kcal_per_kg)
            .sum::<f64>()
            / 1e6
    };
    writeln!(
        out,
        "  food (Mcal): got {:.1}, brought {:.1}; eaten {:.1}, spoiled {:.1}, sown {:.1}, left with \
         leavers {:.1}; recipes took {:.1} and made {:.1}",
        mcal(Flow::Got),
        mcal(Flow::Brought),
        mcal(Flow::Eaten),
        mcal(Flow::Spoiled),
        mcal(Flow::Sown),
        mcal(Flow::Departed),
        mcal(Flow::Used),
        mcal(Flow::Made),
    )?;
    Ok(())
}

/// Food moved between households over the year, millions of kilocalories, by channel.
fn report_moved(
    sim: &Sim,
    before: &Transfers,
    after: &Transfers,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    let goods = &sim.rules().catalog.goods;
    let lines: Vec<String> = Channel::ALL
        .iter()
        .map(|&c| {
            let mcal = goods
                .iter()
                .enumerate()
                .filter(|(_, g)| g.purpose == GoodUse::Food)
                .map(|(i, g)| (after.get(c, i) - before.get(c, i)) * g.kcal_per_kg)
                .sum::<f64>()
                / 1e6;
            format!("{} {mcal:.1}", c.label())
        })
        .collect();
    writeln!(
        out,
        "  food moved between households (Mcal): {}",
        lines.join(", ")
    )?;
    Ok(())
}

fn report_year(
    sim: &Sim,
    year: u32,
    start: i64,
    hours: &BTreeMap<u16, f64>,
    adult_hours: f64,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    let now = sim.now();
    let pop = sim.people();
    let catalog = &sim.rules().catalog;
    let goods = &catalog.goods;
    let from = start + i64::from(year - 1) * MINUTES_PER_YEAR;
    let within = |t: Option<civ_core::SimTime>| t.is_some_and(|t| t.minutes() >= from);
    let born = pop
        .records
        .values()
        .filter(|r| r.born.minutes() >= from)
        .count();
    let died = pop
        .records
        .values()
        .filter(|r| within(r.died.map(|(t, _)| t)))
        .count();
    let left = pop.records.values().filter(|r| within(r.left)).count();
    let households: Vec<_> = pop
        .households
        .iter()
        .map(|(_, h)| h)
        .filter(|h| !h.members.is_empty())
        .collect();
    writeln!(
        out,
        "\nYear {year} ({}): {} people in {} households; {born} born, {died} died, {left} left",
        sim.date(),
        pop.living(),
        households.len()
    )?;
    // Harvests and shortages noted this year, and the fields people hold.
    let noted = |kind: civ_agents::ChronicleKind| {
        pop.chronicle
            .iter()
            .filter(|e| e.kind == kind && e.at.minutes() >= from)
            .map(|e| e.number)
            .collect::<Vec<f64>>()
    };
    let harvests = noted(civ_agents::ChronicleKind::HarvestIn);
    let short = noted(civ_agents::ChronicleKind::FoodRanShort);
    let held: Vec<_> = sim
        .land()
        .fields
        .iter()
        .filter(|f| {
            pop.household(f.household)
                .is_some_and(|h| !h.members.is_empty())
        })
        .collect();
    let area: f64 = held.iter().map(|f| f.area_ha()).sum();
    writeln!(
        out,
        "  harvest {:.0} kg; {} fields, {area:.1} ha held; food ran short {} times",
        harvests.iter().sum::<f64>(),
        held.len(),
        short.len()
    )?;
    // Stores, summed over households.
    let mut totals = vec![0.0; goods.len()];
    for h in &households {
        let stores = civ_agents::population::stores_now(h, now, &sim.rules().people, goods);
        for (t, s) in totals.iter_mut().zip(stores) {
            *t += s.max(0.0);
        }
    }
    let line = |purpose: GoodUse| {
        goods
            .iter()
            .zip(&totals)
            .filter(|(g, t)| g.purpose == purpose && **t > 0.005)
            .map(|(g, t)| {
                if purpose == GoodUse::Tool {
                    format!("{} {t:.1}", g.name)
                } else {
                    format!("{} {t:.0}", g.name)
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    writeln!(out, "  food (kg): {}", line(GoodUse::Food))?;
    writeln!(
        out,
        "  fuel and materials (kg): {}; {}",
        line(GoodUse::Fuel),
        line(GoodUse::Material)
    )?;
    writeln!(out, "  tools (standard tools): {}", line(GoodUse::Tool))?;
    let lacking: Vec<String> = goods
        .iter()
        .enumerate()
        .filter(|(_, g)| g.tool.as_ref().is_some_and(|t| t.per_worker > 0.0))
        .map(|(i, g)| {
            let without = households
                .iter()
                .filter(|h| h.stores.get(i).copied().unwrap_or(0.0) < 0.02)
                .count();
            format!("{} {without}", g.name)
        })
        .collect();
    writeln!(out, "  households without: {}", lacking.join(", "))?;
    // Skills of adults.
    let adults: Vec<_> = pop
        .people
        .iter()
        .map(|(_, p)| p)
        .filter(|p| p.age_years(now) >= 15.0)
        .collect();
    let skills: Vec<String> = catalog
        .skills
        .iter()
        .enumerate()
        .map(|(k, s)| {
            let levels: Vec<f64> = adults.iter().map(|p| p.skill(k)).collect();
            let mean = levels.iter().sum::<f64>() / levels.len().max(1) as f64;
            let best = levels.iter().copied().fold(0.0, f64::max);
            format!("{} {mean:.2} (best {best:.2})", s.name)
        })
        .collect();
    writeln!(out, "  adult skills: {}", skills.join(", "))?;
    // Days: hours per adult per day by activity.
    let adult_days = (adult_hours / 24.0).max(1e-9);
    let mut by: Vec<(String, f64)> = hours
        .iter()
        .map(|(&def, &h)| {
            let name = catalog
                .activities
                .get(usize::from(def))
                .map_or_else(|| "waiting".to_owned(), |a| a.name.clone());
            (name, h / adult_days)
        })
        .collect();
    by.sort_by(|a, b| b.1.total_cmp(&a.1));
    let day: Vec<String> = by
        .iter()
        .filter(|(_, h)| *h >= 0.05)
        .map(|(n, h)| format!("{n} {h:.1}"))
        .collect();
    writeln!(out, "  an adult's day (h): {}", day.join(", "))?;
    Ok(())
}
