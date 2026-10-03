//! Land profiles (`kind = "land"`): habitat rules and wild resources (ADR-0004).

use civ_land::{Growth, HabitatRule, LandParams, PathParams, ResourceParams};
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
    pub climate_cv: f64,
    pub climate_autocorrelation: f64,
    pub paths: Paths,
    pub habitat: Vec<Habitat>,
    pub resource: Vec<Resource>,
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
    /// Exactly one of `plant` and `animal`.
    pub plant: Option<Plant>,
    pub animal: Option<Animal>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Plant {
    pub production_per_ha_yr: Vec<f64>,
    pub loss_per_day: f64,
    pub season: [f64; 12],
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Animal {
    pub capacity_per_ha: Vec<f64>,
    pub growth_per_year: f64,
    pub spread_per_month: f64,
}

impl Resource {
    fn growth(&self) -> Option<Growth> {
        match (&self.plant, &self.animal) {
            (Some(p), None) => Some(Growth::Plant {
                production_per_ha_yr: p.production_per_ha_yr.clone(),
                loss_per_day: p.loss_per_day,
                season: p.season,
            }),
            (None, Some(a)) => Some(Growth::Animal {
                capacity_per_ha: a.capacity_per_ha.clone(),
                growth_per_year: a.growth_per_year,
                spread_per_month: a.spread_per_month,
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
                })
                .collect(),
            richness_min: self.richness_min,
            richness_max: self.richness_max,
            richness_feature_m: self.richness_feature_m,
            resources,
            climate_cv: self.climate_cv,
            climate_autocorrelation: self.climate_autocorrelation,
            paths: PathParams {
                wear_per_walk: self.paths.wear_per_walk,
                half_life_days: self.paths.wear_half_life_days,
                trail_at: self.paths.trail_at,
                trail_until: self.paths.trail_until,
            },
        })
    }

    /// Range problems, as messages.
    pub fn problems(&self) -> Vec<String> {
        let mut p = Vec::new();
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
        if !(0.0..=2.0).contains(&self.climate_cv) {
            p.push("`climate_cv` must be between 0 and 2".to_owned());
        }
        if !(-0.99..=0.99).contains(&self.climate_autocorrelation) {
            p.push("`climate_autocorrelation` must be between -0.99 and 0.99".to_owned());
        }
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
            match (&r.plant, &r.animal) {
                (Some(g), None) => {
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
                (None, Some(g)) => {
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
                    "resource `{}` needs exactly one of `[resource.plant]` and `[resource.animal]`",
                    r.id
                )),
            }
        }
        p
    }
}
