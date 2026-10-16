//! A probe of trade laws for M7c slice BV (ADR-0026 §1; the capture brief §4: "measure the
//! volume and the forecast distribution before building organizations"): three founding bands of
//! `BAND` that know where each other camped live `YEARS` years as the content has them. Nothing is
//! set and nothing nudged. It prints, a year at a time, the purchases between settlements, every
//! duty, embargo or recognition proposed, with its stances, and the organizations by stake (M7c
//! slice BW) with their members and applications; and at the end every law of the polities.
//! Whether anyone proposes a trade law, or organizes, at these volumes is the question.
//!
//! ```sh
//! cargo run --release -p civ-sim --example trade_laws -- [seed] [years]
//! ```

use std::path::Path;
use std::process::ExitCode;
use std::sync::atomic::AtomicBool;

use civ_agents::faction::FactionEventKind;
use civ_agents::polity::PolicyKind;
use civ_core::PermanentId;
use civ_sim::{NewWorld, SPEED_MAX, Sim};

/// People in each founding band.
const BAND: u32 = 40;
const DAY: i64 = 24 * 60;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let seed = args.next().and_then(|s| s.parse().ok()).unwrap_or(1);
    let years: i64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(3);
    match run(seed, years) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("trade_laws: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(seed: u64, years: i64) -> Result<(), String> {
    let root = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
        .ok_or("content/ is not above the crate")?;
    let content = civ_content::load(&root)
        .registry
        .ok_or("the content does not load: run `civ-host content validate`")?;
    let mut sim = Sim::create(
        &NewWorld {
            name: "Three Fords".to_owned(),
            seed,
            preset_id: "core:worldgen/river_valley".to_owned(),
            size_cells: 1024,
            band_size: BAND,
            neighbours: vec![BAND, BAND],
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
    let names: Vec<(PermanentId, String)> = sim
        .land()
        .settlements
        .iter()
        .map(|s| (s.id, s.name.clone()))
        .collect();
    let name = |id: PermanentId| {
        names
            .iter()
            .find(|n| n.0 == id)
            .map_or_else(|| "?".to_owned(), |n| n.1.clone())
    };
    let mut seen_laws = 0usize;
    for year in 1..=years {
        sim.advance_minutes(365 * DAY).map_err(|e| e.to_string())?;
        let pop = sim.people();
        let bought: u32 = pop.contacts.years.values().map(|c| c.bought).sum();
        println!(
            "year {year} (day {}): {} people; {bought} purchases between settlements so far",
            sim.now().day_index(),
            pop.people.len()
        );
        for ((y, from, to), c) in &pop.contacts.years {
            if c.bought > 0 {
                println!(
                    "  in year {y}: {} bought {} times at {}'s doors",
                    name(*from),
                    c.bought,
                    name(*to)
                );
            }
        }
        let laws: Vec<_> = pop.polities.iter().flat_map(|p| &p.laws).collect();
        for l in laws.iter().skip(seen_laws) {
            if matches!(
                l.kind,
                PolicyKind::Duty | PolicyKind::Embargo | PolicyKind::Recognize
            ) {
                let (present, support, oppose) = l.counts();
                println!(
                    "  {:?} proposed on day {}: {:?} ({support} for, {oppose} against, {present} \
                     came)",
                    l.kind,
                    l.proposed.day_index(),
                    l.status
                );
            }
        }
        seen_laws = laws.len();
        for f in pop.factions.list.iter().filter(|f| f.stake.is_some()) {
            let count = |k: FactionEventKind| f.history.iter().filter(|e| e.kind == k).count();
            println!(
                "  organization of {} founded on day {} at {}{}: {} members; {} admitted, {} \
                 turned away",
                f.stake.map_or("?", |s| s.purpose.plural_words()),
                f.founded.day_index(),
                name(f.settlement),
                if f.ended.is_some() { ", ended" } else { "" },
                pop.factions.members_of(f.id).count(),
                count(FactionEventKind::Admitted),
                count(FactionEventKind::TurnedAway),
            );
        }
    }
    let pop = sim.people();
    for p in &pop.polities {
        println!("{}:", name(p.settlement));
        for l in &p.laws {
            let (present, support, oppose) = l.counts();
            println!(
                "  {:?} proposed on day {}: {:?} ({support} for, {oppose} against, {present} came)",
                l.kind,
                l.proposed.day_index(),
                l.status
            );
        }
    }
    Ok(())
}
