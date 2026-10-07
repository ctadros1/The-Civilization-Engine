//! Land profiles (`kind = "land"`): habitat rules and wild resources (ADR-0004).

use civ_land::deposits::DepositRule;
use civ_land::{Growth, HabitatRule, LandParams, PathParams, ResourceParams, WeatherParams};
use serde::Deserialize;

/// The `kind` value of a land profile.
pub const KIND: &str = "land";
/// The id segment: `pack:land/name`.
pub const ID_KIND: &str = "land";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LandFile {
    pub kind: String,
    pub id: String,
    pub name: String,
    pub patch_cells: u32,
    pub channel_area_km2: f64,
    pub richness_min: f64,
    pub richness_max: f64,
    pub richness_feature_m: f64,
    /// How the weather is drawn (ADR-0012 §1; content API 23).
    pub weather: WeatherFile,
    pub paths: Paths,
    pub habitat: Vec<Habitat>,
    pub resource: Vec<Resource>,
    /// Where deposits lie (content API 18); a profile may have none.
    #[serde(default)]
    pub deposit: Vec<DepositFile>,
}

/// Where one kind of deposit lies and how large its bodies are (ADR-0010 §1; content API 18).
/// Every figure is a tuning value: the research gives none (03-05 §1.1-1.2 is qualitative).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DepositFile {
    /// The good it yields.
    pub good: String,
    /// `[least, most]` mean slope of the ground it lies under (rise over run).
    pub slope: [f64; 2],
    /// `[least, most]` height above the nearest channel, metres.
    pub hand_m: [f64; 2],
    /// Bodies expected on each square kilometre of land that qualifies.
    pub per_km2: f64,
    /// `[least, most]` radius of a body, metres.
    pub radius_m: [f64; 2],
    /// `[least, most]` depth of what covers it, metres.
    pub top_m: [f64; 2],
    /// `[least, most]` thickness of a body, metres.
    pub thickness_m: [f64; 2],
    /// `[least, most]` quality, 0 to 1.
    pub quality: [f64; 2],
    /// The share of covered bodies that still show (a stream cut, a slip).
    pub exposed_share: f64,
    /// Kilograms of the good in a cubic metre of a body.
    pub density_kg_m3: f64,
    /// Hours a capable adult takes to break a cubic metre of a body out of the ground with the
    /// founders' tools (content API 21); 0, the default, to dig it as earth at the people
    /// profile's `[digging]` rate.
    #[serde(default)]
    pub dig_h_per_m3: f64,
    /// What a working of it is called (content API 21): "pit", the default, or "quarry".
    #[serde(default = "pit")]
    pub working: String,
}

/// A working's name when a deposit rule gives none.
fn pit() -> String {
    "pit".to_owned()
}

/// How a landscape's weather is drawn (ADR-0012 §1; content API 23). Monthly values are for
/// January first. See [`WeatherParams`] for what each means.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WeatherFile {
    pub wet_days: [f64; 12],
    pub rain_share: [f64; 12],
    pub persistence: f64,
    pub gamma_shape: f64,
    pub wet_day_mm: f64,
    pub mean_c: [f64; 12],
    pub day_sd_c: [f64; 12],
    pub day_persistence: f64,
    pub day_range_c: [f64; 12],
    pub wet_day_range: f64,
    pub wet_day_cooling_c: [f64; 12],
    pub normals_at_m: f64,
    pub lapse_c_per_km: f64,
    pub slow_months: f64,
    pub slow_amount: f64,
    pub slow_wet_days: f64,
    pub slow_warmth_c: [f64; 12],
    pub snow_below_c: f64,
    pub melt_mm_per_c: f64,
    pub soil_water_mm: f64,
    pub easy_water_share: f64,
    pub cover_kc: f64,
    /// Content API 24: a day's rain or snow, mm, that keeps people off the ground, and the mean
    /// below which it is frozen (ADR-0012 §5).
    pub wet_ground_mm: f64,
    pub frozen_below_c: f64,
    /// Content API 25: the month's storm, its median in kilopascals on a roof's plan and the
    /// spread of its logarithm, and how much of the snow lying a roof keeps by its pitch
    /// (ADR-0012 §5). They replace `[peak_load]`, the stand-in for weather.
    pub storm_median_kpa: f64,
    pub storm_spread: f64,
    pub roof_snow_share: f64,
    pub roof_snow_full_deg: f64,
    pub roof_snow_shed_deg: f64,
}

