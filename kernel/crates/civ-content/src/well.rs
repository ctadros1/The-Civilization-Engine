//! Well systems (`kind = "well"`, content API 71, M6a slice AY, step three; ADR-0021 §3;
//! research 03-02 §1.4, 12-01 §2.2): the shaft dug and how it is lined, the work a metre of it
//! takes beyond the digging, how deep its diggers will go, and what rots its lining. What a well
//! yields is worked out from its own depth and the ground's water, never authored.

use civ_agents::params::WellDef;
use serde::Deserialize;

/// The `kind` value of a well system.
pub const KIND: &str = "well";
/// The id segment: `pack:well/name`.
pub const ID_KIND: &str = "well";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WellFile {
    pub kind: String,
    pub id: String,
    pub name: String,
    /// The technique its diggers need; empty for none.
    pub technique: String,
    /// The skill its diggers practise, from which its lining's quality is drawn; empty for none.
    pub skill: String,
    pub dig_radius_m: f64,
    pub radius_m: f64,
    pub lining_h_per_m: f64,
    pub water_m: f64,
    pub max_depth_m: f64,
    pub influence_m: f64,
    pub lift_min_per_m: f64,
    pub loss_per_year: f64,
    pub crew: u32,
}

impl WellFile {
    /// The compiled system, its technique and skill resolved to indexes.
    pub fn def(&self, technique: Option<usize>, skill: Option<usize>) -> WellDef {
        WellDef {
            id: self.id.clone(),
            name: self.name.clone(),
            technique,
            skill,
            dig_radius_m: self.dig_radius_m,
            radius_m: self.radius_m,
            lining_h_per_m: self.lining_h_per_m,
            water_m: self.water_m,
            max_depth_m: self.max_depth_m,
            influence_m: self.influence_m,
            lift_min_per_m: self.lift_min_per_m,
            loss_per_year: self.loss_per_year,
            crew: self.crew,
        }
    }

    /// Range problems, as messages.
    pub fn problems(&self) -> Vec<String> {
        let mut p = Vec::new();
        for (name, v, lo, hi) in [
            ("dig_radius_m", self.dig_radius_m, 0.2, 5.0),
            ("radius_m", self.radius_m, 0.1, 5.0),
            ("lining_h_per_m", self.lining_h_per_m, 0.0, 1000.0),
            ("water_m", self.water_m, 0.1, 20.0),
            ("max_depth_m", self.max_depth_m, 0.5, 100.0),
            ("influence_m", self.influence_m, 1.0, 10_000.0),
            ("lift_min_per_m", self.lift_min_per_m, 0.0, 10.0),
            ("loss_per_year", self.loss_per_year, 0.0, 1.0),
            ("crew", f64::from(self.crew), 1.0, 20.0),
        ] {
            if !(v.is_finite() && (lo..=hi).contains(&v)) {
                p.push(format!("`{name}` must be between {lo} and {hi} (got {v})"));
            }
        }
        if self.radius_m > self.dig_radius_m {
            p.push("`radius_m` must not be above `dig_radius_m`: the lining is inside".to_owned());
        }
        if self.influence_m <= self.radius_m {
            p.push("`influence_m` must be above `radius_m`".to_owned());
        }
        p
    }
}
