//! World-generation parameters.
//!
//! These are authored vocabulary: content presets (`content/core/worldgen/*.toml`) supply them and
//! describe *kinds* of landscape. The seed and the map size come from whoever creates the world.
//! Nothing here places a settlement, a resource or an event.
//!
//! Units are SI throughout, except where a field name says otherwise.

/// Parameters for generating terrain and water.
#[derive(Clone, Debug, PartialEq)]
pub struct TerrainParams {
    // Regional structure (coarse context).
    /// Extent of the simulated watershed context relative to the playable map (≥ 1). Rivers can
    /// enter the map from this context (research 03-01 §5.1).
    pub context_factor: f64,
    /// Number of context edges held at base level (1 or 2 adjacent edges). Which edges is decided
    /// by the seed.
    pub outlet_edges: u32,
    /// Elevation of the base-level edges, in metres above sea level (negative for a coast).
    pub base_level_m: f64,
    /// Amplitude of the initial surface's low-frequency relief, in metres.
    pub initial_relief_m: f64,
    /// Initial regional slope toward the base-level edges (m/m).
    pub regional_slope: f64,
    /// Upstream catchment of a trunk river entering the context from beyond it, km² (0 = none).
    /// This is the "explicit upstream boundary flow" of research 03-02 §1.1: a 16 km map cannot
    /// grow a large river from its own rainfall. Where it enters is decided by the seed.
    pub trunk_inflow_km2: f64,
    /// Where the playable map sits in the context: 0 = centred, 1 = against the base-level edge.
    /// Coastal presets use a high value so the coastline is on the map.
    pub window_offset: f64,

    // Uplift and erosion (coarse landscape evolution).
    /// Peak rock uplift rate, mm per year.
    pub uplift_mm_per_yr: f64,
    /// Uplift in lowlands as a fraction of the peak.
    pub lowland_uplift_fraction: f64,
    /// Typical spacing of mountain ranges, km.
    pub range_scale_km: f64,
    /// Strength of subsiding pockets that become closed basins and lakes (0 = none).
    pub basin_strength: f64,
    /// Stream-power erodibility K (with drainage area in m², units m^(1-2m)/yr).
    pub erodibility: f64,
    /// Relative spatial variation of K, for rock of differing resistance (0–0.9).
    pub erodibility_variation: f64,
    /// Drainage-area exponent m of the stream-power law (n = 1).
    pub area_exponent: f64,
    /// Drainage area below which ground is hillslope (diffusion and talus only) rather than
    /// channel, km². Gives convex hilltops instead of rills on every slope.
    pub channel_initiation_km2: f64,
    /// Hillslope diffusivity, m²/yr.
    pub diffusivity_m2_per_yr: f64,
    /// Steepest stable slope of loose material (tan of the angle).
    pub talus_slope: f64,
    /// During landscape evolution, depressions deeper than this keep their water instead of
    /// overflowing, metres (0 = every basin overflows). Arid presets use it: a closed basin
    /// does not erode its own outlet, so it stays closed.
    pub endorheic_depth_m: f64,
    /// Coarse landscape-evolution iterations.
    pub coarse_iterations: u32,
    /// Coarse time step, years.
    pub coarse_dt_years: f64,

    // Refinement to simulation resolution.
    /// Erosion iterations at each refinement level (cell sizes 4×, 2× and 1× the simulation cell).
    pub fine_iterations: [u32; 3],
    /// Fine time step, years.
    pub fine_dt_years: f64,
    /// Amplitude of added detail as a fraction of local relief.
    pub detail_amplitude: f64,
    /// Strength of the seeded perturbation of flow directions (0 = plain D8; see
    /// `routing::Jitter`).
    pub routing_jitter: f64,
    /// Rivers draining at least this much area get an alluvial valley floor, km².
    pub floodplain_min_area_km2: f64,
    /// Floodplain half-width as a multiple of channel width (0 = no floodplains).
    pub floodplain_width_factor: f64,

