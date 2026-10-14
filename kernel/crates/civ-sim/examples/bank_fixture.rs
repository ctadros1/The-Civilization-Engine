//! Controlled bank geometry fixture for the observer and snapshot checks, not a lived demo.
//! No claim of chosen labour or an adopted policy: those BG steps are still being developed.
use civ_land::{
    RectCm,
    banks::{self, BankSegment},
    earth::{self, EarthKind, Earthwork},
};
use civ_sim::{NewWorld, Sim, persist};
use std::path::{Path, PathBuf};
fn main() -> Result<(), String> {
    let folder = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: bank_fixture <save-folder>")?;
    let root = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
        .ok_or("content root")?;
    let content = civ_content::load(&root).registry.ok_or("content")?;
    let mut sim = Sim::create_for_tests(
        &NewWorld {
            name: "Bank geometry fixture".into(),
            seed: 2,
            preset_id: "core:worldgen/river_valley".into(),
            size_cells: 512,
            band_size: 30,
            neighbours: vec![],
            neighbours_known: true,
            regime_id: String::new(),
        },
        &content,
        [7; 16],
    )
    .map_err(|e| e.to_string())?;
    let household = sim
        .people()
        .households
        .iter()
        .next()
        .ok_or("household")?
        .1
        .id;
    let work = sim.allocate_id_for_tests();
    let borrow = sim.allocate_id_for_tests();
    let (hx, hy) = sim.land().settlements[0].hearth_m;
    let x = f64::from(hx) + 40.0;
    let y = f64::from(hy);
    let bank = BankSegment {
        work,
        borrow,
        a: [x, y - 12.0],
        b: [x, y + 12.0],
        height_m: 1.5,
        crest_width_m: 1.0,
        side_run: 2.0,
        compacted_per_bank: 0.9,
        settlement_m: 0.0,
        erosion_m: 0.0,
        breached: false,
        sediment_m3: 0.0,
    };
    let make = |id, kind, rect| Earthwork {
        id,
        kind,
        rect,
        level_cm: 0,
        side_run_cm: 200,
        plot: None,
        deposit: None,
        heap: None,
        household,
        cut_m3: 0.0,
        done: 0.0,
        version: earth::EARTH_VERSION,
        begun: sim.now(),
    };
    let records = [
        make(
            work,
            EarthKind::Bank,
            RectCm {
                x: ((x - 7.0) * 100.0) as i32,
                y: ((y - 12.0) * 100.0) as i32,
                w: 1400,
                h: 2400,
            },
        ),
        make(
            borrow,
            EarthKind::Pit,
            RectCm {
                x: ((x - 20.0) * 100.0) as i32,
                y: ((y - 12.0) * 100.0) as i32,
                w: 1000,
                h: 2400,
            },
        ),
    ];
    let map = sim.map().clone();
    let land = sim.land_mut_for_tests();
    land.earthworks.extend(records);
    banks::place_from_borrow(&bank, &mut land.earthworks, &map, &mut land.ground, 80.0)?;
    land.banks.push(bank);
    let saves = commons_persist::SaveDir::create(
        folder.join(persist::world_dir_name(sim.meta())),
        civ_schema::SAVE_EXTENSION,
    )
    .map_err(|e| e.to_string())?;
    let saved = persist::save(
        &mut sim,
        &saves,
        commons_persist::SaveKind::Manual,
        "Bank geometry fixture",
    )
    .map_err(|e| e.to_string())?;
    persist::load(&saved.path, &content).map_err(|e| e.to_string())?;
    println!("{}", saved.path.display());
    Ok(())
}
