//! Prints a save's chronicle in the observer's words, from one calendar year to another, with how
//! many lived at each year's end: to read what happened to a world kept by the dashboard
//! (`civ-host dashboard --keep-saves`) or the smoke.
//!
//! ```sh
//! cargo run --release -p civ-sim --example chronicle -- <save> [from-year] [to-year]
//! ```

use std::path::Path;

use civ_agents::history::{plain, render};
use civ_sim::persist;

fn main() {
    let mut args = std::env::args().skip(1);
    let save = args.next().expect("a save to read");
    let from: i64 = args.next().map_or(i64::MIN, |a| a.parse().expect("a year"));
    let to: i64 = args.next().map_or(i64::MAX, |a| a.parse().expect("a year"));
    let root = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
        .expect("content/ is above the crate");
    let content = civ_content::load(&root)
        .registry
        .expect("the content loads");
    let sim = persist::load(Path::new(&save), &content).expect("the save loads");
    let people = sim.people();
    let name_of = |id| {
        people
            .records
            .get(&id)
            .map_or_else(|| "someone".to_owned(), |r| r.given.clone())
    };
    println!(
        "{}: {} living on {:?}",
        sim.meta().name,
        people.living(),
        sim.now().date()
    );
    for e in &people.chronicle {
        let d = e.at.date();
        if d.year < from || d.year > to {
            continue;
        }
        println!(
            "{:>3}-{:02}-{:02}  {}",
            d.year,
            d.month,
            d.day,
            plain(&render(e, &name_of))
        );
    }
    for w in people
        .wealth_years
        .iter()
        .filter(|w| w.year >= from && w.year <= to)
    {
        println!(
            "end of year {}: {} people in {} households, Gini of goods {:.2}",
            w.year, w.spread.people, w.spread.households, w.spread.gini_goods
        );
    }
}
