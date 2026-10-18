//! Vehicles (`kind = "vehicle"`, content API 132, M8b slice CO; ADR-0029 §2): a made thing that
//! carries more than a person can, with a physical spec (payload, draft, beam, the speed its crew
//! gives it), what making one takes, and how long it lasts. Using one needs no technique; making
//! one does. M8's only mode here is the boat.

use civ_agents::params::{VehicleDef, VehicleMode};
use serde::Deserialize;

/// The `kind` value of a vehicle.
pub const KIND: &str = "vehicle";
/// The id segment: `pack:vehicle/name`.
pub const ID_KIND: &str = "vehicle";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct VehicleFile {
    pub kind: String,
    pub id: String,
    pub name: String,
    /// `boat`.
    pub mode: String,
    /// The technique its makers need; empty for none.
    pub technique: String,
    /// The skill its makers practise; empty for none.
    pub skill: String,
    /// A tool its makers need, a good with `[tool]`; empty for none.
    pub tool: String,
    /// The good it is made from: one with `[timber]` strengths for a hull cut from a trunk.
    pub good: String,
    /// Kilograms of that good one takes: the trunk felled for a dugout.
    pub material_kg: f64,
    /// Hours of work to make one.
    pub labour_h: f64,
    /// How many may work on one at once.
    pub crew: u32,
    /// What it carries besides its crew, kilograms.
    pub payload_kg: f64,
    /// Water it needs under it loaded, metres.
    pub draft_m: f64,
    /// Its paddlers' speed through still water, metres a second.
    pub paddle_m_s: f64,
    /// Its beam, metres.
    pub width_m: f64,
    /// Years it lasts before it is past use.
    pub life_years: f64,
    /// Kilograms a person loads or unloads a minute (content API 133).
    pub handling_kg_min: f64,
    /// The share of a reach's mean velocity it meets going upstream along the banks (content API
    /// 133).
    pub slack_share: f64,
}

impl VehicleFile {
    /// The mode named, if known.
    pub fn mode(&self) -> Option<VehicleMode> {
        VehicleMode::from_name(&self.mode)
    }

    /// The compiled vehicle, its references resolved to indexes (call after
    /// [`VehicleFile::problems`] found none and all resolved).
    pub fn def(
        &self,
        mode: VehicleMode,
        good: usize,
        technique: Option<usize>,
        skill: Option<usize>,
        tool: Option<usize>,
    ) -> VehicleDef {
        VehicleDef {
            id: self.id.clone(),
            name: self.name.clone(),
            mode,
            technique,
            skill,
            tool,
            good,
            material_kg: self.material_kg,
            labour_h: self.labour_h,
            crew: self.crew,
            payload_kg: self.payload_kg,
            draft_m: self.draft_m,
            paddle_m_s: self.paddle_m_s,
            width_m: self.width_m,
            life_years: self.life_years,
            handling_kg_min: self.handling_kg_min,
            slack_share: self.slack_share,
        }
    }

    /// Range problems, as messages.
    pub fn problems(&self) -> Vec<String> {
        let mut p = Vec::new();
        if self.mode().is_none() {
            p.push(format!("`mode` must be `boat` (got `{}`)", self.mode));
        }
        for (name, v, lo, hi) in [
            ("material_kg", self.material_kg, 1.0, 100_000.0),
            ("labour_h", self.labour_h, 1.0, 100_000.0),
            ("crew", f64::from(self.crew), 1.0, 100.0),
            ("payload_kg", self.payload_kg, 1.0, 1_000_000.0),
            ("draft_m", self.draft_m, 0.01, 20.0),
            ("paddle_m_s", self.paddle_m_s, 0.1, 10.0),
            ("width_m", self.width_m, 0.1, 50.0),
            ("life_years", self.life_years, 0.1, 500.0),
            ("handling_kg_min", self.handling_kg_min, 0.1, 1000.0),
            ("slack_share", self.slack_share, 0.0, 1.0),
        ] {
            if !(v.is_finite() && (lo..=hi).contains(&v)) {
                p.push(format!("`{name}` must be between {lo} and {hi} (got {v})"));
            }
        }
        p
    }
}
