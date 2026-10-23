//! Vehicles (`kind = "vehicle"`, content API 132, M8b slice CO; ADR-0029 §2): a made thing that
//! carries more than a person can, with a physical spec (payload, draft, beam, the speed its crew
//! gives it), what making one takes, and how long it lasts. Using one needs no technique; making
//! one does. M8's modes are the boat and the handcart (`cart`, content API 135, M8b slice CP):
//! a boat's water and paddling fields are a boat's only, a cart's mass, resistances and puller
//! a cart's only.

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
    /// `boat` or `cart`.
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
    /// Water it needs under it loaded, metres (a boat's).
    #[serde(default)]
    pub draft_m: f64,
    /// Its paddlers' speed through still water, metres a second (a boat's).
    #[serde(default)]
    pub paddle_m_s: f64,
    /// Its beam, metres.
    pub width_m: f64,
    /// Years it lasts before it is past use.
    pub life_years: f64,
    /// Kilograms a person loads or unloads a minute (content API 133).
    pub handling_kg_min: f64,
    /// The share of a reach's mean velocity it meets going upstream along the banks (content API
    /// 133; a boat's).
    #[serde(default)]
    pub slack_share: f64,
    /// Its own mass, kilograms (content API 135; a cart's).
    #[serde(default)]
    pub tare_kg: f64,
    /// Rolling resistance on a way that serves carts, and on a worn trail (a cart's).
    #[serde(default)]
    pub way_resistance: f64,
    #[serde(default)]
    pub trail_resistance: f64,
    /// The most a person pulling it can pull, newtons, and their sustained power, watts (a
    /// cart's).
    #[serde(default)]
    pub pull_n: f64,
    #[serde(default)]
    pub power_w: f64,
    /// The steepest grade down, rise over run, its puller can hold it back on (a cart's or a
    /// wagon's).
    #[serde(default)]
    pub brake_grade: f64,
    /// A wagon's rolling resistance on its track, and the gauge it runs on, metres (content API
    /// 168, M10b slice DV).
    #[serde(default)]
    pub rail_resistance: f64,
    #[serde(default)]
    pub gauge_m: f64,
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
            tare_kg: self.tare_kg,
            way_resistance: self.way_resistance,
            trail_resistance: self.trail_resistance,
            pull_n: self.pull_n,
            power_w: self.power_w,
            brake_grade: self.brake_grade,
            rail_resistance: self.rail_resistance,
            gauge_m: self.gauge_m,
        }
    }

    /// Range problems, as messages.
    pub fn problems(&self) -> Vec<String> {
        let mut p = Vec::new();
        let mode = self.mode();
        if mode.is_none() {
            p.push(format!(
                "`mode` must be `boat`, `cart` or `rail` (got `{}`)",
                self.mode
            ));
        }
        let mut ranges = vec![
            ("material_kg", self.material_kg, 1.0, 100_000.0),
            ("labour_h", self.labour_h, 1.0, 100_000.0),
            ("crew", f64::from(self.crew), 1.0, 100.0),
            ("payload_kg", self.payload_kg, 1.0, 1_000_000.0),
            ("width_m", self.width_m, 0.1, 50.0),
            ("life_years", self.life_years, 0.1, 500.0),
            ("handling_kg_min", self.handling_kg_min, 0.1, 1000.0),
        ];
        let boat = [
            ("draft_m", self.draft_m, 0.01, 20.0),
            ("paddle_m_s", self.paddle_m_s, 0.1, 10.0),
            ("slack_share", self.slack_share, 0.0, 1.0),
        ];
        let cart = [
            ("tare_kg", self.tare_kg, 1.0, 10_000.0),
            ("way_resistance", self.way_resistance, 0.001, 0.5),
            ("trail_resistance", self.trail_resistance, 0.001, 0.5),
            ("pull_n", self.pull_n, 1.0, 100_000.0),
            ("power_w", self.power_w, 1.0, 100_000.0),
            ("brake_grade", self.brake_grade, 0.0, 1.0),
        ];
        let rail = [
            ("tare_kg", self.tare_kg, 1.0, 100_000.0),
            ("rail_resistance", self.rail_resistance, 0.0005, 0.1),
            ("gauge_m", self.gauge_m, 0.3, 3.0),
            ("pull_n", self.pull_n, 1.0, 100_000.0),
            ("power_w", self.power_w, 1.0, 100_000.0),
            ("brake_grade", self.brake_grade, 0.0, 1.0),
        ];
        // Each mode's own fields are required of it, and other modes' must be left out (a
        // wagon shares a cart's puller and mass).
        let boats = [
            ("draft_m", self.draft_m, "a boat"),
            ("paddle_m_s", self.paddle_m_s, "a boat"),
            ("slack_share", self.slack_share, "a boat"),
        ];
        let roads = [
            ("way_resistance", self.way_resistance, "a cart"),
            ("trail_resistance", self.trail_resistance, "a cart"),
        ];
        let wheels = [
            ("tare_kg", self.tare_kg, "a cart"),
            ("pull_n", self.pull_n, "a cart"),
            ("power_w", self.power_w, "a cart"),
            ("brake_grade", self.brake_grade, "a cart"),
        ];
        let rails = [
            ("rail_resistance", self.rail_resistance, "a wagon"),
            ("gauge_m", self.gauge_m, "a wagon"),
        ];
        let (own, others): (&[_], Vec<(&str, f64, &str)>) = match mode {
            Some(VehicleMode::Cart) => (&cart, [&boats[..], &rails[..]].concat()),
            Some(VehicleMode::Rail) => (&rail, [&boats[..], &roads[..]].concat()),
            _ => (&boat, [&roads[..], &wheels[..], &rails[..]].concat()),
        };
        ranges.extend_from_slice(own);
        for (field, v, name) in others {
            if v != 0.0 {
                p.push(format!("`{field}` is {name}'s, not this vehicle's"));
            }
        }
        for (field, v, lo, hi) in ranges {
            if !(v.is_finite() && (lo..=hi).contains(&v)) {
                p.push(format!("`{field}` must be between {lo} and {hi} (got {v})"));
            }
        }
        p
    }
}
