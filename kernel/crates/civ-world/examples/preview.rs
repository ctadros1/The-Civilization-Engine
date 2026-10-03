//! Generates a world and writes a shaded-relief preview PNG, with timings and summary figures.
//!
//! ```sh
//! cargo run --release -p civ-world --example preview -- --seed 7 --cells 1024 --out preview.png
//! ```
//!
//! Optional: `--coast` lowers the base level below the sea; `--basins` adds subsiding basins.

use std::io::BufWriter;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

use civ_world::{GenerateRequest, TerrainParams, WATER_LAKE, WATER_OCEAN, WATER_RIVER, generate};

fn arg<T: std::str::FromStr>(name: &str, default: T) -> T {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn flag(name: &str) -> bool {
    std::env::args().any(|a| a == name)
}

fn lerp(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn ramp(t: f32) -> [f32; 3] {
    const STOPS: [(f32, [f32; 3]); 5] = [
        (0.0, [92.0, 138.0, 74.0]),
        (0.25, [150.0, 160.0, 96.0]),
        (0.5, [168.0, 146.0, 104.0]),
        (0.75, [140.0, 120.0, 102.0]),
        (1.0, [235.0, 235.0, 235.0]),
    ];
    let t = t.clamp(0.0, 1.0);
    for pair in STOPS.windows(2) {
        let (t0, c0) = pair[0];
        let (t1, c1) = pair[1];
        if t <= t1 {
            return lerp(c0, c1, (t - t0) / (t1 - t0));
        }
    }
    STOPS[4].1
}

fn main() {
    let seed: u64 = arg("--seed", 7);
    let cells: u32 = arg("--cells", 512);
    let out: String = arg("--out", "preview.png".to_owned());
    let mut params = TerrainParams::default();
    if flag("--coast") {
        params.base_level_m = -40.0;
        params.regional_slope = 0.003;
        params.window_offset = 0.9;
        params.trunk_inflow_km2 = 800.0;
    }
    if flag("--basins") {
        params.base_level_m = 600.0;
        params.basin_strength = 0.7;
        params.trunk_inflow_km2 = 0.0;
        params.outlet_edges = 2;
        params.precipitation_mm_per_yr = 350.0;
        params.evapotranspiration_mm_per_yr = 320.0;
        params.lake_evaporation_mm_per_yr = 1300.0;
        params.endorheic_depth_m = 8.0;
        params.context_factor = arg("--context", 1.25);
    }
    let request = GenerateRequest {
        seed,
        width_cells: cells,
        height_cells: cells,
        cell_size_m: 8.0,
        params,
    };
    let started = Instant::now();
    let mut last_stage = "";
    let map = generate(
        &request,
        &mut |p| {
            if p.stage != last_stage {
                eprintln!(
                    "{:6.2}s  {:5.1}%  {}",
                    started.elapsed().as_secs_f32(),
                    p.fraction * 100.0,
                    p.stage
                );
                last_stage = p.stage;
            }
        },
        &AtomicBool::new(false),
    )
    .unwrap_or_else(|e| panic!("generation failed: {e}"));
    eprintln!("generated in {:.2}s", started.elapsed().as_secs_f32());
    eprintln!("{:#?}", map.stats());

    let (w, h) = (map.width as usize, map.height as usize);
    let c = map.cell_size_m;
    let stats = map.stats();
    let land_lo = map.sea_level_m.max(stats.min_elevation_m);
    let span = (stats.max_elevation_m - land_lo).max(1.0);
    let mut rgb = vec![0u8; w * h * 3];
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let z = |xx: usize, yy: usize| map.elevation[yy * w + xx];
            let dzdx = (z((x + 1).min(w - 1), y) - z(x.saturating_sub(1), y)) / (2.0 * c);
            let dzdy = (z(x, (y + 1).min(h - 1)) - z(x, y.saturating_sub(1))) / (2.0 * c);
            let (nx, ny, nz) = (-dzdx * 2.0, -dzdy * 2.0, 1.0f32);
            let len = (nx * nx + ny * ny + nz * nz).sqrt();
            let (lx, ly, lz) = (-0.5f32, -0.5f32, 0.707f32);
            let shade = ((nx * lx + ny * ly + nz * lz) / len).clamp(0.0, 1.0);
            let base = match map.water[i] {
                WATER_OCEAN => [40.0, 84.0, 128.0],
                WATER_LAKE => [58.0, 112.0, 164.0],
                WATER_RIVER => [64.0, 124.0, 186.0],
                _ => ramp((map.elevation[i] - land_lo) / span),
            };
            let k = if map.water[i] == 0 {
                0.45 + 0.75 * shade
            } else {
                1.0
            };
            for ch in 0..3 {
                rgb[i * 3 + ch] = (base[ch] * k).clamp(0.0, 255.0) as u8;
            }
        }
    }
    let file = std::fs::File::create(&out).expect("create output");
    let mut encoder = png::Encoder::new(BufWriter::new(file), w as u32, h as u32);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().expect("png header");
    writer.write_image_data(&rgb).expect("png data");
    eprintln!("wrote {out}");
}
