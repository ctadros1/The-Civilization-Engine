//! A short real world for verifying the Actions-to-dashboard observation path.
use civ_sim::{NewWorld, Sim, observation::Observation};
use std::{
    path::Path,
    time::{Duration, Instant},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let seconds = std::env::args()
        .nth(1)
        .map_or(Ok(180), |s| s.parse::<u64>())?
        .clamp(15, 600);
    let root = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
        .ok_or("content missing")?;
    let content = civ_content::load(&root).registry.ok_or("content invalid")?;
    let mut sim = Sim::create_for_tests(
        &NewWorld {
            name: "Live homelab world".into(),
            seed: 2,
            preset_id: "core:worldgen/river_valley".into(),
            size_cells: 256,
            band_size: 80,
            neighbours: vec![],
            neighbours_known: false,
            regime_id: String::new(),
        },
        &content,
        [43; 16],
    )?;
    let mut observation = Observation::from_env(&sim);
    if !observation.enabled() {
        return Err("set TCE_OBSERVATION_DIR for this observation check".into());
    }
    let started = Instant::now();
    while started.elapsed() < Duration::from_secs(seconds) {
        observation.poll(&sim);
        sim.advance_minutes(60)?;
    }
    let (frames, cost) = observation.cost();
    println!(
        "Observed {} at day {}: {} people; {frames} frames; {:.3}s exporting / {:.3}s total",
        sim.meta().world_id_hex(),
        sim.now().day_index(),
        sim.people().living(),
        cost.as_secs_f64(),
        started.elapsed().as_secs_f64()
    );
    if frames < 2 {
        return Err("fewer than two live observations".into());
    }
    Ok(())
}
