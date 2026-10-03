//! World-generation presets: authoring types and their compilation into [`TerrainParams`].
//!
//! A preset lists every parameter. Omission is an error, so changing an engine default never
//! silently changes an authored preset (research 01-10 §4.3: "explicit defaults only when omission
//! has a clear meaning").

use civ_world::TerrainParams;
use serde::Deserialize;

/// The `kind` value of a world-generation preset.
pub const KIND: &str = "worldgen_preset";
/// The id segment for world-generation presets: `pack:worldgen/name`.
pub const ID_KIND: &str = "worldgen";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PresetFile {
    pub kind: String,
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub default: bool,
    pub region: Region,
    pub uplift: Uplift,
    pub erosion: Erosion,
    pub refinement: Refinement,
    pub water: Water,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Region {
    pub context_factor: f64,
    pub outlet_edges: u32,
    pub base_level_m: f64,
    pub initial_relief_m: f64,
    pub regional_slope: f64,
    pub trunk_inflow_km2: f64,
    pub window_offset: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Uplift {
    pub uplift_mm_per_yr: f64,
    pub lowland_uplift_fraction: f64,
    pub range_scale_km: f64,
    pub basin_strength: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Erosion {
    pub erodibility: f64,
    pub erodibility_variation: f64,
    pub area_exponent: f64,
    pub channel_initiation_km2: f64,
    pub diffusivity_m2_per_yr: f64,
    pub talus_slope: f64,
    pub endorheic_depth_m: f64,
    pub coarse_iterations: u32,
    pub coarse_dt_years: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Refinement {
    pub fine_iterations: [u32; 3],
    pub fine_dt_years: f64,
    pub detail_amplitude: f64,
    pub routing_jitter: f64,
    pub floodplain_min_area_km2: f64,
    pub floodplain_width_factor: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Water {
    pub sea_level_m: f64,
    pub precipitation_mm_per_yr: f64,
    pub evapotranspiration_mm_per_yr: f64,
    pub lake_evaporation_mm_per_yr: f64,
    pub channel_area_km2: f64,
    pub min_lake_depth_m: f64,
    pub min_lake_area_m2: f64,
    pub channel_width_coefficient: f64,
}

impl PresetFile {
    /// The terrain parameters this preset describes. Field-by-field on purpose: a new parameter
    /// fails to compile here until the authoring format carries it.
    pub fn params(&self) -> TerrainParams {
        let (r, u, e, f, w) = (
            &self.region,
            &self.uplift,
            &self.erosion,
            &self.refinement,
            &self.water,
        );
        TerrainParams {
            context_factor: r.context_factor,
            outlet_edges: r.outlet_edges,
            base_level_m: r.base_level_m,
            initial_relief_m: r.initial_relief_m,
            regional_slope: r.regional_slope,
            trunk_inflow_km2: r.trunk_inflow_km2,
            window_offset: r.window_offset,
            uplift_mm_per_yr: u.uplift_mm_per_yr,
            lowland_uplift_fraction: u.lowland_uplift_fraction,
            range_scale_km: u.range_scale_km,
            basin_strength: u.basin_strength,
            erodibility: e.erodibility,
            erodibility_variation: e.erodibility_variation,
            area_exponent: e.area_exponent,
            channel_initiation_km2: e.channel_initiation_km2,
            diffusivity_m2_per_yr: e.diffusivity_m2_per_yr,
            talus_slope: e.talus_slope,
            endorheic_depth_m: e.endorheic_depth_m,
            coarse_iterations: e.coarse_iterations,
            coarse_dt_years: e.coarse_dt_years,
            fine_iterations: f.fine_iterations,
            fine_dt_years: f.fine_dt_years,
            detail_amplitude: f.detail_amplitude,
            routing_jitter: f.routing_jitter,
            floodplain_min_area_km2: f.floodplain_min_area_km2,
            floodplain_width_factor: f.floodplain_width_factor,
            sea_level_m: w.sea_level_m,
            precipitation_mm_per_yr: w.precipitation_mm_per_yr,
            evapotranspiration_mm_per_yr: w.evapotranspiration_mm_per_yr,
            lake_evaporation_mm_per_yr: w.lake_evaporation_mm_per_yr,
            channel_area_km2: w.channel_area_km2,
            min_lake_depth_m: w.min_lake_depth_m,
            min_lake_area_m2: w.min_lake_area_m2,
            channel_width_coefficient: w.channel_width_coefficient,
        }
    }
}
