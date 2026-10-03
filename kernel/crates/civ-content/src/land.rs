//! Land profiles (`kind = "land"`): habitat rules and wild resources (ADR-0004).

use civ_land::{HabitatRule, LandParams, ResourceParams};
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
    pub habitat: Vec<Habitat>,
    pub resource: Vec<Resource>,
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
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Resource {
    pub id: String,
    pub name: String,
    pub unit: String,
    pub production_per_ha_yr: Vec<f64>,
    pub loss_per_day: f64,
    pub season: [f64; 12],
    pub max_rate_per_hour: f64,
    pub half_rate_stock_per_ha: f64,
}

impl LandFile {
    /// The parameters.
    pub fn params(&self) -> LandParams {
        LandParams {
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
                })
                .collect(),
            richness_min: self.richness_min,
            richness_max: self.richness_max,
            richness_feature_m: self.richness_feature_m,
            resources: self
                .resource
                .iter()
                .map(|r| ResourceParams {
                    id: r.id.clone(),
                    name: r.name.clone(),
                    unit: r.unit.clone(),
                    production_per_ha_yr: r.production_per_ha_yr.clone(),
                    loss_per_day: r.loss_per_day,
                    season: r.season,
                    max_rate_per_hour: r.max_rate_per_hour,
                    half_rate_stock_per_ha: r.half_rate_stock_per_ha,
                })
                .collect(),
            climate_cv: self.climate_cv,
            climate_autocorrelation: self.climate_autocorrelation,
        }
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
        }
        let mut seen = std::collections::HashSet::new();
        for r in &self.resource {
            if !seen.insert(r.id.as_str()) {
                p.push(format!("resource `{}` is listed twice", r.id));
            }
            if r.production_per_ha_yr.len() != self.habitat.len() {
                p.push(format!(
                    "resource `{}` needs one production figure per habitat ({})",
                    r.id,
                    self.habitat.len()
                ));
            }
            if r.production_per_ha_yr
                .iter()
                .any(|v| !(v.is_finite() && *v >= 0.0))
            {
                p.push(format!(
                    "resource `{}` production must be zero or more",
                    r.id
                ));
            }
            if !(r.loss_per_day.is_finite() && (0.0..1.0).contains(&r.loss_per_day)) {
                p.push(format!(
                    "resource `{}` loss_per_day must be in [0, 1)",
                    r.id
                ));
            }
            if r.season.iter().any(|v| !(v.is_finite() && *v >= 0.0)) {
                p.push(format!(
                    "resource `{}` season weights must be zero or more",
                    r.id
                ));
            }
            if !(r.max_rate_per_hour > 0.0 && r.half_rate_stock_per_ha > 0.0) {
                p.push(format!(
                    "resource `{}` needs positive max_rate_per_hour and half_rate_stock_per_ha",
                    r.id
                ));
            }
        }
        p
    }
}
