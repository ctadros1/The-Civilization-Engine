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
        report_market(&sim, &moved_before, &moved, out)?;
        report_firms(&sim, civ_core::SimTime::from_minutes(start), out)?;
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

/// The year's exchange: what households offer, what changed hands by barter and by sale (both
/// sides of each trade), and each settlement's money, if it has one.
fn report_market(
    sim: &Sim,
    before: &Transfers,
    after: &Transfers,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    let pop = sim.people();
    let goods = &sim.rules().catalog.goods;
    let market = &sim.rules().people.market;
    let offering = pop
        .households
        .iter()
        .filter(|(_, h)| !h.offers.is_empty())
        .count();
    let mut offered: Vec<usize> = pop
        .households
        .iter()
        .flat_map(|(_, h)| h.offers.iter().map(|o| usize::from(o.good)))
        .collect();
    offered.sort_unstable();
    offered.dedup();
    let names = |list: &[usize]| {
        list.iter()
            .filter_map(|&g| goods.get(g).map(|d| d.name.clone()))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let moved = |c: Channel| {
        goods
            .iter()
            .enumerate()
            .filter_map(|(g, d)| {
                let v = after.get(c, g) - before.get(c, g);
                (v > 0.005).then(|| format!("{} {v:.1}", d.name))
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    let money: Vec<String> = pop
        .markets
        .iter()
        .map(|m| {
            let shares = m.acceptance();
            match m.money(market.money_share, market.money_min_trades) {
                Some(g) => format!(
                    "{} ({:.0}% of payments)",
                    goods.get(g).map_or("?", |d| d.name.as_str()),
                    shares[g] * 100.0
                ),
                None => "none".to_owned(),
            }
        })
        .collect();
    writeln!(
        out,
        "  trade: {offering} households offer {}; barter moved {}; sales moved {}; money: {}",
        if offered.is_empty() {
            "nothing".to_owned()
        } else {
            names(&offered)
        },
        or_none(moved(Channel::Barter)),
        or_none(moved(Channel::Sale)),
        if money.is_empty() {
            "none".to_owned()
        } else {
            money.join("; ")
        }
    )?;
    Ok(())
}

/// The workshops (slice J): which are open, which closed and why, and what they made and sold
/// since `start`, from their books' months.
fn report_firms(sim: &Sim, start: civ_core::SimTime, out: &mut dyn Write) -> anyhow::Result<()> {
    use civ_agents::firm::BookKind;
    let pop = sim.people();
    let goods = &sim.rules().catalog.goods;
    let name = |f: &civ_agents::firm::Firm| f.name(&pop.name_of(f.founder), goods);
    let open: Vec<String> = pop.firms.iter().filter(|f| f.is_open()).map(name).collect();
    let closed: Vec<String> = pop
        .firms
        .iter()
        .filter_map(|f| {
            f.closed
                .map(|(_, why)| format!("{} ({})", name(f), why.text()))
        })
        .collect();
    let months = civ_agents::market::month_of(start)..civ_agents::market::month_of(sim.now());
    let total = |kind: BookKind| {
        goods
            .iter()
            .enumerate()
            .filter_map(|(g, d)| {
                let v: f32 = pop
                    .firms
                    .iter()
                    .flat_map(|f| f.books.months.iter())
                    .filter(|m| months.contains(&m.month))
                    .map(|m| m.amount(kind, g as u16))
                    .sum();
                (v > 0.005).then(|| format!("{} {v:.1}", d.name))
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    let hours: (f32, f32) = pop
        .firms
        .iter()
        .flat_map(|f| f.books.months.iter())
        .filter(|m| months.contains(&m.month))
        .fold((0.0, 0.0), |(o, h), m| (o + m.owner_h, h + m.hired_h));
    writeln!(
        out,
        "  workshops: {} open{}; {} closed{}; made {}; sold {}; worked {:.0} h by their owners, {:.0} h hired",
        open.len(),
        if open.is_empty() {
            String::new()
        } else {
            format!(" ({})", open.join(", "))
        },
        closed.len(),
        if closed.is_empty() {
            String::new()
        } else {
            format!(" ({})", closed.join(", "))
        },
        or_none(total(BookKind::Made)),
        or_none(total(BookKind::Sold)),
        hours.0,
        hours.1
    )?;
    Ok(())
}

fn or_none(s: String) -> String {
    if s.is_empty() {
        "nothing".to_owned()
    } else {
        s
    }
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
