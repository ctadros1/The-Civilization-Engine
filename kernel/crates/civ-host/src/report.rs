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
use civ_core::PermanentId;
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
    /// The property regime, or the content's default.
    pub regime: Option<String>,
    /// Families the observer sends to the village as it is founded.
    pub families: u32,
    /// Techniques the observer introduces to the band's eldest grown founder as the world begins,
    /// by id.
    pub introduce: Vec<String>,
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
            regime_id: options.regime.clone().unwrap_or_default(),
        },
        content,
        &mut |_| {},
        &AtomicBool::new(false),
    )
    .map_err(|e| anyhow::anyhow!("generation failed: {e}"))?;
    if let Some(problem) = sim.founding_problem() {
        anyhow::bail!("the band could not settle: {problem}");
    }
    if options.families > 0 {
        let people = sim.send_families_to_hearth(options.families);
        writeln!(
            out,
            "{} families sent: {people} people came",
            options.families
        )?;
    }
    for technique in &options.introduce {
        let who = crate::commands::introduce(&mut sim, technique)?;
        writeln!(out, "the observer introduced {technique} to {who}")?;
    }
    writeln!(
        out,
        "{preset} seed {} on a {}-cell map under {}: {} people, {}",
        options.seed,
        options.size,
        sim.regime().name.to_lowercase(),
        sim.people().living(),
        sim.date()
    )?;
    let start = sim.now().minutes();
    let mut flows_before = sim.people().flows();
    let mut moved_before = sim.people().transfers.clone();
    let mut wealth_shown = 0;
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
        report_prices(&sim, out)?;
        report_firms(&sim, civ_core::SimTime::from_minutes(start), out)?;
        report_land(&sim, &moved_before, &moved, out)?;
        report_wealth(&sim, &mut wealth_shown, out)?;
        let year_began = civ_core::SimTime::from_minutes(end - MINUTES_PER_YEAR);
        report_crafts(&sim, year_began, out)?;
        moved_before = moved;
    }
    Ok(())
}

/// What people came to know and lost, what gave way and how builders answered, and how they built,
/// in the year since `since` (M3b): the chronicle's finds, crafts learnt beside a knower, crafts
/// lost and introduced; crafts known by five people or fewer, and who; what each settlement's
/// builders have seen of each technique that has failed there, and how they build for it; the
/// buildings begun in the year and those built after an admired one; and the roof pitches of
/// every building and every household's taste.
fn report_crafts(sim: &Sim, since: civ_core::SimTime, out: &mut dyn Write) -> anyhow::Result<()> {
    use civ_agents::ChronicleKind as K;
    let people = sim.people();
    let year: Vec<_> = people.chronicle.iter().filter(|e| e.at >= since).collect();
    let named = |e: &civ_agents::history::ChronicleEvent| {
        let who = e
            .people
            .first()
            .map_or_else(|| "someone".to_owned(), |&p| people.name_of(p));
        format!("{who}: {}", e.name)
    };
    for (kind, label) in [
        (K::TechniqueIntroduced, "introduced"),
        (K::TechniqueFound, "found"),
        (K::TechniqueLearned, "learnt beside a knower"),
        (K::TechniqueLost, "lost"),
        (K::BuildingFailed, "gave way"),
    ] {
        let lines: Vec<String> = year
            .iter()
            .filter(|e| e.kind == kind)
            .map(|e| named(e))
            .collect();
        if !lines.is_empty() {
            writeln!(out, "  {label}: {}", lines.join("; "))?;
        }
    }
    let catalog = &sim.rules().catalog;
    // Crafts known by a few, and by whom.
    let few: Vec<String> = catalog
        .techniques
        .iter()
        .enumerate()
        .filter_map(|(t, def)| {
            let knowers: Vec<&str> = people
                .people
                .iter()
                .filter(|(_, p)| p.knows(t))
                .map(|(_, p)| p.given.as_str())
                .collect();
            (!knowers.is_empty() && knowers.len() <= 5)
                .then(|| format!("{} ({})", def.name, knowers.join(", ")))
        })
        .collect();
    if !few.is_empty() {
        writeln!(out, "  known by few: {}", few.join("; "))?;
    }
    for t in people.trust.iter().filter(|t| t.failures > 0.0) {
        let technique = catalog
            .techniques
            .get(usize::from(t.technique))
            .map_or("?", |d| d.name.as_str());
        let (_, words) =
            civ_sim::frames::knowledge::trust_words(sim, t.settlement, usize::from(t.technique));
        writeln!(out, "  caution: {technique} {words}")?;
    }
    let buildings = &sim.land().buildings;
    let begun: Vec<_> = buildings.iter().filter(|b| b.started >= since).collect();
    let followed = begun.iter().filter(|b| b.style_from.is_some()).count();
    let pitch = |b: &civ_land::Building| {
        catalog
            .building_index(&b.spec.program)
            .and_then(|i| catalog.buildings.get(i))
            .map(|d| civ_agents::style::traits_of(&b.spec, d).pitch_centideg / 100.0)
    };
    let span = |v: &[f32]| {
        v.iter()
            .fold((f32::MAX, f32::MIN), |(lo, hi), &x| (lo.min(x), hi.max(x)))
    };
    let pitches: Vec<f32> = buildings.iter().filter_map(pitch).collect();
    let tastes: Vec<f32> = people
        .households
        .iter()
        .filter(|(_, h)| !h.members.is_empty())
        .map(|(_, h)| h.taste.pitch_centideg / 100.0)
        .collect();
    if !pitches.is_empty() && !tastes.is_empty() {
        let ((b0, b1), (t0, t1)) = (span(&pitches), span(&tastes));
        writeln!(
            out,
            "  style: {} buildings begun, {followed} after an admired one; roofs pitched {b0:.1}-{b1:.1}°, \
             households' tastes {t0:.1}-{t1:.1}°",
            begun.len()
        )?;
    }
    Ok(())
}

