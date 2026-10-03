//! The command line's one-shot commands: create a world, inspect and verify saves, validate
//! content.

use std::path::Path;
use std::process::ExitCode;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

use anyhow::{Context, anyhow};
use civ_content::ContentRegistry;
use civ_core::SimTime;
use civ_schema::SAVE_EXTENSION;
use civ_sim::{NewWorld, Sim, persist};
use commons_persist::{Limits, SaveDir, SaveKind, SnapshotReader, hex};

/// Loads content, printing every diagnostic. Fails when there are errors.
pub fn load_content(root: &Path) -> anyhow::Result<ContentRegistry> {
    let report = civ_content::load(root);
    for diagnostic in &report.diagnostics {
        eprintln!("{diagnostic}");
    }
    let errors = report.error_count();
    report.registry.ok_or_else(|| {
        anyhow!(
            "the content in {} has {errors} error(s); run `civ-host content validate`",
            root.display()
        )
    })
}

/// `civ-host content validate`: prints diagnostics (or the JSON report) and fails on errors.
pub fn content_validate(root: &Path, json: bool) -> ExitCode {
    let report = civ_content::load(root);
    if json {
        println!("{}", report.to_json());
    } else {
        for diagnostic in &report.diagnostics {
            println!("{diagnostic}");
        }
        match &report.registry {
            Some(registry) => println!(
                "content OK: {} pack(s), {} world preset(s), people `{}`, land `{}`, {} \
                 activities, fingerprint {}",
                registry.packs.len(),
                registry.presets.len(),
                registry.people.id,
                registry.land.id,
                registry.catalog.activities.len(),
                registry.fingerprint_hex()
            ),
            None => println!("content has {} error(s)", report.error_count()),
        }
    }
    if report.registry.is_some() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// What `civ-host new` makes.
#[derive(Clone, Debug)]
pub struct NewOptions {
    /// World-generation seed.
    pub seed: u64,
    /// Preset id; the content's default when absent.
    pub preset: Option<String>,
    /// Cells per side.
    pub size: u32,
    /// World name.
    pub name: String,
}

/// `civ-host new`: generates a world, saves it and prints a summary.
pub fn new_world(
    content: &ContentRegistry,
    saves: &Path,
    options: &NewOptions,
) -> anyhow::Result<()> {
    let preset_id = options
        .preset
        .clone()
        .unwrap_or_else(|| content.default_preset().id.clone());
    let started = Instant::now();
    let mut last_stage = "";
    let mut sim = Sim::create(
        &NewWorld {
            name: options.name.clone(),
            seed: options.seed,
            preset_id,
            size_cells: options.size,
            band_size: 0,
        },
        content,
        &mut |p| {
            if p.stage != last_stage {
                eprintln!("{:>4.0}%  {}", p.fraction * 100.0, p.stage);
                last_stage = p.stage;
            }
        },
        &AtomicBool::new(false),
    )?;
    let generated = started.elapsed();
    let dir = SaveDir::create(
        saves.join(persist::world_dir_name(sim.meta())),
        SAVE_EXTENSION,
    )
    .with_context(|| format!("could not create a save folder in {}", saves.display()))?;
    let published = persist::save(
        &mut sim,
        &dir,
        SaveKind::Manual,
        "Created from the command line",
    )?;
    let (meta, stats, map) = (sim.meta(), sim.stats(), sim.map());
    let (x, y) = map.extent_m();
    println!(
        "“{}”: seed {}, {}, {}×{} cells ({:.1} × {:.1} km), generated in {:.1} s",
        meta.name,
        meta.seed,
        meta.preset_id,
        map.width,
        map.height,
        x / 1000.0,
        y / 1000.0,
        generated.as_secs_f64()
    );
    println!(
        "  elevation {:.0} to {:.0} m; land {:.0}%, ocean {:.0}%, gentle land {:.0}%",
        stats.min_elevation_m,
        stats.max_elevation_m,
        stats.land_fraction * 100.0,
        stats.ocean_fraction * 100.0,
        stats.gentle_land_fraction * 100.0
    );
    println!(
        "  {} river reaches ({:.1} km, largest {:.1} m³/s, order {}), {} lakes ({} closed)",
        stats.reaches,
        stats.river_length_km,
        stats.max_discharge_m3s,
        stats.max_order,
        stats.lakes,
        stats.closed_lakes
    );
    println!(
        "saved {} ({} bytes)",
        published.path.display(),
        published.file_len
    );
    Ok(())
}

/// Civil date and time of Unix milliseconds, UTC (proleptic Gregorian).
pub fn format_unix_ms(ms: i64) -> String {
    let secs = ms.div_euclid(1000);
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    // Howard Hinnant's days-to-civil algorithm.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02} UTC",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

/// `civ-host save info`: prints a save's header, manifest and compatibility.
pub fn save_info(path: &Path, content: &ContentRegistry) -> anyhow::Result<()> {
    let reader = SnapshotReader::open_file(path, Limits::default())
        .with_context(|| format!("{} is not a readable save", path.display()))?;
    let info = reader.info();
    let engine = String::from_utf8_lossy(&info.engine_tag)
        .trim_end_matches('\0')
        .to_owned();
    let summary = persist::describe(path, content);
    println!("file        {}", path.display());
    println!("size        {} bytes", reader.file_len());
    println!("engine      {engine}, world schema {}", info.schema_version);
    println!(
        "world       “{}” ({})",
        summary.world_name,
        hex(&info.world_id)
    );
    println!("snapshot    {}", hex(&info.snapshot_id));
    println!(
        "parent      {}",
        info.parent_id
            .map_or_else(|| "none".to_owned(), |p| hex(&p))
    );
    println!("generation  {}", info.generation);
    println!("written     {}", format_unix_ms(info.created_unix_ms));
    println!(
        "sim time    {} (minute {})",
        SimTime::from_minutes(info.sim_time),
        info.sim_time
    );
    println!("label       {}", info.label);
    println!(
        "content     {}{}",
        hex(&info.content_fingerprint),
        if summary.content_changed {
            " (differs from the loaded content)"
        } else {
            ""
        }
    );
    println!(
        "loadable    {}{}",
        if summary.compatible { "yes" } else { "no" },
        if summary.note.is_empty() {
            String::new()
        } else {
            format!(": {}", summary.note)
        }
    );
    println!();
    println!(
        "{:<8} {:>5} {:>3} {:>5} {:>12} {:>12}",
        "section", "chunk", "ver", "codec", "stored", "raw"
    );
    for chunk in reader.chunks() {
        println!(
            "{:<8} {:>5} {:>3} {:>5} {:>12} {:>12}",
            chunk.tag.as_str(),
            chunk.index,
            chunk.version,
            format!("{:?}", chunk.codec).to_lowercase(),
            chunk.stored_len,
            chunk.raw_len
        );
    }
    Ok(())
}

/// `civ-host save verify`: checks every byte and loads the world. Fails if anything is wrong.
pub fn save_verify(path: &Path, content: &ContentRegistry) -> ExitCode {
    let checked = SnapshotReader::open_file(path, Limits::default()).and_then(|mut reader| {
        reader.verify_all()?;
        Ok(reader.chunks().len())
    });
    let chunks = match checked {
        Ok(chunks) => chunks,
        Err(e) => {
            println!("FAILED: {e}");
            return ExitCode::FAILURE;
        }
    };
    match persist::load(path, content) {
        Ok(sim) => {
            println!(
                "OK: {chunks} chunks verified; “{}” ({}×{} cells, {}) loads",
                sim.meta().name,
                sim.map().width,
                sim.map().height,
                sim.date()
            );
            if sim.content_changed() {
                println!("note: it was saved with different content than is loaded now");
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            println!("FAILED: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unix_times_format_as_utc() {
        assert_eq!(format_unix_ms(0), "1970-01-01 00:00:00 UTC");
        assert_eq!(format_unix_ms(951_782_400_000), "2000-02-29 00:00:00 UTC");
        assert_eq!(format_unix_ms(1_790_000_000_123), "2026-09-21 14:13:20 UTC");
        assert_eq!(format_unix_ms(-1000), "1969-12-31 23:59:59 UTC");
    }
}