impl WeatherFile {
    fn params(&self) -> WeatherParams {
        WeatherParams {
            wet_days: self.wet_days,
            rain_share: self.rain_share,
            persistence: self.persistence,
            gamma_shape: self.gamma_shape,
            wet_day_mm: self.wet_day_mm,
            mean_c: self.mean_c,
            day_sd_c: self.day_sd_c,
            day_persistence: self.day_persistence,
            day_range_c: self.day_range_c,
            wet_day_range: self.wet_day_range,
            wet_day_cooling_c: self.wet_day_cooling_c,
            normals_at_m: self.normals_at_m,
            lapse_c_per_km: self.lapse_c_per_km,
            slow_months: self.slow_months,
            slow_amount: self.slow_amount,
            slow_wet_days: self.slow_wet_days,
            slow_warmth_c: self.slow_warmth_c,
            snow_below_c: self.snow_below_c,
            melt_mm_per_c: self.melt_mm_per_c,
            soil_water_mm: self.soil_water_mm,
            easy_water_share: self.easy_water_share,
            cover_kc: self.cover_kc,
            wet_ground_mm: self.wet_ground_mm,
            frozen_below_c: self.frozen_below_c,
            storm_median_pa: self.storm_median_kpa * 1000.0,
            storm_spread: self.storm_spread,
            roof_snow_share: self.roof_snow_share,
            roof_snow_full_deg: self.roof_snow_full_deg,
            roof_snow_shed_deg: self.roof_snow_shed_deg,
        }
    }

    /// Range problems, as messages.
    fn problems(&self, p: &mut Vec<String>) {
        let one = |name: &str, v: f64, least: f64, most: f64, p: &mut Vec<String>| {
            if !(v.is_finite() && (least..=most).contains(&v)) {
                p.push(format!(
                    "`weather.{name}` must be between {least} and {most} (got {v})"
                ));
            }
        };
        let months = |name: &str, v: &[f64; 12], least: f64, most: f64, p: &mut Vec<String>| {
            if let Some(bad) = v
                .iter()
                .find(|x| !(x.is_finite() && (least..=most).contains(*x)))
            {
                p.push(format!(
                    "every month of `weather.{name}` must be between {least} and {most} (got {bad})"
                ));
            }
        };
        months("wet_days", &self.wet_days, 0.0, 0.95, p);
        months("rain_share", &self.rain_share, 0.0, 1.0, p);
        let sum: f64 = self.rain_share.iter().sum();
        if !(0.98..=1.02).contains(&sum) {
            p.push(format!(
                "the months of `weather.rain_share` must sum to 1 (got {sum:.3})"
            ));
        }
        one("persistence", self.persistence, 0.0, 0.95, p);
        one("gamma_shape", self.gamma_shape, 0.2, 5.0, p);
        one("wet_day_mm", self.wet_day_mm, 0.1, 10.0, p);
        months("mean_c", &self.mean_c, -60.0, 50.0, p);
        months("day_sd_c", &self.day_sd_c, 0.0, 15.0, p);
        one("day_persistence", self.day_persistence, 0.0, 0.99, p);
        months("day_range_c", &self.day_range_c, 0.0, 40.0, p);
        one("wet_day_range", self.wet_day_range, 0.1, 1.0, p);
        months("wet_day_cooling_c", &self.wet_day_cooling_c, -10.0, 10.0, p);
        one("normals_at_m", self.normals_at_m, -500.0, 9000.0, p);
        one("lapse_c_per_km", self.lapse_c_per_km, 0.0, 12.0, p);
        one("slow_months", self.slow_months, 1.0, 600.0, p);
        one("slow_amount", self.slow_amount, 0.0, 1.0, p);
        one("slow_wet_days", self.slow_wet_days, 0.0, 0.5, p);
        months("slow_warmth_c", &self.slow_warmth_c, -5.0, 5.0, p);
        one("snow_below_c", self.snow_below_c, -5.0, 5.0, p);
        one("melt_mm_per_c", self.melt_mm_per_c, 0.0, 20.0, p);
        one("soil_water_mm", self.soil_water_mm, 10.0, 500.0, p);
        one("easy_water_share", self.easy_water_share, 0.1, 0.9, p);
        one("cover_kc", self.cover_kc, 0.1, 2.0, p);
        one("wet_ground_mm", self.wet_ground_mm, 0.5, 100.0, p);
        one("frozen_below_c", self.frozen_below_c, -10.0, 5.0, p);
        one("storm_median_kpa", self.storm_median_kpa, 0.0, 50.0, p);
        one("storm_spread", self.storm_spread, 0.0, 3.0, p);
        one("roof_snow_share", self.roof_snow_share, 0.0, 2.0, p);
        one("roof_snow_full_deg", self.roof_snow_full_deg, 0.0, 90.0, p);
        one("roof_snow_shed_deg", self.roof_snow_shed_deg, 0.0, 90.0, p);
        if self.roof_snow_shed_deg <= self.roof_snow_full_deg {
            p.push(format!(
                "`weather.roof_snow_shed_deg` must be above `weather.roof_snow_full_deg` (got {} \
                 and {})",
                self.roof_snow_shed_deg, self.roof_snow_full_deg
            ));
        }
    }
}

