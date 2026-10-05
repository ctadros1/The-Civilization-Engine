//! `civ-host weather`: a world's weather on the valley floor, year by year (ADR-0012 §1).
//!
//! A world's weather is drawn from its seed and landscape alone, so it can be lived without the
//! world: this replays it from where a new world's begins (a year before the founding), at the
//! height the profile's normals are for, and reports each calendar year with what a crop sown on
//! its typical day would have had of its water. It is the weather the world of that seed lives,
//! whatever its people do; fields higher or lower, or sown on other days, differ.

use std::io::Write;

use civ_content::{ContentRegistry, WorldgenPreset};
use civ_core::time::{DAYS_PER_YEAR, DEFAULT_WORLD_START};
use civ_land::weather::{self, MonthRecord, Unworkable, Weather, root_zone_day};
use civ_land::{Climatology, day_of_year};

/// What to draw.
#[derive(Clone, Debug)]
pub struct WeatherOptions {
    /// The preset (landscape) id; the content's default when `None`.
    pub preset: Option<String>,
    /// The world's seed.
    pub seed: u64,
    /// Calendar years to report, from the world's first.
    pub years: u32,
}

/// One calendar year's weather on the valley floor.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Year {
    /// Its number, the world's first being 1.
    pub year: i64,
    /// Rain and snow, mm of water.
    pub precip_mm: f64,
    /// Wet days.
    pub wet_days: u32,
    /// Mean temperature, °C.
    pub mean_c: f64,
    /// Days of frost.
    pub frost_days: u32,
    /// Days ending with snow lying.
    pub snow_days: u32,
    /// Rain and snow from April to July, mm.
    pub growing_mm: f64,
    /// The share of its water need a crop sown on its typical day had.
    pub crop_water: f64,
    /// That crop's harvest as a share of an average year's (ADR-0012 §2).
    pub crop_factor: f64,
    /// Days of the crop's window for preparing and sowing in the year, and those of them the
    /// ground could be worked (ADR-0012 §5).
    pub window_days: u32,
    pub workable_days: u32,
}

fn preset<'a>(
    content: &'a ContentRegistry,
    id: Option<&str>,
) -> anyhow::Result<&'a WorldgenPreset> {
    match id {
        Some(id) => content
            .preset(id)
            .ok_or_else(|| anyhow::anyhow!("no preset {id} in the content")),
        None => Ok(content.default_preset()),
    }
}

/// The weather a world of `seed` made from `preset` lives on the valley floor, from its first day
/// through `until` (a day index), calling `each` with every day lived, what reached the ground,
/// and why the ground could not be worked that day, if it could not.
pub fn replay(
    content: &ContentRegistry,
    preset: &WorldgenPreset,
    seed: u64,
    until: i64,
    mut each: impl FnMut(i64, &weather::DayWater, Option<Unworkable>),
) -> (Climatology, Weather) {
    let climate = civ_sim::climatology_of(content, preset, seed);
    let params = &content.land.params.weather;
    let soil_params = &content.land.params.soil;
    let at = params.normals_at_m;
    // A new world's weather begins a year before its founding (`Land::create`).
    let first = DEFAULT_WORLD_START.day_index() - DAYS_PER_YEAR;
    let mut w = Weather::new(params, soil_params, &climate, at, at, first);
    for day in first..=until {
        let why = w.unworkable(params, at);
        let lived = w.live(params, soil_params, &climate);
        each(day, &lived, why);
    }
    (climate, w)
}

