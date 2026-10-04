//! A world with a workshop in it, for the observer's end-to-end tests (M3a slice J). Workshops
//! arise only where a tool is wanted that nobody offers, and runs differ, so a short natural run
//! may have none. Here a band lives a day; then its largest household is made master knappers
//! with flint and wood to spare, its neighbours lose their sickles and get grain to pay with, and
//! their want of sickles is kept on record, as if they went on asking, until the workshop the
//! household sets up has sold one and posted a wage. Then the world is saved. Test scaffolding,
//! as in `tests/firms.rs`: the observer never edits people.
//!
//! ```sh
//! cargo run --release -p civ-sim --example workshop_world -- <saves folder>
//! ```

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::AtomicBool;

use civ_agents::firm::BookKind;
use civ_agents::market::Market;
use civ_core::PermanentId;
use civ_sim::{NewWorld, Sim, persist};
use commons_persist::{SaveDir, SaveKind};

/// Days the want of sickles is kept on record at most.
const MAX_DAYS: u32 = 120;

fn main() -> ExitCode {
    let Some(saves) = std::env::args().nth(1).map(PathBuf::from) else {
        eprintln!("usage: workshop_world <saves folder>");
        return ExitCode::FAILURE;
    };
    match make(&saves) {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("workshop_world: {e}");
            ExitCode::FAILURE
        }
    }
}

fn make(saves: &Path) -> Result<String, String> {
    let root = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
        .ok_or("content/ is not above the crate")?;
    let content = civ_content::load(&root)
        .registry
        .ok_or("the content does not load: run `civ-host content validate`")?;
    let mut sim = Sim::create(
        &NewWorld {
            name: "Workshop Valley".to_owned(),
            seed: 3,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 512,
            band_size: 0,
        },
        &content,
        &mut |_| {},
        &AtomicBool::new(false),
    )
    .map_err(|e| e.to_string())?;
    sim.advance_minutes(24 * 60).map_err(|e| e.to_string())?;
    let rules = sim.rules().clone();
    let goods = &rules.catalog.goods;
    let good = |id: &str| goods.iter().position(|d| d.id == id).ok_or(id.to_owned());
    let sickle = good("core:good/sickle")?;
    let knapping = rules
        .catalog
        .skills
        .iter()
        .position(|k| k.id == "core:skill/knapping")
        .ok_or("no knapping skill")?;
    let mut families: Vec<(usize, PermanentId)> = sim
        .people()
        .households
        .iter()
        .map(|(_, h)| (h.members.len(), h.id))
        .collect();
    families.sort_unstable_by(|a, b| b.cmp(a));
    let maker = families.first().ok_or("nobody settled")?.1;
    let now = sim.now();
    let ages: Vec<f64> = sim
        .people()
        .household(maker)
        .ok_or("no maker")?
        .members
        .iter()
        .filter_map(|m| sim.people().person(*m))
        .map(|p| p.age_years(now))
        .collect();
    let wants = civ_agents::make::tool_wants_for(
        &rules.catalog,
        &ages,
        rules.people.family.independent_age,
    );
    let set = |sim: &mut Sim, household: PermanentId, kg: &[(usize, f64)], level: f64| {
        let pop = sim.people_mut_for_tests();
        if let Some(h) = pop
            .households
            .iter_mut()
            .map(|(_, h)| h)
            .find(|h| h.id == household)
        {
            for &(g, v) in kg {
                h.stores[g] = v;
            }
        }
        for (_, p) in pop.people.iter_mut() {
            if p.household == household {
                p.set_skill(knapping, level);
            }
        }
    };
    let (toolstone, timber, grain) = (
        good("core:good/toolstone")?,
        good("core:good/timber")?,
        good("core:good/grain")?,
    );
    set(
        &mut sim,
        maker,
        &[
            (sickle, wants[sickle] + civ_agents::make::SPARE_TOOL),
            (toolstone, 30.0),
            (timber, 50.0),
            (grain, 4000.0),
        ],
        1.0,
    );
    for &(_, other) in &families[1..] {
        set(
            &mut sim,
            other,
            &[
                (sickle, 0.0),
                (toolstone, 0.0),
                (timber, 0.0),
                (grain, 3000.0),
            ],
            0.0,
        );
    }
    let settlement = sim
        .people()
        .household(maker)
        .and_then(|h| h.settlement)
        .ok_or("the maker has no settlement")?;
    let ready = |sim: &Sim| {
        sim.people().firms.iter().any(|f| {
            f.is_open()
                && f.wage.is_some()
                && f.books
                    .months
                    .iter()
                    .any(|m| m.amount(BookKind::Sold, sickle as u16) > 0.0)
        })
    };
    let mut days = 0;
    while !ready(&sim) {
        if days == MAX_DAYS {
            return Err(format!(
                "no workshop sold a sickle and posted a wage in {MAX_DAYS} days: {:?}",
                sim.people().firms
            ));
        }
        let day = sim.now().day_index();
        let pop = sim.people_mut_for_tests();
        if !pop.markets.iter().any(|m| m.settlement == settlement) {
            pop.markets.push(Market::new(settlement, goods.len(), day));
        }
        if let Some(m) = pop.markets.iter_mut().find(|m| m.settlement == settlement)
            && m.unmet[sickle] < 4.0
        {
            let more = 4.0 - m.unmet[sickle];
            m.record_unmet(sickle, more, 40.0 * more);
        }
        sim.advance_minutes(24 * 60).map_err(|e| e.to_string())?;
        days += 1;
    }
    let dir = SaveDir::create(
        saves.join(persist::world_dir_name(sim.meta())),
        civ_schema::SAVE_EXTENSION,
    )
    .map_err(|e| e.to_string())?;
    let saved = persist::save(&mut sim, &dir, SaveKind::Manual, "A workshop at work")
        .map_err(|e| e.to_string())?;
    Ok(format!(
        "saved {} after {days} days with {} workshop(s)",
        saved.path.display(),
        sim.people().firms.len()
    ))
}