    // Water.
    /// Sea level, metres.
    pub sea_level_m: f64,
    /// Mean annual precipitation, mm.
    pub precipitation_mm_per_yr: f64,
    /// Mean annual evapotranspiration from land, mm (runoff = precipitation − this).
    pub evapotranspiration_mm_per_yr: f64,
    /// Mean annual evaporation from open water, mm.
    pub lake_evaporation_mm_per_yr: f64,
    /// Drainage area at which a channel begins, km².
    pub channel_area_km2: f64,
    /// Shallower depressions are treated as terrain artifacts and filled, metres.
    pub min_lake_depth_m: f64,
    /// Smaller depressions are treated as terrain artifacts and filled, m².
    pub min_lake_area_m2: f64,
    /// Coefficient `a` of channel width `w = a·Q^0.5` (w in m, Q in m³/s).
    pub channel_width_coefficient: f64,
}

impl Default for TerrainParams {
    /// A temperate inland river valley. The values are tuning starting points (research 03-01:
    /// "m = 0.5, n = 1 as a tuning choice"), not calibrations.
    fn default() -> Self {
        TerrainParams {
            context_factor: 2.0,
            outlet_edges: 1,
            base_level_m: 60.0,
            initial_relief_m: 40.0,
            regional_slope: 0.004,
            trunk_inflow_km2: 1500.0,
            window_offset: 0.0,
            uplift_mm_per_yr: 0.6,
            lowland_uplift_fraction: 0.04,
            range_scale_km: 14.0,
            basin_strength: 0.0,
            erodibility: 1.0e-5,
            erodibility_variation: 0.35,
            area_exponent: 0.5,
            channel_initiation_km2: 0.0,
            diffusivity_m2_per_yr: 0.04,
            talus_slope: 0.85,
            endorheic_depth_m: 0.0,
            coarse_iterations: 150,
            coarse_dt_years: 20_000.0,
            fine_iterations: [10, 6, 3],
            fine_dt_years: 2_000.0,
            detail_amplitude: 0.25,
            routing_jitter: 0.35,
            floodplain_min_area_km2: 2.0,
            floodplain_width_factor: 45.0,
            sea_level_m: 0.0,
            precipitation_mm_per_yr: 800.0,
            evapotranspiration_mm_per_yr: 500.0,
            lake_evaporation_mm_per_yr: 900.0,
            channel_area_km2: 0.5,
            min_lake_depth_m: 2.0,
            min_lake_area_m2: 5_000.0,
            channel_width_coefficient: 4.0,
        }
    }
}

impl TerrainParams {
    /// Every parameter as a (name, value) pair in a fixed order: for content fingerprints and for
    /// showing a world's provenance. The destructuring is exhaustive on purpose, so adding a
    /// field fails to compile until it is listed here.
    pub fn named_values(&self) -> Vec<(&'static str, f64)> {
        let TerrainParams {
            context_factor,
            outlet_edges,
            base_level_m,
            initial_relief_m,
            regional_slope,
            trunk_inflow_km2,
            window_offset,
            uplift_mm_per_yr,
            lowland_uplift_fraction,
            range_scale_km,
            basin_strength,
            erodibility,
            erodibility_variation,
            area_exponent,
            channel_initiation_km2,
            diffusivity_m2_per_yr,
            talus_slope,
            endorheic_depth_m,
            coarse_iterations,
            coarse_dt_years,
            fine_iterations,
            fine_dt_years,
            detail_amplitude,
            routing_jitter,
            floodplain_min_area_km2,
            floodplain_width_factor,
            sea_level_m,
            precipitation_mm_per_yr,
            evapotranspiration_mm_per_yr,
            lake_evaporation_mm_per_yr,
            channel_area_km2,
            min_lake_depth_m,
            min_lake_area_m2,
            channel_width_coefficient,
        } = *self;
        vec![
            ("context_factor", context_factor),
            ("outlet_edges", f64::from(outlet_edges)),
            ("base_level_m", base_level_m),
            ("initial_relief_m", initial_relief_m),
            ("regional_slope", regional_slope),
            ("trunk_inflow_km2", trunk_inflow_km2),
            ("window_offset", window_offset),
            ("uplift_mm_per_yr", uplift_mm_per_yr),
            ("lowland_uplift_fraction", lowland_uplift_fraction),
            ("range_scale_km", range_scale_km),
            ("basin_strength", basin_strength),
            ("erodibility", erodibility),
            ("erodibility_variation", erodibility_variation),
            ("area_exponent", area_exponent),
            ("channel_initiation_km2", channel_initiation_km2),
            ("diffusivity_m2_per_yr", diffusivity_m2_per_yr),
            ("talus_slope", talus_slope),
            ("endorheic_depth_m", endorheic_depth_m),
            ("coarse_iterations", f64::from(coarse_iterations)),
            ("coarse_dt_years", coarse_dt_years),
            ("fine_iterations_0", f64::from(fine_iterations[0])),
            ("fine_iterations_1", f64::from(fine_iterations[1])),
            ("fine_iterations_2", f64::from(fine_iterations[2])),
            ("fine_dt_years", fine_dt_years),
            ("detail_amplitude", detail_amplitude),
            ("routing_jitter", routing_jitter),
            ("floodplain_min_area_km2", floodplain_min_area_km2),
            ("floodplain_width_factor", floodplain_width_factor),
            ("sea_level_m", sea_level_m),
            ("precipitation_mm_per_yr", precipitation_mm_per_yr),
            ("evapotranspiration_mm_per_yr", evapotranspiration_mm_per_yr),
            ("lake_evaporation_mm_per_yr", lake_evaporation_mm_per_yr),
            ("channel_area_km2", channel_area_km2),
            ("min_lake_depth_m", min_lake_depth_m),
            ("min_lake_area_m2", min_lake_area_m2),
            ("channel_width_coefficient", channel_width_coefficient),
        ]
    }