/// Draws `options.years` years of the weather of the world of `options.seed` in its landscape.
pub fn years(content: &ContentRegistry, options: &WeatherOptions) -> anyhow::Result<Vec<Year>> {
    let preset = preset(content, options.preset.as_deref())?;
    let params = &content.land.params.weather;
    let soil_params = &content.land.params.soil;
    let last = i64::from(options.years) * DAYS_PER_YEAR - 1;
    let mut out: Vec<Year> = (1..=i64::from(options.years))
        .map(|year| Year {
            year,
            crop_water: 1.0,
            crop_factor: 1.0,
            ..Year::default()
        })
        .collect();
    // Each season's crop water: the root zone, the need and what the crop got, and the year.
    let mut seasons: Vec<(i64, f64, f64)> = Vec::new();
    let (mut water, mut need, mut got) = (0.0, 0.0, 0.0);
    let crop = civ_land::weather::CropWater::of(
        content
            .catalog
            .crops
            .first()
            .ok_or_else(|| anyhow::anyhow!("the content has no crop"))?,
    );
    let (climate, w) = replay(content, preset, options.seed, last, |day, lived, why| {
        if (crop.window.0..=crop.window.1).contains(&day_of_year(day))
            && let Some(y) = usize::try_from(weather::year_of(day) - 1)
                .ok()
                .and_then(|i| out.get_mut(i))
        {
            y.window_days += 1;
            y.workable_days += u32::from(why.is_none());
        }
        let t = day_of_year(day) - crop.typical_sowing;
        if t == 0 {
            (water, need, got) = (lived.soil_before_mm, 0.0, 0.0);
        }
        if (0..i64::from(crop.grow_days)).contains(&t) {
            let n = crop.kc_on(t) * lived.et0_mm[0];
            got += root_zone_day(&mut water, lived.input_mm[0], n, params, soil_params);
            need += n;
            if t + 1 == i64::from(crop.grow_days) {
                seasons.push((weather::year_of(day), need, got));
            }
        }
    });
    for (year, need, got) in seasons {
        if let Some(y) = out.iter_mut().find(|y| y.year == year) {
            y.crop_water = if need > 0.0 { got / need } else { 1.0 };
            y.crop_factor = climate.crop_factor(0, need, got);
        }
    }
    for m in &w.months {
        if let Some(y) = out.iter_mut().find(|y| y.year == m.year) {
            add_month(y, m);
        }
    }
    for y in &mut out {
        y.mean_c /= DAYS_PER_YEAR as f64;
    }
    Ok(out)
}

fn add_month(y: &mut Year, m: &MonthRecord) {
    y.precip_mm += f64::from(m.precip_mm);
    y.wet_days += u32::from(m.wet_days);
    y.mean_c += f64::from(m.temp_sum_c);
    y.frost_days += u32::from(m.frost_days);
    y.snow_days += u32::from(m.snow_days);
    if (3..=6).contains(&m.month) {
        y.growing_mm += f64::from(m.precip_mm);
    }
}

