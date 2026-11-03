//! Optional, read-only observations for the private Actions dashboard. Not a save or command
//! protocol. Hosts explicitly opt in; the scheduler and benchmark paths never call this module.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

use crate::Sim;

const INTERVAL: Duration = Duration::from_secs(5);
const MAX_BYTES: usize = 2_000_000;
static NEXT_STREAM: AtomicU64 = AtomicU64::new(1);

/// One world's publisher. Dropping it removes its frame; parallel worlds have separate files.
#[derive(Debug, Default)]
pub struct Observation {
    path: Option<PathBuf>,
    last: Option<Instant>,
    elapsed: Duration,
    frames: u64,
}

impl Observation {
    /// Opts in only when the invoking step supplies a directory. Timing gates override it.
    pub fn from_env(sim: &Sim) -> Self {
        let dir = std::env::var_os("TCE_OBSERVATION_DIR")
            .filter(|s| !s.is_empty())
            .filter(|_| std::env::var_os("BENCH_GATE").is_none());
        dir.map_or_else(Self::default, |d| Self::in_directory(Path::new(&d), sim))
    }

    /// An explicit destination for an operator or a test; never stored in the world's state.
    pub fn in_directory(dir: &Path, sim: &Sim) -> Self {
        let serial = NEXT_STREAM.fetch_add(1, Ordering::Relaxed);
        Self {
            path: Some(dir.join(format!(
                "{}-{}-{serial}.json",
                sim.meta().world_id_hex(),
                std::process::id()
            ))),
            ..Self::default()
        }
    }

    /// Whether the caller should expose day boundaries for observations.
    pub fn enabled(&self) -> bool {
        self.path.is_some()
    }

    /// Publishes at most once per five real seconds. A failed observation never stops the world.
    pub fn poll(&mut self, sim: &Sim) {
        let Some(path) = self.path.as_ref() else {
            return;
        };
        if self.last.is_some_and(|t| t.elapsed() < INTERVAL) {
            return;
        }
        self.last = Some(Instant::now());
        let started = Instant::now();
        let result = (|| -> Result<(), Box<dyn std::error::Error>> {
            let mut frame = snapshot(sim);
            frame["stream_id"] = json!(
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
            );
            let data = serde_json::to_vec(&frame)?;
            if data.len() > MAX_BYTES {
                return Err("observation exceeds two megabytes".into());
            }
            let parent = path.parent().ok_or("observation directory missing")?;
            std::fs::create_dir_all(parent)?;
            let temp = path.with_extension("tmp");
            std::fs::write(&temp, data)?;
            std::fs::rename(temp, path)?;
            Ok(())
        })();
        self.elapsed += started.elapsed();
        if let Err(e) = result {
            eprintln!("world observation disabled: {e}");
            let _ = std::fs::remove_file(path);
            self.path = None;
        } else {
            self.frames += 1;
        }
    }

    /// Number of successful frames and wall time spent building and writing them.
    pub fn cost(&self) -> (u64, Duration) {
        (self.frames, self.elapsed)
    }
}

impl Drop for Observation {
    fn drop(&mut self) {
        if let Some(path) = &self.path {
            let _ = std::fs::remove_file(path);
        }
    }
}

/// Copies current kernel facts without advancing time, allocating ids or consuming RNG.
/// Terrain is sampled from actual cells, not regenerated from the seed. Entity caps keep the
/// frame bounded; totals disclose anything omitted. Metres from the north-west corner.
pub fn snapshot(sim: &Sim) -> Value {
    let map = sim.map();
    let side = map.width.min(128);
    let mut heights = Vec::with_capacity((side * side) as usize);
    let mut water = Vec::with_capacity(heights.capacity());
    let low = sim.stats().min_elevation_m;
    let relief = (sim.stats().max_elevation_m - low).max(1.0);
    for y in 0..side {
        for x in 0..side {
            let col = (x * map.width / side + map.width / side / 2).min(map.width - 1);
            let row = (y * map.height / side + map.height / side / 2).min(map.height - 1);
            let i = (row * map.width + col) as usize;
            heights.push(((map.elevation[i] - low) / relief * 255.0).clamp(0.0, 255.0) as u8);
            water.push(map.water[i]);
        }
    }
    let settlements: Vec<_> = sim.land().settlements.iter().take(100)
        .map(|s| json!({"id":s.id.get().to_string(),"x":s.hearth_m.0,"y":s.hearth_m.1,"name":s.name.chars().take(80).collect::<String>()})).collect();
    let buildings: Vec<_> = sim
        .land()
        .buildings
        .iter()
        .take(4000)
        .map(|b| {
            let (x, y) = b.spec.footprint.centre();
            json!({"id":b.id.get().to_string(),"x":f64::from(x)/100.0,"y":f64::from(y)/100.0,
               "name":b.spec.program.chars().take(80).collect::<String>()})
        })
        .collect();
    let now = sim.now();
    let people: Vec<_> = sim.people().people.iter().take(8000).map(|(_,p)| {
        // Exactly the same household-resolution position rule as frames::people::person_briefs.
        let (x,y) = sim.people().day_block(p.id, now)
            .map_or_else(|| p.position_at(now.minutes() as f64), |b| b.at);
        json!({"id":p.id.get().to_string(),"x":x,"y":y,"name":p.given.chars().take(80).collect::<String>()})
    }).collect();
    json!({"version":1,"sampled_at":SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs_f64(),
           "publisher_pid":std::process::id(),"world_id":sim.meta().world_id_hex(),
           "name":sim.meta().name,"seed":sim.meta().seed.to_string(),"preset":sim.meta().preset_id,"mode":format!("{:?}",sim.mode()),
           "size":map.extent_m().0,"day":now.day_index(),"minute":now.minutes(),
           "terrain":{"side":side,"heights":heights,"water":water},
           "totals":{"settlements":sim.land().settlements.len(),"buildings":sim.land().buildings.len(),"people":sim.people().living()},
           "settlements":settlements,"buildings":buildings,"people":people})
}