/// The wealth measures recorded since the last report (slice K; ADR-0007 §4): each settlement's
/// at the end of each calendar year.
fn report_wealth(sim: &Sim, shown: &mut usize, out: &mut dyn Write) -> anyhow::Result<()> {
    let years = &sim.people().wealth_years;
    for y in years.iter().skip(*shown) {
        let s = &y.spread;
        let name = sim
            .land()
            .settlements
            .iter()
            .find(|x| x.id == s.settlement)
            .map_or("a settlement", |x| x.name.as_str());
        let count = |share: f64| (share * f64::from(s.households)).round();
        writeln!(
            out,
            "  wealth at the end of year {} in {name}: Gini of goods {:.2}, land worked {:.2}, \
             land held {:.2}, floor area {:.2}; the top tenth have {:.0}% of the goods; {} of {} \
             households hold no land ({:.1} ha held in common) and {} work none; {:.0} h of goods \
             and {:.2} ha worked a head, {:.0} m² a house",
            y.year,
            s.gini_goods,
            s.gini_worked,
            s.gini_held,
            s.gini_floor,
            s.top_tenth_goods * 100.0,
            count(s.holding_none),
            s.households,
            s.common_ha,
            count(s.working_none),
            s.goods_h_per_head,
            s.worked_ha_per_head,
            s.floor_m2_per_house,
        )?;
    }
    *shown = years.len();
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

/// Each settlement's prices as the year ends, hours of the seller's own work a unit: per good,
/// the middle of what its sellers ask (one ask a seller, households and workshops), and what
/// its trades of the last twelve months paid on average.
fn report_prices(sim: &Sim, out: &mut dyn Write) -> anyhow::Result<()> {
    let pop = sim.people();
    let goods = &sim.rules().catalog.goods;
    let month = civ_agents::market::month_of(sim.now());
    for m in &pop.markets {
        let name = sim
            .land()
            .settlements
            .iter()
            .find(|s| s.id == m.settlement)
            .map_or("a settlement", |s| s.name.as_str());
        let mut asks: Vec<Vec<f64>> = vec![Vec::new(); goods.len()];
        let mut seller = |offers: &[civ_agents::market::Offer]| {
            let mut seen: Vec<u16> = Vec::new();
            for o in offers {
                if !seen.contains(&o.good)
                    && let Some(a) = asks.get_mut(usize::from(o.good))
                {
                    seen.push(o.good);
                    a.push(f64::from(o.ask_h));
                }
            }
        };
        for (_, h) in pop.households.iter() {
            if h.settlement == Some(m.settlement) {
                seller(&h.offers);
            }
        }
        for f in &pop.firms {
            if f.is_open() && f.settlement == Some(m.settlement) {
                seller(&f.offers);
            }
        }
        let mut paid: Vec<(f64, f64)> = vec![(0.0, 0.0); goods.len()];
        for h in m.history.iter().filter(|h| h.month + 12 > month) {
            if let Some(p) = paid.get_mut(usize::from(h.good)) {
                p.0 += f64::from(h.units);
                p.1 += f64::from(h.paid_h);
            }
        }
        let mut parts = Vec::new();
        for (g, d) in goods.iter().enumerate() {
            let a = &mut asks[g];
            let (units, hours) = paid[g];
            if a.is_empty() && units <= 0.0 {
                continue;
            }
            let mut words = d.name.to_lowercase();
            if !a.is_empty() {
                a.sort_by(f64::total_cmp);
                let n = a.len();
                let mid = if n % 2 == 1 {
                    a[n / 2]
                } else {
                    (a[n / 2 - 1] + a[n / 2]) / 2.0
                };
                words.push_str(&format!(" asked {mid:.2} ({n})"));
            }
            if units > 0.0 {
                words.push_str(&format!(" paid {:.2} ({units:.0} sold)", hours / units));
            }
            parts.push(words);
        }
        writeln!(
            out,
            "  prices in {name} (hours a unit; sellers, units sold in the last twelve months): {}",
            or_none(parts.join("; "))
        )?;
    }
    Ok(())
}

/// Land under the regime (slice K): the fields and who holds them, how much is let, how many
/// households work no field, and the rent paid over the year.
fn report_land(
    sim: &Sim,
    before: &Transfers,
    after: &Transfers,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    let fields = &sim.land().fields;
    let ha = |f: &civ_land::Field| f.rect.area_ha();
    // (An empty sum of floats is -0.0; adding 0.0 prints it as 0.0.)
    let total: f64 = fields.iter().map(ha).sum::<f64>() + 0.0;
    let by_settlement: f64 = fields
        .iter()
        .filter(|f| matches!(f.holder, civ_land::Party::Settlement(_)))
        .map(ha)
        .sum::<f64>()
        + 0.0;
    let let_out: Vec<&civ_land::Field> = fields.iter().filter(|f| f.lease.is_some()).collect();
    let pop = sim.people();
    let vacant = fields
        .iter()
        .filter(|f| pop.household(f.household).is_none())
        .count();
    let living: Vec<PermanentId> = pop
        .households
        .iter()
        .map(|(_, h)| h)
        .filter(|h| !h.members.is_empty())
        .map(|h| h.id)
        .collect();
    let landless = living
        .iter()
        .filter(|&&h| !fields.iter().any(|f| f.household == h))
        .count();
    let holding = living
        .iter()
        .filter(|&&h| {
            fields
                .iter()
                .any(|f| f.holder == civ_land::Party::Household(h))
        })
        .count();
    let rules = sim.rules();
    let crop = rules.catalog.crops.get(rules.people.farm.crop);
    let rent: f64 = crop.map_or(0.0, |c| {
        [c.good, c.seed_good]
            .into_iter()
            .map(|g| after.get(Channel::Rent, g) - before.get(Channel::Rent, g))
            .sum()
    });
    writeln!(
        out,
        "  land: {} fields ({total:.1} ha), {by_settlement:.1} ha held by settlements; {} of {} \
         households hold land and {landless} work none; {} fields let ({:.1} ha), {vacant} vacant; \
         rent {rent:.0} kg of grain",
        fields.len(),
        holding,
        living.len(),
        let_out.len(),
        let_out.iter().map(|f| ha(f)).sum::<f64>() + 0.0,
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

/// The past twelve whole months' weather on the valley floor, against what those months usually
/// bring, and the water this season's crops had (ADR-0012): what lies behind the harvest.
fn report_weather(sim: &Sim, held: &[&civ_land::Field], out: &mut dyn Write) -> anyhow::Result<()> {
    let land = sim.land();
    let whole: Vec<&civ_land::MonthRecord> = land
        .weather
        .months
        .iter()
        .rev()
        .filter(|m| {
            i64::from(m.days) == civ_core::time::MONTH_LENGTHS[usize::from(m.month.min(11))]
        })
        .take(12)
        .collect();
    if whole.is_empty() {
        return Ok(());
    }
    let rain: f64 = whole.iter().map(|m| f64::from(m.precip_mm)).sum();
    let usual: f64 = whole
        .iter()
        .map(|m| land.climatology.month_precip_mm[usize::from(m.month.min(11))])
        .sum();
    let days: f64 = whole.iter().map(|m| f64::from(m.days)).sum();
    let mean = whole.iter().map(|m| f64::from(m.temp_sum_c)).sum::<f64>() / days.max(1.0);
    let count = |f: &dyn Fn(&civ_land::MonthRecord) -> u8| -> u32 {
        whole.iter().map(|m| u32::from(f(m))).sum()
    };
    // This season's crops, weighted by area: the share of their water need they had, and the
    // share of an average year's harvest that allows.
    let sown: Vec<_> = held.iter().filter(|f| f.need_mm > 0.0).collect();
    let area: f64 = sown.iter().map(|f| f.area_ha()).sum();
    let water = if area > 0.0 {
        let need: f64 = sown
            .iter()
            .map(|f| f64::from(f.need_mm) * f.area_ha())
            .sum();
        let got: f64 = sown.iter().map(|f| f64::from(f.got_mm) * f.area_ha()).sum();
        let factor: f64 = sown
            .iter()
            .map(|f| land.water_factor(f) * f.area_ha())
            .sum::<f64>()
            / area;
        format!(
            "; the crops had {:.0}% of the water they needed (harvest {factor:.2} of an average              year's)",
            100.0 * got / need.max(1e-9)
        )
    } else {
        String::new()
    };
    writeln!(
        out,
        "  weather over {} months: {rain:.0} mm ({usual:.0} usual) in {} wet days, mean {mean:.1} °C,          {} frost days, {} with snow lying{water}",
        whole.len(),
        count(&|m| m.wet_days),
        count(&|m| m.frost_days),
        count(&|m| m.snow_days)
    )?;
    Ok(())
}

/// The soil under the fields held (ADR-0012 §3): the last calendar year's harvests per hectare
/// and how many of them the soil held back, and the nitrogen the soil supplies this year against
/// what native ground of the same richness would.
fn report_soil(sim: &Sim, held: &[&civ_land::Field], out: &mut dyn Write) -> anyhow::Result<()> {
    if held.is_empty() {
        return Ok(());
    }
    let soil = &sim.rules().land.soil;
    let last = i32::try_from(sim.now().date().year - 1).unwrap_or(i32::MIN);
    let (mut reaped, mut kg, mut ha, mut by_soil) = (0usize, 0.0, 0.0, 0usize);
    for f in held {
        if let Some(r) = f.soil.record.iter().rev().find(|r| r.year == last) {
            reaped += 1;
            kg += f64::from(r.kg_per_ha) * f.area_ha();
            ha += f.area_ha();
            by_soil += usize::from(r.limit == civ_land::Limit::Soil);
        }
    }
    let area: f64 = held.iter().map(|f| f.area_ha()).sum();
    let supply: f64 = held
        .iter()
        .map(|f| f64::from(f.soil.supply_n) * f.area_ha())
        .sum();
    let native: f64 = held
        .iter()
        .map(|f| {
            f64::from(civ_land::FieldSoil::native(soil, f64::from(f.ground)).supply_n) * f.area_ha()
        })
        .sum();
    writeln!(
        out,
        "  soil: {reaped} fields reaped in year {last} gave {:.0} kg/ha, {by_soil} held back by \
         the soil; it supplies {:.0} kg N/ha this year, {:.0}% of native ground's",
        kg / ha.max(1e-9),
        supply / area.max(1e-9),
        100.0 * supply / native.max(1e-9)
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
    report_weather(sim, &held, out)?;
    report_soil(sim, &held, out)?;
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