/// Prints the years and a summary.
pub fn run(
    content: &ContentRegistry,
    options: &WeatherOptions,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    let years = years(content, options)?;
    let preset = preset(content, options.preset.as_deref())?;
    let params = &content.land.params.weather;
    let _soil_params = &content.land.params.soil;
    writeln!(
        out,
        "The weather of {} seed {} on the valley floor ({} m), from the world's first year \
         (ADR-0012); the crop is the content's first, sown on its typical day",
        preset.id, options.seed, params.normals_at_m
    )?;
    writeln!(
        out,
        "year  rain mm  wet days  mean °C  frost  snow  Apr-Jul mm  workable  crop water  harvest"
    )?;
    for y in &years {
        writeln!(
            out,
            "{:>4}  {:>7.0}  {:>8}  {:>7.1}  {:>5}  {:>4}  {:>10.0}  {:>8}  {:>9.0}%  {:>7.2}",
            y.year,
            y.precip_mm,
            y.wet_days,
            y.mean_c,
            y.frost_days,
            y.snow_days,
            y.growing_mm,
            format!("{}/{}", y.workable_days, y.window_days),
            100.0 * y.crop_water,
            y.crop_factor
        )?;
    }
    let n = years.len().max(1) as f64;
    let mean = |f: &dyn Fn(&Year) -> f64| years.iter().map(f).sum::<f64>() / n;
    let sd = |f: &dyn Fn(&Year) -> f64| {
        let m = mean(f);
        (years.iter().map(|y| (f(y) - m).powi(2)).sum::<f64>() / n).sqrt()
    };
    let rain = mean(&|y| y.precip_mm);
    let harvest = mean(&|y| y.crop_factor);
    // How much a year is like the one before: the lag-one autocorrelation.
    let lag_one = |f: &dyn Fn(&Year) -> f64| {
        let m = mean(f);
        let var: f64 = years.iter().map(|y| (f(y) - m).powi(2)).sum();
        let cov: f64 = years
            .windows(2)
            .map(|w| (f(&w[0]) - m) * (f(&w[1]) - m))
            .sum();
        if var > 0.0 { cov / var } else { 0.0 }
    };
    writeln!(
        out,
        "{} years: rain {rain:.0} mm a year (CV {:.2}) in {:.0} wet days, mean {:.1} °C, {:.0} \
         frost days and {:.0} with snow lying; harvest factor mean {harvest:.2} (CV {:.2}), below \
         0.8 in {} years and below 0.6 in {}; a year's rain and harvest correlate with the \
         year before's at {:.2} and {:.2}",
        years.len(),
        sd(&|y| y.precip_mm) / rain.max(1e-9),
        mean(&|y| f64::from(y.wet_days)),
        mean(&|y| y.mean_c),
        mean(&|y| f64::from(y.frost_days)),
        mean(&|y| f64::from(y.snow_days)),
        sd(&|y| y.crop_factor) / harvest.max(1e-9),
        years.iter().filter(|y| y.crop_factor < 0.8).count(),
        years.iter().filter(|y| y.crop_factor < 0.6).count(),
        lag_one(&|y| y.precip_mm),
        lag_one(&|y| y.crop_factor),
    )?;
    // The days the ground could be worked in the window for preparing and sowing, against what
    // people plan on: the landscape's share over its long run.
    let climate = civ_sim::climatology_of(content, preset, options.seed);
    let window: u32 = years.iter().map(|y| y.window_days).sum();
    let workable: u32 = years.iter().map(|y| y.workable_days).sum();
    writeln!(
        out,
        "the ground could be worked on {:.0}% of the days of the crop's window for preparing and \
         sowing (people plan on {:.0}%, the landscape's long run); a season's crop usually needs \
         {:.0} mm",
        100.0 * f64::from(workable) / f64::from(window.max(1)),
        100.0 * climate.workable_share.first().copied().unwrap_or(1.0),
        climate.season_need_mm.first().copied().unwrap_or(0.0),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    fn content() -> ContentRegistry {
        let root = civ_content::find_content_root(Path::new(env!("CARGO_MANIFEST_DIR")))
            .expect("content/ is above the crate");
        civ_content::load(&root).registry.expect("content loads")
    }

    #[test]
    fn the_command_replays_the_weather_a_world_of_the_seed_lives() {
        let content = content();
        let preset = content.default_preset();
        let world = civ_sim::Sim::create_for_tests(
            &civ_sim::NewWorld {
                name: "Weather".into(),
                seed: 2,
                preset_id: preset.id.clone(),
                size_cells: 256,
                band_size: 0,
                regime_id: String::new(),
            },
            &content,
            [3; 16],
        )
        .expect("world");
        // A new world has lived the year before its founding; the replay lives the same days.
        let lived = world.land().stock_day;
        let (_, w) = replay(&content, preset, 2, lived, |_, _, _| {});
        assert_eq!(w.months, world.land().weather.months);
        assert_eq!(w.today, world.land().weather.today);
        assert_eq!(w.soil_mm, world.land().weather.soil_mm);
        // The yearly report is the same each time, and plausible.
        let options = WeatherOptions {
            preset: None,
            seed: 2,
            years: 3,
        };
        let a = years(&content, &options).expect("years");
        assert_eq!(a, years(&content, &options).expect("years"));
        assert_eq!(a.len(), 3);
        assert!(a.iter().all(|y| y.precip_mm > 200.0 && y.wet_days > 50));
        assert!(
            a.iter()
                .all(|y| y.crop_factor >= 0.0 && y.crop_water <= 1.0)
        );
        // Each year counts every day of the crop's window for preparing and sowing, and most of
        // them, not all, as days the ground could be worked.
        let crop = &content.catalog.crops[0];
        let window = u32::from(crop.sow_until_day - crop.prepare_from_day) + 1;
        assert!(a.iter().all(|y| y.window_days == window));
        let workable: u32 = a.iter().map(|y| y.workable_days).sum();
        assert!(
            (window..3 * window).contains(&workable),
            "{workable} of {}",
            3 * window
        );
    }
}
