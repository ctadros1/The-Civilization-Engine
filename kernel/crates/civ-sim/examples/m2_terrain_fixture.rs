//! Renderer diagnostic inputs from the kernel, not a lived world or an observer intervention.
use civ_schema::{flatbuffers, wire};
use civ_sim::{NewWorld, Sim, frames};
use std::path::{Path, PathBuf};

fn main() -> Result<(), String> {
    let folder = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: m2_terrain_fixture <folder>")?;
    let root = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
        .ok_or("content root")?;
    let content = civ_content::load(&root).registry.ok_or("content")?;
    let mut sim = Sim::create_for_tests(
        &NewWorld {
            name: "M2 terrain diagnostic".into(),
            seed: 12,
            preset_id: "core:worldgen/river_valley".into(),
            size_cells: 512,
            band_size: 32,
            neighbours: vec![],
            neighbours_known: true,
            regime_id: String::new(),
        },
        &content,
        [12; 16],
    )
    .map_err(|e| e.to_string())?;
    let query = frames::RasterQuery {
        layer: wire::RasterLayer::Elevation,
        level: 0,
        x0: 0,
        y0: 0,
        width: 512,
        height: 512,
    };
    let read = |sim: &Sim| {
        frames::raster_response(
            sim.map(),
            &sim.land().ground,
            sim.stats(),
            &sim.land().flood_depths,
            &query,
        )
        .map_err(|e| e.to_string())
    };
    let baseline = read(&sim)?;
    let (cx, cy) = (0..8)
        .flat_map(|y| (0..8).map(move |x| (x * 64 + 28, y * 64 + 28)))
        .filter(|&(x, y)| sim.map().water[(y * 512 + x) as usize] == 0)
        .min_by_key(|&(x, y)| (i64::from(x) - 256).pow(2) + (i64::from(y) - 256).pow(2))
        .ok_or("no land for the fixture")?;
    let cell = f64::from(sim.map().cell_size_m);
    for dy in 0..2 {
        for dx in 0..2 {
            let delta = if dx == dy { 0.4 } else { -0.4 };
            sim.land_mut_for_tests().ground.add(
                (
                    (f64::from(cx + dx) + 0.5) * cell,
                    (f64::from(cy + dy) + 0.5) * cell,
                ),
                delta * cell * cell,
            );
        }
    }
    let edited = read(&sim)?;
    let height = |bytes: &[u8]| -> Result<f32, String> {
        let response = flatbuffers::root::<wire::Response>(bytes).map_err(|e| e.to_string())?;
        let tile = response.body_as_raster_tile().ok_or("not a raster")?;
        let data = tile.data().ok_or("no samples")?;
        let index = ((cy * 512 + cx) * 2) as usize;
        Ok(
            f32::from(u16::from_le_bytes([data.get(index), data.get(index + 1)])) * tile.scale()
                + tile.offset(),
        )
    };
    let before = height(&baseline)?;
    let after = height(&edited)?;
    if (after - before - 0.4).abs() > 0.02 {
        return Err("the kernel did not publish the local ground change".into());
    }
    std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    std::fs::write(folder.join("baseline.bin"), baseline).map_err(|e| e.to_string())?;
    std::fs::write(folder.join("edited.bin"), edited).map_err(|e| e.to_string())?;
    std::fs::write(
        folder.join("metadata.txt"),
        format!("{cx} {cy} {before} {after}\n"),
    )
    .map_err(|e| e.to_string())?;
    println!(
        "M2_TERRAIN_FIXTURE_OK cells=4 cut_m3=51.2 fill_m3=51.2 before={before} after={after}"
    );
    Ok(())
}