    /// Mean annual runoff from land, m per year.
    pub fn runoff_m_per_yr(&self) -> f64 {
        ((self.precipitation_mm_per_yr - self.evapotranspiration_mm_per_yr) / 1000.0).max(0.0)
    }

    /// Checks every field against its physically meaningful range.
    pub fn validate(&self) -> Result<(), String> {
        let mut errors = Vec::new();
        let mut check = |ok: bool, what: &str| {
            if !ok {
                errors.push(what.to_owned());
            }
        };
        let all_finite = [
            self.context_factor,
            self.base_level_m,
            self.initial_relief_m,
            self.regional_slope,
            self.trunk_inflow_km2,
            self.window_offset,
            self.routing_jitter,
            self.floodplain_min_area_km2,
            self.floodplain_width_factor,
            self.uplift_mm_per_yr,
            self.lowland_uplift_fraction,
            self.range_scale_km,
            self.basin_strength,
            self.erodibility,
            self.erodibility_variation,
            self.area_exponent,
            self.channel_initiation_km2,
            self.diffusivity_m2_per_yr,
            self.talus_slope,
            self.endorheic_depth_m,
            self.coarse_dt_years,
            self.fine_dt_years,
            self.detail_amplitude,
            self.sea_level_m,
            self.precipitation_mm_per_yr,
            self.evapotranspiration_mm_per_yr,
            self.lake_evaporation_mm_per_yr,
            self.channel_area_km2,
            self.min_lake_depth_m,
            self.min_lake_area_m2,
            self.channel_width_coefficient,
        ]
        .iter()
        .all(|v| v.is_finite());
        check(all_finite, "every parameter must be a finite number");
        check(
            (1.0..=4.0).contains(&self.context_factor),
            "context_factor must be within 1–4",
        );
        check(
            (1..=2).contains(&self.outlet_edges),
            "outlet_edges must be 1 or 2",
        );
        check(
            (-500.0..=3000.0).contains(&self.base_level_m),
            "base_level_m must be within -500–3000",
        );
        check(
            (0.0..=2000.0).contains(&self.initial_relief_m),
            "initial_relief_m must be within 0–2000",
        );
        check(
            (0.0..=0.1).contains(&self.regional_slope),
            "regional_slope must be within 0–0.1",
        );
        check(
            (0.0..=1.0e6).contains(&self.trunk_inflow_km2),
            "trunk_inflow_km2 must be within 0–1e6",
        );
        check(
            (0.0..=1.0).contains(&self.window_offset),
            "window_offset must be within 0–1",
        );
        check(
            (0.0..=0.9).contains(&self.routing_jitter),
            "routing_jitter must be within 0–0.9",
        );
        check(
            self.floodplain_min_area_km2 > 0.0 && self.floodplain_min_area_km2 <= 1.0e5,
            "floodplain_min_area_km2 must be within (0, 1e5]",
        );
        check(
            (0.0..=500.0).contains(&self.floodplain_width_factor),
            "floodplain_width_factor must be within 0–500",
        );
        check(
            (0.0..=20.0).contains(&self.uplift_mm_per_yr),
            "uplift_mm_per_yr must be within 0–20",
        );
        check(
            (0.0..=1.0).contains(&self.lowland_uplift_fraction),
            "lowland_uplift_fraction must be within 0–1",
        );
        check(
            (1.0..=200.0).contains(&self.range_scale_km),
            "range_scale_km must be within 1–200",
        );
        check(
            (0.0..=1.0).contains(&self.basin_strength),
            "basin_strength must be within 0–1",
        );
        check(
            self.erodibility > 0.0 && self.erodibility <= 1.0e-2,
            "erodibility must be within (0, 0.01]",
        );
        check(
            (0.0..=0.9).contains(&self.erodibility_variation),
            "erodibility_variation must be within 0–0.9",
        );
        check(
            (0.2..=0.8).contains(&self.area_exponent),
            "area_exponent must be within 0.2–0.8",
        );
        check(
            (0.0..=100.0).contains(&self.channel_initiation_km2),
            "channel_initiation_km2 must be within 0–100",
        );
        check(
            (0.0..=10.0).contains(&self.diffusivity_m2_per_yr),
            "diffusivity_m2_per_yr must be within 0–10",
        );
        check(
            (0.1..=5.0).contains(&self.talus_slope),
            "talus_slope must be within 0.1–5",
        );
        check(
            (0.0..=1000.0).contains(&self.endorheic_depth_m),
            "endorheic_depth_m must be within 0–1000",
        );
        check(
            (1..=2000).contains(&self.coarse_iterations),
            "coarse_iterations must be within 1–2000",
        );
        check(
            self.coarse_dt_years > 0.0 && self.coarse_dt_years <= 1.0e6,
            "coarse_dt_years must be within (0, 1e6]",
        );
        check(
            self.fine_iterations.iter().all(|&n| n <= 200),
            "fine_iterations must each be at most 200",
        );
        check(
            self.fine_dt_years > 0.0 && self.fine_dt_years <= 1.0e5,
            "fine_dt_years must be within (0, 1e5]",
        );
        check(
            (0.0..=2.0).contains(&self.detail_amplitude),
            "detail_amplitude must be within 0–2",
        );
        check(
            (-200.0..=200.0).contains(&self.sea_level_m),
            "sea_level_m must be within -200–200",
        );
        check(
            (0.0..=10_000.0).contains(&self.precipitation_mm_per_yr),
            "precipitation_mm_per_yr must be within 0–10000",
        );
        check(
            (0.0..=10_000.0).contains(&self.evapotranspiration_mm_per_yr),
            "evapotranspiration_mm_per_yr must be within 0–10000",
        );
        check(
            (0.0..=10_000.0).contains(&self.lake_evaporation_mm_per_yr),
            "lake_evaporation_mm_per_yr must be within 0–10000",
        );
        check(
            self.channel_area_km2 > 0.0 && self.channel_area_km2 <= 1000.0,
            "channel_area_km2 must be within (0, 1000]",
        );
        check(
            (0.0..=100.0).contains(&self.min_lake_depth_m),
            "min_lake_depth_m must be within 0–100",
        );
        check(
            (0.0..=1.0e8).contains(&self.min_lake_area_m2),
            "min_lake_area_m2 must be within 0–1e8",
        );
        check(
            self.channel_width_coefficient > 0.0 && self.channel_width_coefficient <= 50.0,
            "channel_width_coefficient must be within (0, 50]",
        );
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid() {
        TerrainParams::default().validate().expect("valid defaults");
    }

    #[test]
    fn out_of_range_values_are_named() {
        let p = TerrainParams {
            outlet_edges: 3,
            erodibility: f64::NAN,
            ..TerrainParams::default()
        };
        let err = p.validate().expect_err("invalid");
        assert!(err.contains("outlet_edges"));
        assert!(err.contains("finite"));
    }

    #[test]
    fn runoff_is_precipitation_minus_evapotranspiration() {
        let p = TerrainParams::default();
        assert!((p.runoff_m_per_yr() - 0.3).abs() < 1e-12);
    }
}