/// How walking wears the ground (research 10-03 §1.1, §2.2).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Paths {
    pub wear_per_walk: f64,
    pub wear_half_life_days: f64,
    pub trail_at: f64,
    pub trail_until: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Habitat {
    pub id: String,
    pub name: String,
    pub min_water_fraction: Option<f64>,
    pub max_median_hand_m: Option<f64>,
    pub max_mean_slope: Option<f64>,
    pub arable: bool,
    /// Person-hours to clear a hectare before it is first broken for a field (woodland); absent
    /// for open ground.
    pub clear_h_per_ha: Option<f64>,
    /// How fast posts set in its ground rot at their foot, against average ground.
    pub wetness: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Resource {
    pub id: String,
    pub name: String,
    /// The good a harvest yields: a good id.
    pub good: String,
    pub unit_kg: f64,
    pub discrete: bool,
    pub in_water: bool,
    pub range_patches: u32,
    pub max_rate_per_hour: f64,
    pub half_rate_stock_per_ha: f64,
    /// Exactly one of `plant`, `animal` and `deposit`.
    pub plant: Option<Plant>,
    pub animal: Option<Animal>,
    pub deposit: Option<Deposit>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Plant {
    pub production_per_ha_yr: Vec<f64>,
    pub loss_per_day: f64,
    pub season: [f64; 12],
    /// Its production follows the soil water month by month (ADR-0012 §5; content API 23).
    #[serde(default)]
    pub follows_water: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Animal {
    pub capacity_per_ha: Vec<f64>,
    pub growth_per_year: f64,
    pub spread_per_month: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Deposit {
    pub stock_per_ha: Vec<f64>,
}

impl Resource {
    fn growth(&self) -> Option<Growth> {
        match (&self.plant, &self.animal, &self.deposit) {
            (Some(p), None, None) => Some(Growth::Plant {
                production_per_ha_yr: p.production_per_ha_yr.clone(),
                loss_per_day: p.loss_per_day,
                season: p.season,
                follows_water: p.follows_water,
            }),
            (None, Some(a), None) => Some(Growth::Animal {
                capacity_per_ha: a.capacity_per_ha.clone(),
                growth_per_year: a.growth_per_year,
                spread_per_month: a.spread_per_month,
            }),
            (None, None, Some(d)) => Some(Growth::Deposit {
                stock_per_ha: d.stock_per_ha.clone(),
            }),
            _ => None,
        }
    }
}

impl LandFile {
    /// The parameters, with each resource's good resolved by `good_index`. `None` if a resource
    /// has no single growth form or names an unknown good ([`LandFile::problems`] and the
    /// cross-file check report those).
    pub fn params(&self, good_index: &dyn Fn(&str) -> Option<usize>) -> Option<LandParams> {
        let mut resources = Vec::with_capacity(self.resource.len());
        for r in &self.resource {
            resources.push(ResourceParams {
                id: r.id.clone(),
                name: r.name.clone(),
                good: good_index(&r.good)?,
                unit_kg: r.unit_kg,
                discrete: r.discrete,
                in_water: r.in_water,
                range_patches: r.range_patches,
                growth: r.growth()?,
                max_rate_per_hour: r.max_rate_per_hour,
                half_rate_stock_per_ha: r.half_rate_stock_per_ha,
            });
        }
        Some(LandParams {
            patch_cells: self.patch_cells,
            channel_area_km2: self.channel_area_km2,
            habitats: self
                .habitat
                .iter()
                .map(|h| HabitatRule {
                    id: h.id.clone(),
                    name: h.name.clone(),
                    min_water_fraction: h.min_water_fraction,
                    max_median_hand_m: h.max_median_hand_m,
                    max_mean_slope: h.max_mean_slope,
                    arable: h.arable,
                    clear_h_per_ha: h.clear_h_per_ha.unwrap_or(0.0),
                    wetness: h.wetness,
                })
                .collect(),
            richness_min: self.richness_min,
            richness_max: self.richness_max,
            richness_feature_m: self.richness_feature_m,
            resources,
            weather: self.weather.params(),
            paths: PathParams {
                wear_per_walk: self.paths.wear_per_walk,
                half_life_days: self.paths.wear_half_life_days,
                trail_at: self.paths.trail_at,
                trail_until: self.paths.trail_until,
            },
            deposits: self
                .deposit
                .iter()
                .map(|d| {
                    let pair = |[lo, hi]: [f64; 2]| (lo, hi);
                    Some(DepositRule {
                        good: good_index(&d.good)?,
                        slope: pair(d.slope),
                        hand_m: pair(d.hand_m),
                        per_km2: d.per_km2,
                        radius_m: pair(d.radius_m),
                        top_m: pair(d.top_m),
                        thickness_m: pair(d.thickness_m),
                        quality: pair(d.quality),
                        exposed_share: d.exposed_share,
                        density_kg_m3: d.density_kg_m3,
                        dig_h_per_m3: d.dig_h_per_m3,
                        working: d.working.clone(),
                    })
                })
                .collect::<Option<Vec<_>>>()?,
        })
    }

    /// Range problems, as messages.
    pub fn problems(&self) -> Vec<String> {
        let mut p = Vec::new();
        for (n, d) in self.deposit.iter().enumerate() {
            let n = n + 1;
            let range =
                |name: &str, [lo, hi]: [f64; 2], least: f64, most: f64, p: &mut Vec<String>| {
                    if !(lo.is_finite() && hi.is_finite() && least <= lo && lo <= hi && hi <= most)
                    {
                        p.push(format!(
                            "deposit {n}'s `{name}` must be [least, most] within {least} to {most} \
                         (got [{lo}, {hi}])"
                        ));
                    }
                };
            range("slope", d.slope, 0.0, 10.0, &mut p);
            range("hand_m", d.hand_m, 0.0, 10_000.0, &mut p);
            range("radius_m", d.radius_m, 0.5, 1_000.0, &mut p);
            range("top_m", d.top_m, 0.0, 100.0, &mut p);
            range("thickness_m", d.thickness_m, 0.05, 100.0, &mut p);
            range("quality", d.quality, 0.0, 1.0, &mut p);
            if !(d.per_km2.is_finite() && (0.0..=1_000.0).contains(&d.per_km2)) {
                p.push(format!(
                    "deposit {n}'s `per_km2` must be between 0 and 1000"
                ));
            }
            if !(d.exposed_share.is_finite() && (0.0..=1.0).contains(&d.exposed_share)) {
                p.push(format!(
                    "deposit {n}'s `exposed_share` must be between 0 and 1"
                ));
            }
            if !(d.density_kg_m3.is_finite() && (100.0..=10_000.0).contains(&d.density_kg_m3)) {
                p.push(format!(
                    "deposit {n}'s `density_kg_m3` must be between 100 and 10000"
                ));
            }
            if !(d.dig_h_per_m3.is_finite() && (0.0..=1_000.0).contains(&d.dig_h_per_m3)) {
                p.push(format!(
                    "deposit {n}'s `dig_h_per_m3` must be between 0 and 1000"
                ));
            }
            let words = d.working.trim();
            if words.is_empty() || words.len() > 24 || words != d.working {
                p.push(format!(
                    "deposit {n}'s `working` must be a name of 1 to 24 characters (got {:?})",
                    d.working
                ));
            }
        }
        if !(4..=64).contains(&self.patch_cells) {
            p.push(format!(
                "`patch_cells` must be between 4 and 64 (got {})",
                self.patch_cells
            ));
        }
        if !(self.channel_area_km2.is_finite() && self.channel_area_km2 > 0.0) {
            p.push("`channel_area_km2` must be positive".to_owned());
        }
        if !(self.richness_min > 0.0 && self.richness_min <= self.richness_max) {
            p.push("richness must satisfy 0 < richness_min <= richness_max".to_owned());
        }
        if !(self.richness_feature_m.is_finite() && self.richness_feature_m > 0.0) {
            p.push("`richness_feature_m` must be positive".to_owned());
        }
        self.weather.problems(&mut p);
        let w = &self.paths;
        if !(w.wear_per_walk > 0.0 && w.wear_per_walk < 1.0) {
            p.push("`paths.wear_per_walk` must be above 0 and below 1".to_owned());
        }
        if !(w.wear_half_life_days.is_finite() && w.wear_half_life_days >= 1.0) {
            p.push("`paths.wear_half_life_days` must be at least 1".to_owned());
        }
        if !(w.trail_until > 0.0 && w.trail_until <= w.trail_at && w.trail_at < 1.0) {
            p.push(
                "trails must satisfy 0 < `paths.trail_until` <= `paths.trail_at` < 1".to_owned(),
            );
        }
        if self.habitat.is_empty() || self.habitat.len() > 32 {
            p.push("a land profile needs 1 to 32 habitats".to_owned());
        }
        if let Some(last) = self.habitat.last()
            && (last.min_water_fraction.is_some()
                || last.max_median_hand_m.is_some()
                || last.max_mean_slope.is_some())
        {
            p.push(format!(
                "the last habitat (`{}`) must have no conditions, so every patch matches one",
                last.id
            ));
        }
        let mut seen = std::collections::HashSet::new();
        for h in &self.habitat {
            if !seen.insert(h.id.as_str()) {
                p.push(format!("habitat `{}` is listed twice", h.id));
            }
            if !(h.wetness.is_finite() && h.wetness >= 0.0) {
                p.push(format!(
                    "habitat `{}`: `wetness` must be zero or more (got {})",
                    h.id, h.wetness
                ));
            }
            if let Some(c) = h.clear_h_per_ha {
                if !(c.is_finite() && c >= 0.0) {
                    p.push(format!(
                        "habitat `{}`: `clear_h_per_ha` must be zero or more (got {c})",
                        h.id
                    ));
                }
                if !h.arable {
                    p.push(format!(
                        "habitat `{}`: only arable ground is cleared for fields (`clear_h_per_ha` \
                         needs `arable = true`)",
                        h.id
                    ));
                }
            }
        }
        let mut seen = std::collections::HashSet::new();
        let habitats = self.habitat.len();
        for r in &self.resource {
            if !seen.insert(r.id.as_str()) {
                p.push(format!("resource `{}` is listed twice", r.id));
            }
            if !(r.unit_kg.is_finite() && r.unit_kg > 0.0) {
                p.push(format!("resource `{}` needs a positive `unit_kg`", r.id));
            }
            if r.range_patches > 8 {
                p.push(format!(
                    "resource `{}` `range_patches` must be at most 8 (got {})",
                    r.id, r.range_patches
                ));
            }
            if !(r.max_rate_per_hour > 0.0 && r.half_rate_stock_per_ha > 0.0) {
                p.push(format!(
                    "resource `{}` needs positive max_rate_per_hour and half_rate_stock_per_ha",
                    r.id
                ));
            }
            let per_habitat = |what: &str, values: &[f64], p: &mut Vec<String>| {
                if values.len() != habitats {
                    p.push(format!(
                        "resource `{}` needs one {what} figure per habitat ({habitats})",
                        r.id
                    ));
                }
                if values.iter().any(|v| !(v.is_finite() && *v >= 0.0)) {
                    p.push(format!("resource `{}` {what} must be zero or more", r.id));
                }
            };
            match (&r.plant, &r.animal, &r.deposit) {
                (None, None, Some(d)) => per_habitat("stock", &d.stock_per_ha, &mut p),
                (Some(g), None, None) => {
                    per_habitat("production", &g.production_per_ha_yr, &mut p);
                    if !(g.loss_per_day.is_finite() && (0.0..1.0).contains(&g.loss_per_day)) {
                        p.push(format!(
                            "resource `{}` loss_per_day must be in [0, 1)",
                            r.id
                        ));
                    }
                    if g.season.iter().any(|v| !(v.is_finite() && *v >= 0.0)) {
                        p.push(format!(
                            "resource `{}` season weights must be zero or more",
                            r.id
                        ));
                    }
                }
                (None, Some(g), None) => {
                    per_habitat("capacity", &g.capacity_per_ha, &mut p);
                    if !(g.growth_per_year.is_finite() && (0.0..=5.0).contains(&g.growth_per_year))
                    {
                        p.push(format!(
                            "resource `{}` growth_per_year must be between 0 and 5",
                            r.id
                        ));
                    }
                    if !(g.spread_per_month.is_finite()
                        && (0.0..=1.0).contains(&g.spread_per_month))
                    {
                        p.push(format!(
                            "resource `{}` spread_per_month must be between 0 and 1",
                            r.id
                        ));
                    }
                }
                _ => p.push(format!(
                    "resource `{}` needs exactly one of `[resource.plant]`, `[resource.animal]` \
                     and `[resource.deposit]`",
                    r.id
                )),
            }
        }
        p
    }
}
