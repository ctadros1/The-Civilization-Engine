//! Bridge systems (`kind = "bridge"`, content API 65, M5c slice AW; research 11-07 §2.1): a span
//! envelope, the members a bridge of it is made of and the work they take, and what wears and
//! fails them. What any one bridge carries is worked out from its own members and condition,
//! never authored (11-07 §1.3).

use civ_agents::params::BridgeDef;
use serde::Deserialize;

/// The `kind` value of a bridge system.
pub const KIND: &str = "bridge";
/// The id segment: `pack:bridge/name`.
pub const ID_KIND: &str = "bridge";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BridgeFile {
    pub kind: String,
    pub id: String,
    pub name: String,
    /// The technique its builders need; empty for none.
    pub technique: String,
    /// The skill its builders practise, from which its members' quality is drawn; empty for none
    /// (content API 66).
    pub skill: String,
    /// The good its members are made of: one with `[timber]` strengths.
    pub good: String,
    pub density_kg_m3: f64,
    pub span_min_m: f64,
    pub span_max_m: f64,
    pub members: u32,
    pub diameter_min_cm: f64,
    pub diameter_max_cm: f64,
    pub bearing_m: f64,
    pub deck_factor: f64,
    pub labour_h_per_m: f64,
    pub loss_per_year: f64,
    pub margin: f64,
    pub fall_kills: f64,
}

impl BridgeFile {
    /// The compiled system, its good, technique and skill resolved to indexes (call after
    /// [`BridgeFile::problems`] found none and both resolved).
    pub fn def(&self, good: usize, technique: Option<usize>, skill: Option<usize>) -> BridgeDef {
        BridgeDef {
            id: self.id.clone(),
            name: self.name.clone(),
            technique,
            skill,
            good,
            density_kg_m3: self.density_kg_m3,
            span_m: (self.span_min_m, self.span_max_m),
            members: self.members,
            diameter_cm: (self.diameter_min_cm, self.diameter_max_cm),
            bearing_m: self.bearing_m,
            deck_factor: self.deck_factor,
            labour_h_per_m: self.labour_h_per_m,
            loss_per_year: self.loss_per_year,
            margin: self.margin,
            fall_kills: self.fall_kills,
        }
    }

    /// Range problems, as messages.
    pub fn problems(&self) -> Vec<String> {
        let mut p = Vec::new();
        for (name, v, lo, hi) in [
            ("density_kg_m3", self.density_kg_m3, 100.0, 3000.0),
            ("span_min_m", self.span_min_m, 0.5, 200.0),
            ("span_max_m", self.span_max_m, 0.5, 200.0),
            ("members", f64::from(self.members), 1.0, 20.0),
            ("diameter_min_cm", self.diameter_min_cm, 2.0, 200.0),
            ("diameter_max_cm", self.diameter_max_cm, 2.0, 200.0),
            ("bearing_m", self.bearing_m, 0.0, 10.0),
            ("deck_factor", self.deck_factor, 0.05, 1.0),
            ("labour_h_per_m", self.labour_h_per_m, 0.1, 10000.0),
            ("loss_per_year", self.loss_per_year, 0.0, 1.0),
            ("margin", self.margin, 1.0, 20.0),
            ("fall_kills", self.fall_kills, 0.0, 1.0),
        ] {
            if !(v.is_finite() && (lo..=hi).contains(&v)) {
                p.push(format!("`{name}` must be between {lo} and {hi} (got {v})"));
            }
        }
        if self.span_min_m > self.span_max_m {
            p.push("`span_min_m` must not be above `span_max_m`".to_owned());
        }
        if self.diameter_min_cm > self.diameter_max_cm {
            p.push("`diameter_min_cm` must not be above `diameter_max_cm`".to_owned());
        }
        p
    }
}
