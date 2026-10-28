use std::collections::HashMap;
use std::path::Path;
use civ_sim::{NewWorld, Sim};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let people: u32 = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(500);
    let days: i64 = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(40);
    let root = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    let mut content = civ_content::load(&root).registry.unwrap();
    let band = &mut content.people.params.band;
    band.max_size = band.max_size.max(people);
    band.min_size = band.min_size.min(people);
    let mut sim = Sim::create_for_tests(&NewWorld { name: "p".into(), seed: 2, preset_id: content.default_preset().id.clone(), size_cells: 768, band_size: people, neighbours: vec![], neighbours_known: false, regime_id: String::new() }, &content, [7; 16]).unwrap();
    sim.advance_minutes(10 * 1440).unwrap();
    let guard = pprof::ProfilerGuardBuilder::default().frequency(997).build().unwrap();
    sim.advance_minutes(days * 1440).unwrap();
    let report = guard.report().build().unwrap();
    let mut by: std::collections::BTreeMap<String, (usize, usize, usize)> = Default::default();
    for b in &sim.land().buildings { let e = by.entry(b.spec.program.clone()).or_default(); e.0 += 1; if b.finished() { e.1 += 1; } if b.standing() { e.2 += 1; } }
    println!("buildings {} (program: all, finished, standing): {:?}", sim.land().buildings.len(), by);
    println!("households {}", sim.people().households.iter().count());
    let (mut selfc, mut incl, mut total): (HashMap<String, isize>, HashMap<String, isize>, isize) = (HashMap::new(), HashMap::new(), 0);
    let mut bcall: HashMap<String, isize> = HashMap::new();
    for (frames, count) in report.data.iter() {
        total += count;
        let names: Vec<String> = frames.frames.iter().flat_map(|f| f.iter().map(|s| format!("{} @{}:{}", s.name().rsplitn(2, "::h").last().unwrap_or(""), s.filename.as_ref().map(|p| p.display().to_string()).unwrap_or_default().rsplit("/src/").next().unwrap_or("").to_string(), s.lineno.unwrap_or(0)))).collect();
        let full: Vec<String> = frames.frames.iter().flat_map(|f| f.iter().map(|s| format!("{}:{}", s.filename.as_ref().map(|p| p.display().to_string()).unwrap_or_default(), s.lineno.unwrap_or(0)))).collect();
        if true {
            let k: Vec<String> = full.iter().filter(|f| f.contains("/kernel/crates/")).filter(|f| !f.contains("person.rs")).take(2).map(|f| f.rsplit("/crates/").next().unwrap_or("").to_string()).collect();
            *bcall.entry(k.join(" < ")).or_default() += count;
        }
        if let Some(n) = names.first() { *selfc.entry(n.clone()).or_default() += count; }
        let mut seen = std::collections::HashSet::new();
        for n in names { if seen.insert(n.clone()) { *incl.entry(n).or_default() += count; } }
    }
    let short = |n: &str| { let n = n.rsplitn(2, "::h").last().unwrap_or(n).to_string(); n.chars().take(150).collect::<String>() };
    for (title, m) in [("BUILDING CALLERS", &bcall), ("SELF", &selfc)] {
        let mut v: Vec<_> = m.iter().collect();
        v.sort_by(|a, b| b.1.cmp(a.1));
        println!("== {title} (of {total} samples)");
        for (n, c) in v.iter().take(if title == "SELF" { 30 } else { 70 }) { println!("{:5.1}% {}", **c as f64 * 100.0 / total as f64, short(n)); }
    }
}
